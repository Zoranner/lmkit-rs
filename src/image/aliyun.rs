//! 阿里云 DashScope 文生图（千问图像等）
//!
//! 使用原生 HTTP：`POST {base_url}/services/aigc/multimodal-generation/generation`，与 OpenAI 的 `images/generations` 不同。
//! `base_url` 一般为 `https://dashscope.aliyuncs.com/api/v1` 或新加坡 `https://dashscope-intl.aliyuncs.com/api/v1`（与「compatible-mode」对话网关不是同一路径）。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{ImageOutput, ImageProvider, ImageSize};
use crate::client::HttpClient;
use crate::config::ProviderConfig;
use crate::error::{Error, Result};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Serialize)]
struct ContentItem {
    text: String,
}

#[derive(Debug, Serialize)]
struct UserMessage {
    role: String,
    content: Vec<ContentItem>,
}

#[derive(Debug, Serialize)]
struct AliyunInput {
    messages: Vec<UserMessage>,
}

#[derive(Debug, Serialize)]
struct AliyunImageParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    negative_prompt: Option<String>,
    prompt_extend: bool,
    watermark: bool,
    size: String,
}

#[derive(Debug, Serialize)]
struct AliyunMultimodalRequest {
    model: String,
    input: AliyunInput,
    parameters: AliyunImageParams,
}

#[derive(Debug, Deserialize)]
struct AliyunMultimodalResponse {
    #[serde(default)]
    code: String,
    #[serde(default)]
    message: String,
    output: Option<AliyunOutput>,
}

#[derive(Debug, Deserialize)]
struct AliyunOutput {
    choices: Vec<AliyunChoice>,
}

#[derive(Debug, Deserialize)]
struct AliyunChoice {
    message: AliyunMessage,
}

#[derive(Debug, Deserialize)]
struct AliyunMessage {
    content: Vec<AliyunContentItem>,
}

#[derive(Debug, Deserialize)]
struct AliyunContentItem {
    image: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AliyunErrorBody {
    code: Option<String>,
    message: Option<String>,
}

fn parse_aliyun_error(body: &str) -> String {
    if let Ok(err) = serde_json::from_str::<AliyunErrorBody>(body) {
        if let (Some(code), Some(message)) = (err.code, err.message) {
            return format!("{code}: {message}");
        }
    }
    body.to_string()
}

pub(crate) struct AliyunQwenImage {
    client: HttpClient,
    api_key: String,
    model: String,
    base_url: String,
}

impl AliyunQwenImage {
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

    /// 百炼文档使用 `宽*高`（半角星号），例如 `1024*1024`、`1792*1024`。
    fn size_param(size: ImageSize) -> &'static str {
        match size {
            ImageSize::Square512 => "512*512",
            ImageSize::Square1024 => "1024*1024",
            ImageSize::Landscape => "1792*1024",
            ImageSize::Portrait => "1024*1792",
        }
    }
}

#[async_trait]
impl ImageProvider for AliyunQwenImage {
    async fn generate(&self, prompt: &str, size: ImageSize) -> Result<ImageOutput> {
        let request = AliyunMultimodalRequest {
            model: self.model.clone(),
            input: AliyunInput {
                messages: vec![UserMessage {
                    role: "user".to_string(),
                    content: vec![ContentItem {
                        text: prompt.to_string(),
                    }],
                }],
            },
            parameters: AliyunImageParams {
                negative_prompt: None,
                prompt_extend: true,
                watermark: false,
                size: Self::size_param(size).to_string(),
            },
        };

        let url = format!(
            "{}/services/aigc/multimodal-generation/generation",
            self.base_url.trim_end_matches('/')
        );

        let resp: AliyunMultimodalResponse = self
            .client
            .post_bearer_json(&url, &self.api_key, &request, |s| parse_aliyun_error(&s))
            .await?;

        if !resp.code.is_empty() {
            let msg = if resp.message.is_empty() {
                resp.code.clone()
            } else {
                format!("{}: {}", resp.code, resp.message)
            };
            return Err(Error::Parse(format!("DashScope image API: {msg}")));
        }

        let output = resp.output.ok_or(Error::MissingField("output"))?;
        let first = output
            .choices
            .into_iter()
            .next()
            .ok_or(Error::MissingField("output.choices[0]"))?;
        let url_str = first
            .message
            .content
            .into_iter()
            .find_map(|c| c.image)
            .ok_or(Error::MissingField(
                "output.choices[0].message.content[].image",
            ))?;

        Ok(ImageOutput::Url(url_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Provider, ProviderConfig};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn generate_returns_image_url() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(
                "/api/v1/services/aigc/multimodal-generation/generation",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "request_id": "r1",
                "output": {
                    "choices": [{
                        "finish_reason": "stop",
                        "message": {
                            "role": "assistant",
                            "content": [{ "image": "https://oss.example.com/out.png" }]
                        }
                    }]
                }
            })))
            .mount(&server)
            .await;

        let cfg = ProviderConfig::with_base_url(
            Provider::Aliyun,
            "sk-test",
            format!("{}/api/v1", server.uri()),
            "qwen-image-plus",
        );
        let gen = AliyunQwenImage::new(&cfg).unwrap();
        let out = gen.generate("一只猫", ImageSize::Square1024).await.unwrap();
        match out {
            ImageOutput::Url(u) => assert_eq!(u, "https://oss.example.com/out.png"),
            ImageOutput::Bytes(_) => panic!("expected URL"),
        }
    }

    #[tokio::test]
    async fn api_error_in_body_returns_err() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(
                "/api/v1/services/aigc/multimodal-generation/generation",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "code": "InvalidParameter",
                "message": "bad size"
            })))
            .mount(&server)
            .await;

        let cfg = ProviderConfig::with_base_url(
            Provider::Aliyun,
            "sk-test",
            format!("{}/api/v1", server.uri()),
            "qwen-image-plus",
        );
        let gen = AliyunQwenImage::new(&cfg).unwrap();
        let err = gen.generate("x", ImageSize::Square1024).await.unwrap_err();
        match err {
            Error::Parse(s) => assert!(s.contains("InvalidParameter"), "{s}"),
            e => panic!("expected Parse error, got {e:?}"),
        }
    }
}
