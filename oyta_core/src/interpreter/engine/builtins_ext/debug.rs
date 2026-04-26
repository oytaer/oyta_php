//! 调试函数模块
//!
//! 包含 var_dump、print_r、json、dd、dump 等调试函数

use anyhow::Result;

use crate::interpreter::value::Value;

/// var_dump — 打印变量的相关信息
pub fn builtin_var_dump(args: &[Value]) -> Result<Value> {
    for arg in args {
        tracing::info!("var_dump: {:?}", arg);
    }
    Ok(Value::Null)
}

/// print_r — 打印关于变量的易于理解的信息
pub fn builtin_print_r(args: &[Value]) -> Result<Value> {
    for arg in args {
        tracing::info!("print_r: {:?}", arg);
    }
    Ok(Value::Bool(true))
}

/// json — 快捷 JSON 编码（OYTAPHP 扩展）
pub fn builtin_json(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(data) => Ok(Value::String(data.to_json_string())),
        None => Ok(Value::String("null".to_string())),
    }
}

/// dd — 转储变量并终止脚本（OYTAPHP 扩展）
pub fn builtin_dd(args: &[Value]) -> Result<Value> {
    for arg in args {
        tracing::info!("dd: {:?}", arg);
    }
    Ok(Value::Null)
}

/// dump — 转储变量（OYTAPHP 扩展）
pub fn builtin_dump(args: &[Value]) -> Result<Value> {
    for arg in args {
        tracing::info!("dump: {:?}", arg);
    }
    Ok(Value::Null)
}
