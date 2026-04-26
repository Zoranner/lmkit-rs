# Changelog

所有用户可见的变更都记录在此文档。

格式基于 [Keep a Changelog](https://keepachangelog.com/)，版本号遵循 [SemVer](https://semver.org/)。

---

## [Unreleased]

---

## [0.1.1] - 2026-04-26

### 新增

- **默认 `base_url`** — `ProviderConfig::new(...)` 直接使用内置厂商常用 API 网关，显式网关使用 `ProviderConfig::with_base_url(...)`。
- **`ChatEvent` 枚举** — 替代原扁平 `ChatChunk` 结构体，流式事件现为 `Delta(String)` / `ToolCallDelta(Vec<ToolCallDelta>)` / `Finish(FinishReason)` 三个变体，消除三字段均可为 `None` 的歧义。
- **`merge_tool_call_deltas`** — 将流式 `ToolCallDelta` 按 `index` 合并为完整 `ToolCall` 列表的工具函数。
- **`ChatResponse::request_id`** — 从响应头提取 request ID（OpenAI 兼容：`x-request-id`；Anthropic：`request-id`；Gemini：始终 `None`）。
- **`ProviderConfig::max_concurrent`** — 携带调用方期望的 provider 级并发上限提示（调度由调用方负责）。
- **`Error::is_retryable()` / `requires_human()`** — 错误分类辅助方法，供调用方实现重试策略。
- **`ChatRequest::preset`** — `RequestPreset` 枚举（`Planning` / `Execution`），未显式设置 `temperature` 时按 preset 选取默认值。

### 破坏性变更

- `ProviderConfig::new` 改为三参数默认网关构造；需要自定义 `base_url` 的调用方改用 `ProviderConfig::with_base_url`。
- `ChatStream` 的 item 类型从 `Result<ChatChunk>` 改为 `Result<ChatEvent>`，调用方需更新 match 逻辑。

---

## [0.1.0] - 2026-03-22

### 新增

- **Chat 流式** — `ChatProvider::chat_stream` 返回 `ChatStream`，每项为 `ChatEvent`（`Delta` / `ToolCallDelta` / `Finish`）。OpenAI 兼容（OpenAI、阿里云、Ollama、智谱）、Anthropic Messages、Google Gemini `streamGenerateContent` 均支持 SSE。示例见 `examples/stream_chat.rs`。
- **Google Gemini Chat** — 新增 `google` feature 和 `Provider::Google`。实现 `generateContent` 端点（API Key 作为 query 参数 `key`）。若 HTTP 200 但 `candidates` 为空，返回含 `promptFeedback` 摘要的解析错误。
- **Google Gemini Embed** — 实现 `embedContent`（单条）和 `batchEmbedContents`（批量）。
- **Anthropic Chat** — 新增 `anthropic` feature 和 `Provider::Anthropic`。实现 Messages 兼容端点（`x-api-key` + `anthropic-version` 头），支持官方及兼容网关。
- **GitHub Actions CI** — 推送 `v*` 标签时自动运行 fmt、clippy、test，通过后发布到 crates.io。

### 变更

- **Rerank 错误语义** — `create_rerank_provider` 对 `OpenAI`/`Ollama` 现返回 `Unsupported` 而非 `ProviderDisabled`，以区分「厂商不支持」与「feature 未启用」。
- **Image 错误语义** — `create_image_provider` 对未启用 feature 的 `OpenAI`/`Aliyun` 现返回 `ProviderDisabled`，行为与 Rerank 一致。
- **工厂穷尽检查** — 去掉 `#[allow(unreachable_patterns)]`，用 cfg 互斥分支保证 match 穷尽。
- **actions/cache 升级** — CI workflow 中 `actions/cache` 从 v3 升级到 v4。
