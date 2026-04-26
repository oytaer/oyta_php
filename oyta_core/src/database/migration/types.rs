//! 迁移类型定义模块
//!
//! 包含迁移相关的数据结构定义

use std::path::PathBuf;

/// 迁移文件信息
#[derive(Debug, Clone)]
pub struct Migration {
    /// 迁移名称
    pub name: String,
    /// 文件名（包含时间戳）
    pub filename: String,
    /// 文件路径
    pub path: PathBuf,
    /// 是否已执行
    pub executed: bool,
    /// 执行时间
    pub executed_at: Option<String>,
    /// 批次号
    pub batch: Option<u32>,
}

/// 迁移记录（数据库存储）
#[derive(Debug, Clone)]
pub struct MigrationRecord {
    /// 记录ID
    pub id: u64,
    /// 迁移名称
    pub migration: String,
    /// 批次号
    pub batch: u32,
    /// 执行时间
    pub executed_at: String,
}
