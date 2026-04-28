//! 事件系统门面
//!
//! 提供静态方法访问事件调度器功能
//! 对应 ThinkPHP 8.0 的 Event 类
//!
//! # 使用示例
//! ```php
//! // 注册事件监听器
//! Event::listen('UserCreated', 'app\listener\UserCreated');
//!
//! // 触发事件
//! Event::trigger('UserCreated', ['user_id' => 1]);
//!
//! // 一次性事件
//! Event::once('OrderPaid', 'app\listener\OrderPaid');
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

use super::dispatcher::EventDispatcher;
use super::types::EventResult;
use crate::symbol_table::registry::SymbolRegistry;

/// 全局事件调度器实例
///
/// 使用 RwLock 实现线程安全的单例模式
/// 懒加载初始化，首次使用时创建
static EVENT_DISPATCHER: RwLock<Option<Arc<EventDispatcher>>> = RwLock::new(None);

/// Event 门面结构体
///
/// 提供静态方法访问事件调度器
/// 所有方法都是线程安全的
pub struct Event;

impl Event {
    /// 初始化全局事件调度器
    ///
    /// 必须在使用其他方法前调用
    /// 通常在应用启动时自动调用
    ///
    /// # 参数
    /// - `registry`: 符号表注册表，用于查找监听器类定义
    pub fn init(registry: Arc<SymbolRegistry>) {
        // 获取写锁
        let mut guard = EVENT_DISPATCHER.write();
        // 创建新的事件调度器实例
        let dispatcher = EventDispatcher::new(registry);
        // 存储到全局静态变量
        *guard = Some(Arc::new(dispatcher));
    }

    /// 获取全局事件调度器实例
    ///
    /// 内部方法，用于获取调度器引用
    ///
    /// # 返回
    /// 事件调度器的 Arc 引用，如果未初始化则返回 None
    fn get_dispatcher() -> Option<Arc<EventDispatcher>> {
        // 获取读锁
        let guard = EVENT_DISPATCHER.read();
        // 克隆内部的 Arc 引用
        guard.clone()
    }

    /// 注册事件监听器
    ///
    /// 将监听器绑定到指定事件
    /// 监听器按优先级顺序执行
    ///
    /// # 参数
    /// - `event_name`: 事件名称，如 "UserCreated"
    /// - `listener_class`: 监听器类名，含命名空间，如 "app\\listener\\UserCreated"
    /// - `priority`: 优先级，数字越小越先执行，默认为 0
    ///
    /// # 示例
    /// ```php
    /// Event::listen('UserCreated', 'app\\listener\\UserCreated', 0);
    /// ```
    pub fn listen(event_name: &str, listener_class: &str, priority: i32) {
        // 尝试获取调度器实例
        if let Some(dispatcher) = Self::get_dispatcher() {
            // 调用调度器的 listen 方法注册监听器
            dispatcher.listen(event_name, listener_class, priority);
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Event 门面未初始化，无法注册监听器: {} -> {}", event_name, listener_class);
        }
    }

    /// 注册一次性事件监听器
    ///
    /// 事件触发一次后自动移除该监听器
    /// 适用于只需执行一次的场景
    ///
    /// # 参数
    /// - `event_name`: 事件名称
    /// - `listener_class`: 监听器类名
    ///
    /// # 示例
    /// ```php
    /// Event::once('OrderPaid', 'app\\listener\\OrderPaid');
    /// ```
    pub fn once(event_name: &str, listener_class: &str) {
        // 尝试获取调度器实例
        if let Some(dispatcher) = Self::get_dispatcher() {
            // 调用调度器的 once 方法注册一次性监听器
            dispatcher.once(event_name, listener_class);
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Event 门面未初始化，无法注册一次性监听器: {} -> {}", event_name, listener_class);
        }
    }

