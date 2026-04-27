//! 多级时间轮核心模块
//!
//! 实现多级时间轮的核心数据结构和操作
//! 支持从毫秒到天级别的超时范围

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use super::level::{LevelConfig, TimeWheel};
use super::slot::Slot;
use super::task::{TaskEntry, TaskType, TaskStatus, TaskInfo, TimerCallback};
use super::error::{TimerError, TimerResult};

/// 多级时间轮配置
#[derive(Debug, Clone)]
pub struct WheelConfig {
    /// 各层级配置
    pub levels: Vec<LevelConfig>,
    
    /// 最大超时时间（毫秒）
    pub max_timeout_ms: u64,
    
    /// 最大任务数
    pub max_tasks: usize,
    
    /// 是否启用统计
    pub enable_stats: bool,
}

impl Default for WheelConfig {
    fn default() -> Self {
        Self {
            // 默认 4 级配置
            levels: vec![
                LevelConfig::millisecond_wheel(),
                LevelConfig::second_wheel(),
                LevelConfig::minute_wheel(),
                LevelConfig::hour_wheel(),
            ],
            // 最大超时 24 小时
            max_timeout_ms: 86400000,
            // 最大任务数
            max_tasks: 100000,
            // 启用统计
            enable_stats: true,
        }
    }
}

/// 任务位置信息
/// 用于快速定位任务
#[derive(Debug, Clone)]
pub struct TaskLocation {
    /// 所在层级
    pub level: u8,
    /// 所在槽位
    pub slot: usize,
    /// 任务 ID
    pub task_id: u64,
}

/// 多级时间轮
/// 实现高性能的分层时间轮定时器
#[derive(Debug)]
pub struct HierarchicalTimingWheel {
    /// 多级时间轮数组
    wheels: Vec<TimeWheel>,
    
    /// 当前时间（毫秒）
    current_ms: AtomicU64,
    
    /// 任务 ID 生成器
    next_id: AtomicU64,
    
    /// 任务映射表（用于快速查找和删除）
    task_map: RwLock<HashMap<u64, TaskLocation>>,
    
    /// 运行状态
    running: AtomicBool,
    
    /// 配置
    config: WheelConfig,
    
    /// 统计信息
    stats: RwLock<WheelStats>,
}

/// 时间轮统计信息
#[derive(Debug, Clone, Default)]
pub struct WheelStats {
    /// 总任务数
    pub total_tasks: u64,
    
    /// 待执行任务数
    pub pending_tasks: u64,
    
    /// 已完成任务数
    pub completed_tasks: u64,
    
    /// 已取消任务数
    pub cancelled_tasks: u64,
    
    /// Tick 计数
    pub tick_count: u64,
    
    /// 各层级任务分布
    pub level_distribution: Vec<u64>,
}

impl HierarchicalTimingWheel {
    /// 创建新的多级时间轮
    ///
    /// # 参数
    /// - `config`: 时间轮配置
    ///
    /// # 返回
    /// 返回新创建的时间轮实例
    pub fn new(config: WheelConfig) -> TimerResult<Self> {
        // 验证配置
        if config.levels.is_empty() {
            return Err(TimerError::InvalidLevelConfig {
                message: "必须至少有一个层级".to_string(),
            });
        }
        
        // 创建各级时间轮
        let wheels: Vec<TimeWheel> = config.levels
            .iter()
            .map(|level_config| TimeWheel::new(level_config.clone()))
            .collect::<TimerResult<Vec<TimeWheel>>>()?;
        
        // 初始化统计信息
        let stats = WheelStats {
            level_distribution: vec![0; wheels.len()],
            ..Default::default()
        };
        
        Ok(Self {
            // 设置时间轮数组
            wheels,
            // 初始时间为 0
            current_ms: AtomicU64::new(0),
            // 初始 ID 为 1
            next_id: AtomicU64::new(1),
            // 初始化任务映射表
            task_map: RwLock::new(HashMap::new()),
            // 初始状态为运行
            running: AtomicBool::new(true),
            // 保存配置
            config,
            // 初始化统计信息
            stats: RwLock::new(stats),
        })
    }
    
    /// 使用默认配置创建时间轮
    ///
    /// # 返回
    /// 返回使用默认配置创建的时间轮实例
    pub fn with_defaults() -> TimerResult<Self> {
        Self::new(WheelConfig::default())
    }
    
