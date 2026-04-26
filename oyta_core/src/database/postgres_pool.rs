//! PostgreSQL 数据库驱动实现
//!
//! 实现 DatabasePool trait，提供 PostgreSQL 数据库支持
//! 基于 sqlx 的 PgPool 实现连接池管理
//! 支持：查询、执行、事务、参数绑定等

use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{Column, PgPool, Postgres, Row};
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;

use crate::interpreter::value::Value;
use super::pool_trait::{DatabasePool, DatabaseTransaction, QueryResult};

/// PostgreSQL 连接池包装器
///
/// 封装 sqlx::PgPool，实现 DatabasePool trait
/// 提供线程安全的连接池管理
pub struct PostgresPool {
    /// 底层连接池
    pool: Arc<PgPool>,
    /// 连接池大小
    pool_size: u32,
}

impl PostgresPool {
    /// 创建新的 PostgreSQL 连接池
    ///
    /// # 参数
    /// - `dsn`: 数据库连接字符串
    ///   格式: postgres://user:password@host:port/database
    /// - `pool_size`: 连接池大小
    ///
    /// # 返回
    /// 创建好的连接池实例
    ///
    /// # 示例
    /// ```ignore
    /// let pool = PostgresPool::new("postgres://root:123456@127.0.0.1:5432/test", 10).await?;
    /// ```
    pub async fn new(dsn: &str, pool_size: u32) -> Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(pool_size)
            .connect(dsn)
            .await
            .with_context(|| format!("无法连接 PostgreSQL 数据库: {}", dsn))?;

        tracing::info!("PostgreSQL 连接池创建成功，大小: {}", pool_size);

        Ok(Self {
            pool: Arc::new(pool),
            pool_size,
        })
    }

    /// 从现有 PgPool 创建包装器
    ///
    /// # 参数
    /// - `pool`: 已有的 PgPool 实例
    pub fn from_pool(pool: PgPool, pool_size: u32) -> Self {
        Self {
            pool: Arc::new(pool),
            pool_size,
        }
    }

    /// 获取底层连接池引用
    pub fn inner(&self) -> &PgPool {
        &self.pool
    }

    /// 将 PostgreSQL 行数据转换为 Value 映射
    ///
    /// # 参数
    /// - `row`: sqlx 的行数据
    ///
    /// # 返回
    /// 字段名到 Value 的映射
    fn row_to_map(row: &sqlx::postgres::PgRow) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        for (i, column) in row.columns().iter().enumerate() {
            let col_name = column.name().to_string();
            let value = Self::row_column_to_value(row, i);
            map.insert(col_name, value);
        }
        map
    }

    /// 将行中的单列转换为 Value
    ///
    /// 按照类型优先级尝试解析：
    /// i64 → f64 → bool → String → Vec<u8> → Null
    ///
    /// # 参数
    /// - `row`: 行数据
    /// - `index`: 列索引
    ///
    /// # 返回
    /// 转换后的 Value
    fn row_column_to_value(row: &sqlx::postgres::PgRow, index: usize) -> Value {
        // 尝试解析为整数
        if let Ok(v) = row.try_get::<i64, _>(index) {
            return Value::Int(v);
        }
        // 尝试解析为浮点数
        if let Ok(v) = row.try_get::<f64, _>(index) {
            return Value::Float(v);
        }
        // 尝试解析为布尔值
        if let Ok(v) = row.try_get::<bool, _>(index) {
            return Value::Bool(v);
        }
        // 尝试解析为字符串
        if let Ok(v) = row.try_get::<String, _>(index) {
            return Value::String(v);
        }
        // 尝试解析为字节数组（用于 BLOB/BYTEA）
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
        // 尝试解析为 JSON
        if let Ok(v) = row.try_get::<serde_json::Value, _>(index) {
            return json_to_value(&v);
        }
        // 默认返回 Null
        Value::Null
    }
}

/// PostgreSQL 事务包装器
///
/// 封装 sqlx 事务，实现 DatabaseTransaction trait
pub struct PostgresTransaction {
    /// 事务对象
    tx: Option<sqlx::Transaction<'static, Postgres>>,
}

impl PostgresTransaction {
    /// 创建新的事务包装器
    pub fn new(tx: sqlx::Transaction<'static, Postgres>) -> Self {
        Self { tx: Some(tx) }
    }
}

