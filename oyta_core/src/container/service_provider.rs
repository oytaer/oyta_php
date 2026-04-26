//! 服务提供者模块
//!
//! 实现服务提供者模式
//! 对应 ThinkPHP 8.0 的 ServiceProvider 类
//! 服务提供者用于封装服务的注册和引导逻辑
//!
//! # 使用方式
//! ```php
//! class CacheServiceProvider extends ServiceProvider {
//!     public function register() {
//!         $this->app->singleton('cache', function($app) {
//!             return new CacheManager($app['config']);
//!         });
//!     }
//!     public function boot() {
//!         // 引导逻辑
//!     }
//! }
//! ```

use super::container::Container;

/// 服务提供者 trait
///
/// 所有服务提供者都需要实现此 trait
/// register() 在容器注册阶段调用
/// boot() 在所有服务注册完成后调用
pub trait ServiceProvider: Send + Sync {
    /// 注册服务到容器
    ///
    /// 此方法中只应做服务绑定，不应做任何副作用操作
    /// 因为此时其他服务可能还未注册
    fn register(&self, container: &Container);

    /// 引导服务
    ///
    /// 此方法在所有服务提供者的 register() 执行完毕后调用
    /// 可以安全地使用其他已注册的服务
    fn boot(&self, _container: &Container) {}

    /// 服务提供者名称
    fn name(&self) -> &str;

    /// 是否延迟注册
    /// 延迟注册的服务提供者只有在被使用时才注册
    fn is_deferred(&self) -> bool {
        false
    }

    /// 提供的服务列表（延迟加载时使用）
    fn provides(&self) -> Vec<String> {
        Vec::new()
    }
}

/// 服务提供者管理器
///
/// 管理所有服务提供者的注册和引导
pub struct ServiceProviderManager {
    /// 已注册的服务提供者
    providers: Vec<Box<dyn ServiceProvider>>,
    /// 已引导的提供者名称
    booted: Vec<String>,
    /// 延迟加载的提供者（服务名 → 提供者索引）
    deferred: std::collections::HashMap<String, usize>,
}

impl ServiceProviderManager {
    /// 创建新的服务提供者管理器
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            booted: Vec::new(),
            deferred: std::collections::HashMap::new(),
        }
    }

    /// 注册服务提供者
    ///
    /// 立即执行 register()，延迟提供者记录映射
    ///
    /// # 参数
    /// - `provider`: 服务提供者实例
    /// - `container`: IoC 容器
    pub fn register(&mut self, provider: Box<dyn ServiceProvider>, container: &Container) {
        if provider.is_deferred() {
            let idx = self.providers.len();
            for service in provider.provides() {
                self.deferred.insert(service, idx);
            }
        } else {
            provider.register(container);
        }

        self.providers.push(provider);
    }

    /// 引导所有服务提供者
    ///
    /// 按注册顺序依次调用每个提供者的 boot() 方法
    ///
    /// # 参数
    /// - `container`: IoC 容器
    pub fn boot_all(&mut self, container: &Container) {
        for provider in &self.providers {
            let name = provider.name().to_string();
            if !self.booted.contains(&name) {
                provider.boot(container);
                self.booted.push(name);
            }
        }
    }

    /// 解析延迟加载的服务
    ///
    /// 当请求的服务由延迟提供者提供时
    /// 先注册该提供者，再返回服务
    ///
    /// # 参数
    /// - `service`: 服务名
    /// - `container`: IoC 容器
    pub fn resolve_deferred(&mut self, service: &str, container: &Container) {
        if let Some(&idx) = self.deferred.get(service) {
            if let Some(provider) = self.providers.get(idx) {
                let name = provider.name().to_string();
                if !self.booted.contains(&name) {
                    provider.register(container);
                    provider.boot(container);
                    self.booted.push(name);
                }
            }
        }
    }

    /// 获取已注册的提供者数量
    pub fn count(&self) -> usize {
        self.providers.len()
    }

    /// 获取已引导的提供者数量
    pub fn booted_count(&self) -> usize {
        self.booted.len()
    }

    /// 检查指定提供者是否已引导
    pub fn is_booted(&self, name: &str) -> bool {
        self.booted.contains(&name.to_string())
    }
}

impl Default for ServiceProviderManager {
    fn default() -> Self {
        Self::new()
    }
}
