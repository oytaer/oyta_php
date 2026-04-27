//! 连接管理模块
//! 
//! 实现 QUIC 连接的管理功能，包括：
//! - 连接生命周期管理
//! - 连接池
//! - 连接迁移
//! - 心跳检测

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::quic::{ConnectionId, ConnectionState, QuicConfig, QuicConnection, QuicConnectionStats};

/// 连接统计信息
#[derive(Debug, Default)]
pub struct ConnectionStats {
    /// 总连接数
    pub total_connections: AtomicU64,
    /// 活跃连接数
    pub active_connections: AtomicU64,
    /// 已关闭连接数
    pub closed_connections: AtomicU64,
    /// 失败连接数
    pub failed_connections: AtomicU64,
    /// 总发送字节数
    pub total_bytes_sent: AtomicU64,
    /// 总接收字节数
    pub total_bytes_received: AtomicU64,
    /// 总数据包数
    pub total_packets: AtomicU64,
    /// 丢失数据包数
    pub lost_packets: AtomicU64,
    /// 重传数据包数
    pub retransmitted_packets: AtomicU64,
    /// 平均 RTT（毫秒）
    pub avg_rtt_ms: AtomicU64,
    /// 最大并发连接数
    pub max_concurrent_connections: AtomicU64,
}

/// 连接池配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// 最大连接数
    pub max_connections: usize,
    /// 最小空闲连接数
    pub min_idle_connections: usize,
    /// 连接空闲超时（毫秒）
    pub idle_timeout_ms: u64,
    /// 连接最大生存时间（毫秒）
    pub max_lifetime_ms: u64,
    /// 健康检查间隔（毫秒）
    pub health_check_interval_ms: u64,
    /// 是否启用连接复用
    pub enable_connection_reuse: bool,
    /// 连接建立超时（毫秒）
    pub connect_timeout_ms: u64,
}

/// 为 ConnectionPoolConfig 实现默认值
impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            min_idle_connections: 10,
            idle_timeout_ms: 60_000,
            max_lifetime_ms: 3600_000,
            health_check_interval_ms: 30_000,
            enable_connection_reuse: true,
            connect_timeout_ms: 10_000,
        }
    }
}

/// 连接管理器
/// 管理所有 QUIC 连接
pub struct ConnectionManager {
    /// 连接池
    connections: RwLock<HashMap<ConnectionId, Arc<QuicConnection>>>,
    /// 地址到连接的映射
    addr_to_conn: RwLock<HashMap<SocketAddr, ConnectionId>>,
    /// 配置
    config: ConnectionPoolConfig,
    /// QUIC 配置
    quic_config: QuicConfig,
    /// 统计信息
    stats: ConnectionStats,
    /// 是否正在运行
    running: AtomicBool,
    /// 是否为服务端
    is_server: bool,
}

