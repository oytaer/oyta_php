//! 模型事件系统模块
//!
//! 实现模型的完整事件机制
//! 对应 ThinkPHP 的模型事件功能
//!
//! 支持的事件：
//! - onAfterRead: 查询后事件
//! - onBeforeInsert: 新增前事件
//! - onAfterInsert: 新增后事件
//! - onBeforeUpdate: 更新前事件
//! - onAfterUpdate: 更新后事件
//! - onBeforeWrite: 写入前事件（新增+更新）
//! - onAfterWrite: 写入后事件
//! - onBeforeDelete: 删除前事件
//! - onAfterDelete: 删除后事件
//! - onBeforeRestore: 恢复前事件
//! - onAfterRestore: 恢复后事件

use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use crate::interpreter::value::Value;
use super::instance::ModelInstance;

/// 事件类型枚举
///
/// 定义所有支持的模型事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelEventType {
    /// 查询后事件
    /// 在模型数据从数据库读取后触发
    AfterRead,
    /// 新增前事件
    /// 在模型插入数据库前触发
    /// 如果事件处理返回 false，则取消插入操作
    BeforeInsert,
    /// 新增后事件
    /// 在模型插入数据库后触发
    AfterInsert,
    /// 更新前事件
    /// 在模型更新数据库前触发
    /// 如果事件处理返回 false，则取消更新操作
    BeforeUpdate,
    /// 更新后事件
    /// 在模型更新数据库后触发
    AfterUpdate,
    /// 写入前事件
    /// 在模型新增或更新前都会触发
    /// 如果事件处理返回 false，则取消写入操作
    BeforeWrite,
    /// 写入后事件
    /// 在模型新增或更新后都会触发
    AfterWrite,
    /// 删除前事件
    /// 在模型删除前触发
    /// 如果事件处理返回 false，则取消删除操作
    BeforeDelete,
    /// 删除后事件
    /// 在模型删除后触发
    AfterDelete,
    /// 恢复前事件
    /// 在软删除模型恢复前触发
    /// 如果事件处理返回 false，则取消恢复操作
    BeforeRestore,
    /// 恢复后事件
    /// 在软删除模型恢复后触发
    AfterRestore,
}

impl ModelEventType {
    /// 获取事件名称
    ///
    /// # 返回值
    /// 事件名称字符串
    pub fn name(&self) -> &'static str {
        match self {
            ModelEventType::AfterRead => "onAfterRead",
            ModelEventType::BeforeInsert => "onBeforeInsert",
            ModelEventType::AfterInsert => "onAfterInsert",
            ModelEventType::BeforeUpdate => "onBeforeUpdate",
            ModelEventType::AfterUpdate => "onAfterUpdate",
            ModelEventType::BeforeWrite => "onBeforeWrite",
            ModelEventType::AfterWrite => "onAfterWrite",
            ModelEventType::BeforeDelete => "onBeforeDelete",
            ModelEventType::AfterDelete => "onAfterDelete",
            ModelEventType::BeforeRestore => "onBeforeRestore",
            ModelEventType::AfterRestore => "onAfterRestore",
        }
    }

    /// 从字符串解析事件类型
    ///
    /// # 参数
    /// - `name`: 事件名称
    ///
    /// # 返回值
    /// 解析成功返回事件类型，否则返回 None
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "onAfterRead" | "AfterRead" => Some(ModelEventType::AfterRead),
            "onBeforeInsert" | "BeforeInsert" => Some(ModelEventType::BeforeInsert),
            "onAfterInsert" | "AfterInsert" => Some(ModelEventType::AfterInsert),
            "onBeforeUpdate" | "BeforeUpdate" => Some(ModelEventType::BeforeUpdate),
            "onAfterUpdate" | "AfterUpdate" => Some(ModelEventType::AfterUpdate),
            "onBeforeWrite" | "BeforeWrite" => Some(ModelEventType::BeforeWrite),
            "onAfterWrite" | "AfterWrite" => Some(ModelEventType::AfterWrite),
            "onBeforeDelete" | "BeforeDelete" => Some(ModelEventType::BeforeDelete),
            "onAfterDelete" | "AfterDelete" => Some(ModelEventType::AfterDelete),
            "onBeforeRestore" | "BeforeRestore" => Some(ModelEventType::BeforeRestore),
            "onAfterRestore" | "AfterRestore" => Some(ModelEventType::AfterRestore),
            _ => None,
        }
    }

    /// 判断是否为前置事件
    ///
    /// # 返回值
    /// 如果是前置事件返回 true
    pub fn is_before(&self) -> bool {
        matches!(
            self,
            ModelEventType::BeforeInsert
                | ModelEventType::BeforeUpdate
                | ModelEventType::BeforeWrite
                | ModelEventType::BeforeDelete
                | ModelEventType::BeforeRestore
        )
    }

    /// 判断是否为后置事件
    ///
    /// # 返回值
    /// 如果是后置事件返回 true
    pub fn is_after(&self) -> bool {
        !self.is_before()
    }
}

