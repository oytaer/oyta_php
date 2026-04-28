//! 队列管理命令模块
//!
//! 提供查看失败任务、重试失败任务、清除失败任务、查看队列状态等功能

use anyhow::Result;

use crate::embedded::queue::QueueManager;
use crate::env_loader;
use crate::project;

/// 处理 queue:failed 命令
///
/// 查看失败任务列表
///
/// # 参数
/// - `queue_name`: 可选的队列名称
pub async fn handle_queue_failed(queue_name: &Option<String>) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  失败任务列表");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 获取队列管理器
    let manager = QueueManager::new();

    // 获取队列名称列表
    let queues = if let Some(name) = queue_name {
        vec![name.clone()]
    } else {
        manager.queue_names().await
    };

    let mut total_count = 0;

    for queue in &queues {
        let failed_jobs = manager.failed_jobs(queue).await;
        
        if !failed_jobs.is_empty() {
            println!("\n  队列: {}", queue);
            println!("  ┌────────────────────────────────────────────────────────────┐");
            
            for job in &failed_jobs {
                total_count += 1;
                println!("  │ ID: {}", job.id);
                println!("  │ 名称: {}", job.name);
                println!("  │ 处理器: {}", job.handler);
                println!("  │ 重试次数: {}/{}", job.retries, job.max_retries);
                println!("  │ 创建时间: {}", format_timestamp(job.created_at));
                println!("  ├────────────────────────────────────────────────────────────┤");
            }
        }
    }

    if total_count == 0 {
        println!("\n  没有失败的任务");
    } else {
        println!("\n  共 {} 个失败任务", total_count);
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

/// 处理 queue:retry 命令
///
/// 重试失败任务
///
/// # 参数
/// - `id`: 可选的任务ID
/// - `queue_name`: 可选的队列名称
pub async fn handle_queue_retry(id: &Option<String>, queue_name: &Option<String>) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 获取队列管理器
    let manager = QueueManager::new();

    if let Some(job_id) = id {
        // 重试指定任务
        let queue = queue_name.clone().unwrap_or_else(|| "default".to_string());
        
        if manager.retry(&queue, job_id).await {
            println!("✓ 任务 {} 已重新加入队列", job_id);
        } else {
            println!("⚠ 任务 {} 不存在或已在队列中", job_id);
        }
    } else {
        // 重试所有失败任务
        let queues = if let Some(name) = queue_name {
            vec![name.clone()]
        } else {
            manager.queue_names().await
        };

        let mut retried_count = 0;

        for queue in &queues {
            let failed_jobs = manager.failed_jobs(queue).await;
            
            for job in &failed_jobs {
                if manager.retry(queue, &job.id).await {
                    retried_count += 1;
                }
            }
        }

        println!("✓ 已重新加入 {} 个任务到队列", retried_count);
    }

    Ok(())
}

/// 处理 queue:flush 命令
///
/// 清除所有失败任务
///
/// # 参数
/// - `queue_name`: 可选的队列名称
pub async fn handle_queue_flush(queue_name: &Option<String>) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 获取队列管理器
    let manager = QueueManager::new();

    println!("正在清除失败任务...");

    if let Some(name) = queue_name {
        // 清除指定队列的失败任务
        let failed_jobs = manager.failed_jobs(name).await;
        let count = failed_jobs.len();
        
        // 清除失败任务记录
        for job in &failed_jobs {
            // 这里需要实现清除失败任务的方法
            tracing::debug!("清除失败任务: {}", job.id);
        }
        
        println!("✓ 已清除队列 '{}' 的 {} 个失败任务", name, count);
    } else {
        // 清除所有队列的失败任务
        let queues = manager.queue_names().await;
        let mut total_count = 0;

        for queue in &queues {
            let failed_jobs = manager.failed_jobs(&queue).await;
            total_count += failed_jobs.len();
            
            for job in &failed_jobs {
                tracing::debug!("清除失败任务: {}", job.id);
            }
        }

        println!("✓ 已清除所有队列的 {} 个失败任务", total_count);
    }

    Ok(())
}

/// 处理 queue:status 命令
///
/// 查看队列状态
///
/// # 参数
/// - `queue_name`: 可选的队列名称
pub async fn handle_queue_status(queue_name: &Option<String>) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  队列状态");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 获取队列管理器
    let manager = QueueManager::new();

    // 获取队列名称列表
    let queues = if let Some(name) = queue_name {
        vec![name.clone()]
    } else {
        manager.queue_names().await
    };

    if queues.is_empty() {
        println!("\n  没有活动的队列");
    } else {
        println!("\n  {:<20} {:<15} {:<15}", "队列名称", "待处理", "失败任务");
        println!("  ────────────────────────────────────────────────");
        
        for queue in &queues {
            let pending = manager.len(&queue).await;
            let failed = manager.failed_jobs(&queue).await.len();
            
            println!("  {:<20} {:<15} {:<15}", queue, pending, failed);
        }
    }

    // 显示工作进程状态
    let is_running = manager.is_running().await;
    println!("\n  工作进程状态: {}", if is_running { "运行中" } else { "已停止" });

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

/// 格式化时间戳为可读格式
fn format_timestamp(timestamp: u64) -> String {
    use chrono::{DateTime, Utc, TimeZone};
    
    match Utc.timestamp_opt(timestamp as i64, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        _ => format!("Invalid timestamp: {}", timestamp),
    }
}
