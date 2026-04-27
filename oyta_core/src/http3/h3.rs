//! HTTP/3 协议实现模块
//! 
//! 实现基于 QUIC 的 HTTP/3 协议，包括：
//! - 请求/响应处理
//! - 头部压缩（QPACK）
//! - 流管理
//! - 服务器推送

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::quic::{QuicConnection, QuicStream, StreamType};

/// HTTP/3 请求
/// 表示一个 HTTP/3 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H3Request {
    /// 请求方法
    pub method: String,
    /// 请求路径
    pub path: String,
    /// 请求协议
    pub scheme: String,
    /// 请求主机
    pub authority: String,
    /// 请求头
    pub headers: HashMap<String, String>,
    /// 请求体
    pub body: Vec<u8>,
    /// 请求 ID
    pub request_id: String,
    /// 请求时间
    pub request_time: u64,
    /// 是否包含请求体
    pub has_body: bool,
    /// 内容类型
    pub content_type: Option<String>,
    /// 内容长度
    pub content_length: Option<u64>,
    /// 客户端地址
    pub client_addr: Option<String>,
    /// 协议版本
    pub protocol: String,
}

/// 为 H3Request 实现相关方法
impl H3Request {
    /// 创建新的 HTTP/3 请求
    /// 
    /// # 参数
    /// - `method`: 请求方法
    /// - `path`: 请求路径
    /// 
    /// # 返回
    /// 新的 HTTP/3 请求实例
    pub fn new(method: &str, path: &str) -> Self {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        Self {
            method: method.to_uppercase(),
            path: path.to_string(),
            scheme: String::from("https"),
            authority: String::new(),
            headers: HashMap::new(),
            body: Vec::new(),
            request_id: format!("req-{}", now),
            request_time: now,
            has_body: false,
            content_type: None,
            content_length: None,
            client_addr: None,
            protocol: String::from("HTTP/3"),
        }
    }
    
    /// 添加请求头
    /// 
    /// # 参数
    /// - `name`: 头名称
    /// - `value`: 头值
    pub fn add_header(&mut self, name: &str, value: &str) {
        // 特殊处理一些常用头
        match name.to_lowercase().as_str() {
            "host" | "authority" => {
                self.authority = value.to_string();
            }
            "content-type" => {
                self.content_type = Some(value.to_string());
            }
            "content-length" => {
                self.content_length = value.parse().ok();
            }
            _ => {}
        }
        
        // 存储头
        self.headers.insert(name.to_string(), value.to_string());
    }
    
    /// 获取请求头
    /// 
    /// # 参数
    /// - `name`: 头名称
    /// 
    /// # 返回
    /// 头值（如果存在）
    pub fn get_header(&self, name: &str) -> Option<&String> {
        // 尝试原始名称
        if let Some(value) = self.headers.get(name) {
            return Some(value);
        }
        
        // 尝试小写名称
        self.headers.iter()
            .find(|(k, _)| k.to_lowercase() == name.to_lowercase())
            .map(|(_, v)| v)
    }
    
    /// 设置请求体
    /// 
    /// # 参数
    /// - `body`: 请求体数据
    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = body;
        self.has_body = !self.body.is_empty();
        self.content_length = Some(self.body.len() as u64);
    }
    
    /// 检查是否为 GET 请求
    pub fn is_get(&self) -> bool {
        self.method == "GET"
    }
    
    /// 检查是否为 POST 请求
    pub fn is_post(&self) -> bool {
        self.method == "POST"
    }
    
    /// 检查是否为 PUT 请求
    pub fn is_put(&self) -> bool {
        self.method == "PUT"
    }
    
    /// 检查是否为 DELETE 请求
    pub fn is_delete(&self) -> bool {
        self.method == "DELETE"
    }
}

/// HTTP/3 响应
/// 表示一个 HTTP/3 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H3Response {
    /// 状态码
    pub status: u16,
    /// 响应头
    pub headers: HashMap<String, String>,
    /// 响应体
    pub body: Vec<u8>,
    /// 响应时间
    pub response_time: u64,
    /// 是否包含响应体
    pub has_body: bool,
    /// 内容类型
    pub content_type: Option<String>,
    /// 内容长度
    pub content_length: Option<u64>,
    /// 是否为服务器推送
    pub is_push: bool,
    /// 推送 ID（如果是服务器推送）
    pub push_id: Option<u64>,
}

