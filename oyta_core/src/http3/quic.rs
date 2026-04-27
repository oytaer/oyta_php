//! QUIC 协议核心实现模块
//! 
//! 实现 QUIC 协议的核心功能，包括：
//! - 连接建立与维护
//! - 数据包处理
//! - 流量控制
//! - 拥塞控制
//! - 丢包恢复

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// QUIC 连接 ID
/// 用于唯一标识一个连接
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(Vec<u8>);

impl ConnectionId {
    /// 创建新的连接 ID
    /// 
    /// # 参数
    /// - `len`: 连接 ID 长度
    /// 
    /// # 返回
    /// 新的连接 ID
    pub fn new(len: usize) -> Self {
        // 使用时间戳和随机数生成连接 ID
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_nanos() as u64;
        
        let mut id = Vec::with_capacity(len);
        for i in 0..len {
            // 简单的伪随机生成
            id.push(((now >> (i * 8)) & 0xFF) as u8);
        }
        
        Self(id)
    }
    
    /// 从字节数组创建连接 ID
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
    
    /// 获取字节数组引用
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    
    /// 获取长度
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// QUIC 配置
/// 定义 QUIC 连接的各种参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicConfig {
    /// 初始最大数据大小
    pub initial_max_data: u64,
    /// 初始最大流数据大小（双向）
    pub initial_max_stream_data_bidi_local: u64,
    /// 初始最大流数据大小（双向远程）
    pub initial_max_stream_data_bidi_remote: u64,
    /// 初始最大流数据大小（单向）
    pub initial_max_stream_data_uni: u64,
    /// 初始最大双向流数
    pub initial_max_streams_bidi: u64,
    /// 初始最大单向流数
    pub initial_max_streams_uni: u64,
    /// 最大空闲超时（毫秒）
    pub max_idle_timeout_ms: u64,
    /// 最大 UDP 载荷大小
    pub max_udp_payload_size: u64,
    /// ACK 延迟指数
    pub ack_delay_exponent: u8,
    /// 最大 ACK 延迟（毫秒）
    pub max_ack_delay_ms: u64,
    /// 禁用活跃迁移
    pub disable_active_migration: bool,
    /// 初始源连接 ID 长度
    pub initial_source_connection_id_length: usize,
    /// 是否启用 0-RTT
    pub enable_0rtt: bool,
    /// 是否启用连接迁移
    pub enable_connection_migration: bool,
    /// 初始拥塞窗口大小
    pub initial_congestion_window: u64,
    /// 最小拥塞窗口大小
    pub minimum_congestion_window: u64,
    /// 丢包检测超时（毫秒）
    pub loss_detection_timeout_ms: u64,
}

/// 为 QuicConfig 实现默认值
impl Default for QuicConfig {
    fn default() -> Self {
        Self {
            // 默认初始最大数据 10MB
            initial_max_data: 10 * 1024 * 1024,
            // 默认双向流本地最大数据 1MB
            initial_max_stream_data_bidi_local: 1024 * 1024,
            // 默认双向流远程最大数据 1MB
            initial_max_stream_data_bidi_remote: 1024 * 1024,
            // 默认单向流最大数据 1MB
            initial_max_stream_data_uni: 1024 * 1024,
            // 默认最大双向流数 100
            initial_max_streams_bidi: 100,
            // 默认最大单向流数 100
            initial_max_streams_uni: 100,
            // 默认最大空闲超时 30 秒
            max_idle_timeout_ms: 30_000,
            // 默认最大 UDP 载荷 1350 字节
            max_udp_payload_size: 1350,
            // 默认 ACK 延迟指数 3
            ack_delay_exponent: 3,
            // 默认最大 ACK 延迟 25 毫秒
            max_ack_delay_ms: 25,
            // 默认不禁用活跃迁移
            disable_active_migration: false,
            // 默认源连接 ID 长度 8 字节
            initial_source_connection_id_length: 8,
            // 默认启用 0-RTT
            enable_0rtt: true,
            // 默认启用连接迁移
            enable_connection_migration: true,
            // 默认初始拥塞窗口 10 个数据包
            initial_congestion_window: 10,
            // 默认最小拥塞窗口 2 个数据包
            minimum_congestion_window: 2,
            // 默认丢包检测超时 1 秒
            loss_detection_timeout_ms: 1000,
        }
    }
}

