//! App 门面类方法实现
//!
//! 提供应用容器操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// App::make 方法实现
pub fn app_make(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let name = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    if crate::container::Container::has(&name) {
        Ok(Value::String(format!("ServiceInstance:{}", name)))
    } else {
        Ok(Value::Null)
    }
}

/// App::bind 方法实现
pub fn app_bind(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let name = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    tracing::info!("App::bind 注册服务: {}", name);
    Ok(Value::Bool(true))
}

/// App::singleton 方法实现
pub fn app_singleton(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let name = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    tracing::info!("App::singleton 注册单例: {}", name);
    Ok(Value::Bool(true))
}

/// App::has 方法实现
pub fn app_has(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let name = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::Bool(crate::container::Container::has(&name)))
}

/// App::instance 方法实现
pub fn app_instance(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let name = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    tracing::info!("App::instance 注册实例: {}", name);
    Ok(Value::Bool(true))
}

/// App::alias 方法实现
pub fn app_alias(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let name = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let alias = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    crate::container::Container::alias(&name, &alias);
    Ok(Value::Bool(true))
}

/// 获取所有 App 门面方法
pub fn get_app_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("make", app_make),
        ("bind", app_bind),
        ("singleton", app_singleton),
        ("has", app_has),
        ("instance", app_instance),
        ("alias", app_alias),
    ]
}
