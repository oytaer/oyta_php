//! 多级缓存驱动模块
//!
//! 实现 L1（内存）+ L2（Redis/文件）的多级缓存策略
//! 读取时先查 L1，未命中再查 L2
//! 写入时同时写入 L1 和 L2
//! 对应 ThinkPHP 8.0 中的多级缓存配置
//!
//! # 配置示例
//! ```php
//! return [
//!     'default' => 'multi',
//!     'stores' => [
//!         'multi' => [
//!             'type' => 'multi',
//!             'drivers' => ['memory', 'redis'],
//!         ],
//!     ],
//! ];
//! ```

use anyhow::Result;
use std::collections::HashMap;

use super::driver::CacheDriver;

/// 多级缓存驱动
///
/// 组合多个缓存驱动形成多级缓存
/// L1 通常为内存缓存（快速但容量有限）
/// L2 通常为 Redis/文件缓存（较慢但持久化）
pub struct MultiLevelCacheDriver {
    /// 缓存层级列表
    /// 按优先级排序：索引 0 为 L1（最快），索引越大越慢
    levels: Vec<Box<dyn CacheDriver>>,
}

impl MultiLevelCacheDriver {
    /// 创建新的多级缓存驱动
    ///
    /// # 参数
    /// - `levels`: 缓存层级列表，按优先级从高到低排列
    ///
    /// # 示例
    /// ```rust
    /// let multi = MultiLevelCacheDriver::new(vec![
    ///     Box::new(MemoryCacheDriver::default()),
    ///     Box::new(FileCacheDriver::new("/tmp/cache")),
    /// ]);
    /// ```
    pub fn new(levels: Vec<Box<dyn CacheDriver>>) -> Self {
        Self { levels }
    }

    /// 获取缓存层级数量
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }
}

impl CacheDriver for MultiLevelCacheDriver {
    /// 从多级缓存中获取值
    /// 按优先级从 L1 到 Ln 依次查找
    /// 如果在低优先级层级找到，会回填到高优先级层级
    fn get(&self, key: &str) -> Option<String> {
        for (i, level) in self.levels.iter().enumerate() {
            if let Some(value) = level.get(key) {
                // 回填到更高优先级的层级
                if i > 0 {
                    for j in 0..i {
                        let _ = self.levels[j].set(key, &value, 0);
                    }
                }
                return Some(value);
            }
        }
        None
    }

    /// 写入多级缓存
    /// 同时写入所有层级
    fn set(&self, key: &str, value: &str, ttl: u64) -> Result<()> {
        let mut last_error = None;
        for level in &self.levels {
            if let Err(e) = level.set(key, value, ttl) {
                last_error = Some(e);
            }
        }
        // 只在所有层级都失败时返回错误
        if let Some(e) = last_error {
            // 至少一个层级成功就不报错
            for level in &self.levels {
                if level.has(key) {
                    return Ok(());
                }
            }
            return Err(e);
        }
        Ok(())
    }

    /// 从多级缓存中删除
    /// 同时删除所有层级
    fn delete(&self, key: &str) -> Result<()> {
        for level in &self.levels {
            let _ = level.delete(key);
        }
        Ok(())
    }

    /// 检查多级缓存中是否存在
    /// 只要在任一层级存在就返回 true
    fn has(&self, key: &str) -> bool {
        self.levels.iter().any(|level| level.has(key))
    }

    /// 自增
    /// 在所有层级上执行自增
    fn increment(&self, key: &str, step: i64) -> Result<i64> {
        let mut result = step;
        for level in &self.levels {
            if let Ok(r) = level.increment(key, step) {
                result = r;
            }
        }
        Ok(result)
    }

    /// 自减
    /// 在所有层级上执行自减
    fn decrement(&self, key: &str, step: i64) -> Result<i64> {
        let mut result = -step;
        for level in &self.levels {
            if let Ok(r) = level.decrement(key, step) {
                result = r;
            }
        }
        Ok(result)
    }

    /// 清空所有层级
    fn clear(&self) -> Result<()> {
        for level in &self.levels {
            let _ = level.clear();
        }
        Ok(())
    }

    /// 批量获取
    fn get_multiple(&self, keys: &[&str]) -> HashMap<String, Option<String>> {
        let mut result = HashMap::new();
        for key in keys {
            result.insert(key.to_string(), self.get(key));
        }
        result
    }

    /// 批量设置
    fn set_multiple(&self, items: &[(String, String)], ttl: u64) -> Result<()> {
        for (key, value) in items {
            self.set(key, value, ttl)?;
        }
        Ok(())
    }

    /// 批量删除
    fn delete_multiple(&self, keys: &[&str]) -> Result<()> {
        for key in keys {
            self.delete(key)?;
        }
        Ok(())
    }

    fn clone_boxed(&self) -> Box<dyn CacheDriver> {
        let levels: Vec<Box<dyn CacheDriver>> = self.levels.iter()
            .map(|level| level.clone_boxed())
            .collect();
        Box::new(MultiLevelCacheDriver::new(levels))
    }
}
