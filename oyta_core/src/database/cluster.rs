//! 数据库集群支持模块
//!
//! 提供多数据库节点的集群管理能力
//! 支持：健康检查、故障转移、连接池监控
//! 适用于高可用、分布式数据库场景

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

use super::pool_trait::DatabasePool;

/// 数据库节点状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    /// 节点正常
    Healthy,
    /// 节点不健康
    Unhealthy,
    /// 节点正在恢复中
    Recovering,
    /// 节点已下线
    Offline,
}

impl std::fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeStatus::Healthy => write!(f, "健康"),
            NodeStatus::Unhealthy => write!(f, "不健康"),
            NodeStatus::Recovering => write!(f, "恢复中"),
            NodeStatus::Offline => write!(f, "已下线"),
        }
    }
}

/// 数据库节点信息
#[derive(Debug, Clone)]
pub struct DatabaseNode {
    /// 节点 ID
    pub id: String,
    /// 节点名称
    pub name: String,
    /// 节点角色（master/slave）
    pub role: NodeRole,
    /// 节点权重（用于负载均衡）
    pub weight: u32,
    /// 节点优先级（用于故障转移）
    pub priority: u32,
    /// 节点状态
    pub status: NodeStatus,
    /// 最后健康检查时间
    pub last_check: Option<Instant>,
    /// 连续失败次数
    pub consecutive_failures: u32,
    /// 最后错误信息
    pub last_error: Option<String>,
    /// 平均响应时间（毫秒）
    pub avg_response_time: f64,
    /// 响应时间历史记录
    pub response_times: Vec<f64>,
}

impl DatabaseNode {
    /// 创建新的数据库节点
    pub fn new(id: &str, name: &str, role: NodeRole) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            role,
            weight: 1,
            priority: 1,
            status: NodeStatus::Healthy,
            last_check: None,
            consecutive_failures: 0,
            last_error: None,
            avg_response_time: 0.0,
            response_times: Vec::new(),
        }
    }

    /// 设置权重
    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// 记录健康检查结果
    pub fn record_check(&mut self, healthy: bool, response_time: f64, error: Option<String>) {
        self.last_check = Some(Instant::now());

        if healthy {
            self.status = NodeStatus::Healthy;
            self.consecutive_failures = 0;
            self.last_error = None;

            // 更新响应时间统计
            self.response_times.push(response_time);
            if self.response_times.len() > 10 {
                self.response_times.remove(0);
            }
            self.avg_response_time = self.response_times.iter().sum::<f64>() / self.response_times.len() as f64;
        } else {
            self.consecutive_failures += 1;
            self.last_error = error;

            // 连续失败 3 次标记为不健康
            if self.consecutive_failures >= 3 {
                self.status = NodeStatus::Unhealthy;
            }
        }
    }

    /// 判断节点是否可用
    pub fn is_available(&self) -> bool {
        matches!(self.status, NodeStatus::Healthy | NodeStatus::Recovering)
    }
}

/// 节点角色
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    /// 主节点
    Master,
    /// 从节点
    Slave,
}

impl std::fmt::Display for NodeRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeRole::Master => write!(f, "主库"),
            NodeRole::Slave => write!(f, "从库"),
        }
    }
}

/// 数据库集群管理器
///
/// 管理多个数据库节点，提供健康检查和故障转移
pub struct DatabaseCluster {
    /// 集群名称
    pub name: String,
    /// 所有节点（节点ID → 节点信息）
    nodes: Arc<RwLock<HashMap<String, DatabaseNode>>>,
    /// 节点连接池（节点ID → 连接池）
    pools: Arc<RwLock<HashMap<String, Arc<dyn DatabasePool>>>>,
    /// 当前主节点 ID
    current_master: Arc<RwLock<Option<String>>>,
    /// 健康检查配置
    health_check_config: HealthCheckConfig,
    /// 健康检查任务句柄
    health_check_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

/// 健康检查配置
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// 检查间隔（秒）
    pub interval_secs: u64,
    /// 超时时间（秒）
    pub timeout_secs: u64,
    /// 连续失败次数阈值
    pub failure_threshold: u32,
    /// 恢复检查间隔（秒）
    pub recovery_interval_secs: u64,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval_secs: 10,
            timeout_secs: 5,
            failure_threshold: 3,
            recovery_interval_secs: 30,
        }
    }
}

