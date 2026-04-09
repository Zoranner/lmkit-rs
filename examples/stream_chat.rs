//! 流式对话示例：从环境变量读取密钥，打印增量文本与可选 tool 增量。
//!
//! ```bash
//! set OPENAI_API_KEY=sk-...
//! cargo run --example stream_chat --features openai,chat
//! ```

use futures::StreamExt;
use lmkit::{create_chat_provider, ChatEvent, ChatRequest, Provider, ProviderConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let cfg = ProviderConfig::new(
        Provider::OpenAI,
        api_key,
        "https://api.openai.com/v1",
        "gpt-4o-mini",
    );
    let chat = create_chat_provider(&cfg)?;
    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "用一句话介绍 Rust".to_string());

    let mut stream = chat
        .complete_stream(&ChatRequest::single_user(prompt))
        .await?;
    while let Some(item) = stream.next().await {
        match item? {
            ChatEvent::Delta(text) => print!("{text}"),
            ChatEvent::ToolCallDelta(deltas) => eprintln!("\n[tool_call_deltas: {deltas:?}]"),
            ChatEvent::Finish(reason) => eprintln!("\n[finish: {reason:?}]"),
        }
    }
    println!();
    Ok(())
}
