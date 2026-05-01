//! 数据库 CRUD 操作模块
//!
//! 提供完整的数据库 CRUD 操作功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//!
//! ## 模块结构
//!
//! - `insert`: 插入操作
//! - `update`: 更新操作
//! - `delete`: 删除操作
//! - `select`: 查询操作

pub mod delete;
pub mod insert;
pub mod select;
pub mod update;

// 重新导出常用类型
pub use delete::{DeleteBuilder, DeleteResult, DestroyBuilder, SoftDeleteConfig};
pub use insert::{InsertBuilder, InsertResult, UpsertBuilder};
pub use select::{ChunkIterator, QueryResult, SelectBuilder};
pub use update::{BatchUpdateBuilder, UpdateBuilder, UpdateResult, UpdateValue};
