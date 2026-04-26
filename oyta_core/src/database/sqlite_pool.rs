//! SQLite 数据库驱动实现
//!
//! 实现 DatabasePool trait，提供 SQLite 数据库支持
//! 基于 sqlx 的 SqlitePool 实现连接池管理
//! 支持：查询、执行、事务、参数绑定等
//! 特点：无需独立数据库服务，适合嵌入式/开发环境

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{Column, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;

use crate::interpreter::value::Value;
use super::pool_trait::{DatabasePool, DatabaseTransaction, QueryResult};

/// SQLite 连接池包装器
///
/// 封装 sqlx::SqlitePool，实现 DatabasePool trait
/// 提供线程安全的连接池管理
/// SQLite 是文件数据库，支持内存模式
pub struct SqlitePoolWrapper {
    /// 底层连接池
    pool: Arc<SqlitePool>,
    /// 数据库路径（用于日志）
    db_path: String,
    /// 连接池大小
    pool_size: u32,
}

impl SqlitePoolWrapper {
    /// 创建新的 SQLite 连接池
    ///
    /// # 参数
    /// - `path`: 数据库文件路径
    ///   - 文件路径: "/path/to/database.db"
    ///   - 内存模式: ":memory:"
    ///   - 临时文件: ""
    /// - `pool_size`: 连接池大小（SQLite 通常为 1）
    ///
    /// # 返回
    /// 创建好的连接池实例
    ///
    /// # 示例
    /// ```ignore
    /// // 文件数据库
    /// let pool = SqlitePoolWrapper::new("/data/app.db", 1).await?;
    ///
    /// // 内存数据库
    /// let pool = SqlitePoolWrapper::new(":memory:", 1).await?;
    /// ```
    pub async fn new(path: &str, pool_size: u32) -> Result<Self> {
        // 构建 SQLite 连接字符串
        let dsn = if path == ":memory:" {
            "sqlite::memory:".to_string()
        } else if path.is_empty() {
            "sqlite::memory:".to_string()
        } else {
            format!("sqlite:{}", path)
        };

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(pool_size)
            .connect(&dsn)
            .await
            .with_context(|| format!("无法连接 SQLite 数据库: {}", path))?;

        tracing::info!("SQLite 连接池创建成功: {}", path);

        Ok(Self {
            pool: Arc::new(pool),
            db_path: path.to_string(),
            pool_size,
        })
    }

    /// 从现有 SqlitePool 创建包装器
    pub fn from_pool(pool: SqlitePool, db_path: &str, pool_size: u32) -> Self {
        Self {
            pool: Arc::new(pool),
            db_path: db_path.to_string(),
            pool_size,
        }
    }

    /// 获取底层连接池引用
    pub fn inner(&self) -> &SqlitePool {
        &self.pool
    }

    /// 获取数据库路径
    pub fn db_path(&self) -> &str {
        &self.db_path
    }

    /// 检查是否为内存数据库
    pub fn is_memory(&self) -> bool {
        self.db_path == ":memory:" || self.db_path.is_empty()
    }

    /// 将 SQLite 行数据转换为 Value 映射
    fn row_to_map(row: &sqlx::sqlite::SqliteRow) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        for (i, column) in row.columns().iter().enumerate() {
            let col_name = column.name().to_string();
            let value = Self::row_column_to_value(row, i);
            map.insert(col_name, value);
        }
        map
    }

    /// 将行中的单列转换为 Value
    fn row_column_to_value(row: &sqlx::sqlite::SqliteRow, index: usize) -> Value {
        // 尝试解析为整数
        if let Ok(v) = row.try_get::<i64, _>(index) {
            return Value::Int(v);
        }
        // 尝试解析为浮点数
        if let Ok(v) = row.try_get::<f64, _>(index) {
            return Value::Float(v);
        }
        // 尝试解析为布尔值（SQLite 存储为 0/1）
        if let Ok(v) = row.try_get::<i64, _>(index) {
            if v == 0 || v == 1 {
                // 可能是布尔值，但我们无法确定，返回整数
                return Value::Int(v);
            }
        }
        // 尝试解析为字符串
        if let Ok(v) = row.try_get::<String, _>(index) {
            return Value::String(v);
        }
        // 尝试解析为字节数组（BLOB）
        if let Ok(v) = row.try_get::<Vec<u8>, _>(index) {
            // 尝试解码为 UTF-8 字符串
            if let Ok(s) = String::from_utf8(v.clone()) {
                return Value::String(s);
            }
            // 否则返回 base64 编码
            return Value::String(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &v,
            ));
        }
        // 默认返回 Null
        Value::Null
    }

    /// 执行 PRAGMA 命令
    ///
    /// SQLite 特有的配置命令
    ///
    /// # 参数
    /// - `pragma`: PRAGMA 命令（不含 PRAGMA 关键字）
    ///
    /// # 示例
    /// ```ignore
    /// pool.pragma("journal_mode=WAL").await?;
    /// pool.pragma("foreign_keys=ON").await?;
    /// ```
    pub async fn pragma(&self, pragma: &str) -> Result<()> {
        let sql = format!("PRAGMA {}", pragma);
        self.execute(&sql).await?;
        tracing::debug!("SQLite PRAGMA: {}", pragma);
        Ok(())
    }

    /// 启用外键约束
    pub async fn enable_foreign_keys(&self) -> Result<()> {
        self.pragma("foreign_keys=ON").await
    }

    /// 设置日志模式
    ///
    /// # 参数
    /// - `mode`: 日志模式（DELETE/TRUNCATE/PERSIST/MEMORY/WAL/OFF）
    pub async fn set_journal_mode(&self, mode: &str) -> Result<()> {
        self.pragma(&format!("journal_mode={}", mode)).await
    }

    /// 设置同步模式
    ///
    /// # 参数
    /// - `mode`: 同步模式（OFF/NORMAL/FULL/EXTRA）
    pub async fn set_synchronous(&self, mode: &str) -> Result<()> {
        self.pragma(&format!("synchronous={}", mode)).await
    }

    /// 获取数据库大小
    pub async fn database_size(&self) -> Result<u64> {
        let result = self.query("SELECT page_count * page_size as size FROM pragma_page_count, pragma_page_size").await?;
        Ok(result.scalar().and_then(|v| match v {
            Value::Int(i) => Some(*i as u64),
            _ => None,
        }).unwrap_or(0))
    }

    /// 优化数据库
    pub async fn optimize(&self) -> Result<()> {
        self.execute("VACUUM").await?;
        self.execute("ANALYZE").await?;
        tracing::info!("SQLite 数据库已优化");
        Ok(())
    }

    /// 备份数据库到文件
    ///
    /// # 参数
    /// - `dest_path`: 目标文件路径
    pub async fn backup_to(&self, dest_path: &str) -> Result<()> {
        let sql = format!("VACUUM INTO '{}'", dest_path);
        self.execute(&sql).await?;
        tracing::info!("SQLite 数据库已备份到: {}", dest_path);
        Ok(())
    }
}

