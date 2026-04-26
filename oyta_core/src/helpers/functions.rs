//! 全局辅助函数实现
//!
//! 提供 OYTAPHP 的全局辅助函数
//! 对应 ThinkPHP 8.0 的全局函数
//! 包括：配置、环境变量、URL、路径、字符串、数组、安全、调试等

use crate::config::facade::Config;
use crate::interpreter::value::Value;

/// 获取配置值（助手函数）
/// 对应 ThinkPHP 的 config() 函数
pub fn config(key: &str) -> Option<String> {
    Config::get_string(key)
}

/// 获取配置值，带默认值
pub fn config_or(key: &str, default: &str) -> String {
    Config::get_string_or(key, default)
}

/// 获取环境变量
/// 对应 ThinkPHP 的 env() 函数
pub fn env(key: &str, default: &str) -> String {
    crate::config::facade::Env::get(key, default)
}

/// JSON 编码
pub fn json_encode(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

/// JSON 解码
pub fn json_decode(s: &str) -> Value {
    serde_json::from_str::<Value>(s).unwrap_or(Value::Null)
}

/// 生成 URL
pub fn url(path: &str) -> String {
    format!("/{}", path.trim_start_matches('/'))
}

/// 获取运行时路径
pub fn runtime_path() -> String {
    std::env::current_dir()
        .map(|p| p.join("runtime").to_string_lossy().to_string())
        .unwrap_or_else(|_| "./runtime".to_string())
}

/// 获取公共路径
pub fn public_path() -> String {
    std::env::current_dir()
        .map(|p| p.join("public").to_string_lossy().to_string())
        .unwrap_or_else(|_| "./public".to_string())
}

/// 字符串转驼峰
pub fn camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut upper_next = false;
    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            upper_next = true;
        } else if upper_next {
            result.push(c.to_ascii_uppercase());
            upper_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// 字符串转蛇形
pub fn snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// 获取应用根路径
pub fn root_path() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string())
}

/// 获取应用路径
pub fn app_path() -> String {
    std::env::current_dir()
        .map(|p| p.join("app").to_string_lossy().to_string())
        .unwrap_or_else(|_| "./app".to_string())
}

/// 获取配置路径
pub fn config_path() -> String {
    std::env::current_dir()
        .map(|p| p.join("config").to_string_lossy().to_string())
        .unwrap_or_else(|_| "./config".to_string())
}

/// 获取存储路径
pub fn storage_path() -> String {
    std::env::current_dir()
        .map(|p| p.join("storage").to_string_lossy().to_string())
        .unwrap_or_else(|_| "./storage".to_string())
}

/// 生成随机字符串
pub fn str_random(length: usize) -> String {
    crate::security::crypto::random_hex(length)
}

/// 生成 UUID v4
pub fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// HTML 实体转义
pub fn escape(s: &str) -> String {
    crate::security::xss::escape_html(s)
}

/// 调试输出（dd 函数）
pub fn dd(value: &Value) -> String {
    let output = serde_json::to_string_pretty(value).unwrap_or_default();
    format!("<pre>{}</pre>", crate::security::xss::escape_html(&output))
}

/// 获取当前时间戳
pub fn now() -> i64 {
    chrono::Local::now().timestamp()
}

/// 格式化日期
pub fn date_format(format: &str) -> String {
    chrono::Local::now().format(format).to_string()
}

/// 重定向 URL
pub fn redirect(url: &str) -> String {
    url.to_string()
}

/// 判断值是否为空
pub fn is_empty(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Bool(b) => !b,
        Value::Int(i) => *i == 0,
        Value::Float(f) => *f == 0.0,
        Value::String(s) => s.is_empty(),
        Value::IndexedArray(arr) => arr.is_empty(),
        Value::AssociativeArray(map) => map.is_empty(),
        _ => false,
    }
}

/// 抛出 HTTP 异常
pub fn abort(code: u16, message: &str) -> Result<Value, String> {
    Err(format!("HTTP {}: {}", code, message))
}

/// 获取客户端 IP
pub fn client_ip() -> String {
    "127.0.0.1".to_string()
}

/// 获取当前请求的域名
pub fn domain() -> String {
    config_or("app.domain", "localhost")
}

/// 获取当前请求的完整 URL
pub fn full_url() -> String {
    format!("http://{}{}", domain(), "")
}
