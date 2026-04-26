//! 事件调度器
//!
//! 实现观察者模式的事件调度
//! 支持通过解释器执行 PHP 监听器的 handle 方法
//! 对应 ThinkPHP 8.0 的 Event 类

use dashmap::DashMap;
use std::sync::Arc;

use crate::interpreter::engine::Interpreter;
use crate::interpreter::value::Value;
use crate::symbol_table::registry::SymbolRegistry;
use super::types::{EventDef, EventResult, ListenerDef};

/// 事件调度器
///
/// 管理事件的注册和触发
/// 监听器按优先级排序执行
/// 支持一次性事件和事件中断
pub struct EventDispatcher {
    /// 事件映射（事件名 → 事件定义）
    events: Arc<DashMap<String, EventDef>>,
    /// 符号表注册表
    registry: Arc<SymbolRegistry>,
}

impl EventDispatcher {
    /// 创建新的事件调度器
    ///
    /// # 参数
    /// - `registry`: 符号表注册表，用于查找监听器类定义
    pub fn new(registry: Arc<SymbolRegistry>) -> Self {
        Self {
            events: Arc::new(DashMap::new()),
            registry,
        }
    }

    /// 注册事件监听器
    ///
    /// 将监听器添加到指定事件的监听器列表中
    /// 监听器按优先级自动排序
    ///
    /// # 参数
    /// - `event_name`: 事件名称
    /// - `listener_class`: 监听器类名
    /// - `priority`: 优先级（数字越小越先执行）
    pub fn listen(&self, event_name: &str, listener_class: &str, priority: i32) {
        let listener = ListenerDef {
            class: listener_class.to_string(),
            priority,
            disabled: false,
        };
        self.events
            .entry(event_name.to_string())
            .and_modify(|e| {
                e.listeners.push(listener.clone());
                e.listeners.sort_by_key(|l| l.priority);
            })
            .or_insert_with(|| EventDef {
                name: event_name.to_string(),
                listeners: vec![listener],
                once: false,
                triggered: false,
            });
    }

    /// 注册一次性事件监听器
    ///
    /// 事件触发一次后自动移除
    ///
    /// # 参数
    /// - `event_name`: 事件名称
    /// - `listener_class`: 监听器类名
    pub fn once(&self, event_name: &str, listener_class: &str) {
        let listener = ListenerDef {
            class: listener_class.to_string(),
            priority: 0,
            disabled: false,
        };
        self.events
            .entry(event_name.to_string())
            .and_modify(|e| {
                e.listeners.push(listener.clone());
                e.once = true;
                e.listeners.sort_by_key(|l| l.priority);
            })
            .or_insert_with(|| EventDef {
                name: event_name.to_string(),
                listeners: vec![listener],
                once: true,
                triggered: false,
            });
    }

    /// 触发事件
    ///
    /// 按优先级顺序执行所有监听器的 handle 方法
    /// 通过解释器执行 PHP 监听器
    ///
    /// # 参数
    /// - `event_name`: 事件名称
    /// - `data`: 事件数据（JSON 格式）
    /// - `interpreter`: PHP 解释器实例
    ///
    /// # 返回
    /// 事件触发结果
    pub fn trigger(
        &self,
        event_name: &str,
        data: &serde_json::Value,
        interpreter: &mut Interpreter,
    ) -> EventResult {
        let mut result = EventResult::new(event_name);

        if let Some(mut event) = self.events.get_mut(event_name) {
            if event.once && event.triggered {
                return result;
            }

            let event_data_value = json_to_value(data);

            for listener in &event.listeners {
                if listener.disabled {
                    continue;
                }

                tracing::debug!("触发事件监听器: {} → {}", event_name, listener.class);

                match self.execute_listener(&listener.class, event_data_value.clone(), interpreter) {
                    Ok(should_stop) => {
                        result.success_count += 1;
                        if should_stop {
                            result.stopped = true;
                            break;
                        }
                    }
                    Err(e) => {
                        result.failure_count += 1;
                        result.errors.push(format!("{}: {}", listener.class, e));
                        tracing::warn!("事件监听器执行失败: {} - {}", listener.class, e);
                    }
                }
            }

            if event.once {
                event.triggered = true;
            }
        }

        result
    }

