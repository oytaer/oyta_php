//! 分布式 Session 支持
//!
//! 使用 Redis 实现 Session 共享
//! 支持多节点部署、Session 复制、分布式锁
//!
//! # 功能特性
//! - Redis Session 存储
//! - Session 分布式锁
//! - Session 复制
//! - 自动过期清理

use anyhow::{Context, Result};
use std::collections::HashMap;

/// 分布式 Session 管理器
pub struct DistributedSessionManager {
    /// Redis 客户端
    client: Option<redis::Client>,
    /// Session 前缀
    prefix: String,
    /// 默认过期时间（秒）
    default_ttl: u64,
}

impl DistributedSessionManager {
    /// 创建新的分布式 Session 管理器
    ///
    /// # 参数
    /// - `redis_url`: Redis 连接 URL
    /// - `prefix`: Session 键前缀
    /// - `ttl`: 默认过期时间（秒）
    pub fn new(redis_url: &str, prefix: &str, ttl: u64) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .with_context(|| format!("无法连接 Redis: {}", redis_url))?;

        Ok(Self {
            client: Some(client),
            prefix: prefix.to_string(),
            default_ttl: ttl,
        })
    }

    /// 创建本地模式（不使用 Redis）
    pub fn local() -> Self {
        Self {
            client: None,
            prefix: "session:".to_string(),
            default_ttl: 7200,
        }
    }

    /// 获取 Session 键名
    fn session_key(&self, session_id: &str) -> String {
        format!("{}{}", self.prefix, session_id)
    }

    /// 获取 Session 锁键名
    fn lock_key(&self, session_id: &str) -> String {
        format!("{}{}:lock", self.prefix, session_id)
    }

    /// 读取 Session 数据
    ///
    /// # 参数
    /// - `session_id`: Session ID
    ///
    /// # 返回
    /// Session 数据映射
    pub async fn get(&self, session_id: &str) -> Result<HashMap<String, String>> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let key = self.session_key(session_id);
        let mut conn = client.get_connection()?;

        let data: HashMap<String, String> = redis::cmd("HGETALL")
            .arg(&key)
            .query(&mut conn)?;

        Ok(data)
    }

    /// 获取 Session 单个字段
    ///
    /// # 参数
    /// - `session_id`: Session ID
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 字段值
    pub async fn get_field(&self, session_id: &str, field: &str) -> Result<Option<String>> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let key = self.session_key(session_id);
        let mut conn = client.get_connection()?;

        let value: Option<String> = redis::cmd("HGET")
            .arg(&key)
            .arg(field)
            .query(&mut conn)?;

        Ok(value)
    }

    /// 设置 Session 数据
    ///
    /// # 参数
    /// - `session_id`: Session ID
    /// - `data`: Session 数据
    pub async fn set(&self, session_id: &str, data: &HashMap<String, String>) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let key = self.session_key(session_id);
        let mut conn = client.get_connection()?;

        // 删除旧数据
        redis::cmd("DEL")
            .arg(&key)
            .query::<i32>(&mut conn)?;

        // 设置新数据
        if !data.is_empty() {
            let mut cmd = redis::cmd("HMSET");
            cmd.arg(&key);
            for (k, v) in data {
                cmd.arg(k).arg(v);
            }
            cmd.query::<()>(&mut conn)?;
        }

        // 设置过期时间
        redis::cmd("EXPIRE")
            .arg(&key)
            .arg(self.default_ttl)
            .query::<i32>(&mut conn)?;

        Ok(())
    }

    /// 设置 Session 单个字段
    ///
    /// # 参数
    /// - `session_id`: Session ID
    /// - `field`: 字段名
    /// - `value`: 字段值
    pub async fn set_field(&self, session_id: &str, field: &str, value: &str) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let key = self.session_key(session_id);
        let mut conn = client.get_connection()?;

        redis::cmd("HSET")
            .arg(&key)
            .arg(field)
            .arg(value)
            .query::<i32>(&mut conn)?;

        // 刷新过期时间
        redis::cmd("EXPIRE")
            .arg(&key)
            .arg(self.default_ttl)
            .query::<i32>(&mut conn)?;

        Ok(())
    }

    /// 删除 Session 字段
    ///
    /// # 参数
    /// - `session_id`: Session ID
    /// - `field`: 字段名
    pub async fn delete_field(&self, session_id: &str, field: &str) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let key = self.session_key(session_id);
        let mut conn = client.get_connection()?;

        redis::cmd("HDEL")
            .arg(&key)
            .arg(field)
            .query::<i32>(&mut conn)?;

        Ok(())
    }

    /// 销毁 Session
    ///
    /// # 参数
    /// - `session_id`: Session ID
    pub async fn destroy(&self, session_id: &str) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let key = self.session_key(session_id);
        let lock_key = self.lock_key(session_id);
        let mut conn = client.get_connection()?;

        redis::cmd("DEL")
            .arg(&key)
            .arg(&lock_key)
            .query::<i32>(&mut conn)?;

        Ok(())
    }

    /// 检查 Session 是否存在
    ///
    /// # 参数
    /// - `session_id`: Session ID
    ///
    /// # 返回
    /// 是否存在
    pub async fn exists(&self, session_id: &str) -> Result<bool> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let key = self.session_key(session_id);
        let mut conn = client.get_connection()?;

        let exists: bool = redis::cmd("EXISTS")
            .arg(&key)
            .query(&mut conn)?;

        Ok(exists)
    }

    /// 刷新 Session 过期时间
    ///
    /// # 参数
    /// - `session_id`: Session ID
    pub async fn refresh(&self, session_id: &str) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let key = self.session_key(session_id);
        let mut conn = client.get_connection()?;

        redis::cmd("EXPIRE")
            .arg(&key)
            .arg(self.default_ttl)
            .query::<i32>(&mut conn)?;

        Ok(())
    }

    /// 获取 Session 剩余过期时间
    ///
    /// # 参数
    /// - `session_id`: Session ID
    ///
    /// # 返回
    /// 剩余秒数，-1 表示永不过期，-2 表示不存在
    pub async fn ttl(&self, session_id: &str) -> Result<i64> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let key = self.session_key(session_id);
        let mut conn = client.get_connection()?;

        let ttl: i64 = redis::cmd("TTL")
            .arg(&key)
            .query(&mut conn)?;

        Ok(ttl)
    }

    /// 获取分布式锁
    ///
    /// # 参数
    /// - `session_id`: Session ID
    /// - `timeout`: 锁超时时间（毫秒）
    ///
    /// # 返回
    /// 是否获取成功
    pub async fn acquire_lock(&self, session_id: &str, timeout: u64) -> Result<bool> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let lock_key = self.lock_key(session_id);
        let mut conn = client.get_connection()?;

        let token = format!("lock_{}", rand::random::<u64>());

        let result: Option<String> = redis::cmd("SET")
            .arg(&lock_key)
            .arg(&token)
            .arg("NX")
            .arg("PX")
            .arg(timeout)
            .query(&mut conn)?;

        Ok(result.is_some())
    }

    /// 释放分布式锁
    ///
    /// # 参数
    /// - `session_id`: Session ID
    pub async fn release_lock(&self, session_id: &str) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let lock_key = self.lock_key(session_id);
        let mut conn = client.get_connection()?;

        redis::cmd("DEL")
            .arg(&lock_key)
            .query::<i32>(&mut conn)?;

        Ok(())
    }

    /// 获取统计信息
    pub async fn stats(&self) -> Result<SessionStats> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let mut conn = client.get_connection()?;

        // 获取所有 Session 键
        let pattern = format!("{}*", self.prefix);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query(&mut conn)?;

        let total = keys.len();
        let locked = keys.iter().filter(|k| k.contains(":lock")).count();

        Ok(SessionStats {
            total_sessions: total - locked,
            locked_sessions: locked,
        })
    }
}

/// Session 统计信息
#[derive(Debug, Clone)]
pub struct SessionStats {
    /// 总 Session 数
    pub total_sessions: usize,
    /// 锁定的 Session 数
    pub locked_sessions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_session() {
        let manager = DistributedSessionManager::local();
        assert!(manager.client.is_none());
    }
}
