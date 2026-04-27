//! 配置中心模块
//! 
//! 实现分布式配置管理功能，包括：
//! - 配置存储
//! - 配置版本管理
//! - 配置变更通知
//! - 配置加密
//! - 多环境支持

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 配置条目
/// 表示一个配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    /// 配置键
    pub key: String,
    /// 配置值
    pub value: String,
    /// 配置版本
    pub version: u64,
    /// 创建时间
    pub created_at: u64,
    /// 更新时间
    pub updated_at: u64,
    /// 配置标签
    pub labels: HashMap<String, String>,
    /// 配置描述
    pub description: String,
    /// 是否加密
    pub encrypted: bool,
    /// 环境
    pub environment: String,
    /// 应用名称
    pub application: String,
    /// 配置来源
    pub source: ConfigSource,
    /// 是否启用
    pub enabled: bool,
    /// 过期时间（0表示永不过期）
    pub expires_at: u64,
}

/// 为 ConfigEntry 实现默认值
impl Default for ConfigEntry {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        Self {
            key: String::new(),
            value: String::new(),
            version: 1,
            created_at: now,
            updated_at: now,
            labels: HashMap::new(),
            description: String::new(),
            encrypted: false,
            environment: String::from("default"),
            application: String::from("default"),
            source: ConfigSource::Manual,
            enabled: true,
            expires_at: 0,
        }
    }
}

/// 配置来源枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigSource {
    /// 手动配置
    Manual,
    /// 文件导入
    FileImport,
    /// 环境变量
    Environment,
    /// 远程同步
    RemoteSync,
    /// 自动发现
    AutoDiscovery,
}

/// 配置变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeEvent {
    /// 事件ID
    pub event_id: String,
    /// 变更类型
    pub change_type: ConfigChangeType,
    /// 配置键
    pub key: String,
    /// 旧值
    pub old_value: Option<String>,
    /// 新值
    pub new_value: Option<String>,
    /// 变更时间
    pub timestamp: u64,
    /// 变更来源
    pub source: String,
    /// 环境
    pub environment: String,
    /// 应用
    pub application: String,
}

/// 配置变更类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigChangeType {
    /// 创建
    Created,
    /// 更新
    Updated,
    /// 删除
    Deleted,
    /// 启用
    Enabled,
    /// 禁用
    Disabled,
}

/// 配置监听器
/// 用于监听配置变更
pub struct ConfigWatcher {
    /// 监听器ID
    pub id: String,
    /// 监听的键模式
    pub key_pattern: String,
    /// 监听的环境
    pub environment: String,
    /// 监听的应用
    pub application: String,
    /// 是否启用
    pub enabled: bool,
    /// 回调函数（存储回调标识）
    pub callback_id: String,
    /// 最后触发时间
    pub last_triggered: u64,
    /// 触发次数
    pub trigger_count: u64,
}

/// 为 ConfigWatcher 实现相关方法
impl ConfigWatcher {
    /// 创建新的配置监听器
    pub fn new(id: &str, key_pattern: &str) -> Self {
        Self {
            id: id.to_string(),
            key_pattern: key_pattern.to_string(),
            environment: String::from("default"),
            application: String::from("default"),
            enabled: true,
            callback_id: String::new(),
            last_triggered: 0,
            trigger_count: 0,
        }
    }
    
    /// 检查键是否匹配
    pub fn matches(&self, key: &str) -> bool {
        // 简单的通配符匹配
        if self.key_pattern == "*" {
            return true;
        }
        
        if self.key_pattern.ends_with('*') {
            let prefix = &self.key_pattern[..self.key_pattern.len() - 1];
            return key.starts_with(prefix);
        }
        
        key == self.key_pattern
    }
}

/// 配置中心统计信息
#[derive(Debug, Default)]
pub struct ConfigCenterStats {
    /// 总配置数
    pub total_configs: AtomicU64,
    /// 总变更次数
    pub total_changes: AtomicU64,
    /// 总监听器数
    pub total_watchers: AtomicU64,
    /// 总查询次数
    pub total_queries: AtomicU64,
    /// 缓存命中次数
    pub cache_hits: AtomicU64,
    /// 缓存未命中次数
    pub cache_misses: AtomicU64,
}

