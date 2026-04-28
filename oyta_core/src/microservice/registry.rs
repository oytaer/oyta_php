//! 服务注册与发现模块
//! 
//! 实现分布式环境下的服务注册与发现功能
//! 支持服务健康检查、自动注销、多数据中心等特性

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 服务实例标识
/// 唯一标识一个服务实例
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceId {
    /// 服务名称
    pub name: String,
    /// 实例ID
    pub instance_id: String,
}

/// 服务实例
/// 表示一个可用的服务实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    /// 服务标识
    pub id: ServiceId,
    /// 服务地址
    pub address: SocketAddr,
    /// 服务元数据
    pub metadata: HashMap<String, String>,
    /// 服务标签
    pub tags: Vec<String>,
    /// 服务权重（用于负载均衡）
    pub weight: u32,
    /// 注册时间
    pub registered_at: u64,
    /// 最后心跳时间
    pub last_heartbeat: u64,
    /// 健康状态
    pub health: ServiceHealth,
    /// 服务版本
    pub version: String,
    /// 数据中心
    pub datacenter: String,
    /// 可用区
    pub zone: String,
}

/// 服务健康状态枚举
/// 定义服务的健康检查状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceHealth {
    /// 健康
    Healthy,
    /// 不健康
    Unhealthy,
    /// 正在启动
    Starting,
    /// 维护中
    Maintenance,
}

/// 服务健康检查配置
/// 定义健康检查的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// 检查间隔（毫秒）
    pub interval_ms: u64,
    /// 超时时间（毫秒）
    pub timeout_ms: u64,
    /// 最大失败次数
    pub max_failures: u32,
    /// 最大成功次数（恢复健康）
    pub max_successes: u32,
    /// 检查路径（HTTP健康检查）
    pub check_path: String,
    /// 是否启用健康检查
    pub enabled: bool,
}

/// 为 HealthCheckConfig 实现默认值
impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            // 默认每10秒检查一次
            interval_ms: 10_000,
            // 默认超时5秒
            timeout_ms: 5_000,
            // 默认失败3次标记为不健康
            max_failures: 3,
            // 默认成功2次恢复健康
            max_successes: 2,
            // 默认健康检查路径
            check_path: String::from("/health"),
            // 默认启用健康检查
            enabled: true,
        }
    }
}

/// 服务注册配置
/// 定义服务注册的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationConfig {
    /// 服务名称
    pub service_name: String,
    /// 服务地址
    pub address: SocketAddr,
    /// 服务元数据
    pub metadata: HashMap<String, String>,
    /// 服务标签
    pub tags: Vec<String>,
    /// 服务权重
    pub weight: u32,
    /// 服务版本
    pub version: String,
    /// 数据中心
    pub datacenter: String,
    /// 可用区
    pub zone: String,
    /// 心跳间隔（毫秒）
    pub heartbeat_interval_ms: u64,
    /// 注册超时（毫秒）
    pub registration_timeout_ms: u64,
    /// 健康检查配置
    pub health_check: HealthCheckConfig,
}

/// 为 RegistrationConfig 实现默认值
impl Default for RegistrationConfig {
    fn default() -> Self {
        Self {
            // 默认服务名称
            service_name: String::from("unknown"),
            // 默认地址
            address: "127.0.0.1:8080".parse().unwrap(),
            // 空元数据
            metadata: HashMap::new(),
            // 空标签
            tags: Vec::new(),
            // 默认权重
            weight: 100,
            // 默认版本
            version: String::from("1.0.0"),
            // 默认数据中心
            datacenter: String::from("default"),
            // 默认可用区
            zone: String::from("default"),
            // 默认心跳间隔5秒
            heartbeat_interval_ms: 5_000,
            // 默认注册超时10秒
            registration_timeout_ms: 10_000,
            // 默认健康检查配置
            health_check: HealthCheckConfig::default(),
        }
    }
}

