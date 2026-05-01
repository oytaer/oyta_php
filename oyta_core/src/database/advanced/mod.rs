//! 数据库高级功能模块
//!
//! 提供数据库高级功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：查询缓存、SQL 监听、SQL 日志、断线重连等

pub mod cache;
pub mod listener;
pub mod logger;
pub mod reconnect;

// 重新导出常用类型
pub use cache::{QueryCache, QueryCacheConfig};
pub use listener::{QueryListener, QueryLogEntry};
pub use logger::{DatabaseLogger, LogLevel};
pub use reconnect::{ReconnectConfig, ReconnectManager};
