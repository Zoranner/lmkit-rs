# lmkit

[![Crates.io](https://img.shields.io/crates/v/lmkit.svg)](https://crates.io/crates/lmkit) [![Docs.rs](https://docs.rs/lmkit/badge.svg)](https://docs.rs/lmkit) [![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**一套代码，接入所有主流 AI 厂商。**

中文 | [English](README.md)

用统一的 trait 和工厂函数调用 OpenAI、Anthropic、Google Gemini、阿里云、Ollama、智谱。**换厂商只需改一行配置，业务代码不动。**

## 为什么选它

- 🔌 **统一接口** — `ChatProvider`、`EmbedProvider` 等 trait 屏蔽厂商差异，业务代码不感知底层 API
- 🔀 **一行切换** — 改一个 `ProviderConfig`，从 OpenAI 切到阿里云或本地 Ollama，其余代码不动
- 📦 **按需编译** — 厂商与能力均为 Cargo feature，只引入需要的依赖，不拉多余的包
- 🌊 **流式 + 工具调用** — 原生 SSE 流式输出，`ChatEvent` 枚举统一携带文本 delta、工具调用增量与结束原因
- 🔍 **清晰的错误** — `ProviderDisabled` / `Unsupported` / `Api` 分级报错，排查一目了然

## 快速开始

### 添加依赖

```toml
[dependencies]
lmkit = { version = "0.1", features = ["openai", "chat", "embed"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

默认已包含 `openai` + `chat` + `embed`。需要其他厂商或能力时调整 feature：

```toml
# 阿里云 + 多轮对话 + 向量 + 重排序
lmkit = { version = "0.1", features = ["aliyun", "chat", "embed", "rerank"] }
```

### 发起对话

```rust
use lmkit::{create_chat_provider, ChatRequest, Provider, ProviderConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = ProviderConfig::new(
        Provider::OpenAI,
        std::env::var("OPENAI_API_KEY")?,
        "gpt-4o-mini",
    );

    let chat = create_chat_provider(&cfg)?;
    let out = chat
        .complete(&ChatRequest::single_user("用一句话介绍 Rust"))
        .await?;
    println!("{}", out.content.unwrap_or_default());
    Ok(())
}
```

### 流式对话

```rust
use futures::StreamExt;
use lmkit::{create_chat_provider, ChatRequest, Provider, ProviderConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = ProviderConfig::new(
        Provider::OpenAI,
        std::env::var("OPENAI_API_KEY")?,
        "gpt-4o-mini",
    );

    let chat = create_chat_provider(&cfg)?;
    let mut stream = chat
        .complete_stream(&ChatRequest::single_user("讲一个笑话"))
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

### 切换厂商

把 `Provider::OpenAI` 改成目标厂商并更新密钥即可，内置厂商已提供默认 `base_url`：

```rust
// 切到阿里云 Qwen
let cfg = ProviderConfig::new(
    Provider::Aliyun,
    std::env::var("DASHSCOPE_API_KEY")?,
    "qwen-turbo",
);

// 切到本地 Ollama
let cfg = ProviderConfig::new(
    Provider::Ollama,
    String::new(),
    "llama3",
);
```

需要代理、私有网关、区域端点，或阿里云文生图这类模态专用路径时，使用 `ProviderConfig::with_base_url` 显式传入 `base_url`。

## 支持的厂商与能力

| 厂商 | Chat | Embed | Rerank | Image |
|:---|:---:|:---:|:---:|:---:|
| OpenAI | ✅ | ✅ | — | ✅ |
| Anthropic | ✅ | — | — | — |
| Google Gemini | ✅ | ✅ | — | — |
| 阿里云 DashScope | ✅ | ✅ | ✅ | ✅ |
| Ollama | ✅ | ✅ | — | — |
| 智谱 | ✅ | ✅ | ✅ | — |

Chat 主路径：`complete`（非流式）与 `complete_stream`（SSE）；`chat` / `chat_stream` 为单轮快捷方式。

## 文档

- 📖 [使用指南](docs/README.md) — 快速上手、功能介绍、厂商配置、错误处理
- 🔧 [API 参考](docs/reference/api.md) — Rust trait、工厂函数、类型说明
- 🌐 [HTTP 端点](docs/reference/http-endpoints.md) — 各厂商请求 / 响应格式
- 🏗️ [设计准则](docs/reference/design.md) — 库的架构思路
- 🤝 [贡献指南](docs/reference/contributing.md) — 参与开发

## 许可证

[MIT](LICENSE)
