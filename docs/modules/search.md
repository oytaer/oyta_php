# 搜索模块

## 模块概述

搜索模块实现了全文搜索引擎，支持倒排索引、BM25/TF-IDF 排序、中英文分词、持久化存储等功能。

## 文件结构

```
search/
├── mod.rs          # 模块入口
├── index.rs        # 倒排索引
├── query.rs        # 查询解析
├── ranker.rs       # 排序算法
├── tokenizer.rs    # 分词器
└── storage.rs      # 存储后端
```

## 核心设计

### 倒排索引结构

```
┌─────────────────────────────────────────────────────────────────┐
│                        倒排索引                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│   词典 (Dictionary)                                               │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │  term    │  倒排列表指针                                   │   │
│   ├─────────────────────────────────────────────────────────┤   │
│   │ "hello"  │  ─────────────────────────────┐              │   │
│   │ "world"  │  ─────────────────────────────┼──┐           │   │
│   │ "rust"   │  ─────────────────────────────┼──┼──┐        │   │
│   └─────────────────────────────────────────────────────────┘   │
│              │              │              │                     │
│              ▼              ▼              ▼                     │
│   倒排列表 (Posting List)                                         │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │ "hello": [doc1: [pos:1,2,5], doc3: [pos:10]]            │   │
│   │ "world": [doc1: [pos:3], doc2: [pos:1,4], doc5: [pos:7]]│   │
│   │ "rust":  [doc2: [pos:2], doc4: [pos:1,3,5]]             │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## 核心组件

### InvertedIndex

倒排索引核心实现：

```rust
pub struct InvertedIndex {
    /// 词典：term → PostingList
    postings: DashMap<String, PostingList>,
    /// 文档存储
    documents: DashMap<u64, Document>,
    /// 文档计数
    document_count: AtomicU64,
    /// 下一个文档 ID
    next_doc_id: AtomicU64,
    /// 索引配置
    config: IndexConfig,
}

pub struct PostingList {
    /// 包含该词的文档数
    doc_freq: u64,
    /// 文档 ID → Posting
    postings: HashMap<u64, Posting>,
}

pub struct Posting {
    /// 文档 ID
    doc_id: u64,
    /// 词频
    term_freq: u64,
    /// 位置列表
    positions: Vec<u32>,
}
```

### QueryParser

查询解析器：

```rust
pub struct QueryParser {
    /// 分词器
    tokenizer: Tokenizer,
    /// 是否启用模糊搜索
    enable_fuzzy: bool,
    /// 模糊搜索编辑距离
    fuzzy_distance: usize,
}
```

**支持的查询类型**：

| 查询类型 | 语法 | 说明 |
|----------|------|------|
| 词项查询 | `hello` | 精确匹配单个词 |
| 短语查询 | `"hello world"` | 精确匹配短语 |
| 布尔查询 | `hello AND world` | 布尔组合查询 |
| 或查询 | `hello OR world` | 任一匹配 |
| 非查询 | `hello NOT world` | 排除匹配 |
| 通配符查询 | `hel*` | 通配符匹配 |
| 模糊查询 | `hello~2` | 编辑距离模糊匹配 |
| 范围查询 | `[10 TO 100]` | 数值范围查询 |

### Ranker

排序算法：

```rust
pub trait Ranker: Send + Sync {
    /// 计算文档得分
    fn score(&self, query: &Query, doc_id: u64, index: &InvertedIndex) -> f64;
}
```

**BM25 排序**：

```
score(D, Q) = Σ IDF(qi) * (f(qi, D) * (k1 + 1)) / (f(qi, D) + k1 * (1 - b + b * |D| / avgdl))

其中：
- IDF(qi) = log((N - n(qi) + 0.5) / (n(qi) + 0.5) + 1)
- f(qi, D) = 词 qi 在文档 D 中的词频
- |D| = 文档 D 的长度
- avgdl = 平均文档长度
- k1, b = 调节参数（默认 k1=1.2, b=0.75）
```

**TF-IDF 排序**：

```
score(D, Q) = Σ tf(qi, D) * idf(qi)

