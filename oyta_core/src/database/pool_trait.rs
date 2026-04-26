//! 数据库驱动抽象模块
//!
//! 定义统一的数据库驱动接口，支持 MySQL、PostgreSQL、SQLite
//! 使用 trait 抽象不同数据库的差异，实现代码复用
//! 支持读写分离、连接池管理、事务处理

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::interpreter::value::Value;

/// 查询结果结构体
///
/// 封装所有数据库查询的返回结果
/// 包含查询到的行数据、影响行数、最后插入ID等
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// 查询到的行数据
    /// 每行是一个字段名到值的映射
    pub rows: Vec<HashMap<String, Value>>,
    /// 受影响的行数（INSERT/UPDATE/DELETE）
    pub affected_rows: u64,
    /// 最后插入的自增 ID（INSERT 操作）
    pub last_insert_id: Option<i64>,
}

impl QueryResult {
    /// 创建空的查询结果
    pub fn empty() -> Self {
        Self {
            rows: Vec::new(),
            affected_rows: 0,
            last_insert_id: None,
        }
    }

    /// 判断结果是否为空
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// 获取行数
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// 获取第一行数据
    pub fn first(&self) -> Option<&HashMap<String, Value>> {
        self.rows.first()
    }

    /// 获取第一行第一列的值
    /// 常用于 COUNT/SUM 等聚合查询
    pub fn scalar(&self) -> Option<&Value> {
        self.rows.first().and_then(|row| row.values().next())
    }
}

impl Default for QueryResult {
    fn default() -> Self {
        Self::empty()
    }
}

/// 数据库连接池抽象 trait
///
/// 定义所有数据库驱动必须实现的方法
/// 支持查询、执行、事务等核心操作
/// 使用 async_trait 支持异步方法
#[async_trait]
pub trait DatabasePool: Send + Sync {
    /// 执行 SQL 查询（SELECT）
    ///
    /// # 参数
    /// - `sql`: SQL 查询语句
    ///
    /// # 返回
    /// 查询结果，包含行数据
    async fn query(&self, sql: &str) -> Result<QueryResult>;

    /// 执行参数化 SQL 查询
    ///
    /// # 参数
    /// - `sql`: 带占位符的 SQL 语句
    /// - `params`: 参数值列表
    ///
    /// # 返回
    /// 查询结果
    async fn query_with_params(&self, sql: &str, params: &[Value]) -> Result<QueryResult>;

    /// 执行 SQL 更新（INSERT/UPDATE/DELETE）
    ///
    /// # 参数
    /// - `sql`: SQL 更新语句
    ///
    /// # 返回
    /// 受影响的行数
    async fn execute(&self, sql: &str) -> Result<u64>;

    /// 执行参数化 SQL 更新
    ///
    /// # 参数
    /// - `sql`: 带占位符的 SQL 语句
    /// - `params`: 参数值列表
    ///
    /// # 返回
    /// 受影响的行数
    async fn execute_with_params(&self, sql: &str, params: &[Value]) -> Result<u64>;

    /// 执行 INSERT 并返回自增 ID
    ///
    /// # 参数
    /// - `sql`: INSERT SQL 语句
    ///
    /// # 返回
    /// 最后插入的自增 ID
    async fn insert_get_id(&self, sql: &str) -> Result<i64>;

    /// 查询单行数据
    ///
    /// # 参数
    /// - `sql`: SQL 查询语句
    ///
    /// # 返回
    /// 单行数据，如果无结果返回 None
    async fn query_one(&self, sql: &str) -> Result<Option<HashMap<String, Value>>> {
        let result = self.query(sql).await?;
        Ok(result.first().cloned())
    }

    /// 查询单个标量值
    ///
    /// 适用于 COUNT/SUM/MAX/MIN 等聚合查询
    ///
    /// # 参数
    /// - `sql`: SQL 查询语句
    ///
    /// # 返回
    /// 标量值
    async fn query_scalar(&self, sql: &str) -> Result<Value> {
        let result = self.query(sql).await?;
        Ok(result.scalar().cloned().unwrap_or(Value::Null))
    }

    /// 开启事务
    ///
    /// # 返回
    /// 事务对象（具体类型由实现决定）
    async fn begin_transaction(&self) -> Result<Box<dyn DatabaseTransaction + Send>>;

    /// 获取连接池状态
    ///
    /// # 返回
    /// (活跃连接数, 空闲连接数, 最大连接数)
    fn pool_status(&self) -> (u32, u32, u32);

    /// 检查连接是否可用
    ///
    /// # 返回
    /// 连接是否健康
    async fn is_healthy(&self) -> bool;

    /// 获取数据库类型
    ///
    /// # 返回
    /// 数据库类型字符串
    fn database_type(&self) -> &str;
}

/// 数据库事务抽象 trait
///
/// 定义事务的提交和回滚操作
#[async_trait]
pub trait DatabaseTransaction {
    /// 提交事务
    async fn commit(self: Box<Self>) -> Result<()>;

    /// 回滚事务
    async fn rollback(self: Box<Self>) -> Result<()>;

    /// 在事务中执行查询
    async fn query(&mut self, sql: &str) -> Result<QueryResult>;

    /// 在事务中执行更新
    async fn execute(&mut self, sql: &str) -> Result<u64>;
}

/// 数据库类型枚举
///
/// 定义支持的数据库类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DatabaseType {
    /// MySQL 数据库
    MySQL,
    /// PostgreSQL 数据库
    PostgreSQL,
    /// SQLite 数据库
    SQLite,
}

