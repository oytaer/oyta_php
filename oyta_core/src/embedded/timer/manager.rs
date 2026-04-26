//! 定时器管理器模块
//!
//! 提供定时任务的创建、启动、停止、暂停等管理功能

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{interval, sleep_until, Instant};

use super::cron::CronExpression;
use super::types::{TaskHandler, TaskResult, TaskStatus, TaskType, TimerTask};
use super::bridge::PhpInterpreterBridge;
use super::executor::{execute_handler, update_task_status};

/// 定时器管理器
pub struct TimerManager {
    /// 定时任务列表
    tasks: Arc<RwLock<HashMap<String, TimerTask>>>,
    /// 任务执行历史
    history: Arc<RwLock<Vec<TaskResult>>>,
    /// PHP 解释器引用（用于执行 PHP 处理器）
    interpreter: Option<Arc<PhpInterpreterBridge>>,
    /// 是否正在运行
    running: Arc<RwLock<bool>>,
}

impl TimerManager {
    /// 创建新的定时器管理器
    ///
    /// # 返回值
    /// 新的 TimerManager 实例
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            interpreter: None,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// 设置 PHP 解释器桥接器
    ///
    /// # 参数
    /// - `bridge`: PHP 解释器桥接器
    ///
    /// # 返回值
    /// 返回自身，支持链式调用
    pub fn with_interpreter(mut self, bridge: Arc<PhpInterpreterBridge>) -> Self {
        self.interpreter = Some(bridge);
        self
    }

    /// 添加定时任务
    ///
    /// # 参数
    /// - `task`: 定时任务
    ///
    /// # 返回
    /// 任务ID
    pub async fn add_task(&self, task: TimerTask) -> String {
        let task_id = task.id.clone();
        let task_name = task.name.clone();

        tracing::info!("注册定时任务: {} ({})", task_name, task_id);

        // 将任务添加到列表
        self.tasks.write().await.insert(task_id.clone(), task);

        task_id
    }

    /// 添加固定间隔任务
    ///
    /// # 参数
    /// - `name`: 任务名称
    /// - `interval_secs`: 执行间隔（秒）
    /// - `handler`: 任务处理器
    ///
    /// # 返回
    /// 任务ID
    pub async fn every(&self, name: &str, interval_secs: u64, handler: TaskHandler) -> String {
        // 生成任务ID
        let task_id = format!("timer_{}", uuid::Uuid::new_v4());

        // 创建任务
        let task = TimerTask {
            id: task_id.clone(),
            name: name.to_string(),
            task_type: TaskType::Interval { secs: interval_secs },
            handler,
            status: TaskStatus::Pending,
            last_run: None,
            next_run: None,
            run_count: 0,
            max_runs: 0,
            enabled: true,
            metadata: HashMap::new(),
        };

        // 添加任务
        self.add_task(task).await
    }

    /// 添加 Cron 任务
    ///
    /// # 参数
    /// - `name`: 任务名称
    /// - `expression`: Cron 表达式
    /// - `handler`: 任务处理器
    ///
    /// # 返回
    /// 任务ID
    pub async fn cron(&self, name: &str, expression: &str, handler: TaskHandler) -> String {
        // 生成任务ID
        let task_id = format!("cron_{}", uuid::Uuid::new_v4());

        // 创建任务
        let task = TimerTask {
            id: task_id.clone(),
            name: name.to_string(),
            task_type: TaskType::Cron {
                expression: expression.to_string(),
            },
            handler,
            status: TaskStatus::Pending,
            last_run: None,
            next_run: None,
            run_count: 0,
            max_runs: 0,
            enabled: true,
            metadata: HashMap::new(),
        };

        // 添加任务
        self.add_task(task).await
    }

    /// 添加一次性任务
    ///
    /// # 参数
    /// - `name`: 任务名称
    /// - `delay_secs`: 延迟时间（秒）
    /// - `handler`: 任务处理器
    ///
    /// # 返回
    /// 任务ID
    pub async fn once(&self, name: &str, delay_secs: u64, handler: TaskHandler) -> String {
        // 生成任务ID
        let task_id = format!("once_{}", uuid::Uuid::new_v4());

        // 创建任务
        let task = TimerTask {
            id: task_id.clone(),
            name: name.to_string(),
            task_type: TaskType::Once { delay_secs },
            handler,
            status: TaskStatus::Pending,
            last_run: None,
            next_run: None,
            run_count: 0,
            max_runs: 1,
            enabled: true,
            metadata: HashMap::new(),
        };

        // 添加任务
        self.add_task(task).await
    }

    /// 启动所有定时任务
    pub async fn start_all(&self) {
        // 设置运行状态
        *self.running.write().await = true;

        // 获取所有任务ID
        let tasks = self.tasks.read().await;
        let task_ids: Vec<String> = tasks.keys().cloned().collect();
        drop(tasks);

        // 启动每个任务
        for task_id in task_ids {
            self.start_task(&task_id).await;
        }

        tracing::info!("定时器管理器已启动");
    }

