//! 负载均衡模块
//! 
//! 实现多种负载均衡算法，支持：
//! - 轮询
//! - 加权轮询
//! - 最少连接
//! - 随机
//! - 一致性哈希
//! - 加权最少连接

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::registry::ServiceInstance;

/// 后端服务器
/// 表示负载均衡的目标服务器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendServer {
    /// 服务器地址
    pub address: SocketAddr,
    /// 服务器权重
    pub weight: u32,
    /// 当前连接数
    pub connections: u32,
    /// 是否健康
    pub healthy: bool,
    /// 响应时间（毫秒）
    pub response_time_ms: u64,
    /// 最后请求时间
    pub last_request_time: u64,
    /// 服务器元数据
    pub metadata: HashMap<String, String>,
}

/// 负载均衡策略枚举
/// 定义可用的负载均衡算法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadBalanceStrategy {
    /// 轮询
    RoundRobin,
    /// 加权轮询
    WeightedRoundRobin,
    /// 最少连接
    LeastConnections,
    /// 加权最少连接
    WeightedLeastConnections,
    /// 随机
    Random,
    /// 加权随机
    WeightedRandom,
    /// 一致性哈希
    ConsistentHash,
    /// 响应时间优先
    LeastResponseTime,
    /// 源地址哈希
    SourceHash,
}

/// 为 LoadBalanceStrategy 实现默认值
impl Default for LoadBalanceStrategy {
    fn default() -> Self {
        // 默认使用轮询策略
        LoadBalanceStrategy::RoundRobin
    }
}

/// 负载均衡器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// 负载均衡策略
    pub strategy: LoadBalanceStrategy,
    /// 重试次数
    pub retry_count: u32,
    /// 重试延迟（毫秒）
    pub retry_delay_ms: u64,
    /// 连接超时（毫秒）
    pub connect_timeout_ms: u64,
    /// 请求超时（毫秒）
    pub request_timeout_ms: u64,
    /// 健康检查间隔（毫秒）
    pub health_check_interval_ms: u64,
    /// 是否启用健康检查
    pub enable_health_check: bool,
    /// 一致性哈希虚拟节点数
    pub hash_virtual_nodes: u32,
}

/// 为 LoadBalancerConfig 实现默认值
impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            // 默认轮询策略
            strategy: LoadBalanceStrategy::RoundRobin,
            // 默认重试3次
            retry_count: 3,
            // 默认重试延迟100毫秒
            retry_delay_ms: 100,
            // 默认连接超时5秒
            connect_timeout_ms: 5_000,
            // 默认请求超时30秒
            request_timeout_ms: 30_000,
            // 默认健康检查间隔10秒
            health_check_interval_ms: 10_000,
            // 默认启用健康检查
            enable_health_check: true,
            // 默认虚拟节点数
            hash_virtual_nodes: 150,
        }
    }
}

/// 负载均衡统计信息
#[derive(Debug, Default)]
pub struct LoadBalancerStats {
    /// 总请求数
    pub total_requests: AtomicU64,
    /// 成功请求数
    pub successful_requests: AtomicU64,
    /// 失败请求数
    pub failed_requests: AtomicU64,
    /// 总响应时间（毫秒）
    pub total_response_time_ms: AtomicU64,
    /// 当前活跃连接数
    pub active_connections: AtomicU64,
}

/// 负载均衡器
/// 根据配置的策略选择后端服务器
pub struct LoadBalancer {
    /// 后端服务器列表
    backends: RwLock<Vec<BackendServer>>,
    /// 配置
    config: LoadBalancerConfig,
    /// 统计信息
    stats: LoadBalancerStats,
    /// 轮询计数器
    round_robin_counter: AtomicU64,
    /// 加权轮询当前权重
    weighted_rr_current: RwLock<HashMap<SocketAddr, i64>>,
    /// 一致性哈希环
    hash_ring: RwLock<Vec<(u64, SocketAddr)>>,
}

