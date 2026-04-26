//! WebSocket 内嵌服务
//!
//! 提供完整的 WebSocket 服务支持
//! 支持：房间管理、心跳检测、连接限制、消息广播
//!
//! # 功能特性
//! - 房间管理（join/leave/broadcast）
//! - 心跳检测（可配置间隔）
//! - 连接限制（最大连接数/超时断开）
//! - 消息协议（JSON/文本/二进制）
//! - 事件回调（open/message/close/error）

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};

/// WebSocket 连接管理器
pub struct WebSocketManager {
    /// 活跃连接
    connections: Arc<dashmap::DashMap<String, WebSocketConnection>>,
    /// 房间管理（房间名 → 连接ID集合）
    rooms: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// 配置
    config: WebSocketConfig,
}

/// WebSocket 配置
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// 最大连接数
    pub max_connections: usize,
    /// 心跳间隔（秒）
    pub heartbeat_interval: u64,
    /// 心跳超时（秒）
    pub heartbeat_timeout: u64,
    /// 最大消息大小（字节）
    pub max_message_size: usize,
    /// 连接超时（秒，0表示不超时）
    pub connection_timeout: u64,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            heartbeat_interval: 30,
            heartbeat_timeout: 60,
            max_message_size: 1024 * 1024, // 1MB
            connection_timeout: 0,         // 不超时
        }
    }
}

/// WebSocket 连接
pub struct WebSocketConnection {
    /// 连接 ID
    pub id: String,
    /// 发送通道
    pub tx: mpsc::UnboundedSender<WebSocketMessage>,
    /// 连接时间
    pub connected_at: Instant,
    /// 最后活跃时间
    pub last_active: Instant,
    /// 用户数据
    pub user_data: HashMap<String, serde_json::Value>,
    /// 连接的房间列表
    pub rooms: HashSet<String>,
}

/// WebSocket 消息
#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    /// 文本消息
    Text(String),
    /// 二进制消息
    Binary(Vec<u8>),
    /// Ping 消息
    Ping(Vec<u8>),
    /// Pong 消息
    Pong(Vec<u8>),
    /// 关闭消息
    Close(Option<u16>, Option<String>),
}

/// WebSocket 事件
#[derive(Debug, Clone)]
pub enum WebSocketEvent {
    /// 连接打开
    Open {
        connection_id: String,
    },
    /// 收到消息
    Message {
        connection_id: String,
        message: WebSocketMessage,
    },
    /// 连接关闭
    Close {
        connection_id: String,
        code: Option<u16>,
        reason: Option<String>,
    },
    /// 发生错误
    Error {
        connection_id: String,
        error: String,
    },
}

/// 事件回调类型
pub type EventCallback = Box<dyn Fn(WebSocketEvent) + Send + Sync>;

impl WebSocketManager {
    /// 创建新的 WebSocket 管理器
    pub fn new() -> Self {
        Self {
            connections: Arc::new(dashmap::DashMap::new()),
            rooms: Arc::new(RwLock::new(HashMap::new())),
            config: WebSocketConfig::default(),
        }
    }

