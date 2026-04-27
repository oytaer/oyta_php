//! 定时器任务定义模块
//!
//! 定义定时器任务的数据结构和相关类型
//! 支持一次性任务、周期性任务和有限次周期任务

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 任务类型
/// 定义定时器任务的不同类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    /// 一次性任务
    /// 在指定延迟后执行一次
    Once,
    
    /// 周期性任务
    /// 按指定间隔重复执行，直到被取消
    Periodic {
        /// 执行间隔（毫秒）
        interval_ms: u64,
    },
    
    /// 有限次周期任务
    /// 按指定间隔执行指定次数后自动停止
    Times {
        /// 执行间隔（毫秒）
        interval_ms: u64,
        /// 剩余执行次数
        remaining: u32,
    },
}

/// 任务状态
/// 定义任务在生命周期中的不同状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 等待执行
    /// 任务已创建，等待触发时间到达
    Pending,
    
    /// 正在执行
    /// 任务回调正在执行中
    Running,
    
    /// 已暂停
    /// 任务被暂停，不会被执行
    Paused,
    
    /// 已完成
    /// 任务已执行完毕（一次性任务或有限次任务）
    Completed,
    
    /// 已取消
    /// 任务被取消，不会再执行
    Cancelled,
}

/// 定时器回调类型
/// 定义任务触发时调用的回调
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimerCallback {
    /// PHP 闭包回调
    /// 存储闭包的 JSON 序列化表示
    Closure {
        /// 闭包的 JSON 序列化表示
        closure_json: String,
        /// 参数列表（JSON 序列化）
        args_json: Vec<String>,
    },
    
    /// PHP 函数名回调
    /// 调用指定的 PHP 函数
    Function {
        /// 函数名
        name: String,
        /// 参数列表（JSON 序列化）
        args_json: Vec<String>,
    },
    
    /// PHP 类方法回调
    /// 调用指定类的静态方法或实例方法
    Method {
        /// 类名
        class: String,
        /// 方法名
        method: String,
        /// 参数列表（JSON 序列化）
        args_json: Vec<String>,
        /// 是否为静态方法
        is_static: bool,
    },
    
    /// Rust 原生回调
    /// 用于内部任务，直接调用 Rust 函数
    Native {
        /// 回调标识符
        handler_id: String,
    },
}

/// 任务条目
/// 表示一个定时器任务的完整信息
#[derive(Debug)]
pub struct TaskEntry {
    /// 任务 ID
    /// 唯一标识符，由定时器生成
    pub id: u64,
    
    /// 剩余轮数
    /// 在同一槽位需要转多少圈后才执行
    /// 使用原子类型支持并发访问
    pub rounds: AtomicU32,
    
    /// 任务类型
    /// 决定任务的执行方式
    pub task_type: TaskType,
    
    /// 回调函数
    /// 任务触发时调用的回调
    pub callback: TimerCallback,
    
    /// 任务状态
    /// 使用原子类型支持并发访问
    pub status: AtomicTaskStatus,
    
    /// 创建时间
    /// 任务创建的时间点
    pub created_at: Instant,
    
    /// 预期执行时间
    /// 任务应该执行的时间点
    pub execute_at: u64,
    
    /// 实际执行次数
    /// 记录任务已执行的次数
    pub execution_count: AtomicU64,
    
    /// 最后执行时间
    /// 上次执行的时间点（毫秒时间戳）
    pub last_execution_time: AtomicU64,
    
    /// 任务名称
    /// 可选的任务名称，用于调试和日志
    pub name: Option<String>,
    
    /// 任务标签
    /// 用于分组和过滤
    pub tags: Vec<String>,
    
    /// 优先级
    /// 同一槽位中优先级高的任务先执行
    pub priority: u8,
    
    /// 最大执行时间（毫秒）
    /// 超过此时间视为超时
    pub max_execution_time: Option<u64>,
    
    /// 重试次数
    /// 执行失败时的最大重试次数
    pub max_retries: u32,
    
    /// 当前重试计数
    /// 已重试的次数
    pub retry_count: AtomicU32,
}

/// 原子任务状态
/// 使用原子类型支持并发访问的任务状态
#[derive(Debug)]
pub struct AtomicTaskStatus {
    /// 内部原子值
    inner: AtomicU8,
}

impl AtomicTaskStatus {
    /// 创建新的原子任务状态
    ///
    /// # 参数
    /// - `status`: 初始状态
    ///
    /// # 返回
    /// 返回新创建的原子任务状态实例
    pub fn new(status: TaskStatus) -> Self {
        Self {
            inner: AtomicU8::new(status as u8),
        }
    }
    
