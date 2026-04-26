//! Cache 门面模块
//!
//! 对应 ThinkPHP 8.0 的 Cache 门面
//! 提供静态方法访问缓存功能

use anyhow::Result;

use super::driver::CacheDriver;
use super::manager;

/// Cache 门面
pub struct Cache;

impl Cache {
    /// 获取缓存值
    pub fn get(key: &str) -> Option<String> {
        manager::get_default_driver().and_then(|d| d.get(key))
    }

    /// 获取缓存值，带默认值
    pub fn get_or(key: &str, default: &str) -> String {
        Self::get(key).unwrap_or_else(|| default.to_string())
    }

    /// 设置缓存值
    pub fn set(key: &str, value: &str, ttl: u64) -> Result<()> {
        match manager::get_default_driver() {
            Some(d) => d.set(key, value, ttl),
            None => anyhow::bail!("缓存驱动未初始化"),
        }
    }

    /// 永久设置缓存
    pub fn forever(key: &str, value: &str) -> Result<()> {
        Self::set(key, value, 0)
    }

    /// 删除缓存
    pub fn delete(key: &str) -> Result<()> {
        match manager::get_default_driver() {
            Some(d) => d.delete(key),
            None => anyhow::bail!("缓存驱动未初始化"),
        }
    }

    /// 检查缓存是否存在
    pub fn has(key: &str) -> bool {
        manager::get_default_driver().map(|d| d.has(key)).unwrap_or(false)
    }

    /// 自增
    pub fn increment(key: &str, step: i64) -> Result<i64> {
        match manager::get_default_driver() {
            Some(d) => d.increment(key, step),
            None => anyhow::bail!("缓存驱动未初始化"),
        }
    }

    /// 自减
    pub fn decrement(key: &str, step: i64) -> Result<i64> {
        match manager::get_default_driver() {
            Some(d) => d.decrement(key, step),
            None => anyhow::bail!("缓存驱动未初始化"),
        }
    }

    /// 清空所有缓存
    pub fn clear() -> Result<()> {
        match manager::get_default_driver() {
            Some(d) => d.clear(),
            None => anyhow::bail!("缓存驱动未初始化"),
        }
    }

    /// 记住缓存（如果不存在则设置）
    pub fn remember(key: &str, ttl: u64, value: &str) -> Result<String> {
        if let Some(v) = Self::get(key) {
            return Ok(v);
        }
        Self::set(key, value, ttl)?;
        Ok(value.to_string())
    }
}
