# model-provider 接口文档

本目录描述 crate 的 **对外 Rust API** 与各实现背后的 **HTTP 调用约定**。实现细节以源码为准；若云端接口升级，请以厂商文档为准并在本库发版时核对。

阅读顺序：先扫 [接口一览](interfaces.md)，再读 [Rust 公共 API](rust-api.md) 了解类型与工厂，需要对接网关时查阅 [HTTP 端点汇总](http-api.md)。

相关文件：

- [接口一览](interfaces.md)：能力、工厂、trait、HTTP 摘要对照表
- [Rust 公共 API](rust-api.md)：`ProviderConfig`、错误类型、`create_*_provider`、各能力 trait
- [HTTP 端点汇总](http-api.md)：方法、路径、`base_url` 约定、请求与响应字段摘要
