//! 阿里云 DashScope Rerank

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::client::HttpClient;
use crate::config::ProviderConfig;
use crate::error::Result;
use crate::rerank::{RerankItem, RerankProvider};

#[derive(Debug, Serialize)]
struct AliyunRerankRequest {
    model: String,
    query: String,
    documents: Vec<String>,
    top_n: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct AliyunRerankResponse {
    results: Vec<AliyunRerankResult>,
}

#[derive(Debug, Deserialize)]
struct AliyunRerankResult {
    index: usize,
    relevance_score: f64,
}

#[derive(Debug, Deserialize)]
struct AliyunErrorResponse {
    code: Option<String>,
    message: Option<String>,
}

fn parse_aliyun_error(body: &str) -> String {
    if let Ok(err) = serde_json::from_str::<AliyunErrorResponse>(body) {
        if let (Some(code), Some(message)) = (err.code, err.message) {
            return format!("{}: {}", code, message);
        }
    }
    body.to_string()
}

pub(crate) struct AliyunRerank {
    client: HttpClient,
    api_key: String,
    model: String,
    base_url: String,
}

impl AliyunRerank {
    pub fn new(config: &ProviderConfig, client: HttpClient) -> Self {
        Self {
            client,
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            base_url: config.base_url.clone(),
        }
    }
}

#[async_trait]
impl RerankProvider for AliyunRerank {
    async fn rerank(
        &self,
        query: &str,
        documents: &[&str],
        top_n: Option<usize>,
    ) -> Result<Vec<RerankItem>> {
        let request = AliyunRerankRequest {
            model: self.model.clone(),
            query: query.to_string(),
            documents: documents.iter().map(|s| s.to_string()).collect(),
            top_n,
        };

        let url = format!("{}/reranks", self.base_url.trim_end_matches('/'));

        let rerank_response: AliyunRerankResponse = self
            .client
            .post_bearer_json(&url, &self.api_key, &request, |s| parse_aliyun_error(&s))
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

#[cfg(all(test, feature = "aliyun"))]
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
        ProviderConfig::new(
            Provider::Aliyun,
            "ak",
            server.uri().to_string(),
            "gte-rerank",
        )
    }

    #[tokio::test]
    async fn rerank_success_uses_reranks_path() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/reranks"))
            .and(header("Authorization", "Bearer ak"))
            .and(body_json(serde_json::json!({
                "model": "gte-rerank",
                "query": "q",
                "documents": ["a", "b"],
                "top_n": 1,
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    { "index": 1, "relevance_score": 0.9 },
                    { "index": 0, "relevance_score": 0.1 }
                ]
            })))
            .mount(&server)
            .await;

        let cfg = test_config(&server);
        let r = AliyunRerank::new(&cfg, http_client());
        let items = r.rerank("q", &["a", "b"], Some(1)).await.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].index, 1);
        assert!((items[0].score - 0.9).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn rerank_top_n_omitted_when_none() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/reranks"))
            .and(body_json(serde_json::json!({
                "model": "gte-rerank",
                "query": "x",
                "documents": ["d"],
                "top_n": null,
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{ "index": 0, "relevance_score": 1.0 }]
            })))
            .mount(&server)
            .await;

        let cfg = test_config(&server);
        let r = AliyunRerank::new(&cfg, http_client());
        let items = r.rerank("x", &["d"], None).await.unwrap();
        assert_eq!(items.len(), 1);
    }

    #[tokio::test]
    async fn rerank_api_error_maps_aliyun_json_body() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/reranks"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "code": "InvalidParameter",
                "message": "bad doc"
            })))
            .mount(&server)
            .await;

        let cfg = test_config(&server);
        let r = AliyunRerank::new(&cfg, http_client());
        let err = r.rerank("q", &["a"], None).await.unwrap_err();
        match err {
            Error::Api { status, message } => {
                assert_eq!(status, 400);
                assert_eq!(message, "InvalidParameter: bad doc");
            }
            other => panic!("expected Api, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn rerank_success_non_json_yields_parse() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/reranks"))
            .respond_with(ResponseTemplate::new(200).set_body_string("x"))
            .mount(&server)
            .await;

        let cfg = test_config(&server);
        let r = AliyunRerank::new(&cfg, http_client());
        let err = r.rerank("q", &["a"], None).await.unwrap_err();
        match err {
            Error::Parse(_) => {}
            other => panic!("expected Parse, got {:?}", other),
        }
    }
}