/// 配置中心
/// 核心配置管理实现
pub struct ConfigCenter {
    /// 配置存储：键 -> 配置条目
    configs: RwLock<HashMap<String, ConfigEntry>>,
    /// 配置版本历史：键 -> 版本列表
    version_history: RwLock<HashMap<String, Vec<ConfigEntry>>>,
    /// 配置监听器
    watchers: RwLock<Vec<ConfigWatcher>>,
    /// 变更事件队列
    change_events: RwLock<Vec<ConfigChangeEvent>>,
    /// 统计信息
    stats: ConfigCenterStats,
    /// 是否正在运行
    running: AtomicBool,
    /// 配置
    config: ConfigCenterConfig,
}

/// 配置中心配置
#[derive(Debug, Clone)]
pub struct ConfigCenterConfig {
    /// 最大版本历史数
    pub max_version_history: usize,
    /// 最大事件队列大小
    pub max_event_queue_size: usize,
    /// 是否启用版本历史
    pub enable_version_history: bool,
    /// 是否启用变更事件
    pub enable_change_events: bool,
    /// 默认环境
    pub default_environment: String,
    /// 默认应用
    pub default_application: String,
    /// 加密密钥（用于加密配置）
    pub encryption_key: String,
}

/// 为 ConfigCenterConfig 实现默认值
impl Default for ConfigCenterConfig {
    fn default() -> Self {
        Self {
            max_version_history: 100,
            max_event_queue_size: 1000,
            enable_version_history: true,
            enable_change_events: true,
            default_environment: String::from("default"),
            default_application: String::from("default"),
            encryption_key: String::new(),
        }
    }
}

/// 为 ConfigCenter 实现相关方法
impl ConfigCenter {
    /// 创建新的配置中心
    pub fn new(config: ConfigCenterConfig) -> Self {
        Self {
            configs: RwLock::new(HashMap::new()),
            version_history: RwLock::new(HashMap::new()),
            watchers: RwLock::new(Vec::new()),
            change_events: RwLock::new(Vec::new()),
            stats: ConfigCenterStats::default(),
            running: AtomicBool::new(false),
            config,
        }
    }
    
    /// 使用默认配置创建配置中心
    pub fn with_defaults() -> Self {
        Self::new(ConfigCenterConfig::default())
    }
    
    /// 设置配置
    /// 
    /// # 参数
    /// - `key`: 配置键
    /// - `value`: 配置值
    /// - `environment`: 环境
    /// - `application`: 应用
    /// 
    /// # 返回
    /// 配置条目
    pub fn set(&self, key: &str, value: &str, environment: &str, application: &str) -> ConfigEntry {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 获取写锁
        let mut configs = self.configs.write().unwrap();
        
        // 检查是否已存在
        let (entry, old_value) = if let Some(existing) = configs.get(key) {
            // 更新现有配置
            let updated = ConfigEntry {
                key: key.to_string(),
                value: value.to_string(),
                version: existing.version + 1,
                created_at: existing.created_at,
                updated_at: now,
                labels: existing.labels.clone(),
                description: existing.description.clone(),
                encrypted: existing.encrypted,
                environment: environment.to_string(),
                application: application.to_string(),
                source: existing.source,
                enabled: existing.enabled,
                expires_at: existing.expires_at,
            };
            let old = existing.value.clone();
            configs.insert(key.to_string(), updated.clone());
            (updated, Some(old))
        } else {
            // 创建新配置
            let new_entry = ConfigEntry {
                key: key.to_string(),
                value: value.to_string(),
                version: 1,
                created_at: now,
                updated_at: now,
                labels: HashMap::new(),
                description: String::new(),
                encrypted: false,
                environment: environment.to_string(),
                application: application.to_string(),
                source: ConfigSource::Manual,
                enabled: true,
                expires_at: 0,
            };
            configs.insert(key.to_string(), new_entry.clone());
            (new_entry, None)
        };
        
        // 释放锁
        drop(configs);
        
        // 保存版本历史
        if self.config.enable_version_history {
            self.save_version_history(&entry);
        }
        
        // 触发变更事件
        if self.config.enable_change_events {
            let change_type = if old_value.is_some() {
                ConfigChangeType::Updated
            } else {
                ConfigChangeType::Created
            };
            self.trigger_change_event(&entry, old_value.as_deref(), change_type);
        }
        
        // 更新统计
        self.stats.total_changes.fetch_add(1, Ordering::Relaxed);
        
        entry
    }
    
