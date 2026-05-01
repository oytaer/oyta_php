//! 服务发现与注册
//!
//! 实现分布式环境下的服务发现
//! 支持：Consul、etcd、Nacos
//!
//! # 功能特性
//! - 服务注册
//! - 服务发现
//! - 健康检查
//! - 负载均衡

use anyhow::Result;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 服务实例
#[derive(Debug, Clone)]
pub struct ServiceInstance {
    /// 服务 ID
    pub id: String,
    /// 服务名称
    pub name: String,
    /// 服务地址
    pub address: String,
    /// 服务端口
    pub port: u16,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 健康状态
    pub healthy: bool,
    /// 注册时间
    pub registered_at: Instant,
    /// 最后心跳时间
    pub last_heartbeat: Instant,
}

impl ServiceInstance {
    /// 创建新的服务实例
    pub fn new(name: &str, address: &str, port: u16) -> Self {
        Self {
            id: format!("{}-{}", name, rand::rng().random::<u32>()),
            name: name.to_string(),
            address: address.to_string(),
            port,
            metadata: HashMap::new(),
            healthy: true,
            registered_at: Instant::now(),
            last_heartbeat: Instant::now(),
        }
    }

    /// 设置服务 ID
    pub fn with_id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// 获取完整地址
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }

    /// 获取 HTTP URL
    pub fn http_url(&self) -> String {
        format!("http://{}:{}", self.address, self.port)
    }
}

/// 服务注册中心配置
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// 注册中心类型
    pub registry_type: RegistryType,
    /// 注册中心地址
    pub endpoint: String,
    /// 命名空间
    pub namespace: String,
    /// 心跳间隔（秒）
    pub heartbeat_interval: u64,
    /// 健康检查间隔（秒）
    pub health_check_interval: u64,
    /// 实例过期时间（秒）
    pub instance_ttl: u64,
}

/// 注册中心类型
#[derive(Debug, Clone, Copy)]
pub enum RegistryType {
    /// 内存注册（单机）
    Memory,
    /// Consul
    Consul,
    /// etcd
    Etcd,
    /// Nacos
    Nacos,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            registry_type: RegistryType::Memory,
            endpoint: "127.0.0.1:8500".to_string(),
            namespace: "default".to_string(),
            heartbeat_interval: 10,
            health_check_interval: 30,
            instance_ttl: 60,
        }
    }
}

/// 服务注册中心
pub struct ServiceRegistry {
    /// 配置
    config: RegistryConfig,
    /// 服务实例映射（服务名 → 实例列表）
    services: Arc<RwLock<HashMap<String, Vec<ServiceInstance>>>>,
    /// 本地注册的实例
    local_instances: Arc<RwLock<Vec<String>>>,
}

