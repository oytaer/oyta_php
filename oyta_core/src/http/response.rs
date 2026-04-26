//! HTTP 响应对象模块
//!
//! 封装 Axum 的响应对象，提供 ThinkPHP 风格的响应构建接口
//! 支持 JSON/HTML/重定向/下载等响应类型

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::body::Body;
use serde_json::{json, Value};
use std::collections::HashMap;

/// OYTAPHP 响应对象
/// 封装 HTTP 响应信息，提供 ThinkPHP 风格的响应构建接口
#[derive(Debug, Clone)]
pub struct OytaResponse {
    /// 响应状态码
    pub status_code: u16,
    /// 响应头
    pub headers: HashMap<String, String>,
    /// 响应体内容
    pub body: String,
    /// 响应内容类型
    pub content_type: String,
}

impl OytaResponse {
    /// 创建新的响应对象
    pub fn new() -> Self {
        Self {
            status_code: 200,
            headers: HashMap::new(),
            body: String::new(),
            content_type: "text/html; charset=utf-8".to_string(),
        }
    }

    /// 创建文本响应
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            status_code: 200,
            headers: HashMap::new(),
            body: content.into(),
            content_type: "text/plain; charset=utf-8".to_string(),
        }
    }

    /// 创建 HTML 响应
    pub fn html(content: impl Into<String>) -> Self {
        Self {
            status_code: 200,
            headers: HashMap::new(),
            body: content.into(),
            content_type: "text/html; charset=utf-8".to_string(),
        }
    }

    /// 创建 JSON 响应
    pub fn json(data: &Value) -> Self {
        Self {
            status_code: 200,
            headers: HashMap::new(),
            body: data.to_string(),
            content_type: "application/json; charset=utf-8".to_string(),
        }
    }

    /// 创建成功 JSON 响应
    /// 格式: {"code": 0, "msg": "success", "data": ...}
    pub fn success(data: &Value) -> Self {
        Self::json(&json!({
            "code": 0,
            "msg": "success",
            "data": data,
        }))
    }

    /// 创建错误 JSON 响应
    /// 格式: {"code": code, "msg": msg, "data": null}
    pub fn error(code: i32, msg: &str) -> Self {
        let mut resp = Self::json(&json!({
            "code": code,
            "msg": msg,
            "data": Value::Null,
        }));
        resp.status_code = 200; // 业务错误仍返回 200 HTTP 状态码
        resp
    }

    /// 创建重定向响应
    pub fn redirect(url: &str) -> Self {
        let mut resp = Self::new();
        resp.status_code = 302;
        resp.headers.insert("Location".to_string(), url.to_string());
        resp.body = format!("Redirecting to {}", url);
        resp
    }

    /// 创建永久重定向响应
    pub fn redirect_permanent(url: &str) -> Self {
        let mut resp = Self::new();
        resp.status_code = 301;
        resp.headers.insert("Location".to_string(), url.to_string());
        resp.body = format!("Redirecting to {}", url);
        resp
    }

    /// 创建 404 响应
    pub fn not_found(msg: &str) -> Self {
        let mut resp = Self::html(format!("<h1>404 Not Found</h1><p>{}</p>", msg));
        resp.status_code = 404;
        resp
    }

    /// 创建 500 响应
    pub fn internal_error(msg: &str) -> Self {
        let mut resp = Self::html(format!("<h1>500 Internal Server Error</h1><p>{}</p>", msg));
        resp.status_code = 500;
        resp
    }

    /// 创建调试模式下的详细错误页面
    /// 显示错误信息、堆栈跟踪、请求信息等
    pub fn debug_error(
        status: u16,
        title: &str,
        message: &str,
        file: &str,
        line: usize,
        trace: &str,
        request_info: &str,
    ) -> Self {
        let html = format!(r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <title>{} - OYTAPHP</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: 'SF Mono', 'Fira Code', monospace; background: #1e1e2e; color: #cdd6f4; padding: 20px; }}
        .header {{ background: #f38ba8; color: #1e1e2e; padding: 15px 20px; border-radius: 8px 8px 0 0; }}
        .header h1 {{ font-size: 1.2rem; font-weight: bold; }}
        .header .status {{ font-size: 0.9rem; opacity: 0.8; }}
        .content {{ background: #313244; padding: 20px; border-radius: 0 0 8px 8px; }}
        .message {{ background: #45475a; padding: 12px 16px; border-radius: 6px; margin-bottom: 16px; border-left: 4px solid #f38ba8; }}
        .section {{ margin-bottom: 16px; }}
        .section h2 {{ font-size: 0.9rem; color: #a6adc8; margin-bottom: 8px; text-transform: uppercase; letter-spacing: 1px; }}
        .file {{ background: #1e1e2e; padding: 10px 16px; border-radius: 6px; font-size: 0.85rem; overflow-x: auto; }}
        .file .path {{ color: #89b4fa; }}
        .file .line {{ color: #fab387; }}
        .trace {{ background: #1e1e2e; padding: 12px 16px; border-radius: 6px; font-size: 0.8rem; white-space: pre-wrap; max-height: 300px; overflow-y: auto; }}
        .request {{ background: #1e1e2e; padding: 12px 16px; border-radius: 6px; font-size: 0.8rem; white-space: pre-wrap; max-height: 200px; overflow-y: auto; }}
        .brand {{ text-align: center; margin-top: 20px; color: #585b70; font-size: 0.8rem; }}
    </style>
</head>
<body>
    <div class="header">
        <div class="status">HTTP {}</div>
        <h1>{}</h1>
    </div>
    <div class="content">
        <div class="message">{}</div>
        <div class="section">
            <h2>文件位置</h2>
            <div class="file">
                <span class="path">{}</span> <span class="line">第 {} 行</span>
            </div>
        </div>
        <div class="section">
            <h2>调用栈</h2>
            <div class="trace">{}</div>
        </div>
        <div class="section">
            <h2>请求信息</h2>
            <div class="request">{}</div>
        </div>
    </div>
    <div class="brand">OYTAPHP v{} | 调试模式</div>
</body>
</html>"#,
            title, status, title, message, file, line, trace, request_info, env!("CARGO_PKG_VERSION"));

        let mut resp = Self::html(html);
        resp.status_code = status;
        resp
    }

    /// 设置响应头
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// 设置状态码
    pub fn status(mut self, code: u16) -> Self {
        self.status_code = code;
        self
    }

    /// 设置 Content-Type
    pub fn content_type(mut self, ct: &str) -> Self {
        self.content_type = ct.to_string();
        self
    }

    /// 设置响应体
    pub fn with_body(mut self, body: String) -> Self {
        self.body = body;
        self
    }

    /// 设置 Cookie
    pub fn cookie(mut self, name: &str, value: &str, expire: i64) -> Self {
        let cookie = if expire > 0 {
            format!("{}={}; Max-Age={}; Path=/; HttpOnly; SameSite=Lax", name, value, expire)
        } else {
            format!("{}={}; Path=/; HttpOnly; SameSite=Lax", name, value)
        };
        self.headers.insert("Set-Cookie".to_string(), cookie);
        self
    }
}

impl Default for OytaResponse {
    fn default() -> Self {
        Self::new()
    }
}

/// 将 OytaResponse 转换为 Axum 的 Response
impl IntoResponse for OytaResponse {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::OK);
        let mut builder = axum::http::Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, self.content_type);

        // 添加自定义响应头
        for (key, value) in &self.headers {
            if let Ok(header_name) = axum::http::header::HeaderName::from_bytes(key.as_bytes()) {
                if let Ok(header_value) = axum::http::header::HeaderValue::from_str(value) {
                    builder = builder.header(header_name, header_value);
                }
            }
        }

        builder.body(Body::from(self.body)).unwrap_or_else(|_| {
            axum::http::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal Server Error"))
                .unwrap()
        })
    }
}
