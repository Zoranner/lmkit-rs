//! **Google Gemini** 文本嵌入：[embedContent](https://ai.google.dev/api/rest/v1beta/models.embedContent) 与 [batchEmbedContents](https://ai.google.dev/api/rest/v1beta/models.batchEmbedContents)。
//!
//! 路径：`POST {base_url}/models/{model}:embedContent`（单条）或 `…:batchEmbedContents`（批量）。`{model}` 为 [`ProviderConfig::model`] 原样嵌入路径（如 `gemini-embedding-001`），与对话 `generateContent` 一致。请求体中的 `model` 字段为资源名 **`models/{model}`**（若配置已含 `models/` 前缀则不再重复拼接），见官方 REST 示例。
//!
//! 鉴权与对话相同：**query 参数 `key`** 传递 `api_key`，无 Bearer。
//!
//! 请求体含 `content.parts[].text`；若配置中 [`ProviderConfig::dimension`] 已设置，则发送 **`outputDimensionality`**（与官方字段一致），用于输出维数；较早模型（如文档所述 `models/embedding-001`）可能不支持该字段，须以厂商响应为准。
//!
//! 成功时解析 `embedding.values`（单条）或 `embeddings[].values`（批量）。若向量长度与配置的 `dimension` 不一致，返回 [`Error::Parse`]。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::EmbedProvider;
use crate::client::HttpClient;
use crate::config::ProviderConfig;
use crate::error::{Error, Result};
use crate::util::normalize_for_embedding;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmbedContentRequest<'a> {
    model: String,
    content: GeminiContent<'a>,
    output_dimensionality: usize,
}

#[derive(Debug, Serialize)]
struct GeminiContent<'a> {
    parts: Vec<GeminiPart<'a>>,
}

#[derive(Debug, Serialize)]
struct GeminiPart<'a> {
    text: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchEmbedContentsRequest<'a> {
    requests: Vec<EmbedContentRequest<'a>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EmbedContentResponse {
    embedding: ContentEmbedding,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchEmbedContentsResponse {
    #[serde(default)]
    embeddings: Vec<ContentEmbedding>,
}

#[derive(Debug, Deserialize)]
struct ContentEmbedding {
    #[serde(default)]
    values: Vec<f32>,
}

pub(crate) struct GoogleGeminiEmbed {
    client: HttpClient,
    api_key: String,
    /// 用于 URL 路径段，与 chat 一致（不含 `models/` 前缀）。
    model_path_id: String,
    /// 请求体 `model` 字段：`models/...`
    model_resource: String,
    base_url: String,
    dimension: usize,
}

fn model_resource_name(model: &str) -> String {
    let m = model.trim();
    if m.starts_with("models/") {
        m.to_string()
    } else {
        format!("models/{m}")
    }
}

/// URL 路径中的 `{model}` 段（与 `generateContent` 一致）：不含 `models/` 前缀。
fn model_path_segment(model: &str) -> String {
    let m = model.trim();
    m.strip_prefix("models/").unwrap_or(m).to_string()
}

impl GoogleGeminiEmbed {
    pub fn new(config: &ProviderConfig, dimension: usize, client: HttpClient) -> Self {
        let model_resource = model_resource_name(&config.model);
        Self {
            client,
            api_key: config.api_key.clone(),
            model_path_id: model_path_segment(&config.model),
            model_resource,
            base_url: config.base_url.clone(),
            dimension,
        }
    }

    fn check_vector_len(values: &[f32], expected: usize) -> Result<()> {
        if values.len() != expected {
            return Err(Error::Parse(format!(
                "Gemini embedding length {} does not match configured dimension {}",
                values.len(),
                expected
            )));
        }
        Ok(())
    }

    async fn embed_content(&self, text: &str) -> Result<Vec<f32>> {
        let req = EmbedContentRequest {
            model: self.model_resource.clone(),
            content: GeminiContent {
                parts: vec![GeminiPart { text }],
            },
            output_dimensionality: self.dimension,
        };
        let base = self.base_url.trim_end_matches('/');
        let url = format!("{}/models/{}:embedContent", base, self.model_path_id);
        let query = [("key", self.api_key.as_str())];

        let resp: EmbedContentResponse = self
            .client
            .post_json_query(&url, &query, &req, |s| s)
            .await?;

        Self::check_vector_len(&resp.embedding.values, self.dimension)?;
        Ok(resp.embedding.values)
    }

    async fn batch_embed_contents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let requests: Vec<EmbedContentRequest<'_>> = texts
            .iter()
            .map(|t| EmbedContentRequest {
                model: self.model_resource.clone(),
                content: GeminiContent {
                    parts: vec![GeminiPart { text: t.as_str() }],
                },
                output_dimensionality: self.dimension,
            })
            .collect();

        let body = BatchEmbedContentsRequest { requests };
        let base = self.base_url.trim_end_matches('/');
        let url = format!("{}/models/{}:batchEmbedContents", base, self.model_path_id);
        let query = [("key", self.api_key.as_str())];

        let resp: BatchEmbedContentsResponse = self
            .client
            .post_json_query(&url, &query, &body, |s| s)
            .await?;

        if resp.embeddings.len() != texts.len() {
            return Err(Error::Parse(format!(
                "Gemini batchEmbedContents returned {} embeddings for {} inputs",
                resp.embeddings.len(),
                texts.len()
            )));
        }