/// SQLite 事务包装器
pub struct SqliteTransaction {
    tx: Option<sqlx::Transaction<'static, Sqlite>>,
}

impl SqliteTransaction {
    pub fn new(tx: sqlx::Transaction<'static, Sqlite>) -> Self {
        Self { tx: Some(tx) }
    }
}

#[async_trait]
impl DatabasePool for SqlitePoolWrapper {
    /// 执行 SQL 查询
    async fn query(&self, sql: &str) -> Result<QueryResult> {
        tracing::debug!("SQLite 查询: {}", sql);

        let rows = sqlx::query(sql)
            .fetch_all(&*self.pool)
            .await
            .with_context(|| format!("SQLite 查询失败: {}", sql))?;

        let result_rows: Vec<HashMap<String, Value>> = rows
            .iter()
            .map(|row| Self::row_to_map(row))
            .collect();

        Ok(QueryResult {
            rows: result_rows,
            affected_rows: 0,
            last_insert_id: None,
        })
    }

    /// 执行参数化查询
    async fn query_with_params(&self, sql: &str, params: &[Value]) -> Result<QueryResult> {
        tracing::debug!("SQLite 参数化查询: {} (参数: {})", sql, params.len());

        let mut query_builder = sqlx::query(sql);

        for param in params {
            query_builder = bind_value_sqlite(query_builder, param);
        }

        let rows = query_builder
            .fetch_all(&*self.pool)
            .await
            .with_context(|| format!("SQLite 参数化查询失败: {}", sql))?;

        let result_rows: Vec<HashMap<String, Value>> = rows
            .iter()
            .map(|row| Self::row_to_map(row))
            .collect();

        Ok(QueryResult {
            rows: result_rows,
            affected_rows: 0,
            last_insert_id: None,
        })
    }

