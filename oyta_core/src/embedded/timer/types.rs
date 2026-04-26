//! 定时器类型定义模块
//!
//! 包含定时任务相关的数据结构定义

use std::collections::HashMap;

/// 定时任务定义
#[derive(Debug, Clone)]
pub struct TimerTask {
    /// 任务ID
    pub id: String,
    /// 任务名称
    pub name: String,
    /// 任务类型
    pub task_type: TaskType,
    /// 任务处理器
    pub handler: TaskHandler,
    /// 任务状态
    pub status: TaskStatus,
    /// 最后执行时间
    pub last_run: Option<String>,
    /// 下次执行时间
    pub next_run: Option<String>,
    /// 执行次数
    pub run_count: u64,
    /// 最大执行次数（0 表示无限制）
    pub max_runs: u64,
    /// 是否启用
    pub enabled: bool,
    /// 任务元数据
    pub metadata: HashMap<String, String>,
}

/// 任务类型
#[derive(Debug, Clone)]
pub enum TaskType {
    /// 固定间隔执行
    Interval {
        /// 执行间隔（秒）
        secs: u64,
    },
    /// Cron 表达式调度
    Cron {
        /// Cron 表达式
        expression: String,
    },
    /// 一次性任务
    Once {
        /// 延迟时间（秒）
        delay_secs: u64,
    },
    /// 延迟任务
    Delayed {
        /// 延迟时间（秒）
        delay_secs: u64,
        /// 之后转为间隔任务
        interval_secs: u64,
    },
}

/// 任务处理器
#[derive(Debug, Clone)]
pub enum TaskHandler {
    /// PHP 类方法
    PhpClass {
        /// 类名
        class: String,
        /// 方法名
        method: String,
        /// 参数
        args: Vec<String>,
    },
    /// PHP 闭包（序列化）
    PhpClosure {
        /// 闭包代码
        code: String,
    },
    /// 原生 Rust 函数
    Native {
        /// 函数名
        name: String,
    },
    /// Shell 命令
    Shell {
        /// 命令
        command: String,
    },
}

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    /// 待执行
    Pending,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 已暂停
    Paused,
    /// 已失败
    Failed,
}

/// 任务执行结果
#[derive(Debug, Clone)]
pub struct TaskResult {
    /// 任务ID
    pub task_id: String,
    /// 是否成功
    pub success: bool,
    /// 执行时间（毫秒）
    pub duration_ms: u64,
    /// 输出信息
    pub output: String,
    /// 错误信息
    pub error: Option<String>,
}
