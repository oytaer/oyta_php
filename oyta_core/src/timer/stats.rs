//! 定时器统计模块
//!
//! 实现定时器的统计信息收集和报告
//! 提供任务执行、性能等统计功能

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::collections::HashMap;

/// 定时器统计信息
/// 收集和存储定时器的运行统计数据
#[derive(Debug, Default)]
pub struct TimerStats {
    /// 总任务数
    pub total_tasks: AtomicU64,
    
    /// 待执行任务数
    pub pending_tasks: AtomicU64,
    
    /// 已完成任务数
    pub completed_tasks: AtomicU64,
    
    /// 已取消任务数
    pub cancelled_tasks: AtomicU64,
    
    /// 执行失败任务数
    pub failed_tasks: AtomicU64,
    
    /// Tick 计数
    pub tick_count: AtomicU64,
    
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: AtomicU64,
    
    /// 最大执行时间（毫秒）
    pub max_execution_time_ms: AtomicU64,
    
    /// 最小执行时间（毫秒）
    pub min_execution_time_ms: AtomicU64,
    
    /// 各层级任务分布
    pub level_distribution: RwLock<HashMap<u8, u64>>,
    
    /// 任务类型分布
    pub type_distribution: RwLock<HashMap<String, u64>>,
    
    /// 错误计数
    pub error_count: AtomicU64,
    
    /// 重试计数
    pub retry_count: AtomicU64,
}

impl TimerStats {
    /// 创建新的统计实例
    ///
    /// # 返回
    /// 返回新创建的统计实例
    pub fn new() -> Self {
        Self {
            // 初始化所有计数器为 0
            total_tasks: AtomicU64::new(0),
            pending_tasks: AtomicU64::new(0),
            completed_tasks: AtomicU64::new(0),
            cancelled_tasks: AtomicU64::new(0),
            failed_tasks: AtomicU64::new(0),
            tick_count: AtomicU64::new(0),
            total_execution_time_ms: AtomicU64::new(0),
            max_execution_time_ms: AtomicU64::new(0),
            min_execution_time_ms: AtomicU64::new(u64::MAX),
            // 初始化分布映射
            level_distribution: RwLock::new(HashMap::new()),
            type_distribution: RwLock::new(HashMap::new()),
            error_count: AtomicU64::new(0),
            retry_count: AtomicU64::new(0),
        }
    }
    
    /// 记录任务添加
    ///
    /// # 参数
    /// - `level`: 任务所在层级
    /// - `task_type`: 任务类型
    pub fn record_task_added(&self, level: u8, task_type: &str) {
        // 递增总任务数
        self.total_tasks.fetch_add(1, Ordering::SeqCst);
        
        // 递增待执行任务数
        self.pending_tasks.fetch_add(1, Ordering::SeqCst);
        
        // 更新层级分布
        if let Ok(mut dist) = self.level_distribution.write() {
            *dist.entry(level).or_insert(0) += 1;
        }
        
        // 更新类型分布
        if let Ok(mut dist) = self.type_distribution.write() {
            *dist.entry(task_type.to_string()).or_insert(0) += 1;
        }
    }
    
