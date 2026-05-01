//! Event 门面类方法实现
//!
//! 提供事件操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Event::dispatch 方法实现
pub fn event_dispatch(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let event_name = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    tracing::debug!("分发事件: {}", event_name);
    Ok(Value::Bool(true))
}

/// Event::listen 方法实现
pub fn event_listen(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let event_name = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    tracing::debug!("注册事件监听: {}", event_name);
    Ok(Value::Bool(true))
}

/// 获取所有 Event 门面方法
pub fn get_event_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("dispatch", event_dispatch),
        ("listen", event_listen),
    ]
}
