//! 单级时间轮模块
//!
//! 实现单级时间轮的数据结构和操作
//! 每级时间轮包含多个槽位，按固定间隔推进

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use super::slot::Slot;
use super::task::TaskEntry;
use super::error::{TimerError, TimerResult};

/// 单级时间轮配置
/// 定义单级时间轮的基本参数
#[derive(Debug, Clone)]
pub struct LevelConfig {
    /// 每槽时间间隔（毫秒）
    /// 决定时间轮的精度
    pub tick_ms: u64,
    
    /// 槽位数量
    /// 决定时间轮的周期
    pub slots_count: usize,
    
    /// 层级
    /// 0 = 毫秒轮，1 = 秒轮，2 = 分钟轮，3 = 小时轮
    pub level: u8,
}

impl LevelConfig {
    /// 创建新的层级配置
    ///
    /// # 参数
    /// - `tick_ms`: 每槽时间间隔（毫秒）
    /// - `slots_count`: 槽位数量
    /// - `level`: 层级
    ///
    /// # 返回
    /// 返回新创建的层级配置
    pub fn new(tick_ms: u64, slots_count: usize, level: u8) -> Self {
        Self {
            tick_ms,
            slots_count,
            level,
        }
    }
    
    /// 计算时间轮周期
    ///
    /// # 返回
    /// 返回时间轮的完整周期（毫秒）
    pub fn period_ms(&self) -> u64 {
        // 周期 = 每槽间隔 × 槽位数量
        self.tick_ms * self.slots_count as u64
    }
    
    /// 创建毫秒轮配置
    ///
    /// # 返回
    /// 返回毫秒轮的标准配置
    pub fn millisecond_wheel() -> Self {
        Self::new(1, 64, 0)
    }
    
    /// 创建秒轮配置
    ///
    /// # 返回
    /// 返回秒轮的标准配置
    pub fn second_wheel() -> Self {
        Self::new(64, 60, 1)  // 实际每槽约 64ms
    }
    
    /// 创建分钟轮配置
    ///
    /// # 返回
    /// 返回分钟轮的标准配置
    pub fn minute_wheel() -> Self {
        Self::new(3840, 60, 2)  // 实际每槽约 3.84s
    }
    
    /// 创建小时轮配置
    ///
    /// # 返回
    /// 返回小时轮的标准配置
    pub fn hour_wheel() -> Self {
        Self::new(230400, 24, 3)  // 实际每槽约 3.84 分钟
    }
}

/// 单级时间轮
/// 实现单级时间轮的核心数据结构
#[derive(Debug)]
pub struct TimeWheel {
    /// 槽位数组
    /// 存储所有槽位
    slots: Vec<Arc<Slot>>,
    
    /// 当前指针位置
    /// 使用原子类型支持并发访问
    current: AtomicUsize,
    
    /// 每个槽位的时间间隔（毫秒）
    /// 决定时间轮的精度
    tick_ms: u64,
    
    /// 总槽数
    /// 决定时间轮的周期
    slots_count: usize,
    
    /// 层级
    /// 0 = 毫秒轮，1 = 秒轮，2 = 分钟轮，3 = 小时轮
    level: u8,
    
    /// 配置
    /// 保存完整配置信息
    config: LevelConfig,
}

impl TimeWheel {
    /// 创建新的单级时间轮
    ///
    /// # 参数
    /// - `config`: 层级配置
    ///
    /// # 返回
    /// 返回新创建的时间轮实例
    pub fn new(config: LevelConfig) -> TimerResult<Self> {
        // 验证配置
        if config.slots_count == 0 {
            return Err(TimerError::InvalidLevelConfig {
                message: "槽位数量必须大于 0".to_string(),
            });
        }
        
        if config.tick_ms == 0 {
            return Err(TimerError::InvalidLevelConfig {
                message: "时间间隔必须大于 0".to_string(),
            });
        }
        
        // 创建槽位数组
        let slots: Vec<Arc<Slot>> = (0..config.slots_count)
            .map(|i| Arc::new(Slot::new(i)))
            .collect();
        
        Ok(Self {
            // 设置槽位数组
            slots,
            // 初始指针位置为 0
            current: AtomicUsize::new(0),
            // 设置时间间隔
            tick_ms: config.tick_ms,
            // 设置槽位数量
            slots_count: config.slots_count,
            // 设置层级
            level: config.level,
            // 保存配置
            config,
        })
    }
    