    /// 启动单个任务
    ///
    /// # 参数
    /// - `task_id`: 任务ID
    async fn start_task(&self, task_id: &str) {
        // 获取任务信息
        let tasks = self.tasks.read().await;
        let task = tasks.get(task_id).cloned();
        drop(tasks);

        if let Some(task) = task {
            // 检查是否启用
            if !task.enabled {
                return;
            }

            // 克隆任务信息用于异步任务
            let task_id = task.id.clone();
            let task_type = task.task_type.clone();
            let handler = task.handler.clone();
            let max_runs = task.max_runs;
            let tasks_ref = self.tasks.clone();
            let history_ref = self.history.clone();
            let interpreter_ref = self.interpreter.clone();

            // 启动异步任务
            tokio::spawn(async move {
                match task_type {
                    // 固定间隔任务
                    TaskType::Interval { secs } => {
                        let mut timer = interval(Duration::from_secs(secs));

                        loop {
                            // 等待下一个执行时间
                            timer.tick().await;

                            // 检查是否应该执行
                            let should_run = {
                                let tasks = tasks_ref.read().await;
                                if let Some(t) = tasks.get(&task_id) {
                                    if !t.enabled || t.status == TaskStatus::Paused {
                                        false
                                    } else if max_runs > 0 {
                                        t.run_count < max_runs
                                    } else {
                                        true
                                    }
                                } else {
                                    false
                                }
                            };

                            if !should_run {
                                break;
                            }

                            // 执行任务
                            let result = execute_handler(&handler, &interpreter_ref).await;

                            // 更新任务状态
                            update_task_status(&tasks_ref, &task_id, &result).await;

                            // 记录历史
                            history_ref.write().await.push(result);
                        }
                    }
                    // Cron 任务
                    TaskType::Cron { expression } => {
                        // 解析 Cron 表达式
                        let cron_expr = match CronExpression::parse(&expression) {
                            Ok(expr) => expr,
                            Err(e) => {
                                tracing::error!("无效的 Cron 表达式: {} - {}", expression, e);
                                return;
                            }
                        };

                        loop {
                            // 计算下次执行时间
                            let now = chrono::Local::now();
                            let next_run = cron_expr.next_run(&now);

                            if let Some(next) = next_run {
                                // 计算等待时间
                                let duration = (next - now).to_std().unwrap_or(Duration::ZERO);

                                // 等待到执行时间
                                sleep_until(Instant::now() + duration).await;

                                // 检查是否应该执行
                                let should_run = {
                                    let tasks = tasks_ref.read().await;
                                    if let Some(t) = tasks.get(&task_id) {
                                        t.enabled && t.status != TaskStatus::Paused
                                    } else {
                                        false
                                    }
                                };

                                if !should_run {
                                    break;
                                }

                                // 执行任务
                                let result = execute_handler(&handler, &interpreter_ref).await;

                                // 更新任务状态
                                update_task_status(&tasks_ref, &task_id, &result).await;

                                // 记录历史
                                history_ref.write().await.push(result);
                            } else {
                                // 无法计算下次执行时间，退出
                                break;
                            }
                        }
                    }
                    // 一次性任务
                    TaskType::Once { delay_secs } => {
                        // 等待延迟时间
                        sleep_until(Instant::now() + Duration::from_secs(delay_secs)).await;

                        // 执行任务
                        let result = execute_handler(&handler, &interpreter_ref).await;

                        // 更新任务状态
                        update_task_status(&tasks_ref, &task_id, &result).await;

                        // 记录历史
                        history_ref.write().await.push(result);

                        // 标记为已完成
                        let mut tasks = tasks_ref.write().await;
                        if let Some(t) = tasks.get_mut(&task_id) {
                            t.status = TaskStatus::Completed;
                        }
                    }
                    // 延迟任务
                    TaskType::Delayed {
                        delay_secs,
                        interval_secs,
                    } => {
                        // 先等待延迟时间
                        sleep_until(Instant::now() + Duration::from_secs(delay_secs)).await;

                        // 执行第一次
                        let result = execute_handler(&handler, &interpreter_ref).await;
                        update_task_status(&tasks_ref, &task_id, &result).await;
                        history_ref.write().await.push(result);

                        // 然后按间隔执行
                        let mut timer = interval(Duration::from_secs(interval_secs));

                        loop {
                            // 等待下一个执行时间
                            timer.tick().await;

                            // 检查是否应该执行
                            let should_run = {
                                let tasks = tasks_ref.read().await;
                                if let Some(t) = tasks.get(&task_id) {
                                    t.enabled && t.status != TaskStatus::Paused
                                } else {
                                    false
                                }
                            };

                            if !should_run {
                                break;
                            }

                            // 执行任务
                            let result = execute_handler(&handler, &interpreter_ref).await;
                            update_task_status(&tasks_ref, &task_id, &result).await;
                            history_ref.write().await.push(result);
                        }
                    }
                }
            });
        }
    }

