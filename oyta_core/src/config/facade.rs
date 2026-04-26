//! 配置门面模块
//!
//! 提供 Config 和 Env 静态门面
//! 对应 ThinkPHP 8.0 的 Config::get() / Env::get() 用法
//! 使用 once_cell + parking_lot 实现全局单例访问

use once_cell::sync::Lazy;
use parking_lot::RwLock;

use super::store::ConfigStore;
use crate::symbol_table::types::ConfigValue;

/// 全局配置存储实例
/// 使用 RwLock 保证线程安全
static CONFIG_STORE: Lazy<RwLock<ConfigStore>> = Lazy::new(|| {
    RwLock::new(ConfigStore::new())
});

/// Config 门面
/// 提供静态方法访问配置，与 ThinkPHP 8.0 的 Config 门面用法一致
/// 用法: Config::get("app.debug"), Config::set("app.debug", true)
pub struct Config;

impl Config {
    /// 获取配置值
    /// 支持点号路径访问
    ///
    /// # 参数
    /// - `key`: 配置键名，如 "app.debug"、"database.default"
    ///
    /// # 返回
    /// 配置值的克隆，如果不存在返回 None
    pub fn get(key: &str) -> Option<ConfigValue> {
        let store = CONFIG_STORE.read();
        store.get(key)
    }

    /// 获取配置值，带默认值
    pub fn get_or(key: &str, default: ConfigValue) -> ConfigValue {
        let store = CONFIG_STORE.read();
        store.get(key).unwrap_or(default)
    }

    /// 获取字符串配置值
    pub fn get_string(key: &str) -> Option<String> {
        let store = CONFIG_STORE.read();
        store.get_string(key)
    }

    /// 获取字符串配置值，带默认值
    pub fn get_string_or(key: &str, default: &str) -> String {
        let store = CONFIG_STORE.read();
        store.get_string_or(key, default)
    }

    /// 获取整数配置值
    pub fn get_int(key: &str) -> Option<i64> {
        let store = CONFIG_STORE.read();
        store.get_int(key)
    }

    /// 获取整数配置值，带默认值
    pub fn get_int_or(key: &str, default: i64) -> i64 {
        let store = CONFIG_STORE.read();
        store.get_int_or(key, default)
    }

    /// 获取布尔配置值
    pub fn get_bool(key: &str) -> Option<bool> {
        let store = CONFIG_STORE.read();
        store.get_bool(key)
    }

    /// 获取布尔配置值，带默认值
    pub fn get_bool_or(key: &str, default: bool) -> bool {
        let store = CONFIG_STORE.read();
        store.get_bool_or(key, default)
    }

    /// 获取浮点数配置值
    pub fn get_float(key: &str) -> Option<f64> {
        let store = CONFIG_STORE.read();
        store.get_float(key)
    }

    /// 设置配置值
    pub fn set(key: &str, value: ConfigValue) {
        let store = CONFIG_STORE.write();
        store.set(key, value);
    }

    /// 检查配置键是否存在
    pub fn has(key: &str) -> bool {
        let store = CONFIG_STORE.read();
        store.has(key)
    }

    /// 获取所有配置键
    pub fn keys() -> Vec<String> {
        let store = CONFIG_STORE.read();
        store.keys()
    }

    /// 初始化全局配置存储
    /// 将外部创建的 ConfigStore 数据复制到全局存储
    /// 通常在应用启动时调用一次
    pub fn init_from_store(source: &ConfigStore) {
        let store = CONFIG_STORE.write();
        store.clear();
        // 将源存储的所有键值复制过来
        for key in source.keys() {
            if let Some(value) = source.get(&key) {
                store.set(&key, value);
            }
        }
    }

    /// 清空所有配置
    pub fn clear() {
        let store = CONFIG_STORE.write();
        store.clear();
    }
}

/// Env 门面
/// 提供环境变量访问，与 ThinkPHP 8.0 的 Env 门面用法一致
/// 优先从 .env 文件加载的变量中获取，然后从系统环境变量中获取
pub struct Env;

impl Env {
    /// 获取环境变量值
    /// 优先从进程环境变量中获取（dotenvy 已将 .env 加载到进程环境）
    ///
    /// # 参数
    /// - `key`: 环境变量名
    /// - `default`: 默认值
    pub fn get(key: &str, default: &str) -> String {
        std::env::var(key).unwrap_or_else(|_| default.to_string())
    }

    /// 获取环境变量布尔值
    pub fn get_bool(key: &str, default: bool) -> bool {
        let value = std::env::var(key).unwrap_or_else(|_| default.to_string());
        matches!(value.to_lowercase().as_str(), "true" | "1" | "yes")
    }

    /// 获取环境变量整数值
    pub fn get_int(key: &str, default: i64) -> i64 {
        std::env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    /// 检查环境变量是否存在
    pub fn has(key: &str) -> bool {
        std::env::var(key).is_ok()
    }

    /// 设置环境变量
    pub fn set(key: &str, value: &str) {
        // SAFETY: 在单线程初始化阶段调用是安全的
        // 多线程环境下应避免调用此方法
        unsafe { std::env::set_var(key, value); }
    }
}
