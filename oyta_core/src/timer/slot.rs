//! 时间轮槽位模块
//!
//! 实现时间轮的槽位数据结构
//! 每个槽位存储一个任务队列

use std::sync::{Arc, RwLock};
use std::collections::VecDeque;
use super::task::TaskEntry;

/// 时间轮槽位
/// 存储同一触发时间的任务队列
#[derive(Debug)]
pub struct Slot {
    /// 任务队列
    /// 使用 VecDeque 存储任务，支持高效的头部删除
    tasks: RwLock<VecDeque<Arc<TaskEntry>>>,
    
    /// 槽位索引
    /// 在时间轮中的位置
    index: usize,
}

impl Slot {
    /// 创建新的槽位
    ///
    /// # 参数
    /// - `index`: 槽位索引
    ///
    /// # 返回
    /// 返回新创建的槽位实例
    pub fn new(index: usize) -> Self {
        Self {
            // 初始化空的任务队列
            tasks: RwLock::new(VecDeque::new()),
            // 设置槽位索引
            index,
        }
    }
    
    /// 添加任务到槽位
    ///
    /// # 参数
    /// - `task`: 要添加的任务
    ///
    /// # 返回
    /// 返回添加后的任务数量
    pub fn add_task(&self, task: Arc<TaskEntry>) -> usize {
        // 获取写锁
        let mut tasks = self.tasks.write().unwrap();
        // 添加任务到队列尾部
        tasks.push_back(task);
        // 返回当前任务数量
        tasks.len()
    }
    
    /// 获取任务数量
    ///
    /// # 返回
    /// 返回槽位中的任务数量
    pub fn len(&self) -> usize {
        // 获取读锁
        let tasks = self.tasks.read().unwrap();
        // 返回任务数量
        tasks.len()
    }
    
    /// 检查槽位是否为空
    ///
    /// # 返回
    /// 如果槽位为空返回 true，否则返回 false
    pub fn is_empty(&self) -> bool {
        // 获取读锁
        let tasks = self.tasks.read().unwrap();
        // 检查是否为空
        tasks.is_empty()
    }
    
    /// 获取槽位索引
    ///
    /// # 返回
    /// 返回槽位索引
    pub fn get_index(&self) -> usize {
        self.index
    }
    
    /// 清空槽位
    /// 移除所有任务
    pub fn clear(&self) {
        // 获取写锁
        let mut tasks = self.tasks.write().unwrap();
        // 清空队列
        tasks.clear();
    }
    
    /// 排空槽位
    /// 取出所有任务并清空槽位
    ///
    /// # 返回
    /// 返回所有任务的向量
    pub fn drain(&self) -> Vec<Arc<TaskEntry>> {
        // 获取写锁
        let mut tasks = self.tasks.write().unwrap();
        // 取出所有任务
        tasks.drain(..).collect()
    }
    
    /// 收集可执行的任务
    /// 遍历任务队列，收集轮数为 0 且状态为 Pending 的任务
    ///
    /// # 返回
    /// 返回可执行的任务列表
    pub fn collect_executable_tasks(&self) -> Vec<Arc<TaskEntry>> {
        // 获取读锁
        let tasks = self.tasks.read().unwrap();
        
        // 收集可执行的任务
        let executable: Vec<Arc<TaskEntry>> = tasks
            .iter()
            .filter(|task| {
                // 检查轮数是否为 0
                task.get_rounds() == 0
                // 检查状态是否为 Pending
                && task.get_status() == super::task::TaskStatus::Pending
            })
            .cloned()
            .collect();
        
        executable
    }
    
    /// 处理任务轮数
    /// 遍历任务队列，递减轮数并移除已取消的任务
    ///
    /// # 返回
    /// 返回需要保留的任务数量
    pub fn process_rounds(&self) -> usize {
        // 获取写锁
        let mut tasks = self.tasks.write().unwrap();
        
        // 保留有效任务
        tasks.retain(|task| {
            // 移除已取消或已完成的任务
            if task.is_cancelled() || task.is_completed() {
                return false;
            }
            
            // 递减轮数
            let rounds = task.get_rounds();
            if rounds > 0 {
                task.decrement_rounds();
            }
            
            // 保留任务
            true
        });
        
        // 返回保留的任务数量
        tasks.len()
    }
    
    /// 移除指定任务
    ///
    /// # 参数
    /// - `task_id`: 要移除的任务 ID
    ///
    /// # 返回
    /// 如果找到并移除任务返回 true，否则返回 false
    pub fn remove_task(&self, task_id: u64) -> bool {
        // 获取写锁
        let mut tasks = self.tasks.write().unwrap();
        
        // 查找并移除任务
        let initial_len = tasks.len();
        tasks.retain(|task| task.id != task_id);
        
        // 检查是否移除了任务
        tasks.len() < initial_len
    }
    
    /// 查找任务
    ///
    /// # 参数
    /// - `task_id`: 要查找的任务 ID
    ///
    /// # 返回
    /// 返回任务的引用，如果不存在返回 None
    pub fn find_task(&self, task_id: u64) -> Option<Arc<TaskEntry>> {
        // 获取读锁
        let tasks = self.tasks.read().unwrap();
        
        // 查找任务
        tasks.iter()
            .find(|task| task.id == task_id)
            .cloned()
    }
    
    /// 获取所有任务
    ///
    /// # 返回
    /// 返回所有任务的克隆列表
    pub fn get_all_tasks(&self) -> Vec<Arc<TaskEntry>> {
        // 获取读锁
        let tasks = self.tasks.read().unwrap();
        // 克隆所有任务
        tasks.iter().cloned().collect()
    }
    
