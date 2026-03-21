# 开发计划

与当前仓库对齐的待办与路线说明。实现细节以 `docs/design-guidelines.md` 为准。

---

## 示例与文档卫生

- [ ] 新增 `examples/`：至少覆盖「换厂商同一套代码」与多 feature 组合（可与 README 快速开始呼应）
- [ ] `docs/contributing.md` 中克隆地址仍为占位 `your-repo`，有公开仓库后改为真实 URL

---

## 功能补全（按需求排期）

### 流式 Chat（基础需求）

- [ ] **API 设计**
  - 新增 trait `ChatStreamProvider` 或在现有 `ChatProvider` 增加流式方法（例如 `async fn chat_stream(&self, prompt: &str) -> Result<impl Stream<Item = Result<ChatChunk>>>`）
  - 确定统一返回类型：`ChatChunk` 需抽象出各厂商共同字段（如 `delta.content`、`finish_reason`），避免暴露厂商专属字段
- [ ] **HTTP 与 SSE 解析**
  - OpenAI 兼容（OpenAI / Aliyun / Ollama / Zhipu）：`stream: true`，SSE 格式 `data: {...}`，结束 `data: [DONE]`
  - Anthropic：SSE 事件类型 `content_block_delta` / `message_stop` 等，需单独解析器
  - Google：不支持 SSE，可能需要轮询或文档标注不支持流式
- [ ] **错误处理**
  - 流中途失败映射为 `Error`（网络中断、解析错误、服务端错误）
  - 区分「连接阶段失败」与「流式阶段失败」
- [ ] **feature gate**
  - 流式实现建议挂在现有 `chat` feature 下，不新增 feature（避免分裂矩阵）
- [ ] **文档与示例**
  - `docs/http-endpoints.md` 补充各厂商流式端点差异
  - `README.md` 与 `docs/api-reference.md` 同步流式用法
  - `examples/` 中增加流式示例
- [ ] **测试**
  - wiremock 模拟 SSE 响应，覆盖正常流、中途错误、边界情况

---

### 其他能力

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
| Chat | ✅ | 已列厂商均支持（非流式） |
| Chat Stream | 未实现 | 见上方规划 |
| Embed | ✅ | Anthropic 除外 |
| Rerank | ✅ | 仅阿里云、智谱 |
| Image | ✅ | 仅 OpenAI、阿里云 |
| Audio | 占位 | Trait 与工厂已存在，无远端实现 |

---

## 非目标（未写入新 trait 前不承诺）

- 自动重试与 429 退避
- 共享 `reqwest::Client` 连接池策略
- 语音 multipart
- 按厂商维护模型白名单
- 发请求前按模型名本地拦截