    /// 停止所有任务
    pub async fn stop_all(&self) {
        // 设置运行状态
        *self.running.write().await = false;

        // 停止所有任务
        let mut tasks = self.tasks.write().await;
        for task in tasks.values_mut() {
            task.enabled = false;
            task.status = TaskStatus::Paused;
        }

        tracing::info!("所有定时任务已停止");
    }

    /// 暂停任务
    ///
    /// # 参数
    /// - `task_id`: 任务ID
    ///
    /// # 返回
    /// 是否成功
    pub async fn pause_task(&self, task_id: &str) -> bool {
        let mut tasks = self.tasks.write().await;

        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TaskStatus::Paused;
            task.enabled = false;
            tracing::info!("任务已暂停: {}", task_id);
            return true;
        }

        false
    }

    /// 恢复任务
    ///
    /// # 参数
    /// - `task_id`: 任务ID
    ///
    /// # 返回
    /// 是否成功
    pub async fn resume_task(&self, task_id: &str) -> bool {
        let mut tasks = self.tasks.write().await;

        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TaskStatus::Pending;
            task.enabled = true;
            tracing::info!("任务已恢复: {}", task_id);
            return true;
        }

        false
    }

    /// 删除任务
    ///
    /// # 参数
    /// - `task_id`: 任务ID
    ///
    /// # 返回
    /// 是否成功
    pub async fn remove_task(&self, task_id: &str) -> bool {
        let mut tasks = self.tasks.write().await;

        if tasks.remove(task_id).is_some() {
            tracing::info!("任务已删除: {}", task_id);
            return true;
        }

        false
    }

    /// 获取任务状态
    ///
    /// # 参数
    /// - `task_id`: 任务ID
    ///
    /// # 返回
    /// 任务信息
    pub async fn get_task(&self, task_id: &str) -> Option<TimerTask> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// 获取所有任务
    ///
    /// # 返回
    /// 任务列表
    pub async fn get_all_tasks(&self) -> Vec<TimerTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// 获取任务数量
    ///
    /// # 返回
    /// 任务数量
    pub async fn task_count(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks.len()
    }

    /// 获取执行历史
    ///
    /// # 参数
    /// - `limit`: 最大返回数量
    ///
    /// # 返回
    /// 执行历史列表
    pub async fn get_history(&self, limit: usize) -> Vec<TaskResult> {
        let history = self.history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }

    /// 清空执行历史
    pub async fn clear_history(&self) {
        let mut history = self.history.write().await;
        history.clear();
        tracing::info!("执行历史已清空");
    }
}

impl Default for TimerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试定时器管理器创建
    #[tokio::test]
    async fn test_timer_manager_new() {
        let manager = TimerManager::new();
        assert_eq!(manager.task_count().await, 0);
    }

    /// 测试添加间隔任务
    #[tokio::test]
    async fn test_add_interval_task() {
        let manager = TimerManager::new();

        let task_id = manager
            .every(
                "test_task",
                60,
                TaskHandler::Native {
                    name: "test_function".to_string(),
                },
            )
            .await;

        assert!(!task_id.is_empty());
        assert_eq!(manager.task_count().await, 1);
    }

    /// 测试添加 Cron 任务
    #[tokio::test]
    async fn test_add_cron_task() {
        let manager = TimerManager::new();

        let task_id = manager
            .cron(
                "hourly_task",
                "0 * * * *",
                TaskHandler::Native {
                    name: "hourly_function".to_string(),
                },
            )
            .await;

        assert!(!task_id.is_empty());
    }

    /// 测试添加一次性任务
    #[tokio::test]
    async fn test_add_once_task() {
        let manager = TimerManager::new();

        let task_id = manager
            .once(
                "delayed_task",
                10,
                TaskHandler::Native {
                    name: "delayed_function".to_string(),
                },
            )
            .await;

        assert!(!task_id.is_empty());

        let task = manager.get_task(&task_id).await.unwrap();
        assert_eq!(task.max_runs, 1);
    }

    /// 测试暂停和恢复任务
    #[tokio::test]
    async fn test_pause_resume_task() {
        let manager = TimerManager::new();

        let task_id = manager
            .every(
                "test_task",
                60,
                TaskHandler::Native {
                    name: "test".to_string(),
                },
            )
            .await;

        // 暂停任务
        assert!(manager.pause_task(&task_id).await);

        let task = manager.get_task(&task_id).await.unwrap();
        assert_eq!(task.status, TaskStatus::Paused);

        // 恢复任务
        assert!(manager.resume_task(&task_id).await);

        let task = manager.get_task(&task_id).await.unwrap();
        assert_eq!(task.status, TaskStatus::Pending);
    }

    /// 测试删除任务
    #[tokio::test]
    async fn test_remove_task() {
        let manager = TimerManager::new();

        let task_id = manager
            .every(
                "test_task",
                60,
                TaskHandler::Native {
                    name: "test".to_string(),
                },
            )
            .await;

        assert_eq!(manager.task_count().await, 1);
        assert!(manager.remove_task(&task_id).await);
        assert_eq!(manager.task_count().await, 0);
    }
}
