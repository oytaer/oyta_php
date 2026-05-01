//! 锁定查询模块
//!
//! 提供数据库锁定查询功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：悲观锁、共享锁、FOR UPDATE、SKIP LOCKED 等

use crate::interpreter::value::Value;

/// 锁类型枚举
///
/// 定义支持的锁类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LockType {
    /// 排他锁（FOR UPDATE）
    ForUpdate,
    /// 共享锁（LOCK IN SHARE MODE / FOR SHARE）
    ForShare,
    /// 带跳过锁定行的排他锁（FOR UPDATE SKIP LOCKED）
    ForUpdateSkipLocked,
    /// 带跳过锁定行的共享锁（FOR SHARE SKIP LOCKED）
    ForShareSkipLocked,
    /// 带 NOWAIT 的排他锁（FOR UPDATE NOWAIT）
    ForUpdateNoWait,
    /// 带 NOWAIT 的共享锁（FOR SHARE NOWAIT）
    ForShareNoWait,
}

impl LockType {
    /// 获取锁类型名称
    ///
    /// # 返回
    /// 锁类型名称字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            LockType::ForUpdate => "FOR UPDATE",
            LockType::ForShare => "FOR SHARE",
            LockType::ForUpdateSkipLocked => "FOR UPDATE SKIP LOCKED",
            LockType::ForShareSkipLocked => "FOR SHARE SKIP LOCKED",
            LockType::ForUpdateNoWait => "FOR UPDATE NOWAIT",
            LockType::ForShareNoWait => "FOR SHARE NOWAIT",
        }
    }
}

/// 锁定构建器
///
/// 提供链式调用的锁定构建方法
#[derive(Debug, Clone)]
pub struct LockBuilder {
    /// 锁类型
    pub lock_type: LockType,
    /// 锁定的表名列表（用于多表锁定）
    pub tables: Vec<String>,
    /// 锁定的索引名
    pub index: Option<String>,
    /// 数据库类型
    pub db_type: DatabaseType,
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

impl LockBuilder {
    /// 创建新的锁定构建器
    ///
    /// # 参数
    /// - `lock_type`: 锁类型
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的锁定构建器实例
    pub fn new(lock_type: LockType, db_type: DatabaseType) -> Self {
        Self {
            lock_type,
            tables: Vec::new(),
            index: None,
            db_type,
        }
    }

    /// 创建排他锁构建器
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的锁定构建器实例
    pub fn for_update(db_type: DatabaseType) -> Self {
        Self::new(LockType::ForUpdate, db_type)
    }

    /// 创建共享锁构建器
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的锁定构建器实例
    pub fn for_share(db_type: DatabaseType) -> Self {
        Self::new(LockType::ForShare, db_type)
    }

    /// 创建带 SKIP LOCKED 的排他锁构建器
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的锁定构建器实例
    pub fn for_update_skip_locked(db_type: DatabaseType) -> Self {
        Self::new(LockType::ForUpdateSkipLocked, db_type)
    }

    /// 创建带 NOWAIT 的排他锁构建器
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的锁定构建器实例
    pub fn for_update_no_wait(db_type: DatabaseType) -> Self {
        Self::new(LockType::ForUpdateNoWait, db_type)
    }

    /// 添加锁定的表
    ///
    /// # 参数
    /// - `table`: 表名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn table(mut self, table: &str) -> Self {
        self.tables.push(table.to_string());
        self
    }

