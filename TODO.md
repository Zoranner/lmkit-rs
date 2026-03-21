# 后续工作与维护备忘

本文件跟踪实现缺口、工程化事项与发版前检查。设计原则与错误语义约定以 [设计准则](docs/design-guidelines.md) 为准；对外能力矩阵以根目录 [README](README.md) 为准。与 README 中已标 ❌/🔧、但尚未立项的具体厂商能力，不默认排进下文「待办」，需要时单独开任务评估 HTTP 契约与 feature 组合。

## 已完成（摘要）

对话、向量、重排序、文生图等模态在默认或全 feature 组合下已有相应实现；`HttpClient::post_bearer_json`、chat、embed（含 OpenAI 兼容与智谱）、rerank（阿里云与智谱）、图像（OpenAI 与阿里云）等路径已配 wiremock 或模块内单测覆盖成功体、部分 4xx、解析失败与请求体验证。`docs/` 下的接口索引、HTTP 汇总、Rust API 说明与设计准则已建立；crate 根与各模态的 rustdoc 已补充 feature 与行为说明。重排序工厂已对 `OpenAI` / `Ollama` 使用 `Unsupported`，并在 `rerank::factory_tests` 中覆盖与 `cargo test --features rerank,openai` 下的 `ProviderDisabled` 路径；`CHANGELOG.md` 已记录该行为变更。

## 高优先级

- [ ] **错误模型**：`rerank` 工厂已对 OpenAI/Ollama 使用 `Unsupported`（见 `CHANGELOG.md`）。其余模态（如 `chat` / `embed` / `image`）若仍存在「无实现却落 `ProviderDisabled`」的边角，可逐项对齐；是否需要为 `ProviderDisabled` 补充 `capability` 等字段仍待整体评估。

## 中优先级（质量与工程）

- [ ] **`#[allow(unreachable_patterns)]`**：现见于 `chat` / `embed` / `image` 的工厂 `match`（`rerank` 已无此属性）。评估用小宏、`cfg` 拆分或生成式分支，在全 feature 下去掉压制同时保持可读与穷尽性检查。
- [ ] **CI**：仓库内尚无持续集成配置。建议至少门禁 `cargo test --all-features`（与设计准则中「全 feature 测试可重复执行」一致）；可选叠加 `cargo doc --all-features --no-deps`、`cargo clippy --all-features -D warnings`。
- [ ] **变更记录**：已有 `CHANGELOG.md`（Unreleased）；发版时补齐版本号并持续写入用户可见变化，与 README / `docs/` 交叉引用。

## 低优先级（能力与规模）

- [ ] **弹性**：可选重试、429/5xx 退避、可选注入或共享 `reqwest::Client`（准则中列为非目标，采纳前需更新准则并单独设计，避免与现有「单次 JSON」语义混同）。
- [ ] **流式与 multipart**：与现有 `post_bearer_json` 不同的 trait 与请求路径需单独设计（如流式 chat、语音/大图等多部分请求）。
- [ ] **audio**：对接具体厂商，或长期保持占位并在 README / rustdoc 中明确冻结说明（与当前准则一致即可）。
- [ ] **examples/**：增加多厂商或多 feature 的示例工程，便于接入与人工对照（与单测互补）。

## 发布 crates.io 时再集中处理

- [ ] 声明 **MSRV**（`Cargo.toml` 的 `rust-version` 与/或 README）。
- [ ] 核对 **docs.rs** 上各 feature 组合下公开 trait、工厂与类型是否可见，并与 `docs/` 索引一致。

## 日常维护时可顺手核对

README 能力矩阵、`docs/interfaces.md` 与工厂实际分支是否一致；厂商侧 API 变更时以官方文档为准并考虑在变更记录中提示调用方。
