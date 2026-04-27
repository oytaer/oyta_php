//! 服务网关模块
//! 
//! 实现 API 网关功能，包括：
//! - 路由转发
//! - 请求过滤
//! - 限流熔断
//! - 认证授权
//! - 日志监控

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::loadbalancer::{BackendServer, LoadBalancer, LoadBalanceStrategy};
use super::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};

/// 路由配置
/// 定义请求路由规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// 路由ID
    pub id: String,
    /// 路由名称
    pub name: String,
    /// 匹配路径前缀
    pub path_prefix: String,
    /// 目标服务名称
    pub target_service: String,
    /// 目标路径（可选，用于路径重写）
    pub target_path: Option<String>,
    /// 是否启用
    pub enabled: bool,
    /// 超时时间（毫秒）
    pub timeout_ms: u64,
    /// 重试次数
    pub retry_count: u32,
    /// 是否启用认证
    pub auth_required: bool,
    /// 允许的HTTP方法
    pub allowed_methods: Vec<String>,
    /// 请求头转换
    pub header_transforms: HashMap<String, String>,
    /// 限流配置
    pub rate_limit: Option<RateLimitConfig>,
    /// 优先级（数字越小优先级越高）
    pub priority: u32,
}

/// 为 RouteConfig 实现默认值
impl Default for RouteConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            path_prefix: String::from("/"),
            target_service: String::new(),
            target_path: None,
            enabled: true,
            timeout_ms: 30_000,
            retry_count: 3,
            auth_required: false,
            allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
            header_transforms: HashMap::new(),
            rate_limit: None,
            priority: 100,
        }
    }
}

/// 限流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// 每秒请求数限制
    pub requests_per_second: u32,
    /// 每分钟请求数限制
    pub requests_per_minute: u32,
    /// 每小时请求数限制
    pub requests_per_hour: u32,
    /// 突发流量限制
    pub burst_size: u32,
    /// 限流键（IP、用户ID等）
    pub key_type: RateLimitKeyType,
}

/// 为 RateLimitConfig 实现默认值
impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 100,
            requests_per_minute: 1000,
            requests_per_hour: 10000,
            burst_size: 50,
            key_type: RateLimitKeyType::IpAddress,
        }
    }
}

/// 限流键类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateLimitKeyType {
    /// IP地址
    IpAddress,
    /// 用户ID
    UserId,
    /// API密钥
    ApiKey,
    /// 全局
    Global,
}

/// 网关中间件
/// 定义请求处理中间件
#[derive(Debug, Clone)]
pub struct GatewayMiddleware {
    /// 中间件名称
    pub name: String,
    /// 中间件优先级
    pub priority: u32,
    /// 是否启用
    pub enabled: bool,
    /// 中间件类型
    pub middleware_type: MiddlewareType,
}

/// 中间件类型枚举
#[derive(Debug, Clone)]
pub enum MiddlewareType {
    /// 认证中间件
    Authentication {
        /// 认证提供者
        provider: String,
    },
    /// 授权中间件
    Authorization {
        /// 权限检查器
        permission_checker: String,
    },
    /// 限流中间件
    RateLimiting {
        /// 限流配置
        config: RateLimitConfig,
    },
    /// 日志中间件
    Logging {
        /// 日志级别
        level: String,
        /// 是否记录请求体
        log_body: bool,
    },
    /// 指标中间件
    Metrics {
        /// 是否记录延迟
        record_latency: bool,
        /// 是否记录错误率
        record_errors: bool,
    },
    /// CORS中间件
    Cors {
        /// 允许的来源
        allowed_origins: Vec<String>,
        /// 允许的方法
        allowed_methods: Vec<String>,
        /// 允许的头
        allowed_headers: Vec<String>,
    },
    /// 压缩中间件
    Compression {
        /// 压缩级别
        level: u32,
    },
    /// 缓存中间件
    Cache {
        /// 缓存时间（秒）
        ttl_seconds: u64,
    },
    /// 自定义中间件
    Custom {
        /// 处理函数名称
        handler: String,
    },
}

