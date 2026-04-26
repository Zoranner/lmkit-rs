# 文本重排序

`RerankProvider` 对一组候选文档按与 query 的相关性重新排序，常用于 RAG 管道的召回后精排阶段。

## 厂商支持

目前仅 **阿里云** 和 **智谱** 支持重排序：

| 厂商 | feature |
|:---|:---|
| 阿里云 DashScope | `aliyun` + `rerank` |
| 智谱 GLM | `zhipu` + `rerank` |

其他厂商调用会返回 `Unsupported` 错误。

---

## 添加依赖

```toml
[dependencies]
lmkit = { version = "0.1", features = ["aliyun", "rerank"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

---

## 基本用法

```rust
use lmkit::{create_rerank_provider, Provider, ProviderConfig};

let cfg = ProviderConfig::with_base_url(
    Provider::Aliyun,
    std::env::var("DASHSCOPE_API_KEY")?,
    "https://dashscope.aliyuncs.com/api/v1", // rerank 使用独立网关
    "gte-rerank",
);

let rerank = create_rerank_provider(&cfg)?;

let query = "Rust 的内存安全机制";
let documents = vec![
    "Rust 通过所有权系统和借用检查器在编译期保证内存安全",
    "Python 使用引用计数和垃圾回收管理内存",
    "所有权系统防止了悬空指针和数据竞争",
    "今天的天气非常晴朗",
];

let results = rerank.rerank(query, &documents, Some(3)).await?;

for item in &results {
    println!("[{:.4}] {}", item.score, documents[item.index]);
}
```

输出示例：

```
[0.9823] Rust 通过所有权系统和借用检查器在编译期保证内存安全
[0.9415] 所有权系统防止了悬空指针和数据竞争
[0.1203] Python 使用引用计数和垃圾回收管理内存
```

---

## 参数说明

```rust
rerank.rerank(
    query,          // &str：搜索查询
    &documents,     // &[&str]：候选文档列表
    Some(3),        // Option<usize>：返回前 N 条；None 表示返回全部
).await?
```

### RerankItem 字段

| 字段 | 类型 | 说明 |
|:---|:---|:---|
| `index` | `usize` | 文档在原始 `documents` 中的位置 |
| `score` | `f64` | 相关性分数，通常 0.0–1.0，越高越相关 |

结果按 `score` 降序排列。

---

## 与向量检索配合（RAG 精排）

典型的 RAG 管道：先向量检索召回 top-K，再重排序取 top-N：

```rust
// 1. 向量检索：从向量数据库取出相关文档
let candidates: Vec<String> = vector_db.search(&query_vector, top_k: 20).await?;

// 2. 重排序：精排取前 5
let docs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();
let ranked = rerank.rerank(query, &docs, Some(5)).await?;

// 3. 按排名取最终上下文
let context: Vec<&str> = ranked.iter().map(|r| docs[r.index]).collect();
```

---

## 各厂商网关

| 厂商 | `base_url` | 常用模型 |
|:---|:---|:---|
| 阿里云 | `https://dashscope.aliyuncs.com/api/v1` | `gte-rerank`、`gte-rerank-v2` |
| 智谱（中国站） | `https://open.bigmodel.cn/api/paas/v4` | `rerank` |
| 智谱（国际站） | `https://api.z.ai/api/paas/v4` | `rerank` |

> 阿里云 rerank 使用独立网关（`/api/v1`），与 Chat/Embed 的 `/compatible-mode/v1` 不同。

---

## 下一步

- [文生图](image.md)
- [各厂商配置详解](providers.md)