/// 为 LoadBalancer 实现相关方法
impl LoadBalancer {
    /// 创建新的负载均衡器
    /// 
    /// # 参数
    /// - `config`: 负载均衡器配置
    /// 
    /// # 返回
    /// 新的负载均衡器实例
    pub fn new(config: LoadBalancerConfig) -> Self {
        Self {
            // 初始化空的后端列表
            backends: RwLock::new(Vec::new()),
            // 存储配置
            config,
            // 初始化统计信息
            stats: LoadBalancerStats::default(),
            // 初始化轮询计数器
            round_robin_counter: AtomicU64::new(0),
            // 初始化加权轮询状态
            weighted_rr_current: RwLock::new(HashMap::new()),
            // 初始化哈希环
            hash_ring: RwLock::new(Vec::new()),
        }
    }
    
    /// 使用默认配置创建负载均衡器
    pub fn with_defaults() -> Self {
        Self::new(LoadBalancerConfig::default())
    }
    
    /// 添加后端服务器
    /// 
    /// # 参数
    /// - `backend`: 后端服务器
    pub fn add_backend(&self, backend: BackendServer) {
        // 获取写锁
        let mut backends = self.backends.write().unwrap();
        // 添加后端
        backends.push(backend);
        // 重建哈希环
        drop(backends);
        self.rebuild_hash_ring();
    }
    
    /// 移除后端服务器
    /// 
    /// # 参数
    /// - `address`: 服务器地址
    /// 
    /// # 返回
    /// 是否成功移除
    pub fn remove_backend(&self, address: &SocketAddr) -> bool {
        // 获取写锁
        let mut backends = self.backends.write().unwrap();
        let len_before = backends.len();
        backends.retain(|b| &b.address != address);
        let removed = len_before != backends.len();
        drop(backends);
        
        // 重建哈希环
        if removed {
            self.rebuild_hash_ring();
        }
        
        removed
    }
    
    /// 更新后端服务器状态
    /// 
    /// # 参数
    /// - `address`: 服务器地址
    /// - `healthy`: 健康状态
    /// 
    /// # 返回
    /// 是否成功更新
    pub fn update_backend_health(&self, address: &SocketAddr, healthy: bool) -> bool {
        let mut backends = self.backends.write().unwrap();
        for backend in backends.iter_mut() {
            if &backend.address == address {
                backend.healthy = healthy;
                return true;
            }
        }
        false
    }
    
    /// 更新后端服务器连接数
    /// 
    /// # 参数
    /// - `address`: 服务器地址
    /// - `delta`: 连接数变化量
    pub fn update_connections(&self, address: &SocketAddr, delta: i32) {
        let mut backends = self.backends.write().unwrap();
        for backend in backends.iter_mut() {
            if &backend.address == address {
                if delta > 0 {
                    backend.connections = backend.connections.saturating_add(delta as u32);
                } else {
                    backend.connections = backend.connections.saturating_sub((-delta) as u32);
                }
                break;
            }
        }
    }
    
    /// 获取所有后端服务器
    /// 
    /// # 返回
    /// 后端服务器列表
    pub fn get_backends(&self) -> Vec<BackendServer> {
        self.backends.read().unwrap().clone()
    }
    
    /// 获取健康的后端服务器
    /// 
    /// # 返回
    /// 健康的后端服务器列表
    pub fn get_healthy_backends(&self) -> Vec<BackendServer> {
        self.backends.read().unwrap()
            .iter()
            .filter(|b| b.healthy)
            .cloned()
            .collect()
    }
    
    /// 选择后端服务器
    /// 
    /// # 返回
    /// 选中的后端服务器（如果有）
    pub fn select(&self) -> Option<BackendServer> {
        self.select_with_key(None)
    }
    
