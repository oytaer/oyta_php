//! OYTAPHP 核心运行时入口文件
//!
//! 这是 oyta 二进制的入口点，负责：
//! 1. 解析命令行参数
//! 2. 初始化日志系统
//! 3. 检测项目根目录
//! 4. 加载环境变量
//! 5. 分发到对应的命令处理器

use anyhow::Result;

// 引入各模块
mod cache;
mod cli;
mod cluster;
mod composer;
mod config;
mod container;
mod coroutine;
mod database;
mod debug;
mod embedded;
mod env_loader;
mod event;
mod fastcgi;
mod helpers;
mod http;
mod http3;
mod i18n;
mod interpreter;
mod logging;
mod middleware;
mod microservice;
mod monitor;
mod parser;
mod project;
mod router;
mod sandbox;
mod search;
mod security;
mod serialize;
mod session;
mod simd;
mod symbol_table;
mod template;
mod timer;
mod validator;
mod watcher;

/// 程序入口函数
/// 使用 tokio 异步运行时，因为后续的 HTTP 服务、热重载等功能都需要异步支持
#[tokio::main]
async fn main() -> Result<()> {
    // 第一步：解析命令行参数
    // 使用 clap 解析，如果用户输入了无效命令会自动显示帮助信息并退出
    let args = cli::args::parse();

    // 第二步：初始化日志系统
    // 根据是否为 run 命令来决定日志级别
    // run 命令默认 info 级别，其他命令默认 warn 级别
    let debug_mode = match &args.command {
        Some(cli::args::Commands::Run { debug, .. }) => *debug,
        _ => false,
    };
    logging::setup::init(debug_mode)?;

    // 第三步：记录启动信息
    // 输出 OYTAPHP 版本和启动命令
    tracing::info!("OYTAPHP v{}", env!("CARGO_PKG_VERSION"));

    // 第四步：分发到对应的命令处理器
    // 如果没有指定子命令，显示帮助信息
    match &args.command {
        Some(command) => {
            // 记录用户执行的命令
            tracing::debug!("执行命令: {}", cli::args::command_description(command));
            // 分发到命令处理器
            cli::dispatch::dispatch(command).await?;
        }
        None => {
            // 未指定命令时，显示帮助信息
            // 使用 clap 的 print_help 方法
            use clap::CommandFactory;
            cli::args::Cli::command().print_help()?;
        }
    }

    Ok(())
}