/// QUIC 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// 初始状态
    Initial,
    /// 正在握手
    Handshaking,
    /// 已连接
    Connected,
    /// 正在关闭
    Closing,
    /// 已关闭
    Closed,
    /// 拥塞
    Congested,
}

/// 为 ConnectionState 实现默认值
impl Default for ConnectionState {
    fn default() -> Self {
        ConnectionState::Initial
    }
}

/// QUIC 流
/// 表示一个 QUIC 流
#[derive(Debug)]
pub struct QuicStream {
    /// 流 ID
    pub stream_id: u64,
    /// 流类型
    pub stream_type: StreamType,
    /// 流状态
    pub state: StreamState,
    /// 发送偏移量
    pub send_offset: AtomicU64,
    /// 接收偏移量
    pub recv_offset: AtomicU64,
    /// 最大发送数据
    pub max_send_data: AtomicU64,
    /// 最大接收数据
    pub max_recv_data: AtomicU64,
    /// 发送缓冲区
    pub send_buffer: RwLock<Vec<u8>>,
    /// 接收缓冲区
    pub recv_buffer: RwLock<Vec<u8>>,
    /// 是否可读
    pub readable: AtomicBool,
    /// 是否可写
    pub writable: AtomicBool,
}

/// 流类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamType {
    /// 客户端发起的双向流
    ClientBidi,
    /// 服务端发起的双向流
    ServerBidi,
    /// 客户端发起的单向流
    ClientUni,
    /// 服务端发起的单向流
    ServerUni,
}

/// 流状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamState {
    /// 空闲
    Idle,
    /// 打开
    Open,
    /// 半关闭（本地）
    HalfClosedLocal,
    /// 半关闭（远程）
    HalfClosedRemote,
    /// 已关闭
    Closed,
    /// 重置
    Reset,
}

/// 为 QuicStream 实现相关方法
impl QuicStream {
    /// 创建新的 QUIC 流
    /// 
    /// # 参数
    /// - `stream_id`: 流 ID
    /// - `stream_type`: 流类型
    /// 
    /// # 返回
    /// 新的 QUIC 流实例
    pub fn new(stream_id: u64, stream_type: StreamType) -> Self {
        Self {
            stream_id,
            stream_type,
            state: StreamState::Idle,
            send_offset: AtomicU64::new(0),
            recv_offset: AtomicU64::new(0),
            max_send_data: AtomicU64::new(1024 * 1024),
            max_recv_data: AtomicU64::new(1024 * 1024),
            send_buffer: RwLock::new(Vec::new()),
            recv_buffer: RwLock::new(Vec::new()),
            readable: AtomicBool::new(true),
            writable: AtomicBool::new(true),
        }
    }
    
    /// 发送数据
    /// 
    /// # 参数
    /// - `data`: 要发送的数据
    /// 
    /// # 返回
    /// 发送的字节数
    pub fn send(&self, data: &[u8]) -> usize {
        // 获取写锁
        let mut buffer = self.send_buffer.write().unwrap();
        
        // 检查是否可写
        if !self.writable.load(Ordering::Relaxed) {
            return 0;
        }
        
        // 计算可发送的数据量
        let current_offset = self.send_offset.load(Ordering::Relaxed);
        let max_data = self.max_send_data.load(Ordering::Relaxed);
        let available = max_data.saturating_sub(current_offset);
        
        // 限制发送数据量
        let to_send = data.len().min(available as usize);
        
        // 写入缓冲区
        buffer.extend_from_slice(&data[..to_send]);
        
        // 更新偏移量
        self.send_offset.fetch_add(to_send as u64, Ordering::Relaxed);
        
        to_send
    }
    
    /// 接收数据
    /// 
    /// # 返回
    /// 接收到的数据
    pub fn recv(&self) -> Vec<u8> {
        // 获取写锁
        let mut buffer = self.recv_buffer.write().unwrap();
        
        // 检查是否可读
        if !self.readable.load(Ordering::Relaxed) {
            return Vec::new();
        }
        
        // 取出所有数据
        let data = buffer.clone();
        buffer.clear();
        
        data
    }
    