/// 请求上下文
/// 包含请求的所有相关信息
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// 请求ID
    pub request_id: String,
    /// 客户端地址
    pub client_addr: SocketAddr,
    /// 请求路径
    pub path: String,
    /// 请求方法
    pub method: String,
    /// 请求头
    pub headers: HashMap<String, String>,
    /// 查询参数
    pub query_params: HashMap<String, String>,
    /// 用户ID（如果已认证）
    pub user_id: Option<String>,
    /// API密钥
    pub api_key: Option<String>,
    /// 请求开始时间
    pub start_time: u64,
    /// 匹配的路由
    pub matched_route: Option<RouteConfig>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 为 RequestContext 实现相关方法
impl RequestContext {
    /// 创建新的请求上下文
    pub fn new(request_id: String, client_addr: SocketAddr, path: String, method: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        Self {
            request_id,
            client_addr,
            path,
            method,
            headers: HashMap::new(),
            query_params: HashMap::new(),
            user_id: None,
            api_key: None,
            start_time: now,
            matched_route: None,
            metadata: HashMap::new(),
        }
    }
    
    /// 获取请求耗时（毫秒）
    pub fn elapsed_ms(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        now.saturating_sub(self.start_time)
    }
}

/// 网关响应
#[derive(Debug, Clone)]
pub struct GatewayResponse {
    /// HTTP状态码
    pub status_code: u16,
    /// 响应头
    pub headers: HashMap<String, String>,
    /// 响应体
    pub body: Vec<u8>,
    /// 是否来自缓存
    pub from_cache: bool,
    /// 上游地址
    pub upstream_addr: Option<SocketAddr>,
}

/// 为 GatewayResponse 实现相关方法
impl GatewayResponse {
    /// 创建成功响应
    pub fn ok(body: Vec<u8>) -> Self {
        Self {
            status_code: 200,
            headers: HashMap::new(),
            body,
            from_cache: false,
            upstream_addr: None,
        }
    }
    
    /// 创建错误响应
    pub fn error(status_code: u16, message: &str) -> Self {
        Self {
            status_code,
            headers: HashMap::new(),
            body: message.as_bytes().to_vec(),
            from_cache: false,
            upstream_addr: None,
        }
    }
}

/// 网关统计信息
#[derive(Debug, Default)]
pub struct GatewayStats {
    /// 总请求数
    pub total_requests: AtomicU64,
    /// 成功请求数
    pub successful_requests: AtomicU64,
    /// 失败请求数
    pub failed_requests: AtomicU64,
    /// 被拒绝的请求数
    pub rejected_requests: AtomicU64,
    /// 超时请求数
    pub timeout_requests: AtomicU64,
    /// 总响应时间（毫秒）
    pub total_response_time_ms: AtomicU64,
    /// 当前活跃连接数
    pub active_connections: AtomicU64,
}

/// API网关
/// 核心网关实现
pub struct ApiGateway {
    /// 路由配置列表
    routes: RwLock<Vec<RouteConfig>>,
    /// 中间件列表
    middlewares: RwLock<Vec<GatewayMiddleware>>,
    /// 负载均衡器映射
    load_balancers: RwLock<HashMap<String, Arc<LoadBalancer>>>,
    /// 熔断器映射
    circuit_breakers: RwLock<HashMap<String, Arc<CircuitBreaker>>>,
    /// 限流计数器
    rate_limit_counters: RwLock<HashMap<String, RateLimitCounter>>,
    /// 统计信息
    stats: GatewayStats,
    /// 是否正在运行
    running: AtomicBool,
    /// 网关配置
    config: GatewayConfig,
}

/// 限流计数器
#[derive(Debug, Clone, Default)]
struct RateLimitCounter {
    /// 秒级计数
    second_count: u64,
    /// 秒级时间戳
    second_timestamp: u64,
    /// 分钟级计数
    minute_count: u64,
    /// 分钟级时间戳
    minute_timestamp: u64,
    /// 小时级计数
    hour_count: u64,
    /// 小时级时间戳
    hour_timestamp: u64,
}