    /// 获取配置
    /// 
    /// # 参数
    /// - `key`: 配置键
    /// 
    /// # 返回
    /// 配置值（如果存在）
    pub fn get(&self, key: &str) -> Option<String> {
        // 更新统计
        self.stats.total_queries.fetch_add(1, Ordering::Relaxed);
        
        let configs = self.configs.read().unwrap();
        configs.get(key).map(|e| {
            self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
            e.value.clone()
        })
    }
    
    /// 获取配置条目
    /// 
    /// # 参数
    /// - `key`: 配置键
    /// 
    /// # 返回
    /// 配置条目（如果存在）
    pub fn get_entry(&self, key: &str) -> Option<ConfigEntry> {
        let configs = self.configs.read().unwrap();
        configs.get(key).cloned()
    }
    
    /// 删除配置
    /// 
    /// # 参数
    /// - `key`: 配置键
    /// 
    /// # 返回
    /// 是否成功删除
    pub fn delete(&self, key: &str) -> bool {
        let mut configs = self.configs.write().unwrap();
        
        if let Some(entry) = configs.remove(key) {
            // 释放锁
            drop(configs);
            
            // 触发删除事件
            if self.config.enable_change_events {
                self.trigger_change_event(&entry, Some(&entry.value), ConfigChangeType::Deleted);
            }
            
            // 更新统计
            self.stats.total_changes.fetch_add(1, Ordering::Relaxed);
            
            true
        } else {
            false
        }
    }
    
    /// 检查配置是否存在
    /// 
    /// # 参数
    /// - `key`: 配置键
    /// 
    /// # 返回
    /// 是否存在
    pub fn exists(&self, key: &str) -> bool {
        let configs = self.configs.read().unwrap();
        configs.contains_key(key)
    }
    
    /// 获取所有配置键
    /// 
    /// # 返回
    /// 配置键列表
    pub fn keys(&self) -> Vec<String> {
        let configs = self.configs.read().unwrap();
        configs.keys().cloned().collect()
    }
    
    /// 获取指定前缀的所有配置
    /// 
    /// # 参数
    /// - `prefix`: 键前缀
    /// 
    /// # 返回
    /// 匹配的配置条目列表
    pub fn get_by_prefix(&self, prefix: &str) -> Vec<ConfigEntry> {
        let configs = self.configs.read().unwrap();
        configs.values()
            .filter(|e| e.key.starts_with(prefix))
            .cloned()
            .collect()
    }
    
    /// 获取指定环境的所有配置
    /// 
    /// # 参数
    /// - `environment`: 环境名称
    /// 
    /// # 返回
    /// 配置条目列表
    pub fn get_by_environment(&self, environment: &str) -> Vec<ConfigEntry> {
        let configs = self.configs.read().unwrap();
        configs.values()
            .filter(|e| e.environment == environment)
            .cloned()
            .collect()
    }
    
    /// 获取指定应用的所有配置
    /// 
    /// # 参数
    /// - `application`: 应用名称
    /// 
    /// # 返回
    /// 配置条目列表
    pub fn get_by_application(&self, application: &str) -> Vec<ConfigEntry> {
        let configs = self.configs.read().unwrap();
        configs.values()
            .filter(|e| e.application == application)
            .cloned()
            .collect()
    }
    