impl DatabaseCluster {
    /// 创建新的数据库集群
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            nodes: Arc::new(RwLock::new(HashMap::new())),
            pools: Arc::new(RwLock::new(HashMap::new())),
            current_master: Arc::new(RwLock::new(None)),
            health_check_config: HealthCheckConfig::default(),
            health_check_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// 设置健康检查配置
    pub fn with_health_check_config(mut self, config: HealthCheckConfig) -> Self {
        self.health_check_config = config;
        self
    }

    /// 添加节点
    ///
    /// # 参数
    /// - `node`: 节点信息
    /// - `pool`: 节点连接池
    pub async fn add_node(&self, node: DatabaseNode, pool: Arc<dyn DatabasePool>) {
        let node_id = node.id.clone();
        let node_id_for_log = node_id.clone();

        // 如果是主节点，设置当前主节点
        if node.role == NodeRole::Master {
            let mut master = self.current_master.write().await;
            *master = Some(node_id.clone());
        }

        // 添加节点信息
        self.nodes.write().await.insert(node_id.clone(), node);

        // 添加连接池
        self.pools.write().await.insert(node_id, pool);

        tracing::info!("集群 [{}] 添加节点: {}", self.name, node_id_for_log);
    }

    /// 移除节点
    pub async fn remove_node(&self, node_id: &str) {
        self.nodes.write().await.remove(node_id);
        self.pools.write().await.remove(node_id);

        tracing::info!("集群 [{}] 移除节点: {}", self.name, node_id);
    }

    /// 获取节点信息
    pub async fn get_node(&self, node_id: &str) -> Option<DatabaseNode> {
        self.nodes.read().await.get(node_id).cloned()
    }

    /// 获取节点连接池
    pub async fn get_pool(&self, node_id: &str) -> Option<Arc<dyn DatabasePool>> {
        self.pools.read().await.get(node_id).cloned()
    }

    /// 获取当前主节点
    pub async fn get_master(&self) -> Option<(DatabaseNode, Arc<dyn DatabasePool>)> {
        let master_id = self.current_master.read().await.clone()?;

        let node = self.nodes.read().await.get(&master_id)?.clone();
        let pool = self.pools.read().await.get(&master_id)?.clone();

        Some((node, pool))
    }

    /// 获取所有可用的从节点
    pub async fn get_available_slaves(&self) -> Vec<(DatabaseNode, Arc<dyn DatabasePool>)> {
        let nodes = self.nodes.read().await;
        let pools = self.pools.read().await;

        nodes
            .values()
            .filter(|n| n.role == NodeRole::Slave && n.is_available())
            .filter_map(|n| {
                pools.get(&n.id).map(|p| (n.clone(), p.clone()))
            })
            .collect()
    }

    /// 获取所有节点状态
    pub async fn get_all_nodes_status(&self) -> HashMap<String, NodeStatus> {
        self.nodes
            .read()
            .await
            .iter()
            .map(|(id, node)| (id.clone(), node.status))
            .collect()
    }

    /// 执行健康检查
    pub async fn health_check(&self) {
        let nodes = self.nodes.read().await;
        let pools = self.pools.read().await;

        let node_ids: Vec<String> = nodes.keys().cloned().collect();

        for node_id in node_ids {
            let pool = pools.get(&node_id).cloned();
            let node_info = nodes.get(&node_id).cloned();

            if let (Some(pool), Some(_node_info)) = (pool, node_info) {
                let start = Instant::now();

                // 执行健康检查
                let healthy = tokio::time::timeout(
                    Duration::from_secs(self.health_check_config.timeout_secs),
                    pool.is_healthy(),
                )
                .await
                .unwrap_or(false);

                let response_time = start.elapsed().as_millis() as f64;

                // 释放读锁
                drop(nodes);
                drop(pools);

                let mut nodes_mut = self.nodes.write().await;
                if let Some(node) = nodes_mut.get_mut(&node_id) {
                    let error = if !healthy {
                        Some("健康检查失败".to_string())
                    } else {
                        None
                    };
                    node.record_check(healthy, response_time, error);

                    tracing::debug!(
                        "集群 [{}] 节点 [{}] 健康检查: {} (响应时间: {:.2}ms)",
                        self.name,
                        node_id,
                        if healthy { "通过" } else { "失败" },
                        response_time
                    );
                }
                return;
            }
        }
    }