    /// 获取当前指针位置
    ///
    /// # 返回
    /// 返回当前指针位置
    pub fn get_current(&self) -> usize {
        self.current.load(Ordering::SeqCst)
    }
    
    /// 获取时间间隔
    ///
    /// # 返回
    /// 返回每个槽位的时间间隔（毫秒）
    pub fn get_tick_ms(&self) -> u64 {
        self.tick_ms
    }
    
    /// 获取槽位数量
    ///
    /// # 返回
    /// 返回总槽位数量
    pub fn get_slots_count(&self) -> usize {
        self.slots_count
    }
    
    /// 获取层级
    ///
    /// # 返回
    /// 返回当前层级
    pub fn get_level(&self) -> u8 {
        self.level
    }
    
    /// 获取时间轮周期
    ///
    /// # 返回
    /// 返回时间轮的完整周期（毫秒）
    pub fn get_period_ms(&self) -> u64 {
        self.config.period_ms()
    }
    
    /// 计算任务应该放入的槽位
    ///
    /// # 参数
    /// - `delay_ms`: 延迟时间（毫秒）
    /// - `current_time`: 当前时间（毫秒）
    ///
    /// # 返回
    /// 返回 (槽位索引, 轮数)
    pub fn calculate_position(&self, delay_ms: u64, current_time: u64) -> (usize, u32) {
        // 计算周期
        let period = self.get_period_ms();
        
        // 计算槽位
        let current_slot = self.get_current();
        let ticks = delay_ms / self.tick_ms;
        let slot = (current_slot + ticks as usize) % self.slots_count;
        
        // 计算轮数
        let rounds = (delay_ms / period) as u32;
        
        (slot, rounds)
    }
    
    /// 添加任务到指定槽位
    ///
    /// # 参数
    /// - `slot_index`: 槽位索引
    /// - `task`: 任务
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误
    pub fn add_task(&self, slot_index: usize, task: Arc<TaskEntry>) -> TimerResult<()> {
        // 验证槽位索引
        if slot_index >= self.slots_count {
            return Err(TimerError::InvalidTaskParams {
                message: format!("槽位索引 {} 超出范围", slot_index),
            });
        }
        
        // 添加任务到槽位
        self.slots[slot_index].add_task(task);
        
        Ok(())
    }
    
    /// 推进时间轮
    /// 将指针前进一格
    ///
    /// # 返回
    /// 如果时间轮转完一圈返回 true，否则返回 false
    pub fn advance(&self) -> bool {
        // 获取当前指针
        let current = self.current.load(Ordering::SeqCst);
        
        // 计算新位置
        let new_current = (current + 1) % self.slots_count;
        
        // 更新指针
        self.current.store(new_current, Ordering::SeqCst);
        
        // 检查是否转完一圈
        new_current == 0
    }
    
    /// 获取当前槽位
    ///
    /// # 返回
    /// 返回当前槽位的引用
    pub fn get_current_slot(&self) -> Arc<Slot> {
        let current = self.get_current();
        self.slots[current].clone()
    }
    
    /// 获取指定槽位
    ///
    /// # 参数
    /// - `index`: 槽位索引
    ///
    /// # 返回
    /// 返回槽位的引用，如果索引无效返回 None
    pub fn get_slot(&self, index: usize) -> Option<Arc<Slot>> {
        if index < self.slots_count {
            Some(self.slots[index].clone())
        } else {
            None
        }
    }
    
