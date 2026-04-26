//! 日志系统初始化模块
//!
//! 负责初始化 tracing 日志系统
//! 使用 tracing-subscriber 实现结构化日志输出
//! 支持以下特性：
//! - 根据调试模式自动调整日志级别
//! - 彩色输出（终端环境下）
//! - 环境变量过滤（RUST_LOG 环境变量）
//! - 文件日志输出（按日期分割）
//! - 多通道日志（default/error/sql）

use anyhow::Result;
use std::path::Path;
use tracing_subscriber::{
    fmt,
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use super::file_writer::FileWriter;

/// 初始化日志系统（仅终端输出）
pub fn init(debug_mode: bool) -> Result<()> {
    let default_level = if debug_mode { "debug" } else { "info" };
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_level));
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_file(debug_mode)
        .with_line_number(debug_mode)
        .with_timer(fmt::time::time())
        .with_ansi(true);
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()?;
    Ok(())
}

/// 初始化日志系统（终端 + 文件输出）
///
/// # 参数
/// - `debug_mode`: 是否为调试模式
/// - `log_dir`: 日志文件目录
pub fn init_with_file(debug_mode: bool, log_dir: &Path) -> Result<()> {
    let default_level = if debug_mode { "debug" } else { "info" };
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_level));

    // 终端输出层
    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_file(debug_mode)
        .with_line_number(debug_mode)
        .with_timer(fmt::time::time())
        .with_ansi(true);

    // 文件输出层
    let file_writer = FileWriter::new(log_dir, "oyta")
        .with_max_files(30);
    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_target(true)
        .with_file(false)
        .with_line_number(false)
        .with_timer(fmt::time::time())
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .try_init()?;

    // 初始化通道日志管理器
    super::channel::init_global(log_dir);

    Ok(())
}
