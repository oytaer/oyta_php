//! Config 门面类方法实现
//!
//! 提供配置操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};
use crate::symbol_table::types::ConfigValue;

use super::types::FacadeMethod;

/// 将 ConfigValue 递归转换为 Value
fn config_value_to_value(value: ConfigValue) -> Value {
    match value {
        ConfigValue::String(s) => Value::String(s),
        ConfigValue::Int(i) => Value::Int(i),
        ConfigValue::Float(f) => Value::Float(f),
        ConfigValue::Bool(b) => Value::Bool(b),
        ConfigValue::Null => Value::Null,
        ConfigValue::IndexedArray(arr) => {
            Value::IndexedArray(arr.iter()
                .map(|v| config_value_to_value(v.clone()))
                .collect())
        }
        ConfigValue::AssociativeArray(map) => {
            Value::AssociativeArray(map.iter()
                .map(|(k, v)| (k.clone(), config_value_to_value(v.clone())))
                .collect())
        }
    }
}

/// Config::get 方法实现
pub fn config_get(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let default = args.get(1).cloned().unwrap_or(Value::Null);

    match crate::config::facade::Config::get(&key) {
        Some(value) => Ok(config_value_to_value(value)),
        None => Ok(default),
    }
}

/// Config::set 方法实现
pub fn config_set(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let value = args.get(1).cloned().unwrap_or(Value::Null);

    let config_value = match value {
        Value::String(s) => ConfigValue::String(s),
        Value::Int(i) => ConfigValue::Int(i),
        Value::Float(f) => ConfigValue::Float(f),
        Value::Bool(b) => ConfigValue::Bool(b),
        _ => ConfigValue::Null,
    };

    crate::config::facade::Config::set(&key, config_value);
    Ok(Value::Bool(true))
}

/// Config::has 方法实现
pub fn config_has(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    Ok(Value::Bool(crate::config::facade::Config::has(&key)))
}

/// 获取所有 Config 门面方法
pub fn get_config_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("get", config_get),
        ("set", config_set),
        ("has", config_has),
    ]
}
