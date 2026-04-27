//! 全文搜索引擎模块
//!
//! 提供高性能的全文搜索功能
//! 支持倒排索引、BM25 排序、中文分词、模糊搜索等功能

// 引入子模块
pub mod index;
pub mod tokenizer;
pub mod query;
pub mod ranker;
pub mod storage;

// 重导出主要类型
pub use index::{InvertedIndex, IndexConfig, Document};
pub use tokenizer::{Tokenizer, Token, TokenType};
pub use query::{QueryParser, Query, QueryResult};
pub use ranker::{Ranker, ScoringMethod, ScoredDocument};
pub use storage::{StorageBackend, StorageConfig, StorageError, StorageManager, MemoryStorage, FileStorage, ShardedStorage, IndexMetadata, SnapshotManager};