    /// 关闭发送端
    pub fn close_send(&self) {
        self.writable.store(false, Ordering::Relaxed);
    }
    
    /// 关闭接收端
    pub fn close_recv(&self) {
        self.readable.store(false, Ordering::Relaxed);
    }
    
    /// 完全关闭流
    pub fn close(&self) {
        self.writable.store(false, Ordering::Relaxed);
        self.readable.store(false, Ordering::Relaxed);
    }
    
    /// 重置流
    pub fn reset(&self) {
        self.close();
        // 清空缓冲区
        self.send_buffer.write().unwrap().clear();
        self.recv_buffer.write().unwrap().clear();
    }
}

/// QUIC 连接
/// 表示一个 QUIC 连接
pub struct QuicConnection {
    /// 连接 ID
    pub connection_id: ConnectionId,
    /// 本地地址
    pub local_addr: SocketAddr,
    /// 远程地址
    pub remote_addr: SocketAddr,
    /// 连接状态
    pub state: RwLock<ConnectionState>,
    /// 配置
    pub config: QuicConfig,
    /// 流映射
    pub streams: RwLock<HashMap<u64, Arc<QuicStream>>>,
    /// 下一个流 ID（客户端双向）
    pub next_stream_id_client_bidi: AtomicU64,
    /// 下一个流 ID（服务端双向）
    pub next_stream_id_server_bidi: AtomicU64,
    /// 下一个流 ID（客户端单向）
    pub next_stream_id_client_uni: AtomicU64,
    /// 下一个流 ID（服务端单向）
    pub next_stream_id_server_uni: AtomicU64,
    /// 连接统计
    pub stats: QuicConnectionStats,
    /// 是否为服务端
    pub is_server: bool,
    /// 创建时间
    pub created_at: u64,
    /// 最后活动时间
    pub last_activity: AtomicU64,
}

/// QUIC 连接统计信息
#[derive(Debug, Default)]
pub struct QuicConnectionStats {
    /// 发送的数据包数
    pub packets_sent: AtomicU64,
    /// 接收的数据包数
    pub packets_received: AtomicU64,
    /// 发送的字节数
    pub bytes_sent: AtomicU64,
    /// 接收的字节数
    pub bytes_received: AtomicU64,
    /// 丢失的数据包数
    pub packets_lost: AtomicU64,
    /// 重传的数据包数
    pub packets_retransmitted: AtomicU64,
    /// 当前拥塞窗口
    pub congestion_window: AtomicU64,
    /// 当前 RTT（毫秒）
    pub rtt_ms: AtomicU64,
    /// 最小 RTT（毫秒）
    pub min_rtt_ms: AtomicU64,
    /// 平滑 RTT（毫秒）
    pub smoothed_rtt_ms: AtomicU64,
    /// RTT 方差
    pub rtt_variance: AtomicU64,
    /// 当前流数量
    pub stream_count: AtomicU32,
}

/// 为 QuicConnection 实现相关方法
impl QuicConnection {
    /// 创建新的 QUIC 连接
    /// 
    /// # 参数
    /// - `local_addr`: 本地地址
    /// - `remote_addr`: 远程地址
    /// - `config`: QUIC 配置
    /// - `is_server`: 是否为服务端
    /// 
    /// # 返回
    /// 新的 QUIC 连接实例
    pub fn new(
        local_addr: SocketAddr,
        remote_addr: SocketAddr,
        config: QuicConfig,
        is_server: bool,
    ) -> Self {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 生成连接 ID
        let connection_id = ConnectionId::new(config.initial_source_connection_id_length);
        
        Self {
            connection_id,
            local_addr,
            remote_addr,
            state: RwLock::new(ConnectionState::Initial),
            config,
            streams: RwLock::new(HashMap::new()),
            next_stream_id_client_bidi: AtomicU64::new(0),
            next_stream_id_server_bidi: AtomicU64::new(1),
            next_stream_id_client_uni: AtomicU64::new(2),
            next_stream_id_server_uni: AtomicU64::new(3),
            stats: QuicConnectionStats::default(),
            is_server,
            created_at: now,
            last_activity: AtomicU64::new(now),
        }
    }
    
