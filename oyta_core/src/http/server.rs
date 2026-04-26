//! Axum HTTP 服务器模块
//!
//! 封装 Axum Web 服务器，提供 OYTAPHP 的 HTTP 服务
//! 支持：
//! 1. 静态文件服务（public/ 目录）
//! 2. PHP 请求路由分发
//! 3. 优雅关闭
//! 4. 多 worker 进程

use axum::Router;
use axum::routing::any;
use axum::http::{HeaderValue, Method};
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;

use crate::http::handler::php_handler;
use crate::http::state::AppState;

/// 服务器配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// 监听地址
    pub host: String,
    /// 监听端口
    pub port: u16,
    /// 项目根目录
    pub project_root: PathBuf,
    /// public 目录路径
    pub public_dir: PathBuf,
    /// Worker 进程数
    pub workers: usize,
    /// 是否开启调试模式
    pub debug: bool,
}

/// 启动 Axum HTTP 服务器
pub async fn start_server(config: ServerConfig, state: AppState) -> anyhow::Result<()> {
    let addr = format!("{}:{}", config.host, config.port);
    let socket_addr: SocketAddr = addr.parse()
        .map_err(|e| anyhow::anyhow!("无效的监听地址 {}: {}", addr, e))?;

    let cors = if config.debug {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH, Method::OPTIONS])
            .allow_headers(Any)
    } else {
        CorsLayer::new()
            .allow_origin("http://localhost".parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST])
            .allow_headers(Any)
    };

    let app = Router::new()
        .route("/{*path}", any(php_handler))
        .route("/", any(php_handler))
        .nest_service("/static", ServeDir::new(config.public_dir.join("static")))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(socket_addr).await
        .map_err(|e| anyhow::anyhow!("无法绑定地址 {}: {}", addr, e))?;

    tracing::info!(
        "服务器已启动: http://{}:{}",
        config.host, config.port
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| anyhow::anyhow!("服务器运行错误: {}", e))?;

    tracing::info!("服务器已停止");
    Ok(())
}

/// 优雅关闭信号监听
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("无法监听 Ctrl+C 信号");
    tracing::info!("收到关闭信号，正在优雅关闭...");
}
