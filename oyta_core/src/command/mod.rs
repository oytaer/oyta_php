//! 命令行模块入口
//!
//! 只负责声明子模块，不包含任何功能实现代码
//! 所有命令按功能分组到不同的子模块中

pub mod args;
pub mod cache;
pub mod composer;
pub mod config;
pub mod info;
pub mod init;
pub mod make;
pub mod optimize;
pub mod queue;
pub mod route;
pub mod server;
pub mod service;

use anyhow::Result;

use args::Commands;
use cache::{handle_cache_clear, handle_cache_forget, handle_cache_status};
use composer::{
    handle_composer_clear_cache, handle_composer_config, handle_composer_diagnose,
    handle_composer_dump_autoload, handle_composer_install, handle_composer_outdated,
    handle_composer_remove, handle_composer_require, handle_composer_show,
    handle_composer_update, handle_composer_validate,
};
use config::{handle_config_cache, handle_config_clear, handle_config_get, handle_config_set};
use info::{handle_list, handle_version};
use init::{handle_init, handle_new, handle_install, handle_uninstall};
use make::{
    handle_build, handle_make_command, handle_make_controller, handle_make_event,
    handle_make_job, handle_make_listener, handle_make_middleware, handle_make_model,
    handle_make_service, handle_make_subscribe, handle_make_task, handle_make_validate,
};
use optimize::{
    handle_clear, handle_optimize, handle_optimize_route, handle_optimize_schema,
};
use queue::{handle_queue_failed, handle_queue_flush, handle_queue_retry, handle_queue_status};
use route::{handle_route_cache, handle_route_clear, handle_route_list};
use server::{handle_fastcgi, handle_run};
use service::{handle_service_discover, handle_service_list, handle_service_show, handle_vendor_publish};

/// 命令分发入口
///
/// 根据命令类型调用对应的处理函数
///
/// # 参数
/// - `command`: 解析后的命令枚举
///
/// # 返回值
/// 命令执行成功返回 Ok(())，失败返回错误信息
pub async fn dispatch(command: &Commands) -> Result<()> {
    match command {
        // 服务器命令
        Commands::Run {
            host,
            port,
            daemon,
            debug,
            workers,
        } => handle_run(host.as_deref(), *port, daemon, debug, workers).await,

        Commands::Fastcgi { socket, debug } => handle_fastcgi(socket, debug).await,

        // 项目初始化命令
        Commands::New { name, app } => handle_new(name, app.as_deref()).await,

        Commands::Init { name } => handle_init(name.as_deref()).await,

        Commands::Install { path } => handle_install(path).await,

        Commands::Uninstall { path } => handle_uninstall(path).await,

        // 代码生成命令
        Commands::Build { name } => handle_build(name).await,

        Commands::MakeController { name, plain, api } => {
            handle_make_controller(name, *plain, *api).await
        }

        Commands::MakeModel { name } => handle_make_model(name).await,

        Commands::MakeMiddleware { name } => handle_make_middleware(name).await,

        Commands::MakeValidate { name } => handle_make_validate(name).await,

        Commands::MakeEvent { name } => handle_make_event(name).await,

        Commands::MakeListener { name } => handle_make_listener(name).await,

        Commands::MakeSubscribe { name } => handle_make_subscribe(name).await,

        Commands::MakeService { name } => handle_make_service(name).await,

        Commands::MakeCommand { name } => handle_make_command(name).await,

        Commands::MakeJob { name } => handle_make_job(name).await,

        Commands::MakeTask { name } => handle_make_task(name).await,

        // 路由命令
        Commands::RouteList => handle_route_list().await,

        Commands::RouteCache => handle_route_cache().await,

        Commands::RouteClear => handle_route_clear().await,

        // 缓存命令
        Commands::CacheClear { tag } => handle_cache_clear(tag).await,

        Commands::CacheForget { key } => handle_cache_forget(key).await,

        Commands::CacheStatus => handle_cache_status().await,

        // 队列命令
        Commands::QueueFailed { queue } => handle_queue_failed(queue).await,

        Commands::QueueRetry { id, queue } => handle_queue_retry(id, queue).await,

        Commands::QueueFlush { queue } => handle_queue_flush(queue).await,

        Commands::QueueStatus { queue } => handle_queue_status(queue).await,

        // 服务命令
        Commands::ServiceList => handle_service_list().await,

        Commands::ServiceShow { name } => handle_service_show(name).await,

        Commands::ServiceDiscover => handle_service_discover().await,

        // 配置命令
        Commands::ConfigCache => handle_config_cache().await,

        Commands::ConfigClear => handle_config_clear().await,

        Commands::ConfigGet { key } => handle_config_get(key).await,

        Commands::ConfigSet { key, value } => handle_config_set(key, value).await,

        // 优化命令
        Commands::Optimize => handle_optimize().await,

        Commands::OptimizeRoute => handle_optimize_route().await,

        Commands::OptimizeSchema => handle_optimize_schema().await,

        Commands::Clear => handle_clear().await,

        // Composer 命令
        Commands::ComposerInstall { no_dev, optimize } => {
            handle_composer_install(*no_dev, *optimize).await
        }

        Commands::ComposerUpdate { packages, no_dev } => {
            handle_composer_update(packages, *no_dev).await
        }

        Commands::ComposerRequire { packages, dev } => {
            handle_composer_require(packages, *dev).await
        }

        Commands::ComposerRemove { packages } => handle_composer_remove(packages).await,

        Commands::ComposerDumpAutoload { optimize } => {
            handle_composer_dump_autoload(*optimize).await
        }

        Commands::ComposerShow { package, direct } => {
            handle_composer_show(package, *direct).await
        }

        Commands::ComposerOutdated => handle_composer_outdated().await,

        Commands::ComposerValidate => handle_composer_validate().await,

        Commands::ComposerConfig { args, list } => handle_composer_config(args, *list).await,

        Commands::ComposerClearCache => handle_composer_clear_cache().await,

        Commands::ComposerDiagnose => handle_composer_diagnose().await,

        // 扩展包命令
        Commands::VendorPublish { package, force } => {
            handle_vendor_publish(package, *force).await
        }

        // 信息命令
        Commands::Version => handle_version().await,

        Commands::List => handle_list().await,
    }
}
