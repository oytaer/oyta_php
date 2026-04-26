//! 缓存标签模块
//!
//! 实现缓存标签功能
//! 允许按标签分组管理缓存，支持按标签批量清除
//! 对应 ThinkPHP 8.0 中的 Cache::tag() 功能
//!
//! # 使用方式
//! ```php
//! Cache::tag('user')->set('user_1', $data);
//! Cache::tag('user')->clear();
//! ```
//!
//! 实现原理：
//! 每个标签对应一个集合，存储属于该标签的所有缓存键
//! 清除标签时，遍历集合中的所有键进行删除

use anyhow::Result;
use std::collections::{HashMap, HashSet};

use super::driver::CacheDriver;

/// 缓存标签管理器
///
/// 管理缓存键与标签的映射关系
/// 支持按标签批量清除缓存
pub struct CacheTagManager {
    /// 底层缓存驱动
    driver: Box<dyn CacheDriver>,
    /// 标签到键集合的映射
    /// 键：标签名，值：该标签下的所有缓存键
    tags: HashMap<String, HashSet<String>>,
    /// 键到标签集合的映射
    /// 键：缓存键，值：该键所属的所有标签
    key_tags: HashMap<String, HashSet<String>>,
}

impl CacheTagManager {
    /// 创建新的缓存标签管理器
    ///
    /// # 参数
    /// - `driver`: 底层缓存驱动
    pub fn new(driver: Box<dyn CacheDriver>) -> Self {
        Self {
            driver,
            tags: HashMap::new(),
            key_tags: HashMap::new(),
        }
    }

    /// 为缓存键添加标签
    ///
    /// # 参数
    /// - `key`: 缓存键
    /// - `tags`: 标签列表
    pub fn add_tags(&mut self, key: &str, tags: &[&str]) {
        for tag in tags {
            // 添加到标签→键映射
            self.tags
                .entry(tag.to_string())
                .or_insert_with(HashSet::new)
                .insert(key.to_string());

            // 添加到键→标签映射
            self.key_tags
                .entry(key.to_string())
                .or_insert_with(HashSet::new)
                .insert(tag.to_string());
        }
    }