    /// 使用键选择后端服务器（用于一致性哈希）
    /// 
    /// # 参数
    /// - `key`: 哈希键（可选）
    /// 
    /// # 返回
    /// 选中的后端服务器
    pub fn select_with_key(&self, key: Option<&[u8]>) -> Option<BackendServer> {
        // 获取健康的后端
        let healthy_backends: Vec<BackendServer> = self.get_healthy_backends();
        
        // 如果没有健康的后端，返回None
        if healthy_backends.is_empty() {
            return None;
        }
        
        // 根据策略选择后端
        let selected = match self.config.strategy {
            // 轮询策略
            LoadBalanceStrategy::RoundRobin => {
                self.select_round_robin(&healthy_backends)
            }
            // 加权轮询策略
            LoadBalanceStrategy::WeightedRoundRobin => {
                self.select_weighted_round_robin(&healthy_backends)
            }
            // 最少连接策略
            LoadBalanceStrategy::LeastConnections => {
                self.select_least_connections(&healthy_backends)
            }
            // 加权最少连接策略
            LoadBalanceStrategy::WeightedLeastConnections => {
                self.select_weighted_least_connections(&healthy_backends)
            }
            // 随机策略
            LoadBalanceStrategy::Random => {
                self.select_random(&healthy_backends)
            }
            // 加权随机策略
            LoadBalanceStrategy::WeightedRandom => {
                self.select_weighted_random(&healthy_backends)
            }
            // 一致性哈希策略
            LoadBalanceStrategy::ConsistentHash => {
                self.select_consistent_hash(&healthy_backends, key)
            }
            // 响应时间优先策略
            LoadBalanceStrategy::LeastResponseTime => {
                self.select_least_response_time(&healthy_backends)
            }
            // 源地址哈希策略
            LoadBalanceStrategy::SourceHash => {
                self.select_source_hash(&healthy_backends, key)
            }
        };
        
        // 更新统计
        if selected.is_some() {
            self.stats.total_requests.fetch_add(1, Ordering::Relaxed);
        }
        
        selected
    }
    
    /// 轮询选择
    fn select_round_robin(&self, backends: &[BackendServer]) -> Option<BackendServer> {
        if backends.is_empty() {
            return None;
        }
        
        // 获取当前索引
        let index = self.round_robin_counter.fetch_add(1, Ordering::Relaxed);
        // 返回对应索引的后端
        Some(backends[(index as usize) % backends.len()].clone())
    }
    
    /// 加权轮询选择
    fn select_weighted_round_robin(&self, backends: &[BackendServer]) -> Option<BackendServer> {
        if backends.is_empty() {
            return None;
        }
        
        // 计算总权重
        let total_weight: u32 = backends.iter().map(|b| b.weight).sum();
        if total_weight == 0 {
            return backends.first().cloned();
        }
        
        // 使用平滑加权轮询算法
        let mut current = self.weighted_rr_current.write().unwrap();
        
        // 初始化当前权重
        for backend in backends {
            current.entry(backend.address)
                .or_insert(backend.weight as i64);
        }
        
        // 找到最大当前权重的后端
        let mut selected_idx = 0;
        let mut max_weight = i64::MIN;
        
        for (i, backend) in backends.iter().enumerate() {
            if let Some(weight) = current.get(&backend.address) {
                if *weight > max_weight {
                    max_weight = *weight;
                    selected_idx = i;
                }
            }
        }
        
        // 减去总权重
        if let Some(weight) = current.get_mut(&backends[selected_idx].address) {
            *weight -= total_weight as i64;
        }
        
        // 所有后端加上自己的权重
        for backend in backends {
            if let Some(weight) = current.get_mut(&backend.address) {
                *weight += backend.weight as i64;
            }
        }
        
        Some(backends[selected_idx].clone())
    }
    
    /// 最少连接选择
    fn select_least_connections(&self, backends: &[BackendServer]) -> Option<BackendServer> {
        backends.iter()
            .min_by_key(|b| b.connections)
            .cloned()
    }
    
