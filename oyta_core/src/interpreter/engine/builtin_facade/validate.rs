//! Validate 门面类方法实现
//!
//! 提供验证操作的静态方法

use std::collections::HashMap;

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Validate::check 方法实现
pub fn validate_check(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(true))
}

/// Validate::make 方法实现
pub fn validate_make(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Object(ObjectInstance {
        class_name: "Validator".to_string(),
        properties: HashMap::new(),
    }))
}

/// 获取所有 Validate 门面方法
pub fn get_validate_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("check", validate_check),
        ("make", validate_make),
    ]
}
