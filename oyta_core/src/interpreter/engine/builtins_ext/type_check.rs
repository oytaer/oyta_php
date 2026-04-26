//! 类型检测函数模块
//!
//! 包含 is_null、is_array、is_string 等类型检测函数

use anyhow::Result;

use crate::interpreter::value::Value;

/// is_null — 检测变量是否为 null
pub fn builtin_is_null(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(args.first().map(|v| v.is_null()).unwrap_or(true)))
}

/// is_array — 检测变量是否是数组
pub fn builtin_is_array(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(
        args.first(),
        Some(Value::IndexedArray(_)) | Some(Value::AssociativeArray(_))
    )))
}

/// is_string — 检测变量是否是字符串
pub fn builtin_is_string(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args.first(), Some(Value::String(_)))))
}

/// is_int — 检测变量是否是整数
pub fn builtin_is_int(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args.first(), Some(Value::Int(_)))))
}

/// is_float — 检测变量是否是浮点数
pub fn builtin_is_float(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args.first(), Some(Value::Float(_)))))
}

/// is_bool — 检测变量是否是布尔值
pub fn builtin_is_bool(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args.first(), Some(Value::Bool(_)))))
}

/// is_object — 检测变量是否是对象
pub fn builtin_is_object(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(args.first(), Some(Value::Object(_)))))
}

/// is_callable — 检测变量是否是可调用的
pub fn builtin_is_callable(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(matches!(
        args.first(),
        Some(Value::Callable(_))
    )))
}

/// is_numeric — 检测变量是否为数字或数字字符串
pub fn builtin_is_numeric(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::Int(_)) | Some(Value::Float(_)) => Ok(Value::Bool(true)),
        Some(Value::String(s)) => Ok(Value::Bool(s.parse::<f64>().is_ok())),
        _ => Ok(Value::Bool(false)),
    }
}

/// is_empty — 检测变量是否为空
pub fn builtin_is_empty(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(
        !args.first().map(|v| v.is_truthy()).unwrap_or(true),
    ))
}
