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

// ==================== Request 助手函数 ====================

/// 获取当前请求对象
/// 对应 ThinkPHP 的 request() 函数
pub fn request() -> Option<crate::http::request::Request> {
    crate::http::RequestFacade::get_request()
}

/// 获取输入变量
/// 对应 ThinkPHP 的 input() 函数
/// 
/// # 用法示例
/// - input("name") - 获取 param 变量
/// - input("get.name") - 获取 GET 变量
/// - input("post.name") - 获取 POST 变量
/// - input("get.id/d") - 获取 GET 变量并转为整数
pub fn input(key: &str) -> serde_json::Value {
    input_default(key, serde_json::Value::Null)
}

/// 获取输入变量（带默认值）
pub fn input_default(key: &str, default: serde_json::Value) -> serde_json::Value {
    // 解析 key，格式为 "method.name" 或 "name"
    let (method, name) = if key.contains('.') {
        let parts: Vec<&str> = key.splitn(2, '.').collect();
        (parts[0], parts[1])
    } else {
        ("param", key)
    };

    // 检查是否为判断变量是否存在
    if name.starts_with('?') {
        let actual_name = &name[1..];
        let exists = match method {
            "get" => crate::http::RequestFacade::has(actual_name, "get"),
            "post" => crate::http::RequestFacade::has(actual_name, "post"),
            "put" => crate::http::RequestFacade::has(actual_name, "put"),
            "delete" => crate::http::RequestFacade::has(actual_name, "delete"),
            "session" => crate::http::RequestFacade::has(actual_name, "session"),
            "cookie" => crate::http::RequestFacade::has(actual_name, "cookie"),
            "server" => crate::http::RequestFacade::has(actual_name, "server"),
            "env" => crate::http::RequestFacade::has(actual_name, "env"),
            "file" => crate::http::RequestFacade::has(actual_name, "file"),
            "route" => crate::http::RequestFacade::has(actual_name, "route"),
            "middleware" => crate::http::RequestFacade::has(actual_name, "middleware"),
            "request" => crate::http::RequestFacade::has(actual_name, "request"),
            _ => crate::http::RequestFacade::has(actual_name, "param"),
        };
        return serde_json::Value::Bool(exists);
    }

    // 检查是否为获取全部变量
    if name.is_empty() || name == "." {
        let all_value: serde_json::Value = match method {
            "get" => {
                let m = crate::http::RequestFacade::get_all();
                serde_json::to_value(m).unwrap_or(serde_json::Value::Object(Default::default()))
            },
            "post" => {
                let m = crate::http::RequestFacade::post_all();
                serde_json::to_value(m).unwrap_or(serde_json::Value::Object(Default::default()))
            },
            "put" => {
                let m = crate::http::RequestFacade::put_all();
                serde_json::to_value(m).unwrap_or(serde_json::Value::Object(Default::default()))
            },
            "delete" => {
                let m = crate::http::RequestFacade::delete_all();
                serde_json::to_value(m).unwrap_or(serde_json::Value::Object(Default::default()))
            },
            "session" => {
                let m = crate::http::RequestFacade::session_all();
                serde_json::to_value(m).unwrap_or(serde_json::Value::Object(Default::default()))
            },
            "cookie" => {
                let m = crate::http::RequestFacade::cookie_all();
                serde_json::to_value(m).unwrap_or(serde_json::Value::Object(Default::default()))
            },
            "server" => {
                let m = crate::http::RequestFacade::server_all();
                serde_json::to_value(m).unwrap_or(serde_json::Value::Object(Default::default()))
            },
            "env" => {
                let m = crate::http::RequestFacade::env_all();
                serde_json::to_value(m).unwrap_or(serde_json::Value::Object(Default::default()))
            },
            "route" => {
                let m = crate::http::RequestFacade::route_all();
                serde_json::to_value(m).unwrap_or(serde_json::Value::Object(Default::default()))
            },
            "middleware" => {
                let m = crate::http::RequestFacade::middleware_all();
                serde_json::to_value(m).unwrap_or(serde_json::Value::Object(Default::default()))
            },
            _ => {
                let m = crate::http::RequestFacade::param_all(true);
                serde_json::to_value(m).unwrap_or(serde_json::Value::Object(Default::default()))
            },
        };
        return all_value;
    }

    // 获取单个变量
    match method {
        "get" => crate::http::RequestFacade::get_with_modifier(name),
        "post" => crate::http::RequestFacade::post_with_modifier(name),
        "put" => crate::http::RequestFacade::put(name).map(|v| serde_json::Value::String(v)).unwrap_or(default),
        "delete" => crate::http::RequestFacade::delete(name).map(|v| serde_json::Value::String(v)).unwrap_or(default),
        "session" => crate::http::RequestFacade::session(name).unwrap_or(default),
        "cookie" => crate::http::RequestFacade::cookie(name).map(|v| serde_json::Value::String(v)).unwrap_or(default),
        "server" => crate::http::RequestFacade::server(name).map(|v| serde_json::Value::String(v)).unwrap_or(default),
        "env" => crate::http::RequestFacade::env(name).map(|v| serde_json::Value::String(v)).unwrap_or(default),
        "route" => crate::http::RequestFacade::route(name).map(|v| serde_json::Value::String(v)).unwrap_or(default),
        "middleware" => crate::http::RequestFacade::middleware(name).unwrap_or(default),
        "request" => crate::http::RequestFacade::request(name).map(|v| serde_json::Value::String(v)).unwrap_or(default),
        _ => crate::http::RequestFacade::param_with_modifier(name),
    }
}

