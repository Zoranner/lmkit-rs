//! 对话补全：非流式 JSON。
//!
//! # [`ChatProvider::chat`]
//!
//! `prompt` 为本轮**唯一用户文本**：OpenAI 兼容分支将其作为单条 `role: user` 消息发送；Anthropic、Google 的请求 JSON 形状见各自实现（Google 单轮示例不发送可选字段 `role`）。均不含 system、多轮 history；若需要须另行扩展 API 或直连接口。
//!
//! **`OpenAI` / `Aliyun` / `Ollama` / `Zhipu`**：OpenAI 兼容 `POST {base_url}/chat/completions`。请求体中 **`temperature` 固定为 `0.2`**，调用方无法通过本 trait 覆盖。
//!
//! **`Anthropic`**（`anthropic` + `chat`）：**Anthropic Messages 兼容**实现，见源码 `anthropic_compat.rs`（模块 `anthropic_compat`，类型 `AnthropicCompatChat`）。`POST {base_url}/messages`，请求头 `x-api-key` 与 `anthropic-version`。与 OpenAI 兼容分支相同，`temperature` 固定为 `0.2`；`max_tokens` 由实现内常量指定。Coding Plan / 代理等若兼容同一契约，只需改 `base_url`。
//!
//! **`Google`**（`google` + `chat`）：**Gemini generateContent** 实现，见源码 `google_gemini.rs`（类型 `GoogleGeminiChat`）。`POST {base_url}/models/{model}:generateContent`，API Key 为 URL query **`key`**（非 Bearer），与[官方 REST 示例](https://ai.google.dev/api/rest/v1beta/models/generateContent)一致。`base_url` 示例：`https://generativelanguage.googleapis.com/v1beta`；请求体为官方单轮形态（`contents[].parts`，可选 `role` 省略）；`temperature` 固定为 `0.2`。若安全策略拦截导致无 `candidates`，见该文件 rustdoc。
//!
//! # URL 与鉴权（OpenAI 兼容分支）
//!
//! 请求地址为 `{base_url}/chat/completions`，其中 `base_url` 来自 [`ProviderConfig`]，会先对 `base_url` 做 `trim_end_matches('/')` 再拼接路径段。
//!
//! 鉴权为 `Authorization: Bearer {api_key}`。**空字符串密钥仍会原样放入请求头**；网关是否接受由上游决定（例如部分本地 Ollama 部署不校验 Bearer）。

#[cfg(feature = "anthropic")]
mod anthropic_compat;
#[cfg(feature = "google")]
mod google_gemini;
mod openai_compat;

#[cfg(feature = "anthropic")]
use anthropic_compat::AnthropicCompatChat;
#[cfg(feature = "anthropic")]
pub use anthropic_compat::ANTHROPIC_VERSION;
use async_trait::async_trait;
#[cfg(feature = "google")]
use google_gemini::GoogleGeminiChat;
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

        #[cfg(feature = "google")]
        Provider::Google => Ok(Box::new(GoogleGeminiChat::new(config)?)),
        #[cfg(not(feature = "google"))]
        Provider::Google => Err(crate::error::Error::ProviderDisabled("google".to_string())),

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
