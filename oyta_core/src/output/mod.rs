//! 输出缓冲控制模块入口
//!
//! 提供完整的 PHP 输出缓冲控制功能
//! 包括：ob_start, ob_get_contents, ob_end_flush 等
//!
//! # 模块结构
//! - `buffer`: 输出缓冲栈管理
//! - `callback`: 缓冲回调处理
//! - `handler`: 处理器标志定义
//! - `types`: 类型定义

pub mod buffer;
pub mod callback;
pub mod handler;
pub mod types;

// 重新导出主要类型
pub use buffer::OutputBuffer;
pub use callback::OutputCallback;
pub use handler::OutputHandlerFlags;
pub use types::{BufferState, OutputHandler};
