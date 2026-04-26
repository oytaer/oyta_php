//! 数据库查询执行模块
//!
//! 提供实际的 SQL 查询执行能力
//! 基于 sqlx 的 MySQL 连接池执行查询
//! 支持：SELECT/INSERT/UPDATE/DELETE 操作
//! 支持：参数绑定、事务、分页查询

use anyhow::{Context, Result};
use sqlx::{Column, MySql, Row};
use std::collections::HashMap;
use std::sync::Arc;

use crate::interpreter::value::Value;
use super::connection::get_manager_mut;

/// 查询结果
///
/// 封装 SQL 查询的返回数据
/// SELECT 查询返回行数据，INSERT/UPDATE/DELETE 返回影响行数
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// 查询到的行数据（每行是一个字段名→值的映射）
    pub rows: Vec<HashMap<String, Value>>,
    /// 影响的行数
    pub affected_rows: u64,
    /// 最后插入的自增 ID
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

/// 执行 SQL 查询（SELECT）
///
/// 使用默认连接池执行 SELECT 查询
/// 自动将 sqlx 的行数据转换为 Value 类型
///
/// # 参数
/// - `sql`: SQL 查询语句
///
/// # 返回
/// 查询结果，包含行数据和元信息
pub async fn query(sql: &str) -> Result<QueryResult> {
    let pool = get_default_pool().await?;
    let rows = sqlx::query(sql)
        .fetch_all(&*pool)
        .await
        .with_context(|| format!("SQL 查询执行失败: {}", sql))?;

    let mut result_rows = Vec::new();
    for row in rows {
        let mut map = HashMap::new();
        for (i, column) in row.columns().iter().enumerate() {
            let col_name = column.name().to_string();
            let value = row_to_value(&row, i);
            map.insert(col_name, value);
        }
        result_rows.push(map);
    }

    Ok(QueryResult {
        rows: result_rows,
        affected_rows: 0,
        last_insert_id: None,
    })
}

/// 执行参数化 SQL 查询（SELECT）
///
/// 使用参数绑定防止 SQL 注入
/// 参数使用 ? 占位符
///
/// # 参数
/// - `sql`: 带占位符的 SQL 查询语句
/// - `params`: 参数值列表
///
/// # 返回
/// 查询结果
pub async fn query_with_params(sql: &str, params: &[Value]) -> Result<QueryResult> {
    let pool = get_default_pool().await?;
    let mut query_builder = sqlx::query(sql);

    for param in params {
        query_builder = bind_value(query_builder, param);
    }

    let rows = query_builder
        .fetch_all(&*pool)
        .await
        .with_context(|| format!("参数化查询执行失败: {}", sql))?;

    let mut result_rows = Vec::new();
    for row in rows {
        let mut map = HashMap::new();
        for (i, column) in row.columns().iter().enumerate() {
            let col_name = column.name().to_string();
            let value = row_to_value(&row, i);
            map.insert(col_name, value);
        }
        result_rows.push(map);
    }

    Ok(QueryResult {
        rows: result_rows,
        affected_rows: 0,
        last_insert_id: None,
    })
}

/// 执行 SQL 更新（INSERT/UPDATE/DELETE）
///
/// 使用默认连接池执行写操作
/// 返回受影响的行数和最后插入 ID
///
/// # 参数
/// - `sql`: SQL 更新语句
///
/// # 返回
/// 受影响的行数
pub async fn execute(sql: &str) -> Result<u64> {
    let pool = get_default_pool().await?;
    let result = sqlx::query(sql)
        .execute(&*pool)
        .await
        .with_context(|| format!("SQL 执行失败: {}", sql))?;

    Ok(result.rows_affected())
}

/// 执行参数化 SQL 更新
///
/// # 参数
/// - `sql`: 带占位符的 SQL 更新语句
/// - `params`: 参数值列表
///
/// # 返回
/// 受影响的行数
pub async fn execute_with_params(sql: &str, params: &[Value]) -> Result<u64> {
    let pool = get_default_pool().await?;
    let mut query_builder = sqlx::query(sql);

    for param in params {
        query_builder = bind_value(query_builder, param);
    }

    let result = query_builder
        .execute(&*pool)
        .await
        .with_context(|| format!("参数化执行失败: {}", sql))?;

    Ok(result.rows_affected())
}

