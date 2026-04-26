//! HTTP 请求处理器模块
//!
//! 处理所有进入的 HTTP 请求
//! 根据请求路径分发到对应的控制器方法
//! 支持路由匹配、自动路由、解释器执行等

use axum::body::Body;
use axum::extract::State;
use axum::http::Request;
use axum::response::IntoResponse;
use std::collections::HashMap;

use crate::http::request::Request as OytaRequest;
use crate::http::response::OytaResponse;
use crate::http::state::AppState;
use crate::interpreter::engine::Interpreter;
use crate::interpreter::value::{ObjectInstance, Value};
use crate::router::types::{HttpMethod, RouteHandler};

/// PHP 请求处理器
/// 所有非静态文件的请求都经过此处理器
/// 解析请求路径，分发到对应的控制器方法
pub async fn php_handler(State(state): State<AppState>, req: Request<Body>) -> impl IntoResponse {
    let (parts, _body) = req.into_parts();

    let oyta_req = OytaRequest::from_axum_parts(parts).await;

    let path = oyta_req.path.clone();
    let method = HttpMethod::from_str(&oyta_req.method);

    tracing::debug!("处理请求: {} {}", oyta_req.method, path);

    match dispatch_request(&state, &oyta_req, &method) {
        Ok(response) => response.into_response(),
        Err(e) => {
            tracing::error!("请求处理错误: {} - {}", path, e);
            if state.debug {
                let request_info = format!(
                    "方法: {}\n路径: {}\n查询: {}\nIP: {}",
                    oyta_req.method,
                    oyta_req.path,
                    oyta_req.query_string.as_deref().unwrap_or(""),
                    oyta_req.client_ip.as_deref().unwrap_or("unknown")
                );
                OytaResponse::debug_error(
                    500,
                    "服务器内部错误",
                    &e.to_string(),
                    "",
                    0,
                    &format!("{:?}", e),
                    &request_info,
                ).into_response()
            } else {
                OytaResponse::internal_error("服务器内部错误").into_response()
            }
        }
    }
}

/// 分发请求到对应的控制器方法
/// 1. 尝试从路由注册表中匹配定义路由
/// 2. 如果没有匹配，尝试自动路由（控制器/方法）
/// 3. 如果都没有匹配，返回 404
pub fn dispatch_request(
    state: &AppState,
    req: &OytaRequest,
    method: &HttpMethod,
) -> anyhow::Result<OytaResponse> {
    if req.path == "/" || req.path.is_empty() {
        return Ok(welcome_page());
    }

    if req.path == "/favicon.ico" {
        return Ok(OytaResponse::new().status(204));
    }

    // 第一步：尝试从路由注册表匹配定义路由
    if let Some(response) = try_route_match(state, req, method) {
        return Ok(response);
    }

    // 第二步：尝试自动路由匹配
    if let Some(response) = try_auto_route(state, req, method) {
        return Ok(response);
    }

    // 第三步：尝试 MISS 路由（404 兜底处理器）
    if let Some(response) = try_miss_route(state, req) {
        return Ok(response);
    }

    // 没有匹配的路由，返回 404
    Ok(OytaResponse::not_found(&format!(
        "页面不存在: {}", req.path
    )))
}

/// 尝试从路由注册表匹配定义路由
fn try_route_match(
    state: &AppState,
    req: &OytaRequest,
    method: &HttpMethod,
) -> Option<OytaResponse> {
    let route_match = state.route_registry.match_route(method, &req.path)?;

    let mut oyta_req = req.clone();
    for (key, value) in &route_match.params {
        oyta_req.route_params.insert(key.clone(), value.clone());
    }

    match &route_match.route.handler {
        RouteHandler::ControllerMethod { controller, method: action } => {
            execute_controller(state, controller, action, &oyta_req)
        }
        RouteHandler::Static { content, content_type, status } => {
            Some(OytaResponse::new()
                .status(*status)
                .content_type(content_type)
                .with_body(content.clone()))
        }
        RouteHandler::Redirect { url, status } => {
            let mut resp = OytaResponse::new().status(*status);
            resp.headers.insert("Location".to_string(), url.clone());
            Some(resp)
        }
        RouteHandler::Closure { .. } => {
            Some(OytaResponse::error(500, "闭包路由暂不支持"))
        }
    }
}

/// 尝试自动路由匹配
/// 根据路径自动推断控制器和方法
/// 单应用: /controller/action
/// 多应用: /app/controller/action
/// 只有在符号表中找到对应控制器时才返回匹配
fn try_auto_route(
    state: &AppState,
    req: &OytaRequest,
    _method: &HttpMethod,
) -> Option<OytaResponse> {
    let path = req.path.trim_start_matches('/');
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if parts.is_empty() {
        return None;
    }

    // 单应用模式下的自动路由
    // /index → Index 控制器的 index 方法
    // /index/hello → Index 控制器的 hello 方法
    // /user/list → User 控制器的 list 方法
    if parts.len() <= 2 {
        let controller_name = kebab_to_pascal(parts[0]);
        let method_name = if parts.len() > 1 {
            kebab_to_camel(parts[1])
        } else {
            "index".to_string()
        };

        let full_class = format!("app\\controller\\{}", controller_name);

        // 在符号表中查找控制器，只有存在才匹配
        if state.registry.find_class(&full_class).is_none() {
            // 尝试模糊匹配
            let fuzzy_results = state.registry.find_class_fuzzy(&controller_name);
            let controller_def = fuzzy_results.into_iter().find(|c| c.is_controller());
            if controller_def.is_none() {
                return None;
            }
        }

        return execute_controller(state, &full_class, &method_name, req);
    }

    None
}

