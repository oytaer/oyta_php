//! 流管理模块
//! 
//! 实现 QUIC 流的管理功能，包括：
//! - 流创建与销毁
//! - 流状态管理
//! - 流量控制
//! - 流优先级

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 流 ID 类型
/// 用于唯一标识一个流
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StreamId(pub u64);

impl StreamId {
    /// 创建新的流 ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    /// 获取原始值
    pub fn value(&self) -> u64 {
        self.0
    }
    
    /// 检查是否为客户端发起
    pub fn is_client_initiated(&self) -> bool {
        self.0 % 2 == 0
    }
    
    /// 检查是否为服务端发起
    pub fn is_server_initiated(&self) -> bool {
        self.0 % 2 == 1
    }
    
    /// 检查是否为双向流
    pub fn is_bidi(&self) -> bool {
        self.0 % 4 < 2
    }
    
    /// 检查是否为单向流
    pub fn is_uni(&self) -> bool {
        self.0 % 4 >= 2
    }
}

/// 为 StreamId 实现默认值
impl Default for StreamId {
    fn default() -> Self {
        Self(0)
    }
}

/// 流类型枚举
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

/// 流状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamState {
    /// 空闲状态
    Idle,
    /// 打开状态
    Open,
    /// 半关闭（本地）
    HalfClosedLocal,
    /// 半关闭（远程）
    HalfClosedRemote,
    /// 已关闭
    Closed,
    /// 数据已发送
    DataSent,
    /// 数据已接收
    DataReceived,
    /// 重置已发送
    ResetSent,
    /// 重置已接收
    ResetReceived,
}

/// 为 StreamState 实现默认值
impl Default for StreamState {
    fn default() -> Self {
        StreamState::Idle
    }
}

/// 流优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StreamPriority(pub u8);

impl StreamPriority {
    /// 最高优先级
    pub const HIGHEST: Self = Self(0);
    /// 高优先级
    pub const HIGH: Self = Self(64);
    /// 普通优先级
    pub const NORMAL: Self = Self(128);
    /// 低优先级
    pub const LOW: Self = Self(192);
    /// 最低优先级
    pub const LOWEST: Self = Self(255);
    
    /// 创建新的优先级
    pub fn new(level: u8) -> Self {
        Self(level)
    }
    
    /// 获取优先级级别
    pub fn level(&self) -> u8 {
        self.0
    }
}

/// 为 StreamPriority 实现默认值
impl Default for StreamPriority {
    fn default() -> Self {
        Self::NORMAL
    }
}

/// 流统计信息
#[derive(Debug, Default)]
pub struct StreamStats {
    /// 发送的字节数
    pub bytes_sent: AtomicU64,
    /// 接收的字节数
    pub bytes_received: AtomicU64,
    /// 发送的数据包数
    pub packets_sent: AtomicU64,
    /// 接收的数据包数
    pub packets_received: AtomicU64,
    /// 重传次数
    pub retransmissions: AtomicU64,
    /// 流持续时间（毫秒）
    pub duration_ms: AtomicU64,
}

/// 流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// 最大发送数据量
    pub max_send_data: u64,
    /// 最大接收数据量
    pub max_recv_data: u64,
    /// 发送缓冲区大小
    pub send_buffer_size: usize,
    /// 接收缓冲区大小
    pub recv_buffer_size: usize,
    /// 是否启用流优先级
    pub enable_priority: bool,
    /// 默认优先级
    pub default_priority: StreamPriority,
}

/// 为 StreamConfig 实现默认值
impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            max_send_data: 1024 * 1024,
            max_recv_data: 1024 * 1024,
            send_buffer_size: 64 * 1024,
            recv_buffer_size: 64 * 1024,
            enable_priority: true,
            default_priority: StreamPriority::NORMAL,
        }
    }
}

/// 流管理器
/// 管理所有流的生命周期
pub struct StreamManager {
    /// 流映射
    streams: RwLock<HashMap<StreamId, Arc<ManagedStream>>>,
    /// 配置
    config: StreamConfig,
    /// 下一个客户端双向流 ID
    next_client_bidi: AtomicU64,
    /// 下一个服务端双向流 ID
    next_server_bidi: AtomicU64,
    /// 下一个客户端单向流 ID
    next_client_uni: AtomicU64,
    /// 下一个服务端单向流 ID
    next_server_uni: AtomicU64,
    /// 是否为服务端
    is_server: bool,
    /// 统计信息
    stats: StreamManagerStats,
}

