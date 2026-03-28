# 文本向量化

`EmbedProvider` 将文本转换为浮点数向量，用于语义搜索、相似度计算、RAG 等场景。

## 添加依赖

```toml
[dependencies]
lmkit = { version = "0.2", features = ["openai", "embed"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

---

## 配置

向量化必须设置 `dimension`，其值需与所选模型的实际输出维度一致：

```rust
use lmkit::{create_embed_provider, Provider, ProviderConfig};

let mut cfg = ProviderConfig::new(
    Provider::OpenAI,
    std::env::var("OPENAI_API_KEY")?,
    "https://api.openai.com/v1",
    "text-embedding-3-small",
);
cfg.dimension = Some(1536); // 必填

let embed = create_embed_provider(&cfg)?;
```

如果没有设置 `dimension`，工厂函数会返回 `MissingConfig { field: "dimension" }` 错误。

---

## 单条向量化

```rust
let vector: Vec<f64> = embed.encode("Hello, world!").await?;
println!("维度: {}", vector.len()); // 等于 cfg.dimension
```

输入文本会自动做首尾空白裁剪和连续空白折叠。

---

## 批量向量化

```rust
let texts = vec!["Rust 的所有权", "Python 的 GIL", "Go 的 goroutine"];
let vectors: Vec<Vec<f64>> = embed.encode_batch(&texts.iter().map(|s| *s).collect::<Vec<_>>()).await?;

for (text, vec) in texts.iter().zip(&vectors) {
    println!("{text}: {} 维", vec.len());
}
```

批量请求视厂商而定，可能合并为单次 HTTP 调用。

---

## 获取维度

```rust
let dim = embed.dimension();
println!("向量维度: {dim}");
```

---

## 各厂商模型参考

| 厂商 | 常用模型 | 维度 |
|:---|:---|:---|
| OpenAI | `text-embedding-3-small` | 1536 |
| OpenAI | `text-embedding-3-large` | 3072 |
| OpenAI | `text-embedding-ada-002` | 1536 |
| 阿里云 | `text-embedding-v3` | 1024 |
| 阿里云 | `text-embedding-v2` | 1536 |
| Google | `text-embedding-004` | 768 |
| Google | `embedding-001` | 768 |
| Ollama | 取决于具体模型 | 模型相关 |
| 智谱 | `embedding-3` | 2048 |

> 以上维度仅供参考，请以厂商文档为准。

---

## 语义相似度示例

```rust
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    dot / (norm_a * norm_b)
}

let v1 = embed.encode("Rust 内存安全").await?;
let v2 = embed.encode("C++ RAII 与内存管理").await?;
let v3 = embed.encode("今天天气真好").await?;

println!("Rust vs C++: {:.4}", cosine_similarity(&v1, &v2)); // 较高
println!("Rust vs 天气: {:.4}", cosine_similarity(&v1, &v3)); // 较低
```

---

## 下一步

- [文本重排序](rerank.md)
- [各厂商配置详解](providers.md)