    /// 加载当前状态
    ///
    /// # 返回
    /// 返回当前任务状态
    pub fn load(&self) -> TaskStatus {
        // 将原子值转换为任务状态
        match self.inner.load(Ordering::SeqCst) {
            0 => TaskStatus::Pending,
            1 => TaskStatus::Running,
            2 => TaskStatus::Paused,
            3 => TaskStatus::Completed,
            4 => TaskStatus::Cancelled,
            _ => TaskStatus::Pending,
        }
    }
    
    /// 存储新状态
    ///
    /// # 参数
    /// - `status`: 要存储的新状态
    pub fn store(&self, status: TaskStatus) {
        // 将任务状态转换为原子值并存储
        self.inner.store(status as u8, Ordering::SeqCst);
    }
    
    /// 比较并交换状态
    /// 原子操作：如果当前状态等于 expected，则更新为 new
    ///
    /// # 参数
    /// - `expected`: 期望的当前状态
    /// - `new`: 要更新的新状态
    ///
    /// # 返回
    /// 如果成功返回 Ok(旧状态)，否则返回 Err(当前状态)
    pub fn compare_exchange(
        &self,
        expected: TaskStatus,
        new: TaskStatus,
    ) -> Result<TaskStatus, TaskStatus> {
        // 执行原子比较交换
        self.inner.compare_exchange(
            expected as u8,
            new as u8,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ).map(|v| Self::u8_to_status(v))
         .map_err(|v| Self::u8_to_status(v))
    }
    
    /// 将 u8 值转换为任务状态
    fn u8_to_status(value: u8) -> TaskStatus {
        match value {
            0 => TaskStatus::Pending,
            1 => TaskStatus::Running,
            2 => TaskStatus::Paused,
            3 => TaskStatus::Completed,
            4 => TaskStatus::Cancelled,
            _ => TaskStatus::Pending,
        }
    }
}

impl TaskEntry {
    /// 创建新的一次性任务
    ///
    /// # 参数
    /// - `id`: 任务 ID
    /// - `delay_ms`: 延迟时间（毫秒）
    /// - `callback`: 回调函数
    /// - `current_time`: 当前时间（毫秒）
    ///
    /// # 返回
    /// 返回新创建的任务条目
    pub fn once(
        id: u64,
        delay_ms: u64,
        callback: TimerCallback,
        current_time: u64,
    ) -> Self {
        Self {
            // 设置任务 ID
            id,
            // 初始轮数为 0
            rounds: AtomicU32::new(0),
            // 设置为一次性任务
            task_type: TaskType::Once,
            // 设置回调函数
            callback,
            // 初始状态为等待执行
            status: AtomicTaskStatus::new(TaskStatus::Pending),
            // 记录创建时间
            created_at: Instant::now(),
            // 计算预期执行时间
            execute_at: current_time + delay_ms,
            // 初始执行次数为 0
            execution_count: AtomicU64::new(0),
            // 初始最后执行时间为 0
            last_execution_time: AtomicU64::new(0),
            // 名称默认为空
            name: None,
            // 标签默认为空
            tags: Vec::new(),
            // 优先级默认为 0
            priority: 0,
            // 最大执行时间默认为空
            max_execution_time: None,
            // 最大重试次数默认为 0
            max_retries: 0,
            // 重试计数初始为 0
            retry_count: AtomicU32::new(0),
        }
    }
    
    /// 创建新的周期性任务
    ///
    /// # 参数
    /// - `id`: 任务 ID
    /// - `interval_ms`: 执行间隔（毫秒）
    /// - `callback`: 回调函数
    /// - `current_time`: 当前时间（毫秒）
    ///
    /// # 返回
    /// 返回新创建的任务条目
    pub fn periodic(
        id: u64,
        interval_ms: u64,
        callback: TimerCallback,
        current_time: u64,
    ) -> Self {
        Self {
            // 设置任务 ID
            id,
            // 初始轮数为 0
            rounds: AtomicU32::new(0),
            // 设置为周期性任务
            task_type: TaskType::Periodic { interval_ms },
            // 设置回调函数
            callback,
            // 初始状态为等待执行
            status: AtomicTaskStatus::new(TaskStatus::Pending),
            // 记录创建时间
            created_at: Instant::now(),
            // 计算预期执行时间
            execute_at: current_time + interval_ms,
            // 初始执行次数为 0
            execution_count: AtomicU64::new(0),
            // 初始最后执行时间为 0
            last_execution_time: AtomicU64::new(0),
            // 名称默认为空
            name: None,
            // 标签默认为空
            tags: Vec::new(),
            // 优先级默认为 0
            priority: 0,
            // 最大执行时间默认为空
            max_execution_time: None,
            // 最大重试次数默认为 0
            max_retries: 0,
            // 重试计数初始为 0
            retry_count: AtomicU32::new(0),
        }
    }
    
