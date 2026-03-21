//! 对话补全：OpenAI 兼容 `POST …/chat/completions`，**非流式** JSON。
//!
//! # [`ChatProvider::chat`]
//!
//! `prompt` 会作为**唯一一条** `role: user` 消息发送，不包含 system / 多轮 history。若需要多轮或 system prompt，须另行扩展 API 或直接使用上游 HTTP。
//!
//! 请求体中 **`temperature` 固定为 `0.2`**，调用方无法通过本 trait 覆盖。
//!
//! # URL 与鉴权
//!
//! 请求地址为 `{base_url}/chat/completions`，其中 `base_url` 来自 [`ProviderConfig`]，会先对 `base_url` 做 `trim_end_matches('/')` 再拼接路径段。
//!
//! 鉴权为 `Authorization: Bearer {api_key}`，与其它模态相同，均为 JSON POST。**空字符串密钥仍会原样放入请求头**；网关是否接受由上游决定（例如部分本地 Ollama 部署不校验 Bearer）。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::client::HttpClient;
use crate::config::Provider;
use crate::config::ProviderConfig;
use crate::error::{Error, Result};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

#[async_trait]
pub trait ChatProvider: Send + Sync {
    /// 单轮用户消息补全；语义与模块级文档一致。
    async fn chat(&self, prompt: &str) -> Result<String>;
}

#[derive(Debug, Serialize)]
struct OpenaiChatRequest {
    model: String,
    messages: Vec<OpenaiChatMessage>,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenaiChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenaiChatResponse {
    choices: Vec<OpenaiChatChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenaiChatChoice {
    message: OpenaiChatMessage,
}

pub(crate) struct OpenaiCompatChat {
    client: HttpClient,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenaiCompatChat {
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        let timeout = config.timeout.unwrap_or(DEFAULT_TIMEOUT);
        let client = HttpClient::new(timeout)?;
        Ok(Self {
            client,
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            base_url: config.base_url.clone(),
        })
    }
}

#[async_trait]
impl ChatProvider for OpenaiCompatChat {
    async fn chat(&self, prompt: &str) -> Result<String> {
        let request = OpenaiChatRequest {
            model: self.model.clone(),
            messages: vec![OpenaiChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.2,
        };

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let chat_response: OpenaiChatResponse = self
            .client
            .post_bearer_json(&url, &self.api_key, &request, |s| s)
            .await?;

        chat_response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or(Error::MissingField("choices[0].message"))
    }
}

pub(crate) fn create(config: &ProviderConfig) -> Result<Box<dyn ChatProvider>> {
    match config.provider {
        #[cfg(feature = "openai")]
        Provider::OpenAI => Ok(Box::new(OpenaiCompatChat::new(config)?)),
        #[cfg(not(feature = "openai"))]
        Provider::OpenAI => Err(Error::ProviderDisabled("openai".to_string())),

        #[cfg(feature = "aliyun")]
        Provider::Aliyun => Ok(Box::new(OpenaiCompatChat::new(config)?)),
        #[cfg(not(feature = "aliyun"))]
        Provider::Aliyun => Err(Error::ProviderDisabled("aliyun".to_string())),

        #[cfg(feature = "ollama")]
        Provider::Ollama => Ok(Box::new(OpenaiCompatChat::new(config)?)),
        #[cfg(not(feature = "ollama"))]
        Provider::Ollama => Err(Error::ProviderDisabled("ollama".to_string())),

        #[cfg(feature = "zhipu")]
        Provider::Zhipu => Ok(Box::new(OpenaiCompatChat::new(config)?)),
        #[cfg(not(feature = "zhipu"))]
        Provider::Zhipu => Err(Error::ProviderDisabled("zhipu".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_config(server: &MockServer) -> ProviderConfig {
        ProviderConfig::new(
            Provider::OpenAI,
            "test-key",
            server.uri().to_string(),
            "gpt-4o-mini",
        )
    }

    #[tokio::test]
    async fn chat_success_returns_assistant_content() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("Authorization", "Bearer test-key"))
            .and(body_json(serde_json::json!({
                "model": "gpt-4o-mini",
                "messages": [{ "role": "user", "content": "hello" }],
                "temperature": 0.2,
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{
                    "message": { "role": "assistant", "content": "hi there" }
                }]
            })))
            .mount(&server)
            .await;

        let chat = OpenaiCompatChat::new(&test_config(&server)).unwrap();
        let reply = chat.chat("hello").await.unwrap();
        assert_eq!(reply, "hi there");
    }

    #[tokio::test]
    async fn chat_base_url_trailing_slash_normalized() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{
                    "message": { "role": "assistant", "content": "ok" }
                }]
            })))
            .mount(&server)
            .await;

        let mut cfg = test_config(&server);
        cfg.base_url = format!("{}/", server.uri());
        let chat = OpenaiCompatChat::new(&cfg).unwrap();
        assert_eq!(chat.chat("x").await.unwrap(), "ok");
    }

    #[tokio::test]
    async fn chat_empty_choices_yields_missing_field() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": []
            })))
            .mount(&server)
            .await;

        let chat = OpenaiCompatChat::new(&test_config(&server)).unwrap();
        let err = chat.chat("x").await.unwrap_err();
        match err {
            Error::MissingField(name) => assert_eq!(name, "choices[0].message"),
            other => panic!("expected MissingField, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn chat_non_success_maps_to_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(401).set_body_string("invalid key"))
            .mount(&server)
            .await;

        let chat = OpenaiCompatChat::new(&test_config(&server)).unwrap();
        let err = chat.chat("x").await.unwrap_err();
        match err {
            Error::Api { status, message } => {
                assert_eq!(status, 401);
                assert_eq!(message, "invalid key");
            }
            other => panic!("expected Api, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn chat_success_body_not_json_yields_parse() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&server)
            .await;

        let chat = OpenaiCompatChat::new(&test_config(&server)).unwrap();
        let err = chat.chat("x").await.unwrap_err();
        match err {
            Error::Parse(_) => {}
            other => panic!("expected Parse, got {:?}", other),
        }
    }
}
