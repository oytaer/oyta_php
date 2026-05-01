//! 数据库迁移模块
//!
//! 提供数据库迁移功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：表创建、表修改、索引管理等

pub mod migrator;
pub mod schema;

// 重新导出常用类型
pub use migrator::{Migration, MigrationManager, MigrationStatus};
pub use schema::{ColumnDefinition, ForeignKeyDefinition, IndexDefinition, SchemaBuilder, TableDefinition};
