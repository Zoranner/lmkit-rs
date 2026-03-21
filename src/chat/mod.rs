//! 对话补全：非流式 JSON。
//!
//! # [`ChatProvider::chat`]
//!
//! `prompt` 会作为**唯一一条** `role: user` 消息发送，不包含 system / 多轮 history。若需要多轮或 system prompt，须另行扩展 API 或直接使用上游 HTTP。
//!
//! **`OpenAI` / `Aliyun` / `Ollama` / `Zhipu`**：OpenAI 兼容 `POST {base_url}/chat/completions`。请求体中 **`temperature` 固定为 `0.2`**，调用方无法通过本 trait 覆盖。
//!
//! **`Anthropic`**（`anthropic` + `chat`）：**Anthropic Messages 兼容**实现，见源码 `anthropic_compat.rs`（模块 `anthropic_compat`，类型 `AnthropicCompatChat`）。`POST {base_url}/messages`，请求头 `x-api-key` 与 `anthropic-version`。与 OpenAI 兼容分支相同，`temperature` 固定为 `0.2`；`max_tokens` 由实现内常量指定。Coding Plan / 代理等若兼容同一契约，只需改 `base_url`。
//!
//! # URL 与鉴权（OpenAI 兼容分支）
//!
//! 请求地址为 `{base_url}/chat/completions`，其中 `base_url` 来自 [`ProviderConfig`]，会先对 `base_url` 做 `trim_end_matches('/')` 再拼接路径段。
//!
//! 鉴权为 `Authorization: Bearer {api_key}`。**空字符串密钥仍会原样放入请求头**；网关是否接受由上游决定（例如部分本地 Ollama 部署不校验 Bearer）。

#[cfg(feature = "anthropic")]
mod anthropic_compat;
mod openai_compat;

#[cfg(feature = "anthropic")]
use anthropic_compat::AnthropicCompatChat;
#[cfg(feature = "anthropic")]
pub use anthropic_compat::ANTHROPIC_VERSION;
use async_trait::async_trait;
use openai_compat::OpenaiCompatChat;

use crate::config::Provider;
use crate::config::ProviderConfig;
use crate::error::Result;

#[async_trait]
pub trait ChatProvider: Send + Sync {
    /// 单轮用户消息补全；语义与模块级文档一致。
    async fn chat(&self, prompt: &str) -> Result<String>;
}

pub(crate) fn create(config: &ProviderConfig) -> Result<Box<dyn ChatProvider>> {
    match config.provider {
        #[cfg(feature = "openai")]
        Provider::OpenAI => Ok(Box::new(OpenaiCompatChat::new(config)?)),
        #[cfg(not(feature = "openai"))]
        Provider::OpenAI => Err(crate::error::Error::ProviderDisabled("openai".to_string())),

        #[cfg(feature = "aliyun")]
        Provider::Aliyun => Ok(Box::new(OpenaiCompatChat::new(config)?)),
        #[cfg(not(feature = "aliyun"))]
        Provider::Aliyun => Err(crate::error::Error::ProviderDisabled("aliyun".to_string())),

        #[cfg(feature = "anthropic")]
        Provider::Anthropic => Ok(Box::new(AnthropicCompatChat::new(config)?)),
        #[cfg(not(feature = "anthropic"))]
        Provider::Anthropic => Err(crate::error::Error::ProviderDisabled(
            "anthropic".to_string(),
        )),

        #[cfg(feature = "ollama")]
        Provider::Ollama => Ok(Box::new(OpenaiCompatChat::new(config)?)),
        #[cfg(not(feature = "ollama"))]
        Provider::Ollama => Err(crate::error::Error::ProviderDisabled("ollama".to_string())),

        #[cfg(feature = "zhipu")]
        Provider::Zhipu => Ok(Box::new(OpenaiCompatChat::new(config)?)),
        #[cfg(not(feature = "zhipu"))]
        Provider::Zhipu => Err(crate::error::Error::ProviderDisabled("zhipu".to_string())),
    }
}
