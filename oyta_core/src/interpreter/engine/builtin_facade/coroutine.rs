//! Coroutine 门面类方法实现
//!
//! 提供协程操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Coroutine::create 方法实现
pub fn coroutine_create(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Int(1))
}

/// Coroutine::wait 方法实现
pub fn coroutine_wait(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(true))
}

/// 获取所有 Coroutine 门面方法
pub fn get_coroutine_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("create", coroutine_create),
        ("wait", coroutine_wait),
    ]
}
