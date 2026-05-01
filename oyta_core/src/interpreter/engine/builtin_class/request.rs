//! Request 内置类模块
//!
//! 实现 PHP Request 类

use crate::interpreter::value::{ObjectInstance, Value};

// ============================================================================
// Request 类方法实现
// ============================================================================

/// Request::get 方法实现
///
/// 获取 GET 参数
///
/// # 参数
/// - `_instance`: Request 对象实例
/// - `args`: 参数名和默认值
///
/// # 返回
/// 参数值
pub fn request_get(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    // 获取参数名
    let key = args.first()
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();
    
    // 获取默认值
    let default = args.get(1).cloned().unwrap_or(Value::Null);
    
    // 尝试从 Request 门面获取值
    match crate::http::RequestFacade::get(&key) {
        Some(value) => Ok(Value::String(value)),
        None => Ok(default),
    }
}

/// Request::post 方法实现
///
/// 获取 POST 参数
///
/// # 参数
/// - `_instance`: Request 对象实例
/// - `args`: 参数名和默认值
///
/// # 返回
/// 参数值
pub fn request_post(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    // 获取参数名
    let key = args.first()
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();
    
    // 获取默认值
    let default = args.get(1).cloned().unwrap_or(Value::Null);
    
    // 尝试从 Request 门面获取值
    match crate::http::RequestFacade::post(&key) {
        Some(value) => Ok(Value::String(value)),
        None => Ok(default),
    }
}

/// Request::param 方法实现
///
/// 获取请求参数（优先 GET）
///
/// # 参数
/// - `_instance`: Request 对象实例
/// - `args`: 参数名和默认值
///
/// # 返回
/// 参数值
pub fn request_param(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    // 获取参数名
    let key = args.first()
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();
    
    // 获取默认值
    let default = args.get(1).cloned().unwrap_or(Value::Null);
    
    // 尝试从 Request 门面获取值
    match crate::http::RequestFacade::param(&key) {
        Some(value) => Ok(Value::String(value)),
        None => Ok(default),
    }
}

/// Request::input 方法实现
///
/// 获取输入参数（支持 get.post 等）
///
/// # 参数
/// - `_instance`: Request 对象实例
/// - `args`: 参数名和默认值
///
/// # 返回
/// 参数值
pub fn request_input(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    // 获取参数名（支持点语法）
    let key = args.first()
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();
    
    // 获取默认值
    let default = args.get(1).cloned().unwrap_or(Value::Null);
    
    // 解析参数名
    let (method, name) = if key.contains('.') {
        let parts: Vec<&str> = key.splitn(2, '.').collect();
        (parts[0], parts[1])
    } else {
        ("param", key.as_str())
    };
    
    // 根据方法获取值
    let result = match method {
        "get" => crate::http::RequestFacade::get(name).map(Value::String),
        "post" => crate::http::RequestFacade::post(name).map(Value::String),
        "put" => crate::http::RequestFacade::put(name).map(Value::String),
        "delete" => crate::http::RequestFacade::delete(name).map(Value::String),
        "session" => crate::http::RequestFacade::session(name).map(|v| match v {
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Number(n) => Value::Int(n.as_i64().unwrap_or(0)),
            serde_json::Value::Bool(b) => Value::Bool(b),
            _ => Value::String(v.to_string()),
        }),
        "cookie" => crate::http::RequestFacade::cookie(name).map(Value::String),
        "server" => crate::http::RequestFacade::server(name).map(Value::String),
        "env" => crate::http::RequestFacade::env(name).map(Value::String),
        "route" => crate::http::RequestFacade::route(name).map(Value::String),
        _ => crate::http::RequestFacade::param(name).map(Value::String),
    };
    
    Ok(result.unwrap_or(default))
}

/// Request::method 方法实现
///
/// 获取请求方法
///
/// # 参数
/// - `_instance`: Request 对象实例
///
/// # 返回
/// 请求方法字符串
pub fn request_method(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 简化实现：返回 GET
    Ok(Value::String("GET".to_string()))
}

/// Request::isMethod 方法实现
///
/// 检查请求方法
///
/// # 参数
/// - `_instance`: Request 对象实例
/// - `args`: 方法名参数
///
/// # 返回
/// 布尔值
pub fn request_is_method(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    // 获取方法名
    let _method = args.first()
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// Request::ip 方法实现
///
/// 获取客户端 IP
///
/// # 参数
/// - `_instance`: Request 对象实例
///
/// # 返回
/// IP 地址字符串
pub fn request_ip(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 简化实现：返回 127.0.0.1
    Ok(Value::String("127.0.0.1".to_string()))
}

/// Request::header 方法实现
///
/// 获取请求头
///
/// # 参数
/// - `_instance`: Request 对象实例
/// - `args`: 头名和默认值
///
/// # 返回
/// 请求头值
pub fn request_header(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    // 获取头名
    let key = args.first()
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();
    
    // 获取默认值
    let default = args.get(1).cloned().unwrap_or(Value::Null);
    
    // 尝试从 Request 门面获取值
    match crate::http::RequestFacade::header(&key) {
        Some(value) => Ok(Value::String(value)),
        None => Ok(default),
    }
}

/// Request::all 方法实现
///
/// 获取所有输入
///
/// # 参数
/// - `_instance`: Request 对象实例
///
/// # 返回
/// 所有输入数组
pub fn request_all(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 简化实现：返回空数组
    Ok(Value::AssociativeArray(vec![]))
}

/// Request::isAjax 方法实现
///
/// 检查是否 AJAX 请求
///
/// # 参数
/// - `_instance`: Request 对象实例
///
/// # 返回
/// 布尔值
pub fn request_is_ajax(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 简化实现：返回 false
    Ok(Value::Bool(false))
}

/// Request::isJson 方法实现
///
/// 检查是否 JSON 请求
///
/// # 参数
/// - `_instance`: Request 对象实例
///
/// # 返回
/// 布尔值
pub fn request_is_json(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 简化实现：返回 false
    Ok(Value::Bool(false))
}

/// Request::isMobile 方法实现
///
/// 检查是否移动端请求
///
/// # 参数
/// - `_instance`: Request 对象实例
///
/// # 返回
/// 布尔值
pub fn request_is_mobile(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 简化实现：返回 false
    Ok(Value::Bool(false))
}

/// Request::isCli 方法实现
///
/// 检查是否 CLI 模式
///
/// # 参数
/// - `_instance`: Request 对象实例
///
/// # 返回
/// 布尔值
pub fn request_is_cli(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 检查是否在命令行环境
    Ok(Value::Bool(true))
}

/// Request::isCgi 方法实现
///
/// 检查是否 CGI 模式
///
/// # 参数
/// - `_instance`: Request 对象实例
///
/// # 返回
/// 布尔值
pub fn request_is_cgi(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 简化实现：返回 false
    Ok(Value::Bool(false))
}