impl DatabaseType {
    /// 从字符串解析数据库类型
    ///
    /// # 参数
    /// - `s`: 数据库类型字符串
    ///
    /// # 返回
    /// 对应的数据库类型枚举
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "mysql" => DatabaseType::MySQL,
            "pgsql" | "postgres" | "postgresql" => DatabaseType::PostgreSQL,
            "sqlite" => DatabaseType::SQLite,
            _ => DatabaseType::MySQL,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &str {
        match self {
            DatabaseType::MySQL => "mysql",
            DatabaseType::PostgreSQL => "postgresql",
            DatabaseType::SQLite => "sqlite",
        }
    }

    /// 获取默认端口
    pub fn default_port(&self) -> u16 {
        match self {
            DatabaseType::MySQL => 3306,
            DatabaseType::PostgreSQL => 5432,
            DatabaseType::SQLite => 0,
        }
    }

    /// 获取占位符格式
    ///
    /// MySQL 使用 ?，PostgreSQL 使用 $1, $2...
    pub fn placeholder(&self, index: usize) -> String {
        match self {
            DatabaseType::MySQL | DatabaseType::SQLite => "?".to_string(),
            DatabaseType::PostgreSQL => format!("${}", index),
        }
    }

    /// 是否支持 RETURNING 子句
    ///
    /// PostgreSQL 和 SQLite3.35+ 支持 RETURNING
    pub fn supports_returning(&self) -> bool {
        matches!(self, DatabaseType::PostgreSQL | DatabaseType::SQLite)
    }
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// SQL 方言差异处理
///
/// 不同数据库的 SQL 语法差异
pub struct SqlDialect {
    /// 数据库类型
    pub db_type: DatabaseType,
}

impl SqlDialect {
    /// 创建新的 SQL 方言处理器
    pub fn new(db_type: DatabaseType) -> Self {
        Self { db_type }
    }

    /// 获取标识符引用字符
    ///
    /// MySQL 使用反引号，PostgreSQL/SQLite 使用双引号
    pub fn quote_identifier(&self, name: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("`{}`", name),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", name),
        }
    }

    /// 获取 LIMIT 语法
    ///
    /// 所有数据库都支持 LIMIT，但语法略有不同
    pub fn limit_syntax(&self, limit: u64, offset: Option<u64>) -> String {
        match self.db_type {
            DatabaseType::MySQL | DatabaseType::PostgreSQL | DatabaseType::SQLite => {
                if let Some(off) = offset {
                    format!("LIMIT {} OFFSET {}", limit, off)
                } else {
                    format!("LIMIT {}", limit)
                }
            }
        }
    }

    /// 获取 INSERT 返回 ID 的语法
    ///
    /// MySQL 使用 LAST_INSERT_ID()
    /// PostgreSQL 使用 RETURNING id
    /// SQLite 使用 last_insert_rowid() 或 RETURNING
    pub fn insert_returning_id(&self, _table: &str, pk: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => "SELECT LAST_INSERT_ID()".to_string(),
            DatabaseType::PostgreSQL => format!("RETURNING {}", pk),
            DatabaseType::SQLite => format!("SELECT last_insert_rowid()"),
        }
    }

    /// 获取当前时间函数
    pub fn current_timestamp(&self) -> &str {
        match self.db_type {
            DatabaseType::MySQL => "NOW()",
            DatabaseType::PostgreSQL => "CURRENT_TIMESTAMP",
            DatabaseType::SQLite => "datetime('now')",
        }
    }

    /// 获取布尔值表示
    ///
    /// MySQL 使用 1/0，PostgreSQL 使用 true/false
    pub fn boolean_literal(&self, value: bool) -> String {
        match self.db_type {
            DatabaseType::MySQL => if value { "1" } else { "0" }.to_string(),
            DatabaseType::PostgreSQL => if value { "TRUE" } else { "FALSE" }.to_string(),
            DatabaseType::SQLite => if value { "1" } else { "0" }.to_string(),
        }
    }

    /// 获取字符串连接操作符
    ///
    /// MySQL 使用 CONCAT()，PostgreSQL/SQLite 使用 ||
    pub fn string_concat(&self, parts: &[&str]) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("CONCAT({})", parts.join(", ")),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => parts.join(" || "),
        }
    }

    /// 获取自动递增列定义
    pub fn auto_increment(&self) -> &str {
        match self.db_type {
            DatabaseType::MySQL => "AUTO_INCREMENT",
            DatabaseType::PostgreSQL => "SERIAL",
            DatabaseType::SQLite => "AUTOINCREMENT",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_type_from_str() {
        assert_eq!(DatabaseType::from_str("mysql"), DatabaseType::MySQL);
        assert_eq!(DatabaseType::from_str("postgres"), DatabaseType::PostgreSQL);
        assert_eq!(DatabaseType::from_str("pgsql"), DatabaseType::PostgreSQL);
        assert_eq!(DatabaseType::from_str("postgresql"), DatabaseType::PostgreSQL);
        assert_eq!(DatabaseType::from_str("sqlite"), DatabaseType::SQLite);
    }

    #[test]
    fn test_placeholder() {
        assert_eq!(DatabaseType::MySQL.placeholder(1), "?");
        assert_eq!(DatabaseType::PostgreSQL.placeholder(1), "$1");
        assert_eq!(DatabaseType::PostgreSQL.placeholder(2), "$2");
        assert_eq!(DatabaseType::SQLite.placeholder(1), "?");
    }

    #[test]
    fn test_sql_dialect() {
        let mysql = SqlDialect::new(DatabaseType::MySQL);
        assert_eq!(mysql.quote_identifier("table"), "`table`");
        assert_eq!(mysql.current_timestamp(), "NOW()");

        let pg = SqlDialect::new(DatabaseType::PostgreSQL);
        assert_eq!(pg.quote_identifier("table"), "\"table\"");
        assert_eq!(pg.current_timestamp(), "CURRENT_TIMESTAMP");
    }
}
