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

pub mod cluster;
pub mod connection;
pub mod executor;
pub mod facade;
pub mod migration;
pub mod model;
pub mod pool_trait;
pub mod postgres_pool;
pub mod query_builder;
pub mod read_write_split;
pub mod relations;
pub mod sqlite_pool;

// 导出 Db 门面
pub use facade::Db;