    /// 添加定时任务
    ///
    /// # 参数
    /// - `delay_ms`: 延迟时间（毫秒）
    /// - `callback`: 回调函数
    /// - `task_type`: 任务类型
    ///
    /// # 返回
    /// 返回任务 ID
    pub fn add_task(
        &self,
        delay_ms: u64,
        callback: TimerCallback,
        task_type: TaskType,
    ) -> TimerResult<u64> {
        // 验证延迟时间
        if delay_ms > self.config.max_timeout_ms {
            return Err(TimerError::timeout_too_large(delay_ms, self.config.max_timeout_ms));
        }
        
        // 生成任务 ID
        let task_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        
        // 获取当前时间
        let current_time = self.current_ms.load(Ordering::SeqCst);
        
        // 创建任务条目
        let task = match task_type {
            TaskType::Once => Arc::new(TaskEntry::once(task_id, delay_ms, callback, current_time)),
            TaskType::Periodic { interval_ms } => Arc::new(TaskEntry::periodic(task_id, interval_ms, callback, current_time)),
            TaskType::Times { interval_ms, remaining } => Arc::new(TaskEntry::times(task_id, interval_ms, remaining, callback, current_time)),
        };
        
        // 计算目标层级和槽位
        let (level, slot, rounds) = self.calculate_position(delay_ms)?;
        
        // 设置任务轮数
        task.set_rounds(rounds);
        
        // 添加到对应层级的时间轮
        self.wheels[level as usize].add_task(slot, task.clone())?;
        
        // 记录任务位置
        self.task_map.write().unwrap().insert(task_id, TaskLocation {
            level,
            slot,
            task_id,
        });
        
        // 更新统计信息
        if self.config.enable_stats {
            let mut stats = self.stats.write().unwrap();
            stats.total_tasks += 1;
            stats.pending_tasks += 1;
            stats.level_distribution[level as usize] += 1;
        }
        
        Ok(task_id)
    }
    
    /// 计算任务应该放入的层级、槽位和轮数
    ///
    /// # 参数
    /// - `delay_ms`: 延迟时间（毫秒）
    ///
    /// # 返回
    /// 返回 (层级, 槽位, 轮数)
    fn calculate_position(&self, delay_ms: u64) -> TimerResult<(u8, usize, u32)> {
        // 遍历各级时间轮
        for (level, wheel) in self.wheels.iter().enumerate() {
            // 获取时间轮周期
            let wheel_period = wheel.get_period_ms();
            
            // 检查任务是否适合当前层级
            if delay_ms < wheel_period {
                // 计算槽位
                let current_slot = wheel.get_current();
                let ticks = delay_ms / wheel.get_tick_ms();
                let slot = (current_slot + ticks as usize) % wheel.get_slots_count();
                
                // 计算轮数
                let rounds = (delay_ms / wheel_period) as u32;
                
                return Ok((level as u8, slot, rounds));
            }
        }
        
        // 超出最大层级，放入最高层级
        let last_wheel = self.wheels.last().ok_or(TimerError::InvalidLevelConfig {
            message: "没有可用的时间轮层级".to_string(),
        })?;
        
        // 计算最高层级的位置
        let current_slot = last_wheel.get_current();
        let ticks = delay_ms / last_wheel.get_tick_ms();
        let slot = (current_slot + ticks as usize) % last_wheel.get_slots_count();
        let rounds = (delay_ms / last_wheel.get_period_ms()) as u32;
        
        Ok(((self.wheels.len() - 1) as u8, slot, rounds))
    }
    
    /// Tick 推进（每毫秒调用一次）
    ///
    /// # 返回
    /// 返回待执行的任务列表
    pub fn tick(&self) -> Vec<Arc<TaskEntry>> {
        // 检查运行状态
        if !self.running.load(Ordering::SeqCst) {
            return Vec::new();
        }
        
        // 收集待执行的任务
        let mut ready_tasks = Vec::new();
        
        // 推进毫秒轮
        let overflow = self.wheels[0].advance();
        
        // 收集当前槽位的可执行任务
        let current_tasks = self.wheels[0].collect_current_executable();
        ready_tasks.extend(current_tasks);
        
        // 处理溢出（毫秒轮转完一圈）
        if overflow {
            self.cascade(1);
        }
        
        // 更新当前时间
        self.current_ms.fetch_add(1, Ordering::SeqCst);
        
        // 更新统计
        if self.config.enable_stats {
            self.stats.write().unwrap().tick_count += 1;
        }
        
        ready_tasks
    }
    
