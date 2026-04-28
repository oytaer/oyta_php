//! 服务注册表模块
//!
//! 管理所有服务的注册、查找和状态跟踪
//! 提供线程安全的服务存储和访问机制
//!
//! # 功能
//! - 服务注册与注销
//! - 服务查找与获取
//! - 服务状态管理
//! - 服务依赖解析

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::traits::{Service, ServiceConfig, ServiceMetadata, ServiceState};

/// 服务包装器
///
/// 包装服务实例，提供额外的状态和元数据管理
struct ServiceWrapper {
    /// 服务实例
    service: Box<dyn Service>,
    /// 服务元数据
    metadata: ServiceMetadata,
    /// 服务状态
    state: ServiceState,
}

/// 服务注册表
///
/// 线程安全的服务管理容器
/// 使用读写锁实现并发访问
pub struct ServiceRegistry {
    /// 服务映射表（服务名 → 服务包装器）
    services: Arc<RwLock<HashMap<String, ServiceWrapper>>>,
    /// 类型索引（服务类型 → 服务名列表）
    type_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// 延迟加载服务列表
    deferred: Arc<RwLock<Vec<String>>>,
}

impl ServiceRegistry {
    /// 创建新的服务注册表
    ///
    /// # 返回值
    /// 空的服务注册表实例
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            type_index: Arc::new(RwLock::new(HashMap::new())),
            deferred: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 注册服务
    ///
    /// 将服务添加到注册表中
    ///
    /// # 参数
    /// - `service`: 服务实例
    /// - `metadata`: 服务元数据
    ///
    /// # 返回值
    /// 注册成功返回 Ok(())，名称冲突返回 Err
    pub async fn register(
        &self,
        mut service: Box<dyn Service>,
        metadata: ServiceMetadata,
    ) -> anyhow::Result<()> {
        let name = service.name().to_string();
        let service_type = service.service_type().to_string();
        let is_deferred = service.is_deferred();

        // 检查服务是否已存在
        {
            let services = self.services.read().await;
            if services.contains_key(&name) {
                return Err(anyhow::anyhow!("服务 {} 已存在", name));
            }
        }

        // 调用服务的 register 方法
        service.register()?;
        
        // 设置服务状态为已注册
        service.set_state(ServiceState::Registered);

        // 创建服务包装器
        let wrapper = ServiceWrapper {
            service,
            metadata,
            state: ServiceState::Registered,
        };

        // 添加到服务表
        {
            let mut services = self.services.write().await;
            services.insert(name.clone(), wrapper);
        }

        // 更新类型索引
        {
            let mut type_index = self.type_index.write().await;
            type_index
                .entry(service_type)
                .or_insert_with(Vec::new)
                .push(name.clone());
        }

        // 如果是延迟加载，添加到延迟列表
        if is_deferred {
            let mut deferred = self.deferred.write().await;
            deferred.push(name.clone());
        }

        tracing::debug!("服务 {} 已注册", name);
        Ok(())
    }

    /// 注销服务
    ///
    /// 从注册表中移除服务
    ///
    /// # 参数
    /// - `name`: 服务名称
    ///
    /// # 返回值
    /// 注销成功返回 true，服务不存在返回 false
    pub async fn unregister(&self, name: &str) -> bool {
        // 从服务表中移除
        let wrapper = {
            let mut services = self.services.write().await;
            services.remove(name)
        };

        if let Some(wrapper) = wrapper {
            let service_type = wrapper.metadata.service_type.clone();

            // 从类型索引中移除
            {
                let mut type_index = self.type_index.write().await;
                if let Some(names) = type_index.get_mut(&service_type) {
                    names.retain(|n| n != name);
                    if names.is_empty() {
                        type_index.remove(&service_type);
                    }
                }
            }

            // 从延迟列表中移除
            {
                let mut deferred = self.deferred.write().await;
                deferred.retain(|n| n != name);
            }

            tracing::debug!("服务 {} 已注销", name);
            true
        } else {
            false
        }
    }

    /// 检查服务是否存在
    ///
    /// # 参数
    /// - `name`: 服务名称
    ///
    /// # 返回值
    /// 服务存在返回 true
    pub async fn has(&self, name: &str) -> bool {
        let services = self.services.read().await;
        services.contains_key(name)
    }