/// 事件处理结果
///
/// 用于控制事件是否继续传播
#[derive(Debug, Clone)]
pub enum EventResult {
    /// 继续执行
    Continue,
    /// 停止执行（取消操作）
    Stop,
    /// 返回值
    Value(Value),
}

impl Default for EventResult {
    fn default() -> Self {
        EventResult::Continue
    }
}

/// 事件回调类型（同步）
///
/// 事件处理函数的类型定义
pub type EventCallback = Box<dyn Fn(&mut ModelInstance) -> EventResult + Send + Sync>;

/// 事件回调类型（异步）
///
/// 异步事件处理函数的类型定义
pub type AsyncEventCallback = Box<dyn Fn(ModelInstance) -> Pin<Box<dyn Future<Output = (ModelInstance, EventResult)> + Send>> + Send + Sync>;

/// 模型事件监听器
///
/// 存储单个事件的所有监听器
pub struct ModelEventListener {
    /// 同步监听器列表
    sync_listeners: Vec<EventCallback>,
    /// 异步监听器列表
    async_listeners: Vec<AsyncEventCallback>,
}

/// 手动实现 Debug trait，因为闭包不支持 Debug
impl fmt::Debug for ModelEventListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 格式化输出监听器信息
        f.debug_struct("ModelEventListener")
            .field("sync_listeners", &format!("{} sync listeners", self.sync_listeners.len()))
            .field("async_listeners", &format!("{} async listeners", self.async_listeners.len()))
            .finish()
    }
}

/// 实现 Default trait
impl Default for ModelEventListener {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelEventListener {
    /// 创建新的事件监听器
    pub fn new() -> Self {
        Self {
            sync_listeners: Vec::new(),
            async_listeners: Vec::new(),
        }
    }

    /// 添加同步监听器
    ///
    /// # 参数
    /// - `callback`: 回调函数
    pub fn add_sync(&mut self, callback: EventCallback) {
        self.sync_listeners.push(callback);
    }

    /// 添加异步监听器
    ///
    /// # 参数
    /// - `callback`: 异步回调函数
    pub fn add_async(&mut self, callback: AsyncEventCallback) {
        self.async_listeners.push(callback);
    }

    /// 触发同步事件
    ///
    /// # 参数
    /// - `model`: 模型实例
    ///
    /// # 返回值
    /// 事件处理结果
    pub fn trigger_sync(&self, model: &mut ModelInstance) -> EventResult {
        // 遍历所有同步监听器
        for listener in &self.sync_listeners {
            // 执行监听器
            let result = listener(model);

            // 如果返回 Stop，则停止传播
            if matches!(result, EventResult::Stop) {
                return EventResult::Stop;
            }
        }

        EventResult::Continue
    }

