//! 中间件模块
//!
//! 提供 OYTAPHP 的中间件功能
//! 支持洋葱模型、全局/路由/控制器中间件
//! 内置 SessionInit、CORS、LoadLangPack、RequestCache 等中间件

pub mod builtin;
pub mod dispatcher;
pub mod types;
pub mod facade;

pub use facade::Middleware;
