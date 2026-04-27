//! 协程调度器核心模块
//!
//! 实现工作窃取调度算法，高效管理协程任务

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::task::{CoroutineTask, TaskId, TaskState, TaskPriority};

/// 调度器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// 工作线程数量
    pub worker_threads: usize,
    /// 最大任务队列大小
    pub max_queue_size: usize,
    /// 任务执行超时（毫秒）
    pub task_timeout_ms: u64,
    /// 是否启用工作窃取
    pub enable_work_stealing: bool,
    /// 空闲线程休眠阈值（毫秒）
    pub idle_sleep_ms: u64,
    /// 是否启用任务优先级
    pub enable_priority: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            max_queue_size: 10000,
            task_timeout_ms: 30000,
            enable_work_stealing: true,
            idle_sleep_ms: 10,
            enable_priority: true,
        }
    }
}

impl SchedulerConfig {
    /// 创建新的配置
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 设置工作线程数量
    pub fn worker_threads(mut self, n: usize) -> Self {
        self.worker_threads = n.max(1);
        self
    }
    
    /// 设置最大队列大小
    pub fn max_queue_size(mut self, size: usize) -> Self {
        self.max_queue_size = size;
        self
    }
    
    /// 启用工作窃取
    pub fn enable_work_stealing(mut self) -> Self {
        self.enable_work_stealing = true;
        self
    }
    
    /// 禁用工作窃取
    pub fn disable_work_stealing(mut self) -> Self {
        self.enable_work_stealing = false;
        self
    }
}

/// 调度器统计信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchedulerStats {
    /// 总任务数
    pub total_tasks: u64,
    /// 已完成任务数
    pub completed_tasks: u64,
    /// 失败任务数
    pub failed_tasks: u64,
    /// 当前活跃任务数
    pub active_tasks: u64,
    /// 当前排队任务数
    pub queued_tasks: u64,
    /// 工作窃取次数
    pub work_steals: u64,
    /// 平均任务执行时间（微秒）
    pub avg_task_time_us: u64,
    /// 最大任务执行时间（微秒）
    pub max_task_time_us: u64,
    /// 总执行时间（毫秒）
    pub total_runtime_ms: u64,
}

impl SchedulerStats {
    /// 获取吞吐量（任务/秒）
    pub fn throughput(&self) -> f64 {
        if self.total_runtime_ms == 0 {
            return 0.0;
        }
        self.completed_tasks as f64 / (self.total_runtime_ms as f64 / 1000.0)
    }
    
    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        let total = self.completed_tasks + self.failed_tasks;
        if total == 0 {
            return 0.0;
        }
        self.completed_tasks as f64 / total as f64
    }
}

/// 工作线程状态
#[derive(Debug, Clone)]
struct WorkerState {
    /// 线程 ID
    id: usize,
    /// 本地任务队列
    local_queue: VecDeque<TaskId>,
    /// 是否正在工作
    is_working: bool,
    /// 已处理任务数
    tasks_processed: u64,
}

/// 协程调度器
/// 管理协程任务的调度和执行
#[derive(Debug)]
pub struct CoroutineScheduler {
    /// 配置
    config: SchedulerConfig,
    /// 全局任务队列
    global_queue: RwLock<VecDeque<TaskId>>,
    /// 任务存储
    tasks: RwLock<HashMap<TaskId, CoroutineTask>>,
    /// 工作线程状态
    workers: RwLock<Vec<WorkerState>>,
    /// 统计信息
    stats: RwLock<SchedulerStats>,
    /// 是否正在运行
    running: AtomicBool,
    /// 下一个任务 ID
    next_task_id: AtomicU64,
    /// 启动时间
    start_time: RwLock<Option<Instant>>,
}

impl CoroutineScheduler {
    /// 创建新的协程调度器
    pub fn new() -> Self {
        Self::with_config(SchedulerConfig::default())
    }
    
    /// 使用配置创建调度器
    pub fn with_config(config: SchedulerConfig) -> Self {
        // 创建工作线程状态
        let workers: Vec<WorkerState> = (0..config.worker_threads)
            .map(|id| WorkerState {
                id,
                local_queue: VecDeque::new(),
                is_working: false,
                tasks_processed: 0,
            })
            .collect();
        
        Self {
            config,
            global_queue: RwLock::new(VecDeque::new()),
            tasks: RwLock::new(HashMap::new()),
            workers: RwLock::new(workers),
            stats: RwLock::new(SchedulerStats::default()),
            running: AtomicBool::new(false),
            next_task_id: AtomicU64::new(1),
            start_time: RwLock::new(None),
        }
    }
    