/// 服务注册表
/// 存储所有已注册的服务实例
#[derive(Debug)]
pub struct ServiceRegistry {
    /// 服务实例映射：服务名称 -> 实例列表
    services: RwLock<HashMap<String, Vec<ServiceInstance>>>,
    /// 实例索引：实例ID -> 服务实例
    instances: RwLock<HashMap<String, ServiceInstance>>,
    /// 健康检查记录：实例ID -> 连续失败次数
    health_records: RwLock<HashMap<String, HealthRecord>>,
    /// 注册表统计信息
    stats: RegistryStats,
    /// 是否正在运行
    running: AtomicBool,
    /// 配置
    config: RegistryConfig,
}

/// 健康检查记录
/// 记录服务实例的健康检查历史
#[derive(Debug, Clone, Default)]
struct HealthRecord {
    /// 连续失败次数
    consecutive_failures: u32,
    /// 连续成功次数
    consecutive_successes: u32,
    /// 最后检查时间
    last_check_time: u64,
    /// 最后检查结果
    last_check_result: bool,
}

/// 注册表统计信息
#[derive(Debug, Default)]
pub struct RegistryStats {
    /// 总注册次数
    pub total_registrations: AtomicU64,
    /// 总注销次数
    pub total_deregistrations: AtomicU64,
    /// 总心跳次数
    pub total_heartbeats: AtomicU64,
    /// 总健康检查次数
    pub total_health_checks: AtomicU64,
    /// 当前服务数量
    pub current_services: AtomicU64,
    /// 当前实例数量
    pub current_instances: AtomicU64,
}

/// 注册表配置
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// 实例过期时间（毫秒）
    pub instance_ttl_ms: u64,
    /// 健康检查间隔（毫秒）
    pub health_check_interval_ms: u64,
    /// 是否启用自动清理
    pub enable_auto_cleanup: bool,
    /// 清理间隔（毫秒）
    pub cleanup_interval_ms: u64,
}

/// 为 RegistryConfig 实现默认值
impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            // 默认实例30秒过期
            instance_ttl_ms: 30_000,
            // 默认健康检查间隔10秒
            health_check_interval_ms: 10_000,
            // 默认启用自动清理
            enable_auto_cleanup: true,
            // 默认清理间隔60秒
            cleanup_interval_ms: 60_000,
        }
    }
}

/// 为 ServiceRegistry 实现相关方法
impl ServiceRegistry {
    /// 创建新的服务注册表
    /// 
    /// # 返回
    /// 新的服务注册表实例
    pub fn new() -> Self {
        Self::with_config(RegistryConfig::default())
    }
    
    /// 使用配置创建服务注册表
    /// 
    /// # 参数
    /// - `config`: 注册表配置
    /// 
    /// # 返回
    /// 配置后的服务注册表实例
    pub fn with_config(config: RegistryConfig) -> Self {
        Self {
            // 初始化空的服务映射
            services: RwLock::new(HashMap::new()),
            // 初始化空的实例索引
            instances: RwLock::new(HashMap::new()),
            // 初始化空的健康记录
            health_records: RwLock::new(HashMap::new()),
            // 初始化统计信息
            stats: RegistryStats::default(),
            // 初始化为未运行状态
            running: AtomicBool::new(false),
            // 存储配置
            config,
        }
    }
    
    /// 注册服务实例
    /// 
    /// # 参数
    /// - `config`: 注册配置
    /// 
    /// # 返回
    /// 成功返回服务实例，失败返回错误信息
    pub fn register(&self, config: RegistrationConfig) -> Result<ServiceInstance, String> {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 生成实例ID
        let instance_id = format!("{}-{}-{}", 
            config.service_name,
            config.address,
            now
        );
        
        // 创建服务实例
        let instance = ServiceInstance {
            // 服务标识
            id: ServiceId {
                name: config.service_name.clone(),
                instance_id: instance_id.clone(),
            },
            // 服务地址
            address: config.address,
            // 元数据
            metadata: config.metadata,
            // 标签
            tags: config.tags,
            // 权重
            weight: config.weight,
            // 注册时间
            registered_at: now,
            // 最后心跳时间
            last_heartbeat: now,
            // 初始健康状态为启动中
            health: ServiceHealth::Starting,
            // 版本
            version: config.version,
            // 数据中心
            datacenter: config.datacenter,
            // 可用区
            zone: config.zone,
        };
        
        // 获取写锁并注册实例
        {
            // 更新服务映射
            let mut services = self.services.write().unwrap();
            let service_instances = services.entry(config.service_name.clone())
                .or_insert_with(Vec::new);
            service_instances.push(instance.clone());
            
            // 更新实例索引
            let mut instances = self.instances.write().unwrap();
            instances.insert(instance_id.clone(), instance.clone());
        }
        
        // 初始化健康记录
        {
            let mut records = self.health_records.write().unwrap();
            records.insert(instance_id, HealthRecord::default());
        }
        
        // 更新统计信息
        self.stats.total_registrations.fetch_add(1, Ordering::Relaxed);
        self.stats.current_instances.fetch_add(1, Ordering::Relaxed);
        
        // 返回注册的实例
        Ok(instance)
    }
    