/// 尝试 MISS 路由
/// 当定义路由和自动路由都不匹配时
/// 使用 MISS 路由作为兜底处理器
/// 对应 ThinkPHP 8.0 中的 Route::miss() 功能
fn try_miss_route(
    state: &AppState,
    req: &OytaRequest,
) -> Option<OytaResponse> {
    let miss_route = state.route_registry.get_miss_route()?;

    match &miss_route.handler {
        RouteHandler::ControllerMethod { controller, method: action } => {
            execute_controller(state, controller, action, req)
        }
        RouteHandler::Static { content, content_type, status } => {
            Some(OytaResponse::new()
                .status(*status)
                .content_type(content_type)
                .with_body(content.clone()))
        }
        RouteHandler::Redirect { url, status } => {
            let mut resp = OytaResponse::new().status(*status);
            resp.headers.insert("Location".to_string(), url.clone());
            Some(resp)
        }
        RouteHandler::Closure { .. } => {
            Some(OytaResponse::error(404, "页面不存在"))
        }
    }
}

/// 执行控制器方法
/// 先执行中间件链，再通过解释器执行 PHP 控制器方法
fn execute_controller(
    state: &AppState,
    class_name: &str,
    method_name: &str,
    req: &OytaRequest,
) -> Option<OytaResponse> {
    let class_def = match state.registry.find_class(class_name) {
        Some(c) => c,
        None => {
            let fuzzy = state.registry.find_class_fuzzy(class_name);
            match fuzzy.into_iter().find(|c| c.is_controller()) {
                Some(c) => c,
                None => {
                    tracing::debug!("控制器不存在: {}", class_name);
                    return None;
                }
            }
        }
    };

    let method_def = match class_def.find_method(method_name) {
        Some(m) => m,
        None => {
            tracing::debug!("方法不存在: {}::{}", class_name, method_name);
            return None;
        }
    };

    if method_def.visibility != crate::symbol_table::types::Visibility::Public {
        tracing::debug!("方法不可访问: {}::{} (非 public)", class_name, method_name);
        return None;
    }

    let mut interpreter = Interpreter::new(state.registry.clone());
    let args = build_method_args(req, &method_def.params);
    let request_value = build_request_value(req);

    let middleware = state.middleware.read().unwrap();
    if middleware.count() > 0 {
        let registry = state.registry.clone();
        let full_name = class_def.full_name.clone();
        let result = middleware.dispatch(req, &mut interpreter, move |_req| {
            let mut interp = Interpreter::new(registry);
            match interp.execute_controller_method(&full_name, method_name, args, Some(request_value)) {
                Ok(value) => value_to_response(&value),
                Err(e) => {
                    tracing::error!("执行控制器方法失败: {}::{} - {}", full_name, method_name, e);
                    OytaResponse::internal_error("服务器内部错误")
                }
            }
        });
        Some(result)
    } else {
        match interpreter.execute_controller_method(&class_def.full_name, method_name, args, Some(request_value)) {
            Ok(value) => Some(value_to_response(&value)),
            Err(e) => {
                tracing::error!("执行控制器方法失败: {}::{} - {}", class_name, method_name, e);
                if state.debug {
                    let request_info = format!(
                        "方法: {}\n路径: {}\n控制器: {}::{}()",
                        req.method, req.path, class_name, method_name
                    );
                    Some(OytaResponse::debug_error(
                        500,
                        &format!("控制器执行错误: {}::{}", class_name, method_name),
                        &e.to_string(),
                        &class_def.file_path,
                        0,
                        &format!("{:?}", e),
                        &request_info,
                    ))
                } else {
                    Some(OytaResponse::internal_error("服务器内部错误"))
                }
            }
        }
    }
}

/// 根据请求构建方法参数
fn build_method_args(
    req: &OytaRequest,
    params: &[crate::symbol_table::types::ParamDef],
) -> Vec<Value> {
    let mut args = Vec::new();

    for param in params {
        let param_name = param.name.trim_start_matches('$');
        let value = if let Some(v) = req.route_params.get(param_name) {
            Value::String(v.clone())
        } else if let Some(v) = req.query_params.get(param_name) {
            Value::String(v.clone())
        } else if let Some(v) = req.form_data.get(param_name) {
            Value::String(v.clone())
        } else if let Some(default) = &param.default_value {
            parse_default_value(default)
        } else {
            Value::Null
        };
        args.push(value);
    }

    args
}

