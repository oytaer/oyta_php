//! 软删除模块
//!
//! 提供模型软删除功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 软删除不会真正删除记录，而是标记为已删除

use crate::interpreter::value::Value;

/// 软删除类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SoftDeleteType {
    /// 使用 NULL 标记未删除
    Null,
    /// 使用时间戳标记
    Timestamp,
    /// 使用布尔值标记
    Boolean,
    /// 使用状态值标记
    Status,
}

impl Default for SoftDeleteType {
    fn default() -> Self {
        Self::Null
    }
}

/// 软删除配置
#[derive(Debug, Clone)]
pub struct SoftDeleteConfig {
    /// 是否启用软删除
    pub enabled: bool,
    /// 软删除字段名
    pub field: String,
    /// 软删除类型
    pub delete_type: SoftDeleteType,
    /// 已删除标记值
    pub deleted_value: Option<Value>,
    /// 未删除标记值
    pub not_deleted_value: Option<Value>,
}

impl Default for SoftDeleteConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            field: "deleted_at".to_string(),
            delete_type: SoftDeleteType::Null,
            deleted_value: None,
            not_deleted_value: Some(Value::Null),
        }
    }
}

impl SoftDeleteConfig {
    /// 创建新的软删除配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置是否启用
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// 设置软删除字段名
    pub fn field(mut self, field: &str) -> Self {
        self.field = field.to_string();
        self
    }

    /// 设置软删除类型
    pub fn delete_type(mut self, delete_type: SoftDeleteType) -> Self {
        self.delete_type = delete_type;
        self
    }

    /// 设置已删除标记值
    pub fn deleted_value(mut self, value: Value) -> Self {
        self.deleted_value = Some(value);
        self
    }

    /// 设置未删除标记值
    pub fn not_deleted_value(mut self, value: Value) -> Self {
        self.not_deleted_value = Some(value);
        self
    }

    /// 获取当前删除标记值
    pub fn get_deleted_value(&self) -> Value {
        match self.delete_type {
            SoftDeleteType::Null => {
                Value::String(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
            }
            SoftDeleteType::Timestamp => {
                Value::Int(chrono::Local::now().timestamp())
            }
            SoftDeleteType::Boolean => {
                Value::Bool(true)
            }
            SoftDeleteType::Status => {
                self.deleted_value.clone().unwrap_or(Value::Int(1))
            }
        }
    }

    /// 构建查询排除已删除记录的 WHERE 条件
    pub fn build_not_deleted_condition(&self) -> (String, Vec<Value>) {
        match self.delete_type {
            SoftDeleteType::Null | SoftDeleteType::Timestamp => {
                (format!("{} IS NULL", self.field), Vec::new())
            }
            SoftDeleteType::Boolean => {
                (format!("{} = ?", self.field), vec![Value::Bool(false)])
            }
            SoftDeleteType::Status => {
                let value = self.not_deleted_value.clone().unwrap_or(Value::Int(0));
                (format!("{} = ?", self.field), vec![value])
            }
        }
    }

    /// 构建查询仅包含已删除记录的 WHERE 条件
    pub fn build_only_deleted_condition(&self) -> (String, Vec<Value>) {
        match self.delete_type {
            SoftDeleteType::Null | SoftDeleteType::Timestamp => {
                (format!("{} IS NOT NULL", self.field), Vec::new())
            }
            SoftDeleteType::Boolean => {
                (format!("{} = ?", self.field), vec![Value::Bool(true)])
            }
            SoftDeleteType::Status => {
                let value = self.deleted_value.clone().unwrap_or(Value::Int(1));
                (format!("{} = ?", self.field), vec![value])
            }
        }
    }
}

/// 软删除管理器
pub struct SoftDeleteManager {
    /// 配置
    pub config: SoftDeleteConfig,
}

impl SoftDeleteManager {
    /// 创建新的软删除管理器
    pub fn new(config: SoftDeleteConfig) -> Self {
        Self { config }
    }

    /// 获取软删除字段名
    pub fn field(&self) -> &str {
        &self.config.field
    }

    /// 获取删除标记值
    pub fn deleted_value(&self) -> Value {
        self.config.get_deleted_value()
    }

    /// 构建排除已删除记录的条件
    pub fn build_not_deleted_where(&self) -> (String, Vec<Value>) {
        self.config.build_not_deleted_condition()
    }

    /// 构建仅包含已删除记录的条件
    pub fn build_only_deleted_where(&self) -> (String, Vec<Value>) {
        self.config.build_only_deleted_condition()
    }

    /// 判断记录是否已删除
    pub fn is_deleted(&self, data: &std::collections::HashMap<String, Value>) -> bool {
        match data.get(&self.config.field) {
            Some(Value::Null) => false,
            Some(Value::Bool(b)) => *b,
            Some(Value::Int(i)) => *i != 0,
            Some(Value::String(_)) => true,
            None => false,
            _ => false,
        }
    }
}

/// 可恢复删除 trait
pub trait Restorable {
    /// 恢复已删除的记录
    fn restore(&mut self);

    /// 强制删除（真正删除）
    fn force_delete(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soft_delete_config() {
        let config = SoftDeleteConfig::new();
        let (condition, _) = config.build_not_deleted_condition();
        assert!(condition.contains("IS NULL"));
    }

    #[test]
    fn test_soft_delete_boolean() {
        let config = SoftDeleteConfig::new()
            .delete_type(SoftDeleteType::Boolean);

        let (condition, bindings) = config.build_not_deleted_condition();
        assert!(condition.contains("= ?"));
        assert_eq!(bindings.len(), 1);
    }

    #[test]
    fn test_soft_delete_status() {
        let config = SoftDeleteConfig::new()
            .delete_type(SoftDeleteType::Status)
            .deleted_value(Value::Int(2))
            .not_deleted_value(Value::Int(0));

        let (condition, bindings) = config.build_not_deleted_condition();
        assert_eq!(bindings[0], Value::Int(0));
    }

    #[test]
    fn test_is_deleted() {
        let manager = SoftDeleteManager::new(SoftDeleteConfig::new());

        let mut deleted_record = std::collections::HashMap::new();
        deleted_record.insert("deleted_at".to_string(), Value::String("2024-01-01".to_string()));
        assert!(manager.is_deleted(&deleted_record));

        let mut active_record = std::collections::HashMap::new();
        active_record.insert("deleted_at".to_string(), Value::Null);
        assert!(!manager.is_deleted(&active_record));
    }
}