    /// 获取标签下的所有缓存键
    ///
    /// # 参数
    /// - `tag`: 标签名
    pub fn get_tag_keys(&self, tag: &str) -> Vec<String> {
        self.tags
            .get(tag)
            .map(|keys| keys.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// 获取缓存键所属的所有标签
    ///
    /// # 参数
    /// - `key`: 缓存键
    pub fn get_key_tags(&self, key: &str) -> Vec<String> {
        self.key_tags
            .get(key)
            .map(|tags| tags.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// 设置带标签的缓存
    ///
    /// # 参数
    /// - `key`: 缓存键
    /// - `value`: 缓存值
    /// - `ttl`: 过期时间（秒），0 表示永不过期
    /// - `tags`: 标签列表
    pub fn set_with_tags(
        &mut self,
        key: &str,
        value: &str,
        ttl: u64,
        tags: &[&str],
    ) -> Result<()> {
        // 先设置缓存值
        self.driver.set(key, value, ttl)?;

        // 再建立标签映射
        self.add_tags(key, tags);

        Ok(())
    }

    /// 获取缓存值
    ///
    /// # 参数
    /// - `key`: 缓存键
    pub fn get(&self, key: &str) -> Option<String> {
        self.driver.get(key)
    }

    /// 删除缓存值
    /// 同时清除标签映射
    ///
    /// # 参数
    /// - `key`: 缓存键
    pub fn delete(&mut self, key: &str) -> Result<()> {
        // 从底层驱动删除
        self.driver.delete(key)?;

        // 清除标签映射
        if let Some(tags) = self.key_tags.remove(key) {
            for tag in &tags {
                if let Some(keys) = self.tags.get_mut(tag) {
                    keys.remove(key);
                    if keys.is_empty() {
                        self.tags.remove(tag);
                    }
                }
            }
        }

        Ok(())
    }

    /// 清除指定标签下的所有缓存
    ///
    /// # 参数
    /// - `tag`: 标签名
    pub fn clear_tag(&mut self, tag: &str) -> Result<()> {
        if let Some(keys) = self.tags.remove(tag) {
            for key in &keys {
                // 从底层驱动删除
                let _ = self.driver.delete(key);

                // 从键→标签映射中移除该标签
                if let Some(key_tags) = self.key_tags.get_mut(key) {
                    key_tags.remove(tag);
                    if key_tags.is_empty() {
                        self.key_tags.remove(key);
                    }
                }
            }
        }
        Ok(())
    }

    /// 清除多个标签下的所有缓存
    ///
    /// # 参数
    /// - `tags`: 标签列表
    pub fn clear_tags(&mut self, tags: &[&str]) -> Result<()> {
        for tag in tags {
            self.clear_tag(tag)?;
        }
        Ok(())
    }

    /// 检查标签是否存在
    ///
    /// # 参数
    /// - `tag`: 标签名
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains_key(tag)
    }

    /// 获取所有标签名
    pub fn all_tags(&self) -> Vec<String> {
        self.tags.keys().cloned().collect()
    }

    /// 获取标签数量
    pub fn tag_count(&self) -> usize {
        self.tags.len()
    }

    /// 清空所有缓存和标签映射
    pub fn clear_all(&mut self) -> Result<()> {
        self.driver.clear()?;
        self.tags.clear();
        self.key_tags.clear();
        Ok(())
    }
}

/// 带标签的缓存操作构建器
/// 提供链式调用 API
///
/// # 使用方式
/// ```rust
/// let tagged = TaggedCache::new(driver);
/// tagged.tag("user").set("user_1", "data", 3600);
/// tagged.tag("user").clear();
/// ```
pub struct TaggedCache {
    /// 底层缓存驱动
    driver: Box<dyn CacheDriver>,
    /// 当前操作的标签列表
    current_tags: Vec<String>,
    /// 标签管理器
    manager: CacheTagManager,
}

impl TaggedCache {
    /// 创建新的带标签缓存
    pub fn new(driver: Box<dyn CacheDriver>) -> Self {
        let manager_driver = driver.clone_boxed();
        Self {
            driver,
            current_tags: Vec::new(),
            manager: CacheTagManager::new(manager_driver),
        }
    }

    /// 设置当前操作的标签
    /// 支持链式调用
    pub fn tag(mut self, tag: &str) -> Self {
        self.current_tags.push(tag.to_string());
        self
    }

    /// 设置多个标签
    pub fn tags(mut self, tags: &[&str]) -> Self {
        for tag in tags {
            self.current_tags.push(tag.to_string());
        }
        self
    }

    /// 设置缓存值（带当前标签）
    pub fn set(&mut self, key: &str, value: &str, ttl: u64) -> Result<()> {
        self.driver.set(key, value, ttl)?;

        let tags: Vec<&str> = self.current_tags.iter().map(|s| s.as_str()).collect();
        self.manager.add_tags(key, &tags);

        Ok(())
    }

    /// 获取缓存值
    pub fn get(&self, key: &str) -> Option<String> {
        self.driver.get(key)
    }

    /// 删除缓存值
    pub fn delete(&mut self, key: &str) -> Result<()> {
        self.manager.delete(key)
    }

    /// 清除当前标签下的所有缓存
    pub fn clear(&mut self) -> Result<()> {
        for tag in &self.current_tags.clone() {
            self.manager.clear_tag(tag)?;
        }
        self.current_tags.clear();
        Ok(())
    }

    /// 检查缓存是否存在
    pub fn has(&self, key: &str) -> bool {
        self.driver.has(key)
    }

    /// 记住缓存（不存在时回调获取并设置）
    pub fn remember<F>(&mut self, key: &str, ttl: u64, callback: F) -> Option<String>
    where
        F: FnOnce() -> Option<String>,
    {
        if let Some(value) = self.get(key) {
            return Some(value);
        }

        if let Some(value) = callback() {
            let _ = self.set(key, &value, ttl);
            return Some(value);
        }

        None
    }
}
