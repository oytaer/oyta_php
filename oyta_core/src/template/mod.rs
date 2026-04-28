//! 模板引擎模块
//!
//! 提供 OYTAPHP 的模板渲染功能
//! 支持：OYTAPHP 原生模板、Tera 模板引擎
//! 通过配置切换模板引擎类型

pub mod engine;
pub mod facade;
pub mod tera_engine;

// 导出 View 门面
pub use facade::View;
