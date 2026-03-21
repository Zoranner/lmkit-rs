# 后续工作与维护备忘

本文件用于跟踪实现缺口、工程化事项与发版前检查。设计约定见 [设计准则](docs/design-guidelines.md)；能力矩阵与快速接入见根目录 [README](README.md)。README 中已标为未支持或需专用契约的厂商能力，若要落地须单独评估 HTTP 与 feature 组合后再写入此处。

## 当前状态（摘要）

对话、向量、重排序、文生图等模态在对应 feature 下已有实现；`HttpClient::post_bearer_json` 与各工厂路径配合 wiremock 或模块内测试，覆盖了成功体、部分 4xx、解析失败与请求体验证。`docs/` 下的接口索引、HTTP 汇总、Rust API 说明与设计准则已建立；crate 与各子模块 rustdoc 已补充 feature 与行为说明。各模态工厂对 `ProviderDisabled` / `Unsupported` 的划分与 `rerank` 一致：`chat` / `embed` / `image` 的 `create` 已用互斥 `cfg` 分支去掉 `#[allow(unreachable_patterns)]`；文生图对未启用厂商 feature 的 `OpenAI` / `Aliyun` 返回 `ProviderDisabled`（见 [CHANGELOG](CHANGELOG.md)）。`image` 与 `rerank` 一样配有工厂级单测；占位能力 `audio` 仍为 `Unsupported`。

## 近期值得做

仓库内尚无 CI（例如 GitHub Actions）。建议至少门禁 `cargo test --all-features`，与设计准则中「全 feature 测试可重复执行」一致；可按需叠加 `cargo doc --all-features --no-deps`、`cargo clippy --all-features -D warnings`、`cargo fmt --check`。

## 质量与文档

`CHANGELOG.md` 的 Unreleased 需在发版时写入版本号并持续记录用户可见变更；与 README、`docs/` 交叉核对，避免矩阵或错误说明滞后。

新增或调整 `Provider` 枚举、工厂分支或错误变体时，按设计准则回归区分 `ProviderDisabled` 与 `Unsupported`；若要在 `ProviderDisabled` 上附加 `capability` 等字段，须先评估破坏性与文档成本。

## 能力与架构（中长期）

下列项在设计准则中目前列为非目标或占位；采纳前须更新准则并单独设计 trait / HTTP 路径，避免与现有「单次 JSON」语义混同：可选重试、429/5xx 退避、共享或注入 `reqwest::Client`；流式 chat、multipart（语音/大图等）。

`audio` 仍为类型与工厂占位，可择一对接具体厂商，或长期冻结并在 README / rustdoc 中保持「未接远端」的显式说明。

增加 `examples/` 下的最小可运行示例（多厂商或多 feature 组合），与单测互补，便于接入与人工对照。

## 发布 crates.io 时集中处理

在 `Cargo.toml` 中声明 **MSRV**（`rust-version`），并在 README 或文档中写明；按需补齐 `repository`、`documentation`、`homepage` 等元数据，便于 docs.rs 与依赖方溯源。

发版前在本地或 CI 用 `cargo doc --all-features --no-deps` 核对公开 trait、工厂与类型在各 feature 下的可见性，并与 `docs/interfaces.md`、README 矩阵一致。

## 日常维护

变更厂商 HTTP 契约或矩阵描述时，对照工厂实际分支与 `docs/interfaces.md`；以厂商官方文档为准，并在变更记录中提示调用方核对。