    /// 执行单个监听器
    ///
    /// 通过解释器执行 PHP 监听器的 handle 方法
    /// PHP 监听器的 handle 方法签名：
    /// ```php
    /// public function handle($event): void
    /// ```
    ///
    /// 如果 handle 方法返回 false，则中断后续监听器执行
    ///
    /// # 参数
    /// - `class_name`: 监听器类名
    /// - `data`: 事件数据
    /// - `interpreter`: PHP 解释器
    ///
    /// # 返回
    /// 是否中断后续监听器执行（true=中断）
    fn execute_listener(
        &self,
        class_name: &str,
        data: Value,
        interpreter: &mut Interpreter,
    ) -> Result<bool, anyhow::Error> {
        if let Some(class_def) = self.registry.find_class(class_name) {
            if class_def.find_method("handle").is_some() {
                match interpreter.execute_controller_method(class_name, "handle", vec![data], None) {
                    Ok(result) => {
                        match &result {
                            Value::Bool(false) => Ok(true),
                            _ => Ok(false),
                        }
                    }
                    Err(e) => Err(e),
                }
            } else {
                tracing::debug!("监听器类无 handle 方法，跳过: {}", class_name);
                Ok(false)
            }
        } else {
            tracing::debug!("监听器类未找到，跳过: {}", class_name);
            Ok(false)
        }
    }

    /// 获取事件的所有监听器
    ///
    /// # 参数
    /// - `event_name`: 事件名称
    ///
    /// # 返回
    /// 监听器列表
    pub fn get_listeners(&self, event_name: &str) -> Vec<ListenerDef> {
        self.events
            .get(event_name)
            .map(|e| e.listeners.clone())
            .unwrap_or_default()
    }

    /// 移除事件的所有监听器
    ///
    /// # 参数
    /// - `event_name`: 事件名称
    pub fn remove(&self, event_name: &str) {
        self.events.remove(event_name);
    }

    /// 检查事件是否存在
    ///
    /// # 参数
    /// - `event_name`: 事件名称
    pub fn has(&self, event_name: &str) -> bool {
        self.events.contains_key(event_name)
    }

    /// 获取所有已注册的事件名
    pub fn event_names(&self) -> Vec<String> {
        self.events.iter().map(|e| e.key().clone()).collect()
    }

    /// 从符号表自动发现事件监听器
    ///
    /// 扫描符号表中所有实现了 Listener 接口的类
    pub fn discover_from_registry(&self) {
        let listeners = self.registry.get_listeners();
        for listener_class in listeners {
            for method in &listener_class.methods {
                if method.name == "handle" {
                    let event_name = listener_class
                        .namespace
                        .as_ref()
                        .map(|ns| format!("{}\\{}", ns, listener_class.name))
                        .unwrap_or_else(|| listener_class.name.clone());
                    self.listen(&event_name, &listener_class.full_name, 0);
                }
            }
        }
    }

    /// 禁用指定监听器
    ///
    /// # 参数
    /// - `event_name`: 事件名称
    /// - `listener_class`: 监听器类名
    pub fn disable_listener(&self, event_name: &str, listener_class: &str) {
        if let Some(mut event) = self.events.get_mut(event_name) {
            for listener in &mut event.listeners {
                if listener.class == listener_class {
                    listener.disabled = true;
                }
            }
        }
    }

