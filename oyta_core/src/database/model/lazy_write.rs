//! 延迟写入模块
//!
//! 实现模型的延迟写入功能
//! 对应 ThinkPHP 的延迟写入功能
//!
//! 主要功能：
//! - 延迟写入字段配置
//! - 批量写入优化
//! - 延迟队列管理

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::database::executor;
use crate::interpreter::value::Value;
use super::instance::ModelInstance;
use super::types::ModelConfig;

/// 延迟写入配置
#[derive(Debug, Clone)]
pub struct LazyWriteConfig {
    /// 延迟写入的字段列表
    pub lazy_fields: Vec<String>,
    /// 延迟时间（毫秒）
    pub delay_ms: u64,
    /// 批量写入阈值
    pub batch_threshold: usize,
}

impl LazyWriteConfig {
    /// 创建新的延迟写入配置
    pub fn new() -> Self {
        Self {
            lazy_fields: Vec::new(),
            delay_ms: 1000,
            batch_threshold: 100,
        }
    }

    /// 添加延迟写入字段
    pub fn add_field(&mut self, field: &str) {
        if !self.lazy_fields.contains(&field.to_string()) {
            self.lazy_fields.push(field.to_string());
        }
    }

    /// 设置延迟时间
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// 设置批量写入阈值
    pub fn with_batch_threshold(mut self, threshold: usize) -> Self {
        self.batch_threshold = threshold;
        self
    }

    /// 检查字段是否为延迟写入字段
    pub fn is_lazy_field(&self, field: &str) -> bool {
        self.lazy_fields.contains(&field.to_string())
    }
}

impl Default for LazyWriteConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 延迟写入项
#[derive(Debug, Clone)]
pub struct LazyWriteItem {
    /// 表名
    pub table: String,
    /// 主键值
    pub id: Value,
    /// 字段名
    pub field: String,
    /// 新值
    pub value: Value,
    /// 创建时间
    pub created_at: Instant,
}

/// 延迟写入队列
#[derive(Debug, Default)]
pub struct LazyWriteQueue {
    /// 待写入项列表
    items: Vec<LazyWriteItem>,
    /// 配置
    config: LazyWriteConfig,
}

impl LazyWriteQueue {
    /// 创建新的延迟写入队列
    pub fn new(config: LazyWriteConfig) -> Self {
        Self {
            items: Vec::new(),
            config,
        }
    }

    /// 添加延迟写入项
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `id`: 主键值
    /// - `field`: 字段名
    /// - `value`: 新值
    pub fn push(&mut self, table: &str, id: Value, field: &str, value: Value) {
        // 检查是否为延迟写入字段
        if !self.config.is_lazy_field(field) {
            return;
        }

        // 检查是否已存在相同的项
        let existing = self.items.iter_mut().find(|item| {
            item.table == table && item.id == id && item.field == field
        });

        if let Some(item) = existing {
            // 更新已存在的项
            item.value = value;
        } else {
            // 添加新项
            self.items.push(LazyWriteItem {
                table: table.to_string(),
                id,
                field: field.to_string(),
                value,
                created_at: Instant::now(),
            });
        }

        // 检查是否达到批量写入阈值
        if self.items.len() >= self.config.batch_threshold {
            // 触发批量写入
            // 注意：这里需要异步执行，简化处理
        }
    }

    /// 获取待写入项数量
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 检查是否有需要写入的项
    pub fn has_pending(&self) -> bool {
        !self.items.is_empty()
    }

    /// 获取过期的项（超过延迟时间）
    pub fn get_expired_items(&self) -> Vec<&LazyWriteItem> {
        let now = Instant::now();
        let delay = Duration::from_millis(self.config.delay_ms);

        self.items
            .iter()
            .filter(|item| now.duration_since(item.created_at) >= delay)
            .collect()
    }

