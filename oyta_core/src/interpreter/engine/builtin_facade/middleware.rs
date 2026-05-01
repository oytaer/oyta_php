//! Middleware 门面类方法实现
//!
//! 提供中间件操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Middleware::add 方法实现
pub fn middleware_add(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(true))
}

/// Middleware::remove 方法实现
pub fn middleware_remove(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(true))
}

/// Middleware::has 方法实现
pub fn middleware_has(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(false))
}

/// Middleware::clear 方法实现
pub fn middleware_clear(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(true))
}

/// 获取所有 Middleware 门面方法
pub fn get_middleware_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("add", middleware_add),
        ("remove", middleware_remove),
        ("has", middleware_has),
        ("clear", middleware_clear),
    ]
}