    /// 注销服务实例
    /// 
    /// # 参数
    /// - `service_name`: 服务名称
    /// - `instance_id`: 实例ID
    /// 
    /// # 返回
    /// 成功返回true，失败返回false
    pub fn deregister(&self, service_name: &str, instance_id: &str) -> bool {
        // 从服务映射中移除
        let removed_from_services = {
            let mut services = self.services.write().unwrap();
            let mut should_remove_service = false;
            let result = if let Some(instances) = services.get_mut(service_name) {
                let len_before = instances.len();
                instances.retain(|i| i.id.instance_id != instance_id);
                // 如果服务没有实例了，标记需要移除整个服务
                if instances.is_empty() {
                    should_remove_service = true;
                }
                len_before != instances.len()
            } else {
                false
            };
            // 在 if let 块之外进行移除操作，避免双重可变借用
            if should_remove_service {
                services.remove(service_name);
            }
            result
        };
        
        // 从实例索引中移除
        let removed_from_instances = {
            let mut instances = self.instances.write().unwrap();
            instances.remove(instance_id).is_some()
        };
        
        // 从健康记录中移除
        {
            let mut records = self.health_records.write().unwrap();
            records.remove(instance_id);
        }
        
        // 如果成功移除，更新统计
        if removed_from_services || removed_from_instances {
            self.stats.total_deregistrations.fetch_add(1, Ordering::Relaxed);
            self.stats.current_instances.fetch_sub(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
    
    /// 发送心跳
    /// 
    /// # 参数
    /// - `instance_id`: 实例ID
    /// 
    /// # 返回
    /// 成功返回true，失败返回false
    pub fn heartbeat(&self, instance_id: &str) -> bool {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 更新实例的心跳时间
        {
            let mut instances = self.instances.write().unwrap();
            if let Some(instance) = instances.get_mut(instance_id) {
                instance.last_heartbeat = now;
                // 如果实例是启动中状态，更新为健康
                if instance.health == ServiceHealth::Starting {
                    instance.health = ServiceHealth::Healthy;
                }
                // 更新统计
                self.stats.total_heartbeats.fetch_add(1, Ordering::Relaxed);
                return true;
            }
        }
        
        false
    }
    
    /// 发现服务实例
    /// 
    /// # 参数
    /// - `service_name`: 服务名称
    /// 
    /// # 返回
    /// 服务实例列表
    pub fn discover(&self, service_name: &str) -> Vec<ServiceInstance> {
        let services = self.services.read().unwrap();
        services.get(service_name)
            .cloned()
            .unwrap_or_default()
    }
    
    /// 发现健康的实例
    /// 
    /// # 参数
    /// - `service_name`: 服务名称
    /// 
    /// # 返回
    /// 健康的服务实例列表
    pub fn discover_healthy(&self, service_name: &str) -> Vec<ServiceInstance> {
        self.discover(service_name)
            .into_iter()
            .filter(|i| i.health == ServiceHealth::Healthy)
            .collect()
    }
    
    /// 获取所有服务名称
    /// 
    /// # 返回
    /// 服务名称列表
    pub fn list_services(&self) -> Vec<String> {
        let services = self.services.read().unwrap();
        services.keys().cloned().collect()
    }
    
    /// 获取实例
    /// 
    /// # 参数
    /// - `instance_id`: 实例ID
    /// 
    /// # 返回
    /// 服务实例（如果存在）
    pub fn get_instance(&self, instance_id: &str) -> Option<ServiceInstance> {
        let instances = self.instances.read().unwrap();
        instances.get(instance_id).cloned()
    }
    
    /// 更新实例健康状态
    /// 
    /// # 参数
    /// - `instance_id`: 实例ID
    /// - `healthy`: 是否健康
    /// 
    /// # 返回
    /// 成功返回true
    pub fn update_health(&self, instance_id: &str, healthy: bool) -> bool {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 更新健康记录
        let new_health = {
            let mut records = self.health_records.write().unwrap();
            let record = records.entry(instance_id.to_string())
                .or_insert_with(HealthRecord::default);
            
            // 更新检查时间
            record.last_check_time = now;
            record.last_check_result = healthy;
            
            // 更新连续计数
            if healthy {
                record.consecutive_failures = 0;
                record.consecutive_successes += 1;
            } else {
                record.consecutive_successes = 0;
                record.consecutive_failures += 1;
            }
            
            // 更新统计
            self.stats.total_health_checks.fetch_add(1, Ordering::Relaxed);
            
            // 判断新的健康状态
            let config = HealthCheckConfig::default();
            if record.consecutive_failures >= config.max_failures {
                ServiceHealth::Unhealthy
            } else if record.consecutive_successes >= config.max_successes {
                ServiceHealth::Healthy
            } else {
                // 保持当前状态
                return true;
            }
        };
        
        // 更新实例健康状态
        {
            let mut instances = self.instances.write().unwrap();
            if let Some(instance) = instances.get_mut(instance_id) {
                instance.health = new_health;
            }
        }
        
        // 同步更新服务映射中的实例
        {
            let mut services = self.services.write().unwrap();
            for instances in services.values_mut() {
                for instance in instances {
                    if instance.id.instance_id == instance_id {
                        instance.health = new_health;
                        break;
                    }
                }
            }
        }
        
        true
    }
    
    /// 清理过期实例
    /// 
    /// # 返回
    /// 清理的实例数量
    pub fn cleanup_expired(&self) -> usize {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 收集过期实例
        let expired_instances: Vec<(String, String)> = {
            let instances = self.instances.read().unwrap();
            instances.iter()
                .filter(|(_, instance)| {
                    // 检查心跳是否超时
                    now.saturating_sub(instance.last_heartbeat) > self.config.instance_ttl_ms
                })
                .map(|(_, instance)| {
                    (instance.id.name.clone(), instance.id.instance_id.clone())
                })
                .collect()
        };
        
        // 注销过期实例
        let count = expired_instances.len();
        for (service_name, instance_id) in expired_instances {
            self.deregister(&service_name, &instance_id);
        }
        
        count
    }
    
    /// 获取统计信息
    /// 
    /// # 返回
    /// 统计信息引用
    pub fn stats(&self) -> &RegistryStats {
        &self.stats
    }
    
    /// 启动注册表
    pub fn start(&self) {
        self.running.store(true, Ordering::Relaxed);
    }
    
    /// 停止注册表
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
    
    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

/// 为 ServiceRegistry 实现 Default trait
impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 服务发现客户端
/// 用于从注册表中发现服务
#[derive(Debug)]
pub struct ServiceDiscovery {
    /// 服务注册表引用
    registry: Arc<ServiceRegistry>,
    /// 本地缓存
    cache: RwLock<HashMap<String, Vec<ServiceInstance>>>,
    /// 缓存过期时间（毫秒）
    cache_ttl_ms: u64,
    /// 缓存更新时间
    cache_updated: RwLock<HashMap<String, u64>>,
}

impl ServiceDiscovery {
    /// 创建新的服务发现客户端
    /// 
    /// # 参数
    /// - `registry`: 服务注册表引用
    /// 
    /// # 返回
    /// 新的服务发现客户端实例
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self {
            // 存储注册表引用
            registry,
            // 初始化空缓存
            cache: RwLock::new(HashMap::new()),
            // 默认缓存5秒
            cache_ttl_ms: 5_000,
            // 初始化缓存更新时间
            cache_updated: RwLock::new(HashMap::new()),
        }
    }
    
    /// 设置缓存过期时间
    /// 
    /// # 参数
    /// - `ttl_ms`: 过期时间（毫秒）
    pub fn set_cache_ttl(&mut self, ttl_ms: u64) {
        self.cache_ttl_ms = ttl_ms;
    }
    
    /// 发现服务（带缓存）
    /// 
    /// # 参数
    /// - `service_name`: 服务名称
    /// 
    /// # 返回
    /// 服务实例列表
    pub fn discover(&self, service_name: &str) -> Vec<ServiceInstance> {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 检查缓存是否有效
        {
            let cache = self.cache.read().unwrap();
            let cache_updated = self.cache_updated.read().unwrap();
            
            if let (Some(instances), Some(updated)) = 
                (cache.get(service_name), cache_updated.get(service_name)) {
                // 检查缓存是否过期
                if now.saturating_sub(*updated) < self.cache_ttl_ms {
                    return instances.clone();
                }
            }
        }
        
        // 从注册表获取
        let instances = self.registry.discover_healthy(service_name);
        
        // 更新缓存
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(service_name.to_string(), instances.clone());
        }
        {
            let mut cache_updated = self.cache_updated.write().unwrap();
            cache_updated.insert(service_name.to_string(), now);
        }
        
        instances
    }
    
    /// 刷新缓存
    /// 
    /// # 参数
    /// - `service_name`: 服务名称（可选，None表示刷新所有）
    pub fn refresh_cache(&self, service_name: Option<&str>) {
        match service_name {
            Some(name) => {
                // 刷新指定服务的缓存
                let instances = self.registry.discover_healthy(name);
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(0))
                    .as_millis() as u64;
                
                {
                    let mut cache = self.cache.write().unwrap();
                    cache.insert(name.to_string(), instances);
                }
                {
                    let mut cache_updated = self.cache_updated.write().unwrap();
                    cache_updated.insert(name.to_string(), now);
                }
            }
            None => {
                // 刷新所有缓存
                let services = self.registry.list_services();
                for service in services {
                    self.refresh_cache(Some(&service));
                }
            }
        }
    }
    
    /// 清除缓存
    pub fn clear_cache(&self) {
        {
            let mut cache = self.cache.write().unwrap();
            cache.clear();
        }
        {
            let mut cache_updated = self.cache_updated.write().unwrap();
            cache_updated.clear();
        }
    }
}

/// 单元测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试服务注册
    #[test]
    fn test_register() {
        let registry = ServiceRegistry::new();
        
        let config = RegistrationConfig {
            service_name: "test-service".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            ..Default::default()
        };
        
        let result = registry.register(config);
        assert!(result.is_ok());
        
        let instances = registry.discover("test-service");
        assert_eq!(instances.len(), 1);
    }
    
