# 后续改进（由代码评审整理）

当前实现已可用于「单请求 JSON 调用」场景。下列条目按优先级大致为：先澄清契约与错误语义，再补测试与弹性，最后做流式与多模态。

文档与 API 预期：在 crate 文档中写明 Chat 为单轮、固定 temperature、OpenAI 兼容路径；说明 Ollama 等无 key 时 Bearer 头行为取决于服务端。为 `image` / `audio` 工厂与 trait 标注「尚未接具体厂商」，避免调用方误以为已可用。

错误类型：`ProviderDisabled` 与「某厂商根本未实现某能力」（例如 OpenAI + rerank）在文案上易混淆，可考虑细分变体或在消息里带上 `capability` 字段。

`match` 与 feature：`#[allow(unreachable_patterns)]` 在全开 feature 时会屏蔽兜底分支的穷尽检查，长期可用小宏或生成式分支减少这类折中。

弹性与规模：按需增加重试、429/5xx 退避、可选共享 `reqwest::Client`；大响应与流式、multipart（语音/图像）与当前 `post_bearer_json` 模型不同，需单独设计 trait 与请求路径。

测试：为 `HttpClient` 或各 provider 增加 wiremock 类集成测试，覆盖成功体、非 JSON 200、4xx 体解析。
