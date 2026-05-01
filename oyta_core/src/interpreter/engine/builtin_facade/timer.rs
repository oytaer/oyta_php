//! Timer 门面类方法实现
//!
//! 提供定时器操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Timer::after 方法实现
pub fn timer_after(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let delay_ms = args.first()
        .map(|v| v.to_int())
        .unwrap_or(0);
    Ok(Value::Int(delay_ms))
}

/// Timer::every 方法实现
pub fn timer_every(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let interval_ms = args.first()
        .map(|v| v.to_int())
        .unwrap_or(0);
    Ok(Value::Int(interval_ms))
}

/// Timer::cancel 方法实现
pub fn timer_cancel(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(true))
}

/// 获取所有 Timer 门面方法
pub fn get_timer_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("after", timer_after),
        ("every", timer_every),
        ("cancel", timer_cancel),
    ]
}
