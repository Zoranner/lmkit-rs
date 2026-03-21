//! 智谱 Embedding（请求体不含 `dimensions`）

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::EmbedProvider;
use crate::client::HttpClient;
use crate::config::ProviderConfig;
use crate::error::Result;
use crate::util::normalize_for_embedding;

#[derive(Debug, Serialize)]
struct ZhipuEmbedRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ZhipuEmbedResponse {
    data: Vec<ZhipuEmbedData>,
}

#[derive(Debug, Deserialize)]
struct ZhipuEmbedData {
    embedding: Vec<f32>,
}

pub(crate) struct ZhipuEmbed {
    client: HttpClient,
    api_key: String,
    model: String,
    base_url: String,
    dimension: usize,
}

impl ZhipuEmbed {
    pub fn new(config: &ProviderConfig, dimension: usize, client: HttpClient) -> Self {
        tracing::info!(
            "ZhipuEmbed: model={}, dimension={}, base_url={}",
            config.model,
            dimension,
            config.base_url
        );
        Self {
            client,
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            base_url: config.base_url.clone(),
            dimension,
        }
    }
}

#[async_trait]
impl EmbedProvider for ZhipuEmbed {
    async fn encode(&self, text: &str) -> Result<Vec<f32>> {
        let normalized = normalize_for_embedding(text);
        let embeddings = self.encode_batch(&[&normalized]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| crate::error::Error::MissingField("embeddings[0]"))
    }

    async fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let normalized: Vec<String> = texts.iter().map(|t| normalize_for_embedding(t)).collect();

        let request = ZhipuEmbedRequest {
            model: self.model.clone(),
            input: normalized,
        };

        let url = format!("{}/embeddings", self.base_url.trim_end_matches('/'));

        let embed_response: ZhipuEmbedResponse = self
            .client
            .post_bearer_json(&url, &self.api_key, &request, |s| s)
            .await?;

        Ok(embed_response
            .data
            .into_iter()
            .map(|d| d.embedding)
            .collect())
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

#[cfg(all(test, feature = "zhipu"))]
mod tests {
    use super::*;
    use crate::config::{Provider, ProviderConfig};
    use crate::error::Error;
    use std::time::Duration;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn http_client() -> HttpClient {
        HttpClient::new(Duration::from_secs(30)).unwrap()
    }

    fn test_config(server: &MockServer) -> ProviderConfig {
        let mut cfg = ProviderConfig::new(
            Provider::Zhipu,
            "zk",
            server.uri().to_string(),
            "embedding-3",
        );
        cfg.dimension = Some(2);
        cfg
    }

    #[tokio::test]
    async fn zhipu_request_has_no_dimensions_field() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .and(header("Authorization", "Bearer zk"))
            .and(body_json(serde_json::json!({
                "model": "embedding-3",
                "input": ["hello"],
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{ "embedding": [0.5, -0.5] }]
            })))
            .mount(&server)
            .await;

        let cfg = test_config(&server);
        let dim = cfg.dimension.unwrap();
        let emb = ZhipuEmbed::new(&cfg, dim, http_client());
        let v = emb.encode("hello").await.unwrap();
        assert_eq!(v, vec![0.5f32, -0.5]);
    }

    #[tokio::test]
    async fn zhipu_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(400).set_body_string("bad"))
            .mount(&server)
            .await;

        let cfg = test_config(&server);
        let dim = cfg.dimension.unwrap();
        let emb = ZhipuEmbed::new(&cfg, dim, http_client());
        let err = emb.encode("x").await.unwrap_err();
        match err {
            Error::Api { status, message } => {
                assert_eq!(status, 400);
                assert_eq!(message, "bad");
            }
            other => panic!("expected Api, got {:?}", other),
        }
    }
}
