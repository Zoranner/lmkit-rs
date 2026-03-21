//! OpenAI 兼容对话：`POST …/chat/completions`。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::client::HttpClient;
use crate::config::ProviderConfig;
use crate::error::{Error, Result};

use super::ChatProvider;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Provider;
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