/// 为 H3Response 实现相关方法
impl H3Response {
    /// 创建新的 HTTP/3 响应
    /// 
    /// # 参数
    /// - `status`: 状态码
    /// 
    /// # 返回
    /// 新的 HTTP/3 响应实例
    pub fn new(status: u16) -> Self {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        Self {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
            response_time: now,
            has_body: false,
            content_type: None,
            content_length: None,
            is_push: false,
            push_id: None,
        }
    }
    
    /// 创建成功响应（200）
    pub fn ok() -> Self {
        Self::new(200)
    }
    
    /// 创建创建响应（201）
    pub fn created() -> Self {
        Self::new(201)
    }
    
    /// 创建无内容响应（204）
    pub fn no_content() -> Self {
        Self::new(204)
    }
    
    /// 创建错误请求响应（400）
    pub fn bad_request() -> Self {
        Self::new(400)
    }
    
    /// 创建未授权响应（401）
    pub fn unauthorized() -> Self {
        Self::new(401)
    }
    
    /// 创建禁止访问响应（403）
    pub fn forbidden() -> Self {
        Self::new(403)
    }
    
    /// 创建未找到响应（404）
    pub fn not_found() -> Self {
        Self::new(404)
    }
    
    /// 创建内部错误响应（500）
    pub fn internal_error() -> Self {
        Self::new(500)
    }
    
    /// 创建服务不可用响应（503）
    pub fn service_unavailable() -> Self {
        Self::new(503)
    }
    
    /// 添加响应头
    /// 
    /// # 参数
    /// - `name`: 头名称
    /// - `value`: 头值
    pub fn add_header(&mut self, name: &str, value: &str) {
        // 特殊处理一些常用头
        match name.to_lowercase().as_str() {
            "content-type" => {
                self.content_type = Some(value.to_string());
            }
            "content-length" => {
                self.content_length = value.parse().ok();
            }
            _ => {}
        }
        
        // 存储头
        self.headers.insert(name.to_string(), value.to_string());
    }
    
    /// 设置响应体
    /// 
    /// # 参数
    /// - `body`: 响应体数据
    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = body;
        self.has_body = !self.body.is_empty();
        self.content_length = Some(self.body.len() as u64);
    }
    
    /// 设置 JSON 响应体
    /// 
    /// # 参数
    /// - `json`: JSON 字符串
    pub fn set_json(&mut self, json: &str) {
        self.add_header("Content-Type", "application/json");
        self.set_body(json.as_bytes().to_vec());
    }
    
    /// 设置文本响应体
    /// 
    /// # 参数
    /// - `text`: 文本内容
    pub fn set_text(&mut self, text: &str) {
        self.add_header("Content-Type", "text/plain; charset=utf-8");
        self.set_body(text.as_bytes().to_vec());
    }
    
    /// 设置 HTML 响应体
    /// 
    /// # 参数
    /// - `html`: HTML 内容
    pub fn set_html(&mut self, html: &str) {
        self.add_header("Content-Type", "text/html; charset=utf-8");
        self.set_body(html.as_bytes().to_vec());
    }
    
    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }
    
    /// 检查是否为重定向
    pub fn is_redirect(&self) -> bool {
        self.status >= 300 && self.status < 400
    }
    
    /// 检查是否为客户端错误
    pub fn is_client_error(&self) -> bool {
        self.status >= 400 && self.status < 500
    }
    
    /// 检查是否为服务器错误
    pub fn is_server_error(&self) -> bool {
        self.status >= 500 && self.status < 600
    }
}

/// HTTP/3 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H3ServerConfig {
    /// 服务器名称
    pub server_name: String,
    /// 监听地址
    pub listen_addr: String,
    /// 监听端口
    pub listen_port: u16,
    /// 是否启用服务器推送
    pub enable_push: bool,
    /// 最大并发流数
    pub max_concurrent_streams: u64,
    /// 最大头部大小
    pub max_header_size: usize,
    /// 最大请求体大小
    pub max_request_body_size: usize,
    /// 请求超时（毫秒）
    pub request_timeout_ms: u64,
    /// 是否启用 QPACK 动态表
    pub enable_qpack_dynamic_table: bool,
    /// QPACK 动态表大小
    pub qpack_dynamic_table_size: usize,
    /// 是否启用 0-RTT
    pub enable_0rtt: bool,
}

