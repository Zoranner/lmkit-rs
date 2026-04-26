//! 智谱 Rerank

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::client::HttpClient;
use crate::config::ProviderConfig;
use crate::error::Result;
use crate::rerank::{RerankItem, RerankProvider};

#[derive(Debug, Serialize)]
struct ZhipuRerankRequest {
    model: String,
    query: String,
    documents: Vec<String>,
    top_n: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ZhipuRerankResponse {
    results: Vec<ZhipuRerankResult>,
}

#[derive(Debug, Deserialize)]
struct ZhipuRerankResult {
    index: usize,
    relevance_score: f64,
}

pub(crate) struct ZhipuRerank {
    client: HttpClient,
    api_key: String,
    model: String,
    base_url: String,
}

impl ZhipuRerank {
    pub fn new(config: &ProviderConfig, client: HttpClient) -> Self {
        tracing::warn!("ZhipuRerank: 已知部分环境下分数接近常数，必要时可改用阿里云 Rerank");
        Self {
            client,
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            base_url: config.base_url.clone(),
        }
    }
}

#[async_trait]
impl RerankProvider for ZhipuRerank {
    async fn rerank(
        &self,
        query: &str,
        documents: &[&str],
        top_n: Option<usize>,
    ) -> Result<Vec<RerankItem>> {
        let request = ZhipuRerankRequest {
            model: self.model.clone(),
            query: query.to_string(),
            documents: documents.iter().map(|s| s.to_string()).collect(),
            top_n,
        };

        let url = format!("{}/rerank", self.base_url.trim_end_matches('/'));

        let rerank_response: ZhipuRerankResponse = self
            .client
            .post_bearer_json(&url, &self.api_key, &request, |s| s)
            .await?;

        Ok(rerank_response
            .results
            .into_iter()
            .map(|r| RerankItem {
                index: r.index,
                score: r.relevance_score,
            })
            .collect())
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
        HttpClient::new(Duration::from_secs(60)).unwrap()
    }

    fn test_config(server: &MockServer) -> ProviderConfig {
        ProviderConfig::with_base_url(
            Provider::Zhipu,
            "zk",
            server.uri().to_string(),
            "rerank-model",
        )
    }

    #[tokio::test]
    async fn rerank_success_uses_rerank_path() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rerank"))
            .and(header("Authorization", "Bearer zk"))
            .and(body_json(serde_json::json!({
                "model": "rerank-model",
                "query": "q",
                "documents": ["a", "b"],
                "top_n": null,
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{ "index": 0, "relevance_score": 0.88 }]
            })))
            .mount(&server)
            .await;

        let cfg = test_config(&server);
        let r = ZhipuRerank::new(&cfg, http_client());
        let items = r.rerank("q", &["a", "b"], None).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].index, 0);
        assert!((items[0].score - 0.88).abs() < 1e-9);
    }

    #[tokio::test]
    async fn rerank_non_success_maps_to_api() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rerank"))
            .respond_with(ResponseTemplate::new(403).set_body_string("forbidden"))
            .mount(&server)
            .await;

        let cfg = test_config(&server);
        let r = ZhipuRerank::new(&cfg, http_client());
        let err = r.rerank("q", &["a"], None).await.unwrap_err();
        match err {
            Error::Api { status, message } => {
                assert_eq!(status, 403);
                assert_eq!(message, "forbidden");
            }
            other => panic!("expected Api, got {:?}", other),
        }
    }
}
