//! HTTP 请求对象模块
//!
//! 封装 Axum 的请求对象，提供 ThinkPHP 风格的请求访问接口
//! 支持 param/input/header/cookie/file 等方法
//! 完整实现 ThinkPHP 8.0 Request 类的所有功能

use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use chrono::{DateTime, Utc};
use regex::Regex;

/// 变量修饰符枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableModifier {
    /// 强制转换为字符串类型
    String,
    /// 强制转换为整型类型
    Int,
    /// 强制转换为布尔类型
    Bool,
    /// 强制转换为数组类型
    Array,
    /// 强制转换为浮点类型
    Float,
}

/// 上传文件信息
#[derive(Debug, Clone)]
pub struct UploadedFile {
    /// 文件名
    pub name: String,
    /// 临时文件路径
    pub tmp_name: String,
    /// 文件类型
    pub mime_type: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 上传错误码
    pub error: i32,
}

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

    // ========== 新增字段 ==========
    /// 服务器变量 ($_SERVER)
    pub server_params: HashMap<String, String>,
    /// 环境变量 ($_ENV)
    pub env_params: HashMap<String, String>,
    /// Cookie 数据
    pub cookies: HashMap<String, String>,
    /// Session 数据
    pub session_data: HashMap<String, Value>,
    /// 上传文件信息
    pub files: HashMap<String, Vec<UploadedFile>>,
    /// PUT 数据
    pub put_data: HashMap<String, String>,
    /// DELETE 数据
    pub delete_data: HashMap<String, String>,
    /// 中间件传递的变量
    pub middleware_data: HashMap<String, Value>,
    /// 当前控制器名
    pub controller: Option<String>,
    /// 当前操作名
    pub action: Option<String>,
    /// 当前控制器分级目录
    pub layer: Option<String>,
    /// 当前路由对象实例（路由规则名称）
    pub route_rule: Option<String>,
    /// 当前应用名（多应用模式）
    pub app_name: Option<String>,
    /// 请求时间
    pub request_time: Option<DateTime<Utc>>,
    /// 全局过滤方法
    pub filter: Vec<String>,
    /// 请求类型伪装变量名
    pub var_method: String,
    /// AJAX 伪装变量名
    pub var_ajax: String,
    /// PJAX 伪装变量名
    pub var_pjax: String,
    /// 伪静态后缀
    pub url_suffix: Option<String>,
    /// 原始请求类型（伪装前的真实类型）
    pub original_method: Option<String>,
    /// 客户端地址（SocketAddr）
    pub remote_addr: Option<SocketAddr>,
    /// 服务器端口
    pub server_port: Option<u16>,
    /// 是否为 CLI 模式
    pub is_cli: bool,
    /// 是否为 CGI 模式
    pub is_cgi: bool,
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
            // 新增字段初始化
            server_params: HashMap::new(),
            env_params: HashMap::new(),
            cookies: HashMap::new(),
            session_data: HashMap::new(),
            files: HashMap::new(),
            put_data: HashMap::new(),
            delete_data: HashMap::new(),
            middleware_data: HashMap::new(),
            controller: None,
            action: None,
            layer: None,
            route_rule: None,
            app_name: None,
            request_time: Some(Utc::now()),
            filter: Vec::new(),
            var_method: "_method".to_string(),
            var_ajax: "_ajax".to_string(),
            var_pjax: "_pjax".to_string(),
            url_suffix: None,
            original_method: None,
            remote_addr: None,
            server_port: None,
            is_cli: false,
            is_cgi: false,
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

        // 解析 Cookie
        let mut cookies = HashMap::new();
        if let Some(cookie_header) = headers.get("cookie") {
            for pair in cookie_header.split(';') {
                let pair = pair.trim();
                if let Some((key, value)) = pair.split_once('=') {
                    cookies.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }

        Self {
            method: method.clone(),
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
            // 新增字段初始化
            server_params: HashMap::new(),
            env_params: HashMap::new(),
            cookies,
            session_data: HashMap::new(),
            files: HashMap::new(),
            put_data: HashMap::new(),
            delete_data: HashMap::new(),
            middleware_data: HashMap::new(),
            controller: None,
            action: None,
            layer: None,
            route_rule: None,
            app_name: None,
            request_time: Some(Utc::now()),
            filter: Vec::new(),
            var_method: "_method".to_string(),
            var_ajax: "_ajax".to_string(),
            var_pjax: "_pjax".to_string(),
            url_suffix: None,
            original_method: Some(method),
            remote_addr: None,
            server_port: None,
            is_cli: false,
            is_cgi: false,
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

    // ==================== 请求信息获取方法 ====================

    /// 获取当前访问域名或IP
    /// 对应 ThinkPHP 的 host() 方法
    pub fn host(&self) -> String {
        self.headers.get("host")
            .cloned()
            .unwrap_or_else(|| "localhost".to_string())
    }

    /// 获取当前访问协议
    /// 对应 ThinkPHP 的 scheme() 方法
    pub fn scheme(&self) -> String {
        if self.headers.get("x-forwarded-proto").map(|s| s.as_str()) == Some("https") {
            return "https".to_string();
        }
        if self.uri.starts_with("https://") {
            return "https".to_string();
        }
        "http".to_string()
    }

    /// 获取当前访问的端口
    /// 对应 ThinkPHP 的 port() 方法
    pub fn port(&self) -> u16 {
        self.server_port.unwrap_or(if self.scheme() == "https" { 443 } else { 80 })
    }

    /// 获取当前请求的 REMOTE_PORT
    /// 对应 ThinkPHP 的 remotePort() 方法
    pub fn remote_port(&self) -> Option<u16> {
        self.remote_addr.map(|addr| addr.port())
    }

    /// 获取当前请求的 SERVER_PROTOCOL
    /// 对应 ThinkPHP 的 protocol() 方法
    pub fn protocol(&self) -> String {
        self.version.clone()
    }

    /// 获取当前包含协议的域名
    /// 对应 ThinkPHP 的 domain() 方法
    pub fn domain(&self) -> String {
        format!("{}://{}", self.scheme(), self.host())
    }

    /// 获取当前访问的子域名
    /// 对应 ThinkPHP 的 subDomain() 方法
    pub fn sub_domain(&self) -> String {
        let host = self.host();
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() > 2 {
            parts[0].to_string()
        } else {
            String::new()
        }
    }

    /// 获取当前访问的泛域名
    /// 对应 ThinkPHP 的 panDomain() 方法
    pub fn pan_domain(&self) -> String {
        let host = self.host();
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() > 2 {
            parts[..parts.len() - 2].join(".")
        } else {
            String::new()
        }
    }

    /// 获取当前访问的根域名
    /// 对应 ThinkPHP 的 rootDomain() 方法
    pub fn root_domain(&self) -> String {
        let host = self.host();
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() >= 2 {
            parts[parts.len() - 2..].join(".")
        } else {
            host
        }
    }

    /// 获取当前完整URL
    /// 对应 ThinkPHP 的 url() 方法
    /// @param complete 是否包含域名
    pub fn url(&self, complete: bool) -> String {
        if complete {
            if let Some(ref qs) = self.query_string {
                format!("{}://{}{}?{}", self.scheme(), self.host(), self.path, qs)
            } else {
                format!("{}://{}{}", self.scheme(), self.host(), self.path)
            }
        } else {
            if let Some(ref qs) = self.query_string {
                format!("{}?{}", self.path, qs)
            } else {
                self.path.clone()
            }
        }
    }

    /// 获取当前URL（不含QUERY_STRING）
    /// 对应 ThinkPHP 的 baseUrl() 方法
    /// @param complete 是否包含域名
    pub fn base_url(&self, complete: bool) -> String {
        if complete {
            format!("{}://{}{}", self.scheme(), self.host(), self.path)
        } else {
            self.path.clone()
        }
    }

    /// 获取当前请求的QUERY_STRING参数
    /// 对应 ThinkPHP 的 query() 方法（注意：与获取GET参数的query方法不同）
    pub fn query_string_value(&self) -> String {
        self.query_string.clone().unwrap_or_default()
    }

    /// 获取当前执行的文件
    /// 对应 ThinkPHP 的 baseFile() 方法
    pub fn base_file(&self) -> String {
        self.server_params.get("SCRIPT_NAME")
            .cloned()
            .unwrap_or_else(|| "/index.php".to_string())
    }

    /// 获取URL访问根地址
    /// 对应 ThinkPHP 的 root() 方法
    /// @param complete 是否包含域名
    pub fn root(&self, complete: bool) -> String {
        let script_name = self.base_file();
        let root = if script_name.ends_with(".php") {
            script_name.rfind('/').map(|i| &script_name[..i]).unwrap_or("")
        } else {
            &script_name
        };
        if complete {
            format!("{}://{}{}", self.scheme(), self.host(), root)
        } else {
            root.to_string()
        }
    }

    /// 获取URL访问根目录
    /// 对应 ThinkPHP 的 rootUrl() 方法
    pub fn root_url(&self) -> String {
        self.root(false)
    }

    /// 获取当前请求URL的pathinfo信息（含URL后缀）
    /// 对应 ThinkPHP 的 pathinfo() 方法
    pub fn path_info(&self) -> String {
        self.path.clone()
    }

    /// 获取当前URL的访问后缀
    /// 对应 ThinkPHP 的 ext() 方法
    pub fn ext(&self) -> String {
        if let Some(ref suffix) = self.url_suffix {
            return suffix.clone();
        }
        let path = self.path.clone();
        if let Some(pos) = path.rfind('.') {
            path[pos + 1..].to_string()
        } else {
            String::new()
        }
    }

    /// 获取当前请求的时间
    /// 对应 ThinkPHP 的 time() 方法
    pub fn time(&self) -> i64 {
        self.request_time.map(|t| t.timestamp()).unwrap_or(0)
    }

    /// 获取当前请求的时间（DateTime格式）
    pub fn time_datetime(&self) -> Option<DateTime<Utc>> {
        self.request_time
    }

    /// 获取当前请求的资源类型
    /// 对应 ThinkPHP 的 type() 方法
    pub fn type_name(&self) -> String {
        self.headers.get("accept")
            .cloned()
            .unwrap_or_else(|| "*/*".to_string())
    }

    /// 获取当前请求类型
    /// 对应 ThinkPHP 的 method() 方法
    /// @param original 是否获取原始请求类型（伪装前）
    pub fn method(&self, original: bool) -> String {
        if original {
            self.original_method.clone().unwrap_or_else(|| self.method.clone())
        } else {
            self.method.clone()
        }
    }

    /// 获取当前请求的路由对象实例
    /// 对应 ThinkPHP 的 rule() 方法
    pub fn rule(&self) -> Option<&String> {
        self.route_rule.as_ref()
    }

    // ==================== 控制器/操作信息获取 ====================

    /// 获取当前请求的控制器名
    /// 对应 ThinkPHP 的 controller() 方法
    /// @param lowercase 是否转换为小写
    /// @param base 是否只返回实际控制器名（不含分级目录）
    pub fn controller_name(&self, lowercase: bool, base: bool) -> String {
        let controller = if base {
            self.controller.as_ref()
                .and_then(|c| c.rsplit('/').next())
                .unwrap_or("")
                .to_string()
        } else {
            self.controller.clone().unwrap_or_default()
        };
        if lowercase {
            controller.to_lowercase()
        } else {
            controller
        }
    }

    /// 获取当前请求的操作名
    /// 对应 ThinkPHP 的 action() 方法
    /// @param lowercase 是否转换为小写
    pub fn action_name(&self, lowercase: bool) -> String {
        let action = self.action.clone().unwrap_or_default();
        if lowercase {
            action.to_lowercase()
        } else {
            action
        }
    }

    /// 获取当前请求的控制器分级目录
    /// 对应 ThinkPHP 的 layer() 方法
    pub fn layer_name(&self) -> String {
        self.layer.clone().unwrap_or_default()
    }

    /// 获取当前应用名（多应用模式）
    pub fn app_name_value(&self) -> String {
        self.app_name.clone().unwrap_or_default()
    }

    // ==================== 变量获取方法 ====================

    /// 获取PARAM变量（自动识别当前请求类型）
    /// 对应 ThinkPHP 的 param() 方法
    /// 优先级：路由参数 > POST数据 > GET参数
    pub fn param_value(&self, key: &str) -> Option<String> {
        // 先检查路由参数
        if let Some(val) = self.route_params.get(key) {
            return Some(val.clone());
        }
        // 再检查POST数据
        if let Some(val) = self.form_data.get(key) {
            return Some(val.clone());
        }
        // 最后检查GET参数
        self.query_params.get(key).cloned()
    }

    /// 获取PARAM变量（带默认值）
    pub fn param_default(&self, key: &str, default: &str) -> String {
        self.param_value(key).unwrap_or_else(|| default.to_string())
    }

    /// 获取所有PARAM变量
    pub fn param_all(&self, filter: bool) -> HashMap<String, String> {
        let mut result = HashMap::new();
        // 先添加GET参数
        for (k, v) in &self.query_params {
            result.insert(k.clone(), v.clone());
        }
        // 再添加POST数据（覆盖同名GET参数）
        for (k, v) in &self.form_data {
            result.insert(k.clone(), v.clone());
        }
        // 最后添加路由参数（覆盖同名参数）
        for (k, v) in &self.route_params {
            result.insert(k.clone(), v.clone());
        }
        if filter && !self.filter.is_empty() {
            // 应用全局过滤
            for (_, v) in result.iter_mut() {
                *v = self.apply_filter(v.clone());
            }
        }
        result
    }

    /// 获取GET变量
    /// 对应 ThinkPHP 的 get() 方法
    pub fn get(&self, key: &str) -> Option<String> {
        self.query_params.get(key).cloned()
    }

    /// 获取GET变量（带默认值）
    pub fn get_default(&self, key: &str, default: &str) -> String {
        self.query_params.get(key).cloned().unwrap_or_else(|| default.to_string())
    }

    /// 获取所有GET变量
    pub fn get_all(&self) -> &HashMap<String, String> {
        &self.query_params
    }

    /// 获取POST变量
    /// 对应 ThinkPHP 的 post() 方法（增强版）
    pub fn post_value(&self, key: &str) -> Option<String> {
        self.form_data.get(key).cloned()
    }

    /// 获取所有POST变量
    pub fn post_all(&self) -> &HashMap<String, String> {
        &self.form_data
    }

    /// 获取PUT变量
    /// 对应 ThinkPHP 的 put() 方法
    pub fn put(&self, key: &str) -> Option<String> {
        self.put_data.get(key).cloned()
    }

    /// 获取PUT变量（带默认值）
    pub fn put_default(&self, key: &str, default: &str) -> String {
        self.put_data.get(key).cloned().unwrap_or_else(|| default.to_string())
    }

    /// 获取所有PUT变量
    pub fn put_all(&self) -> &HashMap<String, String> {
        &self.put_data
    }

    /// 获取DELETE变量
    /// 对应 ThinkPHP 的 delete() 方法
    pub fn delete(&self, key: &str) -> Option<String> {
        self.delete_data.get(key).cloned()
    }

    /// 获取DELETE变量（带默认值）
    pub fn delete_default(&self, key: &str, default: &str) -> String {
        self.delete_data.get(key).cloned().unwrap_or_else(|| default.to_string())
    }

    /// 获取所有DELETE变量
    pub fn delete_all(&self) -> &HashMap<String, String> {
        &self.delete_data
    }

    /// 获取SESSION变量
    /// 对应 ThinkPHP 的 session() 方法
    pub fn session(&self, key: &str) -> Option<&Value> {
        self.session_data.get(key)
    }

    /// 获取SESSION变量（带默认值）
    pub fn session_default(&self, key: &str, default: Value) -> Value {
        self.session_data.get(key).cloned().unwrap_or(default)
    }

    /// 获取所有SESSION变量
    pub fn session_all(&self) -> &HashMap<String, Value> {
        &self.session_data
    }

    /// 获取COOKIE变量
    /// 对应 ThinkPHP 的 cookie() 方法
    pub fn cookie(&self, key: &str) -> Option<String> {
        self.cookies.get(key).cloned()
    }

    /// 获取COOKIE变量（带默认值）
    pub fn cookie_default(&self, key: &str, default: &str) -> String {
        self.cookies.get(key).cloned().unwrap_or_else(|| default.to_string())
    }

    /// 获取所有COOKIE变量
    pub fn cookie_all(&self) -> &HashMap<String, String> {
        &self.cookies
    }

    /// 获取REQUEST变量（GET + POST + COOKIE）
    /// 对应 ThinkPHP 的 request() 方法
    pub fn request(&self, key: &str) -> Option<String> {
        // 先检查GET
        if let Some(val) = self.query_params.get(key) {
            return Some(val.clone());
        }
        // 再检查POST
        if let Some(val) = self.form_data.get(key) {
            return Some(val.clone());
        }
        // 最后检查COOKIE
        self.cookies.get(key).cloned()
    }

    /// 获取REQUEST变量（带默认值）
    pub fn request_default(&self, key: &str, default: &str) -> String {
        self.request(key).unwrap_or_else(|| default.to_string())
    }

    /// 获取SERVER变量
    /// 对应 ThinkPHP 的 server() 方法
    pub fn server(&self, key: &str) -> Option<String> {
        let key_upper = key.to_uppercase();
        self.server_params.get(&key_upper).cloned()
    }

    /// 获取SERVER变量（带默认值）
    pub fn server_default(&self, key: &str, default: &str) -> String {
        self.server(key).unwrap_or_else(|| default.to_string())
    }

    /// 获取所有SERVER变量
    pub fn server_all(&self) -> &HashMap<String, String> {
        &self.server_params
    }

    /// 获取ENV变量
    /// 对应 ThinkPHP 的 env() 方法
    pub fn env(&self, key: &str) -> Option<String> {
        let key_upper = key.to_uppercase();
        self.env_params.get(&key_upper).cloned()
    }

    /// 获取ENV变量（带默认值）
    pub fn env_default(&self, key: &str, default: &str) -> String {
        self.env(key).unwrap_or_else(|| default.to_string())
    }

    /// 获取所有ENV变量
    pub fn env_all(&self) -> &HashMap<String, String> {
        &self.env_params
    }

    /// 获取路由变量
    /// 对应 ThinkPHP 的 route() 方法
    pub fn route(&self, key: &str) -> Option<String> {
        self.route_params.get(key).cloned()
    }

    /// 获取路由变量（带默认值）
    pub fn route_default(&self, key: &str, default: &str) -> String {
        self.route_params.get(key).cloned().unwrap_or_else(|| default.to_string())
    }

    /// 获取所有路由变量
    pub fn route_all(&self) -> &HashMap<String, String> {
        &self.route_params
    }

    /// 获取中间件传递的变量
    /// 对应 ThinkPHP 的 middleware() 方法
    pub fn middleware(&self, key: &str) -> Option<&Value> {
        self.middleware_data.get(key)
    }

    /// 获取中间件传递的变量（带默认值）
    pub fn middleware_default(&self, key: &str, default: Value) -> Value {
        self.middleware_data.get(key).cloned().unwrap_or(default)
    }

    /// 获取所有中间件传递的变量
    pub fn middleware_all(&self) -> &HashMap<String, Value> {
        &self.middleware_data
    }

    /// 获取上传文件
    /// 对应 ThinkPHP 的 file() 方法
    pub fn file(&self, key: &str) -> Option<&Vec<UploadedFile>> {
        self.files.get(key)
    }

    /// 获取单个上传文件
    pub fn file_one(&self, key: &str) -> Option<&UploadedFile> {
        self.files.get(key).and_then(|v| v.first())
    }

    /// 获取所有上传文件
    pub fn file_all(&self) -> &HashMap<String, Vec<UploadedFile>> {
        &self.files
    }

    /// 获取所有请求变量（包括文件）
    /// 对应 ThinkPHP 的 all() 方法
    pub fn all(&self) -> HashMap<String, Value> {
        let mut result = HashMap::new();
        // 添加所有参数
        for (k, v) in self.param_all(true) {
            result.insert(k, Value::String(v));
        }
        // 添加文件信息
        for (k, files) in &self.files {
            let files_json: Vec<Value> = files.iter().map(|f| {
                serde_json::json!({
                    "name": f.name,
                    "tmp_name": f.tmp_name,
                    "type": f.mime_type,
                    "size": f.size,
                    "error": f.error
                })
            }).collect();
            result.insert(k.clone(), Value::Array(files_json));
        }
        result
    }

    // ==================== 变量操作方法 ====================

    /// 检测变量是否设置
    /// 对应 ThinkPHP 的 has() 方法
    pub fn has(&self, key: &str, method: &str) -> bool {
        match method.to_lowercase().as_str() {
            "get" => self.query_params.contains_key(key),
            "post" => self.form_data.contains_key(key),
            "put" => self.put_data.contains_key(key),
            "delete" => self.delete_data.contains_key(key),
            "param" => self.param_value(key).is_some(),
            "session" => self.session_data.contains_key(key),
            "cookie" => self.cookies.contains_key(key),
            "server" => self.server_params.contains_key(&key.to_uppercase()),
            "env" => self.env_params.contains_key(&key.to_uppercase()),
            "file" => self.files.contains_key(key),
            "route" => self.route_params.contains_key(key),
            "middleware" => self.middleware_data.contains_key(key),
            "request" => self.request(key).is_some(),
            _ => false,
        }
    }

    /// 获取部分变量
    /// 对应 ThinkPHP 的 only() 方法
    pub fn only(&self, keys: &[&str], method: &str) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for key in keys {
            let value = match method.to_lowercase().as_str() {
                "get" => self.get(key),
                "post" => self.post_value(key),
                "put" => self.put(key),
                "delete" => self.delete(key),
                "param" => self.param_value(key),
                "cookie" => self.cookie(key),
                "route" => self.route(key),
                _ => self.param_value(key),
            };
            if let Some(v) = value {
                result.insert(key.to_string(), v);
            }
        }
        result
    }

    /// 获取部分变量（带默认值）
    pub fn only_with_defaults(&self, keys_defaults: HashMap<&str, &str>, method: &str) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for (key, default) in keys_defaults {
            let value = match method.to_lowercase().as_str() {
                "get" => self.get_default(key, default),
                "post" => self.post(key, default),
                "put" => self.put_default(key, default),
                "delete" => self.delete_default(key, default),
                "param" => self.param_default(key, default),
                "cookie" => self.cookie_default(key, default),
                "route" => self.route_default(key, default),
                _ => self.param_default(key, default),
            };
            result.insert(key.to_string(), value);
        }
        result
    }

    /// 排除某些变量后获取
    /// 对应 ThinkPHP 的 except() 方法
    pub fn except(&self, keys: &[&str], method: &str) -> HashMap<String, String> {
        let all = match method.to_lowercase().as_str() {
            "get" => self.get_all().clone(),
            "post" => self.post_all().clone(),
            "put" => self.put_all().clone(),
            "delete" => self.delete_all().clone(),
            "param" => self.param_all(true),
            "cookie" => self.cookie_all().clone(),
            "route" => self.route_all().clone(),
            _ => self.param_all(true),
        };
        all.into_iter()
            .filter(|(k, _)| !keys.contains(&k.as_str()))
            .collect()
    }

    /// 设置全局过滤方法
    /// 对应 ThinkPHP 的 filter() 方法
    pub fn set_filter(&mut self, filter: Vec<String>) {
        self.filter = filter;
    }

    /// 获取全局过滤方法
    pub fn get_filter(&self) -> &Vec<String> {
        &self.filter
    }

    /// 应用过滤方法
    fn apply_filter(&self, value: String) -> String {
        let mut result = value;
        for filter_name in &self.filter {
            result = match filter_name.as_str() {
                "htmlspecialchars" => html_escape::encode_text(&result).to_string(),
                "strip_tags" => {
                    // 简单的HTML标签去除
                    let re = Regex::new(r"<[^>]*>").unwrap();
                    re.replace_all(&result, "").to_string()
                }
                "trim" => result.trim().to_string(),
                "strtolower" => result.to_lowercase(),
                "strtoupper" => result.to_uppercase(),
                _ => result,
            };
        }
        result
    }

    // ==================== 变量修饰符支持 ====================

    /// 解析变量名和修饰符
    /// 格式：变量名/修饰符，如 id/d
    fn parse_key_modifier(&self, key: &str) -> (String, Option<VariableModifier>) {
        if let Some(pos) = key.find('/') {
            let name = key[..pos].to_string();
            let modifier = match key[pos + 1..].to_lowercase().as_str() {
                "s" => Some(VariableModifier::String),
                "d" => Some(VariableModifier::Int),
                "b" => Some(VariableModifier::Bool),
                "a" => Some(VariableModifier::Array),
                "f" => Some(VariableModifier::Float),
                _ => None,
            };
            (name, modifier)
        } else {
            (key.to_string(), None)
        }
    }

    /// 应用变量修饰符
    fn apply_modifier(&self, value: String, modifier: Option<VariableModifier>) -> Value {
        match modifier {
            Some(VariableModifier::String) => Value::String(value),
            Some(VariableModifier::Int) => {
                Value::Number(value.parse::<i64>().unwrap_or(0).into())
            }
            Some(VariableModifier::Bool) => {
                let lower = value.to_lowercase();
                Value::Bool(matches!(lower.as_str(), "true" | "1" | "on" | "yes"))
            }
            Some(VariableModifier::Array) => {
                // 尝试解析为JSON数组，否则返回空数组
                serde_json::from_str(&value).unwrap_or(Value::Array(vec![]))
            }
            Some(VariableModifier::Float) => {
                Value::Number(serde_json::Number::from_f64(value.parse::<f64>().unwrap_or(0.0)).unwrap_or_else(|| 0.into()))
            }
            None => Value::String(value),
        }
    }

    /// 获取变量并应用修饰符
    /// 对应 ThinkPHP 的变量修饰符功能
    pub fn param_with_modifier(&self, key: &str) -> Value {
        let (name, modifier) = self.parse_key_modifier(key);
        if let Some(value) = self.param_value(&name) {
            self.apply_modifier(value, modifier)
        } else {
            Value::Null
        }
    }

    /// 获取GET变量并应用修饰符
    pub fn get_with_modifier(&self, key: &str) -> Value {
        let (name, modifier) = self.parse_key_modifier(key);
        if let Some(value) = self.get(&name) {
            self.apply_modifier(value, modifier)
        } else {
            Value::Null
        }
    }

    /// 获取POST变量并应用修饰符
    pub fn post_with_modifier(&self, key: &str) -> Value {
        let (name, modifier) = self.parse_key_modifier(key);
        if let Some(value) = self.post_value(&name) {
            self.apply_modifier(value, modifier)
        } else {
            Value::Null
        }
    }

    // ==================== 请求类型判断方法 ====================

    /// 判断是否为 HEAD 请求
    pub fn is_head(&self) -> bool {
        self.method == "HEAD"
    }

    /// 判断是否为 PATCH 请求
    pub fn is_patch(&self) -> bool {
        self.method == "PATCH"
    }

    /// 判断是否为 OPTIONS 请求
    pub fn is_options(&self) -> bool {
        self.method == "OPTIONS"
    }

    /// 判断是否为 PJAX 请求
    pub fn is_pjax(&self) -> bool {
        // 检查 X-PJAX 头或 _pjax 参数
        if self.headers.get("x-pjax").map(|v| v == "true").unwrap_or(false) {
            return true;
        }
        // 检查伪装参数
        self.query_params.get(&self.var_pjax).map(|v| v == "1").unwrap_or(false)
    }

    /// 判断是否为手机访问
    pub fn is_mobile(&self) -> bool {
        if let Some(user_agent) = self.headers.get("user-agent") {
            let ua = user_agent.to_lowercase();
            ua.contains("mobile") || ua.contains("android") || ua.contains("iphone") 
                || ua.contains("ipad") || ua.contains("ipod") || ua.contains("windows phone")
        } else {
            false
        }
    }

    /// 判断是否为 CLI 模式
    pub fn is_cli_mode(&self) -> bool {
        self.is_cli
    }

    /// 判断是否为 CGI 模式
    pub fn is_cgi_mode(&self) -> bool {
        self.is_cgi
    }

    // ==================== 请求类型伪装支持 ====================

    /// 检查并处理请求类型伪装
    /// 返回伪装后的请求类型
    pub fn detect_method_spoof(&mut self) -> String {
        // 检查 _method 参数
        if self.method == "POST" {
            if let Some(spoofed_method) = self.form_data.get(&self.var_method) {
                let method = spoofed_method.to_uppercase();
                // 保存原始方法
                self.original_method = Some(self.method.clone());
                // 更新方法
                self.method = method.clone();
                return method;
            }
        }
        self.method.clone()
    }

    /// 检查 AJAX 伪装
    pub fn detect_ajax_spoof(&self) -> bool {
        self.query_params.get(&self.var_ajax).map(|v| v == "1").unwrap_or(false)
            || self.form_data.get(&self.var_ajax).map(|v| v == "1").unwrap_or(false)
    }

    /// 检查 PJAX 伪装
    pub fn detect_pjax_spoof(&self) -> bool {
        self.query_params.get(&self.var_pjax).map(|v| v == "1").unwrap_or(false)
            || self.form_data.get(&self.var_pjax).map(|v| v == "1").unwrap_or(false)
    }

    /// 设置请求类型伪装变量名
    pub fn set_var_method(&mut self, name: String) {
        self.var_method = name;
    }

    /// 设置 AJAX 伪装变量名
    pub fn set_var_ajax(&mut self, name: String) {
        self.var_ajax = name;
    }

    /// 设置 PJAX 伪装变量名
    pub fn set_var_pjax(&mut self, name: String) {
        self.var_pjax = name;
    }

    // ==================== HTTP头信息方法 ====================

    /// 获取所有HTTP头信息
    /// 对应 ThinkPHP 的 header() 方法（无参数时）
    pub fn header_all(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    /// 获取单个HTTP头信息（带默认值）
    pub fn header_default(&self, key: &str, default: &str) -> String {
        self.headers.get(&key.to_lowercase()).cloned().unwrap_or_else(|| default.to_string())
    }

    // ==================== 伪静态支持 ====================

    /// 设置伪静态后缀
    pub fn set_url_suffix(&mut self, suffix: String) {
        self.url_suffix = Some(suffix);
    }

    /// 获取伪静态后缀
    pub fn get_url_suffix(&self) -> Option<&String> {
        self.url_suffix.as_ref()
    }

    // ==================== 参数绑定支持 ====================

    /// 绑定参数到方法
    /// 返回参数值列表，用于方法调用
    pub fn bind_params(&self, param_names: &[&str]) -> Vec<String> {
        param_names.iter()
            .map(|name| {
                // 支持下划线转驼峰
                let camel_name = snake_to_camel(name);
                self.param_value(name)
                    .or_else(|| self.param_value(&camel_name))
                    .unwrap_or_default()
            })
            .collect()
    }

    /// 绑定参数到方法（带默认值）
    pub fn bind_params_with_defaults(&self, param_defaults: HashMap<&str, &str>) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for (name, default) in param_defaults {
            // 支持下划线转驼峰
            let camel_name = snake_to_camel(name);
            let value = self.param_value(name)
                .or_else(|| self.param_value(&camel_name))
                .unwrap_or_else(|| default.to_string());
            result.insert(name.to_string(), value);
        }
        result
    }

    // ==================== 中间件变量传递 ====================

    /// 设置中间件变量
    pub fn set_middleware(&mut self, key: String, value: Value) {
        self.middleware_data.insert(key, value);
    }

    // ==================== 设置器方法 ====================

    /// 设置控制器名
    pub fn set_controller(&mut self, controller: String) {
        self.controller = Some(controller);
    }

    /// 设置操作名
    pub fn set_action(&mut self, action: String) {
        self.action = Some(action);
    }

    /// 设置分级目录
    pub fn set_layer(&mut self, layer: String) {
        self.layer = Some(layer);
    }

    /// 设置路由规则
    pub fn set_route_rule(&mut self, rule: String) {
        self.route_rule = Some(rule);
    }

    /// 设置应用名
    pub fn set_app_name(&mut self, name: String) {
        self.app_name = Some(name);
    }

    /// 设置服务器变量
    pub fn set_server_params(&mut self, params: HashMap<String, String>) {
        self.server_params = params;
    }

    /// 设置环境变量
    pub fn set_env_params(&mut self, params: HashMap<String, String>) {
        self.env_params = params;
    }

    /// 设置Session数据
    pub fn set_session(&mut self, key: String, value: Value) {
        self.session_data.insert(key, value);
    }

    /// 设置PUT数据
    pub fn set_put_data(&mut self, data: HashMap<String, String>) {
        self.put_data = data;
    }

    /// 设置DELETE数据
    pub fn set_delete_data(&mut self, data: HashMap<String, String>) {
        self.delete_data = data;
    }

    /// 设置上传文件
    pub fn set_files(&mut self, files: HashMap<String, Vec<UploadedFile>>) {
        self.files = files;
    }

    /// 设置CLI模式
    pub fn set_cli(&mut self, is_cli: bool) {
        self.is_cli = is_cli;
    }

    /// 设置CGI模式
    pub fn set_cgi(&mut self, is_cgi: bool) {
        self.is_cgi = is_cgi;
    }

    /// 设置服务器端口
    pub fn set_server_port(&mut self, port: u16) {
        self.server_port = Some(port);
    }

    /// 设置远程地址
    pub fn set_remote_addr(&mut self, addr: SocketAddr) {
        self.remote_addr = Some(addr);
    }
}

/// 下划线转驼峰
fn snake_to_camel(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
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