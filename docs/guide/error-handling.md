# 错误处理

lmkit 使用单一的 `Error` 枚举描述所有错误，方便分支处理。

## Error 枚举

```rust
pub enum Error {
    UnknownProvider { name: String },
    ProviderDisabled { provider: String, capability: String },
    Unsupported { provider: String, capability: String },
    MissingConfig { field: String },
    Api { status: u16, message: String },
    Http { source: reqwest::Error },
    Parse { message: String },
    MissingField { field: String },
}
```

---

## 各错误类型含义

| 错误 | 含义 | 何时出现 |
|:---|:---|:---|
| `UnknownProvider` | 无法识别厂商名 | `"xyz".parse::<Provider>()` 失败 |
| `ProviderDisabled` | Cargo feature 未启用 | 用了 `Aliyun` 但没开 `aliyun` feature |
| `Unsupported` | 厂商不支持该能力 | `OpenAI` 调用 rerank |
| `MissingConfig` | 缺必要配置 | embed 没设置 `dimension` |
| `Api` | 远端返回错误 | HTTP 非 2xx，或流内协议错误 |
| `Http` | 网络层错误 | 连接超时、DNS 失败、TLS 错误 |
| `Parse` | 响应解析失败 | JSON 格式不符合预期 |
| `MissingField` | 响应缺字段 | 预期字段不存在 |

---

## 基本错误处理

```rust
use lmkit::{create_chat_provider, Error};

match create_chat_provider(&cfg) {
    Ok(chat) => { /* 使用 provider */ }
    Err(Error::ProviderDisabled { provider, capability }) => {
        eprintln!("请在 Cargo.toml 中启用 {provider} feature（能力: {capability}）");
    }
    Err(Error::Unsupported { provider, capability }) => {
        eprintln!("{provider} 不支持 {capability}");
    }
    Err(e) => eprintln!("其他错误: {e}"),
}
```

---

## 区分 ProviderDisabled 和 Unsupported

这两个错误容易混淆：

**ProviderDisabled** — 编译时没有启用厂商 feature

```rust
// Cargo.toml: features = ["chat"]（缺少 aliyun）
let cfg = ProviderConfig::new(Provider::Aliyun, ...);
create_chat_provider(&cfg)?;
// Error: ProviderDisabled { provider: "aliyun", capability: "chat" }
```

修复：在 `Cargo.toml` 中添加 `aliyun` feature。

**Unsupported** — 厂商本身不支持该能力

```rust
// Cargo.toml: features = ["openai", "rerank"]
let cfg = ProviderConfig::new(Provider::OpenAI, ...);
create_rerank_provider(&cfg)?;
// Error: Unsupported { provider: "openai", capability: "rerank" }
```

修复：换用支持 rerank 的厂商（阿里云或智谱）。

---

## 处理 API 错误

```rust
use lmkit::Error;

match chat.complete(&request).await {
    Ok(response) => println!("{}", response.content.unwrap_or_default()),
    Err(Error::Api { status, message }) => {
        if status == 401 {
            eprintln!("API Key 无效或过期");
        } else if status == 429 {
            eprintln!("请求频率超限，请稍后重试");
        } else {
            eprintln!("API 错误 {status}: {message}");
        }
    }
    Err(Error::Http { source }) => {
        if source.is_timeout() {
            eprintln!("请求超时，可尝试增大 cfg.timeout");
        } else {
            eprintln!("网络错误: {source}");
        }
    }
    Err(e) => eprintln!("未知错误: {e}"),
}
```

---

## 处理流式错误

流式响应中每个 chunk 都可能是错误，需要在循环内处理：

```rust
use futures::StreamExt;

let mut stream = chat.complete_stream(&request).await?;

while let Some(item) = stream.next().await {
    match item {
        Ok(chunk) => {
            if let Some(text) = chunk.delta {
                print!("{text}");
            }
        }
        Err(Error::Api { status, message }) => {
            eprintln!("\n流中断，API 错误 {status}: {message}");
            break;
        }
        Err(e) => {
            eprintln!("\n流中断: {e}");
            break;
        }
    }
}
```

---

## 兼容 std::error::Error

`Error` 实现了 `std::error::Error`，可以直接用 `Box<dyn std::error::Error>` 或 `anyhow`：

```rust
// 直接用 ?
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let response = chat.complete(&request).await?;
    println!("{}", response.content.unwrap_or_default());
    Ok(())
}

// 配合 anyhow
use anyhow::Result;

async fn run() -> Result<()> {
    let response = chat.complete(&request).await?;
    Ok(())
}
```

---

## MissingField 错误

响应缺少预期字段时触发，常见场景：

| 场景 | 错误 |
|:---|:---|
| `chat` / `chat_stream` 调用时响应无文本内容 | `MissingField("response content")` |
| Gemini 工具结果消息未设置 `name` | `MissingField("tool.name")` |

```rust
// 避免 MissingField("tool.name")：使用 tool_with_name
messages.push(ChatMessage::tool_with_name(&call.id, &call.function.name, &result));
```

---

## 下一步

- [各厂商配置详解](providers.md)
- [API 参考](../reference/api.md)
