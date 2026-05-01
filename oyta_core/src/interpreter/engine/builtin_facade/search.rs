//! Search 门面类方法实现
//!
//! 提供搜索操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Search::index 方法实现
pub fn search_index(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(true))
}

/// Search::query 方法实现
pub fn search_query(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::IndexedArray(vec![]))
}

/// 获取所有 Search 门面方法
pub fn get_search_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("index", search_index),
        ("query", search_query),
    ]
}