    /// 创建新的有限次周期任务
    ///
    /// # 参数
    /// - `id`: 任务 ID
    /// - `interval_ms`: 执行间隔（毫秒）
    /// - `times`: 执行次数
    /// - `callback`: 回调函数
    /// - `current_time`: 当前时间（毫秒）
    ///
    /// # 返回
    /// 返回新创建的任务条目
    pub fn times(
        id: u64,
        interval_ms: u64,
        times: u32,
        callback: TimerCallback,
        current_time: u64,
    ) -> Self {
        Self {
            // 设置任务 ID
            id,
            // 初始轮数为 0
            rounds: AtomicU32::new(0),
            // 设置为有限次周期任务
            task_type: TaskType::Times {
                interval_ms,
                remaining: times,
            },
            // 设置回调函数
            callback,
            // 初始状态为等待执行
            status: AtomicTaskStatus::new(TaskStatus::Pending),
            // 记录创建时间
            created_at: Instant::now(),
            // 计算预期执行时间
            execute_at: current_time + interval_ms,
            // 初始执行次数为 0
            execution_count: AtomicU64::new(0),
            // 初始最后执行时间为 0
            last_execution_time: AtomicU64::new(0),
            // 名称默认为空
            name: None,
            // 标签默认为空
            tags: Vec::new(),
            // 优先级默认为 0
            priority: 0,
            // 最大执行时间默认为空
            max_execution_time: None,
            // 最大重试次数默认为 0
            max_retries: 0,
            // 重试计数初始为 0
            retry_count: AtomicU32::new(0),
        }
    }
    
    /// 设置轮数
    ///
    /// # 参数
    /// - `rounds`: 新的轮数值
    pub fn set_rounds(&self, rounds: u32) {
        self.rounds.store(rounds, Ordering::SeqCst);
    }
    
    /// 获取轮数
    ///
    /// # 返回
    /// 返回当前轮数
    pub fn get_rounds(&self) -> u32 {
        self.rounds.load(Ordering::SeqCst)
    }
    
    /// 递减轮数
    ///
    /// # 返回
    /// 返回递减后的轮数
    pub fn decrement_rounds(&self) -> u32 {
        self.rounds.fetch_sub(1, Ordering::SeqCst)
    }
    
    /// 获取任务状态
    ///
    /// # 返回
    /// 返回当前任务状态
    pub fn get_status(&self) -> TaskStatus {
        self.status.load()
    }
    
    /// 设置任务状态
    ///
    /// # 参数
    /// - `status`: 新的任务状态
    pub fn set_status(&self, status: TaskStatus) {
        self.status.store(status);
    }
    
    /// 检查任务是否可执行
    ///
    /// # 返回
    /// 如果任务可以执行返回 true，否则返回 false
    pub fn is_executable(&self) -> bool {
        // 获取当前状态
        let status = self.get_status();
        
        // 只有等待执行状态的任务可以执行
        status == TaskStatus::Pending
    }
    
    /// 检查任务是否已取消
    ///
    /// # 返回
    /// 如果任务已取消返回 true，否则返回 false
    pub fn is_cancelled(&self) -> bool {
        self.get_status() == TaskStatus::Cancelled
    }
    
    /// 检查任务是否已完成
    ///
    /// # 返回
    /// 如果任务已完成返回 true，否则返回 false
    pub fn is_completed(&self) -> bool {
        self.get_status() == TaskStatus::Completed
    }
    
    /// 记录执行
    /// 递增执行次数并更新最后执行时间
    ///
    /// # 参数
    /// - `execution_time`: 执行时间（毫秒时间戳）
    pub fn record_execution(&self, execution_time: u64) {
        // 递增执行次数
        self.execution_count.fetch_add(1, Ordering::SeqCst);
        // 更新最后执行时间
        self.last_execution_time.store(execution_time, Ordering::SeqCst);
    }
    
    /// 获取执行次数
    ///
    /// # 返回
    /// 返回已执行的次数
    pub fn get_execution_count(&self) -> u64 {
        self.execution_count.load(Ordering::SeqCst)
    }
    
