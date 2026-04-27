//! 协程任务模块
//!
//! 定义协程任务的数据结构和状态管理

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// 任务 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub u64);

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Task({})", self.0)
    }
}

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    /// 待执行
    Pending,
    /// 执行中
    Running,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 失败
    Failed,
    /// 等待中（等待其他任务或资源）
    Waiting,
}

impl TaskState {
    /// 检查是否是终态
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Failed)
    }
    
    /// 检查是否可以执行
    pub fn is_runnable(&self) -> bool {
        matches!(self, Self::Pending | Self::Waiting)
    }
}

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    /// 低优先级
    Low = 0,
    /// 普通优先级
    Normal = 1,
    /// 高优先级
    High = 2,
    /// 实时优先级
    Realtime = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    /// 普通任务
    Normal,
    /// 定时任务
    Scheduled {
        /// 执行时间（毫秒时间戳）
        execute_at: u64,
    },
    /// 周期任务
    Periodic {
        /// 间隔（毫秒）
        interval_ms: u64,
        /// 下次执行时间
        next_execute_at: u64,
    },
    /// 依赖任务
    Dependent {
        /// 依赖的任务 ID 列表
        depends_on: Vec<TaskId>,
    },
}

/// 协程任务
/// 表示一个可执行的协程任务
#[derive(Debug, Clone)]
pub struct CoroutineTask {
    /// 任务 ID
    id: TaskId,
    /// 任务名称
    name: String,
    /// 任务状态
    state: TaskState,
    /// 任务优先级
    priority: TaskPriority,
    /// 任务类型
    task_type: TaskType,
    /// 创建时间
    created_at: Instant,
    /// 开始执行时间
    started_at: Option<Instant>,
    /// 完成时间
    completed_at: Option<Instant>,
    /// 执行结果（JSON 格式）
    result: Option<String>,
    /// 错误信息
    error: Option<String>,
    /// 重试次数
    retry_count: u32,
    /// 最大重试次数
    max_retries: u32,
    /// 超时时间（毫秒）
    timeout_ms: u64,
    /// 元数据
    metadata: HashMap<String, String>,
}

impl CoroutineTask {
    /// 创建新的协程任务
    /// 
    /// # 参数
    /// - `name`: 任务名称
    pub fn new(name: &str) -> Self {
        Self {
            id: TaskId(0), // 将由调度器分配
            name: name.to_string(),
            state: TaskState::Pending,
            priority: TaskPriority::Normal,
            task_type: TaskType::Normal,
            created_at: Instant::now(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
            retry_count: 0,
            max_retries: 3,
            timeout_ms: 30000,
            metadata: HashMap::new(),
        }
    }
    
    /// 设置任务 ID
    pub fn set_id(&mut self, id: TaskId) {
        self.id = id;
    }
    
    /// 获取任务 ID
    pub fn id(&self) -> TaskId {
        self.id
    }
    
    /// 获取任务名称
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// 设置任务状态
    pub fn set_state(&mut self, state: TaskState) {
        // 更新时间戳
        match state {
            TaskState::Running => {
                self.started_at = Some(Instant::now());
            }
            TaskState::Completed | TaskState::Failed | TaskState::Cancelled => {
                self.completed_at = Some(Instant::now());
            }
            _ => {}
        }
        
        self.state = state;
    }
    
    /// 获取任务状态
    pub fn state(&self) -> TaskState {
        self.state
    }
    
    /// 设置任务优先级
    pub fn set_priority(&mut self, priority: TaskPriority) {
        self.priority = priority;
    }
    
    /// 获取任务优先级
    pub fn priority(&self) -> TaskPriority {
        self.priority
    }
    
    /// 设置任务类型
    pub fn set_task_type(&mut self, task_type: TaskType) {
        self.task_type = task_type;
    }
    
    /// 获取任务类型
    pub fn task_type(&self) -> &TaskType {
        &self.task_type
    }
    
    /// 设置执行结果
    pub fn set_result(&mut self, result: String) {
        self.result = Some(result);
    }
    
    /// 获取执行结果
    pub fn result(&self) -> Option<&String> {
        self.result.as_ref()
    }
    
    /// 设置错误信息
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }
    
    /// 获取错误信息
    pub fn error(&self) -> Option<&String> {
        self.error.as_ref()
    }
    
    /// 设置最大重试次数
    pub fn set_max_retries(&mut self, max_retries: u32) {
        self.max_retries = max_retries;
    }
    