    /// 层级下沉（从上级时间轮向下级转移任务）
    ///
    /// # 参数
    /// - `level`: 要处理的层级
    fn cascade(&self, level: usize) {
        // 检查层级是否有效
        if level >= self.wheels.len() {
            return;
        }
        
        // 推进当前层级
        let overflow = self.wheels[level].advance();
        
        // 获取当前槽位的所有任务
        let tasks = self.wheels[level].drain_current_slot();
        
        // 获取当前时间
        let current_time = self.current_ms.load(Ordering::SeqCst);
        
        // 重新计算并下沉任务
        for task in tasks {
            // 跳过已取消或已完成的任务
            if task.is_cancelled() || task.is_completed() {
                continue;
            }
            
            // 计算剩余时间
            let remaining = task.execute_at.saturating_sub(current_time);
            
            if remaining > 0 {
                // 重新计算位置
                if let Ok((new_level, new_slot, rounds)) = self.calculate_position(remaining) {
                    // 设置新的轮数
                    task.set_rounds(rounds);
                    
                    // 添加到新的位置
                    if let Err(_) = self.wheels[new_level as usize].add_task(new_slot, task.clone()) {
                        continue;
                    }
                    
                    // 更新任务映射
                    if let Some(location) = self.task_map.write().unwrap().get_mut(&task.id) {
                        location.level = new_level;
                        location.slot = new_slot;
                    }
                }
            } else {
                // 任务已到期，添加到毫秒轮立即执行
                let _ = self.wheels[0].add_task(self.wheels[0].get_current(), task);
            }
        }
        
        // 递归处理更高级别的溢出
        if overflow && level + 1 < self.wheels.len() {
            self.cascade(level + 1);
        }
    }
    
    /// 取消任务
    ///
    /// # 参数
    /// - `task_id`: 任务 ID
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误
    pub fn cancel(&self, task_id: u64) -> TimerResult<()> {
        // 查找任务位置
        let location = self.task_map.read().unwrap()
            .get(&task_id)
            .cloned()
            .ok_or(TimerError::task_not_found(task_id))?;
        
        // 获取任务并设置状态
        if let Some(wheel) = self.wheels.get(location.level as usize) {
            if let Some(slot) = wheel.get_slot(location.slot) {
                if let Some(task) = slot.find_task(task_id) {
                    task.set_status(TaskStatus::Cancelled);
                    
                    // 更新统计
                    if self.config.enable_stats {
                        let mut stats = self.stats.write().unwrap();
                        stats.pending_tasks = stats.pending_tasks.saturating_sub(1);
                        stats.cancelled_tasks += 1;
                    }
                    
                    return Ok(());
                }
            }
        }
        
        Err(TimerError::task_not_found(task_id))
    }
    
    /// 获取任务状态
    ///
    /// # 参数
    /// - `task_id`: 任务 ID
    ///
    /// # 返回
    /// 返回任务信息，如果不存在返回 None
    pub fn status(&self, task_id: u64) -> Option<TaskInfo> {
        // 查找任务位置
        let location = self.task_map.read().unwrap().get(&task_id).cloned()?;
        
        // 获取任务
        let wheel = self.wheels.get(location.level as usize)?;
        let slot = wheel.get_slot(location.slot)?;
        let task = slot.find_task(task_id)?;
        
        // 计算剩余时间
        let current_time = self.current_ms.load(Ordering::SeqCst);
        let remaining_ms = task.execute_at.saturating_sub(current_time);
        
        Some(TaskInfo {
            id: task.id,
            status: task.get_status(),
            task_type: task.task_type.clone(),
            remaining_ms,
            level: location.level,
            slot: location.slot,
            execution_count: task.get_execution_count(),
            created_at: 0,  // 简化处理
            execute_at: task.execute_at,
        })
    }
    
    /// 获取统计信息
    ///
    /// # 返回
    /// 返回时间轮统计信息
    pub fn get_stats(&self) -> WheelStats {
        self.stats.read().unwrap().clone()
    }
    
    /// 停止时间轮
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
    