    /// 触发异步事件
    ///
    /// # 参数
    /// - `model`: 模型实例
    ///
    /// # 返回值
    /// 异步事件处理结果
    pub async fn trigger_async(&self, model: ModelInstance) -> (ModelInstance, EventResult) {
        let mut current_model = model;

        // 遍历所有异步监听器
        for listener in &self.async_listeners {
            // 执行异步监听器
            let (new_model, result) = listener(current_model).await;
            current_model = new_model;

            // 如果返回 Stop，则停止传播
            if matches!(result, EventResult::Stop) {
                return (current_model, EventResult::Stop);
            }
        }

        (current_model, EventResult::Continue)
    }

    /// 清空所有监听器
    pub fn clear(&mut self) {
        self.sync_listeners.clear();
        self.async_listeners.clear();
    }

    /// 获取监听器数量
    pub fn count(&self) -> usize {
        self.sync_listeners.len() + self.async_listeners.len()
    }
}

/// 模型事件管理器
///
/// 管理所有模型的事件监听器
#[derive(Debug, Default)]
pub struct ModelEventManager {
    /// 事件监听器映射
    /// 键：模型类名 + 事件类型
    listeners: HashMap<String, ModelEventListener>,
    /// 全局事件监听器
    /// 对所有模型生效
    global_listeners: HashMap<ModelEventType, ModelEventListener>,
    /// 是否禁用事件
    disabled: bool,
}

impl ModelEventManager {
    /// 创建新的事件管理器
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
            global_listeners: HashMap::new(),
            disabled: false,
        }
    }

    /// 获取事件键名
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `event_type`: 事件类型
    ///
    /// # 返回值
    /// 组合后的键名
    fn get_event_key(model_class: &str, event_type: ModelEventType) -> String {
        format!("{}.{}", model_class, event_type.name())
    }

    /// 注册同步事件监听器
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `event_type`: 事件类型
    /// - `callback`: 回调函数
    ///
    /// # 示例
    /// ```php
    /// Event::listen('app\model\User.BeforeUpdate', function($user) {
    ///     // 处理逻辑
    /// });
    /// ```
    pub fn on<F>(&mut self, model_class: &str, event_type: ModelEventType, callback: F)
    where
        F: Fn(&mut ModelInstance) -> EventResult + Send + Sync + 'static,
    {
        // 获取事件键名
        let key = Self::get_event_key(model_class, event_type);

        // 获取或创建监听器
        let listener = self.listeners.entry(key).or_insert_with(ModelEventListener::new);

        // 添加监听器
        listener.add_sync(Box::new(callback));
    }

    /// 注册异步事件监听器
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `event_type`: 事件类型
    /// - `callback`: 异步回调函数
    pub fn on_async<F, Fut>(&mut self, model_class: &str, event_type: ModelEventType, callback: F)
    where
        F: Fn(ModelInstance) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = (ModelInstance, EventResult)> + Send + 'static,
    {
        // 获取事件键名
        let key = Self::get_event_key(model_class, event_type);

        // 获取或创建监听器
        let listener = self.listeners.entry(key).or_insert_with(ModelEventListener::new);

        // 添加异步监听器
        listener.add_async(Box::new(move |model| Box::pin(callback(model))));
    }

    /// 注册全局事件监听器
    ///
    /// # 参数
    /// - `event_type`: 事件类型
    /// - `callback`: 回调函数
    ///
    /// # 示例
    /// ```php
    /// // 对所有模型的 BeforeInsert 事件生效
    /// Event::listen('BeforeInsert', function($model) {
    ///     // 处理逻辑
    /// });
    /// ```
    pub fn on_global<F>(&mut self, event_type: ModelEventType, callback: F)
    where
        F: Fn(&mut ModelInstance) -> EventResult + Send + Sync + 'static,
    {
        // 获取或创建全局监听器
        let listener = self.global_listeners.entry(event_type).or_insert_with(ModelEventListener::new);

        // 添加监听器
        listener.add_sync(Box::new(callback));
    }

    /// 触发事件（同步）
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `event_type`: 事件类型
    /// - `model`: 模型实例
    ///
    /// # 返回值
    /// 事件处理结果
    pub fn trigger(&self, model_class: &str, event_type: ModelEventType, model: &mut ModelInstance) -> EventResult {
        // 检查是否禁用事件
        if self.disabled {
            return EventResult::Continue;
        }

        // 首先触发全局监听器
        if let Some(global_listener) = self.global_listeners.get(&event_type) {
            let result = global_listener.trigger_sync(model);
            if matches!(result, EventResult::Stop) {
                return EventResult::Stop;
            }
        }

        // 然后触发模型特定监听器
        let key = Self::get_event_key(model_class, event_type);
        if let Some(listener) = self.listeners.get(&key) {
            let result = listener.trigger_sync(model);
            if matches!(result, EventResult::Stop) {
                return EventResult::Stop;
            }
        }

        EventResult::Continue
    }

    /// 触发事件（异步）
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `event_type`: 事件类型
    /// - `model`: 模型实例
    ///
    /// # 返回值
    /// 异步事件处理结果
    pub async fn trigger_async(&self, model_class: &str, event_type: ModelEventType, model: ModelInstance) -> (ModelInstance, EventResult) {
        // 检查是否禁用事件
        if self.disabled {
            return (model, EventResult::Continue);
        }

        let mut current_model = model;

        // 首先触发全局监听器
        if let Some(global_listener) = self.global_listeners.get(&event_type) {
            let (new_model, result) = global_listener.trigger_async(current_model).await;
            current_model = new_model;
            if matches!(result, EventResult::Stop) {
                return (current_model, EventResult::Stop);
            }
        }

        // 然后触发模型特定监听器
        let key = Self::get_event_key(model_class, event_type);
        if let Some(listener) = self.listeners.get(&key) {
            let (new_model, result) = listener.trigger_async(current_model).await;
            current_model = new_model;
            if matches!(result, EventResult::Stop) {
                return (current_model, EventResult::Stop);
            }
        }

        (current_model, EventResult::Continue)
    }

    /// 禁用事件
    ///
    /// # 示例
    /// ```php
    /// $user->withEvent(false)->save();
    /// ```
    pub fn disable(&mut self) {
        self.disabled = true;
    }

    /// 启用事件
    pub fn enable(&mut self) {
        self.disabled = false;
    }

    /// 检查事件是否启用
    pub fn is_enabled(&self) -> bool {
        !self.disabled
    }

    /// 移除事件监听器
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `event_type`: 事件类型
    pub fn off(&mut self, model_class: &str, event_type: ModelEventType) {
        let key = Self::get_event_key(model_class, event_type);
        self.listeners.remove(&key);
    }

    /// 移除全局事件监听器
    ///
    /// # 参数
    /// - `event_type`: 事件类型
    pub fn off_global(&mut self, event_type: ModelEventType) {
        self.global_listeners.remove(&event_type);
    }

    /// 清空所有监听器
    pub fn clear_all(&mut self) {
        self.listeners.clear();
        self.global_listeners.clear();
    }

    /// 获取事件监听器数量
    pub fn count(&self, model_class: &str, event_type: ModelEventType) -> usize {
        let key = Self::get_event_key(model_class, event_type);
        let model_count = self.listeners.get(&key).map(|l| l.count()).unwrap_or(0);
        let global_count = self.global_listeners.get(&event_type).map(|l| l.count()).unwrap_or(0);
        model_count + global_count
    }
}

