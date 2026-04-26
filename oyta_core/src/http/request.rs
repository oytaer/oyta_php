//! HTTP 请求对象模块
//!
//! 封装 Axum 的请求对象，提供 ThinkPHP 风格的请求访问接口
//! 支持 param/input/header/cookie/file 等方法

use serde_json::Value;
use std::collections::HashMap;

/// OYTAPHP 请求对象
/// 封装 Axum 的请求信息，提供 ThinkPHP 风格的访问接口
#[derive(Debug, Clone)]
pub struct Request {
    /// 请求方法（GET/POST/PUT/DELETE 等）
    pub method: String,
    /// 请求 URI
    pub uri: String,
    /// 请求路径（不含查询参数）
    pub path: String,
    /// 查询参数字符串
    pub query_string: Option<String>,
    /// HTTP 版本
    pub version: String,
    /// 请求头
    pub headers: HashMap<String, String>,
    /// GET 查询参数
    pub query_params: HashMap<String, String>,
    /// POST 表单数据
    pub form_data: HashMap<String, String>,
    /// JSON 请求体（如果是 JSON 请求）
    pub json_data: Option<Value>,
    /// 路由参数（如 /user/:id 中的 id）
    pub route_params: HashMap<String, String>,
    /// 客户端 IP 地址
    pub client_ip: Option<String>,
    /// 请求体原始字节
    pub body: Option<Vec<u8>>,
}

impl Request {
    /// 创建空的请求对象
    pub fn new() -> Self {
        Self {
            method: String::new(),
            uri: String::new(),
            path: String::new(),
            query_string: None,
            version: String::new(),
            headers: HashMap::new(),
            query_params: HashMap::new(),
            form_data: HashMap::new(),
            json_data: None,
            route_params: HashMap::new(),
            client_ip: None,
            body: None,
        }
    }

    /// 从 Axum 的请求部件构建 Request
    pub async fn from_axum_parts(parts: axum::http::request::Parts) -> Self {
        let method = parts.method.to_string();
        let uri = parts.uri.to_string();
        let path = parts.uri.path().to_string();
        let query_string = parts.uri.query().map(|q| q.to_string());

        let version = format!("{:?}", parts.version);

        // 提取请求头
        let mut headers = HashMap::new();
        for (name, value) in parts.headers.iter() {
            if let Ok(v) = value.to_str() {
                headers.insert(name.as_str().to_lowercase(), v.to_string());
            }
        }

        // 解析查询参数
        let mut query_params = HashMap::new();
        if let Some(qs) = &query_string {
            for pair in qs.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    query_params.insert(
                        urldecode(key),
                        urldecode(value),
                    );
                }
            }
        }

        Self {
            method,
            uri,
            path,
            query_string,
            version,
            headers,
            query_params,
            form_data: HashMap::new(),
            json_data: None,
            route_params: HashMap::new(),
            client_ip: None,
            body: None,
        }
    }

    /// 获取请求参数（优先级：路由参数 > POST 数据 > GET 参数）
    /// 对应 ThinkPHP 的 input() 方法
    pub fn input(&self, key: &str, default: &str) -> String {
        // 优先从路由参数获取
        if let Some(val) = self.route_params.get(key) {
            return val.clone();
        }
        // 然后从 POST 数据获取
        if let Some(val) = self.form_data.get(key) {
            return val.clone();
        }
        // 最后从 GET 参数获取
        if let Some(val) = self.query_params.get(key) {
            return val.clone();
        }
        default.to_string()
    }

    /// 获取 GET 参数
    /// 对应 ThinkPHP 的 param() 方法（GET 部分）
    pub fn query(&self, key: &str, default: &str) -> String {
        self.query_params.get(key).cloned().unwrap_or_else(|| default.to_string())
    }

    /// 获取 POST 参数
    pub fn post(&self, key: &str, default: &str) -> String {
        self.form_data.get(key).cloned().unwrap_or_else(|| default.to_string())
    }

    /// 获取路由参数
    pub fn param(&self, key: &str, default: &str) -> String {
        self.route_params.get(key).cloned().unwrap_or_else(|| default.to_string())
    }

    /// 获取请求头
    pub fn header(&self, key: &str) -> Option<String> {
        self.headers.get(&key.to_lowercase()).cloned()
    }

    /// 获取所有请求头
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// 判断是否为 GET 请求
    pub fn is_get(&self) -> bool {
        self.method == "GET"
    }

    /// 判断是否为 POST 请求
    pub fn is_post(&self) -> bool {
        self.method == "POST"
    }

    /// 判断是否为 PUT 请求
    pub fn is_put(&self) -> bool {
        self.method == "PUT"
    }

    /// 判断是否为 DELETE 请求
    pub fn is_delete(&self) -> bool {
        self.method == "DELETE"
    }

    /// 判断是否为 AJAX 请求
    pub fn is_ajax(&self) -> bool {
        self.headers.get("x-requested-with")
            .map(|v| v.to_lowercase() == "xmlhttprequest")
            .unwrap_or(false)
    }

    /// 判断是否为 JSON 请求
    pub fn is_json(&self) -> bool {
        self.headers.get("content-type")
            .map(|v| v.contains("application/json"))
            .unwrap_or(false)
    }

    /// 获取 JSON 请求体中的字段
    pub fn json(&self, key: &str) -> Option<&Value> {
        self.json_data.as_ref()?.get(key)
    }

    /// 获取客户端 IP
    pub fn ip(&self) -> String {
        // 优先从 X-Forwarded-For / X-Real-IP 获取
        if let Some(ip) = self.headers.get("x-forwarded-for") {
            let ips: Vec<&str> = ip.split(',').collect();
            if let Some(first) = ips.first() {
                return first.trim().to_string();
            }
        }
        if let Some(ip) = self.headers.get("x-real-ip") {
            return ip.clone();
        }
        self.client_ip.clone().unwrap_or_else(|| "127.0.0.1".to_string())
    }

    /// 获取 Content-Type
    pub fn content_type(&self) -> Option<String> {
        self.headers.get("content-type").cloned()
    }
}

impl Default for Request {
    fn default() -> Self {
        Self::new()
    }
}

/// 简单的 URL 解码
fn urldecode(s: &str) -> String {
    let mut result = Vec::new();
    let mut chars = s.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let hex: String = chars.by_ref().take(2).map(|c| c as char).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte);
            } else {
                result.push(b'%');
                result.extend(hex.bytes());
            }
        } else if b == b'+' {
            result.push(b' ');
        } else {
            result.push(b);
        }
    }
    String::from_utf8_lossy(&result).to_string()
}
