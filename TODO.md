# 后续工作与维护备忘

设计约定以 [设计准则](docs/design-guidelines.md) 为准；能力矩阵与接入说明见根目录 [README](README.md) 与 [docs](docs/README.md)。README 中已标明未支持或需专用契约的能力；落地前须评估 HTTP 形态、Cargo feature 组合与错误语义，再同步实现与文档。

## 当前实现快照

| 维度 | 状态 |
|:--|:--|
| **Chat** | OpenAI 兼容路径（`OpenAI` / `Aliyun` / `Ollama` / `Zhipu`）：`POST …/chat/completions`，Bearer。**Anthropic**：Messages 兼容（`chat/anthropic_compat.rs`，`x-api-key` + `anthropic-version`）。**Google**：Gemini generateContent（`chat/google_gemini.rs`，query `key`）。均为单轮、非流式 JSON。 |
| **Embed** | OpenAI 兼容体：`OpenAI` / `Aliyun` / `Ollama`；智谱专用体：`Zhipu`。**Google**：Gemini `embedContent` / `batchEmbedContents`（`embed/google_gemini.rs`，query `key`）。**Anthropic**：工厂 `Unsupported`（`capability: "embed"`）。 |
| **Rerank** | 仅 `Aliyun`、`Zhipu`；`OpenAI` / `Ollama` / `Anthropic` / `Google` 为 `Unsupported`（非 `ProviderDisabled`）。 |
| **Image** | `OpenAI`、`Aliyun`；未启用对应厂商 feature 时 `OpenAI`/`Aliyun` 为 `ProviderDisabled`；`Ollama` / `Zhipu` / `Anthropic` / `Google` 为 `Unsupported`。 |
| **Audio** | 仅 trait 与工厂占位，`create_*` 始终 `Unsupported`。 |
| **共享 HTTP** | `HttpClient::post_bearer_json`、`post_json_with_headers`、`post_json_query`（见 `client.rs`）。 |
| **工厂穷尽** | `chat` / `embed` / `image` 等与 `rerank` 一致，用互斥 `cfg` 保证 `match` 穷尽，无 `unreachable_patterns` 压制。 |
| **测试** | 主要 HTTP 路径含 wiremock：`openai_compat`、`anthropic_compat`、**`google_gemini`**、`client`（query）、embed / rerank / image 各厂商文件。 |

**CI 与发版**：推送 **`v*`** 标签时 [.github/workflows/cargo-publish.yml](.github/workflows/cargo-publish.yml) 跑 `fmt`、`clippy --all-features -D warnings`、`test --all-features`，通过后 `cargo publish`（Secrets：`CARGO_ACCESS_TOKEN`）。**普通 push / PR 不跑上述检查**。

## 近期可执行项（建议顺序）

