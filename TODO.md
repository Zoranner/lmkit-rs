# 后续改进

当前实现适用于「单次 JSON 请求、非流式」场景。下面按优先级排列待办；已完成项用于对照，避免重复劳动。

## 已完成

- [x] 文生图：OpenAI 兼容 `images/generations`
- [x] 文生图：阿里云 DashScope `multimodal-generation/generation`
- [x] 仓库内 Markdown 文档：`docs/`（接口一览、Rust API、HTTP 端点）
- [x] 图像相关 wiremock 单测（成功体、阿里云 body 内业务错误）

## 高优先级（契约与可预期行为）

- [ ] 在 **crate rustdoc**（`lib.rs` 与各模态模块）写明：Chat 为单轮、`temperature` 固定值、OpenAI 兼容路径；无 API key 时仍发送 Bearer 的行为取决于上游（如 Ollama）
- [ ] 在 rustdoc 中强调：`audio` 工厂当前恒为 `Unsupported`，避免误以为已接厂商
- [ ] 梳理 **`ProviderDisabled` vs `Unsupported`**：是否在 `Error` 中带 `capability` 字段或拆变体，使「feature 未开」与「厂商不支持该能力」可区分

## 中优先级（质量与维护）

- [ ] 为 **chat / embed / rerank** 或 **`HttpClient`** 增加 wiremock 测试：成功 JSON、非 JSON 的 2xx、4xx 响应体解析
- [ ] 减少 **`#[allow(unreachable_patterns)]`**：考虑小宏或生成式分支，在全 feature 下仍保持可读与可检查性

## 低优先级（能力与规模）

- [ ] **弹性**：可选重试、429/5xx 退避、可选共享 `reqwest::Client`
- [ ] **流式与 multipart**：与现有 `post_bearer_json` 不同的 trait 与请求路径需单独设计（语音/大图等）
- [ ] **audio**：对接具体厂商（或长期保持占位并在 README/docs 中冻结说明）