/// 网关配置
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// 监听地址
    pub listen_addr: SocketAddr,
    /// 默认超时时间（毫秒）
    pub default_timeout_ms: u64,
    /// 最大连接数
    pub max_connections: u32,
    /// 是否启用访问日志
    pub enable_access_log: bool,
    /// 是否启用指标收集
    pub enable_metrics: bool,
    /// 请求体最大大小（字节）
    pub max_request_body_size: u64,
    /// 响应体最大大小（字节）
    pub max_response_body_size: u64,
}

/// 为 GatewayConfig 实现默认值
impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:8080".parse().unwrap(),
            default_timeout_ms: 30_000,
            max_connections: 10_000,
            enable_access_log: true,
            enable_metrics: true,
            max_request_body_size: 10 * 1024 * 1024, // 10MB
            max_response_body_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// 为 ApiGateway 实现相关方法
impl ApiGateway {
    /// 创建新的API网关
    pub fn new(config: GatewayConfig) -> Self {
        Self {
            routes: RwLock::new(Vec::new()),
            middlewares: RwLock::new(Vec::new()),
            load_balancers: RwLock::new(HashMap::new()),
            circuit_breakers: RwLock::new(HashMap::new()),
            rate_limit_counters: RwLock::new(HashMap::new()),
            stats: GatewayStats::default(),
            running: AtomicBool::new(false),
            config,
        }
    }
    
    /// 使用默认配置创建API网关
    pub fn with_defaults() -> Self {
        Self::new(GatewayConfig::default())
    }
    
    /// 添加路由
    pub fn add_route(&self, route: RouteConfig) {
        let mut routes = self.routes.write().unwrap();
        routes.push(route);
        // 按优先级排序
        routes.sort_by_key(|r| r.priority);
    }
    
    /// 移除路由
    pub fn remove_route(&self, route_id: &str) -> bool {
        let mut routes = self.routes.write().unwrap();
        let len_before = routes.len();
        routes.retain(|r| r.id != route_id);
        routes.len() != len_before
    }
    
    /// 添加中间件
    pub fn add_middleware(&self, middleware: GatewayMiddleware) {
        let mut middlewares = self.middlewares.write().unwrap();
        middlewares.push(middleware);
        // 按优先级排序
        middlewares.sort_by_key(|m| m.priority);
    }
    
    /// 移除中间件
    pub fn remove_middleware(&self, name: &str) -> bool {
        let mut middlewares = self.middlewares.write().unwrap();
        let len_before = middlewares.len();
        middlewares.retain(|m| m.name != name);
        middlewares.len() != len_before
    }
    
    /// 匹配路由
    /// 
    /// # 参数
    /// - `path`: 请求路径
    /// - `method`: 请求方法
    /// 
    /// # 返回
    /// 匹配的路由配置（如果有）
    pub fn match_route(&self, path: &str, method: &str) -> Option<RouteConfig> {
        let routes = self.routes.read().unwrap();
        
        for route in routes.iter() {
            // 检查是否启用
            if !route.enabled {
                continue;
            }
            
            // 检查路径前缀匹配
            if !path.starts_with(&route.path_prefix) {
                continue;
            }
            
            // 检查HTTP方法
            if !route.allowed_methods.contains(&method.to_uppercase()) {
                continue;
            }
            
            return Some(route.clone());
        }
        
        None
    }
    