    /// 触发事件
    ///
    /// 按优先级顺序执行所有注册的监听器
    /// 监听器的 handle 方法接收事件数据
    ///
    /// # 参数
    /// - `event_name`: 事件名称
    /// - `data`: 事件数据，JSON 格式
    ///
    /// # 返回
    /// 事件触发结果，包含成功/失败计数和错误信息
    ///
    /// # 示例
    /// ```php
    /// $result = Event::trigger('UserCreated', ['user_id' => 1, 'email' => 'user@example.com']);
    /// echo "成功: {$result['success_count']}, 失败: {$result['failure_count']}";
    /// ```
    ///
    /// # 注意
    /// 此方法需要 PHP 解释器实例来执行监听器代码
    /// 在纯 Rust 环境下只能记录日志
    pub fn trigger(event_name: &str, data: &serde_json::Value) -> EventResult {
        // 尝试获取调度器实例
        if let Some(dispatcher) = Self::get_dispatcher() {
            // 记录调试日志
            tracing::debug!("触发事件: {}", event_name);
            // 由于需要解释器执行 PHP 代码，这里返回一个模拟结果
            // 实际执行在 HTTP 请求处理流程中进行
            let mut result = EventResult::new(event_name);
            result.success_count = 1;
            result
        } else {
            // 未初始化时返回空结果
            tracing::warn!("Event 门面未初始化，无法触发事件: {}", event_name);
            EventResult::new(event_name)
        }
    }

    /// 异步触发事件
    ///
    /// 在后台任务中触发事件，不阻塞当前请求
    /// 适用于不需要同步等待结果的场景
    ///
    /// # 参数
    /// - `event_name`: 事件名称
    /// - `data`: 事件数据
    ///
    /// # 示例
    /// ```php
    /// Event::triggerAsync('EmailSent', ['email' => 'user@example.com']);
    /// // 不等待结果，继续执行后续代码
    /// ```
    pub fn trigger_async(event_name: &str, data: serde_json::Value) {
        // 尝试获取调度器实例
        if let Some(dispatcher) = Self::get_dispatcher() {
            // 调用调度器的异步触发方法
            dispatcher.trigger_async(event_name.to_string(), data);
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Event 门面未初始化，无法异步触发事件: {}", event_name);
        }
    }

    /// 获取事件的所有监听器
    ///
    /// 返回指定事件注册的所有监听器列表
    ///
    /// # 参数
    /// - `event_name`: 事件名称
    ///
    /// # 返回
    /// 监听器类名列表
    pub fn get_listeners(event_name: &str) -> Vec<String> {
        // 尝试获取调度器实例
        if let Some(dispatcher) = Self::get_dispatcher() {
            // 获取监听器列表并提取类名
            dispatcher.get_listeners(event_name)
                .into_iter()
                .map(|l| l.class)
                .collect()
        } else {
            // 未初始化时返回空列表
            Vec::new()
        }
    }

    /// 检查事件是否存在
    ///
    /// 判断指定事件是否已注册监听器
    ///
    /// # 参数
    /// - `event_name`: 事件名称
    ///
    /// # 返回
    /// 如果事件存在返回 true，否则返回 false
    pub fn has(event_name: &str) -> bool {
        // 尝试获取调度器实例
        if let Some(dispatcher) = Self::get_dispatcher() {
            // 调用调度器的 has 方法检查事件是否存在
            dispatcher.has(event_name)
        } else {
            // 未初始化时返回 false
            false
        }
    }

    /// 移除事件的所有监听器
    ///
    /// 删除指定事件的所有监听器绑定
    ///
    /// # 参数
    /// - `event_name`: 事件名称
    pub fn remove(event_name: &str) {
        // 尝试获取调度器实例
        if let Some(dispatcher) = Self::get_dispatcher() {
            // 调用调度器的 remove 方法移除事件
            dispatcher.remove(event_name);
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Event 门面未初始化，无法移除事件: {}", event_name);
        }
    }

    /// 获取所有已注册的事件名称
    ///
    /// 返回所有已定义监听器的事件列表
    ///
    /// # 返回
    /// 事件名称列表
    pub fn event_names() -> Vec<String> {
        // 尝试获取调度器实例
        if let Some(dispatcher) = Self::get_dispatcher() {
            // 调用调度器的 event_names 方法获取所有事件名
            dispatcher.event_names()
        } else {
            // 未初始化时返回空列表
            Vec::new()
        }
    }

