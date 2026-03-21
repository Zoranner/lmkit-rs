//! OpenAI 兼容 Embeddings（阿里云 / OpenAI / Ollama）

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::EmbedProvider;
use crate::client::HttpClient;
use crate::config::ProviderConfig;
use crate::error::Result;
use crate::util::normalize_for_embedding;

#[derive(Debug, Serialize)]
struct OpenaiEmbedRequest {
    model: String,
    input: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct OpenaiEmbedResponse {
    data: Vec<OpenaiEmbedData>,
}

#[derive(Debug, Deserialize)]
struct OpenaiEmbedData {
    embedding: Vec<f32>,
}

pub(crate) struct OpenaiCompatEmbed {
    client: HttpClient,
    api_key: String,
    model: String,
    base_url: String,
    dimension: usize,
}

impl OpenaiCompatEmbed {
    pub fn new(config: &ProviderConfig, dimension: usize, client: HttpClient) -> Self {
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
impl EmbedProvider for OpenaiCompatEmbed {
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

        let request = OpenaiEmbedRequest {
            model: self.model.clone(),
            input: normalized,
            dimensions: Some(self.dimension),
        };

        let url = format!("{}/embeddings", self.base_url.trim_end_matches('/'));

        let embed_response: OpenaiEmbedResponse = self
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

#[cfg(test)]
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
            Provider::OpenAI,
            "ek",
            server.uri().to_string(),
            "text-embedding-3-small",
        );
        cfg.dimension = Some(2);
        cfg
    }

    #[tokio::test]
    async fn encode_batch_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .and(header("Authorization", "Bearer ek"))
            .and(body_json(serde_json::json!({
                "model": "text-embedding-3-small",
                "input": ["a b", "c"],
                "dimensions": 2,
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    { "embedding": [0.1, 0.2] },
                    { "embedding": [0.3, 0.4] }
                ]
            })))
            .mount(&server)
            .await;

        let cfg = test_config(&server);
        let dim = cfg.dimension.unwrap();
        let emb = OpenaiCompatEmbed::new(&cfg, dim, http_client());
        let out = emb.encode_batch(&["  a \n b", "c"]).await.unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], vec![0.1f32, 0.2]);
        assert_eq!(out[1], vec![0.3f32, 0.4]);
        assert_eq!(emb.dimension(), 2);
    }

    #[tokio::test]
    async fn encode_single_delegates_to_batch() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{ "embedding": [1.0, 0.0] }]
            })))
            .mount(&server)
            .await;

        let cfg = test_config(&server);
        let dim = cfg.dimension.unwrap();
        let emb = OpenaiCompatEmbed::new(&cfg, dim, http_client());
        let v = emb.encode("hello").await.unwrap();
        assert_eq!(v, vec![1.0f32, 0.0]);
    }

    #[tokio::test]
    async fn encode_empty_data_yields_missing_field() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": [] })))
            .mount(&server)
            .await;

        let cfg = test_config(&server);
        let dim = cfg.dimension.unwrap();
        let emb = OpenaiCompatEmbed::new(&cfg, dim, http_client());
        let err = emb.encode("x").await.unwrap_err();
        match err {
            Error::MissingField(name) => assert_eq!(name, "embeddings[0]"),
            other => panic!("expected MissingField, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn embeddings_non_success_maps_to_api() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(429).set_body_string("slow down"))
            .mount(&server)
            .await;

        let cfg = test_config(&server);
        let dim = cfg.dimension.unwrap();
        let emb = OpenaiCompatEmbed::new(&cfg, dim, http_client());
        let err = emb.encode_batch(&["a"]).await.unwrap_err();
        match err {
            Error::Api { status, message } => {
                assert_eq!(status, 429);
                assert_eq!(message, "slow down");
            }
            other => panic!("expected Api, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn embeddings_success_body_not_json_yields_parse() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&server)
            .await;

        let cfg = test_config(&server);
        let dim = cfg.dimension.unwrap();
        let emb = OpenaiCompatEmbed::new(&cfg, dim, http_client());
        let err = emb.encode_batch(&["a"]).await.unwrap_err();
        match err {
            Error::Parse(_) => {}
            other => panic!("expected Parse, got {:?}", other),
        }
    }
}
