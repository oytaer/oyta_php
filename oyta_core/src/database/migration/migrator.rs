//! 迁移器模块
//!
//! 提供数据库迁移管理功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库

use anyhow::Result;
use std::collections::HashMap;

/// 迁移状态枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MigrationStatus {
    /// 待执行
    Pending,
    /// 已执行
    Executed,
    /// 已回滚
    RolledBack,
    /// 执行失败
    Failed,
}

/// 迁移记录
#[derive(Debug, Clone)]
pub struct Migration {
    /// 迁移名称
    pub name: String,
    /// 迁移版本
    pub version: String,
    /// 迁移文件名
    pub filename: String,
    /// 迁移状态
    pub status: MigrationStatus,
    /// 执行时间
    pub executed_at: Option<String>,
    /// 批次号
    pub batch: Option<i32>,
}

impl Migration {
    /// 创建新的迁移
    pub fn new(name: &str, version: &str, filename: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            filename: filename.to_string(),
            status: MigrationStatus::Pending,
            executed_at: None,
            batch: None,
        }
    }

    /// 标记为已执行
    pub fn mark_executed(&mut self, batch: i32) {
        self.status = MigrationStatus::Executed;
        self.batch = Some(batch);
        self.executed_at = Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    }

    /// 标记为已回滚
    pub fn mark_rolled_back(&mut self) {
        self.status = MigrationStatus::RolledBack;
        self.executed_at = None;
        self.batch = None;
    }
}

/// 迁移管理器
pub struct MigrationManager {
    /// 迁移列表
    pub migrations: HashMap<String, Migration>,
    /// 当前批次号
    pub current_batch: i32,
    /// 迁移表名
    pub table_name: String,
}

impl Default for MigrationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationManager {
    /// 创建新的迁移管理器
    pub fn new() -> Self {
        Self {
            migrations: HashMap::new(),
            current_batch: 0,
            table_name: "migrations".to_string(),
        }
    }

    /// 设置迁移表名
    pub fn table_name(mut self, name: &str) -> Self {
        self.table_name = name.to_string();
        self
    }

    /// 注册迁移
    pub fn register(&mut self, migration: Migration) {
        self.migrations.insert(migration.name.clone(), migration);
    }

    /// 获取待执行的迁移
    pub fn get_pending_migrations(&self) -> Vec<&Migration> {
        self.migrations.values()
            .filter(|m| m.status == MigrationStatus::Pending)
            .collect()
    }

    /// 获取已执行的迁移
    pub fn get_executed_migrations(&self) -> Vec<&Migration> {
        self.migrations.values()
            .filter(|m| m.status == MigrationStatus::Executed)
            .collect()
    }

    /// 获取最后一批迁移
    pub fn get_last_batch(&self) -> Vec<&Migration> {
        let batch = self.get_last_batch_number();
        self.migrations.values()
            .filter(|m| m.batch == Some(batch))
            .collect()
    }

    /// 获取最后一批号
    pub fn get_last_batch_number(&self) -> i32 {
        self.migrations.values()
            .filter_map(|m| m.batch)
            .max()
            .unwrap_or(0)
    }

    /// 执行迁移
    pub fn run_migration(&mut self, name: &str) -> Result<()> {
        // 先检查迁移是否存在且状态正确
        let migration_status = self.migrations.get(name)
            .map(|m| m.status.clone())
            .ok_or_else(|| anyhow::anyhow!("Migration {} not found", name))?;

        if migration_status != MigrationStatus::Pending {
            return Err(anyhow::anyhow!("Migration {} is not pending", name));
        }

        // 开始新批次
        let last_batch = self.get_last_batch_number();
        if last_batch >= self.current_batch {
            self.current_batch = last_batch + 1;
        }

        // 标记迁移为已执行
        if let Some(migration) = self.migrations.get_mut(name) {
            migration.mark_executed(self.current_batch);
        }

        Ok(())
    }

    /// 回滚迁移
    pub fn rollback_migration(&mut self, name: &str) -> Result<()> {
        if let Some(migration) = self.migrations.get_mut(name) {
            if migration.status != MigrationStatus::Executed {
                return Err(anyhow::anyhow!("Migration {} is not executed", name));
            }

            migration.mark_rolled_back();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Migration {} not found", name))
        }
    }

    /// 构建迁移表创建 SQL
    pub fn build_create_table_sql(&self, db_type: DatabaseType) -> String {
        match db_type {
            DatabaseType::MySQL => {
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
                        id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
                        migration VARCHAR(255) NOT NULL,
                        batch INT NOT NULL,
                        executed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4",
                    self.table_name
                )
            }
            DatabaseType::PostgreSQL => {
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
                        id SERIAL PRIMARY KEY,
                        migration VARCHAR(255) NOT NULL,
                        batch INT NOT NULL,
                        executed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                    )",
                    self.table_name
                )
            }
            DatabaseType::SQLite => {
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        migration TEXT NOT NULL,
                        batch INTEGER NOT NULL,
                        executed_at TEXT DEFAULT CURRENT_TIMESTAMP
                    )",
                    self.table_name
                )
            }
        }
    }
}

/// 数据库类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DatabaseType {
    /// MySQL 数据库
    MySQL,
    /// PostgreSQL 数据库
    PostgreSQL,
    /// SQLite 数据库
    SQLite,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration() {
        let migration = Migration::new("create_users_table", "2024_01_01_000001", "2024_01_01_000001_create_users_table.php");
        assert_eq!(migration.status, MigrationStatus::Pending);
    }

    #[test]
    fn test_migration_mark_executed() {
        let mut migration = Migration::new("create_users_table", "2024_01_01_000001", "test.php");
        migration.mark_executed(1);

        assert_eq!(migration.status, MigrationStatus::Executed);
        assert_eq!(migration.batch, Some(1));
        assert!(migration.executed_at.is_some());
    }

    #[test]
    fn test_migration_manager() {
        let mut manager = MigrationManager::new();
        let migration = Migration::new("create_users_table", "2024_01_01_000001", "test.php");
        manager.register(migration);

        assert_eq!(manager.get_pending_migrations().len(), 1);
    }

    #[test]
    fn test_run_migration() {
        let mut manager = MigrationManager::new();
        let migration = Migration::new("create_users_table", "2024_01_01_000001", "test.php");
        manager.register(migration);

        manager.run_migration("create_users_table").unwrap();

        assert_eq!(manager.get_executed_migrations().len(), 1);
        assert_eq!(manager.get_pending_migrations().len(), 0);
    }
}
