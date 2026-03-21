# HTTP 端点汇总

下文中的 `{base_url}` 均来自 `ProviderConfig::base_url`，实现里会对末尾 `/` 做规范化后再拼接路径。除特别说明外，请求头含 `Content-Type: application/json`；多数厂商另加 `Authorization: Bearer {api_key}`，**Anthropic Messages** 见 Chat 一节。成功响应体按 JSON 反序列化；HTTP 状态非 2xx 时，错误信息尽量取自响应体原文或解析后的 `code`/`message`（视实现而定）。

## 通用约定

共享客户端见源码 `HttpClient::post_bearer_json`、`HttpClient::post_json_with_headers` 与 `HttpClient::post_json_query`：单次请求、非流式、整包读响应体。各能力默认超时不尽相同：对话约 60 秒，向量约 30 秒，重排序约 60 秒，图像约 120 秒；可通过 `ProviderConfig::timeout` 覆盖。

## Chat

### OpenAI 兼容

路径：`POST {base_url}/chat/completions`。

适用厂商（在启用对应 `chat` + 厂商 feature 时）：`OpenAI`、`Aliyun`、`Ollama`、`Zhipu`。均为同一套 OpenAI 兼容请求体。

请求头：`Authorization: Bearer {api_key}`。

请求 JSON 字段：`model`（来自配置）、`messages`（单条 user，内容为传入的 `prompt` 字符串）、`temperature`（固定为 `0.2`）。

成功时解析：`choices[0].message.content` 作为返回字符串。

典型 `base_url` 示例：`https://api.openai.com/v1`；阿里云 OpenAI 兼容模式多为 `https://dashscope.aliyuncs.com/compatible-mode/v1`（以厂商文档为准）。

### Anthropic Messages 兼容