    /// 获取最大重试次数
    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }
    
    /// 增加重试计数
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
    
    /// 获取重试次数
    pub fn retry_count(&self) -> u32 {
        self.retry_count
    }
    
    /// 检查是否可以重试
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }
    
    /// 设置超时时间
    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.timeout_ms = timeout_ms;
    }
    
    /// 获取超时时间
    pub fn timeout(&self) -> u64 {
        self.timeout_ms
    }
    
    /// 检查是否超时
    pub fn is_timeout(&self) -> bool {
        if let Some(started) = self.started_at {
            let elapsed = started.elapsed().as_millis() as u64;
            return elapsed > self.timeout_ms;
        }
        false
    }
    
    /// 设置元数据
    pub fn set_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// 获取元数据
    pub fn metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// 获取执行时长
    pub fn execution_duration(&self) -> Option<Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            (Some(start), None) => Some(start.elapsed()),
            _ => None,
        }
    }
    
    /// 获取等待时长
    pub fn wait_duration(&self) -> Duration {
        match self.started_at {
            Some(start) => start.duration_since(self.created_at),
            None => self.created_at.elapsed(),
        }
    }
    
    /// 重置任务状态（用于重试）
    pub fn reset(&mut self) {
        self.state = TaskState::Pending;
        self.started_at = None;
        self.completed_at = None;
        self.result = None;
        self.error = None;
    }
}

/// 任务构建器
/// 用于方便地创建协程任务
pub struct TaskBuilder {
    task: CoroutineTask,
}

impl TaskBuilder {
    /// 创建新的构建器
    pub fn new(name: &str) -> Self {
        Self {
            task: CoroutineTask::new(name),
        }
    }
    
    /// 设置优先级
    pub fn priority(mut self, priority: TaskPriority) -> Self {
        self.task.priority = priority;
        self
    }
    
    /// 设置为高优先级
    pub fn high_priority(mut self) -> Self {
        self.task.priority = TaskPriority::High;
        self
    }
    
    /// 设置为低优先级
    pub fn low_priority(mut self) -> Self {
        self.task.priority = TaskPriority::Low;
        self
    }
    
    /// 设置超时时间
    pub fn timeout(mut self, timeout_ms: u64) -> Self {
        self.task.timeout_ms = timeout_ms;
        self
    }
    
    /// 设置最大重试次数
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.task.max_retries = max_retries;
        self
    }
    
    /// 设置为定时任务
    pub fn scheduled(mut self, execute_at: u64) -> Self {
        self.task.task_type = TaskType::Scheduled { execute_at };
        self
    }
    
    /// 设置为周期任务
    pub fn periodic(mut self, interval_ms: u64) -> Self {
        self.task.task_type = TaskType::Periodic {
            interval_ms,
            next_execute_at: 0,
        };
        self
    }
    
    /// 设置依赖任务
    pub fn depends_on(mut self, task_ids: Vec<TaskId>) -> Self {
        self.task.task_type = TaskType::Dependent { depends_on: task_ids };
        self
    }
    
    /// 设置元数据
    pub fn metadata(mut self, key: &str, value: &str) -> Self {
        self.task.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// 构建任务
    pub fn build(self) -> CoroutineTask {
        self.task
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试任务创建
    #[test]
    fn test_task_creation() {
        let task = CoroutineTask::new("test_task");
        
        assert_eq!(task.name(), "test_task");
        assert_eq!(task.state(), TaskState::Pending);
        assert_eq!(task.priority(), TaskPriority::Normal);
    }
    
    /// 测试任务状态转换
    #[test]
    fn test_state_transition() {
        let mut task = CoroutineTask::new("test");
        
        task.set_state(TaskState::Running);
        assert_eq!(task.state(), TaskState::Running);
        assert!(task.started_at.is_some());
        
        task.set_state(TaskState::Completed);
        assert_eq!(task.state(), TaskState::Completed);
        assert!(task.completed_at.is_some());
    }
    
    /// 测试任务构建器
    #[test]
    fn test_task_builder() {
        let task = TaskBuilder::new("test")
            .high_priority()
            .timeout(5000)
            .max_retries(5)
            .build();
        
        assert_eq!(task.priority(), TaskPriority::High);
        assert_eq!(task.timeout(), 5000);
        assert_eq!(task.max_retries(), 5);
    }
    
    /// 测试重试机制
    #[test]
    fn test_retry() {
        let mut task = CoroutineTask::new("test");
        task.set_max_retries(3);
        
        assert!(task.can_retry());
        
        task.increment_retry();
        task.increment_retry();
        task.increment_retry();
        
        assert!(!task.can_retry());
        assert_eq!(task.retry_count(), 3);
    }
    
    /// 测试超时检查
    #[test]
    fn test_timeout() {
        let mut task = CoroutineTask::new("test");
        task.set_timeout(100);
        
        // 初始不超时
        assert!(!task.is_timeout());
        
        // 设置开始时间
        task.set_state(TaskState::Running);
        
        // 等待超时
        std::thread::sleep(Duration::from_millis(150));
        
        assert!(task.is_timeout());
    }
}
