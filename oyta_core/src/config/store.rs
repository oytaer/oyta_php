//! 配置存储核心模块
//!
//! 提供配置值的存储和访问接口
//! 支持点号路径访问（如 "database.connections.mysql.host"）
//! 支持类型安全的配置值获取
//! 使用 dashmap 实现并发安全的配置存储

use dashmap::DashMap;
use std::sync::Arc;

use crate::symbol_table::types::ConfigValue;

/// 配置存储
/// 使用 DashMap 实现并发安全的配置键值存储
/// 键为点号分隔的配置路径，值为 ConfigValue
#[derive(Debug, Clone)]
pub struct ConfigStore {
    /// 配置数据存储
    /// 键格式: "app.debug"、"database.default" 等
    data: Arc<DashMap<String, ConfigValue>>,
}

impl ConfigStore {
    /// 创建新的空配置存储
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }

    /// 设置配置值
    /// 支持点号路径，如 "app.debug"
    pub fn set(&self, key: &str, value: ConfigValue) {
        self.data.insert(key.to_string(), value);
    }

    /// 获取配置值
    pub fn get(&self, key: &str) -> Option<ConfigValue> {
        // 先尝试直接获取
        if let Some(val) = self.data.get(key) {
            return Some(val.value().clone());
        }

        // 尝试点号路径查找
        // 如 "database.connections.mysql.host" 会在
        // "database" → AssociativeArray → "connections" → AssociativeArray → "mysql" → ... 中查找
        let parts: Vec<&str> = key.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        // 获取第一级键
        let first = self.data.get(parts[0])?;
        let mut current = first.value().clone();

        // 逐级深入
        for part in &parts[1..] {
            current = current.get(part)?.clone();
        }

        Some(current)
    }

    /// 获取字符串配置值
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|v| match v {
            ConfigValue::String(s) => Some(s),
            _ => None,
        })
    }

    /// 获取字符串配置值，带默认值
    pub fn get_string_or(&self, key: &str, default: &str) -> String {
        self.get_string(key).unwrap_or_else(|| default.to_string())
    }

    /// 获取整数配置值
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_i64())
    }

    /// 获取整数配置值，带默认值
    pub fn get_int_or(&self, key: &str, default: i64) -> i64 {
        self.get_int(key).unwrap_or(default)
    }

    /// 获取布尔配置值
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }

    /// 获取布尔配置值，带默认值
    pub fn get_bool_or(&self, key: &str, default: bool) -> bool {
        self.get_bool(key).unwrap_or(default)
    }

    /// 获取浮点数配置值
    pub fn get_float(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| match v {
            ConfigValue::Float(f) => Some(f),
            _ => None,
        })
    }

    /// 检查配置键是否存在
    pub fn has(&self, key: &str) -> bool {
        self.data.contains_key(key) || self.get(key).is_some()
    }

    /// 获取所有顶级配置键
    pub fn keys(&self) -> Vec<String> {
        self.data.iter().map(|e| e.key().clone()).collect()
    }

    /// 批量设置配置（从关联数组展开为点号路径）
    /// 递归展开嵌套的关联数组
    pub fn set_from_associative(&self, prefix: &str, values: &[(String, ConfigValue)]) {
        for (key, value) in values {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };

            match value {
                ConfigValue::AssociativeArray(nested) => {
                    // 递归展开嵌套数组
                    self.set_from_associative(&full_key, nested);
                    // 同时存储原始的嵌套值
                    self.set(&full_key, value.clone());
                }
                _ => {
                    self.set(&full_key, value.clone());
                }
            }
        }
    }

    /// 清空所有配置
    pub fn clear(&self) {
        self.data.clear();
    }

    /// 获取配置项数量
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self::new()
    }
}