适用：`Anthropic` + `chat` + `anthropic`。实现的是 **Anthropic Messages 兼容**契约（以[官方 Messages API](https://docs.anthropic.com/en/api/messages)为参考）；凡提供相同路径、请求头与 JSON 形态的网关均可使用（含多款 **Coding Plan**、代理、转发服务），只需把 `base_url` 换成网关提供的根 URL。

路径：`POST {base_url}/messages`。建议 `base_url` 为含版本前缀的根，例如 `https://api.anthropic.com/v1`（实现会在去掉尾部 `/` 后拼接 `/messages`）。

请求头：`x-api-key`（值为 `api_key`）、`anthropic-version`（库内与官方对齐的常量，见 `model_provider::chat::ANTHROPIC_VERSION`）、`Content-Type: application/json`。

请求 JSON 字段：`model`、`max_tokens`（实现内常量，非调用方可配）、`messages`（单条 user，`content` 为 `prompt` 字符串）、`temperature`（固定为 `0.2`）。

成功时解析：响应 `content` 数组中各 `type: "text"` 块的 `text` 拼接为返回字符串；若无文本块则 `MissingField`。当前实现假定成功体中 `content` 为数组；若上游返回其它形状，需按厂商文档扩展解析。

### Google Gemini（generateContent）

适用：`Google` + `chat` + `google`。实现 [Gemini 生成式 REST](https://ai.google.dev/api/rest/v1beta/models.generateContent) 形态的 **非流式** `generateContent`。

路径：`POST {base_url}/models/{model}:generateContent`，其中 `{model}` 为 `ProviderConfig::model` 原样嵌入路径（如 `gemini-2.0-flash`），`{base_url}` 会先去掉尾部 `/`。**不包含** `Authorization: Bearer`；`api_key` 作为 query 参数 **`key`** 附加在 URL 上。

请求 JSON：与官方示例一致，`contents` 为单条对象，内含 `parts`（一条 `text` 为 `prompt`）。[`Content.role`](https://ai.google.dev/api/caching#Content) 在 Google 文档中为可选，本库单轮请求不发送该字段。另含 `generationConfig.temperature`（固定 `0.2`）。

成功时解析：`candidates[0].content.parts` 中各 `text` 字段拼接为返回字符串；若无非空文本则 `MissingField`。若 HTTP 200 但 `candidates` 为空（例如 prompt 被拦截），官方常带 `promptFeedback`；本库返回 `Parse`，文案中含 `promptFeedback` 摘要，便于对照[官方响应说明](https://ai.google.dev/api/generate-content#v1beta.GenerateContentResponse)。

典型 `base_url`：`https://generativelanguage.googleapis.com/v1beta`（以当前 Google 文档为准）。

## Embed

路径：`POST {base_url}/embeddings`。

OpenAI 兼容实现（`OpenAI` / `Aliyun` / `Ollama`，需 `embed` 与对应厂商 feature）：请求体含 `model`、`input`（字符串数组）、`dimensions`（等于配置中的 `dimension`，必填）。成功时取 `data[].embedding`。

智谱实现（`Zhipu`）：路径相同，请求体为 `model` 与 `input`，**不含** `dimensions` 字段；`ProviderConfig::dimension` 仍须在配置中提供，用于 `EmbedProvider::dimension()` 返回值，并与模型实际输出维数一致。

`Anthropic`、`Google`：工厂返回 `Unsupported`（`capability` 为 `embed`）；未启用对应厂商 feature 时为 `ProviderDisabled`。

典型 `base_url`：官方 OpenAI 同 Chat；Ollama 常为 `http://host:11434/v1`（部署方式依环境而定）。

## Rerank

阿里云（`Aliyun` + `rerank` + `aliyun`）：`POST {base_url}/reranks`（路径段为复数 `reranks`，与智谱不同）。请求体：`model`、`query`、`documents`（字符串数组）、`top_n`（可选）。成功时解析 `results` 数组，每项含 `index`、`relevance_score`，映射为库内 `RerankItem`。

智谱（`Zhipu`）：`POST {base_url}/rerank`，请求与响应字段名与阿里云实现一致（`relevance_score`）。若线上分数异常，实现里已有日志提示可考虑换用阿里云 Rerank。

其他厂商在工厂中未实现重排序时返回 `Unsupported`（`capability` 为 `rerank`），含 `OpenAI`、`Ollama`、`Anthropic`（启用 `anthropic` 时）、`Google`（启用 `google` 时）；若选了阿里云或智谱但未启用对应厂商 feature，则返回 `ProviderDisabled`。未启用 `anthropic` / `google` feature 时选 `Anthropic` / `Google` 为 `ProviderDisabled`。

## Image

### OpenAI 兼容文生图

适用：`OpenAI` + `image` + `openai`。

路径：`POST {base_url}/images/generations`。

请求体：`model`、`prompt`、`n`（固定为 `1`）、`size`（由 `ImageSize` 映射为 OpenAI 常见字符串：`512x512`、`1024x1024`、`1792x1024`、`1024x1792`）。

成功时解析 `data[0]`：若存在 `url` 则返回 `ImageOutput::Url`；否则若存在 `b64_json` 则 base64 解码为 `ImageOutput::Bytes`。

典型 `base_url`：`https://api.openai.com/v1`。

### 阿里云 DashScope 文生图

适用：`Aliyun` + `image` + `aliyun`。

路径：`POST {base_url}/services/aigc/multimodal-generation/generation`。注意此处 `base_url` 一般为原生 API 根，例如 `https://dashscope.aliyuncs.com/api/v1` 或国际站 `https://dashscope-intl.aliyuncs.com/api/v1`，**不是** `compatible-mode/v1` 对话网关。

请求体顶层：`model`；`input.messages` 为单条 user，其 `content` 为含 `text` 的数组；`parameters` 含 `negative_prompt`（可选，未传则省略）、`prompt_extend`（实现中为 `true`）、`watermark`（`false`）、`size`（`宽*高` 星号分隔：`512*512`、`1024*1024`、`1792*1024`、`1024*1792`）。

成功时若 HTTP 200 且 JSON 内 `code` 非空，库侧按业务错误处理（见 `Error::Parse`）。否则解析 `output.choices[0].message.content` 中第一项带 `image` 字段的 URL，返回 `ImageOutput::Url`。

工厂阶段：`Ollama` / `Zhipu` / `Anthropic` / `Google` 返回 `Unsupported`（`capability` 为 `image`）；选了 `OpenAI` 或 `Aliyun` 但未启用对应厂商 feature 时返回 `ProviderDisabled`。未启用 `anthropic` / `google` feature 时选 `Anthropic` / `Google` 为 `ProviderDisabled`。

## Audio

未对接任何远端 HTTP。`create_transcription_provider` / `create_speech_provider` 直接返回 `Unsupported`，占位供后续扩展。
