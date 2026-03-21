# model-provider

用同一套配置在 Rust 里调用多家云的 **对话**、**向量**、**重排序** API（HTTPS，默认 rustls）。按需打开 Cargo feature，用不到的厂商不会编进产物。

各能力的 Rust API 与 HTTP 约定已整理在 [docs 目录](docs/README.md)（含 [接口一览](docs/interfaces.md)）。维护与扩展本库时的设计约定见 [docs/design-guidelines.md](docs/design-guidelines.md)。

## 🚀 快速接入

在 `Cargo.toml` 里添加依赖与 feature：

```toml
[dependencies]
model-provider = { version = "0.2", features = ["openai", "chat", "embed"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

默认已包含 `openai` + `chat` + `embed`。要用阿里云 rerank，改成例如 `features = ["aliyun", "chat", "embed", "rerank"]`。

下面示例：填好网关地址和密钥后即可调用（需在 async 运行时里执行）。

```rust
use model_provider::{
    create_chat_provider, create_embed_provider, Provider, ProviderConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = ProviderConfig::new(
        Provider::OpenAI,
        std::env::var("API_KEY")?,
        "https://api.openai.com/v1",
        "gpt-4o-mini",
    );
    cfg.dimension = Some(1536); // embed 必填

    let chat = create_chat_provider(&cfg)?;
    let reply = chat.chat("用一句话介绍 Rust").await?;
    println!("{reply}");

    let emb = create_embed_provider(&cfg)?;
    let v = emb.encode("hello").await?;
    println!("dim = {}", v.len());
    Ok(())
}
```

## 📊 各厂商支持的能力

同一能力在不同云上都是「填 `base_url` + 模型名」，由对方是否提供 OpenAI 兼容接口决定；embed 一般要设置 `dimension`（维数依模型而定）。

| 厂商 | Chat | Embed | Rerank | Image |
|:---:|:---:|:---:|:---:|:---:|
| OpenAI | ✅ | ✅ | ❌ | ✅ |
| Anthropic | ✅ 🔧 | ❌ | ❌ | ❌ |
| Google (Gemini) | ✅ 🔧 | ❌ | ❌ | ❌ |
| 阿里云 | ✅ | ✅ | ✅ | ✅ 🔧 |
| Ollama | ✅ | ✅ | ❌ | ❌ |
| 智谱 | ✅ | ✅ 🔧 | ✅ | ❌ |

图例：🔧 表示该能力与 **OpenAI 兼容形态不一致**（路径、请求体或鉴权之一与 `…/chat/completions` + Bearer 不同）。矩阵只作概览；**各厂商的 URL、`base_url`、`model` 写法与请求字段**以 [HTTP 端点汇总](docs/http-api.md) 与 [接口一览](docs/interfaces.md) 为准，避免在 README 里随厂商数量膨胀。`audio` 仍为占位，仅暴露类型，见该文档 Audio 一节。

## 📜 许可证

[MIT](LICENSE)
