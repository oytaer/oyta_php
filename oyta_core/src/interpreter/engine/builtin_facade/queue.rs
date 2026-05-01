//! Queue 门面类方法实现
//!
//! 提供队列操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Queue::push 方法实现
pub fn queue_push(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let job_name = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let queue = args.get(2)
        .map(|v| v.to_string_value())
        .unwrap_or_else(|| "default".to_string());
    tracing::debug!("Queue::push - 任务: {}, 队列: {}", job_name, queue);
    Ok(Value::Bool(true))
}

/// Queue::later 方法实现
pub fn queue_later(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let delay = args.first()
        .map(|v| v.to_int())
        .unwrap_or(0);
    let job_name = args.get(1)
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let queue = args.get(3)
        .map(|v| v.to_string_value())
        .unwrap_or_else(|| "default".to_string());
    tracing::debug!("Queue::later - 延迟: {}秒, 任务: {}, 队列: {}", delay, job_name, queue);
    Ok(Value::Bool(true))
}

/// Queue::pop 方法实现
pub fn queue_pop(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Null)
}

/// Queue::len 方法实现
pub fn queue_len(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Int(0))
}

/// Queue::clear 方法实现
pub fn queue_clear(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let queue = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_else(|| "default".to_string());
    tracing::debug!("Queue::clear - 队列: {}", queue);
    Ok(Value::Bool(true))
}

/// Queue::failedJobs 方法实现
pub fn queue_failed_jobs(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::IndexedArray(vec![]))
}

/// Queue::retry 方法实现
pub fn queue_retry(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let queue = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_else(|| "default".to_string());
    let job_id = args.get(1)
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    tracing::debug!("Queue::retry - 队列: {}, 任务ID: {}", queue, job_id);
    Ok(Value::Bool(true))
}

/// 获取所有 Queue 门面方法
pub fn get_queue_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("push", queue_push),
        ("later", queue_later),
        ("pop", queue_pop),
        ("len", queue_len),
        ("clear", queue_clear),
        ("failedJobs", queue_failed_jobs),
        ("retry", queue_retry),
    ]
}
