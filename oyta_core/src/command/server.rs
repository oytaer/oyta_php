//! 服务器相关命令模块
//!
//! 包含 HTTP 服务器和 FastCGI 服务器的启动命令处理

use anyhow::Result;

use crate::config;
use crate::env_loader;
use crate::http::state::AppState;
use crate::middleware::dispatcher::MiddlewareDispatcher;
use crate::parser::scanner::Scanner;
use crate::project;

/// 处理 run 命令：启动 HTTP 服务器
///
/// 启动流程：
/// 1. 检测项目根目录
/// 2. 加载 .env 环境变量
/// 3. 确保运行时目录存在
/// 4. 启动 Axum HTTP 服务
///
/// # 参数
/// - `host`: 监听地址
/// - `port`: 监听端口
/// - `daemon`: 是否守护进程模式
/// - `debug`: 是否调试模式
/// - `workers`: 工作进程数量
pub async fn handle_run(
    host: &str,
    port: u16,
    _daemon: &bool,
    is_debug: &bool,
    workers: &Option<usize>,
) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 确保运行时目录存在
    project.ensure_runtime_dirs()?;

    // 扫描项目文件
    let mut scanner = Scanner::new();
    let _scan_result = scanner.scan_all(&project.root)?;

    // 加载配置
    let config_store = config::store::ConfigStore::new();
    let config_loader = config::loader::ConfigLoader::new(&project.config_dir);
    let mut php_parser = crate::parser::php_parser::PhpParser::new();
    config_loader.load(&config_store, &mut php_parser)?;
    config::facade::Config::init_from_store(&config_store);

    // 构建路由表
    let route_registry = crate::router::registry::RouteRegistry::new();
    let controllers = scanner.registry().get_controllers();
    route_registry.generate_auto_routes(&controllers, project.is_multi_app(), None);

    // 解析路由定义文件
    let route_dir = project.root.join("route");
    let _ = crate::router::route_parser::RouteFileParser::parse_route_dir(
        &route_dir,
        &route_registry,
        &mut php_parser,
    )?;

    // 构建中间件调度器
    let registry_arc = std::sync::Arc::new(scanner.registry().clone());
    let mut middleware_dispatcher = MiddlewareDispatcher::new(registry_arc.clone());
    middleware_dispatcher.discover_from_registry();

    // 构建应用状态
    let route_registry_arc = std::sync::Arc::new(route_registry);
    let app_state = AppState::new(
        registry_arc,
        route_registry_arc,
        middleware_dispatcher,
        project.root.clone(),
        *is_debug,
    );

    // 从环境变量读取工作进程数，默认为 2
    let worker_count = workers.unwrap_or_else(|| {
        std::env::var("WORKERS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(2)
    });

    // 输出启动信息
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("  OYTAPHP v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("  监听地址: {}:{}", host, port);
    tracing::info!("  调试模式: {}", if *is_debug { "开启" } else { "关闭" });
    tracing::info!("  工作进程: {}", worker_count);
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 构建服务器配置
    let server_config = crate::http::server::ServerConfig {
        host: host.to_string(),
        port,
        project_root: project.root.clone(),
        public_dir: project.public_dir.clone(),
        workers: worker_count,
        debug: *is_debug,
    };

    // 如果是调试模式，启动热重载
    if *is_debug {
        let hot_reload_registry = app_state.registry.clone();
        let hot_reload_root = project.root.clone();
        tokio::spawn(async move {
            match crate::watcher::hot_reload::HotReloader::new(&hot_reload_root, hot_reload_registry)
            {
                Ok(mut reloader) => reloader.run().await,
                Err(e) => tracing::warn!("热重载启动失败: {}", e),
            }
        });
    }

    // 启动服务器
    crate::http::server::start_server(server_config, app_state).await?;

    Ok(())
}

/// 处理 FastCGI 命令：启动 FastCGI 服务器
///
/// 用于宝塔面板/Nginx 部署，类似于 PHP-FPM 的工作方式
/// 通过 Unix Socket 接收 Nginx 转发的请求
///
/// # 参数
/// - `socket`: Unix Socket 路径
/// - `is_debug`: 是否开启调试模式
pub async fn handle_fastcgi(socket: &str, is_debug: &bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 确保运行时目录存在
    project.ensure_runtime_dirs()?;

    // 扫描项目文件
    let mut scanner = Scanner::new();
    let _scan_result = scanner.scan_all(&project.root)?;

    // 加载配置
    let config_store = config::store::ConfigStore::new();
    let config_loader = config::loader::ConfigLoader::new(&project.config_dir);
    let mut php_parser = crate::parser::php_parser::PhpParser::new();
    config_loader.load(&config_store, &mut php_parser)?;
    config::facade::Config::init_from_store(&config_store);

    // 构建路由表
    let route_registry = crate::router::registry::RouteRegistry::new();
    let controllers = scanner.registry().get_controllers();
    route_registry.generate_auto_routes(&controllers, project.is_multi_app(), None);

    // 解析路由定义文件
    let route_dir = project.root.join("route");
    let _ = crate::router::route_parser::RouteFileParser::parse_route_dir(
        &route_dir,
        &route_registry,
        &mut php_parser,
    )?;

    // 构建中间件调度器
    let registry_arc = std::sync::Arc::new(scanner.registry().clone());
    let mut middleware_dispatcher = MiddlewareDispatcher::new(registry_arc.clone());
    middleware_dispatcher.discover_from_registry();

    // 构建应用状态
    let route_registry_arc = std::sync::Arc::new(route_registry);
    let app_state = AppState::new(
        registry_arc,
        route_registry_arc,
        middleware_dispatcher,
        project.root.clone(),
        *is_debug,
    );

    // 解析 Socket 路径
    let socket_path = if socket.starts_with('/') {
        std::path::PathBuf::from(socket)
    } else {
        project.root.join(socket)
    };

    // 输出启动信息
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("  OYTAPHP FastCGI v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("  Socket: {}", socket_path.display());
    tracing::info!("  调试模式: {}", if *is_debug { "开启" } else { "关闭" });
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 创建并启动 FastCGI 服务器
    let fastcgi_config = crate::fastcgi::server::FastCGIServerConfig {
        socket_path,
        project_root: project.root,
        max_connections: 100,
    };

    let server = crate::fastcgi::server::FastCGIServer::new(
        fastcgi_config,
        std::sync::Arc::new(app_state),
    );
    server.start().await?;

    Ok(())
}
