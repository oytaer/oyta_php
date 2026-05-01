//! 数据库模块
//!
//! 提供 OYTAPHP 的数据库功能
//! 支持连接管理、查询构建器、Model ORM、Db 门面等
//! 基于 sqlx 实现异步 MySQL/PostgreSQL/SQLite 连接
//!
//! # 功能特性
//! - 多数据库驱动支持（MySQL/PostgreSQL/SQLite）
//! - 读写分离（主从架构）
//! - 数据库集群支持
//! - 连接池管理
//! - 健康检查与故障转移
//! - 数据库迁移
//! - 高级查询构建器
//! - CRUD 操作
//! - ORM 扩展功能
//! - 查询缓存、SQL 监听、断线重连

// 原有模块
pub mod cluster;
pub mod connection;
pub mod executor;
pub mod facade;
pub mod model;
pub mod pool_trait;
pub mod postgres_pool;
pub mod query_builder;
pub mod read_write_split;
pub mod relations;
pub mod sqlite_pool;

// 新增模块
pub mod advanced;
pub mod crud;
pub mod migration;
pub mod orm;
pub mod query;

// 导出 Db 门面
pub use facade::Db;

// 导出常用类型
pub use query::{
    AggregateBuilder, AggregateExecutor, AggregateType, CountBuilder, DatabaseType,
    Field, FieldBuilder, HavingBuilder, JoinBuilder, JoinType, LockBuilder, LockType,
    OrderBuilder, PaginateBuilder, PaginateResult, SubQueryBuilder, UnionBuilder, UnionType,
    WhereBuilder, WhereType,
};

pub use crud::{
    DeleteBuilder, InsertBuilder, SelectBuilder, UpdateBuilder, UpdateValue,
    InsertResult, UpdateResult, DeleteResult, QueryResult,
};

pub use orm::{
    AutoTimestampConfig, TimestampManager, TimestampType,
    SoftDeleteConfig, SoftDeleteManager, SoftDeleteType,
    JsonFieldOperator, JsonPath,
    EventContext, EventManager, EventType,
};

pub use migration::{
    Migration, MigrationManager, MigrationStatus,
    ColumnDefinition, ForeignKeyDefinition, IndexDefinition, SchemaBuilder, TableDefinition,
};

pub use advanced::{
    QueryCache, QueryCacheConfig,
    QueryListener, QueryLogEntry,
    DatabaseLogger, LogLevel,
    ReconnectConfig, ReconnectManager,
};