    /// 设置锁定的索引
    ///
    /// # 参数
    /// - `index`: 索引名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn index(mut self, index: &str) -> Self {
        self.index = Some(index.to_string());
        self
    }

    /// 构建锁定 SQL
    ///
    /// # 返回
    /// 锁定 SQL 字符串
    pub fn build(&self) -> String {
        match self.db_type {
            DatabaseType::MySQL => self.build_mysql_lock(),
            DatabaseType::PostgreSQL => self.build_postgres_lock(),
            DatabaseType::SQLite => self.build_sqlite_lock(),
        }
    }

    /// 构建 MySQL 锁定 SQL
    ///
    /// # 返回
    /// MySQL 锁定 SQL 字符串
    fn build_mysql_lock(&self) -> String {
        match self.lock_type {
            LockType::ForUpdate => {
                if !self.tables.is_empty() {
                    let tables: Vec<String> = self.tables.iter()
                        .map(|t| format!("`{}`", t))
                        .collect();
                    format!("FOR UPDATE OF {}", tables.join(", "))
                } else {
                    "FOR UPDATE".to_string()
                }
            }
            LockType::ForShare => {
                // MySQL 使用 LOCK IN SHARE MODE
                "LOCK IN SHARE MODE".to_string()
            }
            LockType::ForUpdateSkipLocked => {
                // MySQL 8.0+ 支持 SKIP LOCKED
                "FOR UPDATE SKIP LOCKED".to_string()
            }
            LockType::ForUpdateNoWait => {
                // MySQL 8.0+ 支持 NOWAIT
                "FOR UPDATE NOWAIT".to_string()
            }
            LockType::ForShareSkipLocked | LockType::ForShareNoWait => {
                // MySQL 不支持这些组合，回退到基本共享锁
                "LOCK IN SHARE MODE".to_string()
            }
        }
    }

    /// 构建 PostgreSQL 锁定 SQL
    ///
    /// # 返回
    /// PostgreSQL 锁定 SQL 字符串
    fn build_postgres_lock(&self) -> String {
        let mut sql = String::new();

        match self.lock_type {
            LockType::ForUpdate | LockType::ForUpdateSkipLocked | LockType::ForUpdateNoWait => {
                sql.push_str("FOR UPDATE");
            }
            LockType::ForShare | LockType::ForShareSkipLocked | LockType::ForShareNoWait => {
                sql.push_str("FOR SHARE");
            }
        }

        // 添加 OF 子句
        if !self.tables.is_empty() {
            let tables: Vec<String> = self.tables.iter()
                .map(|t| format!("\"{}\"", t))
                .collect();
            sql.push_str(&format!(" OF {}", tables.join(", ")));
        }

        // 添加 SKIP LOCKED 或 NOWAIT
        match self.lock_type {
            LockType::ForUpdateSkipLocked | LockType::ForShareSkipLocked => {
                sql.push_str(" SKIP LOCKED");
            }
            LockType::ForUpdateNoWait | LockType::ForShareNoWait => {
                sql.push_str(" NOWAIT");
            }
            _ => {}
        }

        sql
    }

    /// 构建 SQLite 锁定 SQL
    ///
    /// # 返回
    /// SQLite 锁定 SQL 字符串
    fn build_sqlite_lock(&self) -> String {
        // SQLite 不支持 FOR UPDATE，返回空字符串
        // SQLite 默认使用数据库级锁定
        String::new()
    }
}

/// 悲观锁查询构建器
///
/// 用于构建悲观锁查询
#[derive(Debug, Clone)]
pub struct PessimisticLockBuilder {
    /// 锁定构建器
    pub lock_builder: LockBuilder,
    /// 是否使用主库
    pub use_master: bool,
}

impl PessimisticLockBuilder {
    /// 创建新的悲观锁构建器
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的悲观锁构建器实例
    pub fn new(db_type: DatabaseType) -> Self {
        Self {
            lock_builder: LockBuilder::for_update(db_type),
            use_master: true,
        }
    }

    /// 设置使用主库
    ///
    /// # 参数
    /// - `use_master`: 是否使用主库
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn use_master(mut self, use_master: bool) -> Self {
        self.use_master = use_master;
        self
    }

    /// 设置锁类型
    ///
    /// # 参数
    /// - `lock_type`: 锁类型
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn lock_type(mut self, lock_type: LockType) -> Self {
        self.lock_builder = LockBuilder::new(lock_type, self.lock_builder.db_type);
        self
    }

    /// 构建锁定 SQL
    ///
    /// # 返回
    /// 锁定 SQL 字符串
    pub fn build(&self) -> String {
        self.lock_builder.build()
    }
}

/// 乐观锁配置
///
/// 用于配置乐观锁行为
#[derive(Debug, Clone)]
pub struct OptimisticLockConfig {
    /// 版本字段名
    pub version_field: String,
    /// 是否自动递增版本
    pub auto_increment: bool,
}

impl Default for OptimisticLockConfig {
    fn default() -> Self {
        Self {
            version_field: "version".to_string(),
            auto_increment: true,
        }
    }
}

impl OptimisticLockConfig {
    /// 创建新的乐观锁配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置版本字段名
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 修改后的配置
    pub fn version_field(mut self, field: &str) -> Self {
        self.version_field = field.to_string();
        self
    }

    /// 构建乐观锁 UPDATE SQL 片段
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `current_version`: 当前版本号
    ///
    /// # 返回
    /// (WHERE 条件, 新版本号)
    pub fn build_update_condition(&self, table: &str, current_version: i64) -> (String, i64) {
        let new_version = current_version + 1;
        let where_condition = format!("{} = {}", self.version_field, current_version);
        (where_condition, new_version)
    }

