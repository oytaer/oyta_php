//! 缓存驱动模块
//!
//! 定义缓存驱动接口和实现
//! 支持：内存缓存（moka）、文件缓存

use anyhow::Result;
use std::collections::HashMap;

/// 缓存驱动 trait
/// 所有缓存驱动都需要实现此接口
pub trait CacheDriver: Send + Sync {
    /// 获取缓存值
    fn get(&self, key: &str) -> Option<String>;

    /// 设置缓存值（带过期时间，秒为单位，0 表示永不过期）
    fn set(&self, key: &str, value: &str, ttl: u64) -> Result<()>;

    /// 删除缓存
    fn delete(&self, key: &str) -> Result<()>;

    /// 检查缓存是否存在
    fn has(&self, key: &str) -> bool;

    /// 自增
    fn increment(&self, key: &str, step: i64) -> Result<i64>;

    /// 自减
    fn decrement(&self, key: &str, step: i64) -> Result<i64>;

    /// 清空所有缓存
    fn clear(&self) -> Result<()>;

    /// 获取多个缓存值
    fn get_multiple(&self, keys: &[&str]) -> HashMap<String, Option<String>> {
        let mut result = HashMap::new();
        for key in keys {
            result.insert(key.to_string(), self.get(key));
        }
        result
    }

    /// 设置多个缓存值
    fn set_multiple(&self, items: &[(String, String)], ttl: u64) -> Result<()> {
        for (key, value) in items {
            self.set(key, value, ttl)?;
        }
        Ok(())
    }

    /// 删除多个缓存
    fn delete_multiple(&self, keys: &[&str]) -> Result<()> {
        for key in keys {
            self.delete(key)?;
        }
        Ok(())
    }

    /// 克隆为 Box 化的 trait 对象
    /// 用于多级缓存等需要克隆驱动的场景
    fn clone_boxed(&self) -> Box<dyn CacheDriver>;
}

/// 内存缓存驱动
/// 使用 moka 实现高性能内存缓存
pub struct MemoryCacheDriver {
    cache: moka::sync::Cache<String, CacheEntry>,
}

/// 缓存条目（含过期时间）
#[derive(Debug, Clone)]
struct CacheEntry {
    /// 缓存值
    value: String,
    /// 过期时间戳（秒），0 表示永不过期
    expire_at: i64,
}

impl MemoryCacheDriver {
    /// 创建新的内存缓存驱动
    pub fn new(max_capacity: usize) -> Self {
        let cache = moka::sync::Cache::builder()
            .max_capacity(max_capacity as u64)
            .build();
        Self { cache }
    }

    /// 检查条目是否过期
    fn is_expired(entry: &CacheEntry) -> bool {
        if entry.expire_at == 0 {
            return false;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        now > entry.expire_at
    }
}

impl Default for MemoryCacheDriver {
    fn default() -> Self {
        Self::new(10000)
    }
}

impl CacheDriver for MemoryCacheDriver {
    fn get(&self, key: &str) -> Option<String> {
        self.cache.get(key).and_then(|entry| {
            if Self::is_expired(&entry) {
                self.cache.invalidate(key);
                None
            } else {
                Some(entry.value)
            }
        })
    }

    fn set(&self, key: &str, value: &str, ttl: u64) -> Result<()> {
        let expire_at = if ttl > 0 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            now + ttl as i64
        } else {
            0
        };
        self.cache.insert(key.to_string(), CacheEntry {
            value: value.to_string(),
            expire_at,
        });
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<()> {
        self.cache.invalidate(key);
        Ok(())
    }

    fn has(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    fn increment(&self, key: &str, step: i64) -> Result<i64> {
        let current = self.get(key)
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);
        let new_val = current + step;
        self.set(key, &new_val.to_string(), 0)?;
        Ok(new_val)
    }

    fn decrement(&self, key: &str, step: i64) -> Result<i64> {
        self.increment(key, -step)
    }

    fn clear(&self) -> Result<()> {
        self.cache.invalidate_all();
        Ok(())
    }

    fn clone_boxed(&self) -> Box<dyn CacheDriver> {
        Box::new(MemoryCacheDriver::new(10000))
    }
}
/// 将缓存数据存储在文件系统中
pub struct FileCacheDriver {
    /// 缓存目录路径
    cache_dir: std::path::PathBuf,
}

impl FileCacheDriver {
    /// 创建新的文件缓存驱动
    pub fn new(cache_dir: &std::path::Path) -> Self {
        // 确保缓存目录存在
        if !cache_dir.exists() {
            let _ = std::fs::create_dir_all(cache_dir);
        }
        Self {
            cache_dir: cache_dir.to_path_buf(),
        }
    }

    /// 获取缓存文件路径
    fn cache_file_path(&self, key: &str) -> std::path::PathBuf {
        // 使用 MD5 哈希作为文件名，避免特殊字符问题
        let hash = format!("{:x}", md5_hash(key.as_bytes()));
        self.cache_dir.join(hash)
    }
}

impl CacheDriver for FileCacheDriver {
    fn get(&self, key: &str) -> Option<String> {
        let path = self.cache_file_path(key);
        let content = std::fs::read_to_string(&path).ok()?;
        // 文件格式: timestamp\nvalue
        // timestamp 为 0 表示永不过期
        let mut parts = content.splitn(2, '\n');
        let timestamp_str = parts.next()?;
        let value = parts.next()?;

        let timestamp: i64 = timestamp_str.parse().ok()?;
        if timestamp > 0 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            if now > timestamp {
                // 缓存已过期
                let _ = std::fs::remove_file(&path);
                return None;
            }
        }

        Some(value.to_string())
    }

    fn set(&self, key: &str, value: &str, ttl: u64) -> Result<()> {
        let path = self.cache_file_path(key);
        let expire_at = if ttl > 0 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            now + ttl as i64
        } else {
            0
        };
        let content = format!("{}\n{}", expire_at, value);
        std::fs::write(&path, content)?;
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<()> {
        let path = self.cache_file_path(key);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    fn has(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    fn increment(&self, key: &str, step: i64) -> Result<i64> {
        let current = self.get(key)
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);
        let new_val = current + step;
        self.set(key, &new_val.to_string(), 0)?;
        Ok(new_val)
    }

    fn decrement(&self, key: &str, step: i64) -> Result<i64> {
        self.increment(key, -step)
    }

    fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.cache_dir)? {
                if let Ok(entry) = entry {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
        Ok(())
    }

    fn clone_boxed(&self) -> Box<dyn CacheDriver> {
        Box::new(FileCacheDriver::new(&self.cache_dir))
    }
}

/// 简单的 MD5 哈希（用于缓存文件名）
fn md5_hash(data: &[u8]) -> u128 {
    use std::hash::Hasher;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash_slice(data, &mut hasher);
    hasher.finish() as u128
}
