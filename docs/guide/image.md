# 文生图

`ImageProvider` 根据文本描述生成图片。目前支持 **OpenAI** 和 **阿里云**。

## 厂商支持

| 厂商 | feature | 常用模型 |
|:---|:---|:---|
| OpenAI | `openai` + `image` | `dall-e-3`、`dall-e-2` |
| 阿里云 | `aliyun` + `image` | `wanx-v1`、`wanx2.1-t2i-turbo` |

---

## 添加依赖

```toml
[dependencies]
lmkit = { version = "0.1", features = ["openai", "image"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

---

## 基本用法

```rust
use lmkit::{create_image_provider, ImageSize, Provider, ProviderConfig};

let cfg = ProviderConfig::new(
    Provider::OpenAI,
    std::env::var("OPENAI_API_KEY")?,
    "dall-e-3",
);

let image = create_image_provider(&cfg)?;

let output = image.generate(
    "一只在宇宙中漂浮的橙色猫，背景是星云，赛博朋克风格",
    ImageSize::Square1024,
).await?;
```

---

## 处理输出

`generate` 返回 `ImageOutput`，有两种变体：

```rust
use lmkit::ImageOutput;

match output {
    ImageOutput::Url(url) => {
        println!("图片 URL: {url}");
        // 可用 reqwest 等下载
    }
    ImageOutput::Bytes(bytes) => {
        // 直接保存到文件
        std::fs::write("output.png", &bytes)?;
        println!("已保存，共 {} 字节", bytes.len());
    }
}
```

> 不同厂商和模型返回类型不同：OpenAI DALL-E 通常返回 URL；阿里云通常也返回 URL。

---

## 图片尺寸

```rust
use lmkit::ImageSize;

ImageSize::Square512      // 512 × 512
ImageSize::Square1024     // 1024 × 1024
ImageSize::Landscape      // 1792 × 1024（横向）
ImageSize::Portrait       // 1024 × 1792（纵向）
```

> 并非所有厂商都支持所有尺寸。例如 DALL-E 2 不支持非正方形尺寸，传入时厂商侧会报错。

---

## 各厂商网关

| 厂商 | `base_url` | HTTP 路径 |
|:---|:---|:---|
| OpenAI | `https://api.openai.com/v1` | `POST /images/generations` |
| 阿里云 | `https://dashscope.aliyuncs.com/api/v1` | `POST /services/aigc/multimodal-generation/generation` |

> 阿里云文生图使用独立网关（`/api/v1`），与 Chat/Embed 的 `/compatible-mode/v1` 不同。

---

## 下一步

- [各厂商配置详解](providers.md)
- [错误处理](error-handling.md)