/// 为 H3ServerConfig 实现默认值
impl Default for H3ServerConfig {
    fn default() -> Self {
        Self {
            server_name: String::from("OYTAPHP-H3"),
            listen_addr: String::from("0.0.0.0"),
            listen_port: 443,
            enable_push: true,
            max_concurrent_streams: 100,
            max_header_size: 64 * 1024,
            max_request_body_size: 10 * 1024 * 1024,
            request_timeout_ms: 30_000,
            enable_qpack_dynamic_table: true,
            qpack_dynamic_table_size: 4 * 1024,
            enable_0rtt: true,
        }
    }
}

/// HTTP/3 服务器统计信息
#[derive(Debug, Default)]
pub struct H3ServerStats {
    /// 总请求数
    pub total_requests: AtomicU64,
    /// 成功请求数
    pub successful_requests: AtomicU64,
    /// 失败请求数
    pub failed_requests: AtomicU64,
    /// 总响应时间（毫秒）
    pub total_response_time_ms: AtomicU64,
    /// 当前活跃连接数
    pub active_connections: AtomicU64,
    /// 总推送次数
    pub total_pushes: AtomicU64,
    /// 0-RTT 请求数
    pub zero_rtt_requests: AtomicU64,
}

/// 请求处理器类型
pub type RequestHandler = Box<dyn Fn(&H3Request) -> H3Response + Send + Sync>;

/// HTTP/3 服务器
pub struct H3Server {
    /// 服务器配置
    pub config: H3ServerConfig,
    /// 路由映射
    pub routes: RwLock<HashMap<String, Arc<RequestHandler>>>,
    /// 中间件列表
    pub middlewares: RwLock<Vec<Arc<RequestHandler>>>,
    /// 统计信息
    pub stats: H3ServerStats,
    /// 是否正在运行
    pub running: AtomicBool,
    /// 连接管理器
    pub connections: RwLock<Vec<Arc<QuicConnection>>>,
}

/// 为 H3Server 实现相关方法
impl H3Server {
    /// 创建新的 HTTP/3 服务器
    /// 
    /// # 参数
    /// - `config`: 服务器配置
    /// 
    /// # 返回
    /// 新的 HTTP/3 服务器实例
    pub fn new(config: H3ServerConfig) -> Self {
        Self {
            config,
            routes: RwLock::new(HashMap::new()),
            middlewares: RwLock::new(Vec::new()),
            stats: H3ServerStats::default(),
            running: AtomicBool::new(false),
            connections: RwLock::new(Vec::new()),
        }
    }
    
    /// 使用默认配置创建服务器
    pub fn with_defaults() -> Self {
        Self::new(H3ServerConfig::default())
    }
    
    /// 注册路由
    /// 
    /// # 参数
    /// - `method`: HTTP 方法
    /// - `path`: 路径模式
    /// - `handler`: 请求处理器
    pub fn route(&self, method: &str, path: &str, handler: RequestHandler) {
        let key = format!("{}:{}", method.to_uppercase(), path);
        let mut routes = self.routes.write().unwrap();
        routes.insert(key, Arc::new(handler));
    }
    
    /// 注册 GET 路由
    pub fn get(&self, path: &str, handler: RequestHandler) {
        self.route("GET", path, handler);
    }
    
    /// 注册 POST 路由
    pub fn post(&self, path: &str, handler: RequestHandler) {
        self.route("POST", path, handler);
    }
    
    /// 注册 PUT 路由
    pub fn put(&self, path: &str, handler: RequestHandler) {
        self.route("PUT", path, handler);
    }
    
    /// 注册 DELETE 路由
    pub fn delete(&self, path: &str, handler: RequestHandler) {
        self.route("DELETE", path, handler);
    }
    
    /// 添加中间件
    /// 
    /// # 参数
    /// - `middleware`: 中间件处理器
    pub fn use_middleware(&self, middleware: RequestHandler) {
        let mut middlewares = self.middlewares.write().unwrap();
        middlewares.push(Arc::new(middleware));
    }
    
