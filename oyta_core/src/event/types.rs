//! 事件类型定义
//!
//! 定义事件和监听器的数据结构
//! 对应 ThinkPHP 8.0 的事件系统

use serde::{Deserialize, Serialize};

/// 事件定义
///
/// 描述一个事件及其关联的监听器列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDef {
    /// 事件名称
    /// 例如：AppInit、HttpRun、UserLogin
    pub name: String,
    /// 监听器列表
    /// 按优先级排序，优先级数字越小越先执行
    pub listeners: Vec<ListenerDef>,
    /// 是否只触发一次
    /// true: 触发后自动移除所有监听器
    pub once: bool,
    /// 是否已触发过（仅 once=true 时使用）
    pub triggered: bool,
}

/// 监听器定义
///
/// 描述一个事件监听器
/// 监听器类必须实现 handle 方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerDef {
    /// 监听器类名（含命名空间）
    /// 例如：app\listener\UserLoginListener
    pub class: String,
    /// 优先级（数字越小越先执行）
    pub priority: i32,
    /// 是否禁用
    pub disabled: bool,
}

/// 事件触发结果
///
/// 记录事件触发后各监听器的执行情况
#[derive(Debug, Clone)]
pub struct EventResult {
    /// 事件名称
    pub event_name: String,
    /// 成功执行的监听器数量
    pub success_count: usize,
    /// 失败的监听器数量
    pub failure_count: usize,
    /// 是否被某个监听器中断
    pub stopped: bool,
    /// 错误信息列表
    pub errors: Vec<String>,
}

impl EventResult {
    /// 创建空的事件结果
    pub fn new(event_name: &str) -> Self {
        Self {
            event_name: event_name.to_string(),
            success_count: 0,
            failure_count: 0,
            stopped: false,
            errors: Vec::new(),
        }
    }
}