    /// 收集当前槽位的可执行任务
    ///
    /// # 返回
    /// 返回可执行的任务列表
    pub fn collect_current_executable(&self) -> Vec<Arc<TaskEntry>> {
        self.get_current_slot().collect_executable_tasks()
    }
    
    /// 处理当前槽位的任务轮数
    ///
    /// # 返回
    /// 返回保留的任务数量
    pub fn process_current_rounds(&self) -> usize {
        self.get_current_slot().process_rounds()
    }
    
    /// 排空当前槽位
    ///
    /// # 返回
    /// 返回所有任务
    pub fn drain_current_slot(&self) -> Vec<Arc<TaskEntry>> {
        self.get_current_slot().drain()
    }
    
    /// 重置时间轮
    /// 将指针重置为 0 并清空所有槽位
    pub fn reset(&self) {
        // 重置指针
        self.current.store(0, Ordering::SeqCst);
        
        // 清空所有槽位
        for slot in &self.slots {
            slot.clear();
        }
    }
    
    /// 获取总任务数量
    ///
    /// # 返回
    /// 返回所有槽位中的任务总数
    pub fn total_tasks(&self) -> usize {
        self.slots.iter().map(|s| s.len()).sum()
    }
    
    /// 获取非空槽位数量
    ///
    /// # 返回
    /// 返回包含任务的槽位数量
    pub fn non_empty_slots(&self) -> usize {
        self.slots.iter().filter(|s| !s.is_empty()).count()
    }
    
    /// 查找任务
    ///
    /// # 参数
    /// - `task_id`: 任务 ID
    ///
    /// # 返回
    /// 返回 (槽位索引, 任务)，如果未找到返回 None
    pub fn find_task(&self, task_id: u64) -> Option<(usize, Arc<TaskEntry>)> {
        // 遍历所有槽位
        for (index, slot) in self.slots.iter().enumerate() {
            // 在槽位中查找任务
            if let Some(task) = slot.find_task(task_id) {
                return Some((index, task));
            }
        }
        None
    }
    
    /// 移除任务
    ///
    /// # 参数
    /// - `task_id`: 任务 ID
    ///
    /// # 返回
    /// 如果找到并移除任务返回 true，否则返回 false
    pub fn remove_task(&self, task_id: u64) -> bool {
        // 遍历所有槽位
        for slot in &self.slots {
            // 尝试移除任务
            if slot.remove_task(task_id) {
                return true;
            }
        }
        false
    }
    
    /// 获取层级统计信息
    ///
    /// # 返回
    /// 返回层级统计信息
    pub fn get_stats(&self) -> LevelStats {
        LevelStats {
            // 层级
            level: self.level,
            // 当前指针位置
            current: self.get_current(),
            // 总任务数
            total_tasks: self.total_tasks(),
            // 非空槽位数
            non_empty_slots: self.non_empty_slots(),
            // 时间间隔
            tick_ms: self.tick_ms,
            // 周期
            period_ms: self.get_period_ms(),
        }
    }
}

/// 层级统计信息
#[derive(Debug, Clone)]
pub struct LevelStats {
    /// 层级
    pub level: u8,
    /// 当前指针位置
    pub current: usize,
    /// 总任务数
    pub total_tasks: usize,
    /// 非空槽位数
    pub non_empty_slots: usize,
    /// 时间间隔（毫秒）
    pub tick_ms: u64,
    /// 周期（毫秒）
    pub period_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::task::{TaskEntry, TimerCallback};
    
    /// 测试层级配置
    #[test]
    fn test_level_config() {
        // 创建毫秒轮配置
        let config = LevelConfig::millisecond_wheel();
        
        // 验证
        assert_eq!(config.tick_ms, 1);
        assert_eq!(config.slots_count, 64);
        assert_eq!(config.level, 0);
        assert_eq!(config.period_ms(), 64);
    }
    
