//! 命令分发模块
//!
//! 负责将解析后的命令分发到对应的处理器
//! 每个命令都有独立的处理函数，确保职责清晰
//!
//! # 模块结构
//! - `server`: 服务器相关命令（run, fastcgi）
//! - `build`: 构建命令
//! - `make`: 所有 make:* 命令
//! - `route`: 路由相关命令
//! - `optimize`: 优化相关命令
//! - `composer`: Composer 相关命令
//! - `service`: 服务发现相关命令
//! - `info`: 版本和列表命令
//! - `helpers`: 辅助函数
//! - `types`: 类型定义

pub mod build;
pub mod composer;
pub mod helpers;
pub mod info;
pub mod make;
pub mod optimize;
pub mod route;
pub mod server;
pub mod service;
pub mod types;
pub mod version_utils;

use anyhow::Result;

use super::args::Commands;
use build::handle_build;
use composer::{
    handle_composer_clear_cache, handle_composer_config, handle_composer_diagnose,
    handle_composer_dump_autoload, handle_composer_install, handle_composer_outdated,
    handle_composer_remove, handle_composer_require, handle_composer_show,
    handle_composer_update, handle_composer_validate,
};
use info::{handle_list, handle_version};
use make::{
    handle_make_command, handle_make_controller, handle_make_event, handle_make_listener,
    handle_make_middleware, handle_make_model, handle_make_service, handle_make_subscribe,
    handle_make_validate,
};
use optimize::{
    handle_clear, handle_config_cache, handle_optimize, handle_optimize_route,
    handle_optimize_schema,
};
use route::{handle_route_cache, handle_route_list};
use server::{handle_fastcgi, handle_run};
use service::{handle_service_discover, handle_vendor_publish};

/// 命令分发入口
/// 根据命令类型调用对应的处理函数
/// 这是所有命令执行的统一入口点
///
/// # 参数
/// - `command`: 解析后的命令枚举
///
/// # 返回值
/// 命令执行成功返回 Ok(())，失败返回错误信息
pub async fn dispatch(command: &Commands) -> Result<()> {
    // 根据命令类型进行分发
    match command {
        // 启动 HTTP 服务器
        Commands::Run {
            host,
            port,
            daemon,
            debug,
            workers,
        } => handle_run(host, *port, daemon, debug, workers).await,

        // 启动 FastCGI 服务器
        Commands::Fastcgi { socket, debug } => handle_fastcgi(socket, debug).await,

        // 构建应用目录
        Commands::Build { name } => handle_build(name).await,

        // 创建控制器
        Commands::MakeController { name, plain, api } => {
            handle_make_controller(name, *plain, *api).await
        }

        // 创建模型
        Commands::MakeModel { name } => handle_make_model(name).await,

        // 创建中间件
        Commands::MakeMiddleware { name } => handle_make_middleware(name).await,

        // 创建验证器
        Commands::MakeValidate { name } => handle_make_validate(name).await,

        // 创建事件
        Commands::MakeEvent { name } => handle_make_event(name).await,

        // 创建监听器
        Commands::MakeListener { name } => handle_make_listener(name).await,

        // 创建订阅者
        Commands::MakeSubscribe { name } => handle_make_subscribe(name).await,

        // 创建服务类
        Commands::MakeService { name } => handle_make_service(name).await,

        // 创建自定义命令
        Commands::MakeCommand { name } => handle_make_command(name).await,

        // 查看路由列表
        Commands::RouteList => handle_route_list().await,

        // 缓存路由
        Commands::RouteCache => handle_route_cache().await,

        // 优化应用
        Commands::Optimize => handle_optimize().await,

        // 生成路由缓存
        Commands::OptimizeRoute => handle_optimize_route().await,

        // 生成数据表字段缓存
        Commands::OptimizeSchema => handle_optimize_schema().await,

        // 清除运行时缓存
        Commands::Clear => handle_clear().await,

        // 缓存配置
        Commands::ConfigCache => handle_config_cache().await,

        // Composer 安装依赖
        Commands::ComposerInstall { no_dev, optimize } => {
            handle_composer_install(*no_dev, *optimize).await
        }

        // Composer 更新依赖
        Commands::ComposerUpdate { packages, no_dev } => {
            handle_composer_update(packages, *no_dev).await
        }

        // Composer 添加依赖
        Commands::ComposerRequire { packages, dev } => {
            handle_composer_require(packages, *dev).await
        }

        // Composer 移除依赖
        Commands::ComposerRemove { packages } => handle_composer_remove(packages).await,

        // Composer 重新生成自动加载
        Commands::ComposerDumpAutoload { optimize } => {
            handle_composer_dump_autoload(*optimize).await
        }

        // Composer 查看已安装的包
        Commands::ComposerShow { package, direct } => {
            handle_composer_show(package, *direct).await
        }

        // Composer 查看过时的包
        Commands::ComposerOutdated => handle_composer_outdated().await,

        // Composer 验证 composer.json
        Commands::ComposerValidate => handle_composer_validate().await,

        // Composer 配置
        Commands::ComposerConfig { args, list } => handle_composer_config(args, *list).await,

        // Composer 清除缓存
        Commands::ComposerClearCache => handle_composer_clear_cache().await,

        // Composer 诊断
        Commands::ComposerDiagnose => handle_composer_diagnose().await,

        // 自动注册扩展包系统服务
        Commands::ServiceDiscover => handle_service_discover().await,

        // 发布扩展配置文件
        Commands::VendorPublish { package, force } => {
            handle_vendor_publish(package, *force).await
        }

        // 查看版本
        Commands::Version => handle_version().await,

        // 列出所有命令
        Commands::List => handle_list().await,
    }
}
