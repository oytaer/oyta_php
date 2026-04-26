//! 类型转换函数模块
//!
//! 包含 intval、floatval、strval、boolval 等类型转换函数

use anyhow::Result;

use crate::interpreter::value::Value;

/// intval — 获取变量的整数值
pub fn builtin_intval(args: &[Value]) -> Result<Value> {
    Ok(Value::Int(args.first().map(|v| v.to_int()).unwrap_or(0)))
}

/// floatval — 获取变量的浮点值
pub fn builtin_floatval(args: &[Value]) -> Result<Value> {
    Ok(Value::Float(
        args.first().map(|v| v.to_float()).unwrap_or(0.0),
    ))
}

/// strval — 获取变量的字符串值
pub fn builtin_strval(args: &[Value]) -> Result<Value> {
    Ok(Value::String(
        args.first()
            .map(|v| v.to_string_value())
            .unwrap_or_default(),
    ))
}

/// boolval — 获取变量的布尔值
pub fn builtin_boolval(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(
        args.first().map(|v| v.is_truthy()).unwrap_or(false),
    ))
}