    /// 构建版本检查 SQL
    ///
    /// # 参数
    /// - `expected_version`: 期望的版本号
    ///
    /// # 返回
    /// WHERE 条件 SQL
    pub fn build_version_check(&self, expected_version: i64) -> String {
        format!("{} = {}", self.version_field, expected_version)
    }
}

/// 锁等待配置
///
/// 配置锁等待行为
#[derive(Debug, Clone)]
pub struct LockWaitConfig {
    /// 是否等待锁
    pub wait: bool,
    /// 等待超时时间（毫秒）
    pub timeout_ms: Option<u64>,
}

impl Default for LockWaitConfig {
    fn default() -> Self {
        Self {
            wait: true,
            timeout_ms: None,
        }
    }
}

impl LockWaitConfig {
    /// 创建新的锁等待配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置不等待锁
    ///
    /// # 返回
    /// 修改后的配置
    pub fn no_wait(mut self) -> Self {
        self.wait = false;
        self
    }

    /// 设置等待超时
    ///
    /// # 参数
    /// - `timeout_ms`: 超时时间（毫秒）
    ///
    /// # 返回
    /// 修改后的配置
    pub fn timeout(mut self, timeout_ms: u64) -> Self {
        self.wait = true;
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// 构建锁等待 SQL 片段
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 锁等待 SQL 片段
    pub fn build(&self, db_type: DatabaseType) -> String {
        if !self.wait {
            return match db_type {
                DatabaseType::MySQL => "NOWAIT".to_string(),
                DatabaseType::PostgreSQL => "NOWAIT".to_string(),
                DatabaseType::SQLite => String::new(),
            };
        }

        if let Some(timeout) = self.timeout_ms {
            match db_type {
                DatabaseType::MySQL => {
                    // MySQL 使用 SET innodb_lock_wait_timeout
                    // 这里返回注释提示
                    format!("/* WAIT {}ms */", timeout)
                }
                DatabaseType::PostgreSQL => {
                    // PostgreSQL 不支持在 SQL 中设置超时
                    format!("/* WAIT {}ms */", timeout)
                }
                DatabaseType::SQLite => String::new(),
            }
        } else {
            String::new()
        }
    }
}

/// 死锁检测配置
#[derive(Debug, Clone)]
pub struct DeadlockConfig {
    /// 是否启用死锁检测
    pub enabled: bool,
    /// 死锁重试次数
    pub retry_count: u32,
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
}

impl Default for DeadlockConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retry_count: 3,
            retry_interval_ms: 100,
        }
    }
}

impl DeadlockConfig {
    /// 创建新的死锁配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置重试次数
    pub fn retry_count(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    /// 设置重试间隔
    pub fn retry_interval(mut self, interval_ms: u64) -> Self {
        self.retry_interval_ms = interval_ms;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_builder_for_update() {
        let builder = LockBuilder::for_update(DatabaseType::MySQL);
        let sql = builder.build();
        assert_eq!(sql, "FOR UPDATE");
    }

    #[test]
    fn test_lock_builder_for_share_mysql() {
        let builder = LockBuilder::for_share(DatabaseType::MySQL);
        let sql = builder.build();
        assert_eq!(sql, "LOCK IN SHARE MODE");
    }

    #[test]
    fn test_lock_builder_for_update_skip_locked() {
        let builder = LockBuilder::for_update_skip_locked(DatabaseType::MySQL);
        let sql = builder.build();
        assert_eq!(sql, "FOR UPDATE SKIP LOCKED");
    }

    #[test]
    fn test_lock_builder_with_table() {
        let builder = LockBuilder::for_update(DatabaseType::MySQL)
            .table("users");
        let sql = builder.build();
        assert!(sql.contains("`users`"));
    }

    #[test]
    fn test_postgres_lock() {
        let builder = LockBuilder::for_update(DatabaseType::PostgreSQL)
            .table("users");
        let sql = builder.build();
        assert!(sql.contains("FOR UPDATE"));
        assert!(sql.contains("OF \"users\""));
    }

    #[test]
    fn test_sqlite_lock() {
        let builder = LockBuilder::for_update(DatabaseType::SQLite);
        let sql = builder.build();
        assert!(sql.is_empty());
    }

    #[test]
    fn test_optimistic_lock_config() {
        let config = OptimisticLockConfig::new()
            .version_field("ver");

        let (where_condition, new_version) = config.build_update_condition("users", 1);
        assert!(where_condition.contains("ver = 1"));
        assert_eq!(new_version, 2);
    }

    #[test]
    fn test_lock_wait_config() {
        let config = LockWaitConfig::new().no_wait();
        let sql = config.build(DatabaseType::MySQL);
        assert_eq!(sql, "NOWAIT");
    }
}
