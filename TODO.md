# 后续工作与维护备忘

本文件跟踪实现缺口、工程化、厂商扩展与发版检查。设计约定以 [设计准则](docs/design-guidelines.md) 为准；能力矩阵与接入说明见根目录 [README](README.md)。README 中已标明未支持或需专用契约的能力，若要落地须先评估 HTTP 形态、Cargo feature 组合与错误语义，再落到实现与文档。

## 当前状态摘要

在对应 feature 组合下，**对话**（含 OpenAI 兼容与 **Anthropic Messages**）、**向量、重排序、文生图**均已对接 HTTP；共享的 `HttpClient::post_bearer_json` 与 `post_json_with_headers` 与各厂商分支配合 wiremock 或模块内测试，覆盖了成功体、部分 4xx、解析失败与请求体验证。`docs/` 下接口索引、HTTP 汇总、Rust API 摘要与设计准则已齐；crate 与各子模块 rustdoc 说明了 feature 边界与 `ProviderDisabled` / `Unsupported` 的划分（`chat` / `embed` / `image` 工厂用互斥 `cfg` 保证 `match` 穷尽，与 `rerank` 一致）。

**音频（`audio`）**仅有 trait 与工厂占位，创建函数始终返回 `Unsupported`，不发起请求。

**CI 与发版**：推送 **`v*`** 标签时由 [.github/workflows/cargo-publish.yml](.github/workflows/cargo-publish.yml) 执行 `cargo fmt --check`、`cargo clippy --all-targets --all-features -D warnings`、`cargo test --all-features`，通过后 `cargo publish`（仓库 Secrets 需配置 `CARGO_ACCESS_TOKEN`）。工作流使用 `dtolnay/rust-toolchain@stable`，`permissions: contents: read`。本库未提交 `Cargo.lock`，缓存 key 使用 `Cargo.toml` 的 hash。**普通 push / PR 仍不跑上述检查**，日常依赖本地或将来另加 workflow。

## 发版前建议优先处理

下一版打标签前，建议把下面几项当作「发版清单」逐项勾掉。

将 [CHANGELOG](CHANGELOG.md) 里 **Unreleased** 中已对用户可见的条目归档到具体版本号下，并继续在 Unreleased 累积新变更。当前 crate 版本为 **0.2.0**，若该版本已对外发布过而记录里从未出现过对应版本区块，可补写一条 **0.2.0** 历史段落，避免与后续版本对比时断层。

`Cargo.toml` 的 **`version`** 必须与即将推送的标签一致（例如 `v0.2.1` 对应 `0.2.1`）。

发版前执行 `cargo doc --all-features --no-deps`，核对公开 trait、工厂与类型在各 feature 下的可见性，并与 [docs/interfaces.md](docs/interfaces.md)、README 矩阵对照。

## 包元数据与 crates.io 呈现

当前 `[package]` 仍较精简，建议在正式发布或下一次 semver 小版本时补齐，便于依赖方与 docs.rs。

声明 **`rust-version`（MSRV）**，并在 README 或 [docs/rust-api.md](docs/rust-api.md) 中写明，与 docs.rs 构建预期一致。

补齐 **`repository`**（及按需 **`documentation`** / **`homepage`**），并视情况增加 **`readme`**（指向根 README，改善 crates.io 展示）、**`keywords`** / **`categories`**，减少「空主页」观感。

## 文档与示例一致性

变更厂商 HTTP 契约、工厂分支或 `Error` 变体时，同步 README 矩阵、[docs/http-api.md](docs/http-api.md)、[docs/rust-api.md](docs/rust-api.md) 与设计准则中相关表述；用户可见行为变化必须进 CHANGELOG。

仓库尚无 **`examples/`** 目录。可增加最小可运行示例（例如仅 `openai`+`chat`，或 `full` 下各模态一条），与单测互补，方便接入方复制与人工联调。

## 工程与 CI（可选加深）

