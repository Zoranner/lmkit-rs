# Changelog

## Unreleased

### Added

- **Google（Gemini）**：新增 Cargo feature `google` 与 `Provider::Google`。`chat` 模态实现 **Gemini generateContent**（`POST {base_url}/models/{model}:generateContent`，API Key 为 query 参数 `key`；与 [官方 REST 文档](https://ai.google.dev/api/rest/v1beta/models/generateContent) 及示例 curl 一致）。请求体 `contents` 为单条仅含 `parts`（与官方单轮示例一致；[`Content.role`](https://ai.google.dev/api/caching#Content) 为可选故省略）。若 HTTP 200 但无 `candidates`（如安全拦截），解析 `promptFeedback` 并经 `Error::Parse` 返回摘要。`embed` / `rerank` / `image` 工厂返回 `Unsupported`。共享客户端新增 `HttpClient::post_json_query`。对话实现见 `chat/google_gemini.rs`（`GoogleGeminiChat`）。

- **Anthropic**：新增 Cargo feature `anthropic` 与 `Provider::Anthropic`。`chat` 模态实现 **Anthropic Messages 兼容**（参考 [官方 Messages API](https://docs.anthropic.com/en/api/messages)：`POST {base_url}/messages`，`x-api-key` 与 `anthropic-version`），便于同一套代码对接官方及兼容网关（含常见 Coding Plan 通道，取决于对方是否遵循同一契约）。`embed` / `rerank` / `image` 工厂返回 `Unsupported`。共享客户端新增 `HttpClient::post_json_with_headers`。对话实现按文件拆分：`chat/openai_compat.rs`、`chat/anthropic_compat.rs`（`AnthropicCompatChat`）。

- GitHub Actions：推送 `v*` 标签时运行 `fmt`、`clippy`（全 feature）、`test`（全 feature），通过后发布至 crates.io（需配置 `CARGO_ACCESS_TOKEN`）；工具链使用 `dtolnay/rust-toolchain@stable`（替代已归档的 `actions-rs/toolchain`）。

### Changed

- **Rerank**：`create_rerank_provider` 对 `OpenAI` / `Ollama` 现返回 `Error::Unsupported`（`capability: "rerank"`），而不再返回 `Error::ProviderDisabled`，以区分「未启用厂商 feature」与「该厂商在本模态无实现」。未启用 `aliyun` / `zhipu` feature 时仍选阿里云 / 智谱的，仍为 `ProviderDisabled`（行为未变）。若依赖旧错误变体区分 OpenAI/Ollama 重排序，请改为匹配 `Unsupported`。

- **Image**：`create_image_provider` 在启用 `image` 但未启用 `openai` / `aliyun` 时，对 `OpenAI` / `Aliyun` 现返回 `ProviderDisabled`（与重排序、设计准则一致）；此前会落入 `Unsupported`。`Ollama` / `Zhipu` 仍为 `Unsupported`（`capability: "image"`）。

- **Chat / Embed / Image 工厂**：去掉 `#[allow(unreachable_patterns)]`，用与 `rerank` 相同的 `cfg` 互斥分支保证 `match` 穷尽，便于在全 feature 下依赖编译器检查。