    /// 记录任务完成
    pub fn record_task_completed(&self) {
        // 递减待执行任务数
        self.pending_tasks.fetch_sub(1, Ordering::SeqCst);
        
        // 递增已完成任务数
        self.completed_tasks.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录任务取消
    pub fn record_task_cancelled(&self) {
        // 递减待执行任务数
        self.pending_tasks.fetch_sub(1, Ordering::SeqCst);
        
        // 递增已取消任务数
        self.cancelled_tasks.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录任务失败
    pub fn record_task_failed(&self) {
        // 递减待执行任务数
        self.pending_tasks.fetch_sub(1, Ordering::SeqCst);
        
        // 递增失败任务数
        self.failed_tasks.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录 Tick
    pub fn record_tick(&self) {
        self.tick_count.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录执行时间
    ///
    /// # 参数
    /// - `execution_time_ms`: 执行时间（毫秒）
    pub fn record_execution_time(&self, execution_time_ms: u64) {
        // 累加总执行时间
        self.total_execution_time_ms.fetch_add(execution_time_ms, Ordering::SeqCst);
        
        // 更新最大执行时间
        let mut max = self.max_execution_time_ms.load(Ordering::SeqCst);
        while execution_time_ms > max {
            match self.max_execution_time_ms.compare_exchange(
                max,
                execution_time_ms,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(current) => max = current,
            }
        }
        
        // 更新最小执行时间
        let mut min = self.min_execution_time_ms.load(Ordering::SeqCst);
        while execution_time_ms < min {
            match self.min_execution_time_ms.compare_exchange(
                min,
                execution_time_ms,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(current) => min = current,
            }
        }
    }
    
    /// 记录错误
    pub fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录重试
    pub fn record_retry(&self) {
        self.retry_count.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 获取总任务数
    pub fn get_total_tasks(&self) -> u64 {
        self.total_tasks.load(Ordering::SeqCst)
    }
    
    /// 获取待执行任务数
    pub fn get_pending_tasks(&self) -> u64 {
        self.pending_tasks.load(Ordering::SeqCst)
    }
    
    /// 获取已完成任务数
    pub fn get_completed_tasks(&self) -> u64 {
        self.completed_tasks.load(Ordering::SeqCst)
    }
    
    /// 获取已取消任务数
    pub fn get_cancelled_tasks(&self) -> u64 {
        self.cancelled_tasks.load(Ordering::SeqCst)
    }
    
    /// 获取 Tick 计数
    pub fn get_tick_count(&self) -> u64 {
        self.tick_count.load(Ordering::SeqCst)
    }
    
    /// 获取平均执行时间
    ///
    /// # 返回
    /// 返回平均执行时间（毫秒）
    pub fn get_avg_execution_time(&self) -> f64 {
        let total = self.total_execution_time_ms.load(Ordering::SeqCst);
        let count = self.completed_tasks.load(Ordering::SeqCst);
        
        if count > 0 {
            total as f64 / count as f64
        } else {
            0.0
        }
    }
    
    /// 获取最大执行时间
    pub fn get_max_execution_time(&self) -> u64 {
        self.max_execution_time_ms.load(Ordering::SeqCst)
    }
    
    /// 获取最小执行时间
    pub fn get_min_execution_time(&self) -> u64 {
        let min = self.min_execution_time_ms.load(Ordering::SeqCst);
        if min == u64::MAX {
            0
        } else {
            min
        }
    }
    
    /// 获取错误率
    ///
    /// # 返回
    /// 返回错误率（0.0 - 1.0）
    pub fn get_error_rate(&self) -> f64 {
        let total = self.total_tasks.load(Ordering::SeqCst);
        let errors = self.error_count.load(Ordering::SeqCst);
        
        if total > 0 {
            errors as f64 / total as f64
        } else {
            0.0
        }
    }
    
    /// 获取成功率
    ///
    /// # 返回
    /// 返回成功率（0.0 - 1.0）
    pub fn get_success_rate(&self) -> f64 {
        1.0 - self.get_error_rate()
    }
    
    /// 获取统计摘要
    ///
    /// # 返回
    /// 返回统计摘要的 JSON 字符串
    pub fn get_summary(&self) -> String {
        format!(
            r#"{{
    "total_tasks": {},
    "pending_tasks": {},
    "completed_tasks": {},
    "cancelled_tasks": {},
    "failed_tasks": {},
    "tick_count": {},
    "avg_execution_time_ms": {:.2},
    "max_execution_time_ms": {},
    "min_execution_time_ms": {},
    "error_count": {},
    "retry_count": {},
    "error_rate": {:.2},
    "success_rate": {:.2}
}}"#,
            self.get_total_tasks(),
            self.get_pending_tasks(),
            self.get_completed_tasks(),
            self.get_cancelled_tasks(),
            self.failed_tasks.load(Ordering::SeqCst),
            self.get_tick_count(),
            self.get_avg_execution_time(),
            self.get_max_execution_time(),
            self.get_min_execution_time(),
            self.error_count.load(Ordering::SeqCst),
            self.retry_count.load(Ordering::SeqCst),
            self.get_error_rate(),
            self.get_success_rate()
        )
    }
    
    /// 重置统计信息
    pub fn reset(&self) {
        // 重置所有计数器
        self.total_tasks.store(0, Ordering::SeqCst);
        self.pending_tasks.store(0, Ordering::SeqCst);
        self.completed_tasks.store(0, Ordering::SeqCst);
        self.cancelled_tasks.store(0, Ordering::SeqCst);
        self.failed_tasks.store(0, Ordering::SeqCst);
        self.tick_count.store(0, Ordering::SeqCst);
        self.total_execution_time_ms.store(0, Ordering::SeqCst);
        self.max_execution_time_ms.store(0, Ordering::SeqCst);
        self.min_execution_time_ms.store(u64::MAX, Ordering::SeqCst);
        self.error_count.store(0, Ordering::SeqCst);
        self.retry_count.store(0, Ordering::SeqCst);
        
        // 清空分布映射
        if let Ok(mut dist) = self.level_distribution.write() {
            dist.clear();
        }
        if let Ok(mut dist) = self.type_distribution.write() {
            dist.clear();
        }
    }
}

impl Clone for TimerStats {
    fn clone(&self) -> Self {
        Self {
            total_tasks: AtomicU64::new(self.total_tasks.load(Ordering::SeqCst)),
            pending_tasks: AtomicU64::new(self.pending_tasks.load(Ordering::SeqCst)),
            completed_tasks: AtomicU64::new(self.completed_tasks.load(Ordering::SeqCst)),
            cancelled_tasks: AtomicU64::new(self.cancelled_tasks.load(Ordering::SeqCst)),
            failed_tasks: AtomicU64::new(self.failed_tasks.load(Ordering::SeqCst)),
            tick_count: AtomicU64::new(self.tick_count.load(Ordering::SeqCst)),
            total_execution_time_ms: AtomicU64::new(self.total_execution_time_ms.load(Ordering::SeqCst)),
            max_execution_time_ms: AtomicU64::new(self.max_execution_time_ms.load(Ordering::SeqCst)),
            min_execution_time_ms: AtomicU64::new(self.min_execution_time_ms.load(Ordering::SeqCst)),
            level_distribution: RwLock::new(self.level_distribution.read().unwrap().clone()),
            type_distribution: RwLock::new(self.type_distribution.read().unwrap().clone()),
            error_count: AtomicU64::new(self.error_count.load(Ordering::SeqCst)),
            retry_count: AtomicU64::new(self.retry_count.load(Ordering::SeqCst)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试统计创建
    #[test]
    fn test_stats_creation() {
        let stats = TimerStats::new();
        
        assert_eq!(stats.get_total_tasks(), 0);
        assert_eq!(stats.get_pending_tasks(), 0);
        assert_eq!(stats.get_completed_tasks(), 0);
    }
    
    /// 测试记录任务添加
    #[test]
    fn test_record_task_added() {
        let stats = TimerStats::new();
        
        stats.record_task_added(0, "once");
        stats.record_task_added(1, "periodic");
        
        assert_eq!(stats.get_total_tasks(), 2);
        assert_eq!(stats.get_pending_tasks(), 2);
    }
    
    /// 测试记录任务完成
    #[test]
    fn test_record_task_completed() {
        let stats = TimerStats::new();
        
        stats.record_task_added(0, "once");
        stats.record_task_completed();
        
        assert_eq!(stats.get_completed_tasks(), 1);
        assert_eq!(stats.get_pending_tasks(), 0);
    }
    
    /// 测试记录执行时间
    #[test]
    fn test_record_execution_time() {
        let stats = TimerStats::new();
        
        stats.record_execution_time(100);
        stats.record_execution_time(200);
        stats.record_execution_time(50);
        
        assert_eq!(stats.get_max_execution_time(), 200);
        assert_eq!(stats.get_min_execution_time(), 50);
    }
    
    /// 测试统计摘要
    #[test]
    fn test_get_summary() {
        let stats = TimerStats::new();
        
        stats.record_task_added(0, "once");
        stats.record_task_completed();
        
        let summary = stats.get_summary();
        assert!(summary.contains("\"total_tasks\": 1"));
    }
    
    /// 测试重置
    #[test]
    fn test_reset() {
        let stats = TimerStats::new();
        
        stats.record_task_added(0, "once");
        stats.record_task_completed();
        stats.reset();
        
        assert_eq!(stats.get_total_tasks(), 0);
        assert_eq!(stats.get_completed_tasks(), 0);
    }
}
