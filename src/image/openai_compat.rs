//! OpenAI 兼容 `POST .../images/generations`（亦适用于提供同路径的兼容网关）

use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{ImageOutput, ImageProvider, ImageSize};
use crate::client::HttpClient;
use crate::config::ProviderConfig;
use crate::error::{Error, Result};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Serialize)]
struct OpenaiImageRequest {
    model: String,
    prompt: String,
    n: u32,
    size: String,
}

#[derive(Debug, Deserialize)]
struct OpenaiImageResponse {
    data: Vec<OpenaiImageData>,
}

#[derive(Debug, Deserialize)]
struct OpenaiImageData {
    url: Option<String>,
    b64_json: Option<String>,
}

pub(crate) struct OpenaiCompatImage {
    client: HttpClient,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenaiCompatImage {
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

    /// 与 DALL·E 2/3 常见 `size` 字符串对齐；其它模型（如 GPT Image）若不接受这些值，需换用兼容的 [`ProviderConfig::model`] 或后续扩展映射。
    fn size_param(size: ImageSize) -> &'static str {
        match size {
            ImageSize::Square512 => "512x512",
            ImageSize::Square1024 => "1024x1024",
            ImageSize::Landscape => "1792x1024",
            ImageSize::Portrait => "1024x1792",
        }
    }
}

#[async_trait]
impl ImageProvider for OpenaiCompatImage {
    async fn generate(&self, prompt: &str, size: ImageSize) -> Result<ImageOutput> {
        let request = OpenaiImageRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            n: 1,
            size: Self::size_param(size).to_string(),
        };

        let url = format!("{}/images/generations", self.base_url.trim_end_matches('/'));

        let resp: OpenaiImageResponse = self
            .client
            .post_bearer_json(&url, &self.api_key, &request, |s| s)
            .await?;

        let first = resp
            .data
            .into_iter()
            .next()
            .ok_or(Error::MissingField("data[0]"))?;

        if let Some(u) = first.url {
            return Ok(ImageOutput::Url(u));
        }

        if let Some(b64) = first.b64_json {
            let bytes = B64
                .decode(b64.trim())
                .map_err(|e| Error::Parse(format!("invalid base64 in b64_json: {e}")))?;
            return Ok(ImageOutput::Bytes(bytes));
        }

        Err(Error::MissingField("data[0].url|b64_json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Provider, ProviderConfig};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn generate_returns_url() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/images/generations"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "created": 1,
                "data": [{ "url": "https://example.com/x.png" }]
            })))
            .mount(&server)
            .await;

        let cfg = ProviderConfig::with_base_url(
            Provider::OpenAI,
            "test-key",
            format!("{}/v1", server.uri()),
            "dall-e-3",
        );
        let gen = OpenaiCompatImage::new(&cfg).unwrap();
        let out = gen
            .generate("a red circle", ImageSize::Square1024)
            .await
            .unwrap();
        match out {
            ImageOutput::Url(u) => assert_eq!(u, "https://example.com/x.png"),
            ImageOutput::Bytes(_) => panic!("expected URL"),
        }
    }

    #[tokio::test]
    async fn generate_returns_bytes_from_b64() {
        let server = MockServer::start().await;
        let raw = b"hello";
        let b64 = B64.encode(raw);
        Mock::given(method("POST"))
            .and(path("/v1/images/generations"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "created": 1,
                "data": [{ "b64_json": b64 }]
            })))
            .mount(&server)
            .await;

        let cfg = ProviderConfig::with_base_url(
            Provider::OpenAI,
            "test-key",
            format!("{}/v1", server.uri()),
            "gpt-image-1",
        );
        let gen = OpenaiCompatImage::new(&cfg).unwrap();
        let out = gen.generate("x", ImageSize::Square1024).await.unwrap();
        match out {
            ImageOutput::Bytes(b) => assert_eq!(b, raw.as_slice()),
            ImageOutput::Url(_) => panic!("expected bytes"),
        }
    }
}