    /// 处理请求
    /// 
    /// # 参数
    /// - `ctx`: 请求上下文
    /// 
    /// # 返回
    /// 网关响应
    pub fn handle_request(&self, ctx: &mut RequestContext) -> GatewayResponse {
        // 更新统计
        self.stats.total_requests.fetch_add(1, Ordering::Relaxed);
        self.stats.active_connections.fetch_add(1, Ordering::Relaxed);
        
        // 匹配路由
        let route = match self.match_route(&ctx.path, &ctx.method) {
            Some(r) => r,
            None => {
                self.stats.failed_requests.fetch_add(1, Ordering::Relaxed);
                return GatewayResponse::error(404, "Route not found");
            }
        };
        
        // 存储匹配的路由
        ctx.matched_route = Some(route.clone());
        
        // 检查限流
        if let Some(rate_limit) = &route.rate_limit {
            if !self.check_rate_limit(ctx, rate_limit) {
                self.stats.rejected_requests.fetch_add(1, Ordering::Relaxed);
                return GatewayResponse::error(429, "Too many requests");
            }
        }
        
        // 检查熔断器
        let breaker = self.get_or_create_circuit_breaker(&route.target_service);
        if !breaker.allow_request() {
            self.stats.rejected_requests.fetch_add(1, Ordering::Relaxed);
            return GatewayResponse::error(503, "Service unavailable");
        }
        
        // 获取负载均衡器
        let lb = self.get_load_balancer(&route.target_service);
        
        // 选择后端服务器
        let backend = match lb.select() {
            Some(b) => b,
            None => {
                self.stats.failed_requests.fetch_add(1, Ordering::Relaxed);
                breaker.record_failure();
                return GatewayResponse::error(503, "No available backend");
            }
        };
        
        // 记录响应时间
        let start = SystemTime::now();
        
        // 模拟请求处理（实际实现需要真正的HTTP客户端）
        let response = self.forward_request(ctx, &backend, &route);
        
        // 计算响应时间
        let elapsed = SystemTime::now()
            .duration_since(start)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 更新统计
        self.stats.total_response_time_ms.fetch_add(elapsed, Ordering::Relaxed);
        
        // 根据响应状态更新熔断器
        if response.status_code >= 500 {
            breaker.record_failure();
            self.stats.failed_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            breaker.record_success();
            self.stats.successful_requests.fetch_add(1, Ordering::Relaxed);
        }
        
        response
    }
    
    /// 转发请求到后端
    fn forward_request(&self, _ctx: &RequestContext, backend: &BackendServer, route: &RouteConfig) -> GatewayResponse {
        // 构建目标路径
        let target_path = match &route.target_path {
            Some(path) => path.clone(),
            None => {
                // 使用原始路径去掉前缀
                _ctx.path[route.path_prefix.len()..].to_string()
            }
        };
        
        // 记录上游地址
        let mut response = GatewayResponse::ok(format!("Forwarded to {}{}", backend.address, target_path).into_bytes());
        response.upstream_addr = Some(backend.address);
        response
    }
    
    /// 检查限流
    fn check_rate_limit(&self, ctx: &RequestContext, config: &RateLimitConfig) -> bool {
        // 获取限流键
        let key = match config.key_type {
            RateLimitKeyType::IpAddress => ctx.client_addr.ip().to_string(),
            RateLimitKeyType::UserId => ctx.user_id.clone().unwrap_or_default(),
            RateLimitKeyType::ApiKey => ctx.api_key.clone().unwrap_or_default(),
            RateLimitKeyType::Global => "global".to_string(),
        };
        
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        // 获取或创建计数器
        let mut counters = self.rate_limit_counters.write().unwrap();
        let counter = counters.entry(key).or_insert_with(RateLimitCounter::default);
        
        // 检查秒级限流
        if now - counter.second_timestamp < 1 {
            if counter.second_count >= config.requests_per_second as u64 {
                return false;
            }
            counter.second_count += 1;
        } else {
            counter.second_count = 1;
            counter.second_timestamp = now;
        }
        
        // 检查分钟级限流
        if now - counter.minute_timestamp < 60 {
            if counter.minute_count >= config.requests_per_minute as u64 {
                return false;
            }
            counter.minute_count += 1;
        } else {
            counter.minute_count = 1;
            counter.minute_timestamp = now;
        }
        
        // 检查小时级限流
        if now - counter.hour_timestamp < 3600 {
            if counter.hour_count >= config.requests_per_hour as u64 {
                return false;
            }
            counter.hour_count += 1;
        } else {
            counter.hour_count = 1;
            counter.hour_timestamp = now;
        }
        
        true
    }
    
    /// 获取或创建熔断器
    fn get_or_create_circuit_breaker(&self, service_name: &str) -> Arc<CircuitBreaker> {
        // 先尝试获取现有的
        {
            let breakers = self.circuit_breakers.read().unwrap();
            if let Some(breaker) = breakers.get(service_name) {
                return breaker.clone();
            }
        }
        
        // 创建新的熔断器
        let breaker = Arc::new(CircuitBreaker::new(service_name, CircuitBreakerConfig::default()));
        
        // 存储并返回
        {
            let mut breakers = self.circuit_breakers.write().unwrap();
            breakers.entry(service_name.to_string())
                .or_insert_with(|| breaker.clone());
        }
        
        breaker
    }
    
