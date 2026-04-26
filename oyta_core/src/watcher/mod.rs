//! 文件监听与热重载模块
//!
//! 使用 notify 库监听项目目录的文件变化
//! 当 PHP 文件发生变化时，自动重新解析并更新符号表

pub mod hot_reload;
pub mod watcher;