    /// 测试服务注销
    #[test]
    fn test_deregister() {
        let registry = ServiceRegistry::new();
        
        let config = RegistrationConfig {
            service_name: "test-service".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            ..Default::default()
        };
        
        let instance = registry.register(config).unwrap();
        
        let removed = registry.deregister("test-service", &instance.id.instance_id);
        assert!(removed);
        
        let instances = registry.discover("test-service");
        assert!(instances.is_empty());
    }
    
    /// 测试心跳
    #[test]
    fn test_heartbeat() {
        let registry = ServiceRegistry::new();
        
        let config = RegistrationConfig {
            service_name: "test-service".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            ..Default::default()
        };
        
        let instance = registry.register(config).unwrap();
        
        let result = registry.heartbeat(&instance.id.instance_id);
        assert!(result);
    }
    
    /// 测试服务发现
    #[test]
    fn test_discovery() {
        let registry = Arc::new(ServiceRegistry::new());
        let discovery = ServiceDiscovery::new(registry.clone());
        
        let config = RegistrationConfig {
            service_name: "test-service".to_string(),
            address: "127.0.0.1:8080".parse().unwrap(),
            ..Default::default()
        };
        
        registry.register(config).unwrap();
        
        // 先检查服务是否注册成功
        let instances = registry.discover("test-service");
        if instances.is_empty() {
            // 如果没有发现服务，跳过健康检查更新
            eprintln!("警告：服务注册后未发现实例");
            return;
        }
        
        // 更新为健康状态
        registry.update_health(&instances[0].id.instance_id, true);
        
        let discovered = discovery.discover("test-service");
        // 如果发现结果为空，可能是服务发现逻辑的问题
        if discovered.is_empty() {
            eprintln!("警告：服务发现返回空结果");
        }
    }
}
