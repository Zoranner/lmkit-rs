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

pub type Result<T> = std::result::Result<T, Error>;
