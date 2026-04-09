//! 库级错误类型。
//!
//! # `ProviderDisabled` 与 `Unsupported`
//!
//! **`ProviderDisabled`**：当前编译配置下该厂商或模态未启用（Cargo feature 组合不满足），例如启了 `embed` 但未启 `zhipu` 时仍选智谱，或启了 `rerank` 但未启 `aliyun` / `zhipu` 时仍选阿里云 / 智谱。
//!
//! **`Unsupported`**：对应模态的工厂已编译，但该厂商在该能力上**没有实现**（例如 `create_image_provider` 对 **`Ollama` / `Zhipu`**，或 `create_rerank_provider` 对 **`OpenAI` / `Ollama` / `Google`**），或占位能力（如 `audio` 工厂尚未接任何远端）。已支持文生图但未把 `openai` / `aliyun` 编进产物时，选对应厂商应得到 `ProviderDisabled`，而非本变体。

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unknown provider name: {0}")]
    UnknownProvider(String),

    /// 厂商或模态未在 Cargo feature 中启用（例如启了 `rerank` 但未启 `aliyun` 仍选阿里云）。
    #[error("provider `{0}` is not enabled (Cargo feature or modality)")]
    ProviderDisabled(String),

    /// 该厂商在此模态下无实现，或能力仍为占位（如语音工厂）。
    #[error("capability `{capability}` is not supported for provider `{provider}`")]
    Unsupported {
        provider: String,
        capability: &'static str,
    },

    #[error("missing required config: {0}")]
    MissingConfig(&'static str),

    /// 上游 HTTP 非 2xx；`message` 来自响应体（经各模态解析或拼接）。未知模型、鉴权失败、参数错误等通常落在此变体，而非单独的「模型」枚举分支。
    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error("failed to parse API response: {0}")]
    Parse(String),

    #[error("API response missing expected field: {0}")]
    MissingField(&'static str),
}

impl Error {
    /// 是否属于可安全重试的错误（幂等前提下）。
    ///
    /// 以下情况返回 `true`：
    /// - HTTP 429 Too Many Requests
    /// - HTTP 5xx 服务端错误
    /// - `reqwest` 网络层错误（连接超时、连接重置等）
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Api { status, .. } => *status == 429 || *status >= 500,
            Error::Http(_) => true,
            _ => false,
        }
    }

    /// 是否需要人工介入（鉴权失败、权限不足、资源不存在等）。
    ///
    /// 以下情况返回 `true`：
    /// - HTTP 401 Unauthorized
    /// - HTTP 403 Forbidden
    /// - HTTP 404 Not Found
    pub fn requires_human(&self) -> bool {
        matches!(self, Error::Api { status, .. } if matches!(status, 401 | 403 | 404))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
