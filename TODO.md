# 后续改进

当前实现适用于「单次 JSON 请求、非流式」场景。下面按优先级排列待办；已完成项用于对照，避免重复劳动。

## 已完成

- [x] 文生图：OpenAI 兼容 `images/generations`
- [x] 文生图：阿里云 DashScope `multimodal-generation/generation`
- [x] 仓库内 Markdown 文档：`docs/`（接口一览、Rust API、HTTP 端点、[设计准则](docs/design-guidelines.md)）
- [x] 图像相关 wiremock 单测（成功体、阿里云 body 内业务错误）
- [x] crate rustdoc（`lib.rs`、`chat`、`Error`）：Chat 单轮、固定 `temperature`、OpenAI 兼容路径、Bearer/空 key；`audio` 工厂恒 `Unsupported`；`ProviderDisabled` 与 `Unsupported` 文义说明（未改 `Error` 结构）
- [x] 模态模块 `embed` / `rerank` / `image` / `audio`：模块级 rustdoc、`pub mod`、trait/类型简要说明；crate 根摘要补充 rerank 路径（阿里云 `…/reranks`）、文生图双路径；`docs/http-api.md` 中阿里云 rerank 与实现对齐
- [x] `HttpClient::post_bearer_json`：wiremock（200 成功、422+`map_err_body`、200 非 JSON/结构不符 -> `Parse`、请求头 Bearer 与 `application/json`）

## 高优先级（契约与可预期行为）

- [ ] 梳理 **`ProviderDisabled` vs `Unsupported`**：是否在 `Error` 中带 `capability` 等字段或拆变体，使「Cargo feature / 编译分支不可用」与「该模态已编译但该厂商无实现」可区分；顺带统一各工厂 `match` 的返回语义（例如 `rerank` 在已启用 `rerank` + `openai` 时对 OpenAI 仍返回 `ProviderDisabled`，与准则中的理想划分易混淆）

## 中优先级（质量与维护）

- [x] **chat**：wiremock（成功体、`base_url` 尾斜杠、`choices` 空 -> `MissingField`、4xx -> `Api`、2xx 非 JSON -> `Parse`；请求体验证 `model` / 单条 `user` / `temperature: 0.2` 与 Bearer）
- [x] **embed**：`openai_compat` wiremock（批量/单条、`normalize` 后 `input`、`dimensions`、空 `data` -> `MissingField`、`Api`、`Parse`）；**智谱**在 `zhipu` feature 下验证请求体无 `dimensions` 与错误映射（`cargo test --all-features`）
- [x] **rerank**：阿里云 `…/reranks`（含 `top_n` 有值/为 `null`、`parse_aliyun_error`、非 JSON 成功体）；智谱 `…/rerank`（`cargo test --all-features`）
- [ ] 减少 **`#[allow(unreachable_patterns)]`**：考虑小宏或生成式分支，在全 feature 下仍保持可读与可检查性
- [ ] **CI**：至少 `cargo test --all-features`（可选再加 `cargo doc --all-features`），与 [设计准则](docs/design-guidelines.md) 中「全 feature 测试与文档可重复执行」一致
- [ ] **变更记录**：维护 `CHANGELOG.md`（或等价发版说明），记录用户可见 API / 行为变化

## 低优先级（能力与规模）

- [ ] **弹性**：可选重试、429/5xx 退避、可选共享 `reqwest::Client`
- [ ] **流式与 multipart**：与现有 `post_bearer_json` 不同的 trait 与请求路径需单独设计（语音/大图等）
- [ ] **audio**：对接具体厂商（或长期保持占位并在 README/docs 中冻结说明）
- [ ] **`examples/`**：多厂商或多 feature 的示例 crate，便于接入与回归对照（与单元测试互补）

## 发布 crates.io 时再定

下列仅在准备对外发布包时集中处理即可，不必阻塞日常开发。

- [ ] 声明 **MSRV**（README 或 `Cargo.toml` `rust-version`）
- [ ] 核对 **docs.rs** 上各 feature 组合下 trait / 工厂是否在文档中可见、是否与 `docs/` 索引一致

## 产品向能力扩展（单独立项）

README 能力矩阵中已标为不支持（❌）或专用实现（🔧）的方向（如 OpenAI rerank、Ollama 文生图、智谱 embed 深化等），若业务需要应**单独开任务**评估 HTTP 契约与 feature 组合，不默认并入上文维护类待办。