/// 为 ConnectionManager 实现相关方法
impl ConnectionManager {
    /// 创建新的连接管理器
    /// 
    /// # 参数
    /// - `config`: 连接池配置
    /// - `quic_config`: QUIC 配置
    /// - `is_server`: 是否为服务端
    /// 
    /// # 返回
    /// 新的连接管理器实例
    pub fn new(config: ConnectionPoolConfig, quic_config: QuicConfig, is_server: bool) -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
            addr_to_conn: RwLock::new(HashMap::new()),
            config,
            quic_config,
            stats: ConnectionStats::default(),
            running: AtomicBool::new(false),
            is_server,
        }
    }
    
    /// 使用默认配置创建连接管理器
    pub fn with_defaults(is_server: bool) -> Self {
        Self::new(
            ConnectionPoolConfig::default(),
            QuicConfig::default(),
            is_server,
        )
    }
    
    /// 创建新连接
    /// 
    /// # 参数
    /// - `local_addr`: 本地地址
    /// - `remote_addr`: 远程地址
    /// 
    /// # 返回
    /// 新创建的连接
    pub fn create_connection(
        &self,
        local_addr: SocketAddr,
        remote_addr: SocketAddr,
    ) -> Result<Arc<QuicConnection>, String> {
        // 检查连接数限制
        let current_count = self.connections.read().unwrap().len();
        if current_count >= self.config.max_connections {
            return Err("连接数已达上限".to_string());
        }
        
        // 创建连接
        let conn = Arc::new(QuicConnection::new(
            local_addr,
            remote_addr,
            self.quic_config.clone(),
            self.is_server,
        ));
        
        // 存储连接
        let conn_id = conn.connection_id.clone();
        {
            let mut connections = self.connections.write().unwrap();
            connections.insert(conn_id.clone(), conn.clone());
        }
        
        // 存储地址映射
        {
            let mut addr_map = self.addr_to_conn.write().unwrap();
            addr_map.insert(remote_addr, conn_id);
        }
        
        // 更新统计
        self.stats.total_connections.fetch_add(1, Ordering::Relaxed);
        self.stats.active_connections.fetch_add(1, Ordering::Relaxed);
        
        // 更新最大并发数
        let active = self.stats.active_connections.load(Ordering::Relaxed);
        let max = self.stats.max_concurrent_connections.load(Ordering::Relaxed);
        if active > max {
            self.stats.max_concurrent_connections.store(active, Ordering::Relaxed);
        }
        
        Ok(conn)
    }
    
    /// 获取连接
    /// 
    /// # 参数
    /// - `conn_id`: 连接 ID
    /// 
    /// # 返回
    /// 连接引用（如果存在）
    pub fn get_connection(&self, conn_id: &ConnectionId) -> Option<Arc<QuicConnection>> {
        let connections = self.connections.read().unwrap();
        connections.get(conn_id).cloned()
    }
    
    /// 通过地址获取连接
    /// 
    /// # 参数
    /// - `addr`: 远程地址
    /// 
    /// # 返回
    /// 连接引用（如果存在）
    pub fn get_connection_by_addr(&self, addr: &SocketAddr) -> Option<Arc<QuicConnection>> {
        // 查找连接 ID
        let conn_id = {
            let addr_map = self.addr_to_conn.read().unwrap();
            addr_map.get(addr).cloned()
        }?;
        
        // 获取连接
        self.get_connection(&conn_id)
    }
    
    /// 关闭连接
    /// 
    /// # 参数
    /// - `conn_id`: 连接 ID
    /// 
    /// # 返回
    /// 是否成功关闭
    pub fn close_connection(&self, conn_id: &ConnectionId) -> bool {
        // 获取连接
        let conn = {
            let connections = self.connections.read().unwrap();
            connections.get(conn_id).cloned()
        };
        
        if let Some(conn) = conn {
            // 关闭连接
            conn.close();
            
            // 从连接池移除
            {
                let mut connections = self.connections.write().unwrap();
                connections.remove(conn_id);
            }
            
            // 从地址映射移除
            {
                let mut addr_map = self.addr_to_conn.write().unwrap();
                addr_map.remove(&conn.remote_addr);
            }
            
            // 更新统计
            self.stats.active_connections.fetch_sub(1, Ordering::Relaxed);
            self.stats.closed_connections.fetch_add(1, Ordering::Relaxed);
            
            true
        } else {
            false
        }
    }
    
    /// 获取所有活跃连接
    /// 
    /// # 返回
    /// 活跃连接列表
    pub fn get_active_connections(&self) -> Vec<Arc<QuicConnection>> {
        let connections = self.connections.read().unwrap();
        connections.values()
            .filter(|c| c.get_state() != ConnectionState::Closed)
            .cloned()
            .collect()
    }
    
    /// 获取连接数量
    /// 
    /// # 返回
    /// 当前连接数量
    pub fn connection_count(&self) -> usize {
        self.connections.read().unwrap().len()
    }
    
    /// 清理空闲连接
    /// 
    /// # 返回
    /// 清理的连接数量
    pub fn cleanup_idle_connections(&self) -> usize {
        let mut connections = self.connections.write().unwrap();
        let mut addr_map = self.addr_to_conn.write().unwrap();
        
        // 收集空闲连接
        let idle_conns: Vec<ConnectionId> = connections.iter()
            .filter(|(_, conn)| conn.is_idle())
            .map(|(id, _)| id.clone())
            .collect();
        
        // 关闭并移除空闲连接
        let count = idle_conns.len();
        for conn_id in &idle_conns {
            if let Some(conn) = connections.remove(conn_id) {
                conn.close();
                addr_map.remove(&conn.remote_addr);
            }
        }
        
        // 更新统计
        self.stats.active_connections.fetch_sub(count as u64, Ordering::Relaxed);
        self.stats.closed_connections.fetch_add(count as u64, Ordering::Relaxed);
        
        count
    }
    
    /// 健康检查
    /// 
    /// # 返回
    /// 健康连接数量
    pub fn health_check(&self) -> usize {
        let connections = self.connections.read().unwrap();
        
        let mut healthy = 0;
        for conn in connections.values() {
            let state = conn.get_state();
            if state == ConnectionState::Connected {
                healthy += 1;
            }
        }
        
        healthy
    }
    
    /// 更新统计信息
    pub fn update_stats(&self) {
        let connections = self.connections.read().unwrap();
        
        let mut total_bytes_sent = 0u64;
        let mut total_bytes_received = 0u64;
        let mut total_packets = 0u64;
        let mut lost_packets = 0u64;
        let mut retransmitted = 0u64;
        let mut total_rtt = 0u64;
        let mut rtt_count = 0u64;
        
        for conn in connections.values() {
            let stats = conn.stats();
            total_bytes_sent += stats.bytes_sent.load(Ordering::Relaxed);
            total_bytes_received += stats.bytes_received.load(Ordering::Relaxed);
            total_packets += stats.packets_sent.load(Ordering::Relaxed) 
                          + stats.packets_received.load(Ordering::Relaxed);
            lost_packets += stats.packets_lost.load(Ordering::Relaxed);
            retransmitted += stats.packets_retransmitted.load(Ordering::Relaxed);
            
            let rtt = stats.rtt_ms.load(Ordering::Relaxed);
            if rtt > 0 {
                total_rtt += rtt;
                rtt_count += 1;
            }
        }
        
        self.stats.total_bytes_sent.store(total_bytes_sent, Ordering::Relaxed);
        self.stats.total_bytes_received.store(total_bytes_received, Ordering::Relaxed);
        self.stats.total_packets.store(total_packets, Ordering::Relaxed);
        self.stats.lost_packets.store(lost_packets, Ordering::Relaxed);
        self.stats.retransmitted_packets.store(retransmitted, Ordering::Relaxed);
        
        if rtt_count > 0 {
            self.stats.avg_rtt_ms.store(total_rtt / rtt_count, Ordering::Relaxed);
        }
    }
    
    /// 获取统计信息
    pub fn stats(&self) -> &ConnectionStats {
        &self.stats
    }
    
    /// 启动连接管理器
    pub fn start(&self) {
        self.running.store(true, Ordering::Relaxed);
    }
    
    /// 停止连接管理器
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
        
        // 关闭所有连接
        let connections = self.connections.read().unwrap();
        for conn in connections.values() {
            conn.close();
        }
    }
    
    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
    
    /// 迁移连接
    /// 
    /// # 参数
    /// - `conn_id`: 连接 ID
    /// - `new_addr`: 新地址
    /// 
    /// # 返回
    /// 是否成功迁移
    pub fn migrate_connection(&self, conn_id: &ConnectionId, new_addr: SocketAddr) -> bool {
        // 获取连接
        let conn = {
            let connections = self.connections.read().unwrap();
            connections.get(conn_id).cloned()
        };
        
        if let Some(conn) = conn {
            let old_addr = conn.remote_addr;
            
            // 更新地址映射
            {
                let mut addr_map = self.addr_to_conn.write().unwrap();
                addr_map.remove(&old_addr);
                addr_map.insert(new_addr, conn_id.clone());
            }
            
            // 注意：实际实现需要更新连接的远程地址
            // 这里只是演示
            
            true
        } else {
            false
        }
    }
    
    /// 获取连接池状态
    /// 
    /// # 返回
    /// 连接池状态信息
    pub fn get_pool_status(&self) -> ConnectionPoolStatus {
        let connections = self.connections.read().unwrap();
        
        let mut active = 0;
        let mut idle = 0;
        let mut handshaking = 0;
        let mut closing = 0;
        
        for conn in connections.values() {
            match conn.get_state() {
                ConnectionState::Connected => active += 1,
                ConnectionState::Initial | ConnectionState::Handshaking => handshaking += 1,
                ConnectionState::Closing => closing += 1,
                _ => idle += 1,
            }
        }
        
        ConnectionPoolStatus {
            total: connections.len(),
            active,
            idle,
            handshaking,
            closing,
            max_connections: self.config.max_connections,
        }
    }
}

