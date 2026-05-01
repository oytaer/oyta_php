//! Response 门面类方法实现
//!
//! 提供响应操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Response::json 方法实现
pub fn response_json(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let data = args.first().cloned().unwrap_or(Value::Null);
    let json_str = data.to_json_string();
    Ok(Value::String(json_str))
}

/// Response::redirect 方法实现
pub fn response_redirect(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let url = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    Ok(Value::String(format!("redirect:{}", url)))
}

/// 获取所有 Response 门面方法
pub fn get_response_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("json", response_json),
        ("redirect", response_redirect),
    ]
}
