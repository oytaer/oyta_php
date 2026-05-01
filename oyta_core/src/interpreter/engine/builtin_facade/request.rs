//! Request 门面类方法实现
//!
//! 提供请求操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Request::param 方法实现
pub fn request_param(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    if args.is_empty() {
        let all = crate::http::RequestFacade::param_all(true);
        let result: Vec<Value> = all.into_iter().map(|(k, v)| {
            Value::AssociativeArray(vec![(k, Value::String(v))])
        }).collect();
        return Ok(Value::IndexedArray(result));
    }
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let default = args.get(1).cloned().unwrap_or(Value::Null);
    match crate::http::RequestFacade::param(&key) {
        Some(v) => Ok(Value::String(v)),
        None => Ok(default),
    }
}

/// Request::get 方法实现
pub fn request_get(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    if args.is_empty() {
        let all = crate::http::RequestFacade::get_all();
        let result: Vec<Value> = all.into_iter().map(|(k, v)| {
            Value::AssociativeArray(vec![(k, Value::String(v))])
        }).collect();
        return Ok(Value::IndexedArray(result));
    }
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let default = args.get(1).cloned().unwrap_or(Value::Null);
    match crate::http::RequestFacade::get(&key) {
        Some(v) => Ok(Value::String(v)),
        None => Ok(default),
    }
}

/// Request::post 方法实现
pub fn request_post(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    if args.is_empty() {
        let all = crate::http::RequestFacade::post_all();
        let result: Vec<Value> = all.into_iter().map(|(k, v)| {
            Value::AssociativeArray(vec![(k, Value::String(v))])
        }).collect();
        return Ok(Value::IndexedArray(result));
    }
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let default = args.get(1).cloned().unwrap_or(Value::Null);
    match crate::http::RequestFacade::post(&key) {
        Some(v) => Ok(Value::String(v)),
        None => Ok(default),
    }
}

/// Request::put 方法实现
pub fn request_put(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    if args.is_empty() {
        let all = crate::http::RequestFacade::put_all();
        let result: Vec<Value> = all.into_iter().map(|(k, v)| {
            Value::AssociativeArray(vec![(k, Value::String(v))])
        }).collect();
        return Ok(Value::IndexedArray(result));
    }
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let default = args.get(1).cloned().unwrap_or(Value::Null);
    match crate::http::RequestFacade::put(&key) {
        Some(v) => Ok(Value::String(v)),
        None => Ok(default),
    }
}

/// Request::delete 方法实现
pub fn request_delete(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    if args.is_empty() {
        let all = crate::http::RequestFacade::delete_all();
        let result: Vec<Value> = all.into_iter().map(|(k, v)| {
            Value::AssociativeArray(vec![(k, Value::String(v))])
        }).collect();
        return Ok(Value::IndexedArray(result));
    }
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let default = args.get(1).cloned().unwrap_or(Value::Null);
    match crate::http::RequestFacade::delete(&key) {
        Some(v) => Ok(Value::String(v)),
        None => Ok(default),
    }
}

/// Request::session 方法实现
pub fn request_session(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    if args.is_empty() {
        let all = crate::http::RequestFacade::session_all();
        let result: Vec<Value> = all.into_iter().map(|(k, v)| {
            match v {
                serde_json::Value::String(s) => Value::AssociativeArray(vec![(k, Value::String(s))]),
                serde_json::Value::Number(n) => Value::AssociativeArray(vec![(k, Value::Int(n.as_i64().unwrap_or(0)))]),
                serde_json::Value::Bool(b) => Value::AssociativeArray(vec![(k, Value::Bool(b))]),
                _ => Value::AssociativeArray(vec![(k, Value::String(v.to_string()))]),
            }
        }).collect();
        return Ok(Value::IndexedArray(result));
    }
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let default = args.get(1).cloned().unwrap_or(Value::Null);
    match crate::http::RequestFacade::session(&key) {
        Some(v) => match v {
            serde_json::Value::String(s) => Ok(Value::String(s)),
            serde_json::Value::Number(n) => Ok(Value::Int(n.as_i64().unwrap_or(0))),
            serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
            serde_json::Value::Null => Ok(default),
            _ => Ok(Value::String(v.to_string())),
        },
        None => Ok(default),
    }
}

/// Request::cookie 方法实现
pub fn request_cookie(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    if args.is_empty() {
        let all = crate::http::RequestFacade::cookie_all();
        let result: Vec<Value> = all.into_iter().map(|(k, v)| {
            Value::AssociativeArray(vec![(k, Value::String(v))])
        }).collect();
        return Ok(Value::IndexedArray(result));
    }
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let default = args.get(1).cloned().unwrap_or(Value::Null);
    match crate::http::RequestFacade::cookie(&key) {
        Some(v) => Ok(Value::String(v)),
        None => Ok(default),
    }
}

