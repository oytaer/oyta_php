//! CLI 优化命令模块
//!
//! 提供应用优化相关的命令实现
//! 包括：路由缓存、配置缓存、数据库字段缓存、自动加载优化
//!
//! # 命令列表
//! - `route:cache` - 生成路由缓存
//! - `config:cache` - 生成配置缓存
//! - `optimize` - 执行所有优化
//! - `optimize:route` - 仅优化路由
//! - `optimize:schema` - 生成数据库字段缓存
//!
//! # 模块结构
//! - `types`: 类型定义
//! - `optimizer`: 优化器主体
//! - `route_cache`: 路由缓存
//! - `config_cache`: 配置缓存
//! - `schema_cache`: 数据库结构缓存
//! - `autoload`: 自动加载优化

pub mod autoload;
pub mod config_cache;
pub mod optimizer;
pub mod route_cache;
pub mod schema_cache;
pub mod types;

// 重新导出主要类型
pub use optimizer::Optimizer;
pub use types::{ColumnInfo, IndexInfo, RouteInfo, TableSchema};
