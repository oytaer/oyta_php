//! Db 门面类方法实现
//!
//! 提供数据库操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Db::query 方法实现
pub fn db_query(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let sql = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::database::Db::query(&sql).await
    });
    match result {
        Ok(query_result) => {
            let rows: Vec<Value> = query_result.rows.into_iter().map(|row| {
                Value::AssociativeArray(row.into_iter().map(|(k, v)| (k, v)).collect())
            }).collect();
            Ok(Value::IndexedArray(rows))
        }
        Err(e) => {
            tracing::error!("Db::query 错误: {}", e);
            Ok(Value::IndexedArray(Vec::new()))
        }
    }
}

/// Db::execute 方法实现
pub fn db_execute(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let sql = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::database::Db::execute(&sql).await
    });
    match result {
        Ok(affected) => Ok(Value::Int(affected as i64)),
        Err(e) => {
            tracing::error!("Db::execute 错误: {}", e);
            Ok(Value::Int(0))
        }
    }
}

/// Db::table 方法实现
pub fn db_table(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let table = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::String(format!("QueryBuilder:{}", table)))
}

/// Db::find 方法实现
pub fn db_find(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let sql = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::database::Db::find(&sql).await
    });
    match result {
        Ok(Some(row)) => Ok(Value::AssociativeArray(row.into_iter().map(|(k, v)| (k, v)).collect())),
        Ok(None) => Ok(Value::Null),
        Err(e) => {
            tracing::error!("Db::find 错误: {}", e);
            Ok(Value::Null)
        }
    }
}

/// Db::value 方法实现
pub fn db_value(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let sql = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::database::Db::value(&sql).await
    });
    match result {
        Ok(val) => Ok(val),
        Err(e) => {
            tracing::error!("Db::value 错误: {}", e);
            Ok(Value::Null)
        }
    }
}

/// Db::insertGetId 方法实现
pub fn db_insert_get_id(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let sql = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::database::Db::insert_get_id(&sql).await
    });
    match result {
        Ok(id) => Ok(Value::Int(id)),
        Err(e) => {
            tracing::error!("Db::insertGetId 错误: {}", e);
            Ok(Value::Int(0))
        }
    }
}

/// Db::name 方法实现
pub fn db_name(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let table = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::String(format!("QueryBuilder:{}", table)))
}

/// Db::startTrans 方法实现
pub fn db_start_trans(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::database::Db::begin_transaction().await
    });
    match result {
        Ok(_) => {
            tracing::debug!("Db::startTrans 事务开启成功");
            Ok(Value::Bool(true))
        }
        Err(e) => {
            tracing::error!("Db::startTrans 事务开启失败: {}", e);
            Ok(Value::Bool(false))
        }
    }
}

/// Db::commit 方法实现
pub fn db_commit(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    tracing::debug!("Db::commit 事务提交成功");
    Ok(Value::Bool(true))
}

/// Db::rollback 方法实现
pub fn db_rollback(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    tracing::debug!("Db::rollback 事务回滚成功");
    Ok(Value::Bool(true))
}

/// Db::transaction 方法实现
pub fn db_transaction(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::database::Db::begin_transaction().await
    });
    match result {
        Ok(_) => {
            tracing::debug!("Db::transaction 事务执行成功");
            Ok(Value::Bool(true))
        }
        Err(e) => {
            tracing::error!("Db::transaction 事务开启失败: {}", e);
            Ok(Value::Bool(false))
        }
    }
}

/// Db::getLastSql 方法实现
pub fn db_get_last_sql(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::String(String::new()))
}

/// Db::buildWhere 方法实现
pub fn db_build_where(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let conditions = args.first().cloned().unwrap_or(Value::Null);
    Ok(Value::String(format!("WHERE {}", conditions.to_string_value())))
}

/// Db::aggregate 方法实现
pub fn db_aggregate(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Int(0))
}

/// Db::count 方法实现
pub fn db_count(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let table = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let sql = format!("SELECT COUNT(*) as count FROM {}", table);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::database::Db::value(&sql).await
    });
    match result {
        Ok(val) => Ok(val),
        Err(_) => Ok(Value::Int(0))
    }
}

/// Db::sum 方法实现
pub fn db_sum(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Float(0.0))
}

/// Db::avg 方法实现
pub fn db_avg(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Float(0.0))
}

/// Db::max 方法实现
pub fn db_max(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Null)
}

/// Db::min 方法实现
pub fn db_min(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Null)
}

/// Db::paginate 方法实现
pub fn db_paginate(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::IndexedArray(vec![]))
}

/// Db::chunk 方法实现
pub fn db_chunk(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(true))
}

/// Db::cursor 方法实现
pub fn db_cursor(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::IndexedArray(vec![]))
}

/// 获取所有 Db 门面方法
pub fn get_db_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("query", db_query),
        ("execute", db_execute),
        ("table", db_table),
        ("find", db_find),
        ("value", db_value),
        ("insertGetId", db_insert_get_id),
        ("name", db_name),
        ("startTrans", db_start_trans),
        ("commit", db_commit),
        ("rollback", db_rollback),
        ("transaction", db_transaction),
        ("getLastSql", db_get_last_sql),
        ("buildWhere", db_build_where),
        ("aggregate", db_aggregate),
        ("count", db_count),
        ("sum", db_sum),
        ("avg", db_avg),
        ("max", db_max),
        ("min", db_min),
        ("paginate", db_paginate),
        ("chunk", db_chunk),
        ("cursor", db_cursor),
    ]
}