/// Request::server 方法实现
pub fn request_server(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    if args.is_empty() {
        let all = crate::http::RequestFacade::server_all();
        let result: Vec<Value> = all.into_iter().map(|(k, v)| {
            Value::AssociativeArray(vec![(k, Value::String(v))])
        }).collect();
        return Ok(Value::IndexedArray(result));
    }
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let default = args.get(1).cloned().unwrap_or(Value::Null);
    match crate::http::RequestFacade::server(&key) {
        Some(v) => Ok(Value::String(v)),
        None => Ok(default),
    }
}

/// Request::env 方法实现
pub fn request_env(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    if args.is_empty() {
        let all = crate::http::RequestFacade::env_all();
        let result: Vec<Value> = all.into_iter().map(|(k, v)| {
            Value::AssociativeArray(vec![(k, Value::String(v))])
        }).collect();
        return Ok(Value::IndexedArray(result));
    }
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let default = args.get(1).cloned().unwrap_or(Value::Null);
    match crate::http::RequestFacade::env(&key) {
        Some(v) => Ok(Value::String(v)),
        None => Ok(default),
    }
}

/// Request::route 方法实现
pub fn request_route(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    if args.is_empty() {
        let all = crate::http::RequestFacade::route_all();
        let result: Vec<Value> = all.into_iter().map(|(k, v)| {
            Value::AssociativeArray(vec![(k, Value::String(v))])
        }).collect();
        return Ok(Value::IndexedArray(result));
    }
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let default = args.get(1).cloned().unwrap_or(Value::Null);
    match crate::http::RequestFacade::route(&key) {
        Some(v) => Ok(Value::String(v)),
        None => Ok(default),
    }
}

/// Request::file 方法实现
pub fn request_file(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    if args.is_empty() {
        let all = crate::http::RequestFacade::file_all();
        let result: Vec<Value> = all.into_iter().flat_map(|(k, files)| {
            let k = k.clone();
            files.into_iter().map(move |f| {
                Value::AssociativeArray(vec![
                    ("key".to_string(), Value::String(k.clone())),
                    ("name".to_string(), Value::String(f.name)),
                    ("tmp_name".to_string(), Value::String(f.tmp_name)),
                    ("type".to_string(), Value::String(f.mime_type)),
                    ("size".to_string(), Value::Int(f.size as i64)),
                    ("error".to_string(), Value::Int(f.error as i64)),
                ])
            })
        }).collect();
        return Ok(Value::IndexedArray(result));
    }
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    
    match crate::http::RequestFacade::file(&key) {
        Some(files) => {
            let file_values: Vec<Value> = files.iter().map(|f| {
                Value::AssociativeArray(vec![
                    ("name".to_string(), Value::String(f.name.clone())),
                    ("tmp_name".to_string(), Value::String(f.tmp_name.clone())),
                    ("type".to_string(), Value::String(f.mime_type.clone())),
                    ("size".to_string(), Value::Int(f.size as i64)),
                    ("error".to_string(), Value::Int(f.error as i64)),
                ])
            }).collect();
            Ok(Value::IndexedArray(file_values))
        }
        None => Ok(Value::Null),
    }
}

/// Request::has 方法实现
pub fn request_has(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let method = args.get(1).map(|v| v.to_string_value()).unwrap_or_else(|| "param".to_string());
    
    Ok(Value::Bool(crate::http::RequestFacade::has(&key, &method)))
}

/// Request::only 方法实现
pub fn request_only(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let result = crate::http::RequestFacade::param_all(true);
    let values: Vec<Value> = result.into_iter().map(|(k, v)| {
        Value::AssociativeArray(vec![(k, Value::String(v))])
    }).collect();
    Ok(Value::IndexedArray(values))
}

/// Request::except 方法实现
pub fn request_except(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let result = crate::http::RequestFacade::param_all(true);
    let values: Vec<Value> = result.into_iter().map(|(k, v)| {
        Value::AssociativeArray(vec![(k, Value::String(v))])
    }).collect();
    Ok(Value::IndexedArray(values))
}

/// Request::method 方法实现
pub fn request_method(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let original = args.first().map(|v| v.to_bool()).unwrap_or(false);
    Ok(Value::String(crate::http::RequestFacade::method(original)))
}

/// Request::ip 方法实现
pub fn request_ip(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::String(crate::http::RequestFacade::ip()))
}

/// Request::host 方法实现
pub fn request_host(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::String(crate::http::RequestFacade::host()))
}

/// Request::domain 方法实现
pub fn request_domain(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::String(crate::http::RequestFacade::domain()))
}

/// Request::scheme 方法实现
pub fn request_scheme(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::String(crate::http::RequestFacade::scheme()))
}

/// Request::port 方法实现
pub fn request_port(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Int(crate::http::RequestFacade::port() as i64))
}

/// Request::url 方法实现
pub fn request_url(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let complete = args.first().map(|v| v.to_bool()).unwrap_or(false);
    Ok(Value::String(crate::http::RequestFacade::url(complete)))
}

/// Request::baseUrl 方法实现
pub fn request_base_url(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let complete = args.first().map(|v| v.to_bool()).unwrap_or(false);
    Ok(Value::String(crate::http::RequestFacade::base_url(complete)))
}