#[async_trait]
impl DatabasePool for PostgresPool {
    /// 执行 SQL 查询
    async fn query(&self, sql: &str) -> Result<QueryResult> {
        tracing::debug!("PostgreSQL 查询: {}", sql);

        let rows = sqlx::query(sql)
            .fetch_all(&*self.pool)
            .await
            .with_context(|| format!("PostgreSQL 查询失败: {}", sql))?;

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
    ///
    /// PostgreSQL 使用 $1, $2... 占位符
    /// 此方法会自动转换 ? 为 $N 格式
    async fn query_with_params(&self, sql: &str, params: &[Value]) -> Result<QueryResult> {
        // 将 ? 占位符转换为 $1, $2... 格式
        let pg_sql = convert_placeholders_to_pg(sql);
        tracing::debug!("PostgreSQL 参数化查询: {} (参数: {})", pg_sql, params.len());

        let mut query_builder = sqlx::query(&pg_sql);

        for param in params {
            query_builder = bind_value_pg(query_builder, param);
        }

        let rows = query_builder
            .fetch_all(&*self.pool)
            .await
            .with_context(|| format!("PostgreSQL 参数化查询失败: {}", pg_sql))?;

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
        tracing::debug!("PostgreSQL 执行: {}", sql);

        let result = sqlx::query(sql)
            .execute(&*self.pool)
            .await
            .with_context(|| format!("PostgreSQL 执行失败: {}", sql))?;

        Ok(result.rows_affected())
    }

    /// 执行参数化更新
    async fn execute_with_params(&self, sql: &str, params: &[Value]) -> Result<u64> {
        let pg_sql = convert_placeholders_to_pg(sql);
        tracing::debug!("PostgreSQL 参数化执行: {} (参数: {})", pg_sql, params.len());

        let mut query_builder = sqlx::query(&pg_sql);

        for param in params {
            query_builder = bind_value_pg(query_builder, param);
        }

        let result = query_builder
            .execute(&*self.pool)
            .await
            .with_context(|| format!("PostgreSQL 参数化执行失败: {}", pg_sql))?;

        Ok(result.rows_affected())
    }

    /// 执行 INSERT 并返回自增 ID
    ///
    /// PostgreSQL 使用 RETURNING 子句返回 ID
    async fn insert_get_id(&self, sql: &str) -> Result<i64> {
        tracing::debug!("PostgreSQL INSERT: {}", sql);

        // 如果 SQL 不包含 RETURNING，添加它
        let final_sql = if sql.to_uppercase().contains("RETURNING") {
            sql.to_string()
        } else {
            format!("{} RETURNING id", sql)
        };

        let row = sqlx::query(&final_sql)
            .fetch_one(&*self.pool)
            .await
            .with_context(|| format!("PostgreSQL INSERT 失败: {}", final_sql))?;

        // 获取返回的 ID
        let id: i64 = row.try_get(0).unwrap_or(0);
        Ok(id)
    }

    /// 开启事务
    async fn begin_transaction(&self) -> Result<Box<dyn DatabaseTransaction + Send>> {
        let tx = self.pool.begin().await?;
        tracing::debug!("PostgreSQL 事务已开启");
        Ok(Box::new(PostgresTransaction::new(tx)))
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
        "postgresql"
    }
}

#[async_trait]
impl DatabaseTransaction for PostgresTransaction {
    /// 提交事务
    async fn commit(mut self: Box<Self>) -> Result<()> {
        if let Some(tx) = self.tx.take() {
            tx.commit().await?;
            tracing::debug!("PostgreSQL 事务已提交");
        }
        Ok(())
    }