    /// 加权最少连接选择
    fn select_weighted_least_connections(&self, backends: &[BackendServer]) -> Option<BackendServer> {
        backends.iter()
            .min_by(|a, b| {
                // 计算连接数与权重的比率
                let ratio_a = a.connections as f64 / a.weight.max(1) as f64;
                let ratio_b = b.connections as f64 / b.weight.max(1) as f64;
                ratio_a.partial_cmp(&ratio_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }
    
    /// 随机选择
    fn select_random(&self, backends: &[BackendServer]) -> Option<BackendServer> {
        if backends.is_empty() {
            return None;
        }
        
        // 使用简单的时间戳作为随机种子
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_nanos() as usize;
        
        let index = now % backends.len();
        Some(backends[index].clone())
    }
    
    /// 加权随机选择
    fn select_weighted_random(&self, backends: &[BackendServer]) -> Option<BackendServer> {
        if backends.is_empty() {
            return None;
        }
        
        // 计算总权重
        let total_weight: u32 = backends.iter().map(|b| b.weight).sum();
        if total_weight == 0 {
            return backends.first().cloned();
        }
        
        // 使用时间戳作为随机数
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_nanos() as u32 % total_weight;
        
        // 根据权重选择
        let mut cumulative = 0u32;
        for backend in backends {
            cumulative += backend.weight;
            if now < cumulative {
                return Some(backend.clone());
            }
        }
        
        backends.last().cloned()
    }
    
    /// 一致性哈希选择
    fn select_consistent_hash(&self, backends: &[BackendServer], key: Option<&[u8]>) -> Option<BackendServer> {
        if backends.is_empty() {
            return None;
        }
        
        // 计算键的哈希值
        let hash = match key {
            Some(k) => Self::hash_key(k),
            None => {
                // 没有键时使用随机值
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(0))
                    .as_nanos() as u64;
                now
            }
        };
        
        // 从哈希环中查找
        let ring = self.hash_ring.read().unwrap();
        if ring.is_empty() {
            return backends.first().cloned();
        }
        
        // 二分查找第一个大于等于hash的节点
        let pos = ring.partition_point(|(h, _)| *h < hash);
        let (_, addr) = if pos < ring.len() {
            &ring[pos]
        } else {
            // 环形，回到第一个节点
            &ring[0]
        };
        
        // 查找对应的后端
        backends.iter()
            .find(|b| &b.address == addr)
            .cloned()
            .or_else(|| backends.first().cloned())
    }
    
    /// 响应时间优先选择
    fn select_least_response_time(&self, backends: &[BackendServer]) -> Option<BackendServer> {
        backends.iter()
            .min_by_key(|b| b.response_time_ms)
            .cloned()
    }
    
    /// 源地址哈希选择
    fn select_source_hash(&self, backends: &[BackendServer], key: Option<&[u8]>) -> Option<BackendServer> {
        if backends.is_empty() {
            return None;
        }
        
        // 计算哈希值
        let hash = match key {
            Some(k) => Self::hash_key(k),
            None => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(0))
                    .as_nanos() as u64;
                now
            }
        };
        
        // 选择后端
        let index = (hash as usize) % backends.len();
        Some(backends[index].clone())
    }
    
    /// 计算键的哈希值
    fn hash_key(key: &[u8]) -> u64 {
        // 使用简单的哈希算法
        let mut hash: u64 = 5381;
        for byte in key {
            hash = hash.wrapping_mul(33).wrapping_add(*byte as u64);
        }
        hash
    }
    
    /// 重建一致性哈希环
    fn rebuild_hash_ring(&self) {
        let backends = self.backends.read().unwrap();
        let mut ring = self.hash_ring.write().unwrap();
        ring.clear();
        
        // 为每个后端创建虚拟节点
        for backend in backends.iter() {
            for i in 0..self.config.hash_virtual_nodes {
                // 计算虚拟节点的哈希值
                let key = format!("{}:{}", backend.address, i);
                let hash = Self::hash_key(key.as_bytes());
                ring.push((hash, backend.address));
            }
        }
        
        // 排序哈希环
        ring.sort_by_key(|(h, _)| *h);
    }
    
    /// 记录请求结果
    /// 
    /// # 参数
    /// - `success`: 是否成功
    /// - `response_time_ms`: 响应时间（毫秒）
    pub fn record_result(&self, success: bool, response_time_ms: u64) {
        if success {
            self.stats.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.stats.failed_requests.fetch_add(1, Ordering::Relaxed);
        }
        self.stats.total_response_time_ms.fetch_add(response_time_ms, Ordering::Relaxed);
    }
    