    /// 启动健康检查任务
    pub async fn start_health_check(&self) {
        let nodes = self.nodes.clone();
        let pools = self.pools.clone();
        let config = self.health_check_config.clone();
        let cluster_name = self.name.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(config.interval_secs));

            loop {
                interval.tick().await;

                let nodes_guard = nodes.read().await;
                let pools_guard = pools.read().await;

                let node_ids: Vec<String> = nodes_guard.keys().cloned().collect();

                for node_id in node_ids {
                    let pool = pools_guard.get(&node_id).cloned();

                    if let Some(pool) = pool {
                        let start = Instant::now();

                        let healthy = tokio::time::timeout(
                            Duration::from_secs(config.timeout_secs),
                            pool.is_healthy(),
                        )
                        .await
                        .unwrap_or(false);

                        let response_time = start.elapsed().as_millis() as f64;

                        drop(nodes_guard);
                        drop(pools_guard);

                        let mut nodes_mut = nodes.write().await;
                        if let Some(n) = nodes_mut.get_mut(&node_id) {
                            let error = if !healthy {
                                Some("健康检查失败".to_string())
                            } else {
                                None
                            };
                            n.record_check(healthy, response_time, error);

                            tracing::trace!(
                                "集群 [{}] 节点 [{}] 状态: {}",
                                cluster_name,
                                node_id,
                                n.status
                            );
                        }
                        break;
                    }
                }
            }
        });

        let mut handle_guard = self.health_check_handle.lock().await;
        *handle_guard = Some(handle);

        tracing::info!("集群 [{}] 健康检查任务已启动", self.name);
    }

    /// 停止健康检查任务
    pub async fn stop_health_check(&self) {
        let mut handle_guard = self.health_check_handle.lock().await;
        if let Some(handle) = handle_guard.take() {
            handle.abort();
            tracing::info!("集群 [{}] 健康检查任务已停止", self.name);
        }
    }

    /// 故障转移
    ///
    /// 当主节点故障时，自动切换到优先级最高的从节点
    pub async fn failover(&self) -> Result<String> {
        let mut master = self.current_master.write().await;

        // 获取当前主节点
        let old_master_id = master.clone();

        // 标记旧主节点为不健康
        if let Some(old_id) = &old_master_id {
            let mut nodes = self.nodes.write().await;
            if let Some(node) = nodes.get_mut(old_id) {
                node.status = NodeStatus::Unhealthy;
                tracing::warn!(
                    "集群 [{}] 主节点 [{}] 故障，开始故障转移",
                    self.name,
                    old_id
                );
            }
        }

        // 选择新的主节点（优先级最高的可用从节点）
        let nodes = self.nodes.read().await;
        let new_master = nodes
            .values()
            .filter(|n| n.role == NodeRole::Slave && n.is_available())
            .max_by_key(|n| n.priority);

        if let Some(new_master_node) = new_master {
            let new_master_id = new_master_node.id.clone();
            *master = Some(new_master_id.clone());

            tracing::info!(
                "集群 [{}] 故障转移完成: {} → {}",
                self.name,
                old_master_id.unwrap_or_default(),
                new_master_id
            );

            Ok(new_master_id)
        } else {
            anyhow::bail!("集群 [{}] 没有可用的从节点进行故障转移", self.name);
        }
    }

    /// 获取集群统计信息
    pub async fn stats(&self) -> ClusterStats {
        let nodes = self.nodes.read().await;

        let total_nodes = nodes.len();
        let healthy_nodes = nodes.values().filter(|n| n.status == NodeStatus::Healthy).count();
        let unhealthy_nodes = nodes.values().filter(|n| n.status == NodeStatus::Unhealthy).count();
        let master_count = nodes.values().filter(|n| n.role == NodeRole::Master).count();
        let slave_count = nodes.values().filter(|n| n.role == NodeRole::Slave).count();

        let avg_response_time = nodes
            .values()
            .map(|n| n.avg_response_time)
            .sum::<f64>()
            / if total_nodes > 0 { total_nodes as f64 } else { 1.0 };

        ClusterStats {
            name: self.name.clone(),
            total_nodes,
            healthy_nodes,
            unhealthy_nodes,
            master_count,
            slave_count,
            avg_response_time,
        }
    }
}