/// 全局事件管理器实例
///
/// 使用线程安全的单例模式
static EVENT_MANAGER: once_cell::sync::Lazy<RwLock<ModelEventManager>> =
    once_cell::sync::Lazy::new(|| RwLock::new(ModelEventManager::new()));

/// 获取全局事件管理器
///
/// # 返回值
/// 全局事件管理器的读写锁
pub fn get_event_manager() -> &'static RwLock<ModelEventManager> {
    &EVENT_MANAGER
}

/// 注册事件监听器（便捷函数）
///
/// # 参数
/// - `model_class`: 模型类名
/// - `event_type`: 事件类型
/// - `callback`: 回调函数
pub fn register_event<F>(model_class: &str, event_type: ModelEventType, callback: F)
where
    F: Fn(&mut ModelInstance) -> EventResult + Send + Sync + 'static,
{
    // 获取可写引用
    if let Ok(mut manager) = EVENT_MANAGER.write() {
        manager.on(model_class, event_type, callback);
    }
}

/// 触发事件（便捷函数）
///
/// # 参数
/// - `model_class`: 模型类名
/// - `event_type`: 事件类型
/// - `model`: 模型实例
///
/// # 返回值
/// 事件处理结果
pub fn fire_event(model_class: &str, event_type: ModelEventType, model: &mut ModelInstance) -> EventResult {
    // 获取可读引用
    if let Ok(manager) = EVENT_MANAGER.read() {
        manager.trigger(model_class, event_type, model)
    } else {
        EventResult::Continue
    }
}

