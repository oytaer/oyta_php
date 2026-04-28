//! 服务管理模块
//!
//! 提供完整的服务生命周期管理，支持：
//! - 注解驱动服务声明
//! - 独立配置文件
//! - 按需启动（惰性初始化）
//! - 服务依赖管理
//!
//! # 模块结构
//! - `traits`: 服务 trait 定义
//! - `registry`: 服务注册表
//! - `config`: 服务配置加载器
//! - `annotation`: 注解解析器
//! - `bootstrap`: 服务启动管理器
//!
//! # 使用示例
//! ```php
//! // 方式1：注解声明
//! #[Service(name: 'websocket', enable: true, config: ['port' => 9501])]
//! class WebSocketService { }
//!
//! // 方式2：配置文件 (config/websocket.php)
//! return [
//!     'enable' => true,
//!     'port' => 9501,
//! ];
//!
//! // 方式3：门面调用（惰性初始化）
//! Cache::set('key', 'value');  // 自动启动缓存服务
//! ```

// 子模块声明
pub mod annotation;
pub mod bootstrap;
pub mod config;
pub mod registry;
pub mod traits;

// 公开类型重导出
pub use annotation::{
    Annotation, AnnotationParser, AnnotationParams, AnnotationType, QueueJobAnnotation,
    ServiceAnnotation, TimerTaskAnnotation, WebSocketHandlerAnnotation,
};
pub use bootstrap::ServiceBootstrap;
pub use config::{
    CacheConfig, FileCacheConfig, QueueConfig, RedisConfig, ServiceConfigLoader,
    TimerConfig, WebSocketConfig,
};
pub use registry::ServiceRegistry;
pub use traits::{
    BackgroundService, Service, ServiceConfig, ServiceHandle, ServiceMetadata, ServiceState,
};

// 全局访问函数
pub use bootstrap::get_service_bootstrap;
pub use registry::get_service_registry;
