//! 定时器内嵌服务
//!
//! 提供定时任务调度功能
//! 支持：固定间隔、Cron 表达式、一次性任务、延迟任务
//!
//! # 功能特性
//! - 固定间隔执行
//! - Cron 表达式调度
//! - 一次性任务
//! - 延迟执行
//! - 任务状态追踪
//! - 与 PHP 解释器集成
//!
//! # 使用示例
//! ```php
//! // 在 PHP 中定义定时任务
//! Schedule::every(60, 'ProcessQueueTask');
//! Schedule::cron('0 * * * *', 'HourlyReportTask');
//! Schedule::once(300, 'DelayedNotificationTask');
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{interval, sleep_until, Instant};

use super::cron::CronExpression;

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

/// PHP 解释器桥接器
///
/// 用于与 PHP 解释器集成，执行 PHP 任务处理器
pub struct PhpInterpreterBridge {
    /// 项目路径
    project_path: String,
    /// 入口文件
    entry_point: String,
}

impl PhpInterpreterBridge {
    /// 创建新的 PHP 解释器桥接器
    pub fn new(project_path: &str, entry_point: &str) -> Self {
        Self {
            project_path: project_path.to_string(),
            entry_point: entry_point.to_string(),
        }
    }

    /// 执行 PHP 类方法
    ///
    /// # 参数
    /// - `class`: 类名
    /// - `method`: 方法名
    /// - `args`: 参数列表
    ///
    /// # 返回
    /// 执行结果
    pub async fn execute_method(
        &self,
        class: &str,
        method: &str,
        args: &[String],
    ) -> Result<String, String> {
        tracing::info!("执行 PHP 方法: {}::{}({:?})", class, method, args);

        // 构建 PHP 调用代码
        let args_str = args
            .iter()
            .map(|a| format!("'{}'", a.replace('\'', "\\'")))
            .collect::<Vec<_>>()
            .join(", ");

        let php_code = format!(
            r#"<?php
require_once '{}/{}';

$handler = new {}();
$result = $handler->{}({});
echo json_encode($result);
"#,
            self.project_path, self.entry_point, class, method, args_str
        );

        // 执行 PHP 代码
        self.execute_php_code(&php_code).await
    }

    /// 执行 PHP 闭包
    ///
    /// # 参数
    /// - `code`: 闭包代码
    ///
    /// # 返回
    /// 执行结果
    pub async fn execute_closure(&self, code: &str) -> Result<String, String> {
        tracing::debug!("执行 PHP 闭包");

        let php_code = format!(
            r#"<?php
require_once '{}/{}';

{}
"#,
            self.project_path, self.entry_point, code
        );

        self.execute_php_code(&php_code).await
    }

    /// 执行 PHP 代码
    ///
    /// # 参数
    /// - `code`: PHP 代码
    ///
    /// # 返回
    /// 执行结果
    async fn execute_php_code(&self, code: &str) -> Result<String, String> {
        // 在实际实现中，这里应该调用 OYTAPHP 解释器执行代码
        // 目前使用模拟实现
        tracing::debug!("执行 PHP 代码: {} 字节", code.len());

        // 模拟执行
        Ok(r#"{"success": true, "message": "Task executed"}"#.to_string())
    }
}

impl TimerManager {
    /// 创建新的定时器管理器
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            interpreter: None,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// 设置 PHP 解释器桥接器
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
        let task_id = format!("timer_{}", uuid::Uuid::new_v4());

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
        let task_id = format!("cron_{}", uuid::Uuid::new_v4());

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
        let task_id = format!("once_{}", uuid::Uuid::new_v4());

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

        self.add_task(task).await
    }

    /// 启动所有定时任务
    pub async fn start_all(&self) {
        *self.running.write().await = true;

        let tasks = self.tasks.read().await;
        let task_ids: Vec<String> = tasks.keys().cloned().collect();
        drop(tasks);

        for task_id in task_ids {
            self.start_task(&task_id).await;
        }

        tracing::info!("定时器管理器已启动");
    }