/// 模型观察者 trait
///
/// 用于将模型事件集中管理
/// 对应 ThinkPHP 的模型观察者
///
/// # 示例
/// ```php
/// class UserObserver {
///     public function onBeforeUpdate(User $user) {
///         // 处理逻辑
///     }
///     
///     public function onAfterDelete(User $user) {
///         // 处理逻辑
///     }
/// }
/// ```
pub trait ModelObserver: Send + Sync {
    /// 查询后事件
    fn on_after_read(&self, _model: &mut ModelInstance) -> EventResult {
        EventResult::Continue
    }

    /// 新增前事件
    fn on_before_insert(&self, _model: &mut ModelInstance) -> EventResult {
        EventResult::Continue
    }

    /// 新增后事件
    fn on_after_insert(&self, _model: &mut ModelInstance) -> EventResult {
        EventResult::Continue
    }

    /// 更新前事件
    fn on_before_update(&self, _model: &mut ModelInstance) -> EventResult {
        EventResult::Continue
    }

    /// 更新后事件
    fn on_after_update(&self, _model: &mut ModelInstance) -> EventResult {
        EventResult::Continue
    }

    /// 写入前事件
    fn on_before_write(&self, _model: &mut ModelInstance) -> EventResult {
        EventResult::Continue
    }

    /// 写入后事件
    fn on_after_write(&self, _model: &mut ModelInstance) -> EventResult {
        EventResult::Continue
    }

    /// 删除前事件
    fn on_before_delete(&self, _model: &mut ModelInstance) -> EventResult {
        EventResult::Continue
    }

    /// 删除后事件
    fn on_after_delete(&self, _model: &mut ModelInstance) -> EventResult {
        EventResult::Continue
    }

    /// 恢复前事件
    fn on_before_restore(&self, _model: &mut ModelInstance) -> EventResult {
        EventResult::Continue
    }

    /// 恢复后事件
    fn on_after_restore(&self, _model: &mut ModelInstance) -> EventResult {
        EventResult::Continue
    }
}

/// 注册模型观察者
///
/// # 参数
/// - `model_class`: 模型类名
/// - `observer`: 观察者实例
pub fn register_observer(model_class: &str, observer: Arc<dyn ModelObserver>) {
    // 获取可写引用
    if let Ok(mut manager) = EVENT_MANAGER.write() {
        // 注册所有事件
        let obs = observer.clone();
        manager.on(model_class, ModelEventType::AfterRead, move |model| obs.on_after_read(model));

        let obs = observer.clone();
        manager.on(model_class, ModelEventType::BeforeInsert, move |model| obs.on_before_insert(model));

        let obs = observer.clone();
        manager.on(model_class, ModelEventType::AfterInsert, move |model| obs.on_after_insert(model));

        let obs = observer.clone();
        manager.on(model_class, ModelEventType::BeforeUpdate, move |model| obs.on_before_update(model));

        let obs = observer.clone();
        manager.on(model_class, ModelEventType::AfterUpdate, move |model| obs.on_after_update(model));

        let obs = observer.clone();
        manager.on(model_class, ModelEventType::BeforeWrite, move |model| obs.on_before_write(model));

        let obs = observer.clone();
        manager.on(model_class, ModelEventType::AfterWrite, move |model| obs.on_after_write(model));

        let obs = observer.clone();
        manager.on(model_class, ModelEventType::BeforeDelete, move |model| obs.on_before_delete(model));

        let obs = observer.clone();
        manager.on(model_class, ModelEventType::AfterDelete, move |model| obs.on_after_delete(model));

        let obs = observer.clone();
        manager.on(model_class, ModelEventType::BeforeRestore, move |model| obs.on_before_restore(model));

        let obs = observer.clone();
        manager.on(model_class, ModelEventType::AfterRestore, move |model| obs.on_after_restore(model));
    }
}