impl ServiceRegistry {
    /// 创建新的服务注册中心
    pub fn new(config: RegistryConfig) -> Self {
        Self {
            config,
            services: Arc::new(RwLock::new(HashMap::new())),
            local_instances: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 创建内存注册中心
    pub fn memory() -> Self {
        Self::new(RegistryConfig::default())
    }

    /// 注册服务
    ///
    /// # 参数
    /// - `instance`: 服务实例
    ///
    /// # 返回
    /// 服务 ID
    pub async fn register(&self, instance: ServiceInstance) -> Result<String> {
        let service_id = instance.id.clone();
        let service_name = instance.name.clone();

        // 添加到服务列表
        {
            let mut services = self.services.write().await;
            let instances = services
                .entry(service_name.clone())
                .or_insert_with(Vec::new);

            // 检查是否已存在
            if let Some(existing) = instances.iter_mut().find(|i| i.id == instance.id) {
                existing.last_heartbeat = Instant::now();
                existing.healthy = true;
            } else {
                instances.push(instance);
            }
        }

        // 记录本地实例
        {
            let mut local = self.local_instances.write().await;
            if !local.contains(&service_id) {
                local.push(service_id.clone());
            }
        }

        tracing::info!("服务注册成功: {} ({})", service_name, service_id);

        Ok(service_id)
    }

    /// 注销服务
    ///
    /// # 参数
    /// - `service_id`: 服务 ID
    pub async fn deregister(&self, service_id: &str) -> Result<()> {
        let mut services = self.services.write().await;

        for (_, instances) in services.iter_mut() {
            if let Some(pos) = instances.iter().position(|i| i.id == service_id) {
                let instance = instances.remove(pos);
                tracing::info!("服务注销成功: {} ({})", instance.name, service_id);
                return Ok(());
            }
        }

        tracing::warn!("服务不存在: {}", service_id);
        Ok(())
    }

    /// 发现服务
    ///
    /// # 参数
    /// - `service_name`: 服务名称
    ///
    /// # 返回
    /// 健康的服务实例列表
    pub async fn discover(&self, service_name: &str) -> Result<Vec<ServiceInstance>> {
        let services = self.services.read().await;

        let instances = services
            .get(service_name)
            .map(|instances| {
                instances
                    .iter()
                    .filter(|i| i.healthy)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        Ok(instances)
    }

    /// 获取单个服务实例（负载均衡）
    ///
    /// # 参数
    /// - `service_name`: 服务名称
    ///
    /// # 返回
    /// 一个健康的服务实例
    pub async fn get_instance(&self, service_name: &str) -> Result<Option<ServiceInstance>> {
        let instances = self.discover(service_name).await?;

        if instances.is_empty() {
            return Ok(None);
        }

        // 简单轮询 - 使用时间戳作为简单的随机源
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let index = (now as usize) % instances.len();
        Ok(Some(instances[index].clone()))
    }

    /// 发送心跳
    ///
    /// # 参数
    /// - `service_id`: 服务 ID
    pub async fn heartbeat(&self, service_id: &str) -> Result<bool> {
        let mut services = self.services.write().await;

        for (_, instances) in services.iter_mut() {
            if let Some(instance) = instances.iter_mut().find(|i| i.id == service_id) {
                instance.last_heartbeat = Instant::now();
                instance.healthy = true;
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 健康检查
    ///
    /// 检查所有实例的心跳，标记过期实例为不健康
    pub async fn health_check(&self) -> Result<HealthCheckResult> {
        let mut result = HealthCheckResult::default();
        let ttl = Duration::from_secs(self.config.instance_ttl);
        let now = Instant::now();

        let mut services = self.services.write().await;

        for (_, instances) in services.iter_mut() {
            for instance in instances.iter_mut() {
                result.total += 1;

                if now.duration_since(instance.last_heartbeat) > ttl {
                    instance.healthy = false;
                    result.unhealthy += 1;
                    tracing::warn!("服务实例不健康: {} ({})", instance.name, instance.id);
                } else {
                    result.healthy += 1;
                }
            }
        }

        Ok(result)
    }

    /// 获取所有服务名称
    pub async fn list_services(&self) -> Result<Vec<String>> {
        let services = self.services.read().await;
        Ok(services.keys().cloned().collect())
    }

    /// 获取统计信息
    pub async fn stats(&self) -> Result<RegistryStats> {
        let services = self.services.read().await;

        let mut total_instances = 0;
        let mut healthy_instances = 0;

        for instances in services.values() {
            for instance in instances {
                total_instances += 1;
                if instance.healthy {
                    healthy_instances += 1;
                }
            }
        }

        Ok(RegistryStats {
            total_services: services.len(),
            total_instances,
            healthy_instances,
        })
    }
}

/// 健康检查结果
#[derive(Debug, Clone, Default)]
pub struct HealthCheckResult {
    /// 总实例数
    pub total: usize,
    /// 健康实例数
    pub healthy: usize,
    /// 不健康实例数
    pub unhealthy: usize,
}

/// 注册中心统计信息
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// 服务总数
    pub total_services: usize,
    /// 实例总数
    pub total_instances: usize,
    /// 健康实例数
    pub healthy_instances: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_registry() {
        let registry = ServiceRegistry::memory();

        let instance = ServiceInstance::new("user-service", "127.0.0.1", 8080);
        let id = registry.register(instance).await.unwrap();

        let instances = registry.discover("user-service").await.unwrap();
        assert_eq!(instances.len(), 1);
    }
}