    /// 注册事件订阅器
    ///
    /// 事件订阅器是一个类，通过 subscribe 方法定义多个事件监听
    /// 对应 ThinkPHP 8.0 中的 Event::subscribe() 功能
    ///
    /// PHP 订阅器示例：
    /// ```php
    /// class UserEventSubscriber {
    ///     public function subscribe($event) {
    ///         $event->listen('UserCreated', [self::class, 'onUserCreated']);
    ///         $event->listen('UserDeleted', [self::class, 'onUserDeleted']);
    ///     }
    /// }
    /// ```
    ///
    /// # 参数
    /// - `subscriber_class`: 订阅器类名（含命名空间）
    pub fn subscribe(&self, subscriber_class: &str) {
        // 从符号表查找订阅器类
        if let Some(class_def) = self.registry.find_class(subscriber_class) {
            if class_def.find_method("subscribe").is_some() {
                // 遍历订阅器类的方法，自动注册以 "on" 开头的方法为事件监听器
                for method in &class_def.methods {
                    if method.name.starts_with("on") && method.name != "subscribe" {
                        // 将方法名转换为事件名
                        // onUserCreated → UserCreated
                        let event_name = method.name[2..].to_string();
                        self.listen(&event_name, &format!("{}@{}", subscriber_class, method.name), 0);
                    }
                }
            }
        }
    }

    /// 从配置文件加载事件定义
    ///
    /// 解析 config/event.php 中的事件配置
    /// ThinkPHP 8.0 的事件配置格式：
    /// ```php
    /// return [
    ///     'listen' => [
    ///         'UserCreated' => ['app\listener\UserCreated'],
    ///     ],
    ///     'subscribe' => [
    ///         'app\subscribe\UserEventSubscriber',
    ///     ],
    /// ];
    /// ```
    ///
    /// # 参数
    /// - `listen_config`: listen 配置（事件名 → 监听器列表）
    /// - `subscribe_config`: subscribe 配置（订阅器类名列表）
    pub fn load_from_config(
        &self,
        listen_config: &std::collections::HashMap<String, Vec<String>>,
        subscribe_config: &[String],
    ) {
        // 注册 listen 配置中的事件监听器
        for (event_name, listeners) in listen_config {
            for (priority, listener_class) in listeners.iter().enumerate() {
                self.listen(event_name, listener_class, priority as i32);
            }
        }

        // 注册 subscribe 配置中的订阅器
        for subscriber_class in subscribe_config {
            self.subscribe(subscriber_class);
        }
    }

    /// 异步触发事件
    ///
    /// 在单独的 Tokio 任务中触发事件
    /// 适用于不需要同步等待结果的场景
    ///
    /// # 参数
    /// - `event_name`: 事件名称
    /// - `data`: 事件数据
    pub fn trigger_async(
        &self,
        event_name: String,
        _data: serde_json::Value,
    ) {
        let events = self.events.clone();
        let _registry = self.registry.clone();

        tokio::spawn(async move {
            if let Some(event) = events.get(&event_name) {
                if event.once && event.triggered {
                    return;
                }

                tracing::debug!("异步触发事件: {}", event_name);

                // 异步事件暂时只记录日志
                // 实际的 PHP 解释器执行需要在同步上下文中进行
                for listener in &event.listeners {
                    if !listener.disabled {
                        tracing::info!("异步事件监听器: {} → {}", event_name, listener.class);
                    }
                }

                if event.once {
                    drop(event);
                    if let Some(mut event) = events.get_mut(&event_name) {
                        event.triggered = true;
                    }
                }
            }
        });
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new(Arc::new(SymbolRegistry::new()))
    }
}

/// 将 JSON 值转换为解释器 Value
///
/// 递归转换 JSON 数据结构为 Value 枚举
///
/// # 参数
/// - `json`: JSON 值引用
///
/// # 返回
/// 对应的 Value
fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            Value::IndexedArray(arr.iter().map(json_to_value).collect())
        }
        serde_json::Value::Object(map) => Value::AssociativeArray(
            map.iter()
                .map(|(k, v)| (k.clone(), json_to_value(v)))
                .collect(),
        ),
    }
}
