//! FastCGI 服务器
//!
//! 监听 Unix Socket 或 TCP 端口，接受 Nginx 的 FastCGI 请求
//!
//! 注意：FastCGI 模块仅在 Unix 系统上可用

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};

use crate::http::state::AppState;
use crate::fastcgi::handler::FastCGIHandler;

/// FastCGI 服务器配置
pub struct FastCGIServerConfig {
    /// Unix Socket 路径
    pub socket_path: PathBuf,
    /// 项目根目录
    pub project_root: PathBuf,
    /// 最大连接数
    pub max_connections: usize,
}

/// FastCGI 服务器
pub struct FastCGIServer {
    /// 配置
    config: FastCGIServerConfig,
    /// 应用状态
    app_state: Arc<AppState>,
}

impl FastCGIServer {
    /// 创建新的 FastCGI 服务器
    pub fn new(config: FastCGIServerConfig, app_state: Arc<AppState>) -> Self {
        FastCGIServer { config, app_state }
    }

    /// 启动服务器（Unix 系统）
    #[cfg(unix)]
    pub async fn start(&self) -> Result<()> {
        // 删除已存在的 socket 文件
        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)?;
        }

        // 创建 socket 目录（如果不存在）
        if let Some(parent) = self.config.socket_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
                // 设置目录权限，允许 Nginx 访问
                std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o755))?;
            }
        }

        // 绑定 Unix Socket
        let listener = UnixListener::bind(&self.config.socket_path)?;
        
        // 设置 socket 文件权限，允许 Nginx 访问
        std::fs::set_permissions(&self.config.socket_path, std::fs::Permissions::from_mode(0o666))?;

        tracing::info!(
            "FastCGI 服务已启动: unix:{}",
            self.config.socket_path.display()
        );

        // 接受连接
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let handler = FastCGIHandler::new(
                        self.app_state.clone(),
                        self.config.project_root.clone(),
                    );

                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, handler).await {
                            tracing::error!("处理 FastCGI 连接失败: {}", e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("接受 FastCGI 连接失败: {}", e);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// 启动服务器（Windows 不支持）
    #[cfg(not(unix))]
    pub async fn start(&self) -> Result<()> {
        anyhow::bail!("FastCGI 模块在 Windows 上不可用，请使用 HTTP 模式运行");
    }
}

/// 处理单个连接（Unix 系统）
#[cfg(unix)]
async fn handle_connection(mut stream: UnixStream, handler: FastCGIHandler) -> Result<()> {
    // 读取请求数据
    let mut buffer = Vec::new();
    let mut read_buffer = [0u8; 8192];
    
    // 读取所有数据
    loop {
        match stream.read(&mut read_buffer).await {
            Ok(0) => break,
            Ok(n) => {
                buffer.extend_from_slice(&read_buffer[..n]);
                // 检查是否读取完整请求
                if buffer.len() > 8 {
                    // 检查 FastCGI 记录边界
                    let content_length = u16::from_be_bytes([buffer[4], buffer[5]]) as usize;
                    let padding_length = buffer[6] as usize;
                    let record_length = 8 + content_length + padding_length;
                    if buffer.len() >= record_length {
                        break;
                    }
                }
            }
            Err(e) => {
                tracing::error!("读取 FastCGI 请求失败：{}", e);
                return Err(anyhow::anyhow!("读取失败：{}", e));
            }
        }
    }



    // 使用同步方式处理
    use std::io::Cursor;
    let input = Cursor::new(&buffer);
    let mut output = Vec::new();
    
    match handler.handle_connection(input, &mut output) {
        Ok(()) => {
            // 发送响应
            if let Err(e) = stream.write_all(&output).await {
                tracing::error!("发送 FastCGI 响应失败：{}", e);
            }
            if let Err(e) = stream.flush().await {
                tracing::error!("刷新 FastCGI 响应失败：{}", e);
            }
        }
        Err(e) => {
            tracing::error!("处理 FastCGI 请求失败：{}", e);
        }
    }

    Ok(())
}
