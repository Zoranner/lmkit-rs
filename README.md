# lmkit

[![Crates.io](https://img.shields.io/crates/v/lmkit.svg)](https://crates.io/crates/lmkit) [![Docs.rs](https://docs.rs/lmkit/badge.svg)](https://docs.rs/lmkit) [![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**One config. Every major AI provider.**

[中文](README.zh.md) | English

A unified Rust client for OpenAI, Anthropic, Google Gemini, Aliyun, Ollama, and Zhipu — built around a single trait and factory pattern. **Switch providers by changing one config. Your business logic stays untouched.**

## Why use lmkit

- 🔌 **Unified interface** — `ChatProvider`, `EmbedProvider` and friends abstract away provider differences; your code never talks to raw HTTP
- 🔀 **One-line switching** — swap `ProviderConfig` to move from OpenAI to Aliyun or a local Ollama, zero other changes
- 📦 **Compile only what you need** — providers and modalities are Cargo features; unused ones add zero dependencies
- 🌊 **Streaming + tool calls** — native SSE streaming; `ChatChunk` carries both text `delta` and `tool_call_deltas` in one unified type
- 🔍 **Precise errors** — `ProviderDisabled` / `Unsupported` / `Api` tell you exactly what went wrong and where

## Quick Start

### Add the dependency

```toml
[dependencies]
lmkit = { version = "0.1", features = ["openai", "chat", "embed"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

The defaults already include `openai` + `chat` + `embed`. Mix and match features as needed:

```toml
# Aliyun + multi-turn chat + embeddings + reranking
lmkit = { version = "0.1", features = ["aliyun", "chat", "embed", "rerank"] }
```

### Send a message

```rust
use lmkit::{create_chat_provider, ChatRequest, Provider, ProviderConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = ProviderConfig::new(
        Provider::OpenAI,
        std::env::var("OPENAI_API_KEY")?,
        "https://api.openai.com/v1",
        "gpt-4o-mini",
    );

    let chat = create_chat_provider(&cfg)?;
    let out = chat
        .complete(&ChatRequest::single_user("Explain Rust in one sentence."))
        .await?;
    println!("{}", out.content.unwrap_or_default());
    Ok(())
}
```

### Stream the response

```rust
use futures::StreamExt;
use lmkit::{create_chat_provider, ChatRequest, Provider, ProviderConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = ProviderConfig::new(
        Provider::OpenAI,
        std::env::var("OPENAI_API_KEY")?,
        "https://api.openai.com/v1",
        "gpt-4o-mini",
    );

    let chat = create_chat_provider(&cfg)?;
    let mut stream = chat
        .complete_stream(&ChatRequest::single_user("Tell me a joke."))
        .await?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(text) = chunk.delta {
            print!("{text}");
        }
    }
    println!();
    Ok(())
}
```

### Switch providers

Change `Provider::OpenAI` to your target, update `base_url` and the API key — everything else stays the same:

```rust
// Aliyun Qwen
let cfg = ProviderConfig::new(
    Provider::Aliyun,
    std::env::var("DASHSCOPE_API_KEY")?,
    "https://dashscope.aliyuncs.com/compatible-mode/v1",
    "qwen-turbo",
);

// Local Ollama (no key required)
let cfg = ProviderConfig::new(
    Provider::Ollama,
    String::new(),
    "http://127.0.0.1:11434/v1",
    "llama3",
);
```

## Provider & Capability Matrix

| Provider | Chat | Embed | Rerank | Image |
|:---|:---:|:---:|:---:|:---:|
| OpenAI | ✅ | ✅ | — | ✅ |
| Anthropic | ✅ | — | — | — |
| Google Gemini | ✅ | ✅ | — | — |
| Aliyun DashScope | ✅ | ✅ | ✅ | ✅ |
| Ollama | ✅ | ✅ | — | — |
| Zhipu | ✅ | ✅ | ✅ | — |

Chat primary API: `complete` (blocking) and `complete_stream` (SSE). `chat` / `chat_stream` are single-turn convenience wrappers.

## Documentation

- 📖 [Usage Guide](docs/README.md) — getting started, features, provider config, error handling
- 🔧 [API Reference](docs/reference/api.md) — Rust traits, factory functions, type definitions
- 🌐 [HTTP Endpoints](docs/reference/http-endpoints.md) — per-provider request / response shapes
- 🏗️ [Design Guidelines](docs/reference/design.md) — architecture and extension principles
- 🤝 [Contributing](docs/reference/contributing.md) — how to add providers or modalities

## License

[MIT](LICENSE)