/// 将 HTTP 请求构建为 Value 对象
/// 控制器方法中可通过 $request 访问
fn build_request_value(req: &OytaRequest) -> Value {
    let mut props = HashMap::new();

    props.insert("method".to_string(), Value::String(req.method.clone()));
    props.insert("uri".to_string(), Value::String(req.uri.clone()));
    props.insert("path".to_string(), Value::String(req.path.clone()));
    props.insert("query_string".to_string(), Value::String(req.query_string.clone().unwrap_or_default()));
    props.insert("client_ip".to_string(), Value::String(req.client_ip.clone().unwrap_or_default()));

    let mut query_params = Vec::new();
    for (k, v) in &req.query_params {
        query_params.push((k.clone(), Value::String(v.clone())));
    }
    props.insert("query".to_string(), Value::AssociativeArray(query_params));

    let mut form_data = Vec::new();
    for (k, v) in &req.form_data {
        form_data.push((k.clone(), Value::String(v.clone())));
    }
    props.insert("post".to_string(), Value::AssociativeArray(form_data));

    let mut route_params = Vec::new();
    for (k, v) in &req.route_params {
        route_params.push((k.clone(), Value::String(v.clone())));
    }
    props.insert("route".to_string(), Value::AssociativeArray(route_params));

    let mut headers = Vec::new();
    for (k, v) in &req.headers {
        headers.push((k.clone(), Value::String(v.clone())));
    }
    props.insert("header".to_string(), Value::AssociativeArray(headers));

    props.insert("is_get".to_string(), Value::Bool(req.is_get()));
    props.insert("is_post".to_string(), Value::Bool(req.is_post()));
    props.insert("is_ajax".to_string(), Value::Bool(req.is_ajax()));

    Value::Object(ObjectInstance {
        class_name: "oyta\\Request".to_string(),
        properties: props,
    })
}

/// 将解释器返回值转换为 HTTP 响应
fn value_to_response(value: &Value) -> OytaResponse {
    match value {
        Value::String(s) => {
            if s.starts_with('{') || s.starts_with('[') {
                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(s) {
                    let mut resp = OytaResponse::new().status(200);
                    resp.headers.insert("Content-Type".to_string(), "application/json; charset=utf-8".to_string());
                    resp.body = serde_json::to_string_pretty(&json_val).unwrap_or_else(|_| s.clone());
                    return resp;
                }
            }
            OytaResponse::html(s.clone())
        }
        Value::AssociativeArray(_) | Value::IndexedArray(_) => {
            let json_val = value.to_json_value();
            let mut resp = OytaResponse::new().status(200);
            resp.headers.insert("Content-Type".to_string(), "application/json; charset=utf-8".to_string());
            resp.body = serde_json::to_string_pretty(&json_val).unwrap_or_else(|_| "{}".to_string());
            resp
        }
        Value::Int(i) => OytaResponse::text(i.to_string()),
        Value::Float(f) => OytaResponse::text(f.to_string()),
        Value::Bool(b) => OytaResponse::text(if *b { "true" } else { "false" }.to_string()),
        Value::Null => OytaResponse::new().status(204),
        _ => OytaResponse::text(value.to_string_value()),
    }
}

/// 解析默认值字符串为 Value
fn parse_default_value(s: &str) -> Value {
    match s {
        "null" => Value::Null,
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => {
            if let Ok(i) = s.parse::<i64>() { return Value::Int(i); }
            if let Ok(f) = s.parse::<f64>() { return Value::Float(f); }
            if s.starts_with('\'') && s.ends_with('\'') {
                return Value::String(s[1..s.len()-1].to_string());
            }
            if s.starts_with('"') && s.ends_with('"') {
                return Value::String(s[1..s.len()-1].to_string());
            }
            Value::String(s.to_string())
        }
    }
}

/// 将 kebab-case 转为 PascalCase
fn kebab_to_pascal(s: &str) -> String {
    s.split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// 将 kebab-case 转为 camelCase
fn kebab_to_camel(s: &str) -> String {
    let pascal = kebab_to_pascal(s);
    let mut chars = pascal.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_lowercase().collect::<String>() + chars.as_str(),
    }
}

/// 生成欢迎页面
fn welcome_page() -> OytaResponse {
    let html = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OYTAPHP</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
               background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
               min-height: 100vh; display: flex; align-items: center; justify-content: center; }
        .container { text-align: center; color: white; }
        h1 { font-size: 4rem; margin-bottom: 1rem; text-shadow: 2px 2px 4px rgba(0,0,0,0.3); }
        p { font-size: 1.2rem; opacity: 0.9; margin-bottom: 0.5rem; }
        .version { font-size: 1rem; opacity: 0.7; margin-top: 2rem; }
        .info { margin-top: 2rem; padding: 1rem 2rem; background: rgba(255,255,255,0.1); border-radius: 8px; display: inline-block; }
        .info p { font-size: 0.9rem; opacity: 0.8; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 OYTAPHP</h1>
        <p>高性能 PHP 运行时框架</p>
        <p>兼容 ThinkPHP 8.0 · 基于 Rust 构建</p>
        <div class="version">OYTAPHP v0.1.0</div>
        <div class="info">
            <p>路由: /控制器/方法</p>
            <p>示例: /index/hello</p>
        </div>
    </div>
</body>
</html>"#;

    OytaResponse::html(html)
}