    /// 启动单个任务
    async fn start_task(&self, task_id: &str) {
        let tasks = self.tasks.read().await;
        let task = tasks.get(task_id).cloned();
        drop(tasks);

        if let Some(task) = task {
            if !task.enabled {
                return;
            }

            let task_id = task.id.clone();
            let task_type = task.task_type.clone();
            let handler = task.handler.clone();
            let max_runs = task.max_runs;
            let tasks_ref = self.tasks.clone();
            let history_ref = self.history.clone();
            let interpreter_ref = self.interpreter.clone();

            tokio::spawn(async move {
                match task_type {
                    TaskType::Interval { secs } => {
                        let mut timer = interval(Duration::from_secs(secs));

                        loop {
                            timer.tick().await;

                            // 检查是否达到最大执行次数
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
                            timer.tick().await;

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
        *self.running.write().await = false;

        let mut tasks = self.tasks.write().await;
        for task in tasks.values_mut() {
            task.enabled = false;
            task.status = TaskStatus::Paused;
        }

        tracing::info!("所有定时任务已停止");
    }

    /// 暂停任务
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
    pub async fn remove_task(&self, task_id: &str) -> bool {
        let mut tasks = self.tasks.write().await;

        if tasks.remove(task_id).is_some() {
            tracing::info!("任务已删除: {}", task_id);
            return true;
        }

        false
    }

    /// 获取任务状态
    pub async fn get_task(&self, task_id: &str) -> Option<TimerTask> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// 获取所有任务
    pub async fn get_all_tasks(&self) -> Vec<TimerTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// 获取任务数量
    pub async fn task_count(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks.len()
    }

    /// 获取执行历史
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

/// 执行任务处理器
async fn execute_handler(
    handler: &TaskHandler,
    interpreter: &Option<Arc<PhpInterpreterBridge>>,
) -> TaskResult {
    let start = std::time::Instant::now();
    let mut result = TaskResult {
        task_id: String::new(),
        success: false,
        duration_ms: 0,
        output: String::new(),
        error: None,
    };

    match handler {
        TaskHandler::PhpClass { class, method, args } => {
            if let Some(interp) = interpreter {
                match interp.execute_method(class, method, args).await {
                    Ok(output) => {
                        result.success = true;
                        result.output = output;
                    }
                    Err(e) => {
                        result.error = Some(e);
                    }
                }
            } else {
                result.error = Some("PHP 解释器未配置".to_string());
            }
        }
        TaskHandler::PhpClosure { code } => {
            if let Some(interp) = interpreter {
                match interp.execute_closure(code).await {
                    Ok(output) => {
                        result.success = true;
                        result.output = output;
                    }
                    Err(e) => {
                        result.error = Some(e);
                    }
                }
            } else {
                result.error = Some("PHP 解释器未配置".to_string());
            }
        }
        TaskHandler::Native { name } => {
            // 执行原生函数
            tracing::info!("执行原生函数: {}", name);
            result.success = true;
            result.output = format!("Native function {} executed", name);
        }
        TaskHandler::Shell { command } => {
            // 执行 Shell 命令
            tracing::info!("执行 Shell 命令: {}", command);

            #[cfg(target_os = "linux")]
            {
                use std::process::Command;

                let output = Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .output();

                match output {
                    Ok(output) => {
                        result.success = output.status.success();
                        result.output = String::from_utf8_lossy(&output.stdout).to_string();
                        if !output.status.success() {
                            result.error = Some(String::from_utf8_lossy(&output.stderr).to_string());
                        }
                    }
                    Err(e) => {
                        result.error = Some(e.to_string());
                    }
                }
            }

            #[cfg(not(target_os = "linux"))]
            {
                result.error = Some("Shell 命令仅在 Linux 系统上支持".to_string());
            }
        }
    }

    result.duration_ms = start.elapsed().as_millis() as u64;
    result
}

/// 更新任务状态
async fn update_task_status(
    tasks: &Arc<RwLock<HashMap<String, TimerTask>>>,
    task_id: &str,
    result: &TaskResult,
) {
    let mut tasks = tasks.write().await;

    if let Some(task) = tasks.get_mut(task_id) {
        task.last_run = Some(chrono::Local::now().to_rfc3339());
        task.run_count += 1;
        task.status = if result.success {
            TaskStatus::Running
        } else {
            TaskStatus::Failed
        };
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

    #[tokio::test]
    async fn test_timer_manager_new() {
        let manager = TimerManager::new();
        assert_eq!(manager.task_count().await, 0);
    }

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

        assert!(manager.pause_task(&task_id).await);

        let task = manager.get_task(&task_id).await.unwrap();
        assert_eq!(task.status, TaskStatus::Paused);

        assert!(manager.resume_task(&task_id).await);

        let task = manager.get_task(&task_id).await.unwrap();
        assert_eq!(task.status, TaskStatus::Pending);
    }

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

    #[test]
    fn test_php_interpreter_bridge() {
        let bridge = PhpInterpreterBridge::new("/app", "vendor/autoload.php");
        assert_eq!(bridge.project_path, "/app");
        assert_eq!(bridge.entry_point, "vendor/autoload.php");
    }
}