    /// 生成新的任务 ID
    fn generate_task_id(&self) -> TaskId {
        TaskId(self.next_task_id.fetch_add(1, Ordering::SeqCst))
    }
    
    /// 启动调度器
    pub fn start(&self) {
        // 设置运行状态
        self.running.store(true, Ordering::SeqCst);
        
        // 记录启动时间
        if let Ok(mut start_time) = self.start_time.write() {
            *start_time = Some(Instant::now());
        }
        
        tracing::info!(
            "协程调度器已启动，工作线程数: {}",
            self.config.worker_threads
        );
    }
    
    /// 停止调度器
    pub fn stop(&self) {
        // 清除运行状态
        self.running.store(false, Ordering::SeqCst);
        
        // 更新统计
        if let Ok(mut stats) = self.stats.write() {
            if let Ok(start_time) = self.start_time.read() {
                if let Some(start) = *start_time {
                    stats.total_runtime_ms = start.elapsed().as_millis() as u64;
                }
            }
        }
        
        tracing::info!("协程调度器已停止");
    }
    
    /// 提交任务
    /// 
    /// # 参数
    /// - `task`: 协程任务
    pub fn submit(&self, mut task: CoroutineTask) -> TaskId {
        // 生成任务 ID
        let task_id = self.generate_task_id();
        task.set_id(task_id);
        
        // 存储任务
        if let Ok(mut tasks) = self.tasks.write() {
            tasks.insert(task_id, task);
        }
        
        // 加入队列
        if let Ok(mut queue) = self.global_queue.write() {
            // 检查队列大小
            if queue.len() < self.config.max_queue_size {
                queue.push_back(task_id);
            }
        }
        
        // 更新统计
        if let Ok(mut stats) = self.stats.write() {
            stats.total_tasks += 1;
            stats.queued_tasks += 1;
        }
        
        task_id
    }
    
    /// 提交高优先级任务
    pub fn submit_high_priority(&self, task: CoroutineTask) -> TaskId {
        let mut task = task;
        task.set_priority(TaskPriority::High);
        self.submit(task)
    }
    
    /// 取消任务
    /// 
    /// # 参数
    /// - `task_id`: 任务 ID
    pub fn cancel(&self, task_id: TaskId) -> bool {
        // 从任务存储中移除
        if let Ok(mut tasks) = self.tasks.write() {
            if let Some(mut task) = tasks.remove(&task_id) {
                task.set_state(TaskState::Cancelled);
                return true;
            }
        }
        false
    }
    
    /// 获取任务状态
    /// 
    /// # 参数
    /// - `task_id`: 任务 ID
    pub fn get_task_state(&self, task_id: TaskId) -> Option<TaskState> {
        if let Ok(tasks) = self.tasks.read() {
            tasks.get(&task_id).map(|t| t.state())
        } else {
            None
        }
    }
    
    /// 获取下一个要执行的任务
    fn get_next_task(&self, worker_id: usize) -> Option<TaskId> {
        // 首先尝试从本地队列获取
        if let Ok(mut workers) = self.workers.write() {
            if worker_id < workers.len() {
                if let Some(task_id) = workers[worker_id].local_queue.pop_front() {
                    return Some(task_id);
                }
            }
        }
        
        // 然后从全局队列获取
        if let Ok(mut queue) = self.global_queue.write() {
            if let Some(task_id) = queue.pop_front() {
                // 更新统计
                if let Ok(mut stats) = self.stats.write() {
                    stats.queued_tasks = stats.queued_tasks.saturating_sub(1);
                }
                return Some(task_id);
            }
        }
        
        // 最后尝试工作窃取
        if self.config.enable_work_stealing {
            return self.steal_task(worker_id);
        }
        
        None
    }
    
    /// 工作窃取
    fn steal_task(&self, thief_id: usize) -> Option<TaskId> {
        if let Ok(mut workers) = self.workers.write() {
            // 遍历其他工作线程
            for (id, worker) in workers.iter_mut().enumerate() {
                if id != thief_id && worker.local_queue.len() > 1 {
                    // 窃取一半的任务
                    let steal_count = worker.local_queue.len() / 2;
                    for _ in 0..steal_count {
                        if let Some(task_id) = worker.local_queue.pop_front() {
                            // 更新统计
                            if let Ok(mut stats) = self.stats.write() {
                                stats.work_steals += 1;
                            }
                            return Some(task_id);
                        }
                    }
                }
            }
        }
        None
    }
    
