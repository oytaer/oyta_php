//! 事件系统模块
//!
//! 提供 OYTAPHP 的事件监听和触发功能
//! 对应 ThinkPHP 8.0 的 Event/Listen

pub mod dispatcher;
pub mod types;
pub mod facade;

pub use facade::Event;