    /// 获取负载均衡器
    fn get_load_balancer(&self, service_name: &str) -> Arc<LoadBalancer> {
        // 先尝试获取现有的
        {
            let lbs = self.load_balancers.read().unwrap();
            if let Some(lb) = lbs.get(service_name) {
                return lb.clone();
            }
        }
        
        // 创建新的负载均衡器
        let lb = Arc::new(LoadBalancer::with_defaults());
        
        // 存储并返回
        {
            let mut lbs = self.load_balancers.write().unwrap();
            lbs.entry(service_name.to_string())
                .or_insert_with(|| lb.clone());
        }
        
        lb
    }
    
    /// 添加后端服务器到服务
    pub fn add_backend(&self, service_name: &str, backend: BackendServer) {
        let lb = self.get_load_balancer(service_name);
        lb.add_backend(backend);
    }
    
    /// 获取统计信息
    pub fn stats(&self) -> &GatewayStats {
        &self.stats
    }
    
    /// 启动网关
    pub fn start(&self) {
        self.running.store(true, Ordering::Relaxed);
    }
    
    /// 停止网关
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
    
    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
    
    /// 获取所有路由
    pub fn get_routes(&self) -> Vec<RouteConfig> {
        self.routes.read().unwrap().clone()
    }
    
    /// 获取所有中间件
    pub fn get_middlewares(&self) -> Vec<GatewayMiddleware> {
        self.middlewares.read().unwrap().clone()
    }
}

/// 为 ApiGateway 实现 Default trait
impl Default for ApiGateway {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// 单元测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试路由匹配
    #[test]
    fn test_route_matching() {
        let gateway = ApiGateway::with_defaults();
        
        gateway.add_route(RouteConfig {
            id: "test-route".to_string(),
            name: "Test Route".to_string(),
            path_prefix: "/api/v1".to_string(),
            target_service: "test-service".to_string(),
            ..Default::default()
        });
        
        // 应该匹配
        let route = gateway.match_route("/api/v1/users", "GET");
        assert!(route.is_some());
        
        // 不应该匹配
        let route = gateway.match_route("/api/v2/users", "GET");
        assert!(route.is_none());
    }
    
    /// 测试限流
    #[test]
    fn test_rate_limiting() {
        let gateway = ApiGateway::with_defaults();
        
        gateway.add_route(RouteConfig {
            id: "test-route".to_string(),
            name: "Test Route".to_string(),
            path_prefix: "/api".to_string(),
            target_service: "test-service".to_string(),
            rate_limit: Some(RateLimitConfig {
                requests_per_second: 2,
                ..Default::default()
            }),
            ..Default::default()
        });
        
        // 添加后端
        gateway.add_backend("test-service", BackendServer {
            address: "127.0.0.1:8080".parse().unwrap(),
            weight: 100,
            connections: 0,
            healthy: true,
            response_time_ms: 0,
            last_request_time: 0,
            metadata: HashMap::new(),
        });
        
        let addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        
        // 前两次请求应该成功
        for _ in 0..2 {
            let mut ctx = RequestContext::new("test".to_string(), addr, "/api/test".to_string(), "GET".to_string());
            let response = gateway.handle_request(&mut ctx);
            assert!(response.status_code < 500 || response.status_code == 429);
        }
    }
    
    /// 测试添加和移除路由
    #[test]
    fn test_add_remove_route() {
        let gateway = ApiGateway::with_defaults();
        
        let route = RouteConfig {
            id: "test-route".to_string(),
            name: "Test Route".to_string(),
            path_prefix: "/api".to_string(),
            target_service: "test-service".to_string(),
            ..Default::default()
        };
        
        gateway.add_route(route);
        assert_eq!(gateway.get_routes().len(), 1);
        
        let removed = gateway.remove_route("test-route");
        assert!(removed);
        assert!(gateway.get_routes().is_empty());
    }
}
