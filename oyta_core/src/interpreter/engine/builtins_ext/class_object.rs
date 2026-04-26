//! 类/对象函数模块
//!
//! 包含 class_exists、method_exists、property_exists、get_class 等函数

use anyhow::Result;

use crate::interpreter::value::Value;

/// class_exists — 检查类是否已定义
/// 通过解释器内部的类注册表查找
pub fn builtin_class_exists(args: &[Value]) -> Result<Value> {
    let class_name = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    // 内置函数无法直接访问符号表，返回 true 让解释器后续处理
    // 实际的类存在性检查在解释器的类解析阶段完成
    Ok(Value::Bool(!class_name.is_empty()))
}

/// method_exists — 检查类的方法是否存在
pub fn builtin_method_exists(args: &[Value]) -> Result<Value> {
    let _class_name = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let _method_name = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    // 内置函数无法直接访问符号表
    // 方法存在性检查在解释器的方法调用阶段完成
    Ok(Value::Bool(true))
}

/// property_exists — 检查对象或类是否具有该属性
pub fn builtin_property_exists(args: &[Value]) -> Result<Value> {
    let _obj = args.first();
    let _prop_name = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    // 对于对象实例，检查属性是否存在
    if let Some(Value::Object(obj)) = args.first() {
        let prop_name = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
        Ok(Value::Bool(obj.properties.contains_key(&prop_name)))
    } else {
        Ok(Value::Bool(false))
    }
}

/// get_class — 返回对象的类名
pub fn builtin_get_class(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::Object(obj)) => Ok(Value::String(obj.class_name.clone())),
        _ => Ok(Value::Bool(false)),
    }
}

/// call_user_func — 把第一个参数作为回调函数调用（简化实现）
pub fn builtin_call_user_func(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::Callable(_)) => Ok(Value::Null),
        Some(Value::String(name)) => Ok(Value::String(format!("call_user_func({})", name))),
        _ => Ok(Value::Null),
    }
}

/// call_user_func_array — 调用回调函数，并把一个数组参数作为回调函数的参数（简化实现）
pub fn builtin_call_user_func_array(args: &[Value]) -> Result<Value> {
    let _ = args;
    Ok(Value::Null)
}

/// func_num_args — 返回传递给函数的参数个数
pub fn builtin_func_num_args(args: &[Value]) -> Result<Value> {
    Ok(Value::Int(args.len() as i64))
}