    /// 回滚事务
    async fn rollback(mut self: Box<Self>) -> Result<()> {
        if let Some(tx) = self.tx.take() {
            tx.rollback().await?;
            tracing::debug!("PostgreSQL 事务已回滚");
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
            .map(|row| PostgresPool::row_to_map(row))
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

/// 将 ? 占位符转换为 PostgreSQL 的 $1, $2... 格式
///
/// # 参数
/// - `sql`: 包含 ? 占位符的 SQL 语句
///
/// # 返回
/// 转换后的 SQL 语句
fn convert_placeholders_to_pg(sql: &str) -> String {
    let mut result = String::new();
    let mut param_index = 1;
    let mut chars = sql.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '?' {
            // 检查是否是字符串内的 ?
            result.push_str(&format!("${}", param_index));
            param_index += 1;
        } else if c == '\'' {
            // 处理字符串字面量，跳过内部的 ?
            result.push(c);
            while let Some(&next) = chars.peek() {
                result.push(chars.next().unwrap());
                if next == '\'' {
                    // 检查是否是转义的 ''
                    if chars.peek() == Some(&'\'') {
                        continue;
                    }
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// 将 Value 绑定到 PostgreSQL 查询参数
///
/// # 参数
/// - `query`: sqlx 查询构建器
/// - `value`: 要绑定的值
///
/// # 返回
/// 绑定参数后的查询构建器
fn bind_value_pg<'q>(
    query: sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments>,
    value: &'q Value,
) -> sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments> {
    match value {
        Value::Int(i) => query.bind(*i),
        Value::Float(f) => query.bind(*f),
        Value::String(s) => query.bind(s.as_str()),
        Value::Bool(b) => query.bind(*b),
        Value::Null => query.bind(Option::<String>::None),
        // 对于数组，转换为 JSON
        Value::IndexedArray(arr) => {
            let json_arr: Vec<serde_json::Value> = arr
                .iter()
                .map(|v| value_to_json(v))
                .collect();
            query.bind(serde_json::Value::Array(json_arr))
        }
        Value::AssociativeArray(map) => {
            let json_obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            query.bind(serde_json::Value::Object(json_obj))
        }
        _ => query.bind(value.to_string_value()),
    }
}

/// 将 Value 转换为 JSON 值
fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(i) => serde_json::Value::Number((*i).into()),
        Value::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(*f) {
                serde_json::Value::Number(n)
            } else {
                serde_json::Value::Null
            }
        }
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::IndexedArray(arr) => {
            serde_json::Value::Array(arr.iter().map(value_to_json).collect())
        }
        Value::AssociativeArray(map) => {
            serde_json::Value::Object(
                map.iter()
                    .map(|(k, v)| (k.clone(), value_to_json(v)))
                    .collect(),
            )
        }
        _ => serde_json::Value::Null,
    }
}

/// 将 JSON 值转换为 Value
fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::String(n.to_string())
            }
        }
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            Value::IndexedArray(arr.iter().map(json_to_value).collect())
        }
        serde_json::Value::Object(obj) => {
            Value::AssociativeArray(
                obj.iter()
                    .map(|(k, v)| (k.clone(), json_to_value(v)))
                    .collect(),
            )
        }
    }
}

/// PostgreSQL 连接池配置
///
/// 用于配置连接池参数
pub struct PgPoolConfig {
    /// 最大连接数
    pub max_connections: u32,
    /// 最小连接数
    pub min_connections: u32,
    /// 连接超时时间（秒）
    pub connect_timeout: u64,
    /// 空闲连接超时时间（秒）
    pub idle_timeout: u64,
    /// 最大生命周期（秒）
    pub max_lifetime: u64,
}

impl Default for PgPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 0,
            connect_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 1800,
        }
    }
}

impl PgPoolConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置最大连接数
    pub fn max_connections(mut self, n: u32) -> Self {
        self.max_connections = n;
        self
    }

    /// 设置最小连接数
    pub fn min_connections(mut self, n: u32) -> Self {
        self.min_connections = n;
        self
    }

    /// 设置连接超时时间
    pub fn connect_timeout(mut self, secs: u64) -> Self {
        self.connect_timeout = secs;
        self
    }

    /// 构建连接池
    pub async fn build(&self, dsn: &str) -> Result<PostgresPool> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(self.connect_timeout))
            .idle_timeout(std::time::Duration::from_secs(self.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(self.max_lifetime))
            .connect(dsn)
            .await
            .with_context(|| format!("无法创建 PostgreSQL 连接池: {}", dsn))?;

        Ok(PostgresPool::from_pool(pool, self.max_connections))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_placeholders() {
        assert_eq!(
            convert_placeholders_to_pg("SELECT * FROM users WHERE id = ?"),
            "SELECT * FROM users WHERE id = $1"
        );
        assert_eq!(
            convert_placeholders_to_pg("SELECT * FROM users WHERE id = ? AND name = ?"),
            "SELECT * FROM users WHERE id = $1 AND name = $2"
        );
        // 字符串内的 ? 不应该被转换
        assert_eq!(
            convert_placeholders_to_pg("SELECT * FROM users WHERE name = '?' AND id = ?"),
            "SELECT * FROM users WHERE name = '?' AND id = $1"
        );
    }
}
