# 对话

本文介绍 `ChatProvider` 的全部用法，包括单轮快捷方法、多轮对话、系统消息与流式输出。

## 单轮快捷方法

只需一句话时，用 `chat` / `chat_stream`：

```rust
// 非流式：返回文本字符串
let text = chat.chat("用一句话介绍 Rust").await?;
println!("{text}");

// 流式：返回 ChatStream
let mut stream = chat.chat_stream("用一句话介绍 Rust").await?;
while let Some(item) = stream.next().await {
    if let Some(text) = item?.delta {
        print!("{text}");
    }
}
```

这两个方法内部等价于 `complete` / `complete_stream` 加 `ChatRequest::single_user`，不需要手动构造 `ChatRequest`。

---

## 主路径：ChatRequest

多轮、系统消息、工具调用等场景需要用主路径 `complete` / `complete_stream`，通过 `ChatRequest` 描述请求。

### 最简构造

```rust
use lmkit::ChatRequest;

let request = ChatRequest::single_user("你好");
```

### 完整构造

```rust
use lmkit::{ChatMessage, ChatRequest};

let request = ChatRequest {
    messages: vec![
        ChatMessage::system("你是一个简洁的助手，回答不超过两句话。"),
        ChatMessage::user("Rust 的所有权系统解决了什么问题？"),
    ],
    temperature: Some(0.7),
    max_tokens: Some(512),
    top_p: Some(0.95),
    ..Default::default()
};
```

### ChatRequest 字段

| 字段 | 类型 | 默认值 | 说明 |
|:---|:---|:---|:---|
| `messages` | `Vec<ChatMessage>` | 必填 | 消息历史 |
| `tools` | `Option<Vec<ToolDefinition>>` | `None` | 可用工具列表 |
| `tool_choice` | `Option<ToolChoice>` | `None` | 工具选择策略 |
| `temperature` | `Option<f32>` | 默认 `0.2` | 采样温度，`0.0`–`2.0` |
| `max_tokens` | `Option<u32>` | `None` | 最大生成 token 数 |
| `top_p` | `Option<f32>` | `None` | 核采样概率阈值 |

---

## 系统消息

系统消息放在 `messages` 首位：

```rust
let request = ChatRequest {
    messages: vec![
        ChatMessage::system("你是一个专业的 Rust 工程师，只用代码示例回答问题。"),
        ChatMessage::user("如何用 Rust 读取文件？"),
    ],
    ..Default::default()
};
```

> **Anthropic 注意**：系统消息会自动从 `messages` 中提取为 Messages API 的顶层 `system` 字段，无需特殊处理。

---

## 多轮对话

将完整历史放入 `messages`，模型会基于上下文作答：

```rust
use lmkit::{ChatMessage, ChatRequest, ChatResponse};

// 第一轮
let mut history = vec![
    ChatMessage::user("Rust 的所有权是什么？"),
];
let response: ChatResponse = chat.complete(&ChatRequest {
    messages: history.clone(),
    ..Default::default()
}).await?;

// 把 assistant 回复追加到历史
if let Some(content) = &response.content {
    history.push(ChatMessage::assistant(content));
}

// 第二轮——模型能看到上一轮的上下文
history.push(ChatMessage::user("它和 C++ 的 RAII 有什么区别？"));
let response2 = chat.complete(&ChatRequest {
    messages: history.clone(),
    ..Default::default()
}).await?;
println!("{}", response2.content.unwrap_or_default());
```

---

## 流式对话

`complete_stream` 返回 `ChatStream`（`Pin<Box<dyn Stream<Item = Result<ChatChunk>> + Send>>`），需要 `futures::StreamExt` 驱动：

```toml
[dependencies]
futures = "0.3"
```

```rust
use futures::StreamExt;
use lmkit::{ChatRequest, FinishReason};

let mut stream = chat
    .complete_stream(&ChatRequest::single_user("写一首关于 Rust 的俳句"))
    .await?;

while let Some(item) = stream.next().await {
    let chunk = item?;

    // 文本增量
    if let Some(text) = chunk.delta {
        print!("{text}");
    }

    // 结束原因
    if let Some(reason) = chunk.finish_reason {
        match reason {
            FinishReason::Stop => eprintln!("\n[完成]"),
            FinishReason::Length => eprintln!("\n[超出 max_tokens]"),
            FinishReason::ToolCalls => eprintln!("\n[触发工具调用]"),
            FinishReason::ContentFilter => eprintln!("\n[内容过滤]"),
        }
    }
}
println!();
```

### ChatChunk 字段

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `delta` | `Option<String>` | 文本增量 |
| `tool_call_deltas` | `Option<Vec<ToolCallDelta>>` | 工具调用增量（见[工具调用](tool-calling.md)） |
| `finish_reason` | `Option<FinishReason>` | 结束原因，仅最后一个 chunk 含此字段 |

---

## 采样参数

```rust
let request = ChatRequest {
    messages: vec![ChatMessage::user("写一个创意标题")],
    temperature: Some(1.2),   // 更有创意（高温）
    max_tokens: Some(100),    // 限制长度
    top_p: Some(0.9),
    ..Default::default()
};
```

> `temperature` 未设置时，各厂商实现默认使用 `0.2`。

---

## ChatResponse 字段

`complete` 的返回值：

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `content` | `Option<String>` | 文本回复（工具调用时可能为 `None`） |
| `tool_calls` | `Option<Vec<ToolCall>>` | 工具调用列表（见[工具调用](tool-calling.md)） |
| `finish_reason` | `Option<FinishReason>` | 结束原因 |

---

## 下一步

- [工具调用 / Function Calling](tool-calling.md)
- [各厂商配置详解](providers.md)
- [错误处理](error-handling.md)