    /// 创建新的双向流
    /// 
    /// # 返回
    /// 新创建的流
    pub fn open_bidi_stream(&self) -> Arc<QuicStream> {
        // 根据角色选择流 ID
        let stream_id = if self.is_server {
            self.next_stream_id_server_bidi.fetch_add(4, Ordering::Relaxed)
        } else {
            self.next_stream_id_client_bidi.fetch_add(4, Ordering::Relaxed)
        };
        
        // 确定流类型
        let stream_type = if self.is_server {
            StreamType::ServerBidi
        } else {
            StreamType::ClientBidi
        };
        
        // 创建流
        let stream = Arc::new(QuicStream::new(stream_id, stream_type));
        
        // 存储流
        {
            let mut streams = self.streams.write().unwrap();
            streams.insert(stream_id, stream.clone());
        }
        
        // 更新统计
        self.stats.stream_count.fetch_add(1, Ordering::Relaxed);
        
        stream
    }
    
    /// 创建新的单向流
    /// 
    /// # 返回
    /// 新创建的流
    pub fn open_uni_stream(&self) -> Arc<QuicStream> {
        // 根据角色选择流 ID
        let stream_id = if self.is_server {
            self.next_stream_id_server_uni.fetch_add(4, Ordering::Relaxed)
        } else {
            self.next_stream_id_client_uni.fetch_add(4, Ordering::Relaxed)
        };
        
        // 确定流类型
        let stream_type = if self.is_server {
            StreamType::ServerUni
        } else {
            StreamType::ClientUni
        };
        
        // 创建流
        let stream = Arc::new(QuicStream::new(stream_id, stream_type));
        
        // 存储流
        {
            let mut streams = self.streams.write().unwrap();
            streams.insert(stream_id, stream.clone());
        }
        
        // 更新统计
        self.stats.stream_count.fetch_add(1, Ordering::Relaxed);
        
        stream
    }
    
    /// 获取流
    /// 
    /// # 参数
    /// - `stream_id`: 流 ID
    /// 
    /// # 返回
    /// 流引用（如果存在）
    pub fn get_stream(&self, stream_id: u64) -> Option<Arc<QuicStream>> {
        let streams = self.streams.read().unwrap();
        streams.get(&stream_id).cloned()
    }
    
