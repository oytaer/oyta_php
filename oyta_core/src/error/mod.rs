//! 错误处理系统模块入口
//!
//! 提供完整的 PHP 错误处理功能
//! 包括：错误处理器、异常处理器、关闭函数、错误级别等
//!
//! # 模块结构
//! - `handler`: 错误和异常处理器
//! - `level`: 错误级别定义
//! - `reporter`: 错误报告器
//! - `shutdown`: 关闭函数注册
//! - `types`: 错误类型定义

pub mod handler;
pub mod level;
pub mod reporter;
pub mod shutdown;
pub mod types;

// 重新导出主要类型
pub use handler::ErrorHandler;
pub use level::ErrorLevel;
pub use reporter::ErrorReporter;
pub use shutdown::ShutdownManager;
pub use types::{ErrorInfo, ExceptionInfo};
