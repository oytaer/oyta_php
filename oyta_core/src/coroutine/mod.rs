//! 原生协程调度器模块
//!
//! 提供基于 Rust async/await 的原生协程支持
//! 相比 PHP 原生协程性能提升 5x+，支持工作窃取调度
//!
//! # 内部实现说明
//! - scheduler: 协程调度器（通过 Coroutine 门面暴露）
//! - task: 任务管理（内部实现）
//! - channel: 协程通道（内部实现）
//! - sync: 同步原语（内部实现）

// 允许内部实现未使用警告
#![allow(dead_code)]

// 引入子模块
pub mod scheduler;
pub mod task;
pub mod channel;
pub mod sync;

// 重导出主要类型
pub use scheduler::{CoroutineScheduler, SchedulerConfig, SchedulerStats};
pub use task::{CoroutineTask, TaskId, TaskState, TaskPriority};
pub use channel::{CoroutineChannel, ChannelError};
pub use sync::{CoroutineMutex, CoroutineRwLock, CoroutineSemaphore};
