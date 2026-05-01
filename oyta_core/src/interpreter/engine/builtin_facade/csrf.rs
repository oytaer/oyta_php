//! Csrf 门面类方法实现
//!
//! 提供 CSRF 保护的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Csrf::token 方法实现
pub fn csrf_token(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let token = crate::security::Csrf::token();
    Ok(Value::String(token))
}

/// Csrf::verify 方法实现
pub fn csrf_verify(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let token = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let result = crate::security::Csrf::verify(&token, true);
    Ok(Value::Bool(result))
}

/// Csrf::refresh 方法实现
pub fn csrf_refresh(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let token = crate::security::Csrf::token();
    Ok(Value::String(token))
}

/// 获取所有 Csrf 门面方法
pub fn get_csrf_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("token", csrf_token),
        ("verify", csrf_verify),
        ("refresh", csrf_refresh),
    ]
}
