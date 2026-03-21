# model-provider

用同一套配置在 Rust 里调用多家云的 **对话**、**向量**、**重排序** API（HTTPS，默认 rustls）。按需打开 Cargo feature，用不到的厂商不会编进产物。

## 🚀 快速接入

在 `Cargo.toml` 里写上依赖和 feature（路径发布时改成你的实际路径或 crates.io 版本号）：

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

| 厂商 | Chat | Embed | Rerank |
|:---:|:---:|:---:|:---:|
| OpenAI | ✅ | ✅ | ❌ |
| 阿里云 | ✅ | ✅ | ✅ |
| Ollama | ✅ | ✅ | ❌ |
| 智谱 | ✅ | ✅ 🔧 | ✅ |

图例：🔧 表示「专用请求体」——请求 JSON 与标准 OpenAI 兼容格式不完全一致；上表仅智谱 Embed 使用该标记（例如不传 `dimensions`）。

图像生成、语音识别与合成在库里有 trait 和工厂入口，具体厂商对接仍在迭代；需要时可开 `image` / `audio` feature 查看 API 形状。

## ⚙️ 常用 feature 组合

| 你想用 | `features` 示例 |
|:---|:---|
| 只要 OpenAI 对话 + 向量 | 默认不写，或 `["openai", "chat", "embed"]` |
| 全开厂商与能力 | `["full"]` 或 `["all"]` |
| 仅本地 Ollama | `["ollama", "chat", "embed"]` |
| 阿里云 rerank | 在已有 embed/chat 上再加 `aliyun` 与 `rerank` |

厂商 feature（`openai` / `aliyun` / `ollama` / `zhipu`）与模态 feature（`chat` / `embed` / `rerank` / `image` / `audio`）要同时满足才会在对应工厂里可用；配错组合会得到明确的 `Error`，而不是静默失败。

## 📜 许可证

[MIT](LICENSE)
