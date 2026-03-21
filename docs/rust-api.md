# Rust 公共 API

以下内容对应 `src/lib.rs` 及子模块的 **公开重导出** 与 **工厂函数**。未列出的模块成员多为 `pub(crate)`，不保证稳定。

`chat`、`embed`、`rerank`、`image`、`audio` 已作为 **`pub mod`** 暴露，便于在 **rustdoc** 中阅读各模态模块级契约（与 crate 根摘要互补）。本地可执行 `cargo doc --no-deps --open`；若需一次浏览全部模态文档，可使用 `cargo doc --all-features --no-deps --open`。本节与 [HTTP 文档](http-api.md) 为 Markdown 侧摘要，路径与字段以源码为准。

## 配置与枚举

`ProviderConfig` 字段：`provider`（`Provider` 枚举）、`api_key`、`base_url`、`model`、`dimension`（可选，embed 必填）、`timeout`（可选，覆盖该次 HTTP 超时）。其中 **`model` 原样进入各模态请求**；本库不校验其是否在厂商侧可用，无效或无权使用的模型由上游返回错误，一般为 `Error::Api`（见 [设计准则](design-guidelines.md)「配置与 HTTP 约定」）。

`Provider`：`OpenAI`、`Aliyun`、`Anthropic`、`Google`、`Ollama`、`Zhipu`，`#[non_exhaustive]`，后续可能扩展。启用 `anthropic` 与 `chat` 时，Anthropic Messages 的 `anthropic-version` 请求头与库内常量 `model_provider::chat::ANTHROPIC_VERSION` 一致（与 [HTTP 文档](http-api.md) 对照）。启用 `google` 与 `chat` 时，Gemini 使用 query 参数 `key` 传递 `api_key`，见该文档 Chat 一节。

`FromStr` 可按不区分大小写的字符串解析厂商名（如 `openai`、`Aliyun`）；未知名称返回 `Error::UnknownProvider`。

## 错误

`Error` 变体包括：未知厂商名、`ProviderDisabled`、`Unsupported`、`MissingConfig`、HTTP 非成功（`Api`，含状态码与消息）、HTTP 层错误（`Http`，来自 `reqwest`）、JSON 解析失败（`Parse`）、响应缺字段（`MissingField`）。

`ProviderDisabled` 与 `Unsupported` 的划分以源码中 `Error` 的 rustdoc（`src/error.rs`）为准；也可用 `cargo doc --open` 查看。典型例子：重排序下 `OpenAI` / `Ollama` / `Google`（启用 `google` 时）等为 `Unsupported`（`capability: "rerank"`），未启用阿里云或智谱 feature 却选该厂商时为 `ProviderDisabled`；文生图下 `Ollama` / `Zhipu` / `Google`（启用 `google` 时）等为 `Unsupported`（`capability: "image"`），未启用 `openai` / `aliyun` 却选对应厂商时为 `ProviderDisabled`。各 `create_*` 条目与模块级 rustdoc 中有完整说明。

`Result<T>` 为 `std::result::Result<T, Error>`。

## 工厂函数

均在启用对应模态 feature 时可用；是否还需启用厂商 feature、以及失败时返回 `ProviderDisabled` 还是 `Unsupported`，依各工厂与能力矩阵而定（见上节及下文各 `create_*` 说明）。

`create_chat_provider(&ProviderConfig) -> Result<Box<dyn ChatProvider>>`（feature `chat`）。

`create_embed_provider(&ProviderConfig) -> Result<Box<dyn EmbedProvider>>`（feature `embed`）。`Anthropic` 返回 `Unsupported`（`capability: "embed"`）；未启用对应厂商 feature 时选该厂商为 `ProviderDisabled`。`Google` 在启用 `google` 时为 Gemini 嵌入实现；未启用 `google` feature 时为 `ProviderDisabled`。

`create_rerank_provider(&ProviderConfig) -> Result<Box<dyn RerankProvider>>`（feature `rerank`）。仅阿里云与智谱有实现；`OpenAI` / `Ollama` / `Anthropic` / `Google` 返回 `Unsupported`；选了阿里云或智谱但未启用对应厂商 feature 时返回 `ProviderDisabled`。

`create_image_provider(&ProviderConfig) -> Result<Box<dyn ImageProvider>>`（feature `image`）。`OpenAI` / `Aliyun` 需同时启用对应厂商 feature，否则为 `ProviderDisabled`；`Ollama` / `Zhipu` / `Anthropic` / `Google` 为 `Unsupported`（`capability: "image"`）。

`create_transcription_provider`、`create_speech_provider`（feature `audio`）：当前始终返回 `Unsupported`，仅占位。

## 对话

`ChatProvider`：`async fn chat(&self, prompt: &str) -> Result<String>`。

实现为单轮用户消息、`temperature` 固定为 `0.2`；Anthropic 分支为 **Messages 兼容**（含 `max_tokens` 与响应 `content` 解析）；Google 分支为 **Gemini generateContent**（query `key`）。详见 [HTTP 文档](http-api.md#chat)。

## 向量

`EmbedProvider`：`async fn encode`、`async fn encode_batch`，以及 `fn dimension(&self) -> usize`。

文本在送入请求前会做首尾空白裁剪与连续空白折叠（见 `util::normalize_for_embedding`）。

## 重排序

`RerankItem`：`index: usize`、`score: f64`。

`RerankProvider`：`async fn rerank(&self, query, documents, top_n) -> Result<Vec<RerankItem>>`。

## 图像

`ImageSize`：`Square512`、`Square1024`、`Landscape`、`Portrait`；映射到各厂商字符串的方式见 [HTTP 文档](http-api.md#image)。

`ImageOutput`：`Url(String)` 或 `Bytes(Vec<u8>)`。

`ImageProvider`：`async fn generate(&self, prompt, size) -> Result<ImageOutput>`。

## 音频

`AudioFormat`：`Wav`、`Mp3`、`Ogg`、`Flac`。

`TranscriptionProvider`：`async fn transcribe(&self, audio, format) -> Result<String>`。

`SpeechProvider`：`async fn synthesize(&self, text, voice) -> Result<Vec<u8>>`。

当前无可用实现。

运行 `cargo doc --open --no-deps` 可在本地生成带跳转的 API 文档。
