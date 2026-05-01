//! 自动时间戳模块
//!
//! 提供模型自动时间戳功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 自动管理 created_at 和 updated_at 字段

use crate::interpreter::value::Value;
use chrono::{DateTime, Local, Utc};

/// 时间戳类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimestampType {
    /// 整数时间戳（秒）
    Integer,
    /// 整数时间戳（毫秒）
    IntegerMillisecond,
    /// 日期时间字符串
    DateTime,
    /// ISO 8601 格式
    Iso8601,
}

impl Default for TimestampType {
    fn default() -> Self {
        Self::DateTime
    }
}

/// 自动时间戳配置
#[derive(Debug, Clone)]
pub struct AutoTimestampConfig {
    /// 是否启用自动时间戳
    pub enabled: bool,
    /// 创建时间字段名
    pub created_at_field: String,
    /// 更新时间字段名
    pub updated_at_field: String,
    /// 时间戳类型
    pub timestamp_type: TimestampType,
    /// 时区
    pub timezone: Timezone,
}

impl Default for AutoTimestampConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            created_at_field: "created_at".to_string(),
            updated_at_field: "updated_at".to_string(),
            timestamp_type: TimestampType::DateTime,
            timezone: Timezone::Local,
        }
    }
}

/// 时区枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Timezone {
    /// UTC 时区
    Utc,
    /// 本地时区
    Local,
}

impl AutoTimestampConfig {
    /// 创建新的自动时间戳配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置是否启用
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// 设置创建时间字段名
    pub fn created_at_field(mut self, field: &str) -> Self {
        self.created_at_field = field.to_string();
        self
    }

    /// 设置更新时间字段名
    pub fn updated_at_field(mut self, field: &str) -> Self {
        self.updated_at_field = field.to_string();
        self
    }

    /// 设置时间戳类型
    pub fn timestamp_type(mut self, timestamp_type: TimestampType) -> Self {
        self.timestamp_type = timestamp_type;
        self
    }

    /// 获取当前时间值
    pub fn current_time(&self) -> Value {
        let now = match self.timezone {
            Timezone::Utc => Utc::now(),
            Timezone::Local => Local::now().with_timezone(&Utc),
        };

        match self.timestamp_type {
            TimestampType::Integer => {
                Value::Int(now.timestamp())
            }
            TimestampType::IntegerMillisecond => {
                Value::Int(now.timestamp_millis())
            }
            TimestampType::DateTime => {
                Value::String(now.format("%Y-%m-%d %H:%M:%S").to_string())
            }
            TimestampType::Iso8601 => {
                Value::String(now.to_rfc3339())
            }
        }
    }

    /// 为插入数据添加时间戳
    pub fn add_timestamps_for_insert(&self, data: &mut std::collections::HashMap<String, Value>) {
        if !self.enabled {
            return;
        }

        let now = self.current_time();

        // 只有字段不存在时才设置
        if !data.contains_key(&self.created_at_field) {
            data.insert(self.created_at_field.clone(), now.clone());
        }
        if !data.contains_key(&self.updated_at_field) {
            data.insert(self.updated_at_field.clone(), now);
        }
    }

    /// 为更新数据添加时间戳
    pub fn add_timestamps_for_update(&self, data: &mut std::collections::HashMap<String, Value>) {
        if !self.enabled {
            return;
        }

        let now = self.current_time();
        data.insert(self.updated_at_field.clone(), now);
    }
}

/// 时间戳管理器
pub struct TimestampManager {
    /// 配置
    pub config: AutoTimestampConfig,
}

impl TimestampManager {
    /// 创建新的时间戳管理器
    pub fn new(config: AutoTimestampConfig) -> Self {
        Self { config }
    }

    /// 获取创建时间字段名
    pub fn created_at_field(&self) -> &str {
        &self.config.created_at_field
    }

    /// 获取更新时间字段名
    pub fn updated_at_field(&self) -> &str {
        &self.config.updated_at_field
    }

    /// 处理插入操作
    pub fn handle_insert(&self, data: &mut std::collections::HashMap<String, Value>) {
        self.config.add_timestamps_for_insert(data);
    }

    /// 处理更新操作
    pub fn handle_update(&self, data: &mut std::collections::HashMap<String, Value>) {
        self.config.add_timestamps_for_update(data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_timestamp_config() {
        let config = AutoTimestampConfig::new();
        let mut data = std::collections::HashMap::new();
        data.insert("name".to_string(), Value::String("test".to_string()));

        config.add_timestamps_for_insert(&mut data);

        assert!(data.contains_key("created_at"));
        assert!(data.contains_key("updated_at"));
    }

    #[test]
    fn test_auto_timestamp_disabled() {
        let config = AutoTimestampConfig::new().enabled(false);
        let mut data = std::collections::HashMap::new();
        data.insert("name".to_string(), Value::String("test".to_string()));

        config.add_timestamps_for_insert(&mut data);

        assert!(!data.contains_key("created_at"));
    }

    #[test]
    fn test_custom_field_names() {
        let config = AutoTimestampConfig::new()
            .created_at_field("create_time")
            .updated_at_field("update_time");

        let mut data = std::collections::HashMap::new();
        config.add_timestamps_for_insert(&mut data);

        assert!(data.contains_key("create_time"));
        assert!(data.contains_key("update_time"));
    }
}