    /// 按优先级排序任务
    /// 将任务按优先级从高到低排序
    pub fn sort_by_priority(&self) {
        // 获取写锁
        let mut tasks = self.tasks.write().unwrap();
        // 按优先级降序排序
        tasks.make_contiguous().sort_by(|a, b| {
            b.priority.cmp(&a.priority)
        });
    }
}

/// 槽位迭代器
/// 用于遍历槽位中的任务
pub struct SlotIterator {
    /// 任务列表
    tasks: Vec<Arc<TaskEntry>>,
    /// 当前索引
    current: usize,
}

impl SlotIterator {
    /// 创建新的槽位迭代器
    ///
    /// # 参数
    /// - `slot`: 槽位引用
    ///
    /// # 返回
    /// 返回新创建的迭代器实例
    pub fn new(slot: &Slot) -> Self {
        Self {
            // 获取所有任务
            tasks: slot.get_all_tasks(),
            // 初始索引为 0
            current: 0,
        }
    }
}

impl Iterator for SlotIterator {
    type Item = Arc<TaskEntry>;
    
    fn next(&mut self) -> Option<Self::Item> {
        // 检查是否还有任务
        if self.current < self.tasks.len() {
            // 获取当前任务
            let task = self.tasks[self.current].clone();
            // 递增索引
            self.current += 1;
            // 返回任务
            Some(task)
        } else {
            // 没有更多任务
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::task::{TaskEntry, TimerCallback, TaskStatus};
    
    /// 测试槽位创建
    #[test]
    fn test_slot_creation() {
        // 创建槽位
        let slot = Slot::new(5);
        
        // 验证
        assert_eq!(slot.get_index(), 5);
        assert!(slot.is_empty());
        assert_eq!(slot.len(), 0);
    }
    
    /// 测试添加任务
    #[test]
    fn test_add_task() {
        // 创建槽位
        let slot = Slot::new(0);
        
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
        let count = slot.add_task(task);
        
        // 验证
        assert_eq!(count, 1);
        assert_eq!(slot.len(), 1);
        assert!(!slot.is_empty());
    }
    
    /// 测试查找任务
    #[test]
    fn test_find_task() {
        // 创建槽位
        let slot = Slot::new(0);
        
        // 创建并添加任务
        let task = Arc::new(TaskEntry::once(
            123,
            1000,
            TimerCallback::Function {
                name: "test".to_string(),
                args_json: vec![],
            },
            0,
        ));
        slot.add_task(task);
        
        // 查找任务
        let found = slot.find_task(123);
        assert!(found.is_some());
        
        // 查找不存在的任务
        let not_found = slot.find_task(456);
        assert!(not_found.is_none());
    }
    
    /// 测试移除任务
    #[test]
    fn test_remove_task() {
        // 创建槽位
        let slot = Slot::new(0);
        
        // 创建并添加任务
        let task = Arc::new(TaskEntry::once(
            1,
            1000,
            TimerCallback::Function {
                name: "test".to_string(),
                args_json: vec![],
            },
            0,
        ));
        slot.add_task(task);
        
        // 移除任务
        let removed = slot.remove_task(1);
        assert!(removed);
        assert!(slot.is_empty());
        
        // 移除不存在的任务
        let not_removed = slot.remove_task(999);
        assert!(!not_removed);
    }
    
    /// 测试排空槽位
    #[test]
    fn test_drain() {
        // 创建槽位
        let slot = Slot::new(0);
        
        // 添加多个任务
        for i in 0..5 {
            let task = Arc::new(TaskEntry::once(
                i,
                1000,
                TimerCallback::Function {
                    name: "test".to_string(),
                    args_json: vec![],
                },
                0,
            ));
            slot.add_task(task);
        }
        
        // 排空槽位
        let tasks = slot.drain();
        
        // 验证
        assert_eq!(tasks.len(), 5);
        assert!(slot.is_empty());
    }
    
    /// 测试收集可执行任务
    #[test]
    fn test_collect_executable_tasks() {
        // 创建槽位
        let slot = Slot::new(0);
        
        // 创建任务（轮数为 0）
        let task1 = Arc::new(TaskEntry::once(
            1,
            1000,
            TimerCallback::Function {
                name: "test".to_string(),
                args_json: vec![],
            },
            0,
        ));
        slot.add_task(task1);
        
        // 创建任务（轮数为 1）
        let task2 = Arc::new(TaskEntry::once(
            2,
            1000,
            TimerCallback::Function {
                name: "test".to_string(),
                args_json: vec![],
            },
            0,
        ));
        task2.set_rounds(1);
        slot.add_task(task2);
        
        // 收集可执行任务
        let executable = slot.collect_executable_tasks();
        
        // 验证只有轮数为 0 的任务被收集
        assert_eq!(executable.len(), 1);
        assert_eq!(executable[0].id, 1);
    }
    
    /// 测试槽位迭代器
    #[test]
    fn test_slot_iterator() {
        // 创建槽位
        let slot = Slot::new(0);
        
        // 添加任务
        for i in 0..3 {
            let task = Arc::new(TaskEntry::once(
                i,
                1000,
                TimerCallback::Function {
                    name: "test".to_string(),
                    args_json: vec![],
                },
                0,
            ));
            slot.add_task(task);
        }
        
        // 创建迭代器
        let iter = SlotIterator::new(&slot);
        
        // 遍历任务
        let count = iter.count();
        assert_eq!(count, 3);
    }
}
