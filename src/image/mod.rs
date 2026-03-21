//! 图像生成：OpenAI 兼容 `images/generations`，或阿里云 DashScope 原生 `multimodal-generation/generation`。

#[cfg(all(feature = "aliyun", feature = "image"))]
mod aliyun;
#[cfg(all(feature = "openai", feature = "image"))]
mod openai_compat;

use async_trait::async_trait;

use crate::config::Provider;
use crate::config::ProviderConfig;
use crate::error::{Error, Result};

/// 生成尺寸：OpenAI 使用 `宽x高`；阿里云 DashScope 使用 `宽*高`（实现中分别映射）。
#[derive(Debug, Clone, Copy)]
pub enum ImageSize {
    /// 正方形较小边（OpenAI `512x512` / 阿里云 `512*512`）
    Square512,
    /// 正方形 1K（`1024x1024` / `1024*1024`）
    Square1024,
    /// 横版（OpenAI `1792x1024`；阿里云 `1792*1024`）
    Landscape,
    /// 竖版（OpenAI `1024x1792`；阿里云 `1024*1792`）
    Portrait,
}

#[derive(Debug, Clone)]
pub enum ImageOutput {
    Url(String),
    Bytes(Vec<u8>),
}

#[async_trait]
pub trait ImageProvider: Send + Sync {
    async fn generate(&self, prompt: &str, size: ImageSize) -> Result<ImageOutput>;
}

pub(crate) fn create(config: &ProviderConfig) -> Result<Box<dyn ImageProvider>> {
    #[allow(unreachable_patterns)]
    match config.provider {
        #[cfg(all(feature = "openai", feature = "image"))]
        Provider::OpenAI => Ok(Box::new(openai_compat::OpenaiCompatImage::new(config)?)),
        #[cfg(all(feature = "aliyun", feature = "image"))]
        Provider::Aliyun => Ok(Box::new(aliyun::AliyunQwenImage::new(config)?)),
        p => Err(Error::Unsupported {
            provider: p.to_string(),
            capability: "image",
        }),
    }
}