    /// 获取统计信息
    /// 
    /// # 返回
    /// 统计信息引用
    pub fn stats(&self) -> &LoadBalancerStats {
        &self.stats
    }
    
    /// 从服务实例创建后端服务器
    /// 
    /// # 参数
    /// - `instances`: 服务实例列表
    pub fn from_instances(instances: Vec<ServiceInstance>) -> Self {
        let lb = Self::with_defaults();
        
        for instance in instances {
            let backend = BackendServer {
                address: instance.address,
                weight: instance.weight,
                connections: 0,
                healthy: instance.health == super::registry::ServiceHealth::Healthy,
                response_time_ms: 0,
                last_request_time: 0,
                metadata: instance.metadata,
            };
            lb.add_backend(backend);
        }
        
        lb
    }
}

/// 为 LoadBalancer 实现 Default trait
impl Default for LoadBalancer {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// 单元测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试添加后端
    #[test]
    fn test_add_backend() {
        let lb = LoadBalancer::with_defaults();
        
        let backend = BackendServer {
            address: "127.0.0.1:8080".parse().unwrap(),
            weight: 100,
            connections: 0,
            healthy: true,
            response_time_ms: 0,
            last_request_time: 0,
            metadata: HashMap::new(),
        };
        
        lb.add_backend(backend);
        
        let backends = lb.get_backends();
        assert_eq!(backends.len(), 1);
    }
    
    /// 测试轮询选择
    #[test]
    fn test_round_robin() {
        let lb = LoadBalancer::new(LoadBalancerConfig {
            strategy: LoadBalanceStrategy::RoundRobin,
            ..Default::default()
        });
        
        // 添加多个后端
        for port in [8080, 8081, 8082] {
            lb.add_backend(BackendServer {
                address: format!("127.0.0.1:{}", port).parse().unwrap(),
                weight: 100,
                connections: 0,
                healthy: true,
                response_time_ms: 0,
                last_request_time: 0,
                metadata: HashMap::new(),
            });
        }
        
        // 选择多次，验证轮询
        let first = lb.select().unwrap();
        let second = lb.select().unwrap();
        let third = lb.select().unwrap();
        
        // 应该按顺序选择不同的后端
        assert_ne!(first.address, second.address);
        assert_ne!(second.address, third.address);
    }
    
    /// 测试最少连接选择
    #[test]
    fn test_least_connections() {
        let lb = LoadBalancer::new(LoadBalancerConfig {
            strategy: LoadBalanceStrategy::LeastConnections,
            ..Default::default()
        });
        
        // 添加不同连接数的后端
        lb.add_backend(BackendServer {
            address: "127.0.0.1:8080".parse().unwrap(),
            weight: 100,
            connections: 10,
            healthy: true,
            response_time_ms: 0,
            last_request_time: 0,
            metadata: HashMap::new(),
        });
        
        lb.add_backend(BackendServer {
            address: "127.0.0.1:8081".parse().unwrap(),
            weight: 100,
            connections: 5,
            healthy: true,
            response_time_ms: 0,
            last_request_time: 0,
            metadata: HashMap::new(),
        });
        
        // 应该选择连接数最少的
        let selected = lb.select().unwrap();
        assert_eq!(selected.address, "127.0.0.1:8081".parse().unwrap());
    }
    
    /// 测试移除后端
    #[test]
    fn test_remove_backend() {
        let lb = LoadBalancer::with_defaults();
        
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        lb.add_backend(BackendServer {
            address: addr,
            weight: 100,
            connections: 0,
            healthy: true,
            response_time_ms: 0,
            last_request_time: 0,
            metadata: HashMap::new(),
        });
        
        let removed = lb.remove_backend(&addr);
        assert!(removed);
        
        let backends = lb.get_backends();
        assert!(backends.is_empty());
    }
}
