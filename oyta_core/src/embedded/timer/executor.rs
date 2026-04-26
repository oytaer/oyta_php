//! 任务执行器模块
//!
//! 包含任务执行和状态更新的核心逻辑

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

use super::types::{TaskHandler, TaskResult, TaskStatus, TimerTask};
use super::bridge::PhpInterpreterBridge;

/// 执行任务处理器
///
/// 根据处理器类型执行对应的任务
///
/// # 参数
/// - `handler`: 任务处理器
/// - `interpreter`: PHP 解释器桥接器
///
/// # 返回值
/// 任务执行结果
pub async fn execute_handler(
    handler: &TaskHandler,
    interpreter: &Option<Arc<PhpInterpreterBridge>>,
) -> TaskResult {
    // 记录开始时间
    let start = std::time::Instant::now();

    // 初始化结果
    let mut result = TaskResult {
        task_id: String::new(),
        success: false,
        duration_ms: 0,
        output: String::new(),
        error: None,
    };

    // 根据处理器类型执行
    match handler {
        // PHP 类方法处理器
        TaskHandler::PhpClass { class, method, args } => {
            if let Some(interp) = interpreter {
                // 执行 PHP 方法
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
        // PHP 闭包处理器
        TaskHandler::PhpClosure { code } => {
            if let Some(interp) = interpreter {
                // 执行 PHP 闭包
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
        // 原生函数处理器
        TaskHandler::Native { name } => {
            // 执行原生函数
            tracing::info!("执行原生函数: {}", name);
            result.success = true;
            result.output = format!("Native function {} executed", name);
        }
        // Shell 命令处理器
        TaskHandler::Shell { command } => {
            // 执行 Shell 命令
            tracing::info!("执行 Shell 命令: {}", command);

            // Linux 系统支持
            #[cfg(target_os = "linux")]
            {
                use std::process::Command;

                // 执行命令
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .output();

                // 处理执行结果
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

            // 非 Linux 系统不支持
            #[cfg(not(target_os = "linux"))]
            {
                result.error = Some("Shell 命令仅在 Linux 系统上支持".to_string());
            }
        }
    }

    // 计算执行时间
    result.duration_ms = start.elapsed().as_millis() as u64;
    result
}

/// 更新任务状态
///
/// 根据执行结果更新任务的状态信息
///
/// # 参数
/// - `tasks`: 任务列表引用
/// - `task_id`: 任务ID
/// - `result`: 执行结果
pub async fn update_task_status(
    tasks: &Arc<RwLock<HashMap<String, TimerTask>>>,
    task_id: &str,
    result: &TaskResult,
) {
    // 获取写锁
    let mut tasks = tasks.write().await;

    // 更新任务状态
    if let Some(task) = tasks.get_mut(task_id) {
        // 记录最后执行时间
        task.last_run = Some(chrono::Local::now().to_rfc3339());
        // 增加执行次数
        task.run_count += 1;
        // 根据执行结果设置状态
        task.status = if result.success {
            TaskStatus::Running
        } else {
            TaskStatus::Failed
        };
    }
}
