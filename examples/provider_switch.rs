//! Non-streaming chat example that switches providers through configuration only.
//!
//! ```bash
//! # OpenAI
//! set LMKIT_PROVIDER=openai
//! set OPENAI_API_KEY=sk-...
//! cargo run --example provider_switch --features openai,chat
//!
//! # Aliyun
//! set LMKIT_PROVIDER=aliyun
//! set DASHSCOPE_API_KEY=sk-...
//! cargo run --example provider_switch --features aliyun,chat
//!
//! # Local Ollama
//! set LMKIT_PROVIDER=ollama
//! cargo run --example provider_switch --features ollama,chat
//! ```

use std::str::FromStr;

use lmkit::{create_chat_provider, ChatRequest, Provider, ProviderConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = std::env::var("LMKIT_PROVIDER").unwrap_or_else(|_| "openai".to_string());
    let provider = Provider::from_str(&provider)?;
    let cfg = config_for(provider)?;
    let chat = create_chat_provider(&cfg)?;

    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "Explain Rust in one sentence.".to_string());
    let response = chat.complete(&ChatRequest::single_user(prompt)).await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}

fn config_for(provider: Provider) -> Result<ProviderConfig, Box<dyn std::error::Error>> {
    let cfg = match provider {
        Provider::OpenAI => ProviderConfig::new(
            Provider::OpenAI,
            std::env::var("OPENAI_API_KEY")?,
            "gpt-4o-mini",
        ),
        Provider::Aliyun => ProviderConfig::new(
            Provider::Aliyun,
            std::env::var("DASHSCOPE_API_KEY")?,
            "qwen-turbo",
        ),
        Provider::Ollama => ProviderConfig::new(Provider::Ollama, String::new(), "llama3.2"),
        other => {
            return Err(format!(
                "provider `{other}` is not wired in this example; use openai, aliyun, or ollama"
            )
            .into())
        }
    };

    Ok(cfg)
}
