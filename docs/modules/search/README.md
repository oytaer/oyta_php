# 搜索模块 (Search)

## 模块概述

搜索模块提供高性能的全文搜索功能，支持倒排索引、BM25 排序、中文分词、模糊搜索等功能。

## 模块结构

```
search/
├── mod.rs          # 模块入口
├── index.rs        # 倒排索引
├── tokenizer.rs    # 分词器
├── query.rs        # 查询解析
├── ranker.rs       # 排序器
└── storage.rs      # 存储后端
```

## 主要类型

| 类型 | 说明 |
|------|------|
| InvertedIndex | 倒排索引 |
| Tokenizer | 分词器 |
| QueryParser | 查询解析器 |
| Ranker | 排序器 |
| StorageBackend | 存储后端 |

## Search 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `index` | doc: Document | - | 索引文档 |
| `query` | query: &str | Vec<QueryResult> | 搜索查询 |

## 使用示例

### 创建索引

```php
<?php
use oyta\facade\Search;

// 创建索引
Search::index([
    'id' => 1,
    'title' => 'OYTAPHP 使用指南',
    'content' => 'OYTAPHP 是一个高性能的 PHP 运行时',
]);

Search::index([
    'id' => 2,
    'title' => 'Rust 编程语言',
    'content' => 'Rust 是一门系统编程语言',
]);
```

### 搜索查询

```php
<?php
use oyta\facade\Search;

// 搜索
$results = Search::query('PHP');

foreach ($results as $result) {
    echo "ID: " . $result['id'] . "\n";
    echo "标题: " . $result['title'] . "\n";
    echo "得分: " . $result['score'] . "\n";
}
```

### 配置示例

```php
<?php
// config/search.php
return [
    'analyzer' => 'chinese',  // 中文分词
    'storage' => 'memory',    // 存储后端
    'ranking' => 'bm25',      // 排序算法
];
```

## 技术实现

- 倒排索引
- BM25 排序算法
- 中文分词支持
- 模糊搜索