    /// 执行批量写入
    ///
    /// # 返回值
    /// 写入成功返回 true
    pub async fn flush(&mut self) -> anyhow::Result<bool> {
        if self.items.is_empty() {
            return Ok(true);
        }

        // 按表分组
        let mut grouped: HashMap<String, Vec<&LazyWriteItem>> = HashMap::new();
        for item in &self.items {
            grouped.entry(item.table.clone()).or_insert_with(Vec::new).push(item);
        }

        // 执行批量更新
        for (table, items) in grouped {
            // 按 ID 分组
            let mut by_id: HashMap<String, Vec<&LazyWriteItem>> = HashMap::new();
            for item in items {
                let id_key = item.id.to_string_value();
                by_id.entry(id_key).or_insert_with(Vec::new).push(item);
            }

            // 执行更新
            for (id_key, fields) in by_id {
                // 构建 UPDATE SQL
                let set_parts: Vec<String> = fields
                    .iter()
                    .map(|item| format!("{} = ?", item.field))
                    .collect();

                let sql = format!(
                    "UPDATE {} SET {} WHERE id = ?",
                    table,
                    set_parts.join(", ")
                );

                // 构建参数
                let mut params: Vec<Value> = fields.iter().map(|item| item.value.clone()).collect();
                params.push(fields[0].id.clone());

                // 执行更新
                executor::execute_with_params(&sql, &params).await?;
            }
        }

        // 清空队列
        self.items.clear();

        Ok(true)
    }

    /// 清空队列（不执行写入）
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

/// 延迟写入管理器
pub struct LazyWriteManager {
    /// 延迟写入队列
    queue: LazyWriteQueue,
}

impl LazyWriteManager {
    /// 创建新的延迟写入管理器
    pub fn new(config: LazyWriteConfig) -> Self {
        Self {
            queue: LazyWriteQueue::new(config),
        }
    }

    /// 延迟写入字段
    ///
    /// # 参数
    /// - `model`: 模型实例
    /// - `field`: 字段名
    /// - `value`: 新值
    pub fn lazy_write(&mut self, model: &ModelInstance, field: &str, value: Value) {
        let table = model.config.full_table();
        let id = model.get_key();

        self.queue.push(&table, id, field, value);
    }

    /// 获取队列
    pub fn get_queue(&self) -> &LazyWriteQueue {
        &self.queue
    }

    /// 获取可变队列
    pub fn get_queue_mut(&mut self) -> &mut LazyWriteQueue {
        &mut self.queue
    }

    /// 执行所有延迟写入
    pub async fn flush_all(&mut self) -> anyhow::Result<bool> {
        self.queue.flush().await
    }
}

/// 为 ModelInstance 添加延迟写入扩展方法
impl ModelInstance {
    /// 延迟写入字段
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `value`: 新值
    /// - `manager`: 延迟写入管理器
    pub fn lazy_set(&self, field: &str, value: Value, manager: &mut LazyWriteManager) {
        manager.lazy_write(self, field, value);
    }

    /// 检查字段是否为延迟写入字段
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `config`: 延迟写入配置
    pub fn is_lazy_field(&self, field: &str, config: &LazyWriteConfig) -> bool {
        config.is_lazy_field(field)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_write_config() {
        let mut config = LazyWriteConfig::new()
            .with_delay(500)
            .with_batch_threshold(50);

        config.add_field("views");
        config.add_field("clicks");

        assert!(config.is_lazy_field("views"));
        assert!(config.is_lazy_field("clicks"));
        assert!(!config.is_lazy_field("name"));
    }

    #[test]
    fn test_lazy_write_queue() {
        // 创建配置并添加延迟写入字段
        let mut config = LazyWriteConfig::new();
        config.add_field("views");
        config.add_field("clicks");

        let mut queue = LazyWriteQueue::new(config);

        queue.push("users", Value::Int(1), "views", Value::Int(100));
        queue.push("users", Value::Int(1), "clicks", Value::Int(50));

        assert_eq!(queue.len(), 2);
        assert!(queue.has_pending());
    }

    #[test]
    fn test_lazy_write_queue_update() {
        let mut config = LazyWriteConfig::new();
        config.add_field("views");

        let mut queue = LazyWriteQueue::new(config);

        // 添加相同字段的更新
        queue.push("users", Value::Int(1), "views", Value::Int(100));
        queue.push("users", Value::Int(1), "views", Value::Int(200));

        // 应该只保留最后一次更新
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_lazy_write_manager() {
        // 创建配置并添加延迟写入字段
        let mut config = LazyWriteConfig::new();
        config.add_field("views");

        let mut manager = LazyWriteManager::new(config);

        let model_config = ModelConfig {
            table: "users".to_string(),
            ..ModelConfig::default()
        };
        let mut model = ModelInstance::new(model_config);
        model.attributes.insert("id".to_string(), Value::Int(1));
        model.exists = true;

        manager.lazy_write(&model, "views", Value::Int(100));

        assert!(manager.get_queue().has_pending());
    }
}