        let mut out = Vec::with_capacity(resp.embeddings.len());
        for emb in resp.embeddings {
            Self::check_vector_len(&emb.values, self.dimension)?;
            out.push(emb.values);
        }
        Ok(out)
    }
}

#[async_trait]
impl EmbedProvider for GoogleGeminiEmbed {
    async fn encode(&self, text: &str) -> Result<Vec<f32>> {
        let normalized = normalize_for_embedding(text);
        self.embed_content(&normalized).await
    }

    async fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let normalized: Vec<String> = texts.iter().map(|t| normalize_for_embedding(t)).collect();
        if normalized.is_empty() {
            return Ok(vec![]);
        }
        if normalized.len() == 1 {
            let v = self.embed_content(&normalized[0]).await?;
            return Ok(vec![v]);
        }
        self.batch_embed_contents(&normalized).await
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Provider, ProviderConfig};
    use std::time::Duration;
    use wiremock::matchers::{body_json, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn http_client() -> HttpClient {
        HttpClient::new(Duration::from_secs(30)).unwrap()
    }

    fn test_config(server: &MockServer) -> ProviderConfig {
        let mut cfg = ProviderConfig::with_base_url(
            Provider::Google,
            "AIza-test",
            format!("{}/v1beta", server.uri()),
            "gemini-embedding-001",
        );
        cfg.dimension = Some(3);
        cfg
    }

    #[tokio::test]
    async fn embed_content_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1beta/models/gemini-embedding-001:embedContent"))
            .and(query_param("key", "AIza-test"))
            .and(body_json(serde_json::json!({
                "model": "models/gemini-embedding-001",
                "content": { "parts": [{ "text": "a b" }] },
                "outputDimensionality": 3
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "embedding": { "values": [0.1, 0.2, 0.3] }
            })))
            .mount(&server)
            .await;

        let cfg = test_config(&server);
        let dim = cfg.dimension.unwrap();
        let emb = GoogleGeminiEmbed::new(&cfg, dim, http_client());
        let out = emb.encode("  a \n b").await.unwrap();
        assert_eq!(out, vec![0.1f32, 0.2, 0.3]);
        assert_eq!(emb.dimension(), 3);
    }

    #[tokio::test]
    async fn batch_embed_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(
                "/v1beta/models/gemini-embedding-001:batchEmbedContents",
            ))
            .and(query_param("key", "AIza-test"))
            .and(body_json(serde_json::json!({
                "requests": [
                    {
                        "model": "models/gemini-embedding-001",
                        "content": { "parts": [{ "text": "first" }] },
                        "outputDimensionality": 2
                    },
                    {
                        "model": "models/gemini-embedding-001",
                        "content": { "parts": [{ "text": "second" }] },
                        "outputDimensionality": 2
                    }
                ]
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "embeddings": [
                    { "values": [1.0, 0.0] },
                    { "values": [0.0, 1.0] }
                ]
            })))
            .mount(&server)
            .await;

        let mut cfg = test_config(&server);
        cfg.dimension = Some(2);
        let dim = cfg.dimension.unwrap();
        let emb = GoogleGeminiEmbed::new(&cfg, dim, http_client());
        let out = emb.encode_batch(&["first", "second"]).await.unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], vec![1.0f32, 0.0]);
        assert_eq!(out[1], vec![0.0f32, 1.0]);
    }

    #[tokio::test]
    async fn model_config_with_models_prefix_in_body_only() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1beta/models/gemini-embedding-001:embedContent"))
            .and(query_param("key", "k"))
            .and(body_json(serde_json::json!({
                "model": "models/gemini-embedding-001",
                "content": { "parts": [{ "text": "x" }] },
                "outputDimensionality": 1
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "embedding": { "values": [0.5] }
            })))
            .mount(&server)
            .await;

        let mut cfg = ProviderConfig::with_base_url(
            Provider::Google,
            "k",
            format!("{}/v1beta", server.uri()),
            "models/gemini-embedding-001",
        );
        cfg.dimension = Some(1);
        let emb = GoogleGeminiEmbed::new(&cfg, 1, http_client());
        emb.encode("x").await.unwrap();
    }

    #[tokio::test]
    async fn wrong_length_yields_parse() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1beta/models/gemini-embedding-001:embedContent"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "embedding": { "values": [0.1, 0.2] }
            })))
            .mount(&server)
            .await;

        let cfg = test_config(&server);
        let emb = GoogleGeminiEmbed::new(&cfg, 3, http_client());
        let err = emb.encode("x").await.unwrap_err();
        match err {
            Error::Parse(msg) => assert!(msg.contains("length")),
            e => panic!("expected Parse, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn api_error_maps() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1beta/models/gemini-embedding-001:embedContent"))
            .respond_with(ResponseTemplate::new(400).set_body_string("bad"))
            .mount(&server)
            .await;

        let cfg = test_config(&server);
        let emb = GoogleGeminiEmbed::new(&cfg, 3, http_client());
        let err = emb.encode("x").await.unwrap_err();
        match err {
            Error::Api { status, message } => {
                assert_eq!(status, 400);
                assert_eq!(message, "bad");
            }
            e => panic!("expected Api, got {:?}", e),
        }
    }
}