    /// 批量设置配置
    /// 
    /// # 参数
    /// - `entries`: 配置条目列表
    pub fn set_batch(&self, entries: Vec<ConfigEntry>) {
        for entry in entries {
            self.set(&entry.key, &entry.value, &entry.environment, &entry.application);
        }
    }
    
    /// 回滚配置到指定版本
    /// 
    /// # 参数
    /// - `key`: 配置键
    /// - `version`: 目标版本
    /// 
    /// # 返回
    /// 是否成功回滚
    pub fn rollback(&self, key: &str, version: u64) -> bool {
        // 获取版本历史
        let history = self.version_history.read().unwrap();
        
        if let Some(versions) = history.get(key) {
            // 查找目标版本
            if let Some(target) = versions.iter().find(|e| e.version == version) {
                let target_entry = target.clone();
                // 释放锁
                drop(history);
                
                // 回滚
                self.set(&target_entry.key, &target_entry.value, &target_entry.environment, &target_entry.application);
                return true;
            }
        }
        
        false
    }
    
    /// 获取配置版本历史
    /// 
    /// # 参数
    /// - `key`: 配置键
    /// 
    /// # 返回
    /// 版本历史列表
    pub fn get_version_history(&self, key: &str) -> Vec<ConfigEntry> {
        let history = self.version_history.read().unwrap();
        history.get(key).cloned().unwrap_or_default()
    }
    