/// 模型事件异常
///
/// 当事件处理需要中断操作时抛出
#[derive(Debug)]
pub struct ModelEventException {
    /// 异常消息
    pub message: String,
    /// 事件类型
    pub event_type: ModelEventType,
    /// 模型类名
    pub model_class: String,
}

impl std::fmt::Display for ModelEventException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Model event exception: {} on {} for model {}",
            self.message,
            self.event_type.name(),
            self.model_class
        )
    }
}

impl std::error::Error for ModelEventException {}

impl ModelEventException {
    /// 创建新的模型事件异常
    ///
    /// # 参数
    /// - `message`: 异常消息
    /// - `event_type`: 事件类型
    /// - `model_class`: 模型类名
    pub fn new(message: &str, event_type: ModelEventType, model_class: &str) -> Self {
        Self {
            message: message.to_string(),
            event_type,
            model_class: model_class.to_string(),
        }
    }
}

/// 事件调度器
///
/// 用于在模型操作中调度事件
pub struct EventDispatcher;

impl EventDispatcher {
    /// 分发查询后事件
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `model`: 模型实例
    pub fn dispatch_after_read(model_class: &str, model: &mut ModelInstance) {
        fire_event(model_class, ModelEventType::AfterRead, model);
    }

    /// 分发新增前事件
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `model`: 模型实例
    ///
    /// # 返回值
    /// 是否继续执行
    pub fn dispatch_before_insert(model_class: &str, model: &mut ModelInstance) -> bool {
        let result = fire_event(model_class, ModelEventType::BeforeInsert, model);
        !matches!(result, EventResult::Stop)
    }

    /// 分发新增后事件
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `model`: 模型实例
    pub fn dispatch_after_insert(model_class: &str, model: &mut ModelInstance) {
        fire_event(model_class, ModelEventType::AfterInsert, model);
    }

    /// 分发更新前事件
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `model`: 模型实例
    ///
    /// # 返回值
    /// 是否继续执行
    pub fn dispatch_before_update(model_class: &str, model: &mut ModelInstance) -> bool {
        let result = fire_event(model_class, ModelEventType::BeforeUpdate, model);
        !matches!(result, EventResult::Stop)
    }

    /// 分发更新后事件
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `model`: 模型实例
    pub fn dispatch_after_update(model_class: &str, model: &mut ModelInstance) {
        fire_event(model_class, ModelEventType::AfterUpdate, model);
    }

    /// 分发写入前事件
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `model`: 模型实例
    ///
    /// # 返回值
    /// 是否继续执行
    pub fn dispatch_before_write(model_class: &str, model: &mut ModelInstance) -> bool {
        let result = fire_event(model_class, ModelEventType::BeforeWrite, model);
        !matches!(result, EventResult::Stop)
    }

    /// 分发写入后事件
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `model`: 模型实例
    pub fn dispatch_after_write(model_class: &str, model: &mut ModelInstance) {
        fire_event(model_class, ModelEventType::AfterWrite, model);
    }

    /// 分发删除前事件
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `model`: 模型实例
    ///
    /// # 返回值
    /// 是否继续执行
    pub fn dispatch_before_delete(model_class: &str, model: &mut ModelInstance) -> bool {
        let result = fire_event(model_class, ModelEventType::BeforeDelete, model);
        !matches!(result, EventResult::Stop)
    }