/// 流管理器统计信息
#[derive(Debug, Default)]
pub struct StreamManagerStats {
    /// 总流数
    pub total_streams: AtomicU64,
    /// 活跃流数
    pub active_streams: AtomicU64,
    /// 已关闭流数
    pub closed_streams: AtomicU64,
    /// 重置流数
    pub reset_streams: AtomicU64,
}

/// 为 StreamManager 实现相关方法
impl StreamManager {
    /// 创建新的流管理器
    /// 
    /// # 参数
    /// - `config`: 流配置
    /// - `is_server`: 是否为服务端
    /// 
    /// # 返回
    /// 新的流管理器实例
    pub fn new(config: StreamConfig, is_server: bool) -> Self {
        Self {
            streams: RwLock::new(HashMap::new()),
            config,
            next_client_bidi: AtomicU64::new(0),
            next_server_bidi: AtomicU64::new(1),
            next_client_uni: AtomicU64::new(2),
            next_server_uni: AtomicU64::new(3),
            is_server,
            stats: StreamManagerStats::default(),
        }
    }
    
    /// 使用默认配置创建流管理器
    pub fn with_defaults(is_server: bool) -> Self {
        Self::new(StreamConfig::default(), is_server)
    }
    
    /// 创建新的双向流
    /// 
    /// # 返回
    /// 新创建的流
    pub fn create_bidi_stream(&self) -> Arc<ManagedStream> {
        // 根据角色选择流 ID
        let stream_id = if self.is_server {
            self.next_server_bidi.fetch_add(4, Ordering::Relaxed)
        } else {
            self.next_client_bidi.fetch_add(4, Ordering::Relaxed)
        };
        
        // 确定流类型
        let stream_type = if self.is_server {
            StreamType::ServerBidi
        } else {
            StreamType::ClientBidi
        };
        
        // 创建流
        self.create_stream(StreamId::new(stream_id), stream_type)
    }
    
    /// 创建新的单向流
    /// 
    /// # 返回
    /// 新创建的流
    pub fn create_uni_stream(&self) -> Arc<ManagedStream> {
        // 根据角色选择流 ID
        let stream_id = if self.is_server {
            self.next_server_uni.fetch_add(4, Ordering::Relaxed)
        } else {
            self.next_client_uni.fetch_add(4, Ordering::Relaxed)
        };
        
        // 确定流类型
        let stream_type = if self.is_server {
            StreamType::ServerUni
        } else {
            StreamType::ClientUni
        };
        
        // 创建流
        self.create_stream(StreamId::new(stream_id), stream_type)
    }
    
    /// 创建流
    fn create_stream(&self, stream_id: StreamId, stream_type: StreamType) -> Arc<ManagedStream> {
        // 创建流实例
        let stream = Arc::new(ManagedStream::new(
            stream_id,
            stream_type,
            self.config.clone(),
        ));
        
        // 存储流
        {
            let mut streams = self.streams.write().unwrap();
            streams.insert(stream_id, stream.clone());
        }
        
        // 更新统计
        self.stats.total_streams.fetch_add(1, Ordering::Relaxed);
        self.stats.active_streams.fetch_add(1, Ordering::Relaxed);
        
        stream
    }
    
