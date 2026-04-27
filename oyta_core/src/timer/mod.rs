//! 多级时间轮定时器模块
//!
//! 实现高性能的多级时间轮定时器（Hierarchical Timing Wheel）
//! 提供毫秒级精度的定时任务调度，支持从毫秒到天级别的超时范围
//!
//! # 功能特性
//! - 4级层级设计：毫秒轮、秒轮、分钟轮、小时轮
//! - O(1) 添加/删除任务复杂度
//! - 支持一次性任务和周期性任务
//! - 任务状态追踪和统计
//! - 与 PHP 解释器集成
//!
//! # 架构设计
//! ```text
//! 多级时间轮结构（4级层级设计）：
//!
//! Level 0: 毫秒轮 (Millisecond Wheel)
//! [0][1][2][3]...[63]  ← 64 槽位，每槽 1ms，周期 64ms
//!
//! Level 1: 秒轮 (Second Wheel)
//! [0][1][2]...[59]  ← 60 槽位，每槽 1s，周期 60s
//!
//! Level 2: 分钟轮 (Minute Wheel)
//! [0][1][2]...[59]  ← 60 槽位，每槽 1min，周期 60min
//!
//! Level 3: 小时轮 (Hour Wheel)
//! [0][1][2]...[23]  ← 24 槽位，每槽 1hour，周期 24h
//! ```
//!
//! # 使用示例
//! ```php
//! // 创建多级时间轮定时器
//! $timer = new Timer([
//!     'levels' => [
//!         ['tick_ms' => 1,   'slots' => 64],
//!         ['tick_ms' => 64,  'slots' => 60],
//!         ['tick_ms' => 3840, 'slots' => 60],
//!         ['tick_ms' => 230400, 'slots' => 24],
//!     ],
//! ]);
//!
//! // 添加一次性定时任务
//! $timerId = $timer->after(1000, function() {
//!     echo "1秒后执行";
//! });
//!
//! // 添加周期性任务
//! $timerId = $timer->every(2000, function() {
//!     echo "每2秒执行一次";
//! });
//! ```

// 引入子模块
pub mod wheel;
pub mod level;
pub mod slot;
pub mod task;
pub mod scheduler;
pub mod callback;
pub mod stats;
pub mod error;

// 重新导出主要类型
pub use wheel::HierarchicalTimingWheel;
pub use level::TimeWheel;
pub use slot::Slot;
pub use task::{TaskEntry, TaskType, TaskStatus, TimerCallback};
pub use scheduler::TimerScheduler;
pub use callback::CallbackHandler;
pub use stats::TimerStats;
pub use error::{TimerError, TimerResult};
