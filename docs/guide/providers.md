# 各厂商配置

本文列出每个厂商的 `ProviderConfig` 配置模板、支持的能力与注意事项。

---

## OpenAI

```toml
features = ["openai", "chat", "embed", "image"]
```

```rust
let cfg = ProviderConfig::new(
    Provider::OpenAI,
    std::env::var("OPENAI_API_KEY")?,
    "https://api.openai.com/v1",
    "gpt-4o-mini",          // 或 gpt-4o、gpt-3.5-turbo 等
);
```

| 能力 | 支持 | 常用模型 |
|:---|:---:|:---|
| Chat | ✅ | `gpt-4o`、`gpt-4o-mini`、`gpt-3.5-turbo` |
| Embed | ✅ | `text-embedding-3-small`（1536 维）、`text-embedding-3-large`（3072 维） |
| Image | ✅ | `dall-e-3`、`dall-e-2` |
| Rerank | — | — |

**鉴权**：`Authorization: Bearer <API_KEY>`

**兼容性**：任何支持 OpenAI API 格式的代理或网关（如 One API、New API）都可以把 `base_url` 替换为代理地址，`provider` 保持 `OpenAI`。

---

## Anthropic

```toml
features = ["anthropic", "chat"]
```

```rust
let cfg = ProviderConfig::new(
    Provider::Anthropic,
    std::env::var("ANTHROPIC_API_KEY")?,
    "https://api.anthropic.com/v1",
    "claude-3-5-sonnet-20241022",
);
```

| 能力 | 支持 | 常用模型 |
|:---|:---:|:---|
| Chat | ✅ | `claude-3-5-sonnet-20241022`、`claude-3-5-haiku-20241022`、`claude-3-opus-20240229` |
| Embed | — | — |
| Image | — | — |
| Rerank | — | — |

**鉴权**：`x-api-key: <API_KEY>` + `anthropic-version: 2023-06-01`（自动设置）

**系统消息**：会自动从 `messages` 提取为 Messages API 顶层 `system` 字段。

**工具调用**：支持，格式为 `input_schema`（库内部转换）。

---

## Google Gemini

```toml
features = ["google", "chat", "embed"]
```

```rust
let cfg = ProviderConfig::new(
    Provider::Google,
    std::env::var("GOOGLE_API_KEY")?,
    "https://generativelanguage.googleapis.com/v1beta",
    "gemini-1.5-flash",     // 或 gemini-1.5-pro、gemini-2.0-flash 等
);
```

| 能力 | 支持 | 常用模型 |
|:---|:---:|:---|
| Chat | ✅ | `gemini-1.5-flash`、`gemini-1.5-pro`、`gemini-2.0-flash` |
| Embed | ✅ | `text-embedding-004`（768 维） |
| Image | — | — |
| Rerank | — | — |

**鉴权**：URL query 参数 `?key=<API_KEY>`（自动添加）

**工具调用注意**：工具结果消息 **必须** 使用 `ChatMessage::tool_with_name`，否则会报 `MissingField("tool.name")`：

```rust
// ✅ Gemini 必须指定 name
messages.push(ChatMessage::tool_with_name(&call.id, &call.function.name, &result));
```

---

## 阿里云 DashScope

```toml
features = ["aliyun", "chat", "embed", "rerank", "image"]
```

Chat / Embed 使用兼容网关：

```rust
let cfg = ProviderConfig::new(
    Provider::Aliyun,
    std::env::var("DASHSCOPE_API_KEY")?,
    "https://dashscope.aliyuncs.com/compatible-mode/v1",
    "qwen-turbo",
);
```

Rerank / Image 使用独立网关：

```rust
// Rerank
let mut cfg = ProviderConfig::new(
    Provider::Aliyun,
    std::env::var("DASHSCOPE_API_KEY")?,
    "https://dashscope.aliyuncs.com/api/v1",  // 独立网关
    "gte-rerank",
);

// Image
let mut cfg = ProviderConfig::new(
    Provider::Aliyun,
    std::env::var("DASHSCOPE_API_KEY")?,
    "https://dashscope.aliyuncs.com/api/v1",  // 独立网关
    "wanx-v1",
);
```

| 能力 | 支持 | 常用模型 |
|:---|:---:|:---|
| Chat | ✅ | `qwen-turbo`、`qwen-plus`、`qwen-max`、`qwen-long` |
| Embed | ✅ | `text-embedding-v3`（1024 维）、`text-embedding-v2`（1536 维） |
| Rerank | ✅ | `gte-rerank`、`gte-rerank-v2` |
| Image | ✅ | `wanx-v1`、`wanx2.1-t2i-turbo` |

**鉴权**：`Authorization: Bearer <API_KEY>`

---

## Ollama（本地）

```toml
features = ["ollama", "chat", "embed"]
```

```rust
let cfg = ProviderConfig::new(
    Provider::Ollama,
    String::new(),              // 本地无需 API Key
    "http://127.0.0.1:11434/v1",
    "llama3.2",                 // 或任何已 pull 的模型
);
```

| 能力 | 支持 | 说明 |
|:---|:---:|:---|
| Chat | ✅ | 所有本地模型 |
| Embed | ✅ | 需 pull 支持 embedding 的模型 |
| Image | — | — |
| Rerank | — | — |

**鉴权**：无（本地服务，`api_key` 传空字符串）

**模型管理**：

```bash
# 拉取模型
ollama pull llama3.2
ollama pull nomic-embed-text

# 列出已有模型
ollama list
```

**维度**：Ollama embed 模型维度因模型而异，请查看模型文档后设置 `cfg.dimension`。

---

## 智谱 GLM

```toml
features = ["zhipu", "chat", "embed", "rerank"]
```

```rust
let cfg = ProviderConfig::new(
    Provider::Zhipu,
    std::env::var("ZHIPU_API_KEY")?,
    "https://open.bigmodel.cn/api/paas/v4",
    "glm-4-flash",
);
```

| 能力 | 支持 | 常用模型 |
|:---|:---:|:---|
| Chat | ✅ | `glm-4`、`glm-4-flash`、`glm-4-air` |
| Embed | ✅ | `embedding-3`（2048 维） |
| Rerank | ✅ | `rerank` |
| Image | — | — |

**鉴权**：`Authorization: Bearer <API_KEY>`

---

## OpenAI 兼容厂商

月之暗面（Kimi）、DeepSeek、MiniMax 等使用 OpenAI 兼容 API 的厂商，无需新 feature，只需指定 `Provider::OpenAI` 并换 `base_url`：

```rust
// 月之暗面 Kimi
let cfg = ProviderConfig::new(
    Provider::OpenAI,
    std::env::var("MOONSHOT_API_KEY")?,
    "https://api.moonshot.cn/v1",
    "moonshot-v1-8k",
);

// DeepSeek
let cfg = ProviderConfig::new(
    Provider::OpenAI,
    std::env::var("DEEPSEEK_API_KEY")?,
    "https://api.deepseek.com/v1",
    "deepseek-chat",
);

// MiniMax
let cfg = ProviderConfig::new(
    Provider::OpenAI,
    std::env::var("MINIMAX_API_KEY")?,
    "https://api.minimax.chat/v1",
    "abab6.5s-chat",
);
```

---

## 下一步

- [错误处理](error-handling.md)
- [API 参考](../reference/api.md)
