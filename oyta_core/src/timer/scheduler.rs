//! 定时器调度器模块
//!
//! 实现定时器的调度逻辑
//! 负责任务的执行和周期任务的重调度

use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use super::wheel::HierarchicalTimingWheel;
use super::task::{TaskEntry, TaskType, TaskStatus};
use super::callback::CallbackHandler;
use super::error::TimerResult;

/// 定时器调度器
/// 负责驱动时间轮和处理任务执行
#[derive(Debug)]
pub struct TimerScheduler {
    /// 时间轮引用
    wheel: Arc<HierarchicalTimingWheel>,
    
    /// 回调处理器
    callback_handler: CallbackHandler,
    
    /// 运行标志（使用 Arc 包装以便在线程间共享）
    running: Arc<AtomicBool>,
    
    /// 工作线程句柄
    worker_handle: Option<JoinHandle<()>>,
}

impl TimerScheduler {
    /// 创建新的调度器
    ///
    /// # 参数
    /// - `wheel`: 时间轮引用
    ///
    /// # 返回
    /// 返回新创建的调度器实例
    pub fn new(wheel: Arc<HierarchicalTimingWheel>) -> Self {
        Self {
            // 设置时间轮引用
            wheel,
            // 创建回调处理器
            callback_handler: CallbackHandler::new(),
            // 初始状态为未运行（使用 Arc 包装）
            running: Arc::new(AtomicBool::new(false)),
            // 工作线程句柄初始为空
            worker_handle: None,
        }
    }
    
    /// 启动调度器
    /// 创建工作线程开始驱动时间轮
    ///
    /// # 返回
    /// 成功返回 Ok(())
    pub fn start(&mut self) -> TimerResult<()> {
        // 检查是否已运行
        if self.running.load(Ordering::SeqCst) {
            return Ok(());
        }
        
        // 设置运行标志
        self.running.store(true, Ordering::SeqCst);
        
        // 启动时间轮
        self.wheel.start();
        
        // 克隆引用用于工作线程
        let wheel = Arc::clone(&self.wheel);
        let running = self.running.clone();
        let callback_handler = self.callback_handler.clone();
        
        // 创建工作线程
        let handle = thread::spawn(move || {
            // 循环执行
            while running.load(Ordering::SeqCst) {
                // 推进时间轮并获取待执行任务
                let tasks = wheel.tick();
                
                // 执行任务
                for task in tasks {
                    // 执行任务回调
                    let result = callback_handler.execute(&task);
                    
                    // 处理执行结果
                    match result {
                        Ok(_) => {
                            // 标记任务已执行
                            task.record_execution(wheel.get_current_time());
                            
                            // 处理周期性任务
                            if let TaskType::Periodic { interval_ms } = task.task_type {
                                // 重新添加周期性任务
                                let _ = wheel.add_task(
                                    interval_ms,
                                    task.callback.clone(),
                                    TaskType::Periodic { interval_ms },
                                );
                            }
                        }
                        Err(_) => {
                            // 执行失败，检查是否需要重试
                            if task.can_retry() {
                                task.record_retry();
                                // 重新添加任务
                                if let Some(interval) = task.get_interval() {
                                    let _ = wheel.add_task(
                                        interval,
                                        task.callback.clone(),
                                        task.task_type.clone(),
                                    );
                                }
                            } else {
                                // 标记任务失败
                                task.set_status(TaskStatus::Completed);
                            }
                        }
                    }
                }
                
                // 休眠 1 毫秒
                thread::sleep(Duration::from_millis(1));
            }
        });
        
        // 保存线程句柄
        self.worker_handle = Some(handle);
        
        Ok(())
    }
    
    /// 停止调度器
    /// 停止工作线程
    pub fn stop(&mut self) {
        // 设置停止标志
        self.running.store(false, Ordering::SeqCst);
        
        // 停止时间轮
        self.wheel.stop();
        
        // 等待工作线程结束
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
    
    /// 检查是否运行中
    ///
    /// # 返回
    /// 如果调度器正在运行返回 true
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    /// 手动执行一次 tick
    /// 用于测试或手动驱动场景
    ///
    /// # 返回
    /// 返回执行的任务数量
    pub fn tick_once(&self) -> usize {
        // 推进时间轮
        let tasks = self.wheel.tick();
        
        // 执行任务
        for task in &tasks {
            let _ = self.callback_handler.execute(task);
            task.record_execution(self.wheel.get_current_time());
        }
        
        // 返回执行的任务数量
        tasks.len()
    }
}

impl Clone for TimerScheduler {
    fn clone(&self) -> Self {
        Self {
            wheel: Arc::clone(&self.wheel),
            callback_handler: self.callback_handler.clone(),
            running: Arc::clone(&self.running),
            worker_handle: None,  // 克隆时不复制线程句柄
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试调度器创建
    #[test]
    fn test_scheduler_creation() {
        // 创建时间轮
        let wheel = Arc::new(HierarchicalTimingWheel::with_defaults().unwrap());
        
        // 创建调度器
        let scheduler = TimerScheduler::new(wheel);
        
        // 验证
        assert!(!scheduler.is_running());
    }
    
    /// 测试手动 tick
    #[test]
    fn test_tick_once() {
        // 创建时间轮
        let wheel = Arc::new(HierarchicalTimingWheel::with_defaults().unwrap());
        
        // 创建调度器
        let scheduler = TimerScheduler::new(wheel.clone());
        
        // 执行一次 tick
        let count = scheduler.tick_once();
        
        // 验证
        assert_eq!(count, 0);  // 没有任务
        assert_eq!(wheel.get_current_time(), 1);
    }
}
