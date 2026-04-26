//! 内置函数类型定义模块
//!
//! 定义内置函数的类型签名

use anyhow::Result;

use crate::interpreter::value::Value;

/// 内置函数类型定义
///
/// 每个内置函数接收参数切片，返回 Result<Value>
/// 函数签名：fn(&[Value]) -> Result<Value>
pub type BuiltinFunction = fn(&[Value]) -> Result<Value>;
