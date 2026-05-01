//! 模型事件模块
//!
//! 提供模型事件监听功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：插入前/后、更新前/后、删除前/后等事件

use crate::interpreter::value::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 事件类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    /// 插入前
    BeforeInsert,
    /// 插入后
    AfterInsert,
    /// 更新前
    BeforeUpdate,
    /// 更新后
    AfterUpdate,
    /// 删除前
    BeforeDelete,
    /// 删除后
    AfterDelete,
    /// 软删除前
    BeforeSoftDelete,
    /// 软删除后
    AfterSoftDelete,
    /// 恢复前
    BeforeRestore,
    /// 恢复后
    AfterRestore,
    /// 查询后
    AfterFind,
}

impl EventType {
    /// 获取事件名称
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::BeforeInsert => "beforeInsert",
            EventType::AfterInsert => "afterInsert",
            EventType::BeforeUpdate => "beforeUpdate",
            EventType::AfterUpdate => "afterUpdate",
            EventType::BeforeDelete => "beforeDelete",
            EventType::AfterDelete => "afterDelete",
            EventType::BeforeSoftDelete => "beforeSoftDelete",
            EventType::AfterSoftDelete => "afterSoftDelete",
            EventType::BeforeRestore => "beforeRestore",
            EventType::AfterRestore => "afterRestore",
            EventType::AfterFind => "afterFind",
        }
    }
}

/// 事件上下文
#[derive(Debug, Clone)]
pub struct EventContext {
    /// 模型名称
    pub model_name: String,
    /// 事件类型
    pub event_type: EventType,
    /// 数据
    pub data: HashMap<String, Value>,
    /// 原始数据（用于更新操作）
    pub original: Option<HashMap<String, Value>>,
    /// 是否取消操作
    pub cancelled: bool,
    /// 错误信息
    pub error: Option<String>,
}

impl EventContext {
    /// 创建新的事件上下文
    pub fn new(model_name: &str, event_type: EventType, data: HashMap<String, Value>) -> Self {
        Self {
            model_name: model_name.to_string(),
            event_type,
            data,
            original: None,
            cancelled: false,
            error: None,
        }
    }

    /// 设置原始数据
    pub fn with_original(mut self, original: HashMap<String, Value>) -> Self {
        self.original = Some(original);
        self
    }

    /// 取消操作
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    /// 设置错误
    pub fn set_error(&mut self, error: &str) {
        self.error = Some(error.to_string());
        self.cancelled = true;
    }

    /// 获取数据值
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// 设置数据值
    pub fn set(&mut self, key: &str, value: Value) {
        self.data.insert(key.to_string(), value);
    }
}

/// 事件监听器类型
pub type EventListener = Box<dyn Fn(&mut EventContext) + Send + Sync>;

/// 事件管理器
pub struct EventManager {
    /// 监听器列表
    listeners: Arc<RwLock<HashMap<EventType, Vec<EventListener>>>>,
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EventManager {
    /// 创建新的事件管理器
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册事件监听器
    pub fn listen<F>(&self, event_type: EventType, listener: F)
    where
        F: Fn(&mut EventContext) + Send + Sync + 'static,
    {
        let mut listeners = self.listeners.write().unwrap();
        listeners
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(Box::new(listener));
    }

    /// 触发事件
    pub fn fire(&self, context: &mut EventContext) {
        let listeners = self.listeners.read().unwrap();
        if let Some(event_listeners) = listeners.get(&context.event_type) {
            for listener in event_listeners {
                listener(context);
                if context.cancelled {
                    break;
                }
            }
        }
    }

    /// 移除事件监听器
    pub fn forget(&self, event_type: EventType) {
        let mut listeners = self.listeners.write().unwrap();
        listeners.remove(&event_type);
    }

    /// 清空所有监听器
    pub fn clear(&self) {
        let mut listeners = self.listeners.write().unwrap();
        listeners.clear();
    }
}

/// 模型事件 trait
pub trait ModelEvents {
    /// 插入前事件
    fn before_insert(&mut self, _ctx: &mut EventContext) {}

    /// 插入后事件
    fn after_insert(&mut self, _ctx: &mut EventContext) {}

    /// 更新前事件
    fn before_update(&mut self, _ctx: &mut EventContext) {}

    /// 更新后事件
    fn after_update(&mut self, _ctx: &mut EventContext) {}

    /// 删除前事件
    fn before_delete(&mut self, _ctx: &mut EventContext) {}

    /// 删除后事件
    fn after_delete(&mut self, _ctx: &mut EventContext) {}
}

/// 全局事件管理器
pub static GLOBAL_EVENT_MANAGER: once_cell::sync::Lazy<EventManager> =
    once_cell::sync::Lazy::new(EventManager::new);

/// 注册全局事件监听器
pub fn register_event_listener<F>(event_type: EventType, listener: F)
where
    F: Fn(&mut EventContext) + Send + Sync + 'static,
{
    GLOBAL_EVENT_MANAGER.listen(event_type, listener);
}

/// 触发全局事件
pub fn fire_event(context: &mut EventContext) {
    GLOBAL_EVENT_MANAGER.fire(context);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_context() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), Value::String("test".to_string()));

        let ctx = EventContext::new("User", EventType::BeforeInsert, data);
        assert_eq!(ctx.model_name, "User");
        assert_eq!(ctx.event_type, EventType::BeforeInsert);
    }

    #[test]
    fn test_event_context_cancel() {
        let data = HashMap::new();
        let mut ctx = EventContext::new("User", EventType::BeforeInsert, data);

        ctx.cancel();
        assert!(ctx.cancelled);
    }

    #[test]
    fn test_event_manager() {
        let manager = EventManager::new();
        let called = Arc::new(RwLock::new(false));

        let called_clone = called.clone();
        manager.listen(EventType::BeforeInsert, move |_ctx| {
            *called_clone.write().unwrap() = true;
        });

        let data = HashMap::new();
        let mut ctx = EventContext::new("User", EventType::BeforeInsert, data);
        manager.fire(&mut ctx);

        assert!(*called.read().unwrap());
    }

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(EventType::BeforeInsert.as_str(), "beforeInsert");
        assert_eq!(EventType::AfterUpdate.as_str(), "afterUpdate");
    }
}
