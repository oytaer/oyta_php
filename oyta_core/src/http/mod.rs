//! HTTP 服务模块
//!
//! 提供 OYTAPHP 的 HTTP 服务功能
//! 包括请求处理、响应构建、服务器启动、应用状态等

pub mod facade;
pub mod handler;
pub mod request;
pub mod response;
pub mod server;
pub mod state;

// 导出 Request 门面
pub use facade::RequestFacade;