    /// 关闭流
    /// 
    /// # 参数
    /// - `stream_id`: 流 ID
    /// 
    /// # 返回
    /// 是否成功关闭
    pub fn close_stream(&self, stream_id: u64) -> bool {
        let mut streams = self.streams.write().unwrap();
        if let Some(stream) = streams.remove(&stream_id) {
            stream.close();
            self.stats.stream_count.fetch_sub(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
    
    /// 获取连接状态
    pub fn get_state(&self) -> ConnectionState {
        *self.state.read().unwrap()
    }
    
    /// 设置连接状态
    pub fn set_state(&self, state: ConnectionState) {
        let mut current = self.state.write().unwrap();
        *current = state;
    }
    
    /// 更新活动时间
    pub fn update_activity(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        self.last_activity.store(now, Ordering::Relaxed);
    }
    
    /// 检查连接是否空闲
    pub fn is_idle(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        let last = self.last_activity.load(Ordering::Relaxed);
        now.saturating_sub(last) > self.config.max_idle_timeout_ms
    }
    
    /// 关闭连接
    pub fn close(&self) {
        // 设置状态为关闭
        self.set_state(ConnectionState::Closing);
        
        // 关闭所有流
        let streams = self.streams.read().unwrap();
        for stream in streams.values() {
            stream.close();
        }
        
        // 设置最终状态
        self.set_state(ConnectionState::Closed);
    }
    
    /// 获取统计信息
    pub fn stats(&self) -> &QuicConnectionStats {
        &self.stats
    }
    
    /// 发送数据包（模拟）
    /// 
    /// # 参数
    /// - `data`: 数据
    /// 
    /// # 返回
    /// 发送的字节数
    pub fn send_packet(&self, data: &[u8]) -> usize {
        self.stats.packets_sent.fetch_add(1, Ordering::Relaxed);
        self.stats.bytes_sent.fetch_add(data.len() as u64, Ordering::Relaxed);
        self.update_activity();
        data.len()
    }
    
    /// 接收数据包（模拟）
    /// 
    /// # 参数
    /// - `data`: 数据
    /// 
    /// # 返回
    /// 接收的字节数
    pub fn recv_packet(&self, data: &[u8]) -> usize {
        self.stats.packets_received.fetch_add(1, Ordering::Relaxed);
        self.stats.bytes_received.fetch_add(data.len() as u64, Ordering::Relaxed);
        self.update_activity();
        data.len()
    }
    
    /// 更新 RTT
    /// 
    /// # 参数
    /// - `rtt_ms`: RTT 值（毫秒）
    pub fn update_rtt(&self, rtt_ms: u64) {
        // 更新最小 RTT
        let min_rtt = self.stats.min_rtt_ms.load(Ordering::Relaxed);
        if min_rtt == 0 || rtt_ms < min_rtt {
            self.stats.min_rtt_ms.store(rtt_ms, Ordering::Relaxed);
        }
        
        // 更新平滑 RTT（使用指数加权移动平均）
        let smoothed = self.stats.smoothed_rtt_ms.load(Ordering::Relaxed);
        let variance = self.stats.rtt_variance.load(Ordering::Relaxed);
        
        if smoothed == 0 {
            // 首次测量
            self.stats.smoothed_rtt_ms.store(rtt_ms, Ordering::Relaxed);
            self.stats.rtt_variance.store(rtt_ms / 2, Ordering::Relaxed);
        } else {
            // 更新平滑 RTT
            let new_smoothed = (7 * smoothed + rtt_ms) / 8;
            self.stats.smoothed_rtt_ms.store(new_smoothed, Ordering::Relaxed);
            
            // 更新 RTT 方差
            let new_variance = (3 * variance + (rtt_ms as i64 - smoothed as i64).abs() as u64) / 4;
            self.stats.rtt_variance.store(new_variance, Ordering::Relaxed);
        }
        
        // 更新当前 RTT
        self.stats.rtt_ms.store(rtt_ms, Ordering::Relaxed);
    }
}

/// 单元测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试连接 ID 创建
    #[test]
    fn test_connection_id() {
        let id = ConnectionId::new(8);
        assert_eq!(id.len(), 8);
        assert!(!id.is_empty());
    }
    
    /// 测试 QUIC 配置默认值
    #[test]
    fn test_quic_config_default() {
        let config = QuicConfig::default();
        assert!(config.enable_0rtt);
        assert!(config.enable_connection_migration);
        assert_eq!(config.initial_max_streams_bidi, 100);
    }
    
    /// 测试 QUIC 流
    #[test]
    fn test_quic_stream() {
        let stream = QuicStream::new(0, StreamType::ClientBidi);
        
        assert_eq!(stream.stream_id, 0);
        assert!(stream.writable.load(Ordering::Relaxed));
        assert!(stream.readable.load(Ordering::Relaxed));
        
        // 发送数据
        let sent = stream.send(b"hello");
        assert_eq!(sent, 5);
        
        // 关闭发送端
        stream.close_send();
        assert!(!stream.writable.load(Ordering::Relaxed));
    }
    
    /// 测试 QUIC 连接
    #[test]
    fn test_quic_connection() {
        let local: SocketAddr = "127.0.0.1:443".parse().unwrap();
        let remote: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        
        let conn = QuicConnection::new(local, remote, QuicConfig::default(), true);
        
        assert_eq!(conn.get_state(), ConnectionState::Initial);
        
        // 创建流
        let stream = conn.open_bidi_stream();
        assert!(stream.stream_id % 4 == 1); // 服务端双向流
        
        // 获取流
        let retrieved = conn.get_stream(stream.stream_id);
        assert!(retrieved.is_some());
        
        // 关闭流
        let closed = conn.close_stream(stream.stream_id);
        assert!(closed);
    }
}