    /// 执行 SQL 更新
    async fn execute(&self, sql: &str) -> Result<u64> {
        tracing::debug!("SQLite 执行: {}", sql);

        let result = sqlx::query(sql)
            .execute(&*self.pool)
            .await
            .with_context(|| format!("SQLite 执行失败: {}", sql))?;

        Ok(result.rows_affected())
    }

    /// 执行参数化更新
    async fn execute_with_params(&self, sql: &str, params: &[Value]) -> Result<u64> {
        tracing::debug!("SQLite 参数化执行: {} (参数: {})", sql, params.len());

        let mut query_builder = sqlx::query(sql);

        for param in params {
            query_builder = bind_value_sqlite(query_builder, param);
        }

        let result = query_builder
            .execute(&*self.pool)
            .await
            .with_context(|| format!("SQLite 参数化执行失败: {}", sql))?;

        Ok(result.rows_affected())
    }

    /// 执行 INSERT 并返回自增 ID
    async fn insert_get_id(&self, sql: &str) -> Result<i64> {
        tracing::debug!("SQLite INSERT: {}", sql);

        let result = sqlx::query(sql)
            .execute(&*self.pool)
            .await
            .with_context(|| format!("SQLite INSERT 失败: {}", sql))?;

        Ok(result.last_insert_rowid())
    }

    /// 开启事务
    async fn begin_transaction(&self) -> Result<Box<dyn DatabaseTransaction + Send>> {
        let tx = self.pool.begin().await?;
        tracing::debug!("SQLite 事务已开启");
        Ok(Box::new(SqliteTransaction::new(tx)))
    }

    /// 获取连接池状态
    fn pool_status(&self) -> (u32, u32, u32) {
        let status = self.pool.size();
        let idle = self.pool.num_idle();
        (status as u32 - idle as u32, idle as u32, self.pool_size)
    }

    /// 检查连接是否可用
    async fn is_healthy(&self) -> bool {
        sqlx::query("SELECT 1")
            .fetch_one(&*self.pool)
            .await
            .is_ok()
    }

    /// 获取数据库类型
    fn database_type(&self) -> &str {
        "sqlite"
    }
}

#[async_trait]
impl DatabaseTransaction for SqliteTransaction {
    /// 提交事务
    async fn commit(mut self: Box<Self>) -> Result<()> {
        if let Some(tx) = self.tx.take() {
            tx.commit().await?;
            tracing::debug!("SQLite 事务已提交");
        }
        Ok(())
    }

    /// 回滚事务
    async fn rollback(mut self: Box<Self>) -> Result<()> {
        if let Some(tx) = self.tx.take() {
            tx.rollback().await?;
            tracing::debug!("SQLite 事务已回滚");
        }
        Ok(())
    }

    /// 在事务中执行查询
    async fn query(&mut self, sql: &str) -> Result<QueryResult> {
        let tx = self.tx.as_mut().ok_or_else(|| anyhow::anyhow!("事务已关闭"))?;

        let conn = tx.deref_mut();
        let rows = sqlx::query(sql)
            .fetch_all(conn)
            .await
            .with_context(|| format!("事务查询失败: {}", sql))?;

        let result_rows: Vec<HashMap<String, Value>> = rows
            .iter()
            .map(|row| SqlitePoolWrapper::row_to_map(row))
            .collect();

        Ok(QueryResult {
            rows: result_rows,
            affected_rows: 0,
            last_insert_id: None,
        })
    }

    /// 在事务中执行更新
    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let tx = self.tx.as_mut().ok_or_else(|| anyhow::anyhow!("事务已关闭"))?;

        let conn = tx.deref_mut();
        let result = sqlx::query(sql)
            .execute(conn)
            .await
            .with_context(|| format!("事务执行失败: {}", sql))?;

        Ok(result.rows_affected())
    }
}

