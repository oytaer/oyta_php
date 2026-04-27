//! HTTP/3 和 QUIC 支持模块
//! 
//! 提供基于 QUIC 协议的 HTTP/3 实现，包括：
//! - QUIC 连接管理
//! - HTTP/3 请求/响应处理
//! - 流管理
//! - 0-RTT 支持
//! - 连接迁移

// 引入子模块
pub mod quic;
pub mod h3;
pub mod stream;
pub mod connection;

// 重导出主要类型
pub use quic::{QuicConfig, QuicConnection, QuicStream};
pub use h3::{H3Request, H3Response, H3Server};
pub use stream::{StreamId, StreamType, StreamState};
pub use connection::{ConnectionManager, ConnectionStats};
