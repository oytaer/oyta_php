//! 内置中间件模块
//!
//! 提供 ThinkPHP 8.0 风格的内置中间件
//! 包括：SessionInit、CORS、RequestCache、LoadLangPack、Trace 等
//! 这些中间件在框架启动时自动注册

use crate::http::request::Request as OytaRequest;
use crate::http::response::OytaResponse;
use crate::middleware::types::{MiddlewareDef, MiddlewareKind};

/// 获取所有内置中间件定义
/// 按执行顺序排列
pub fn builtin_middleware_defs() -> Vec<MiddlewareDef> {
    vec![
        MiddlewareDef {
            name: "session_init".to_string(),
            class: "oyta\\middleware\\SessionInit".to_string(),
            params: Vec::new(),
            kind: MiddlewareKind::Global,
            disabled: false,
        },
        MiddlewareDef {
            name: "cors".to_string(),
            class: "oyta\\middleware\\CORS".to_string(),
            params: Vec::new(),
            kind: MiddlewareKind::Global,
            disabled: false,
        },
        MiddlewareDef {
            name: "load_lang_pack".to_string(),
            class: "oyta\\middleware\\LoadLangPack".to_string(),
            params: Vec::new(),
            kind: MiddlewareKind::Global,
            disabled: false,
        },
        MiddlewareDef {
            name: "request_cache".to_string(),
            class: "oyta\\middleware\\RequestCache".to_string(),
            params: Vec::new(),
            kind: MiddlewareKind::Route,
            disabled: false,
        },
    ]
}

/// CORS 中间件处理器
/// 处理跨域请求
/// 在 Rust 层直接处理，无需通过 PHP 解释器
pub fn handle_cors(request: &OytaRequest) -> Option<OytaResponse> {
    // 如果是 OPTIONS 预检请求，直接返回 CORS 头
    if request.method == "OPTIONS" {
        let mut response = OytaResponse::new().status(204);
        response.headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
        response.headers.insert("Access-Control-Allow-Methods".to_string(), "GET, POST, PUT, DELETE, PATCH, OPTIONS".to_string());
        response.headers.insert("Access-Control-Allow-Headers".to_string(), "Content-Type, Authorization, X-Requested-With".to_string());
        response.headers.insert("Access-Control-Max-Age".to_string(), "86400".to_string());
        return Some(response);
    }
    None
}

/// Session 初始化中间件处理器
/// 在 Rust 层处理 Session 的启动
pub fn handle_session_init(request: &OytaRequest) -> Option<OytaResponse> {
    // 检查请求中是否携带 Session Cookie
    let _session_id = request.header("cookie")
        .and_then(|cookie| {
            cookie.split(';')
                .find_map(|part| {
                    let part = part.trim();
                    if part.starts_with("OYTA_SESSION=") {
                        Some(part[13..].to_string())
                    } else {
                        None
                    }
                })
        });
    // Session 初始化在 dispatch 阶段处理
    None
}

/// 为响应添加 CORS 头
pub fn add_cors_headers(response: &mut OytaResponse) {
    response.headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
    response.headers.insert("Access-Control-Allow-Methods".to_string(), "GET, POST, PUT, DELETE, PATCH, OPTIONS".to_string());
    response.headers.insert("Access-Control-Allow-Headers".to_string(), "Content-Type, Authorization, X-Requested-With".to_string());
}
