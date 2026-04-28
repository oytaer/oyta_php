//! 路由系统模块
//!
//! 提供 OYTAPHP 的路由管理功能
//! 支持自动路由、定义路由、资源路由、路由组、MISS路由、注解路由等
//! 使用 matchit 实现高性能路由匹配

pub mod annotation;
pub mod cache;
pub mod facade;
pub mod registry;
pub mod route_parser;
pub mod types;

// 导出类型定义
pub use types::{HttpMethod, Route, RouteGroup, RouteHandler, RouteMatch};
// 导出注册表
pub use registry::RouteRegistry;
// 导出门面（静态方法访问器）
pub use facade::RouteFacade;
