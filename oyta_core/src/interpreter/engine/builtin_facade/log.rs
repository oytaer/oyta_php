//! Log 门面类方法实现
//!
//! 提供日志操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Log::info 方法实现
pub fn log_info(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let message = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    tracing::info!("{}", message);
    Ok(Value::Bool(true))
}

/// Log::error 方法实现
pub fn log_error(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let message = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    tracing::error!("{}", message);
    Ok(Value::Bool(true))
}

/// Log::warning 方法实现
pub fn log_warning(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let message = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    tracing::warn!("{}", message);
    Ok(Value::Bool(true))
}

/// Log::debug 方法实现
pub fn log_debug(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let message = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    tracing::debug!("{}", message);
    Ok(Value::Bool(true))
}

/// 获取所有 Log 门面方法
pub fn get_log_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("info", log_info),
        ("error", log_error),
        ("warning", log_warning),
        ("debug", log_debug),
    ]
}
