//! 应用共享状态模块
//!
//! 定义 Axum 的共享状态，在 HTTP 处理器中可访问
//! 包含符号表注册表、路由注册表、中间件调度器、项目根目录等
//! 使用 Arc 实现多线程安全共享

use std::sync::Arc;

use crate::middleware::dispatcher::MiddlewareDispatcher;
use crate::router::registry::RouteRegistry;
use crate::symbol_table::registry::SymbolRegistry;

/// 应用共享状态
/// 通过 Axum 的 State 机制在请求处理器之间共享
/// 所有字段都是 Arc 包装，确保线程安全
#[derive(Debug, Clone)]
pub struct AppState {
    /// 符号表注册表
    pub registry: Arc<SymbolRegistry>,

    /// 路由注册表
    pub route_registry: Arc<RouteRegistry>,

    /// 中间件调度器
    pub middleware: Arc<std::sync::RwLock<MiddlewareDispatcher>>,

    /// 项目根目录
    pub project_root: Arc<std::path::PathBuf>,

    /// 是否调试模式
    pub debug: bool,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(
        registry: Arc<SymbolRegistry>,
        route_registry: Arc<RouteRegistry>,
        middleware: MiddlewareDispatcher,
        project_root: std::path::PathBuf,
        debug: bool,
    ) -> Self {
        Self {
            registry,
            route_registry,
            middleware: Arc::new(std::sync::RwLock::new(middleware)),
            project_root: Arc::new(project_root),
            debug,
        }
    }
}