/// 获取 PARAM 变量
pub fn param(key: &str) -> Option<String> {
    crate::http::RequestFacade::param(key)
}

/// 获取 PARAM 变量（带默认值）
pub fn param_default(key: &str, default: &str) -> String {
    crate::http::RequestFacade::param_default(key, default)
}

/// 获取 GET 变量
pub fn get_input(key: &str) -> Option<String> {
    crate::http::RequestFacade::get(key)
}

/// 获取 GET 变量（带默认值）
pub fn get_input_default(key: &str, default: &str) -> String {
    crate::http::RequestFacade::get_default(key, default)
}

/// 获取 POST 变量
pub fn post_input(key: &str) -> Option<String> {
    crate::http::RequestFacade::post(key)
}

/// 获取 POST 变量（带默认值）
pub fn post_input_default(key: &str, default: &str) -> String {
    crate::http::RequestFacade::post_default(key, default)
}

/// 获取 Cookie 变量
pub fn cookie_input(key: &str) -> Option<String> {
    crate::http::RequestFacade::cookie(key)
}

/// 获取 Cookie 变量（带默认值）
pub fn cookie_input_default(key: &str, default: &str) -> String {
    crate::http::RequestFacade::cookie_default(key, default)
}

/// 获取 Session 变量
pub fn session_input(key: &str) -> Option<serde_json::Value> {
    crate::http::RequestFacade::session(key)
}

/// 获取 Session 变量（带默认值）
pub fn session_input_default(key: &str, default: serde_json::Value) -> serde_json::Value {
    crate::http::RequestFacade::session_default(key, default)
}

/// 判断变量是否设置
pub fn input_has(key: &str, method: &str) -> bool {
    crate::http::RequestFacade::has(key, method)
}

/// 获取部分变量
pub fn input_only(keys: &[&str], method: &str) -> std::collections::HashMap<String, String> {
    crate::http::RequestFacade::only(keys, method)
}

/// 排除某些变量后获取
pub fn input_except(keys: &[&str], method: &str) -> std::collections::HashMap<String, String> {
    crate::http::RequestFacade::except(keys, method)
}

/// 获取请求头
pub fn header(key: &str) -> Option<String> {
    crate::http::RequestFacade::header(key)
}

/// 获取请求头（带默认值）
pub fn header_default(key: &str, default: &str) -> String {
    crate::http::RequestFacade::header_default(key, default)
}

/// 判断是否为 AJAX 请求
pub fn is_ajax() -> bool {
    crate::http::RequestFacade::is_ajax()
}

/// 判断是否为 POST 请求
pub fn is_post() -> bool {
    crate::http::RequestFacade::is_post()
}

/// 判断是否为 GET 请求
pub fn is_get() -> bool {
    crate::http::RequestFacade::is_get()
}

/// 判断是否为 JSON 请求
pub fn is_json() -> bool {
    crate::http::RequestFacade::is_json()
}

/// 判断是否为手机访问
pub fn is_mobile() -> bool {
    crate::http::RequestFacade::is_mobile()
}

/// 获取当前请求类型
pub fn method(original: bool) -> String {
    crate::http::RequestFacade::method(original)
}

/// 获取当前 URL
pub fn current_url(complete: bool) -> String {
    crate::http::RequestFacade::url(complete)
}

/// 获取当前域名
pub fn current_domain() -> String {
    crate::http::RequestFacade::domain()
}

/// 获取当前路径
pub fn current_path() -> String {
    crate::http::RequestFacade::path()
}

/// 获取客户端 IP（从请求对象获取）
pub fn request_ip() -> String {
    crate::http::RequestFacade::ip()
}
