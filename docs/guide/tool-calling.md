# 工具调用

工具调用（Function Calling）让模型在回复时触发外部函数。本文介绍从定义工具到完成多轮工具链的完整流程。

## 概念

1. **定义工具** — 告诉模型有哪些函数可以调用，以及参数格式（JSON Schema）
2. **模型决策** — 模型返回 `finish_reason: ToolCalls`，并附上调用参数
3. **执行函数** — 业务代码拿到参数，实际执行对应逻辑
4. **反馈结果** — 将执行结果以 `Role::Tool` 消息追加到历史，再次请求模型
5. **最终回复** — 模型根据工具结果生成最终文本

---

## 定义工具

使用 `ToolDefinition::function` 或 `ToolDefinition::function_with_description`：

```rust
use lmkit::{ToolDefinition};
use serde_json::json;

// 无描述
let tool = ToolDefinition::function(
    "get_weather",
    json!({
        "type": "object",
        "properties": {
            "location": {
                "type": "string",
                "description": "城市名称，如「北京」"
            },
            "unit": {
                "type": "string",
                "enum": ["celsius", "fahrenheit"],
                "description": "温度单位"
            }
        },
        "required": ["location"]
    }),
);

// 带描述（推荐）
let tool = ToolDefinition::function_with_description(
    "get_weather",
    "查询指定城市的当前天气",
    json!({
        "type": "object",
        "properties": {
            "location": { "type": "string", "description": "城市名称" },
            "unit": {
                "type": "string",
                "enum": ["celsius", "fahrenheit"]
            }
        },
        "required": ["location"]
    }),
);
```

---

## 非流式工具调用

```rust
use lmkit::{ChatMessage, ChatRequest, FinishReason, ToolChoice};
use serde_json::json;

// 1. 定义工具并发送请求
let request = ChatRequest {
    messages: vec![
        ChatMessage::user("北京今天天气怎么样？"),
    ],
    tools: Some(vec![
        ToolDefinition::function_with_description(
            "get_weather",
            "查询指定城市的当前天气",
            json!({
                "type": "object",
                "properties": {
                    "location": { "type": "string" }
                },
                "required": ["location"]
            }),
        ),
    ]),
    tool_choice: Some(ToolChoice::Auto),  // 让模型决定是否调用
    ..Default::default()
};

let response = chat.complete(&request).await?;

// 2. 检查是否触发了工具调用
if response.finish_reason == Some(FinishReason::ToolCalls) {
    let tool_calls = response.tool_calls.unwrap_or_default();
    for call in &tool_calls {
        println!("调用工具: {}", call.function.name);
        println!("参数: {}", call.function.arguments);
    }
}
```

### ToolCall 字段

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `id` | `String` | 调用 ID，反馈结果时需要带回 |
| `function.name` | `String` | 函数名称 |
| `function.arguments` | `String` | JSON 字符串格式的参数 |

---

## 完整多轮工具链

执行工具后，将结果追加到历史，让模型生成最终回复：