1. **发版**：将 [CHANGELOG](CHANGELOG.md) **Unreleased** 中已对用户可见的条目归档到具体版本号；`Cargo.toml` 的 `version` 与标签一致。注意 Unreleased 中含 **Rerank/Image 错误语义调整**，对依赖 `Error` 变体做分支的调用方可能属破坏性变更，semver 取 **0.3.0** 或 **0.2.1** 前须自行判断并写在 CHANGELOG 中。
2. **crate 元数据**：声明 **`rust-version`（MSRV）**；补齐 **`repository`**（按需 **`documentation`** / **`homepage`**）、**`readme`**、**`keywords`** / **`categories`**（与 [包元数据](#包元数据与-cratesio-呈现) 一节一致）。
3. **可选 CI**：在 **pull_request** 或默认分支 **push** 上跑与发版相同的 fmt / clippy / test（全 feature 或与 PR 匹配的子集）。
4. **示例**：新增 **`examples/`**（如 `openai`+`chat`，`anthropic`+`chat`，`google`+`chat`，或 `full` 下各模态一条），便于复制与联调。
5. **P0 补全（按需排期）**：**Anthropic** 的 **embed**（及 **Google** / **Anthropic** 的 **image** 等）若产品需要，按官方 REST 拆独立实现路径、补 wiremock、更新 README 与 [docs/http-api.md](docs/http-api.md)。**Google embed** 已接（`embed/google_gemini.rs`）。

## 发版前清单

- CHANGELOG：Unreleased 归档；新用户可见行为继续记在 Unreleased。
- `cargo doc --all-features --no-deps`，对照 [docs/interfaces.md](docs/interfaces.md) 与 README 矩阵。
- 可选：`cargo package` 检查打包内容。
- **Anthropic**：同步 **`ANTHROPIC_VERSION`**（`chat/anthropic_compat.rs`）与上游 Messages API 要求及 wiremock 断言。

## 包元数据与 crates.io 呈现

`[package]` 已有 `description`，仍可在下一 semver 小版本继续打磨。须补充：**`rust-version`**、**`repository`**、按需 **`readme` / `documentation` / `homepage` / `keywords` / `categories`**。本库未提交 `Cargo.lock`；工作流缓存 key 使用 `Cargo.toml` hash。

## 文档与示例

变更厂商 HTTP 契约、工厂分支或 `Error` 变体时，同步 README 矩阵、[docs/http-api.md](docs/http-api.md)、[docs/rust-api.md](docs/rust-api.md)、设计准则；用户可见行为必须进 CHANGELOG。

仓库尚无 **`examples/`**（见上文近期项）。

## 工程与 CI（可选加深）

- 合并前自动检查：见「近期可执行项」中的 PR CI。
- `actions/cache` 当前为 **v3**；维护窗口可评估 **v4**。

## 测试与可观测性

新增专用请求体或路径时，为该分支补 wiremock（成功体、业务错误、非 JSON 等），避免仅默认 feature 下通过。

源码中 `tracing` 使用面仍较窄；若强调可观测性，可在设计准则或 rustdoc 中说明调用方需自行挂 subscriber。

## 厂商扩展规划

与稳定性、文档、发版元数据一并排期。每新增一家：`Cargo.toml` feature、[`Provider`](src/config.rs) 与 `FromStr`、各模态工厂、wiremock、README、[docs/http-api.md](docs/http-api.md)、[docs/interfaces.md](docs/interfaces.md)、rustdoc。不必一次打通全模态；优先 **chat** 与刚需 **embed**，再 rerank、image。

**接入优先级**（实施前以各平台最新文档为准）：

| 档位 | 对象 | 本库快照 | 说明 |
|:--|:--|:--|:--|
| P0 | **Anthropic、Google（Gemini）、OpenAI** | OpenAI 全矩阵；Anthropic / Google **chat** 已接；**Google embed** 已接；**Anthropic embed**、**rerank**、**image** 未接或 Unsupported | 三家对话已覆盖；Google / Anthropic 其它模态按需 |
| P1 国内 | **智谱、MiniMax、Kimi、阿里云** | 智谱、阿里云已接；MiniMax、Kimi 未接 | 与「其他 API Key」矩阵对齐 |
| P2 聚合 | **New API、OpenRouter** 等 | 未接入 | 多为 OpenAI/Anthropic 兼容；可 feature 或强化「自定义 `base_url`」文档 |
| P3 | Bedrock、Azure、xAI、Groq 等 | 大多未接入 | 按 parity 或客户需求 |

与产品矩阵对齐的厂商表（与 P0–P3 交叉参考）：

| 厂商 | 本库状态 | 建议路径 |
|:--|:--:|:--|
| OpenAI | 已接入 | P0；兼容基准 |
| Anthropic | chat 已接；embed / rerank / image Unsupported | P0 |
| Google（Gemini） | chat、embed 已接；rerank / image Unsupported | P0 |
| 阿里云百炼 | 已接入 | P1 |
| 智谱 GLM | 已接入 | P1 |
| MiniMax | 未接入 | P1 |
| 月之暗面 Kimi | 未接入 | P1 |
| New API | 未接入 | P2 |
| OpenRouter | 未接入 | P2 |
| Ollama | 已接入 | 本地 OpenAI 兼容 |
| DeepSeek | 未接入 | 多为 OpenAI 兼容 + 自定义 `base_url` |
| 火山引擎（豆包） | 未接入 | 以官方文档为准 |
| 硅基流动 | 未接入 | 多为 OpenAI 兼容 |
| 腾讯混元 / 百川 / 零一万物 / 阶跃 / 小米 MiMo / z.ai 等 | 未接入 | 以官方为准 |
| Azure OpenAI | 未接入 | 常可 OpenAI + 自定义 `base_url` |
| Amazon Bedrock | 未接入 | 签名与形态专用 |
| xAI（Grok） | 未接入 | 以官方为准 |
| 自定义 API | — | OpenAI 兼容则文档化 OpenAI + `base_url` |

**Coding Plan / 套餐通道**（鉴权多与 API Key 同类，差异在 endpoint / SKU / 配额；不少 HTTP 对齐 Anthropic Messages 以便 Claude Code）：

| 通道（示例） | 本库状态 | 规划要点 |
|:--|:--:|:--|
| 智谱 / MiniMax / Kimi / 阿里云 Coding Plan | 未单独枚举 | 端点可能与直连不同；协议以厂商为准 |
| Claude Pro / Max | 部分覆盖 | 与 Messages 同形时可 **`anthropic` + `chat` + 套餐 `base_url`** |
| Codex · ChatGPT Plus、GitHub Copilot | 未接入 | 独立评估 |

第三梯队（P3 之后）：Groq、Mistral、Together、Fireworks 等；多数可走 OpenAI 兼容层。

## 中长期能力与架构

在设计准则中目前为**非目标**或须单独设计前不承诺：**自动重试**、**429/5xx 退避**、**共享/注入 `reqwest::Client`**、**流式 chat**、**multipart（语音/大图）**。采纳须新 trait / 客户端路径并更新准则。

**`audio`**：对接具体厂商（常涉 multipart/流式）或长期占位；若对接须更新矩阵与 HTTP 文档。

## 日常维护

矩阵与文档以厂商官方为准；接口变更在 CHANGELOG 提示调用方核对。调整 `Provider` 时维护 `#[non_exhaustive]`、`FromStr`、工厂分支、feature 表与文档全链路。