    /// 获取流
    /// 
    /// # 参数
    /// - `stream_id`: 流 ID
    /// 
    /// # 返回
    /// 流引用（如果存在）
    pub fn get_stream(&self, stream_id: StreamId) -> Option<Arc<ManagedStream>> {
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
    pub fn close_stream(&self, stream_id: StreamId) -> bool {
        let mut streams = self.streams.write().unwrap();
        
        if let Some(stream) = streams.remove(&stream_id) {
            stream.close();
            self.stats.active_streams.fetch_sub(1, Ordering::Relaxed);
            self.stats.closed_streams.fetch_add(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
    
    /// 重置流
    /// 
    /// # 参数
    /// - `stream_id`: 流 ID
    /// - `error_code`: 错误码
    /// 
    /// # 返回
    /// 是否成功重置
    pub fn reset_stream(&self, stream_id: StreamId, _error_code: u64) -> bool {
        let mut streams = self.streams.write().unwrap();
        
        if let Some(stream) = streams.remove(&stream_id) {
            stream.reset();
            self.stats.active_streams.fetch_sub(1, Ordering::Relaxed);
            self.stats.reset_streams.fetch_add(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
    
    /// 获取所有活跃流
    /// 
    /// # 返回
    /// 活跃流列表
    pub fn get_active_streams(&self) -> Vec<Arc<ManagedStream>> {
        let streams = self.streams.read().unwrap();
        streams.values()
            .filter(|s| s.is_active())
            .cloned()
            .collect()
    }
    
    /// 获取流数量
    /// 
    /// # 返回
    /// 当前流数量
    pub fn stream_count(&self) -> usize {
        self.streams.read().unwrap().len()
    }
    
    /// 获取统计信息
    pub fn stats(&self) -> &StreamManagerStats {
        &self.stats
    }
    
    /// 清理已关闭的流
    /// 
    /// # 返回
    /// 清理的流数量
    pub fn cleanup_closed_streams(&self) -> usize {
        let mut streams = self.streams.write().unwrap();
        let len_before = streams.len();
        
        streams.retain(|_, s| s.is_active());
        
        len_before - streams.len()
    }
}

/// 托管流
/// 由流管理器管理的流
pub struct ManagedStream {
    /// 流 ID
    pub stream_id: StreamId,
    /// 流类型
    pub stream_type: StreamType,
    /// 流状态
    pub state: RwLock<StreamState>,
    /// 优先级
    pub priority: RwLock<StreamPriority>,
    /// 配置
    pub config: StreamConfig,
    /// 发送偏移量
    pub send_offset: AtomicU64,
    /// 接收偏移量
    pub recv_offset: AtomicU64,
    /// 发送缓冲区
    pub send_buffer: RwLock<Vec<u8>>,
    /// 接收缓冲区
    pub recv_buffer: RwLock<Vec<u8>>,
    /// 统计信息
    pub stats: StreamStats,
    /// 创建时间
    pub created_at: u64,
    /// 是否可读
    pub readable: AtomicBool,
    /// 是否可写
    pub writable: AtomicBool,
}

/// 为 ManagedStream 实现相关方法
impl ManagedStream {
    /// 创建新的托管流
    /// 
    /// # 参数
    /// - `stream_id`: 流 ID
    /// - `stream_type`: 流类型
    /// - `config`: 流配置
    /// 
    /// # 返回
    /// 新的托管流实例
    pub fn new(stream_id: StreamId, stream_type: StreamType, config: StreamConfig) -> Self {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        Self {
            stream_id,
            stream_type,
            state: RwLock::new(StreamState::Idle),
            priority: RwLock::new(config.default_priority),
            config,
            send_offset: AtomicU64::new(0),
            recv_offset: AtomicU64::new(0),
            send_buffer: RwLock::new(Vec::new()),
            recv_buffer: RwLock::new(Vec::new()),
            stats: StreamStats::default(),
            created_at: now,
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
        // 检查是否可写
        if !self.writable.load(Ordering::Relaxed) {
            return 0;
        }
        
        // 更新状态为打开
        {
            let mut state = self.state.write().unwrap();
            if *state == StreamState::Idle {
                *state = StreamState::Open;
            }
        }
        
        // 计算可发送的数据量
        let current_offset = self.send_offset.load(Ordering::Relaxed);
        let available = self.config.max_send_data.saturating_sub(current_offset);
        let to_send = data.len().min(available as usize);
        
        // 写入缓冲区
        {
            let mut buffer = self.send_buffer.write().unwrap();
            buffer.extend_from_slice(&data[..to_send]);
        }
        
        // 更新偏移量
        self.send_offset.fetch_add(to_send as u64, Ordering::Relaxed);
        
        // 更新统计
        self.stats.bytes_sent.fetch_add(to_send as u64, Ordering::Relaxed);
        self.stats.packets_sent.fetch_add(1, Ordering::Relaxed);
        
        to_send
    }
    
    /// 接收数据
    /// 
    /// # 返回
    /// 接收到的数据
    pub fn recv(&self) -> Vec<u8> {
        // 检查是否可读
        if !self.readable.load(Ordering::Relaxed) {
            return Vec::new();
        }
        
        // 取出数据
        let data = {
            let mut buffer = self.recv_buffer.write().unwrap();
            std::mem::take(&mut *buffer)
        };
        
        // 更新统计
        if !data.is_empty() {
            self.stats.bytes_received.fetch_add(data.len() as u64, Ordering::Relaxed);
            self.stats.packets_received.fetch_add(1, Ordering::Relaxed);
        }
        
        data
    }
    
    /// 写入接收数据（由底层调用）
    /// 
    /// # 参数
    /// - `data`: 接收到的数据
    /// 
    /// # 返回
    /// 写入的字节数
    pub fn write_recv(&self, data: &[u8]) -> usize {
        // 检查是否可读
        if !self.readable.load(Ordering::Relaxed) {
            return 0;
        }
        
        // 计算可接收的数据量
        let current_offset = self.recv_offset.load(Ordering::Relaxed);
        let available = self.config.max_recv_data.saturating_sub(current_offset);
        let to_write = data.len().min(available as usize);
        
        // 写入缓冲区
        {
            let mut buffer = self.recv_buffer.write().unwrap();
            buffer.extend_from_slice(&data[..to_write]);
        }
        
        // 更新偏移量
        self.recv_offset.fetch_add(to_write as u64, Ordering::Relaxed);
        
        to_write
    }
    
    /// 关闭发送端
    pub fn close_send(&self) {
        self.writable.store(false, Ordering::Relaxed);
        
        // 更新状态
        let mut state = self.state.write().unwrap();
        *state = match *state {
            StreamState::Open => StreamState::HalfClosedLocal,
            StreamState::HalfClosedRemote => StreamState::Closed,
            _ => *state,
        };
    }
    
    /// 关闭接收端
    pub fn close_recv(&self) {
        self.readable.store(false, Ordering::Relaxed);
        
        // 更新状态
        let mut state = self.state.write().unwrap();
        *state = match *state {
            StreamState::Open => StreamState::HalfClosedRemote,
            StreamState::HalfClosedLocal => StreamState::Closed,
            _ => *state,
        };
    }
    
    /// 完全关闭流
    pub fn close(&self) {
        self.writable.store(false, Ordering::Relaxed);
        self.readable.store(false, Ordering::Relaxed);
        
        // 更新状态
        let mut state = self.state.write().unwrap();
        *state = StreamState::Closed;
        
        // 更新持续时间
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        self.stats.duration_ms.store(now - self.created_at, Ordering::Relaxed);
    }
    
    /// 重置流
    pub fn reset(&self) {
        self.close();
        
        // 清空缓冲区
        self.send_buffer.write().unwrap().clear();
        self.recv_buffer.write().unwrap().clear();
        
        // 更新状态
        let mut state = self.state.write().unwrap();
        *state = StreamState::ResetSent;
    }
    
    /// 设置优先级
    /// 
    /// # 参数
    /// - `priority`: 新优先级
    pub fn set_priority(&self, priority: StreamPriority) {
        let mut p = self.priority.write().unwrap();
        *p = priority;
    }
    
    /// 获取优先级
    pub fn get_priority(&self) -> StreamPriority {
        *self.priority.read().unwrap()
    }
    
    /// 获取状态
    pub fn get_state(&self) -> StreamState {
        *self.state.read().unwrap()
    }
    
    /// 检查是否活跃
    pub fn is_active(&self) -> bool {
        matches!(self.get_state(), StreamState::Idle | StreamState::Open | 
                 StreamState::HalfClosedLocal | StreamState::HalfClosedRemote)
    }
    
    /// 检查是否可读
    pub fn can_read(&self) -> bool {
        self.readable.load(Ordering::Relaxed) && self.is_active()
    }
    
    /// 检查是否可写
    pub fn can_write(&self) -> bool {
        self.writable.load(Ordering::Relaxed) && self.is_active()
    }
    
    /// 获取统计信息
    pub fn stats(&self) -> &StreamStats {
        &self.stats
    }
}

/// 单元测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试流 ID
    #[test]
    fn test_stream_id() {
        let id = StreamId::new(0);
        assert!(id.is_client_initiated());
        assert!(id.is_bidi());
        
        let id = StreamId::new(1);
        assert!(id.is_server_initiated());
        assert!(id.is_bidi());
        
        let id = StreamId::new(2);
        assert!(id.is_client_initiated());
        assert!(id.is_uni());
    }
    
    /// 测试流优先级
    #[test]
    fn test_stream_priority() {
        assert!(StreamPriority::HIGHEST < StreamPriority::NORMAL);
        assert!(StreamPriority::NORMAL < StreamPriority::LOWEST);
    }
    
    /// 测试流管理器
    #[test]
    fn test_stream_manager() {
        let manager = StreamManager::with_defaults(false);
        
        // 创建双向流
        let stream = manager.create_bidi_stream();
        assert!(stream.stream_id.is_client_initiated());
        
        // 获取流
        let retrieved = manager.get_stream(stream.stream_id);
        assert!(retrieved.is_some());
        
        // 关闭流
        let closed = manager.close_stream(stream.stream_id);
        assert!(closed);
    }
    
    /// 测试托管流
    #[test]
    fn test_managed_stream() {
        let stream = ManagedStream::new(
            StreamId::new(0),
            StreamType::ClientBidi,
            StreamConfig::default(),
        );
        
        // 发送数据
        let sent = stream.send(b"hello");
        assert_eq!(sent, 5);
        
        // 接收数据
        stream.write_recv(b"world");
        let data = stream.recv();
        assert_eq!(data, b"world");
        
        // 关闭
        stream.close();
        assert!(!stream.is_active());
    }
}