    /// 获取服务状态
    ///
    /// # 参数
    /// - `name`: 服务名称
    ///
    /// # 返回值
    /// 服务状态，不存在返回 None
    pub async fn get_state(&self, name: &str) -> Option<ServiceState> {
        let services = self.services.read().await;
        services.get(name).map(|w| w.state)
    }

    /// 设置服务状态
    ///
    /// # 参数
    /// - `name`: 服务名称
    /// - `state`: 新状态
    pub async fn set_state(&self, name: &str, state: ServiceState) {
        let mut services = self.services.write().await;
        if let Some(wrapper) = services.get_mut(name) {
            wrapper.state = state;
            if let Some(service) = services.get_mut(name) {
                service.service.set_state(state);
            }
        }
    }

    /// 获取服务元数据
    ///
    /// # 参数
    /// - `name`: 服务名称
    ///
    /// # 返回值
    /// 服务元数据的克隆
    pub async fn get_metadata(&self, name: &str) -> Option<ServiceMetadata> {
        let services = self.services.read().await;
        services.get(name).map(|w| w.metadata.clone())
    }

    /// 获取所有服务名称
    ///
    /// # 返回值
    /// 所有已注册服务名称的列表
    pub async fn get_all_names(&self) -> Vec<String> {
        let services = self.services.read().await;
        services.keys().cloned().collect()
    }

    /// 获取指定类型的服务列表
    ///
    /// # 参数
    /// - `service_type`: 服务类型
    ///
    /// # 返回值
    /// 该类型下所有服务名称的列表
    pub async fn get_by_type(&self, service_type: &str) -> Vec<String> {
        let type_index = self.type_index.read().await;
        type_index.get(service_type).cloned().unwrap_or_default()
    }

    /// 获取所有延迟加载的服务
    ///
    /// # 返回值
    /// 延迟加载服务名称的列表
    pub async fn get_deferred(&self) -> Vec<String> {
        let deferred = self.deferred.read().await;
        deferred.clone()
    }