/// 集群统计信息
#[derive(Debug, Clone)]
pub struct ClusterStats {
    /// 集群名称
    pub name: String,
    /// 总节点数
    pub total_nodes: usize,
    /// 健康节点数
    pub healthy_nodes: usize,
    /// 不健康节点数
    pub unhealthy_nodes: usize,
    /// 主节点数
    pub master_count: usize,
    /// 从节点数
    pub slave_count: usize,
    /// 平均响应时间（毫秒）
    pub avg_response_time: f64,
}

impl std::fmt::Display for ClusterStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "集群 [{}]: 总节点={}, 健康={}, 不健康={}, 主库={}, 从库={}, 平均响应={:.2}ms",
            self.name,
            self.total_nodes,
            self.healthy_nodes,
            self.unhealthy_nodes,
            self.master_count,
            self.slave_count,
            self.avg_response_time
        )
    }
}

/// 连接池监控器
///
/// 监控连接池的使用情况
pub struct PoolMonitor {
    /// 监控的连接池
    pool: Arc<dyn DatabasePool>,
    /// 监控名称
    name: String,
    /// 请求计数
    request_count: Arc<RwLock<u64>>,
    /// 错误计数
    error_count: Arc<RwLock<u64>>,
    /// 总响应时间
    total_response_time: Arc<RwLock<f64>>,
}

impl PoolMonitor {
    /// 创建新的监控器
    pub fn new(name: &str, pool: Arc<dyn DatabasePool>) -> Self {
        Self {
            pool,
            name: name.to_string(),
            request_count: Arc::new(RwLock::new(0)),
            error_count: Arc::new(RwLock::new(0)),
            total_response_time: Arc::new(RwLock::new(0.0)),
        }
    }

    /// 记录请求
    pub async fn record_request(&self, duration_ms: f64, success: bool) {
        let mut count = self.request_count.write().await;
        *count += 1;

        let mut time = self.total_response_time.write().await;
        *time += duration_ms;

        if !success {
            let mut errors = self.error_count.write().await;
            *errors += 1;
        }
    }

    /// 获取监控统计
    pub async fn stats(&self) -> MonitorStats {
        let count = *self.request_count.read().await;
        let errors = *self.error_count.read().await;
        let total_time = *self.total_response_time.read().await;

        let (active, idle, max) = self.pool.pool_status();

        MonitorStats {
            name: self.name.clone(),
            total_requests: count,
            error_requests: errors,
            avg_response_time: if count > 0 { total_time / count as f64 } else { 0.0 },
            active_connections: active,
            idle_connections: idle,
            max_connections: max,
        }
    }
}

/// 监控统计信息
#[derive(Debug, Clone)]
pub struct MonitorStats {
    /// 监控名称
    pub name: String,
    /// 总请求数
    pub total_requests: u64,
    /// 错误请求数
    pub error_requests: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time: f64,
    /// 活跃连接数
    pub active_connections: u32,
    /// 空闲连接数
    pub idle_connections: u32,
    /// 最大连接数
    pub max_connections: u32,
}

impl std::fmt::Display for MonitorStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] 请求: {}, 错误: {}, 平均响应: {:.2}ms, 连接: {}/{}",
            self.name,
            self.total_requests,
            self.error_requests,
            self.avg_response_time,
            self.active_connections,
            self.max_connections
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_status() {
        let mut node = DatabaseNode::new("node1", "主库", NodeRole::Master);
        assert_eq!(node.status, NodeStatus::Healthy);

        // 模拟健康检查失败
        node.record_check(false, 100.0, Some("连接超时".to_string()));
        assert_eq!(node.consecutive_failures, 1);
        assert_eq!(node.status, NodeStatus::Healthy); // 还未达到阈值

        // 继续失败
        node.record_check(false, 100.0, Some("连接超时".to_string()));
        node.record_check(false, 100.0, Some("连接超时".to_string()));
        assert_eq!(node.status, NodeStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_cluster_stats() {
        let cluster = DatabaseCluster::new("test-cluster");

        let stats = cluster.stats().await;
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(stats.healthy_nodes, 0);
    }
}