/// Request::path 方法实现
pub fn request_path(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::String(crate::http::RequestFacade::path()))
}

/// Request::pathinfo 方法实现
pub fn request_pathinfo(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::String(crate::http::RequestFacade::path_info()))
}

/// Request::ext 方法实现
pub fn request_ext(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::String(crate::http::RequestFacade::ext()))
}

/// Request::time 方法实现
pub fn request_time(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Int(crate::http::RequestFacade::time()))
}

/// Request::controller 方法实现
pub fn request_controller(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let lowercase = args.first().map(|v| v.to_bool()).unwrap_or(false);
    let base = args.get(1).map(|v| v.to_bool()).unwrap_or(false);
    Ok(Value::String(crate::http::RequestFacade::controller(lowercase, base)))
}

/// Request::action 方法实现
pub fn request_action(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let lowercase = args.first().map(|v| v.to_bool()).unwrap_or(false);
    Ok(Value::String(crate::http::RequestFacade::action(lowercase)))
}

/// Request::layer 方法实现
pub fn request_layer(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::String(crate::http::RequestFacade::layer()))
}

/// Request::isGet 方法实现
pub fn request_is_get(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(crate::http::RequestFacade::is_get()))
}

/// Request::isPost 方法实现
pub fn request_is_post(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(crate::http::RequestFacade::is_post()))
}

/// Request::isPut 方法实现
pub fn request_is_put(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(crate::http::RequestFacade::is_put()))
}

/// Request::isDelete 方法实现
pub fn request_is_delete(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(crate::http::RequestFacade::is_delete()))
}

/// Request::isHead 方法实现
pub fn request_is_head(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(crate::http::RequestFacade::is_head()))
}

/// Request::isPatch 方法实现
pub fn request_is_patch(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(crate::http::RequestFacade::is_patch()))
}

/// Request::isOptions 方法实现
pub fn request_is_options(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(crate::http::RequestFacade::is_options()))
}

/// Request::isAjax 方法实现
pub fn request_is_ajax(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(crate::http::RequestFacade::is_ajax()))
}

/// Request::isPjax 方法实现
pub fn request_is_pjax(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(crate::http::RequestFacade::is_pjax()))
}

/// Request::isJson 方法实现
pub fn request_is_json(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(crate::http::RequestFacade::is_json()))
}

/// Request::isMobile 方法实现
pub fn request_is_mobile(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(crate::http::RequestFacade::is_mobile()))
}

/// Request::isCli 方法实现
pub fn request_is_cli(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(crate::http::RequestFacade::is_cli()))
}

/// Request::isCgi 方法实现
pub fn request_is_cgi(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(crate::http::RequestFacade::is_cgi()))
}

/// Request::header 方法实现
pub fn request_header(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let default = args.get(1).cloned().unwrap_or(Value::Null);
    
    match crate::http::RequestFacade::header(&key) {
        Some(v) => Ok(Value::String(v)),
        None => Ok(default),
    }
}

/// Request::all 方法实现
pub fn request_all(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let all = crate::http::RequestFacade::all();
    let result: Vec<Value> = all.into_iter().map(|(k, v)| {
        match v {
            serde_json::Value::String(s) => Value::AssociativeArray(vec![(k, Value::String(s))]),
            serde_json::Value::Number(n) => Value::AssociativeArray(vec![(k, Value::Int(n.as_i64().unwrap_or(0)))]),
            serde_json::Value::Bool(b) => Value::AssociativeArray(vec![(k, Value::Bool(b))]),
            _ => Value::AssociativeArray(vec![(k, Value::String(v.to_string()))]),
        }
    }).collect();
    Ok(Value::IndexedArray(result))
}

/// 获取所有 Request 门面方法
pub fn get_request_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("param", request_param),
        ("get", request_get),
        ("post", request_post),
        ("put", request_put),
        ("delete", request_delete),
        ("session", request_session),
        ("cookie", request_cookie),
        ("server", request_server),
        ("env", request_env),
        ("route", request_route),
        ("file", request_file),
        ("has", request_has),
        ("only", request_only),
        ("except", request_except),
        ("method", request_method),
        ("ip", request_ip),
        ("host", request_host),
        ("domain", request_domain),
        ("scheme", request_scheme),
        ("port", request_port),
        ("url", request_url),
        ("baseUrl", request_base_url),
        ("path", request_path),
        ("pathinfo", request_pathinfo),
        ("ext", request_ext),
        ("time", request_time),
        ("controller", request_controller),
        ("action", request_action),
        ("layer", request_layer),
        ("isGet", request_is_get),
        ("isPost", request_is_post),
        ("isPut", request_is_put),
        ("isDelete", request_is_delete),
        ("isHead", request_is_head),
        ("isPatch", request_is_patch),
        ("isOptions", request_is_options),
        ("isAjax", request_is_ajax),
        ("isPjax", request_is_pjax),
        ("isJson", request_is_json),
        ("isMobile", request_is_mobile),
        ("isCli", request_is_cli),
        ("isCgi", request_is_cgi),
        ("header", request_header),
        ("all", request_all),
    ]
}