/// 将 Value 绑定到 SQLite 查询参数
fn bind_value_sqlite<'q>(
    query: sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    value: &'q Value,
) -> sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
    match value {
        Value::Int(i) => query.bind(*i),
        Value::Float(f) => query.bind(*f),
        Value::String(s) => query.bind(s.as_str()),
        Value::Bool(b) => query.bind(if *b { 1i64 } else { 0i64 }),
        Value::Null => query.bind(Option::<String>::None),
        _ => query.bind(value.to_string_value()),
    }
}

/// SQLite 连接池选项
pub struct SqlitePoolConfig {
    /// 最大连接数
    pub max_connections: u32,
    /// 是否启用外键约束
    pub foreign_keys: bool,
    /// 日志模式
    pub journal_mode: String,
    /// 同步模式
    pub synchronous: String,
}

impl Default for SqlitePoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 1, // SQLite 通常只需要一个连接
            foreign_keys: true,
            journal_mode: "WAL".to_string(),
            synchronous: "NORMAL".to_string(),
        }
    }
}

impl SqlitePoolConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置最大连接数
    pub fn max_connections(mut self, n: u32) -> Self {
        self.max_connections = n;
        self
    }

    /// 设置是否启用外键约束
    pub fn foreign_keys(mut self, enabled: bool) -> Self {
        self.foreign_keys = enabled;
        self
    }

    /// 设置日志模式
    pub fn journal_mode(mut self, mode: &str) -> Self {
        self.journal_mode = mode.to_string();
        self
    }

    /// 构建连接池
    pub async fn build(&self, path: &str) -> Result<SqlitePoolWrapper> {
        let pool = SqlitePoolWrapper::new(path, self.max_connections).await?;

        // 配置 SQLite
        if self.foreign_keys {
            pool.enable_foreign_keys().await?;
        }
        pool.set_journal_mode(&self.journal_mode).await?;
        pool.set_synchronous(&self.synchronous).await?;

        Ok(pool)
    }
}

/// SQLite 数据库迁移助手
///
/// 提供简单的数据库迁移功能
pub struct SqliteMigrator {
    /// 连接池
    pool: SqlitePoolWrapper,
    /// 迁移表名
    migrations_table: String,
}

impl SqliteMigrator {
    /// 创建新的迁移助手
    pub fn new(pool: SqlitePoolWrapper) -> Self {
        Self {
            pool,
            migrations_table: "migrations".to_string(),
        }
    }

    /// 设置迁移表名
    pub fn migrations_table(mut self, name: &str) -> Self {
        self.migrations_table = name.to_string();
        self
    }

    /// 初始化迁移表
    pub async fn initialize(&self) -> Result<()> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                executed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            self.migrations_table
        );
        self.pool.execute(&sql).await?;
        Ok(())
    }

    /// 执行迁移
    ///
    /// # 参数
    /// - `name`: 迁移名称
    /// - `sql`: 迁移 SQL
    pub async fn migrate(&self, name: &str, sql: &str) -> Result<()> {
        // 检查是否已执行
        let check_sql = format!(
            "SELECT COUNT(*) as count FROM {} WHERE name = ?",
            self.migrations_table
        );
        let result = self.pool.query_with_params(&check_sql, &[Value::String(name.to_string())]).await?;

        let count = result.scalar().and_then(|v| match v {
            Value::Int(i) => Some(*i),
            _ => None,
        }).unwrap_or(0);

        if count > 0 {
            tracing::debug!("迁移已存在，跳过: {}", name);
            return Ok(());
        }

        // 执行迁移
        self.pool.execute(sql).await?;

        // 记录迁移
        let insert_sql = format!(
            "INSERT INTO {} (name) VALUES (?)",
            self.migrations_table
        );
        self.pool.execute_with_params(&insert_sql, &[Value::String(name.to_string())]).await?;

        tracing::info!("迁移执行成功: {}", name);
        Ok(())
    }

    /// 获取已执行的迁移列表
    pub async fn executed_migrations(&self) -> Result<Vec<String>> {
        let sql = format!("SELECT name FROM {} ORDER BY id", self.migrations_table);
        let result = self.pool.query(&sql).await?;

        Ok(result.rows.iter()
            .filter_map(|row| row.get("name").and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            }))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_memory_pool() {
        let pool = SqlitePoolWrapper::new(":memory:", 1).await.unwrap();
        assert!(pool.is_memory());
        assert_eq!(pool.database_type(), "sqlite");
    }
}
