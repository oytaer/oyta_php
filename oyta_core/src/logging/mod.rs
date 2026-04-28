//! 日志系统模块
//!
//! 只负责声明子模块，不包含任何功能实现代码

pub mod setup;
pub mod file_writer;
pub mod channel;
pub mod facade;

pub use facade::Log;