    /// 分发删除后事件
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `model`: 模型实例
    pub fn dispatch_after_delete(model_class: &str, model: &mut ModelInstance) {
        fire_event(model_class, ModelEventType::AfterDelete, model);
    }

    /// 分发恢复前事件
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `model`: 模型实例
    ///
    /// # 返回值
    /// 是否继续执行
    pub fn dispatch_before_restore(model_class: &str, model: &mut ModelInstance) -> bool {
        let result = fire_event(model_class, ModelEventType::BeforeRestore, model);
        !matches!(result, EventResult::Stop)
    }

    /// 分发恢复后事件
    ///
    /// # 参数
    /// - `model_class`: 模型类名
    /// - `model`: 模型实例
    pub fn dispatch_after_restore(model_class: &str, model: &mut ModelInstance) {
        fire_event(model_class, ModelEventType::AfterRestore, model);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ModelConfig;

    fn create_test_model() -> ModelInstance {
        let config = ModelConfig {
            table: "users".to_string(),
            ..ModelConfig::default()
        };
        let mut model = ModelInstance::new(config);
        model.attributes.insert("id".to_string(), Value::Int(1));
        model.attributes.insert("name".to_string(), Value::String("test".to_string()));
        model
    }

    #[test]
    fn test_event_type() {
        assert_eq!(ModelEventType::BeforeInsert.name(), "onBeforeInsert");
        assert_eq!(ModelEventType::AfterRead.name(), "onAfterRead");

        assert!(ModelEventType::BeforeInsert.is_before());
        assert!(!ModelEventType::AfterInsert.is_before());
    }

    #[test]
    fn test_event_manager() {
        let mut manager = ModelEventManager::new();

        // 注册事件
        manager.on("User", ModelEventType::BeforeUpdate, |model| {
            if model.get_attr("name").to_string_value() == "thinkphp" {
                return EventResult::Stop;
            }
            EventResult::Continue
        });

        // 触发事件 - 正常情况
        let mut model1 = create_test_model();
        let result1 = manager.trigger("User", ModelEventType::BeforeUpdate, &mut model1);
        assert!(matches!(result1, EventResult::Continue));

        // 触发事件 - 停止情况
        let mut model2 = create_test_model();
        model2.attributes.insert("name".to_string(), Value::String("thinkphp".to_string()));
        let result2 = manager.trigger("User", ModelEventType::BeforeUpdate, &mut model2);
        assert!(matches!(result2, EventResult::Stop));
    }

    #[test]
    fn test_event_disable() {
        let mut manager = ModelEventManager::new();

        // 注册事件
        manager.on("User", ModelEventType::BeforeInsert, |_model| {
            EventResult::Stop
        });

        // 禁用事件
        manager.disable();

        // 触发事件 - 应该被跳过
        let mut model = create_test_model();
        let result = manager.trigger("User", ModelEventType::BeforeInsert, &mut model);
        assert!(matches!(result, EventResult::Continue));

        // 启用事件
        manager.enable();

        // 触发事件 - 应该执行
        let result = manager.trigger("User", ModelEventType::BeforeInsert, &mut model);
        assert!(matches!(result, EventResult::Stop));
    }

    #[test]
    fn test_global_event() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let mut manager = ModelEventManager::new();

        // 注册全局事件（使用 Arc<AtomicUsize> 来在闭包中计数）
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        manager.on_global(ModelEventType::BeforeInsert, move |_model| {
            // 使用原子操作增加计数器
            counter_clone.fetch_add(1, Ordering::SeqCst);
            EventResult::Continue
        });

        // 触发全局事件
        let mut model = create_test_model();
        manager.trigger("User", ModelEventType::BeforeInsert, &mut model);
        manager.trigger("Article", ModelEventType::BeforeInsert, &mut model);

        // 验证计数器增加了两次
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
}