    /// 获取最后执行时间
    ///
    /// # 返回
    /// 返回最后执行时间（毫秒时间戳）
    pub fn get_last_execution_time(&self) -> u64 {
        self.last_execution_time.load(Ordering::SeqCst)
    }
    
    /// 检查是否需要重试
    ///
    /// # 返回
    /// 如果可以重试返回 true，否则返回 false
    pub fn can_retry(&self) -> bool {
        // 获取当前重试次数
        let current = self.retry_count.load(Ordering::SeqCst);
        // 检查是否还有重试机会
        current < self.max_retries
    }
    
    /// 记录重试
    /// 递增重试计数
    pub fn record_retry(&self) {
        self.retry_count.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 获取任务间隔
    /// 对于周期性任务，返回执行间隔
    ///
    /// # 返回
    /// 返回执行间隔（毫秒），如果是一次性任务返回 None
    pub fn get_interval(&self) -> Option<u64> {
        match self.task_type {
            TaskType::Once => None,
            TaskType::Periodic { interval_ms } => Some(interval_ms),
            TaskType::Times { interval_ms, .. } => Some(interval_ms),
        }
    }
    
    /// 设置任务名称
    ///
    /// # 参数
    /// - `name`: 任务名称
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }
    
    /// 添加标签
    ///
    /// # 参数
    /// - `tag`: 标签名称
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    
    /// 设置优先级
    ///
    /// # 参数
    /// - `priority`: 优先级（0-255，数值越大优先级越高）
    pub fn set_priority(&mut self, priority: u8) {
        self.priority = priority;
    }
    
    /// 设置最大执行时间
    ///
    /// # 参数
    /// - `max_time`: 最大执行时间（毫秒）
    pub fn set_max_execution_time(&mut self, max_time: u64) {
        self.max_execution_time = Some(max_time);
    }
    
    /// 设置最大重试次数
    ///
    /// # 参数
    /// - `retries`: 最大重试次数
    pub fn set_max_retries(&mut self, retries: u32) {
        self.max_retries = retries;
    }
}

/// 任务信息
/// 用于返回任务状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    /// 任务 ID
    pub id: u64,
    /// 任务状态
    pub status: TaskStatus,
    /// 任务类型
    pub task_type: TaskType,
    /// 剩余时间（毫秒）
    pub remaining_ms: u64,
    /// 所在层级
    pub level: u8,
    /// 所在槽位
    pub slot: usize,
    /// 执行次数
    pub execution_count: u64,
    /// 创建时间
    pub created_at: u64,
    /// 预期执行时间
    pub execute_at: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试一次性任务创建
    #[test]
    fn test_once_task_creation() {
        // 创建一次性任务
        let task = TaskEntry::once(
            1,
            1000,
            TimerCallback::Function {
                name: "test".to_string(),
                args_json: vec![],
            },
            0,
        );
        
        // 验证
        assert_eq!(task.id, 1);
        assert_eq!(task.task_type, TaskType::Once);
        assert_eq!(task.get_status(), TaskStatus::Pending);
        assert!(task.get_interval().is_none());
    }
    
    /// 测试周期性任务创建
    #[test]
    fn test_periodic_task_creation() {
        // 创建周期性任务
        let task = TaskEntry::periodic(
            1,
            2000,
            TimerCallback::Function {
                name: "test".to_string(),
                args_json: vec![],
            },
            0,
        );
        
        // 验证
        assert_eq!(task.task_type, TaskType::Periodic { interval_ms: 2000 });
        assert_eq!(task.get_interval(), Some(2000));
    }
    
    /// 测试原子状态操作
    #[test]
    fn test_atomic_status() {
        // 创建原子状态
        let status = AtomicTaskStatus::new(TaskStatus::Pending);
        
        // 验证初始状态
        assert_eq!(status.load(), TaskStatus::Pending);
        
        // 更新状态
        status.store(TaskStatus::Running);
        assert_eq!(status.load(), TaskStatus::Running);
        
        // 比较交换
        let result = status.compare_exchange(TaskStatus::Running, TaskStatus::Completed);
        assert!(result.is_ok());
        assert_eq!(status.load(), TaskStatus::Completed);
    }
    
    /// 测试任务执行记录
    #[test]
    fn test_execution_record() {
        // 创建任务
        let task = TaskEntry::once(1, 1000, TimerCallback::Function {
            name: "test".to_string(),
            args_json: vec![],
        }, 0);
        
        // 记录执行
        task.record_execution(1000);
        
        // 验证
        assert_eq!(task.get_execution_count(), 1);
        assert_eq!(task.get_last_execution_time(), 1000);
    }
}
