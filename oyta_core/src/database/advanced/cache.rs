//! 查询缓存模块
//!
//! 提供数据库查询缓存功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库

use crate::interpreter::value::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// 查询缓存配置
#[derive(Debug, Clone)]
pub struct QueryCacheConfig {
    /// 是否启用缓存
    pub enabled: bool,
    /// 缓存过期时间（秒）
    pub ttl: u64,
    /// 最大缓存条目数
    pub max_entries: usize,
}

impl Default for QueryCacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl: 3600,
            max_entries: 1000,
        }
    }
}

impl QueryCacheConfig {
    /// 创建新的缓存配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置是否启用
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// 设置过期时间
    pub fn ttl(mut self, ttl: u64) -> Self {
        self.ttl = ttl;
        self
    }

    /// 设置最大条目数
    pub fn max_entries(mut self, max_entries: usize) -> Self {
        self.max_entries = max_entries;
        self
    }
}

/// 缓存条目
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// 缓存数据
    pub data: Vec<HashMap<String, Value>>,
    /// 创建时间
    pub created_at: Instant,
    /// 过期时间
    pub expires_at: Instant,
}

impl CacheEntry {
    /// 创建新的缓存条目
    pub fn new(data: Vec<HashMap<String, Value>>, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            data,
            created_at: now,
            expires_at: now + ttl,
        }
    }

    /// 检查是否过期
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

/// 查询缓存
pub struct QueryCache {
    /// 缓存存储
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// 配置
    config: QueryCacheConfig,
}

impl QueryCache {
    /// 创建新的查询缓存
    pub fn new(config: QueryCacheConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 生成缓存键
    pub fn generate_key(&self, sql: &str, bindings: &[Value]) -> String {
        let mut key = sql.to_string();
        for binding in bindings {
            key.push_str(&format!(":{:?}", binding));
        }
        format!("{:x}", md5::compute(key))
    }

    /// 获取缓存
    pub fn get(&self, key: &str) -> Option<Vec<HashMap<String, Value>>> {
        if !self.config.enabled {
            return None;
        }

        let cache = self.cache.read().unwrap();
        cache.get(key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    /// 设置缓存
    pub fn set(&self, key: &str, data: Vec<HashMap<String, Value>>) {
        if !self.config.enabled {
            return;
        }

        let mut cache = self.cache.write().unwrap();

        // 检查是否超过最大条目数
        if cache.len() >= self.config.max_entries {
            self.evict_expired(&mut cache);
            if cache.len() >= self.config.max_entries {
                // 删除最早的条目
                if let Some(oldest_key) = cache.iter()
                    .min_by_key(|(_, e)| e.created_at)
                    .map(|(k, _)| k.clone())
                {
                    cache.remove(&oldest_key);
                }
            }
        }

        let entry = CacheEntry::new(data, Duration::from_secs(self.config.ttl));
        cache.insert(key.to_string(), entry);
    }

    /// 删除缓存
    pub fn forget(&self, key: &str) {
        let mut cache = self.cache.write().unwrap();
        cache.remove(key);
    }

    /// 清空缓存
    pub fn flush(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// 清理过期缓存
    fn evict_expired(&self, cache: &mut HashMap<String, CacheEntry>) {
        cache.retain(|_, entry| !entry.is_expired());
    }

    /// 获取缓存统计
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read().unwrap();
        let total = cache.len();
        let expired = cache.values().filter(|e| e.is_expired()).count();

        CacheStats {
            total,
            expired,
            active: total - expired,
        }
    }
}

/// 缓存统计
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// 总条目数
    pub total: usize,
    /// 过期条目数
    pub expired: usize,
    /// 活跃条目数
    pub active: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry() {
        let data = vec![HashMap::new()];
        let entry = CacheEntry::new(data, Duration::from_secs(60));

        assert!(!entry.is_expired());
    }

    #[test]
    fn test_query_cache() {
        let config = QueryCacheConfig::new().ttl(60);
        let cache = QueryCache::new(config);

        let key = cache.generate_key("SELECT * FROM users", &[]);
        let data = vec![HashMap::from([
            ("id".to_string(), Value::Int(1)),
            ("name".to_string(), Value::String("test".to_string())),
        ])];

        cache.set(&key, data.clone());

        let cached = cache.get(&key);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);
    }

    #[test]
    fn test_cache_forget() {
        let cache = QueryCache::new(QueryCacheConfig::new());

        let key = "test_key";
        let data = vec![HashMap::new()];

        cache.set(key, data);
        cache.forget(key);

        assert!(cache.get(key).is_none());
    }

    #[test]
    fn test_cache_flush() {
        let cache = QueryCache::new(QueryCacheConfig::new());

        cache.set("key1", vec![HashMap::new()]);
        cache.set("key2", vec![HashMap::new()]);

        cache.flush();

        let stats = cache.stats();
        assert_eq!(stats.total, 0);
    }
}