若希望合并前自动发现问题，可新增仅在 **pull_request** 或 **push 到默认分支** 上跑与发版相同的 `fmt` / `clippy` / `test --all-features`（或与 PR 体量匹配的子集），与现有「仅标签触发 publish」并存。

工作流里 `actions/cache` 目前为 **v3**；非紧急，可在维护窗口评估升级到 **v4** 等当前推荐版本。

## 测试与可观测性

现有测试已覆盖各模态主要路径；若某厂商分支与 OpenAI 兼容实现完全复用同一解析逻辑，wiremock 可能已间接覆盖。后续若新增专用请求体或路径，应为该分支补充固定响应用例，避免只在默认 feature 下「碰巧通过」。

源码中仅在少数路径使用 `tracing`（如部分 rerank / embed 日志）。若希望调用方可观测，可在设计准则或 rustdoc 中简短说明「可选接入 tracing subscriber」，避免依赖方误以为默认有结构化日志输出。

## 厂商扩展规划

用户会自然拿上层产品里「其他 API Key」那张矩阵当心理预期；本 crate 长期只覆盖其中少数几家时，Rust 侧容易反复自己补封装。厂商扩展应与稳定性、文档和发版元数据一起作为近期主线，在路线图或 issue 里按下面**优先级**与全表分批关单。每新增一家：`Cargo.toml` 厂商 feature、扩展 [`Provider`](src/config.rs) 与 `FromStr`、各模态工厂分支与 wiremock；并同步 README、[docs/http-api.md](docs/http-api.md)、[docs/interfaces.md](docs/interfaces.md) 与 rustdoc。不必一次打通全模态，优先 **chat** 与刚需 **embed**，再 rerank、image。

**接入优先级**（按调用热度与产品策略，实施前仍以各平台最新文档为准）：

| 档位 | 对象 | 本库快照 | 说明 |
|:--|:--|:--|:--|
| P0 | **Anthropic、Google（Gemini / AI Studio）、OpenAI** 官方 API | OpenAI 已接入；Anthropic、Google 未接入 | 全球侧最火的一类，Agent / IDE 生态默认会碰；Anthropic 为 Messages 等专用契约，Google 为生成式接口，应最先补齐 |
| P1 国内 | **智谱 GLM、MiniMax、Kimi、阿里云** | 智谱、阿里云已接入；MiniMax、Kimi 未接入 | 与国内付费与「其他 API Key」矩阵对齐；MiniMax、Kimi 尽快跟上 |
| P2 通用聚合 | **New API、OpenRouter** 等转发 / 聚合网关 | 未接入 | 一端对接多后端，常见 OpenAI 或 Anthropic 兼容形态，适合独立 feature 或「自定义 `base_url` + 兼容协议」文档与示例 |
| P3 | 其余厂商、海外 Bedrock / Azure / xAI 等及第二梯队托管 | 大多未接入 | 与矩阵 parity 或客户需求再排 |

下表与常见产品矩阵对齐（与 P0–P3 交叉参考）；「建议路径」以各平台现行 HTTP 文档为准。

