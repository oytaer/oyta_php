//! 缓存管理器模块
//!
//! 管理缓存驱动实例
//! 支持多驱动切换：内存、文件、Redis
//! 对应 ThinkPHP 8.0 的 Cache 类

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use super::driver::{CacheDriver, FileCacheDriver, MemoryCacheDriver};
use super::redis_driver::RedisCacheDriver;

/// 缓存管理器
///
/// 管理多个缓存驱动实例
/// 支持运行时切换默认驱动
/// 支持从配置文件自动初始化
pub struct CacheManager {
    /// 缓存驱动实例映射（驱动名 → 驱动实例）
    drivers: HashMap<String, Arc<dyn CacheDriver>>,
    /// 默认驱动名
    default_driver: String,
}

impl CacheManager {
    /// 创建新的缓存管理器
    ///
    /// 自动注册内存和文件两个默认驱动
    pub fn new() -> Self {
        let mut manager = Self {
            drivers: HashMap::new(),
            default_driver: "file".to_string(),
        };
        manager.register_default_drivers();
        manager
    }

    /// 注册默认驱动
    ///
    /// 注册内存缓存和文件缓存两个内置驱动
    fn register_default_drivers(&mut self) {
        self.drivers.insert(
            "memory".to_string(),
            Arc::new(MemoryCacheDriver::default()),
        );

        let cache_dir = std::env::current_dir()
            .unwrap_or_default()
            .join("runtime")
            .join("cache");
        self.drivers.insert(
            "file".to_string(),
            Arc::new(FileCacheDriver::new(&cache_dir)),
        );
    }

    /// 设置默认驱动
    ///
    /// # 参数
    /// - `name`: 驱动名
    pub fn set_default(&mut self, name: &str) {
        self.default_driver = name.to_string();
    }

    /// 获取默认驱动
    ///
    /// # 返回
    /// 默认缓存驱动实例
    pub fn default_driver(&self) -> Option<Arc<dyn CacheDriver>> {
        self.drivers.get(&self.default_driver).cloned()
    }

    /// 获取指定驱动
    ///
    /// # 参数
    /// - `name`: 驱动名
    ///
    /// # 返回
    /// 指定缓存驱动实例
    pub fn driver(&self, name: &str) -> Option<Arc<dyn CacheDriver>> {
        self.drivers.get(name).cloned()
    }

    /// 注册自定义驱动
    ///
    /// # 参数
    /// - `name`: 驱动名
    /// - `driver`: 驱动实例
    pub fn register_driver(&mut self, name: &str, driver: Arc<dyn CacheDriver>) {
        self.drivers.insert(name.to_string(), driver);
    }

    /// 注册 Redis 驱动
    ///
    /// 便捷方法，创建并注册 Redis 缓存驱动
    ///
    /// # 参数
    /// - `name`: 驱动名（默认为 "redis"）
    /// - `url`: Redis 连接 URL
    /// - `prefix`: 键前缀
    pub fn register_redis(&mut self, name: &str, url: &str, prefix: &str) {
        let driver = RedisCacheDriver::new(url, prefix);
        self.drivers.insert(name.to_string(), Arc::new(driver));
    }

    /// 获取所有已注册的驱动名
    pub fn driver_names(&self) -> Vec<String> {
        self.drivers.keys().cloned().collect()
    }

    /// 获取默认驱动名
    pub fn default_driver_name(&self) -> &str {
        &self.default_driver
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局缓存管理器
static CACHE_MANAGER: Lazy<RwLock<CacheManager>> = Lazy::new(|| RwLock::new(CacheManager::new()));

/// 初始化缓存管理器
///
/// 从配置存储中加载缓存配置
/// 自动注册 Redis 驱动（如果配置了）
///
/// # 参数
/// - `config`: 配置存储引用
pub fn init_cache(config: &crate::config::store::ConfigStore) {
    let mut manager = CACHE_MANAGER.write();

    if let Some(driver) = config.get_string("cache.default") {
        manager.set_default(&driver);
    }

    if let Some(redis_url) = config.get_string("cache.stores.redis.url")
        .or_else(|| config.get_string("cache.redis.url"))
    {
        let prefix = config
            .get_string("cache.stores.redis.prefix")
            .or_else(|| config.get_string("cache.prefix"))
            .unwrap_or_else(|| "oyta_cache:".to_string());
        manager.register_redis("redis", &redis_url, &prefix);
    }
}

/// 获取默认缓存驱动
pub fn get_default_driver() -> Option<Arc<dyn CacheDriver>> {
    let manager = CACHE_MANAGER.read();
    manager.default_driver()
}

/// 获取指定缓存驱动
pub fn get_driver(name: &str) -> Option<Arc<dyn CacheDriver>> {
    let manager = CACHE_MANAGER.read();
    manager.driver(name)
}
