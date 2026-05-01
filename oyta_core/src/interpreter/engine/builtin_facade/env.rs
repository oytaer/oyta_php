//! Env 门面类方法实现
//!
//! 提供环境变量操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Env::get 方法实现
pub fn env_get(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let default = args.get(1)
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    Ok(Value::String(crate::config::facade::Env::get(&key, &default)))
}

/// Env::has 方法实现
pub fn env_has(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    Ok(Value::Bool(crate::config::facade::Env::has(&key)))
}

/// 获取所有 Env 门面方法
pub fn get_env_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("get", env_get),
        ("has", env_has),
    ]
}
