# Changelog

## Unreleased

### Changed

- **Rerank**：`create_rerank_provider` 对 `OpenAI` / `Ollama` 现返回 `Error::Unsupported`（`capability: "rerank"`），而不再返回 `Error::ProviderDisabled`，以区分「未启用厂商 feature」与「该厂商在本模态无实现」。未启用 `aliyun` / `zhipu` feature 时仍选阿里云 / 智谱的，仍为 `ProviderDisabled`（行为未变）。若依赖旧错误变体区分 OpenAI/Ollama 重排序，请改为匹配 `Unsupported`。
