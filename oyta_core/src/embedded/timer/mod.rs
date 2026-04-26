//! 定时器内嵌服务
//!
//! 提供定时任务调度功能
//! 支持：固定间隔、Cron 表达式、一次性任务、延迟任务
//!
//! # 功能特性
//! - 固定间隔执行
//! - Cron 表达式调度
//! - 一次性任务
//! - 延迟执行
//! - 任务状态追踪
//! - 与 PHP 解释器集成
//!
//! # 使用示例
//! ```php
//! // 在 PHP 中定义定时任务
//! Schedule::every(60, 'ProcessQueueTask');
//! Schedule::cron('0 * * * *', 'HourlyReportTask');
//! Schedule::once(300, 'DelayedNotificationTask');
//! ```
//!
//! # 模块结构
//! - `types`: 类型定义
//! - `bridge`: PHP 解释器桥接器
//! - `manager`: 定时器管理器
//! - `executor`: 任务执行器

pub mod bridge;
pub mod cron;
pub mod executor;
pub mod manager;
pub mod types;

// 重新导出主要类型
pub use bridge::PhpInterpreterBridge;
pub use executor::{execute_handler, update_task_status};
pub use manager::TimerManager;
pub use types::{TaskHandler, TaskResult, TaskStatus, TaskType, TimerTask};
