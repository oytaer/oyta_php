//! FastCGI 协议支持模块
//!
//! 实现 FastCGI 协议，让 OYTAPHP 可以像 PHP-FPM 一样被 Nginx 调用
//! 这样用户可以像配置 PHP 网站一样简单部署

pub mod protocol;
pub mod server;
pub mod handler;

pub use server::FastCGIServer;
pub use handler::FastCGIHandler;