其中：
- tf(qi, D) = 词 qi 在文档 D 中的词频
- idf(qi) = log(N / n(qi))
```

### Tokenizer

分词器：

```rust
pub struct Tokenizer {
    /// 分词模式
    mode: TokenizeMode,
    /// 是否保留标点
    keep_punctuation: bool,
    /// 是否保留数字
    keep_numbers: bool,
    /// 是否转小写
    lowercase: bool,
    /// 停用词列表
    stop_words: HashSet<String>,
}

pub enum TokenizeMode {
    /// 简单分词（按空格分割）
    Simple,
    /// 中文分词（基于词典）
    Chinese,
    /// 混合分词（中英文混合）
    Mixed,
}
```

### Storage

存储后端：

```rust
pub struct IndexStorage {
    /// 存储路径
    path: PathBuf,
    /// 索引文件句柄
    index_file: File,
    /// 数据文件句柄
    data_file: File,
    /// 统计信息
    stats: StorageStats,
}

pub struct StorageStats {
    /// 总读取次数
    total_reads: AtomicU64,
    /// 总写入次数
    total_writes: AtomicU64,
    /// 缓存命中次数
    cache_hits: AtomicU64,
    /// 缓存未命中次数
    cache_misses: AtomicU64,
    /// 索引大小（字节）
    index_size: AtomicU64,
}
```

## API 使用

### 创建索引

```php
// 创建索引
$index = new \oyta\Search\Index('my_index');

// 添加文档
$index->addDocument([
    'id' => 1,
    'title' => 'Hello World',
    'content' => 'This is a sample document.',
]);

// 批量添加文档
$index->addDocuments([
    ['id' => 2, 'title' => 'Rust Programming', 'content' => 'Rust is a systems language.'],
    ['id' => 3, 'title' => 'PHP Development', 'content' => 'PHP is a web language.'],
]);
```

### 搜索

```php
// 简单搜索
$results = $index->search('hello world');

// 带选项搜索
$results = $index->search('rust programming', [
    'fields' => ['title', 'content'],
    'limit' => 10,
    'offset' => 0,
    'ranker' => 'bm25',
    'highlight' => true,
]);

// 高级查询
$query = new \oyta\Search\Query();
$query->must('title', 'rust')
      ->should('content', 'programming')
      ->mustNot('content', 'java');
$results = $index->search($query);
```

### 删除文档

```php
// 删除单个文档
$index->deleteDocument(1);

// 删除多个文档
$index->deleteDocuments([1, 2, 3]);

// 清空索引
$index->clear();
```

### 持久化

```php
// 保存索引到磁盘
$index->save('/path/to/index');

// 从磁盘加载索引
$index = \oyta\Search\Index::load('/path/to/index');
```

## 搜索结果

```php
$results = $index->search('hello');

foreach ($results as $result) {
    echo "ID: " . $result->getId() . "\n";
    echo "Score: " . $result->getScore() . "\n";
    echo "Title: " . $result->getField('title') . "\n";
    echo "Highlight: " . $result->getHighlight('content') . "\n";
}
```

## 配置说明

```php
// config/search.php
return [
    // 默认索引路径
    'index_path' => runtime_path('search'),
    
    // 分词配置
    'tokenizer' => [
        'mode' => 'mixed', // simple, chinese, mixed
        'lowercase' => true,
        'stop_words' => ['the', 'a', 'an', 'is', 'are'],
    ],
    
    // 排序配置
    'ranker' => [
        'type' => 'bm25',
        'bm25' => [
            'k1' => 1.2,
            'b' => 0.75,
        ],
    ],
    
    // 存储配置
    'storage' => [
        'auto_save' => true,
        'save_interval' => 60, // 秒
    ],
];
```

## 性能特点

| 操作 | 时间复杂度 | 说明 |
|------|------------|------|
| 添加文档 | O(n) | n = 文档词数 |
| 删除文档 | O(n) | n = 文档词数 |
| 搜索 | O(m * log k) | m = 查询词数，k = 结果数 |
| 精确查询 | O(1) | 直接词典查找 |

### 索引大小

- 英文文档：约原始大小的 30-50%
- 中文文档：约原始大小的 50-80%