    /// 获取已启用的服务列表
    ///
    /// # 返回值
    /// 所有已启用服务名称的列表
    pub async fn get_enabled(&self) -> Vec<String> {
        let services = self.services.read().await;
        services
            .iter()
            .filter(|(_, w)| w.service.is_enabled())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// 获取正在运行的服务列表
    ///
    /// # 返回值
    /// 所有正在运行服务名称的列表
    pub async fn get_running(&self) -> Vec<String> {
        let services = self.services.read().await;
        services
            .iter()
            .filter(|(_, w)| w.state.is_running())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// 获取服务数量
    ///
    /// # 返回值
    /// 已注册服务的总数
    pub async fn count(&self) -> usize {
        let services = self.services.read().await;
        services.len()
    }

    /// 清空所有服务
    ///
    /// 移除所有已注册的服务
    pub async fn clear(&self) {
        let mut services = self.services.write().await;
        services.clear();

        let mut type_index = self.type_index.write().await;
        type_index.clear();

        let mut deferred = self.deferred.write().await;
        deferred.clear();

        tracing::debug!("所有服务已清空");
    }

    /// 解析服务依赖
    ///
    /// 检查所有服务的依赖是否满足
    ///
    /// # 返回值
    /// 返回依赖检查结果，包含缺失的依赖
    pub async fn resolve_dependencies(&self) -> HashMap<String, Vec<String>> {
        let services = self.services.read().await;
        let mut missing: HashMap<String, Vec<String>> = HashMap::new();

        for (name, wrapper) in services.iter() {
            for dep in &wrapper.metadata.dependencies {
                if !services.contains_key(dep) {
                    missing
                        .entry(name.clone())
                        .or_insert_with(Vec::new)
                        .push(dep.clone());
                }
            }
        }

        missing
    }

    /// 获取服务启动顺序
    ///
    /// 根据依赖关系计算服务的启动顺序
    /// 使用拓扑排序确保依赖先于服务启动
    ///
    /// # 返回值
    /// 服务启动顺序列表，循环依赖返回 Err
    pub async fn get_startup_order(&self) -> anyhow::Result<Vec<String>> {
        let services = self.services.read().await;
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut result: Vec<String> = Vec::new();

        // 初始化入度和图
        for (name, _) in services.iter() {
            in_degree.insert(name.clone(), 0);
            graph.insert(name.clone(), Vec::new());
        }

        // 构建依赖图
        for (name, wrapper) in services.iter() {
            for dep in &wrapper.metadata.dependencies {
                if let Some(degree) = in_degree.get_mut(dep) {
                    *degree += 1;
                }
                if let Some(adj) = graph.get_mut(dep) {
                    adj.push(name.clone());
                }
            }
        }

        // 拓扑排序
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter_map(|(name, d)| if *d == 0 { Some(name.clone()) } else { None })
            .collect();

        while !queue.is_empty() {
            let node = queue.remove(0);
            result.push(node.clone());

            if let Some(adj) = graph.get(&node) {
                for neighbor in adj {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        // 检查是否有循环依赖
        if result.len() != services.len() {
            return Err(anyhow::anyhow!("存在循环依赖，无法确定启动顺序"));
        }

        Ok(result)
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 虚拟服务（用于内部处理）
struct DummyService {
    name: String,
    state: ServiceState,
    config: ServiceConfig,
}

impl DummyService {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            state: ServiceState::Unregistered,
            config: HashMap::new(),
        }
    }
}

impl Service for DummyService {
    fn name(&self) -> &str {
        &self.name
    }

    fn service_type(&self) -> &str {
        "dummy"
    }

    fn state(&self) -> ServiceState {
        self.state
    }

    fn set_state(&mut self, state: ServiceState) {
        self.state = state;
    }

    fn config(&self) -> &ServiceConfig {
        &self.config
    }
}

/// 全局服务注册表
///
/// 使用懒加载初始化全局单例
pub fn get_service_registry() -> &'static ServiceRegistry {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<ServiceRegistry> = OnceLock::new();
    REGISTRY.get_or_init(ServiceRegistry::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_basic() {
        let registry = ServiceRegistry::new();
        
        // 注册服务
        let service = Box::new(DummyService::new("test_service"));
        let metadata = ServiceMetadata::new("test_service", "test");
        
        registry.register(service, metadata).await.unwrap();
        
        // 检查服务存在
        assert!(registry.has("test_service").await);
        
        // 获取状态
        let state = registry.get_state("test_service").await;
        assert_eq!(state, Some(ServiceState::Registered));
        
        // 注销服务
        assert!(registry.unregister("test_service").await);
        assert!(!registry.has("test_service").await);
    }

    #[tokio::test]
    async fn test_registry_type_index() {
        let registry = ServiceRegistry::new();
        
        // 注册多个同类型服务
        // 注意：DummyService 的 service_type 总是返回 "dummy"
        for i in 0..3 {
            let service = Box::new(DummyService::new(&format!("dummy_{}", i)));
            let metadata = ServiceMetadata::new(&format!("dummy_{}", i), "dummy");
            registry.register(service, metadata).await.unwrap();
        }
        
        // 按类型获取（使用 "dummy" 类型）
        let dummy_services = registry.get_by_type("dummy").await;
        assert_eq!(dummy_services.len(), 3);
    }

    #[tokio::test]
    async fn test_startup_order() {
        let registry = ServiceRegistry::new();
        
        // 注册两个没有依赖关系的服务
        let service_a = Box::new(DummyService::new("service_a"));
        let metadata_a = ServiceMetadata::new("service_a", "dummy");
        registry.register(service_a, metadata_a).await.unwrap();
        
        let service_b = Box::new(DummyService::new("service_b"));
        let metadata_b = ServiceMetadata::new("service_b", "dummy");
        registry.register(service_b, metadata_b).await.unwrap();
        
        // 获取启动顺序（没有依赖关系，顺序可以是任意的）
        let order = registry.get_startup_order().await.unwrap();
        
        // 验证两个服务都在结果中
        assert!(order.contains(&"service_a".to_string()));
        assert!(order.contains(&"service_b".to_string()));
        assert_eq!(order.len(), 2);
    }
}
