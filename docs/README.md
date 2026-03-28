# lmkit 文档

本目录是 lmkit 的完整文档，面向不同受众的阅读路径如下。

---

## 新用户：从这里开始

> 目标：安装、发出第一次请求、了解核心概念

1. [快速上手](guide/getting-started.md) — 添加依赖、第一次对话与向量化、切换厂商
2. [对话](guide/chat.md) — 单轮、多轮、系统消息、流式输出、采样参数
3. [工具调用](guide/tool-calling.md) — Function Calling、流式 tool_calls、多轮工具链
4. [各厂商配置](guide/providers.md) — 每个厂商的 Key、网关地址、模型名与注意事项

---

## 按能力查阅


| 能力    | 文档                                             | 支持厂商                              |
| ----- | ---------------------------------------------- | --------------------------------- |
| 对话    | [guide/chat.md](guide/chat.md)                 | 全部                                |
| 工具调用  | [guide/tool-calling.md](guide/tool-calling.md) | 全部                                |
| 文本向量化 | [guide/embed.md](guide/embed.md)               | OpenAI、Aliyun、Google、Ollama、Zhipu |
| 文本重排序 | [guide/rerank.md](guide/rerank.md)             | Aliyun、Zhipu                      |
| 文生图   | [guide/image.md](guide/image.md)               | OpenAI、Aliyun                     |


---

## 深入参考

> 目标：查类型定义、理解接口约定、排查问题


| 文档                                                         | 内容                        |
| ---------------------------------------------------------- | ------------------------- |
| [reference/api.md](reference/api.md)                       | Trait 定义、工厂函数、所有公开类型、错误枚举 |
| [reference/http-endpoints.md](reference/http-endpoints.md) | 各厂商的请求/响应 JSON 格式、路径、鉴权方式 |
| [guide/error-handling.md](guide/error-handling.md)         | 每种错误的含义与处理模式              |


---

## 维护者与贡献者

> 目标：理解设计决策、接入新厂商、参与开发


| 文档                                                     | 内容                 |
| ------------------------------------------------------ | ------------------ |
| [reference/design.md](reference/design.md)             | 库的边界、架构原则、演进约定     |
| [reference/contributing.md](reference/contributing.md) | 接入新厂商的步骤、测试规范、发版流程 |


---

## 快速定位


| 我想…                         | 去哪里                                         |
| --------------------------- | ------------------------------------------- |
| 第一次使用这个库                    | [快速上手](guide/getting-started.md)            |
| 做多轮对话 / 带系统消息               | [对话 · 多轮对话](guide/chat.md#多轮对话)             |
| 实现流式输出                      | [对话 · 流式对话](guide/chat.md#流式对话)             |
| 让模型调用函数                     | [工具调用](guide/tool-calling.md)               |
| 生成向量 / RAG 检索               | [文本向量化](guide/embed.md)                     |
| 搜索结果精排                      | [文本重排序](guide/rerank.md)                    |
| 文生图                         | [文生图](guide/image.md)                       |
| 切换到阿里云 / Gemini / Anthropic | [各厂商配置](guide/providers.md)                 |
| 理解错误类型                      | [错误处理](guide/error-handling.md)             |
| 查 Rust 类型和 trait            | [API 参考](reference/api.md)                  |
| 了解某厂商的 HTTP 请求格式            | [HTTP 端点](reference/http-endpoints.md)      |
| 接入一个新厂商                     | [贡献指南](reference/contributing.md)           |
| 理解库的设计决策                    | [设计准则](reference/design.md)                 |
| 本地生成 rustdoc                | `cargo doc --all-features --no-deps --open` |


