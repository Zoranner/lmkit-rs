# 快速上手

本文引导你完成从零到第一次成功调用的全过程。

## 添加依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
lmkit = { version = "0.2", features = ["openai", "chat", "embed"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

> **Feature 说明**
>
> - 厂商 feature（`openai`、`anthropic`、`google`、`aliyun`、`ollama`、`zhipu`）控制哪些厂商被编译进来
> - 能力 feature（`chat`、`embed`、`rerank`、`image`）控制哪些能力被编译进来
> - 两者都需要启用，才能调用对应厂商的对应能力

常见组合：

```toml
# 只用 OpenAI 对话
features = ["openai", "chat"]

# 阿里云：对话 + 向量 + 重排序
features = ["aliyun", "chat", "embed", "rerank"]

# 多厂商
features = ["openai", "anthropic", "google", "chat", "embed"]
```

---

## 第一次对话

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
    let response = chat
        .complete(&ChatRequest::single_user("用一句话介绍 Rust"))
        .await?;

    println!("{}", response.content.unwrap_or_default());
    Ok(())
}
```

运行前设置环境变量：

```bash
export OPENAI_API_KEY=sk-xxx   # Linux / macOS
$env:OPENAI_API_KEY="sk-xxx"   # Windows PowerShell
```

---

## 第一次向量化

```rust
use lmkit::{create_embed_provider, Provider, ProviderConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = ProviderConfig::new(
        Provider::OpenAI,
        std::env::var("OPENAI_API_KEY")?,
        "text-embedding-3-small",
    );
    cfg.dimension = Some(1536); // embed 必须设置维度

    let embed = create_embed_provider(&cfg)?;
    let vector = embed.encode("Hello, world!").await?;

    println!("向量维度: {}", vector.len()); // 1536
    Ok(())
}
```

---

## 切换厂商

所有厂商共用相同的 API，只需改配置：

```rust
// OpenAI
let cfg = ProviderConfig::new(
    Provider::OpenAI,
    std::env::var("OPENAI_API_KEY")?,
    "gpt-4o-mini",
);

// Anthropic Claude
let cfg = ProviderConfig::new(
    Provider::Anthropic,
    std::env::var("ANTHROPIC_API_KEY")?,
    "claude-3-5-sonnet-20241022",
);

// Google Gemini
let cfg = ProviderConfig::new(
    Provider::Google,
    std::env::var("GOOGLE_API_KEY")?,
    "gemini-1.5-flash",
);

// 阿里云 DashScope
let cfg = ProviderConfig::new(
    Provider::Aliyun,
    std::env::var("DASHSCOPE_API_KEY")?,
    "qwen-turbo",
);

// 本地 Ollama（无需 API Key）
let cfg = ProviderConfig::new(
    Provider::Ollama,
    String::new(),
    "llama3.2",
);
```

业务代码不需要任何改动，只换配置即可。

---

## ProviderConfig 字段

| 字段 | 类型 | 说明 | 必填 |
|:---|:---|:---|:---:|
| `provider` | `Provider` | 厂商枚举 | ✅ |
| `api_key` | `String` | API 密钥 | ✅ |
| `base_url` | `String` | API 网关地址；`new` 会自动填充默认值 | 默认构造时 — |
| `model` | `String` | 模型名称（原样透传） | ✅ |
| `dimension` | `Option<usize>` | 向量维度（embed 必填） | embed 时 ✅ |
| `timeout` | `Option<Duration>` | 请求超时（覆盖默认值） | — |

```rust
use std::time::Duration;

let mut cfg = ProviderConfig::new(...);
cfg.timeout = Some(Duration::from_secs(60)); // 超长生成时可调大
```

使用代理、私有网关、区域端点，或阿里云文生图原生路径时，改用 `ProviderConfig::with_base_url` 显式传入 `base_url`。

---

## 从字符串解析厂商

运行时从配置文件读取厂商名时：

```rust
let provider: Provider = "openai".parse()?;   // 不区分大小写
let provider: Provider = "Aliyun".parse()?;
let provider: Provider = "GOOGLE".parse()?;
```

---

## 下一步

- [对话：多轮、系统消息、流式](chat.md)
- [工具调用 / Function Calling](tool-calling.md)
- [各厂商配置详解](providers.md)
- [错误处理](error-handling.md)
