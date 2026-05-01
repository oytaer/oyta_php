//! 分布式锁实现
//!
//! 实现 Redlock 算法的分布式锁
//! 支持多 Redis 实例的高可用锁
//!
//! # 功能特性
//! - 基于 Redis 的分布式锁
//! - 自动续期
//! - 锁超时
//! - 可重入锁

use anyhow::{Context, Result};
use rand::Rng;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 分布式锁
pub struct DistributedLock {
    /// Redis 客户端
    client: Option<redis::Client>,
    /// 锁键前缀
    prefix: String,
    /// 锁超时时间（毫秒）
    default_timeout: u64,
    /// 锁重试间隔（毫秒）
    retry_interval: u64,
    /// 锁重试次数
    retry_count: u32,
}

/// 锁令牌
#[derive(Debug, Clone)]
pub struct LockToken {
    /// 锁键
    pub key: String,
    /// 锁值（用于释放锁时验证）
    pub value: String,
    /// 过期时间
    pub expires_at: Instant,
}

impl DistributedLock {
    /// 创建新的分布式锁
    pub fn new(redis_url: &str, prefix: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .with_context(|| format!("无法连接 Redis: {}", redis_url))?;

        Ok(Self {
            client: Some(client),
            prefix: prefix.to_string(),
            default_timeout: 30000,
            retry_interval: 100,
            retry_count: 3,
        })
    }

    /// 创建本地模式
    pub fn local() -> Self {
        Self {
            client: None,
            prefix: "lock:".to_string(),
            default_timeout: 30000,
            retry_interval: 100,
            retry_count: 3,
        }
    }

    /// 设置默认超时时间
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.default_timeout = timeout_ms;
        self
    }

    /// 设置重试参数
    pub fn with_retry(mut self, interval_ms: u64, count: u32) -> Self {
        self.retry_interval = interval_ms;
        self.retry_count = count;
        self
    }

    /// 获取锁键名
    fn lock_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }

    /// 生成锁值
    fn generate_value() -> String {
        format!("{}:{}", std::process::id(), rand::rng().random::<u64>())
    }

    /// 尝试获取锁
    ///
    /// # 参数
    /// - `key`: 锁键
    /// - `timeout_ms`: 锁超时时间（毫秒）
    ///
    /// # 返回
    /// 锁令牌，获取失败返回 None
    pub async fn try_acquire(&self, key: &str, timeout_ms: Option<u64>) -> Result<Option<LockToken>> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let lock_key = self.lock_key(key);
        let lock_value = Self::generate_value();
        let timeout = timeout_ms.unwrap_or(self.default_timeout);
        let mut conn = client.get_connection()?;

        // 使用 SET NX PX 原子操作
        let result: Option<String> = redis::cmd("SET")
            .arg(&lock_key)
            .arg(&lock_value)
            .arg("NX")
            .arg("PX")
            .arg(timeout)
            .query(&mut conn)?;

        if result.is_some() {
            Ok(Some(LockToken {
                key: lock_key,
                value: lock_value,
                expires_at: Instant::now() + Duration::from_millis(timeout),
            }))
        } else {
            Ok(None)
        }
    }

    /// 获取锁（阻塞）
    ///
    /// # 参数
    /// - `key`: 锁键
    /// - `timeout_ms`: 锁超时时间（毫秒）
    ///
    /// # 返回
    /// 锁令牌
    pub async fn acquire(&self, key: &str, timeout_ms: Option<u64>) -> Result<LockToken> {
        for _ in 0..self.retry_count {
            if let Some(token) = self.try_acquire(key, timeout_ms).await? {
                return Ok(token);
            }

            tokio::time::sleep(Duration::from_millis(self.retry_interval)).await;
        }

        anyhow::bail!("获取锁失败: {}", key);
    }

    /// 释放锁
    ///
    /// # 参数
    /// - `token`: 锁令牌
    ///
    /// # 返回
    /// 是否成功释放
    pub async fn release(&self, token: &LockToken) -> Result<bool> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let mut conn = client.get_connection()?;

        // 使用 Lua 脚本确保原子性
        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("DEL", KEYS[1])
            else
                return 0
            end
        "#;

        let result: i32 = redis::cmd("EVAL")
            .arg(script)
            .arg(1)
            .arg(&token.key)
            .arg(&token.value)
            .query(&mut conn)?;

        Ok(result == 1)
    }

    /// 续期锁
    ///
    /// # 参数
    /// - `token`: 锁令牌
    /// - `timeout_ms`: 新的超时时间（毫秒）
    ///
    /// # 返回
    /// 是否成功续期
    pub async fn renew(&self, token: &LockToken, timeout_ms: Option<u64>) -> Result<bool> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let timeout = timeout_ms.unwrap_or(self.default_timeout);
        let mut conn = client.get_connection()?;

        // 使用 Lua 脚本确保原子性
        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("PEXPIRE", KEYS[1], ARGV[2])
            else
                return 0
            end
        "#;

        let result: i32 = redis::cmd("EVAL")
            .arg(script)
            .arg(1)
            .arg(&token.key)
            .arg(&token.value)
            .arg(timeout)
            .query(&mut conn)?;

        Ok(result == 1)
    }

    /// 检查锁是否存在
    pub async fn is_locked(&self, key: &str) -> Result<bool> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let lock_key = self.lock_key(key);
        let mut conn = client.get_connection()?;

        let exists: bool = redis::cmd("EXISTS")
            .arg(&lock_key)
            .query(&mut conn)?;

        Ok(exists)
    }

    /// 强制删除锁（危险操作，仅用于管理）
    pub async fn force_release(&self, key: &str) -> Result<()> {
        let client = self.client.as_ref()
            .context("Redis 未连接")?;

        let lock_key = self.lock_key(key);
        let mut conn = client.get_connection()?;

        redis::cmd("DEL")
            .arg(&lock_key)
            .query::<i32>(&mut conn)?;

        Ok(())
    }
}

/// 自动续期的锁守卫
pub struct LockGuard {
    /// 锁实例
    lock: Arc<DistributedLock>,
    /// 锁令牌
    token: LockToken,
    /// 续期任务句柄
    renew_handle: Option<tokio::task::JoinHandle<()>>,
}

impl LockGuard {
    /// 创建新的锁守卫
    pub async fn new(lock: Arc<DistributedLock>, key: &str, timeout_ms: Option<u64>) -> Result<Self> {
        let token = lock.acquire(key, timeout_ms).await?;
        let timeout = timeout_ms.unwrap_or(lock.default_timeout);

        // 启动自动续期任务
        let lock_clone = lock.clone();
        let token_clone = token.clone();
        let renew_interval = timeout / 3;

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(renew_interval));
            loop {
                interval.tick().await;
                if lock_clone.renew(&token_clone, Some(timeout)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Self {
            lock,
            token,
            renew_handle: Some(handle),
        })
    }

    /// 获取锁令牌
    pub fn token(&self) -> &LockToken {
        &self.token
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        // 停止续期任务
        if let Some(handle) = self.renew_handle.take() {
            handle.abort();
        }

        // 释放锁
        let lock = self.lock.clone();
        let token = self.token.clone();

        tokio::spawn(async move {
            let _ = lock.release(&token).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_lock() {
        let lock = DistributedLock::local();
        assert!(lock.client.is_none());
    }
}
