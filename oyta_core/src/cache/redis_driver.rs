//! Redis 缓存驱动
//!
//! 使用 redis crate 实现基于 Redis 的缓存驱动
//! 支持连接池、TTL、自增/自减等操作
//! 对应 ThinkPHP 8.0 的 Redis 缓存驱动

use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use std::collections::HashMap;

use super::driver::CacheDriver;

/// Redis 缓存驱动
///
/// 使用 Redis 作为缓存后端
/// 支持所有 CacheDriver trait 定义的方法
/// 连接池在首次使用时惰性创建
pub struct RedisCacheDriver {
    /// Redis 连接 URL
    url: String,
    /// 连接池（惰性创建）
    pool: OnceCell<deadpool_redis::Pool>,
    /// 键前缀
    prefix: String,
}

impl RedisCacheDriver {
    /// 创建新的 Redis 缓存驱动
    ///
    /// # 参数
    /// - `url`: Redis 连接 URL，例如 redis://127.0.0.1:6379/0
    /// - `prefix`: 键前缀，用于区分不同应用的缓存
    pub fn new(url: &str, prefix: &str) -> Self {
        Self {
            url: url.to_string(),
            pool: OnceCell::new(),
            prefix: prefix.to_string(),
        }
    }

    /// 获取带前缀的完整键名
    fn full_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }

    /// 获取 Redis 连接池
    ///
    /// 惰性创建连接池，首次调用时初始化
    fn get_pool(&self) -> Result<&deadpool_redis::Pool> {
        self.pool.get_or_try_init(|| {
            let cfg = deadpool_redis::Config::from_url(&self.url);
            cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
                .with_context(|| format!("无法创建 Redis 连接池: {}", self.url))
        })
    }

    /// 获取 Redis 连接
    fn get_connection(&self) -> Result<deadpool_redis::Connection> {
        let pool = self.get_pool()?;
        let rt = tokio::runtime::Runtime::new()
            .with_context(|| "无法创建 tokio runtime")?;
        rt.block_on(pool.get())
            .map_err(|e| anyhow::anyhow!("获取 Redis 连接失败: {}", e))
    }

    /// 在 tokio runtime 中执行异步 Redis 命令
    fn exec<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut deadpool_redis::Connection) -> std::pin::Pin<Box<dyn std::future::Future<Output = redis::RedisResult<T>> + Send>> + Send,
        T: Send + 'static,
    {
        let rt = tokio::runtime::Runtime::new().ok()?;
        rt.block_on(async {
            let mut conn = self.get_connection().ok()?;
            f(&mut conn).await.ok()
        })
    }
}

impl CacheDriver for RedisCacheDriver {
    fn get(&self, key: &str) -> Option<String> {
        let full_key = self.full_key(key);
        let rt = tokio::runtime::Runtime::new().ok()?;
        rt.block_on(async {
            let mut conn = self.get_connection().ok()?;
            let result: Option<String> = redis::cmd("GET")
                .arg(&full_key)
                .query_async(&mut *conn)
                .await
                .ok()
                .flatten();
            result
        })
    }

    fn set(&self, key: &str, value: &str, ttl: u64) -> Result<()> {
        let full_key = self.full_key(key);
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let mut conn = self.get_connection()?;
            if ttl > 0 {
                redis::cmd("SETEX")
                    .arg(&full_key)
                    .arg(ttl)
                    .arg(value)
                    .query_async::<()>(&mut *conn)
                    .await
                    .with_context(|| format!("Redis SETEX 失败: {}", full_key))?;
            } else {
                redis::cmd("SET")
                    .arg(&full_key)
                    .arg(value)
                    .query_async::<()>(&mut *conn)
                    .await
                    .with_context(|| format!("Redis SET 失败: {}", full_key))?;
            }
            Ok(())
        })
    }

    fn delete(&self, key: &str) -> Result<()> {
        let full_key = self.full_key(key);
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let mut conn = self.get_connection()?;
            redis::cmd("DEL")
                .arg(&full_key)
                .query_async::<()>(&mut *conn)
                .await
                .with_context(|| format!("Redis DEL 失败: {}", full_key))?;
            Ok(())
        })
    }

    fn has(&self, key: &str) -> bool {
        let full_key = self.full_key(key);
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(_) => return false,
        };
        rt.block_on(async {
            let mut conn = match self.get_connection() {
                Ok(c) => c,
                Err(_) => return false,
            };
            let exists: bool = redis::cmd("EXISTS")
                .arg(&full_key)
                .query_async(&mut *conn)
                .await
                .unwrap_or(false);
            exists
        })
    }

    fn increment(&self, key: &str, step: i64) -> Result<i64> {
        let full_key = self.full_key(key);
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let mut conn = self.get_connection()?;
            let result: i64 = redis::cmd("INCRBY")
                .arg(&full_key)
                .arg(step)
                .query_async(&mut *conn)
                .await
                .with_context(|| format!("Redis INCRBY 失败: {}", full_key))?;
            Ok(result)
        })
    }

    fn decrement(&self, key: &str, step: i64) -> Result<i64> {
        let full_key = self.full_key(key);
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let mut conn = self.get_connection()?;
            let result: i64 = redis::cmd("DECRBY")
                .arg(&full_key)
                .arg(step)
                .query_async(&mut *conn)
                .await
                .with_context(|| format!("Redis DECRBY 失败: {}", full_key))?;
            Ok(result)
        })
    }

    fn clear(&self) -> Result<()> {
        let pattern = format!("{}*", self.prefix);
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let mut conn = self.get_connection()?;
            let keys: Vec<String> = redis::cmd("KEYS")
                .arg(&pattern)
                .query_async(&mut *conn)
                .await
                .unwrap_or_default();
            if !keys.is_empty() {
                redis::cmd("DEL")
                    .arg(&keys)
                    .query_async::<()>(&mut *conn)
                    .await
                    .with_context(|| "Redis DEL 批量删除失败")?;
            }
            Ok(())
        })
    }

    fn get_multiple(&self, keys: &[&str]) -> HashMap<String, Option<String>> {
        let full_keys: Vec<String> = keys.iter().map(|k| self.full_key(k)).collect();
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(_) => {
                let mut result = HashMap::new();
                for key in keys {
                    result.insert(key.to_string(), None);
                }
                return result;
            }
        };
        rt.block_on(async {
            let mut conn = match self.get_connection() {
                Ok(c) => c,
                Err(_) => {
                    let mut result = HashMap::new();
                    for key in keys {
                        result.insert(key.to_string(), None);
                    }
                    return result;
                }
            };
            let values: Vec<Option<String>> = redis::cmd("MGET")
                .arg(&full_keys)
                .query_async(&mut *conn)
                .await
                .unwrap_or_default();
            let mut result = HashMap::new();
            for (i, key) in keys.iter().enumerate() {
                let value = values.get(i).cloned().flatten();
                result.insert(key.to_string(), value);
            }
            result
        })
    }

    fn clone_boxed(&self) -> Box<dyn CacheDriver> {
        Box::new(RedisCacheDriver::new(&self.url, &self.prefix))
    }
}