/// 连接池状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolStatus {
    /// 总连接数
    pub total: usize,
    /// 活跃连接数
    pub active: usize,
    /// 空闲连接数
    pub idle: usize,
    /// 握手中的连接数
    pub handshaking: usize,
    /// 正在关闭的连接数
    pub closing: usize,
    /// 最大连接数
    pub max_connections: usize,
}

/// 为 ConnectionManager 实现 Default trait
impl Default for ConnectionManager {
    fn default() -> Self {
        Self::with_defaults(false)
    }
}

/// 单元测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试连接管理器创建
    #[test]
    fn test_connection_manager() {
        let manager = ConnectionManager::with_defaults(false);
        
        assert_eq!(manager.connection_count(), 0);
    }
    
    /// 测试连接创建和获取
    #[test]
    fn test_create_get_connection() {
        let manager = ConnectionManager::with_defaults(false);
        
        let local: SocketAddr = "127.0.0.1:443".parse().unwrap();
        let remote: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        
        // 创建连接
        let conn = manager.create_connection(local, remote).unwrap();
        
        // 获取连接
        let retrieved = manager.get_connection(&conn.connection_id);
        assert!(retrieved.is_some());
        
        // 通过地址获取
        let by_addr = manager.get_connection_by_addr(&remote);
        assert!(by_addr.is_some());
    }
    
    /// 测试连接关闭
    #[test]
    fn test_close_connection() {
        let manager = ConnectionManager::with_defaults(false);
        
        let local: SocketAddr = "127.0.0.1:443".parse().unwrap();
        let remote: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        
        let conn = manager.create_connection(local, remote).unwrap();
        
        // 关闭连接
        let closed = manager.close_connection(&conn.connection_id);
        assert!(closed);
        
        // 验证已移除
        let retrieved = manager.get_connection(&conn.connection_id);
        assert!(retrieved.is_none());
    }
    
    /// 测试连接池状态
    #[test]
    fn test_pool_status() {
        let manager = ConnectionManager::with_defaults(false);
        
        let local: SocketAddr = "127.0.0.1:443".parse().unwrap();
        let remote: SocketAddr = "127.0.0.1:12345".parse().unwrap();
        
        manager.create_connection(local, remote).unwrap();
        
        let status = manager.get_pool_status();
        assert_eq!(status.total, 1);
    }
}
