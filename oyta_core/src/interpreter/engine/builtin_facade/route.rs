//! Route 门面类方法实现
//!
//! 提供路由操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Route::get 方法实现
pub fn route_get(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let path = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let handler = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    crate::router::RouteFacade::get(&path, &handler);
    Ok(Value::Bool(true))
}

/// Route::post 方法实现
pub fn route_post(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let path = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let handler = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    crate::router::RouteFacade::post(&path, &handler);
    Ok(Value::Bool(true))
}

/// Route::put 方法实现
pub fn route_put(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let path = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let handler = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    crate::router::RouteFacade::put(&path, &handler);
    Ok(Value::Bool(true))
}

/// Route::delete 方法实现
pub fn route_delete(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let path = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let handler = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    crate::router::RouteFacade::delete(&path, &handler);
    Ok(Value::Bool(true))
}

/// Route::patch 方法实现
pub fn route_patch(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let path = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let handler = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    crate::router::RouteFacade::patch(&path, &handler);
    Ok(Value::Bool(true))
}

/// Route::any 方法实现
pub fn route_any(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let path = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let handler = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    crate::router::RouteFacade::any(&path, &handler);
    Ok(Value::Bool(true))
}

/// Route::group 方法实现
pub fn route_group(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(true))
}

/// Route::resource 方法实现
pub fn route_resource(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let name = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let controller = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    crate::router::RouteFacade::resource(&name, &controller);
    Ok(Value::Bool(true))
}

/// Route::miss 方法实现
pub fn route_miss(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let handler = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    crate::router::RouteFacade::miss(&handler);
    Ok(Value::Bool(true))
}

/// 获取所有 Route 门面方法
pub fn get_route_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("get", route_get),
        ("post", route_post),
        ("put", route_put),
        ("delete", route_delete),
        ("patch", route_patch),
        ("any", route_any),
        ("group", route_group),
        ("resource", route_resource),
        ("miss", route_miss),
    ]
}
