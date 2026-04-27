//! 微服务框架模块
//! 
//! 提供完整的微服务架构支持，包括：
//! - 服务注册与发现
//! - 负载均衡
//! - 熔断器
//! - 服务网关
//! - 配置中心

// 引入子模块
pub mod registry;
pub mod loadbalancer;
pub mod circuit_breaker;
pub mod gateway;
pub mod config_center;

// 重导出主要类型
pub use registry::{ServiceRegistry, ServiceInstance, ServiceHealth};
pub use loadbalancer::{LoadBalancer, LoadBalanceStrategy, BackendServer};
pub use circuit_breaker::{CircuitBreaker, CircuitState, CircuitStats};
pub use gateway::{ApiGateway, RouteConfig, GatewayMiddleware};
pub use config_center::{ConfigCenter, ConfigEntry, ConfigWatcher};