    /// 启动时间轮
    pub fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
    }
    
    /// 检查是否运行中
    ///
    /// # 返回
    /// 如果时间轮正在运行返回 true
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
    
    /// 获取当前时间
    ///
    /// # 返回
    /// 返回当前时间（毫秒）
    pub fn get_current_time(&self) -> u64 {
        self.current_ms.load(Ordering::SeqCst)
    }
    
    /// 获取任务总数
    ///
    /// # 返回
    /// 返回所有层级中的任务总数
    pub fn total_tasks(&self) -> usize {
        self.wheels.iter().map(|w| w.total_tasks()).sum()
    }
    
    /// 获取层级数量
    ///
    /// # 返回
    /// 返回时间轮的层级数量
    pub fn level_count(&self) -> usize {
        self.wheels.len()
    }
    
    /// 获取指定层级的时间轮
    ///
    /// # 参数
    /// - `level`: 层级索引
    ///
    /// # 返回
    /// 返回时间轮引用，如果不存在返回 None
    pub fn get_wheel(&self, level: usize) -> Option<&TimeWheel> {
        self.wheels.get(level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试默认配置创建
    #[test]
    fn test_default_creation() {
        // 创建时间轮
        let wheel = HierarchicalTimingWheel::with_defaults().unwrap();
        
        // 验证
        assert_eq!(wheel.level_count(), 4);
        assert!(wheel.is_running());
        assert_eq!(wheel.get_current_time(), 0);
    }
    
    /// 测试添加一次性任务
    #[test]
    fn test_add_once_task() {
        // 创建时间轮
        let wheel = HierarchicalTimingWheel::with_defaults().unwrap();
        
        // 添加任务
        let task_id = wheel.add_task(
            1000,
            TimerCallback::Function {
                name: "test".to_string(),
                args_json: vec![],
            },
            TaskType::Once,
        ).unwrap();
        
        // 验证
        assert!(task_id > 0);
        assert_eq!(wheel.total_tasks(), 1);
    }
    
    /// 测试添加周期性任务
    #[test]
    fn test_add_periodic_task() {
        // 创建时间轮
        let wheel = HierarchicalTimingWheel::with_defaults().unwrap();
        
        // 添加任务
        let task_id = wheel.add_task(
            2000,
            TimerCallback::Function {
                name: "test".to_string(),
                args_json: vec![],
            },
            TaskType::Periodic { interval_ms: 2000 },
        ).unwrap();
        
        // 验证
        assert!(task_id > 0);
    }
    
    /// 测试取消任务
    #[test]
    fn test_cancel_task() {
        // 创建时间轮
        let wheel = HierarchicalTimingWheel::with_defaults().unwrap();
        
        // 添加任务
        let task_id = wheel.add_task(
            1000,
            TimerCallback::Function {
                name: "test".to_string(),
                args_json: vec![],
            },
            TaskType::Once,
        ).unwrap();
        
        // 取消任务
        wheel.cancel(task_id).unwrap();
        
        // 验证状态
        let status = wheel.status(task_id).unwrap();
        assert_eq!(status.status, TaskStatus::Cancelled);
    }
    
    /// 测试 Tick 推进
    #[test]
    fn test_tick() {
        // 创建时间轮
        let wheel = HierarchicalTimingWheel::with_defaults().unwrap();
        
        // 推进多个 tick
        for _ in 0..100 {
            wheel.tick();
        }
        
        // 验证时间
        assert_eq!(wheel.get_current_time(), 100);
    }
    
    /// 测试超时时间超出最大值
    #[test]
    fn test_timeout_too_large() {
        // 创建时间轮
        let wheel = HierarchicalTimingWheel::with_defaults().unwrap();
        
        // 尝试添加超时过大的任务
        let result = wheel.add_task(
            100000000,  // 超过 24 小时
            TimerCallback::Function {
                name: "test".to_string(),
                args_json: vec![],
            },
            TaskType::Once,
        );
        
        // 验证错误
        assert!(result.is_err());
    }
    
    /// 测试停止和启动
    #[test]
    fn test_stop_start() {
        // 创建时间轮
        let wheel = HierarchicalTimingWheel::with_defaults().unwrap();
        
        // 停止
        wheel.stop();
        assert!(!wheel.is_running());
        
        // 启动
        wheel.start();
        assert!(wheel.is_running());
    }
}