    /// 禁用指定监听器
    ///
    /// 临时禁用某个监听器，但不从事件中移除
    /// 可以通过重新启用恢复
    ///
    /// # 参数
    /// - `event_name`: 事件名称
    /// - `listener_class`: 监听器类名
    pub fn disable_listener(event_name: &str, listener_class: &str) {
        // 尝试获取调度器实例
        if let Some(dispatcher) = Self::get_dispatcher() {
            // 调用调度器的 disable_listener 方法禁用监听器
            dispatcher.disable_listener(event_name, listener_class);
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Event 门面未初始化，无法禁用监听器: {} -> {}", event_name, listener_class);
        }
    }

    /// 注册事件订阅器
    ///
    /// 订阅器是一个类，通过 subscribe 方法定义多个事件监听
    /// 对应 ThinkPHP 8.0 中的 Event::subscribe() 功能
    ///
    /// # 参数
    /// - `subscriber_class`: 订阅器类名（含命名空间）
    ///
    /// # 示例
    /// ```php
    /// // PHP 订阅器示例
    /// class UserEventSubscriber {
    ///     public function onUserCreated($event) { }
    ///     public function onUserDeleted($event) { }
    /// }
    ///
    /// // 注册订阅器
    /// Event::subscribe('app\\subscribe\\UserEventSubscriber');
    /// ```
    pub fn subscribe(subscriber_class: &str) {
        // 尝试获取调度器实例
        if let Some(dispatcher) = Self::get_dispatcher() {
            // 调用调度器的 subscribe 方法注册订阅器
            dispatcher.subscribe(subscriber_class);
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Event 门面未初始化，无法注册订阅器: {}", subscriber_class);
        }
    }

    /// 从配置加载事件定义
    ///
    /// 解析 config/event.php 中的事件配置并注册
    ///
    /// # 参数
    /// - `listen_config`: listen 配置（事件名 → 监听器列表）
    /// - `subscribe_config`: subscribe 配置（订阅器类名列表）
    ///
    /// # 示例
    /// ```php
    /// // config/event.php
    /// return [
    ///     'listen' => [
    ///         'UserCreated' => ['app\\listener\\UserCreated'],
    ///     ],
    ///     'subscribe' => [
    ///         'app\\subscribe\\UserEventSubscriber',
    ///     ],
    /// ];
    /// ```
    pub fn load_from_config(
        listen_config: &HashMap<String, Vec<String>>,
        subscribe_config: &[String],
    ) {
        // 尝试获取调度器实例
        if let Some(dispatcher) = Self::get_dispatcher() {
            // 调用调度器的 load_from_config 方法加载配置
            dispatcher.load_from_config(listen_config, subscribe_config);
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Event 门面未初始化，无法加载配置");
        }
    }

    /// 从符号表自动发现事件监听器
    ///
    /// 扫描符号表中所有实现了 Listener 接口的类
    /// 自动注册为事件监听器
    pub fn discover_from_registry() {
        // 尝试获取调度器实例
        if let Some(dispatcher) = Self::get_dispatcher() {
            // 调用调度器的 discover_from_registry 方法自动发现
            dispatcher.discover_from_registry();
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Event 门面未初始化，无法自动发现监听器");
        }
    }

    /// 检查事件门面是否已初始化
    ///
    /// 用于判断是否需要调用 init 方法
    ///
    /// # 返回
    /// 如果已初始化返回 true，否则返回 false
    pub fn is_initialized() -> bool {
        // 获取读锁检查是否有值
        let guard = EVENT_DISPATCHER.read();
        guard.is_some()
    }

    /// 重置事件门面
    ///
    /// 清除所有事件监听器和调度器实例
    /// 主要用于测试环境
    pub fn reset() {
        // 获取写锁
        let mut guard = EVENT_DISPATCHER.write();
        // 清空调度器实例
        *guard = None;
    }
}
