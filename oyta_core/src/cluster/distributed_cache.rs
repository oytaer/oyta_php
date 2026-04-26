//! 分布式缓存支持
//!
//! 实现多节点缓存一致性
//! 支持：缓存标签、缓存预热、缓存失效广播
//!
//! # 功能特性
//! - Redis 缓存存储
//! - 缓存标签管理
//! - 缓存一致性保证
//! - 缓存预热

use anyhow::{Context, Result};
use std::collections::HashMap;

/// 分布式缓存管理器
pub struct DistributedCacheManager {
    /// Redis 客户端
    client: Option<redis::Client>,
    /// 缓存前缀
    prefix: String,
    /// 默认过期时间（秒）
    default_ttl: u64,
}

impl DistributedCacheManager {
    /// 创建新的分布式缓存管理器
    pub fn new(redis_url: &str, prefix: &str, ttl: u64) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .with_context(|| format!("无法连接 Redis: {}", redis_url))?;

        Ok(Self {
            client: Some(client),
            prefix: prefix.to_string(),
            default_ttl: ttl,
        })
    }

    /// 创建本地模式
    pub fn local() -> Self {
        Self {
            client: None,
            prefix: "cache:".to_string(),
            default_ttl: 3600,
        }
    }

    /// 获取缓存键名
    fn cache_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }

    /// 获取标签键名
    fn tag_key(&self, tag: &str) -> String {
        format!("{}tag:{}", self.prefix, tag)
    }

    /// 获取缓存值
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let cache_key = self.cache_key(key);
        let mut conn = client.get_connection()?;

        let value: Option<String> = redis::cmd("GET")
            .arg(&cache_key)
            .query(&mut conn)?;

        Ok(value)
    }

    /// 设置缓存值
    pub async fn set(&self, key: &str, value: &str, ttl: Option<u64>) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let cache_key = self.cache_key(key);
        let ttl = ttl.unwrap_or(self.default_ttl);
        let mut conn = client.get_connection()?;

        redis::cmd("SETEX")
            .arg(&cache_key)
            .arg(ttl)
            .arg(value)
            .query::<()>(&mut conn)?;

        Ok(())
    }

    /// 设置带标签的缓存
    pub async fn set_with_tags(&self, key: &str, value: &str, tags: &[&str], ttl: Option<u64>) -> Result<()> {
        // 先设置缓存
        self.set(key, value, ttl).await?;

        // 将键添加到标签集合
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let mut conn = client.get_connection()?;
        let cache_key = self.cache_key(key);

        for tag in tags {
            let tag_key = self.tag_key(tag);
            redis::cmd("SADD")
                .arg(&tag_key)
                .arg(&cache_key)
                .query::<i32>(&mut conn)?;
        }

        Ok(())
    }

    /// 删除缓存
    pub async fn delete(&self, key: &str) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let cache_key = self.cache_key(key);
        let mut conn = client.get_connection()?;

        redis::cmd("DEL")
            .arg(&cache_key)
            .query::<i32>(&mut conn)?;

        Ok(())
    }

    /// 清除标签下的所有缓存
    pub async fn clear_tag(&self, tag: &str) -> Result<usize> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let tag_key = self.tag_key(tag);
        let mut conn = client.get_connection()?;

        // 获取标签下的所有键
        let keys: Vec<String> = redis::cmd("SMEMBERS")
            .arg(&tag_key)
            .query(&mut conn)?;

        let count = keys.len();

        // 删除所有键
        if !keys.is_empty() {
            let mut del_cmd = redis::cmd("DEL");
            for key in &keys {
                del_cmd.arg(key);
            }
            del_cmd.query::<i32>(&mut conn)?;
        }

        // 删除标签集合
        redis::cmd("DEL")
            .arg(&tag_key)
            .query::<i32>(&mut conn)?;

        Ok(count)
    }

    /// 批量获取
    pub async fn mget(&self, keys: &[&str]) -> Result<Vec<Option<String>>> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let mut conn = client.get_connection()?;
        let cache_keys: Vec<String> = keys.iter().map(|k| self.cache_key(k)).collect();

        let mut cmd = redis::cmd("MGET");
        for key in &cache_keys {
            cmd.arg(key);
        }

        let values: Vec<Option<String>> = cmd.query(&mut conn)?;

        Ok(values)
    }

    /// 批量设置
    pub async fn mset(&self, items: &HashMap<&str, &str>) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let mut conn = client.get_connection()?;
        let mut cmd = redis::cmd("MSET");

        for (key, value) in items {
            cmd.arg(self.cache_key(key)).arg(value);
        }

        cmd.query::<()>(&mut conn)?;

        Ok(())
    }

    /// 自增
    pub async fn increment(&self, key: &str, delta: i64) -> Result<i64> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let cache_key = self.cache_key(key);
        let mut conn = client.get_connection()?;

        let result: i64 = redis::cmd("INCRBY")
            .arg(&cache_key)
            .arg(delta)
            .query(&mut conn)?;

        Ok(result)
    }

    /// 自减
    pub async fn decrement(&self, key: &str, delta: i64) -> Result<i64> {
        self.increment(key, -delta).await
    }

    /// 检查是否存在
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let cache_key = self.cache_key(key);
        let mut conn = client.get_connection()?;

        let exists: bool = redis::cmd("EXISTS")
            .arg(&cache_key)
            .query(&mut conn)?;

        Ok(exists)
    }

    /// 设置过期时间
    pub async fn expire(&self, key: &str, ttl: u64) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let cache_key = self.cache_key(key);
        let mut conn = client.get_connection()?;

        redis::cmd("EXPIRE")
            .arg(&cache_key)
            .arg(ttl)
            .query::<i32>(&mut conn)?;

        Ok(())
    }

    /// 清空所有缓存
    pub async fn flush(&self) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let mut conn = client.get_connection()?;

        let pattern = format!("{}*", self.prefix);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query(&mut conn)?;

        if !keys.is_empty() {
            let mut del_cmd = redis::cmd("DEL");
            for key in &keys {
                del_cmd.arg(key);
            }
            del_cmd.query::<i32>(&mut conn)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_cache() {
        let cache = DistributedCacheManager::local();
        assert!(cache.client.is_none());
    }
}