/// 执行 INSERT 并返回自增 ID
///
/// # 参数
/// - `sql`: INSERT SQL 语句
///
/// # 返回
/// 最后插入的自增 ID
pub async fn insert_get_id(sql: &str) -> Result<i64> {
    let pool = get_default_pool().await?;
    let result = sqlx::query(sql)
        .execute(&*pool)
        .await
        .with_context(|| format!("INSERT 执行失败: {}", sql))?;

    Ok(result.last_insert_id() as i64)
}

/// 查询单行数据
///
/// # 参数
/// - `sql`: SQL 查询语句
///
/// # 返回
/// 单行数据，如果无结果返回 None
pub async fn query_one(sql: &str) -> Result<Option<HashMap<String, Value>>> {
    let result = query(sql).await?;
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
pub async fn query_scalar(sql: &str) -> Result<Value> {
    let result = query(sql).await?;
    Ok(result.scalar().cloned().unwrap_or(Value::Null))
}

/// 开启事务
///
/// 返回事务对象，调用者负责提交或回滚
///
/// # 返回
/// 事务对象
pub async fn begin_transaction() -> Result<sqlx::Transaction<'static, MySql>> {
    let pool = get_default_pool().await?;
    let tx = pool.begin().await?;
    Ok(tx)
}

/// 提交事务
///
/// # 参数
/// - `tx`: 要提交的事务
pub async fn commit_transaction(tx: sqlx::Transaction<'static, MySql>) -> Result<()> {
    tx.commit().await?;
    Ok(())
}

/// 回滚事务
///
/// # 参数
/// - `tx`: 要回滚的事务
pub async fn rollback_transaction(tx: sqlx::Transaction<'static, MySql>) -> Result<()> {
    tx.rollback().await?;
    Ok(())
}

/// 获取默认连接池
///
/// 从全局连接管理器获取或创建默认连接池
///
/// # 返回
/// MySQL 连接池的 Arc 引用
async fn get_default_pool() -> Result<Arc<sqlx::MySqlPool>> {
    let mut manager = get_manager_mut();
    manager.get_default_pool().await
}

/// 将 sqlx 行数据中的指定列转换为 Value
///
/// 按列索引获取值，尝试按以下类型顺序解析：
/// i64 → f64 → String → Null
///
/// # 参数
/// - `row`: sqlx 行数据引用
/// - `index`: 列索引
///
/// # 返回
/// 转换后的 Value
fn row_to_value(row: &sqlx::mysql::MySqlRow, index: usize) -> Value {
    if let Ok(v) = row.try_get::<i64, _>(index) {
        return Value::Int(v);
    }
    if let Ok(v) = row.try_get::<f64, _>(index) {
        return Value::Float(v);
    }
    if let Ok(v) = row.try_get::<String, _>(index) {
        return Value::String(v);
    }
    if let Ok(v) = row.try_get::<bool, _>(index) {
        return Value::Bool(v);
    }
    Value::Null
}

/// 将 Value 绑定到 sqlx 查询参数
///
/// 根据 Value 类型选择对应的 bind 方法
/// 支持：Int, Float, String, Bool, Null
///
/// # 参数
/// - `query`: sqlx 查询构建器
/// - `value`: 要绑定的值
///
/// # 返回
/// 绑定参数后的查询构建器
fn bind_value<'q>(
    query: sqlx::query::Query<'q, MySql, sqlx::mysql::MySqlArguments>,
    value: &'q Value,
) -> sqlx::query::Query<'q, MySql, sqlx::mysql::MySqlArguments> {
    match value {
        Value::Int(i) => query.bind(*i),
        Value::Float(f) => query.bind(*f),
        Value::String(s) => query.bind(s.as_str()),
        Value::Bool(b) => query.bind(*b),
        Value::Null => query.bind(Option::<String>::None),
        _ => query.bind(value.to_string_value()),
    }
}
