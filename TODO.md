# 开发计划

与当前仓库对齐的待办与路线说明。实现细节以 `docs/reference/design.md` 为准。

---

## 示例与文档卫生

- [x] 补充 `examples/`：除 `stream_chat` 外，增加「换厂商同一套代码」等非流式示例（可与 README 快速开始呼应）
- [x] `docs/reference/contributing.md` 中克隆地址仍为占位 `your-repo`，有公开仓库后改为真实 URL

---

## 小版本发包准备

当前预备发包版本为 `0.1.1`。`CHANGELOG.md` 的 Unreleased 中包含 `ProviderConfig::new` 参数变化和 `ChatStream` item 类型变化，发包前需要在迁移说明中明确标注，避免调用方误判升级成本。

- [x] 确认本轮范围只包含已经落在 Unreleased 的能力：默认 `base_url`、`ChatEvent` 枚举、工具调用增量合并、`request_id`、`max_concurrent`、错误分类辅助方法、`ChatRequest::preset`
- [x] 补一条非流式示例，展示同一业务代码通过 `ProviderConfig` 切换 OpenAI / 阿里云 / Ollama
- [x] 全面检查 README、README.zh、docs guide、API reference 与 rustdoc，确保 `ProviderConfig::new` / `with_base_url`、`ChatEvent`、`ChatStream` 的新签名没有旧写法残留
- [x] 将 `Cargo.toml` 的 `version` 更新为 `0.1.1`
- [x] 将 `CHANGELOG.md` 的 `[Unreleased]` 归档为 `[0.1.1] - YYYY-MM-DD`，并保留新的空 `[Unreleased]`
- [x] 本地执行 `cargo fmt --all -- --check`
- [x] 本地执行 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 本地执行 `cargo test --all-features`
- [x] 本地执行 `cargo doc --all-features --no-deps`
- [ ] 本地执行 `cargo package --allow-dirty` 检查包内容；正式发包前在干净工作区重新执行不带 `--allow-dirty` 的 `cargo package`
- [ ] 确认 GitHub 仓库已配置 `CARGO_ACCESS_TOKEN`
- [ ] 提交版本更新，创建 `v0.1.1` 标签，并推送标签触发 `.github/workflows/cargo-publish.yml`

---

## 功能补全（按需求排期）

- [x] **base_url 默认值** — 为各厂商（OpenAI / Anthropic / Google / 阿里云 / 智谱 / Ollama）在 `ProviderConfig` 中提供默认 `base_url`，简化用户配置
- [ ] **Anthropic Embed** — 官方若提供可用 HTTP API 再接入（当前矩阵为「—」）
- [ ] **Google / Anthropic Image** — 视 API 形态评估独立实现或文档化「仅 OpenAI 兼容路径」
- [ ] **Audio** — `TranscriptionProvider` / `SpeechProvider` 仍为占位；实装通常涉及 multipart/流式，需单独设计后再动工厂与准则文档

---

## 厂商接入 backlog

| 优先级 | 厂商 | 状态 | 说明 |
|:---|:---|:---|:---|
| P0 | OpenAI | ✅ 已接入 | 兼容基准 |
| P0 | Anthropic | ✅ Chat 已接 | Embed / Rerank / Image 未接 |
| P0 | Google Gemini | ✅ Chat / Embed 已接 | Rerank / Image 未接 |
| P1 | 阿里云 | ✅ 已接入 | 全模态（chat / embed / rerank / image） |
| P1 | 智谱 | ✅ 已接入 | Chat / Embed / Rerank |
| P1 | Ollama | ✅ 已接入 | 本地 OpenAI 兼容 |
| P1 | MiniMax | 未接入 | 以官方文档为准 |
| P1 | Kimi | 未接入 | 以官方文档为准 |
| P2 | OpenRouter | 未接入 | 多为 OpenAI 兼容 |
| P2 | New API | 未接入 | 多为 OpenAI 兼容 |
| P3 | DeepSeek | 未接入 | OpenAI 兼容 + 自定义 `base_url` |
| P3 | Azure OpenAI | 未接入 | 同上 |
| P3 | 硅基流动 | 未接入 | 多为 OpenAI 兼容 |
| P3 | 火山引擎 | 未接入 | 以官方文档为准 |
| P3 | Bedrock / xAI / Groq 等 | 未接入 | 按需评估 |

多数 OpenAI 兼容厂商无需新增代码，在 README / `http-endpoints` 中说明 `base_url` 与鉴权即可。

---

## 能力一览（与文档矩阵对齐）

| 能力 | 状态 | 说明 |
|:---|:---|:---|
| Chat | ✅ | 已列厂商均支持（含非流式与流式 `complete_stream`，`ChatEvent` 枚举事件模型） |
| Embed | ✅ | Anthropic 除外 |
| Rerank | ✅ | 仅阿里云、智谱 |
| Image | ✅ | 仅 OpenAI、阿里云 |
| Audio | 占位 | Trait 与工厂已存在，无远端实现 |

---

## 非目标（未写入新 trait 前不承诺）

- 自动重试与 429 退避（`is_retryable()` 已提供判断依据，策略由调用方实现）
- 共享 `reqwest::Client` 连接池策略
- 语音 multipart
- 按厂商维护模型白名单
- 发请求前按模型名本地拦截
- Semaphore 实例化（`max_concurrent` 只携带配置值，调度由调用方负责）
