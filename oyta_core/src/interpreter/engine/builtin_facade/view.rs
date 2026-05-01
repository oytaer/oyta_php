//! View 门面类方法实现
//!
//! 提供视图操作的静态方法

use std::collections::HashMap;

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// View::fetch 方法实现
pub fn view_fetch(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let template = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let result = crate::template::View::fetch(&template, &HashMap::new());
    match result {
        Ok(html) => Ok(Value::String(html)),
        Err(e) => {
            tracing::error!("View::fetch 错误: {}", e);
            Ok(Value::String(String::new()))
        }
    }
}

/// View::assign 方法实现
pub fn view_assign(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let name = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let value = args.get(1).cloned().unwrap_or(Value::Null);
    let json_value = match value {
        Value::String(s) => serde_json::Value::String(s),
        Value::Int(i) => serde_json::Value::Number(i.into()),
        Value::Float(f) => serde_json::json!(f),
        Value::Bool(b) => serde_json::Value::Bool(b),
        Value::Null => serde_json::Value::Null,
        _ => serde_json::Value::Null,
    };
    crate::template::View::assign(&name, json_value);
    Ok(Value::Bool(true))
}

/// View::display 方法实现
pub fn view_display(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let template = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let result = crate::template::View::fetch(&template, &HashMap::new());
    match result {
        Ok(html) => Ok(Value::String(html)),
        Err(e) => {
            tracing::error!("View::display 错误: {}", e);
            Ok(Value::String(String::new()))
        }
    }
}

/// View::has 方法实现
pub fn view_has(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let name = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::Bool(crate::template::View::has(&name)))
}

/// View::get 方法实现
pub fn view_get(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let name = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    match crate::template::View::get(&name) {
        Some(json_val) => {
            match json_val {
                serde_json::Value::String(s) => Ok(Value::String(s)),
                serde_json::Value::Number(n) => Ok(Value::Int(n.as_i64().unwrap_or(0))),
                serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
                serde_json::Value::Null => Ok(Value::Null),
                _ => Ok(Value::String(json_val.to_string())),
            }
        }
        None => Ok(Value::Null),
    }
}

/// View::clear 方法实现
pub fn view_clear(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    crate::template::View::clear();
    Ok(Value::Bool(true))
}

/// 获取所有 View 门面方法
pub fn get_view_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("fetch", view_fetch),
        ("assign", view_assign),
        ("display", view_display),
        ("has", view_has),
        ("get", view_get),
        ("clear", view_clear),
    ]
}