```rust
use lmkit::{ChatMessage, ChatRequest, FinishReason};
use serde_json::{json, Value};

async fn run_agent(chat: &dyn ChatProvider) -> Result<String, Box<dyn std::error::Error>> {
    let weather_tool = ToolDefinition::function_with_description(
        "get_weather",
        "查询指定城市的当前天气",
        json!({
            "type": "object",
            "properties": {
                "location": { "type": "string" }
            },
            "required": ["location"]
        }),
    );

    // 第一轮：用户提问
    let mut messages = vec![
        ChatMessage::user("北京今天天气怎么样？"),
    ];

    let response = chat.complete(&ChatRequest {
        messages: messages.clone(),
        tools: Some(vec![weather_tool.clone()]),
        tool_choice: Some(ToolChoice::Auto),
        ..Default::default()
    }).await?;

    // 如果模型请求调用工具
    if response.finish_reason == Some(FinishReason::ToolCalls) {
        // 把 assistant 的工具调用消息追加到历史
        if let Some(content) = &response.content {
            messages.push(ChatMessage::assistant(content));
        }

        // 执行工具并追加结果
        let tool_calls = response.tool_calls.unwrap_or_default();
        for call in &tool_calls {
            let result = execute_tool(&call.function.name, &call.function.arguments).await;

            // OpenAI 兼容：用 tool；Gemini 需要 tool_with_name
            messages.push(ChatMessage::tool_with_name(
                &call.id,
                &call.function.name,  // Gemini 必须设置 name
                &result,
            ));
        }

        // 第二轮：让模型基于工具结果生成最终回复
        let final_response = chat.complete(&ChatRequest {
            messages,
            tools: Some(vec![weather_tool]),
            ..Default::default()
        }).await?;

        return Ok(final_response.content.unwrap_or_default());
    }

    Ok(response.content.unwrap_or_default())
}

async fn execute_tool(name: &str, arguments: &str) -> String {
    match name {
        "get_weather" => {
            let args: Value = serde_json::from_str(arguments).unwrap_or_default();
            let location = args["location"].as_str().unwrap_or("未知");
            format!("{location} 今天晴，气温 18°C，东风 3 级。")
        }
        _ => format!("未知工具: {name}"),
    }
}
```

---

## 流式工具调用

流式模式下工具调用通过 `ToolCallDelta` 增量返回，需要手动拼接参数：

```rust
use lmkit::{ChatRequest, FinishReason, ToolCallDelta};
use std::collections::HashMap;

let mut stream = chat
    .complete_stream(&ChatRequest {
        messages: vec![ChatMessage::user("北京今天天气怎么样？")],
        tools: Some(vec![weather_tool]),
        tool_choice: Some(ToolChoice::Auto),
        ..Default::default()
    })
    .await?;

// 按 index 聚合增量
let mut calls: HashMap<u32, ToolCallAccumulator> = HashMap::new();

while let Some(item) = stream.next().await {
    let chunk = item?;

    if let Some(text) = &chunk.delta {
        print!("{text}");
    }

    if let Some(deltas) = chunk.tool_call_deltas {
        for delta in deltas {
            let acc = calls.entry(delta.index).or_default();
            if let Some(id) = delta.id {
                acc.id = id;
            }
            if let Some(name) = delta.function_name {
                acc.name = name;
            }
            if let Some(args) = delta.function_arguments {
                acc.arguments.push_str(&args);
            }
        }
    }

    if chunk.finish_reason == Some(FinishReason::ToolCalls) {
        // 流结束后处理完整的工具调用
        for (_, acc) in &calls {
            println!("\n调用: {} ({})", acc.name, acc.id);
            println!("参数: {}", acc.arguments);
        }
    }
}

#[derive(Default)]
struct ToolCallAccumulator {
    id: String,
    name: String,
    arguments: String,
}
```

### ToolCallDelta 字段

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `index` | `u32` | 工具调用序号（并行调用时区分） |
| `id` | `Option<String>` | 调用 ID，首个 delta 含此字段 |
| `function_name` | `Option<String>` | 函数名，首个 delta 含此字段 |
| `function_arguments` | `Option<String>` | 参数片段，需逐片追加拼接 |

---

## ToolChoice

| 值 | 行为 |
|:---|:---|
| `ToolChoice::Auto` | 模型自行决定是否调用（推荐） |
| `ToolChoice::None` | 禁止调用任何工具 |
| `ToolChoice::Required` | 强制调用工具 |
| `ToolChoice::Tool(name)` | 强制调用指定工具 |

---

## Gemini 的特殊要求

Google Gemini 的 `functionResponse` 要求必须携带函数名。**使用 `ChatMessage::tool_with_name` 而非 `ChatMessage::tool`**，否则会收到 `MissingField("tool.name")` 错误：

```rust
// ❌ 对 Gemini 无效
messages.push(ChatMessage::tool(&call.id, &result));

// ✅ 通用写法（兼容所有厂商）
messages.push(ChatMessage::tool_with_name(&call.id, &call.function.name, &result));
```

---

## 下一步

- [向量化](embed.md)
- [各厂商配置详解](providers.md)
- [错误处理](error-handling.md)
