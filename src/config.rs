//! Provider 与连接配置

use std::fmt;
use std::str::FromStr;
use std::time::Duration;

/// 已支持的厂商（`#[non_exhaustive]` 便于后续扩展）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Provider {
    OpenAI,
    Aliyun,
    Anthropic,
    Google,
    Ollama,
    Zhipu,
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Provider::OpenAI => "openai",
            Provider::Aliyun => "aliyun",
            Provider::Anthropic => "anthropic",
            Provider::Google => "google",
            Provider::Ollama => "ollama",
            Provider::Zhipu => "zhipu",
        };
        f.write_str(s)
    }
}

impl FromStr for Provider {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "openai" => Ok(Provider::OpenAI),
            "aliyun" => Ok(Provider::Aliyun),
            "anthropic" => Ok(Provider::Anthropic),
            "google" => Ok(Provider::Google),
            "ollama" => Ok(Provider::Ollama),
            "zhipu" => Ok(Provider::Zhipu),
            other => Err(crate::error::Error::UnknownProvider(other.to_string())),
        }
    }
}

/// 厂商连接与调用参数。`model` 为透传字符串：不在此 crate 内校验名称是否在云端可用，由上游 HTTP 响应反映错误（常见为 [`crate::Error::Api`]）。
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider: Provider,
    pub api_key: String,
    pub base_url: String,
    /// 写入各模态请求体的模型标识；合法性与权限由厂商 API 判定，本库不做本地预检。
    pub model: String,
    pub dimension: Option<usize>,
    /// 覆盖该次请求使用的 HTTP 超时；`None` 表示由各模态默认值决定
    pub timeout: Option<Duration>,
    /// 调用方期望的 provider 级并发上限提示；本库不持有 Semaphore，由调用方（如 Docwise 应用层）按此值自行限流。
    pub max_concurrent: Option<usize>,
}

impl ProviderConfig {
    pub fn new(
        provider: Provider,
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            provider,
            api_key: api_key.into(),
            base_url: base_url.into(),
            model: model.into(),
            dimension: None,
            timeout: None,
            max_concurrent: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Provider;
    use std::str::FromStr;

    #[test]
    fn provider_from_str_case_insensitive() {
        assert_eq!(Provider::from_str("openai").unwrap(), Provider::OpenAI);
        assert_eq!(Provider::from_str("Aliyun").unwrap(), Provider::Aliyun);
        assert_eq!(
            Provider::from_str("Anthropic").unwrap(),
            Provider::Anthropic
        );
        assert_eq!(Provider::from_str("google").unwrap(), Provider::Google);
    }

    #[test]
    fn provider_from_str_unknown() {
        assert!(Provider::from_str("unknown").is_err());
    }
}
