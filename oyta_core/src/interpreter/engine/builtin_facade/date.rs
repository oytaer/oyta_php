//! Date 门面类方法实现
//!
//! 提供日期操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Date::now 方法实现
pub fn date_now(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let now = crate::datetime::Date::now().to_rfc3339();
    Ok(Value::String(now))
}

/// Date::format 方法实现
pub fn date_format(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let format = args.first().map(|v| v.to_string_value()).unwrap_or_else(|| "Y-m-d H:i:s".to_string());
    let formatted = crate::datetime::Date::format(&format);
    Ok(Value::String(formatted))
}

/// Date::parse 方法实现
pub fn date_parse(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let date_str = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    match crate::datetime::Date::parse(&date_str) {
        Some(dt) => Ok(Value::String(dt.to_rfc3339())),
        None => Ok(Value::Bool(false)),
    }
}

/// Date::timestamp 方法实现
pub fn date_timestamp(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let ts = crate::datetime::Date::timestamp();
    Ok(Value::Int(ts))
}

/// 获取所有 Date 门面方法
pub fn get_date_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("now", date_now),
        ("format", date_format),
        ("parse", date_parse),
        ("timestamp", date_timestamp),
    ]
}