    /// 执行一次调度循环
    pub fn tick(&self) -> usize {
        let mut executed = 0;
        
        // 检查是否正在运行
        if !self.running.load(Ordering::SeqCst) {
            return 0;
        }
        
        // 遍历所有工作线程
        let worker_count = if let Ok(workers) = self.workers.read() {
            workers.len()
        } else {
            0
        };
        
        for worker_id in 0..worker_count {
            // 获取下一个任务
            if let Some(task_id) = self.get_next_task(worker_id) {
                // 执行任务
                if self.execute_task(task_id, worker_id) {
                    executed += 1;
                }
            }
        }
        
        executed
    }
    
    /// 执行单个任务
    fn execute_task(&self, task_id: TaskId, worker_id: usize) -> bool {
        let start_time = Instant::now();
        
        // 获取任务
        let task = if let Ok(mut tasks) = self.tasks.write() {
            tasks.remove(&task_id)
        } else {
            return false;
        };
        
        let Some(mut task) = task else {
            return false;
        };
        
        // 更新工作线程状态
        if let Ok(mut workers) = self.workers.write() {
            if worker_id < workers.len() {
                workers[worker_id].is_working = true;
            }
        }
        
        // 更新任务状态
        task.set_state(TaskState::Running);
        
        // 更新统计
        if let Ok(mut stats) = self.stats.write() {
            stats.active_tasks += 1;
        }
        
        // 执行任务（模拟）
        let result = self.run_task(&mut task);
        
        // 计算执行时间
        let execution_time_us = start_time.elapsed().as_micros() as u64;
        
        // 更新统计
        if let Ok(mut stats) = self.stats.write() {
            stats.active_tasks = stats.active_tasks.saturating_sub(1);
            stats.completed_tasks += 1;
            stats.avg_task_time_us = (stats.avg_task_time_us + execution_time_us) / 2;
            if execution_time_us > stats.max_task_time_us {
                stats.max_task_time_us = execution_time_us;
            }
        }
        
        // 更新工作线程状态
        if let Ok(mut workers) = self.workers.write() {
            if worker_id < workers.len() {
                workers[worker_id].is_working = false;
                workers[worker_id].tasks_processed += 1;
            }
        }
        
        result
    }
    
    /// 运行任务
    fn run_task(&self, task: &mut CoroutineTask) -> bool {
        // 模拟任务执行
        // 实际实现应该执行任务的闭包
        task.set_state(TaskState::Completed);
        true
    }
    
    /// 获取统计信息
    pub fn stats(&self) -> SchedulerStats {
        if let Ok(stats) = self.stats.read() {
            stats.clone()
        } else {
            SchedulerStats::default()
        }
    }
    
    /// 获取配置
    pub fn config(&self) -> &SchedulerConfig {
        &self.config
    }
    
    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    /// 获取队列大小
    pub fn queue_size(&self) -> usize {
        if let Ok(queue) = self.global_queue.read() {
            queue.len()
        } else {
            0
        }
    }
    
    /// 获取活跃任务数
    pub fn active_task_count(&self) -> u64 {
        if let Ok(stats) = self.stats.read() {
            stats.active_tasks
        } else {
            0
        }
    }
    
    /// 重置统计信息
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.write() {
            *stats = SchedulerStats::default();
        }
    }
}

impl Default for CoroutineScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试调度器创建
    #[test]
    fn test_scheduler_creation() {
        let scheduler = CoroutineScheduler::new();
        
        assert!(!scheduler.is_running());
    }
    
    /// 测试配置
    #[test]
    fn test_config() {
        let config = SchedulerConfig::new()
            .worker_threads(4)
            .max_queue_size(1000);
        
        assert_eq!(config.worker_threads, 4);
        assert_eq!(config.max_queue_size, 1000);
    }
    
    /// 测试任务提交
    #[test]
    fn test_submit() {
        let scheduler = CoroutineScheduler::new();
        scheduler.start();
        
        let task = CoroutineTask::new("test_task");
        let task_id = scheduler.submit(task);
        
        assert!(task_id.0 > 0);
        assert_eq!(scheduler.queue_size(), 1);
        
        scheduler.stop();
    }
    
    /// 测试统计信息
    #[test]
    fn test_stats() {
        let stats = SchedulerStats {
            total_tasks: 100,
            completed_tasks: 90,
            failed_tasks: 10,
            total_runtime_ms: 1000,
            ..Default::default()
        };
        
        assert!((stats.throughput() - 90.0).abs() < f64::EPSILON);
        assert!((stats.success_rate() - 0.9).abs() < f64::EPSILON);
    }
}