    /// 使用自定义配置创建管理器
    pub fn with_config(config: WebSocketConfig) -> Self {
        Self {
            connections: Arc::new(dashmap::DashMap::new()),
            rooms: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 添加连接
    ///
    /// # 参数
    /// - `id`: 连接 ID
    /// - `tx`: 发送通道
    ///
    /// # 返回
    /// 是否添加成功（超过最大连接数会失败）
    pub async fn add_connection(&self, id: String, tx: mpsc::UnboundedSender<WebSocketMessage>) -> bool {
        // 检查连接数限制
        if self.connections.len() >= self.config.max_connections {
            tracing::warn!("WebSocket 连接数已达上限: {}", self.config.max_connections);
            return false;
        }

        let conn = WebSocketConnection {
            id: id.clone(),
            tx,
            connected_at: Instant::now(),
            last_active: Instant::now(),
            user_data: HashMap::new(),
            rooms: HashSet::new(),
        };

        self.connections.insert(id, conn);
        tracing::debug!("WebSocket 连接已添加，当前连接数: {}", self.connections.len());

        true
    }

    /// 移除连接
    pub async fn remove_connection(&self, id: &str) {
        // 从所有房间移除
        if let Some((_, conn)) = self.connections.remove(id) {
            let mut rooms = self.rooms.write().await;
            for room_name in &conn.rooms {
                if let Some(room) = rooms.get_mut(room_name) {
                    room.remove(id);
                    if room.is_empty() {
                        rooms.remove(room_name);
                    }
                }
            }
        }

        tracing::debug!("WebSocket 连接已移除，当前连接数: {}", self.connections.len());
    }

    /// 加入房间
    ///
    /// # 参数
    /// - `room_name`: 房间名称
    /// - `connection_id`: 连接 ID
    pub async fn join_room(&self, room_name: &str, connection_id: &str) {
        // 添加到房间
        {
            let mut rooms = self.rooms.write().await;
            rooms
                .entry(room_name.to_string())
                .or_insert_with(HashSet::new)
                .insert(connection_id.to_string());
        }

        // 更新连接的房间列表
        if let Some(mut conn) = self.connections.get_mut(connection_id) {
            conn.rooms.insert(room_name.to_string());
        }

        tracing::debug!("连接 {} 加入房间 {}", connection_id, room_name);
    }

    /// 离开房间
    pub async fn leave_room(&self, room_name: &str, connection_id: &str) {
        // 从房间移除
        {
            let mut rooms = self.rooms.write().await;
            if let Some(room) = rooms.get_mut(room_name) {
                room.remove(connection_id);
                if room.is_empty() {
                    rooms.remove(room_name);
                }
            }
        }

        // 更新连接的房间列表
        if let Some(mut conn) = self.connections.get_mut(connection_id) {
            conn.rooms.remove(room_name);
        }

        tracing::debug!("连接 {} 离开房间 {}", connection_id, room_name);
    }

    /// 广播消息到房间
    ///
    /// # 参数
    /// - `room_name`: 房间名称
    /// - `message`: 消息内容
    /// - `exclude`: 排除的连接 ID（可选）
    pub async fn broadcast_to_room(&self, room_name: &str, message: &str, exclude: Option<&str>) {
        let rooms = self.rooms.read().await;

        if let Some(room) = rooms.get(room_name) {
            let message = WebSocketMessage::Text(message.to_string());
            let mut sent_count = 0;

            for conn_id in room {
                if let Some(exclude_id) = exclude {
                    if conn_id == exclude_id {
                        continue;
                    }
                }

                if let Some(conn) = self.connections.get(conn_id) {
                    if conn.tx.send(message.clone()).is_ok() {
                        sent_count += 1;
                    }
                }
            }

            tracing::trace!("房间 {} 广播消息到 {} 个连接", room_name, sent_count);
        }
    }

    /// 广播消息到所有连接
    pub fn broadcast(&self, message: &str) {
        let message = WebSocketMessage::Text(message.to_string());
        let mut sent_count = 0;

        for conn in self.connections.iter() {
            if conn.tx.send(message.clone()).is_ok() {
                sent_count += 1;
            }
        }

        tracing::trace!("全局广播消息到 {} 个连接", sent_count);
    }

    /// 发送消息到指定连接
    pub fn send_to(&self, connection_id: &str, message: &str) -> bool {
        if let Some(conn) = self.connections.get(connection_id) {
            conn.tx.send(WebSocketMessage::Text(message.to_string())).is_ok()
        } else {
            false
        }
    }

    /// 发送二进制消息到指定连接
    pub fn send_binary_to(&self, connection_id: &str, data: &[u8]) -> bool {
        if let Some(conn) = self.connections.get(connection_id) {
            conn.tx.send(WebSocketMessage::Binary(data.to_vec())).is_ok()
        } else {
            false
        }
    }

    /// 更新连接活跃时间
    pub fn update_activity(&self, connection_id: &str) {
        if let Some(mut conn) = self.connections.get_mut(connection_id) {
            conn.last_active = Instant::now();
        }
    }

    /// 设置连接用户数据
    pub fn set_user_data(&self, connection_id: &str, key: &str, value: serde_json::Value) {
        if let Some(mut conn) = self.connections.get_mut(connection_id) {
            conn.user_data.insert(key.to_string(), value);
        }
    }

    /// 获取连接用户数据
    pub fn get_user_data(&self, connection_id: &str, key: &str) -> Option<serde_json::Value> {
        self.connections.get(connection_id)
            .and_then(|conn| conn.user_data.get(key).cloned())
    }

    /// 获取在线连接数
    pub fn online_count(&self) -> usize {
        self.connections.len()
    }

    /// 获取房间连接数
    pub async fn room_count(&self, room_name: &str) -> usize {
        self.rooms.read().await
            .get(room_name)
            .map(|r| r.len())
            .unwrap_or(0)
    }

    /// 获取所有房间名称
    pub async fn get_rooms(&self) -> Vec<String> {
        self.rooms.read().await.keys().cloned().collect()
    }

    /// 获取连接信息
    pub fn get_connection_info(&self, connection_id: &str) -> Option<ConnectionInfo> {
        self.connections.get(connection_id).map(|conn| ConnectionInfo {
            id: conn.id.clone(),
            connected_at: conn.connected_at,
            last_active: conn.last_active,
            rooms: conn.rooms.iter().cloned().collect(),
        })
    }

    /// 检查超时连接
    ///
    /// # 返回
    /// 超时的连接 ID 列表
    pub fn check_timeout_connections(&self) -> Vec<String> {
        let now = Instant::now();
        let timeout = Duration::from_secs(self.config.heartbeat_timeout);

        self.connections
            .iter()
            .filter(|conn| now.duration_since(conn.last_active) > timeout)
            .map(|conn| conn.id.clone())
            .collect()
    }

    /// 发送心跳检测
    pub fn send_heartbeat(&self) {
        for conn in self.connections.iter() {
            let _ = conn.tx.send(WebSocketMessage::Ping(vec![]));
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> WebSocketStats {
        WebSocketStats {
            total_connections: self.connections.len(),
            max_connections: self.config.max_connections,
        }
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 连接信息
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// 连接 ID
    pub id: String,
    /// 连接时间
    pub connected_at: Instant,
    /// 最后活跃时间
    pub last_active: Instant,
    /// 所在房间
    pub rooms: Vec<String>,
}

/// WebSocket 统计信息
#[derive(Debug, Clone)]
pub struct WebSocketStats {
    /// 当前连接数
    pub total_connections: usize,
    /// 最大连接数
    pub max_connections: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_manager() {
        let manager = WebSocketManager::new();

        let (tx, _rx) = mpsc::unbounded_channel();
        let result = manager.add_connection("conn1".to_string(), tx).await;

        assert!(result);
        assert_eq!(manager.online_count(), 1);
    }

    #[tokio::test]
    async fn test_room_management() {
        let manager = WebSocketManager::new();

        let (tx, _rx) = mpsc::unbounded_channel();
        manager.add_connection("conn1".to_string(), tx).await;

        manager.join_room("room1", "conn1").await;
        assert_eq!(manager.room_count("room1").await, 1);

        manager.leave_room("room1", "conn1").await;
        assert_eq!(manager.room_count("room1").await, 0);
    }
}