    /// 测试时间轮创建
    #[test]
    fn test_time_wheel_creation() {
        // 创建配置
        let config = LevelConfig::millisecond_wheel();
        
        // 创建时间轮
        let wheel = TimeWheel::new(config).unwrap();
        
        // 验证
        assert_eq!(wheel.get_level(), 0);
        assert_eq!(wheel.get_current(), 0);
        assert_eq!(wheel.get_slots_count(), 64);
        assert_eq!(wheel.get_tick_ms(), 1);
    }
    
    /// 测试添加任务
    #[test]
    fn test_add_task() {
        // 创建时间轮
        let wheel = TimeWheel::new(LevelConfig::millisecond_wheel()).unwrap();
        
        // 创建任务
        let task = Arc::new(TaskEntry::once(
            1,
            1000,
            TimerCallback::Function {
                name: "test".to_string(),
                args_json: vec![],
            },
            0,
        ));
        
        // 添加任务
        wheel.add_task(10, task).unwrap();
        
        // 验证
        assert_eq!(wheel.total_tasks(), 1);
    }
    
    /// 测试推进时间轮
    #[test]
    fn test_advance() {
        // 创建时间轮
        let wheel = TimeWheel::new(LevelConfig::millisecond_wheel()).unwrap();
        
        // 初始位置为 0
        assert_eq!(wheel.get_current(), 0);
        
        // 推进一格
        let overflow = wheel.advance();
        
        // 验证
        assert_eq!(wheel.get_current(), 1);
        assert!(!overflow);
        
        // 推进到末尾
        for _ in 0..62 {
            wheel.advance();
        }
        assert_eq!(wheel.get_current(), 63);
        
        // 再推进一格，应该回到 0 并返回 true
        let overflow = wheel.advance();
        assert_eq!(wheel.get_current(), 0);
        assert!(overflow);
    }
    
    /// 测试计算位置
    #[test]
    fn test_calculate_position() {
        // 创建时间轮
        let wheel = TimeWheel::new(LevelConfig::millisecond_wheel()).unwrap();
        
        // 计算位置
        let (slot, rounds) = wheel.calculate_position(10, 0);
        
        // 验证
        assert_eq!(slot, 10);
        assert_eq!(rounds, 0);
        
        // 计算超过一轮的位置
        let (slot, rounds) = wheel.calculate_position(100, 0);
        
        // 验证
        assert_eq!(slot, 36);  // 100 % 64 = 36
        assert_eq!(rounds, 1);  // 100 / 64 = 1
    }
    
    /// 测试查找任务
    #[test]
    fn test_find_task() {
        // 创建时间轮
        let wheel = TimeWheel::new(LevelConfig::millisecond_wheel()).unwrap();
        
        // 添加任务
        let task = Arc::new(TaskEntry::once(
            123,
            1000,
            TimerCallback::Function {
                name: "test".to_string(),
                args_json: vec![],
            },
            0,
        ));
        wheel.add_task(10, task).unwrap();
        
        // 查找任务
        let found = wheel.find_task(123);
        assert!(found.is_some());
        let (slot, _) = found.unwrap();
        assert_eq!(slot, 10);
        
        // 查找不存在的任务
        let not_found = wheel.find_task(456);
        assert!(not_found.is_none());
    }
    
    /// 测试移除任务
    #[test]
    fn test_remove_task() {
        // 创建时间轮
        let wheel = TimeWheel::new(LevelConfig::millisecond_wheel()).unwrap();
        
        // 添加任务
        let task = Arc::new(TaskEntry::once(
            1,
            1000,
            TimerCallback::Function {
                name: "test".to_string(),
                args_json: vec![],
            },
            0,
        ));
        wheel.add_task(10, task).unwrap();
        
        // 移除任务
        let removed = wheel.remove_task(1);
        assert!(removed);
        assert_eq!(wheel.total_tasks(), 0);
    }
}
