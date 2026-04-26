//! 数据库迁移系统
//!
//! 提供数据库结构版本控制和迁移管理
//! 支持：创建迁移、执行迁移、回滚迁移、迁移状态查看
//!
//! # 功能特性
//! - 迁移文件生成
//! - 迁移执行与回滚
//! - 迁移状态追踪（持久化到数据库）
//! - 批量迁移管理
//! - 多数据库支持（MySQL/PostgreSQL/SQLite）
//!
//! # 命令支持
//! - `oyta make:migration create_users_table` - 创建迁移
//! - `oyta migrate` - 执行迁移
//! - `oyta migrate:rollback` - 回滚迁移
//! - `oyta migrate:status` - 查看迁移状态
//! - `oyta migrate:reset` - 重置所有迁移
//! - `oyta migrate:fresh` - 删除所有表并重新迁移
//!
//! # 模块结构
//! - `types`: 类型定义
//! - `manager`: 迁移管理器
//! - `helpers`: 辅助函数
//! - `seeder`: 数据填充

pub mod helpers;
pub mod manager;
pub mod seeder;
pub mod types;

// 重新导出主要类型
pub use helpers::{detect_migration_type, extract_table_name, migration_name_to_class, MigrationType};
pub use manager::MigrationManager;
pub use seeder::SeederManager;
pub use types::{Migration, MigrationRecord};