| 厂商（对齐「其他 API Key」） | 本库状态 | 建议路径 |
|:--|:--:|:--|
| OpenAI | 已接入 | P0；作兼容层基准 |
| Anthropic | 已接入（chat，Anthropic Messages **兼容**）；embed / rerank / image 为 `Unsupported` | P0；官方与兼容网关 / Coding Plan（同契约）共用 `base_url`；其它模态待评估 |
| Google（Gemini / AI Studio） | 未接入 | P0；生成式接口，专用实现 |
| 阿里云百炼 | 已接入 | P1；已有专用路径（如 rerank、文生图） |
| 智谱 GLM | 已接入 | P1；部分模态为专用契约 |
| MiniMax | 未接入 | P1；以官方文档为准 |
| 月之暗面 Kimi | 未接入 | P1；多为 OpenAI 兼容 |
| New API | 未接入 | P2；聚合网关，协议以部署配置为准 |
| OpenRouter | 未接入 | P2；多为 OpenAI 兼容 |
| Ollama | 已接入 | 本地 OpenAI 兼容形态 |
| DeepSeek | 未接入 | 多为 OpenAI 兼容，独立 feature |
| 火山引擎（豆包） | 未接入 | 以官方文档为准 |
| 硅基流动 | 未接入 | 多为 OpenAI 兼容 |
| 腾讯混元 | 未接入 | 以官方文档为准 |
| 百川智能 | 未接入 | 以官方文档为准 |
| 零一万物 | 未接入 | 以官方文档为准 |
| 阶跃星辰 | 未接入 | 以官方文档为准 |
| 小米 MiMo | 未接入 | 以官方文档为准 |
| z.ai | 未接入 | 以官方文档为准 |
| Azure OpenAI | 未接入 | 常与 OpenAI 同形；可评估仅文档化「OpenAI + 自定义 `base_url`」 |
| Amazon Bedrock | 未接入 | 签名与调用形态，专用实现 |
| xAI（Grok） | 未接入 | 以官方文档为准 |
| 自定义 API | 未单独枚举 | 若仅 OpenAI 兼容：文档说明选用 OpenAI + 自定义 `base_url` |
| 其他本地推理 | 视需求 | 除 Ollama 外单独立项 |

**Coding Plan** 与上表「API Key」在**鉴权上通常是同一类**（密钥 / Bearer 家族），并不是必然另一套 OAuth；差异主要在 **endpoint、模型 SKU、套餐配额**（例如不限量或大额包月权益）。不少套餐侧暴露的 HTTP 会**对齐 Anthropic Messages 一类形态**，以便 **Claude Code** 与相关 IDE 插件直接复用解析路径——实现上应区分「直连开放平台」与「套餐给出的 `base_url`」、以及字段是否与 Claude Code 适配，而不是先假定鉴权模型完全不同。

| 套餐 / 通道（与产品 Coding Plan 入口对齐，示例） | 本库状态 | 规划要点 |
|:--|:--:|:--|
| 智谱 GLM Coding Plan | 未接入 | 鉴权与智谱 Key 同类；端点 / SKU 可能与百炼直连不同，接口或贴近 Anthropic 以便 Claude Code |
| MiniMax Coding Plan | 未接入 | 同上：套餐 URL 与开放平台分流，协议以厂商说明为准 |
| Kimi Code Plan | 未接入 | 同上 |
| 阿里云百炼 Coding Plan | 未接入 | 与同账号 API Key 产品线关系以官方为准 |
| Claude Pro / Max | 未接入 | 常与 Anthropic API 同属 Messages 形态；endpoint 与配额随套餐变化，单独立项与文档 |
| Codex · ChatGPT Plus | 未接入 | 与 Platform API Key 并行；端点多 OpenAI 兼容，以官方为准 |
| GitHub Copilot | 未接入 | 以 GitHub 文档为准；策略与端点独立评估 |

第三梯队（P3 及 parity 之后）：Groq、Mistral、Together、Fireworks 等海外托管，以及表中未单独展开的长尾厂商；路径仍以官方文档为准，多数可走 OpenAI 兼容层。

## 中长期能力与架构

下列项在设计准则中目前为**非目标**或需单独设计：自动重试、429/5xx 退避、共享或注入 `reqwest::Client`、流式 chat、multipart（语音/大图等）。采纳前须新增 trait / 客户端路径并更新准则，避免与现有「单次 JSON、整包读体」语义混同。

**`audio`** 可择一对接具体厂商（往往涉及 multipart 与流式），或长期保持占位并在 README / rustdoc 中明确「未接远端」。若对接，需单独评估与各厂商 TTS/ASR API 的契约，并更新能力矩阵与 HTTP 文档。

## 日常维护

矩阵与文档以厂商官方说明为准；接口变更时在 CHANGELOG 中提示调用方核对。新增或调整 `Provider` 枚举时勿忘 `#[non_exhaustive]` 与 `FromStr`、工厂分支、feature 表的全链路更新。
