# 接口一览

下表便于从「能力」跳到 Rust 与 HTTP 两端的说明；细节以 [Rust 公共 API](rust-api.md) 与 [HTTP 端点汇总](http-api.md) 为准。

| 能力 | 工厂函数（feature） | 主要 trait | 远端约定摘要 |
|:---|:---|:---|:---|
| 对话 | `create_chat_provider`（`chat`） | `ChatProvider` | OpenAI 兼容：`POST …/chat/completions`；Anthropic Messages：`POST …/messages`（`x-api-key`）；Google Gemini：`POST …/models/{model}:generateContent`（query `key`） |
| 向量 | `create_embed_provider`（`embed`） | `EmbedProvider` | OpenAI 兼容：`POST …/embeddings`；智谱无 `dimensions`；Google Gemini：`…:embedContent` / `…:batchEmbedContents`（query `key`） |
| 重排序 | `create_rerank_provider`（`rerank`） | `RerankProvider` | 阿里云 `POST …/reranks`；智谱 `POST …/rerank` |
| 文生图 | `create_image_provider`（`image`） | `ImageProvider` | OpenAI：`…/images/generations`；阿里云：`…/services/aigc/multimodal-generation/generation` |
| 语音识别 | `create_transcription_provider`（`audio`） | `TranscriptionProvider` | 未实现 |
| 语音合成 | `create_speech_provider`（`audio`） | `SpeechProvider` | 未实现 |

厂商由 `Provider` 与 Cargo feature 共同决定；未启用的组合在工厂阶段失败，不会发 HTTP。厂商与能力的矩阵见仓库根目录 `README.md`。重排序：表中未列出的厂商（如 OpenAI、Ollama、Anthropic、Google）在启用 `rerank` 时走工厂会得到 `Unsupported`，与「未启用阿里云 / 智谱 feature」时的 `ProviderDisabled` 不同，见 [HTTP 文档](http-api.md) 中 Rerank 一节。文生图：`Ollama` / `Zhipu` / `Anthropic` / `Google` 为 `Unsupported`；启用 `image` 但未启用 `openai` / `aliyun` 仍选 OpenAI 或阿里云时为 `ProviderDisabled`，见该文档 Image 一节。向量：`Anthropic` 为 `Unsupported`（`capability` 为 `embed`）。
