//! FastCGI 服务器
//!
//! 监听 Unix Socket 或 TCP 端口，接受 Nginx 的 FastCGI 请求

use anyhow::Result;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{UnixListener, UnixStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

    /// 启动服务器
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
}

/// 处理单个连接
async fn handle_connection(mut stream: UnixStream, handler: FastCGIHandler) -> Result<()> {
    // 读取请求数据
    let mut buffer = Vec::new();
    let mut read_buffer = [0u8; 8192];
    
    // 设置读取超时
    let read_future = async {
        loop {
            match stream.read(&mut read_buffer).await {
                Ok(0) => break, // 连接关闭
                Ok(n) => {
                    buffer.extend_from_slice(&read_buffer[..n]);
                    // 检查是否收到完整的请求
                    if is_request_complete(&buffer) {
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("读取 FastCGI 数据失败: {}", e);
                    return Err(e.into());
                }
            }
        }
        Ok::<_, anyhow::Error>(())
    };

    // 使用超时包装
    if tokio::time::timeout(Duration::from_secs(30), read_future).await.is_err() {
        tracing::warn!("FastCGI 请求超时");
        return Ok(());
    }

    // 处理请求并获取响应
    let response = {
        let cursor = std::io::Cursor::new(&buffer);
        let mut reader = cursor;
        let mut writer = Vec::new();

        // 使用同步处理器处理请求
        handler.handle_connection(&mut reader, &mut writer)?;
        writer
    };

    // 发送响应
    if !response.is_empty() {
        stream.write_all(&response).await?;
        stream.flush().await?;
    }

    Ok(())
}

/// 检查请求是否完整
fn is_request_complete(buffer: &[u8]) -> bool {
    // 简单检查：查找空的 Stdin 记录
    let mut pos = 0;
    let mut found_params_end = false;

    while pos + 8 <= buffer.len() {
        // 读取记录头
        let content_length = u16::from_be_bytes([buffer[pos + 4], buffer[pos + 5]]) as usize;
        let padding_length = buffer[pos + 6] as usize;
        let record_type = buffer[pos + 1];

        // 检查是否是空的 Stdin 记录
        if record_type == 5 && content_length == 0 {
            return true;
        }

        // 检查是否是空的 Params 记录（表示参数结束）
        if record_type == 4 && content_length == 0 {
            found_params_end = true;
        }

        // 移动到下一个记录
        pos += 8 + content_length + padding_length;
    }

    false
}
