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

impl Provider {
    const fn default_base_url(self) -> &'static str {
        match self {
            Provider::OpenAI => "https://api.openai.com/v1",
            Provider::Aliyun => "https://dashscope.aliyuncs.com/compatible-mode/v1",
            Provider::Anthropic => "https://api.anthropic.com/v1",
            Provider::Google => "https://generativelanguage.googleapis.com/v1beta",
            Provider::Ollama => "http://localhost:11434/v1",
            Provider::Zhipu => "https://open.bigmodel.cn/api/paas/v4",
        }
    }
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
    pub fn new(provider: Provider, api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::with_base_url(provider, api_key, provider.default_base_url(), model)
    }

    /// 使用显式 `base_url` 创建配置。
    ///
    /// 用于代理、私有网关、区域化端点，或阿里云文生图等与默认网关不同的路径。
    pub fn with_base_url(
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

    #[test]
    fn provider_default_base_url_matches_official_endpoints() {
        assert_eq!(
            Provider::OpenAI.default_base_url(),
            "https://api.openai.com/v1"
        );
        assert_eq!(
            Provider::Aliyun.default_base_url(),
            "https://dashscope.aliyuncs.com/compatible-mode/v1"
        );
        assert_eq!(
            Provider::Anthropic.default_base_url(),
            "https://api.anthropic.com/v1"
        );
        assert_eq!(
            Provider::Google.default_base_url(),
            "https://generativelanguage.googleapis.com/v1beta"
        );
        assert_eq!(
            Provider::Ollama.default_base_url(),
            "http://localhost:11434/v1"
        );
        assert_eq!(
            Provider::Zhipu.default_base_url(),
            "https://open.bigmodel.cn/api/paas/v4"
        );
    }

    #[test]
    fn provider_config_new_uses_default_base_url() {
        let cfg = super::ProviderConfig::new(Provider::OpenAI, "sk-test", "gpt-test");

        assert_eq!(cfg.provider, Provider::OpenAI);
        assert_eq!(cfg.api_key, "sk-test");
        assert_eq!(cfg.base_url, Provider::OpenAI.default_base_url());
        assert_eq!(cfg.model, "gpt-test");
        assert_eq!(cfg.dimension, None);
        assert_eq!(cfg.timeout, None);
        assert_eq!(cfg.max_concurrent, None);
    }

    #[test]
    fn provider_config_with_base_url_preserves_explicit_endpoint() {
        let cfg = super::ProviderConfig::with_base_url(
            Provider::Zhipu,
            "zk-test",
            "https://api.z.ai/api/paas/v4",
            "glm-test",
        );

        assert_eq!(cfg.provider, Provider::Zhipu);
        assert_eq!(cfg.api_key, "zk-test");
        assert_eq!(cfg.base_url, "https://api.z.ai/api/paas/v4");
        assert_eq!(cfg.model, "glm-test");
        assert_eq!(cfg.dimension, None);
        assert_eq!(cfg.timeout, None);
        assert_eq!(cfg.max_concurrent, None);
    }
}
