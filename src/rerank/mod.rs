//! 查询–文档重排序：非流式 JSON，默认 HTTP 超时约 60 秒（可用 [`ProviderConfig::timeout`] 覆盖）。
//!
//! # 支持的厂商
//!
//! 仅 **`Aliyun`** 与 **`Zhipu`**（均须启用 `rerank` 与对应厂商 feature）。未启用对应厂商 feature 时选择阿里云或智谱会得到 [`Error::ProviderDisabled`]。**`OpenAI`** 与 **`Ollama`** 在本模态无实现，工厂返回 [`Error::Unsupported`]（`capability` 为 `"rerank"`）。
//!
//! # HTTP 路径（注意阿里云为复数）
//!
//! - **阿里云**：`POST {base_url}/reranks`（路径段为 **`reranks`**）。
//! - **智谱**：`POST {base_url}/rerank`。
//!
//! `base_url` 均会先 `trim_end_matches('/')` 再拼接。请求体含 `model`、`query`、`documents`（字符串数组）、`top_n`（可选）。成功时解析 `results[].index` 与 `relevance_score`，映射为 [`RerankItem::index`] 与 [`RerankItem::score`]。
//!
//! 智谱侧若分数异常，实现会在启动时打日志提示可改用阿里云 Rerank（以 `tracing` 为准）。
//!
//! # 鉴权
//!
//! 与其它模态相同：Bearer + JSON POST。

mod aliyun;
mod zhipu;

use async_trait::async_trait;
use std::time::Duration;

use crate::client::HttpClient;
use crate::config::Provider;
use crate::config::ProviderConfig;
use crate::error::{Error, Result};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// 单条排序结果：在原始 `documents` 切片中的下标与相关度分数。
#[derive(Debug, Clone)]
pub struct RerankItem {
    pub index: usize,
    pub score: f64,
}

#[async_trait]
pub trait RerankProvider: Send + Sync {
    /// `top_n` 为 `None` 时由上游默认行为决定返回条数。
    async fn rerank(
        &self,
        query: &str,
        documents: &[&str],
        top_n: Option<usize>,
    ) -> Result<Vec<RerankItem>>;
}

fn http_client(config: &ProviderConfig) -> Result<HttpClient> {
    HttpClient::new(config.timeout.unwrap_or(DEFAULT_TIMEOUT))
}

pub(crate) fn create(config: &ProviderConfig) -> Result<Box<dyn RerankProvider>> {
    match config.provider {
        #[cfg(all(feature = "aliyun", feature = "rerank"))]
        Provider::Aliyun => Ok(Box::new(aliyun::AliyunRerank::new(
            config,
            http_client(config)?,
        ))),
        #[cfg(all(feature = "zhipu", feature = "rerank"))]
        Provider::Zhipu => Ok(Box::new(zhipu::ZhipuRerank::new(
            config,
            http_client(config)?,
        ))),
        #[cfg(not(feature = "aliyun"))]
        Provider::Aliyun => Err(Error::ProviderDisabled("aliyun".to_string())),
        #[cfg(not(feature = "zhipu"))]
        Provider::Zhipu => Err(Error::ProviderDisabled("zhipu".to_string())),
        Provider::OpenAI | Provider::Ollama => Err(Error::Unsupported {
            provider: config.provider.to_string(),
            capability: "rerank",
        }),
    }
}

#[cfg(test)]
mod factory_tests {
    use super::create;
    use crate::config::{Provider, ProviderConfig};
    use crate::error::Error;

    #[cfg(feature = "openai")]
    #[test]
    fn openai_is_unsupported() {
        let cfg = ProviderConfig::new(Provider::OpenAI, "k", "https://x/v1", "m");
        match create(&cfg) {
            Err(Error::Unsupported {
                provider,
                capability,
            }) => {
                assert_eq!(provider, "openai");
                assert_eq!(capability, "rerank");
            }
            Ok(_) => panic!("expected error"),
            Err(e) => panic!("expected Unsupported, got {:?}", e),
        }
    }

    #[cfg(feature = "ollama")]
    #[test]
    fn ollama_is_unsupported() {
        let cfg = ProviderConfig::new(Provider::Ollama, "k", "http://localhost/v1", "m");
        match create(&cfg) {
            Err(Error::Unsupported {
                provider,
                capability,
            }) => {
                assert_eq!(provider, "ollama");
                assert_eq!(capability, "rerank");
            }
            Ok(_) => panic!("expected error"),
            Err(e) => panic!("expected Unsupported, got {:?}", e),
        }
    }

    #[cfg(not(feature = "aliyun"))]
    #[test]
    fn aliyun_disabled_without_aliyun_feature() {
        let cfg = ProviderConfig::new(Provider::Aliyun, "k", "https://x/v1", "m");
        match create(&cfg) {
            Err(Error::ProviderDisabled(s)) => assert_eq!(s, "aliyun"),
            Ok(_) => panic!("expected error"),
            Err(e) => panic!("expected ProviderDisabled, got {:?}", e),
        }
    }

    #[cfg(not(feature = "zhipu"))]
    #[test]
    fn zhipu_disabled_without_zhipu_feature() {
        let cfg = ProviderConfig::new(Provider::Zhipu, "k", "https://x/v1", "m");
        match create(&cfg) {
            Err(Error::ProviderDisabled(s)) => assert_eq!(s, "zhipu"),
            Ok(_) => panic!("expected error"),
            Err(e) => panic!("expected ProviderDisabled, got {:?}", e),
        }
    }
}