    /// 注册配置监听器
    /// 
    /// # 参数
    /// - `watcher`: 配置监听器
    pub fn register_watcher(&self, watcher: ConfigWatcher) {
        let mut watchers = self.watchers.write().unwrap();
        watchers.push(watcher);
        self.stats.total_watchers.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 注销配置监听器
    /// 
    /// # 参数
    /// - `watcher_id`: 监听器ID
    /// 
    /// # 返回
    /// 是否成功注销
    pub fn unregister_watcher(&self, watcher_id: &str) -> bool {
        let mut watchers = self.watchers.write().unwrap();
        let len_before = watchers.len();
        watchers.retain(|w| w.id != watcher_id);
        
        if watchers.len() != len_before {
            self.stats.total_watchers.fetch_sub(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
    
    /// 获取变更事件
    /// 
    /// # 返回
    /// 变更事件列表
    pub fn get_change_events(&self) -> Vec<ConfigChangeEvent> {
        let events = self.change_events.read().unwrap();
        events.clone()
    }
    
    /// 清空变更事件
    pub fn clear_change_events(&self) {
        let mut events = self.change_events.write().unwrap();
        events.clear();
    }
    
    /// 保存版本历史
    fn save_version_history(&self, entry: &ConfigEntry) {
        let mut history = self.version_history.write().unwrap();
        
        // 获取或创建版本列表
        let versions = history.entry(entry.key.clone())
            .or_insert_with(Vec::new);
        
        // 添加新版本
        versions.push(entry.clone());
        
        // 限制历史数量
        if versions.len() > self.config.max_version_history {
            versions.remove(0);
        }
    }
    
    /// 触发变更事件
    fn trigger_change_event(&self, entry: &ConfigEntry, old_value: Option<&str>, change_type: ConfigChangeType) {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 创建事件
        let event = ConfigChangeEvent {
            event_id: format!("evt-{}-{}", entry.key, now),
            change_type,
            key: entry.key.clone(),
            old_value: old_value.map(|s| s.to_string()),
            new_value: Some(entry.value.clone()),
            timestamp: now,
            source: String::from("config-center"),
            environment: entry.environment.clone(),
            application: entry.application.clone(),
        };
        
        // 添加到事件队列
        {
            let mut events = self.change_events.write().unwrap();
            events.push(event);
            
            // 限制队列大小
            if events.len() > self.config.max_event_queue_size {
                events.remove(0);
            }
        }
        
        // 通知监听器
        self.notify_watchers(&entry.key);
    }
    
    /// 通知监听器
    fn notify_watchers(&self, key: &str) {
        let mut watchers = self.watchers.write().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        for watcher in watchers.iter_mut() {
            if watcher.enabled && watcher.matches(key) {
                watcher.last_triggered = now;
                watcher.trigger_count += 1;
            }
        }
    }
    
    /// 启用配置
    /// 
    /// # 参数
    /// - `key`: 配置键
    /// 
    /// # 返回
    /// 是否成功
    pub fn enable(&self, key: &str) -> bool {
        let mut configs = self.configs.write().unwrap();
        if let Some(entry) = configs.get_mut(key) {
            entry.enabled = true;
            return true;
        }
        false
    }
    
    /// 禁用配置
    /// 
    /// # 参数
    /// - `key`: 配置键
    /// 
    /// # 返回
    /// 是否成功
    pub fn disable(&self, key: &str) -> bool {
        let mut configs = self.configs.write().unwrap();
        if let Some(entry) = configs.get_mut(key) {
            entry.enabled = false;
            return true;
        }
        false
    }
    
    /// 获取统计信息
    pub fn stats(&self) -> &ConfigCenterStats {
        &self.stats
    }
    
    /// 启动配置中心
    pub fn start(&self) {
        self.running.store(true, Ordering::Relaxed);
    }
    
    /// 停止配置中心
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
    
    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
    
    /// 导出配置
    /// 
    /// # 返回
    /// 所有配置的JSON字符串
    pub fn export(&self) -> Result<String, String> {
        let configs = self.configs.read().unwrap();
        let entries: Vec<&ConfigEntry> = configs.values().collect();
        
        serde_json::to_string(&entries)
            .map_err(|e| format!("导出失败: {}", e))
    }
    
    /// 导入配置
    /// 
    /// # 参数
    /// - `json`: JSON字符串
    /// 
    /// # 返回
    /// 导入的配置数量
    pub fn import(&self, json: &str) -> Result<usize, String> {
        let entries: Vec<ConfigEntry> = serde_json::from_str(json)
            .map_err(|e| format!("导入失败: {}", e))?;
        
        let count = entries.len();
        self.set_batch(entries);
        
        Ok(count)
    }
}

/// 为 ConfigCenter 实现 Default trait
impl Default for ConfigCenter {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// 单元测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试配置设置和获取
    #[test]
    fn test_set_get() {
        let center = ConfigCenter::with_defaults();
        
        center.set("test.key", "test-value", "default", "default");
        
        let value = center.get("test.key");
        assert_eq!(value, Some("test-value".to_string()));
    }
    
    /// 测试配置删除
    #[test]
    fn test_delete() {
        let center = ConfigCenter::with_defaults();
        
        center.set("test.key", "test-value", "default", "default");
        assert!(center.exists("test.key"));
        
        let deleted = center.delete("test.key");
        assert!(deleted);
        assert!(!center.exists("test.key"));
    }
    
    /// 测试版本历史
    #[test]
    fn test_version_history() {
        let center = ConfigCenter::with_defaults();
        
        center.set("test.key", "value1", "default", "default");
        center.set("test.key", "value2", "default", "default");
        center.set("test.key", "value3", "default", "default");
        
        let history = center.get_version_history("test.key");
        assert_eq!(history.len(), 3);
    }
    
    /// 测试配置监听器
    #[test]
    fn test_watcher() {
        let center = ConfigCenter::with_defaults();
        
        let watcher = ConfigWatcher::new("test-watcher", "test.*");
        center.register_watcher(watcher);
        
        center.set("test.key", "value", "default", "default");
        
        let watchers = center.watchers.read().unwrap();
        assert_eq!(watchers[0].trigger_count, 1);
    }
    
    /// 测试配置导入导出
    #[test]
    fn test_import_export() {
        let center = ConfigCenter::with_defaults();
        
        center.set("key1", "value1", "default", "default");
        center.set("key2", "value2", "default", "default");
        
        let exported = center.export().unwrap();
        
        let new_center = ConfigCenter::with_defaults();
        let count = new_center.import(&exported).unwrap();
        
        assert_eq!(count, 2);
        assert_eq!(new_center.get("key1"), Some("value1".to_string()));
    }
}