    /// 处理请求
    /// 
    /// # 参数
    /// - `request`: HTTP/3 请求
    /// 
    /// # 返回
    /// HTTP/3 响应
    pub fn handle_request(&self, request: &H3Request) -> H3Response {
        // 更新统计
        self.stats.total_requests.fetch_add(1, Ordering::Relaxed);
        
        // 记录开始时间
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 执行中间件
        let middlewares = self.middlewares.read().unwrap();
        for middleware in middlewares.iter() {
            let response = middleware(request);
            if response.status != 200 {
                // 中间件返回非200，中断处理
                return response;
            }
        }
        
        // 查找路由
        let key = format!("{}:{}", request.method, request.path);
        let routes = self.routes.read().unwrap();
        
        let response = if let Some(handler) = routes.get(&key) {
            handler(request)
        } else {
            // 尝试匹配带通配符的路由
            let mut matched = None;
            for (route_key, handler) in routes.iter() {
                if route_key.ends_with('*') {
                    let prefix = &route_key[..route_key.len() - 1];
                    if key.starts_with(prefix) {
                        matched = Some(handler);
                        break;
                    }
                }
            }
            
            if let Some(handler) = matched {
                handler(request)
            } else {
                H3Response::not_found()
            }
        };
        
        // 更新统计
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64 - start;
        
        self.stats.total_response_time_ms.fetch_add(elapsed, Ordering::Relaxed);
        
        if response.is_success() {
            self.stats.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.stats.failed_requests.fetch_add(1, Ordering::Relaxed);
        }
        
        response
    }
    
    /// 服务器推送
    /// 
    /// # 参数
    /// - `request`: 原始请求
    /// - `push_path`: 推送路径
    /// 
    /// # 返回
    /// 推送响应
    pub fn push(&self, request: &H3Request, push_path: &str) -> Option<H3Response> {
        // 检查是否启用推送
        if !self.config.enable_push {
            return None;
        }
        
        // 创建推送请求
        let push_request = H3Request::new("GET", push_path);
        
        // 处理推送请求
        let response = self.handle_request(&push_request);
        
        // 更新统计
        self.stats.total_pushes.fetch_add(1, Ordering::Relaxed);
        
        Some(response)
    }
    
    /// 启动服务器
    pub fn start(&self) {
        self.running.store(true, Ordering::Relaxed);
    }
    
    /// 停止服务器
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
        
        // 关闭所有连接
        let connections = self.connections.read().unwrap();
        for conn in connections.iter() {
            conn.close();
        }
    }
    
    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
    
    /// 获取统计信息
    pub fn stats(&self) -> &H3ServerStats {
        &self.stats
    }
    
    /// 添加连接
    pub fn add_connection(&self, conn: Arc<QuicConnection>) {
        let mut connections = self.connections.write().unwrap();
        connections.push(conn);
        self.stats.active_connections.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 移除连接
    pub fn remove_connection(&self, conn_id: &super::quic::ConnectionId) {
        let mut connections = self.connections.write().unwrap();
        let len_before = connections.len();
        connections.retain(|c| &c.connection_id != conn_id);
        
        if connections.len() != len_before {
            self.stats.active_connections.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

/// 为 H3Server 实现 Default trait
impl Default for H3Server {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// 单元测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试 HTTP/3 请求创建
    #[test]
    fn test_h3_request() {
        let request = H3Request::new("GET", "/api/test");
        
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/api/test");
        assert!(request.is_get());
    }
    
    /// 测试 HTTP/3 响应创建
    #[test]
    fn test_h3_response() {
        let mut response = H3Response::ok();
        
        assert_eq!(response.status, 200);
        assert!(response.is_success());
        
        response.set_json(r#"{"status":"ok"}"#);
        assert!(response.has_body);
    }
    
    /// 测试路由注册
    #[test]
    fn test_route() {
        let server = H3Server::with_defaults();
        
        server.get("/api/test", Box::new(|_| {
            H3Response::ok()
        }));
        
        let request = H3Request::new("GET", "/api/test");
        let response = server.handle_request(&request);
        
        assert_eq!(response.status, 200);
    }
    
    /// 测试 404 响应
    #[test]
    fn test_not_found() {
        let server = H3Server::with_defaults();
        
        let request = H3Request::new("GET", "/nonexistent");
        let response = server.handle_request(&request);
        
        assert_eq!(response.status, 404);
    }
}
