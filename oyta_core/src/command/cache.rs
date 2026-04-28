//! 缓存管理命令模块
//!
//! 提供缓存清除、删除指定缓存、查看缓存状态等功能

use anyhow::Result;

use crate::cache;
use crate::env_loader;
use crate::project;

/// 处理 cache:clear 命令
///
/// 清除所有缓存或指定标签的缓存
///
/// # 参数
/// - `tag`: 可选的缓存标签
pub async fn handle_cache_clear(tag: &Option<String>) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("正在清除缓存...");

    // 清除缓存
    if let Some(tag_name) = tag {
        // 清除指定标签的缓存（目前不支持标签，清除所有）
        cache::facade::Cache::clear()?;
        println!("✓ 已清除标签 '{}' 的缓存", tag_name);
    } else {
        // 清除所有缓存
        cache::facade::Cache::clear()?;
        println!("✓ 已清除所有缓存");
    }

    Ok(())
}

/// 处理 cache:forget 命令
///
/// 删除指定的缓存键
///
/// # 参数
/// - `key`: 缓存键名
pub async fn handle_cache_forget(key: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("正在删除缓存键: {}", key);

    // 删除缓存
    if cache::facade::Cache::has(key) {
        cache::facade::Cache::delete(key)?;
        println!("✓ 已删除缓存键: {}", key);
    } else {
        println!("⚠ 缓存键 '{}' 不存在", key);
    }

    Ok(())
}

/// 处理 cache:status 命令
///
/// 查看缓存状态信息
pub async fn handle_cache_status() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  缓存状态");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 显示缓存目录
    let cache_dir = project.runtime_dir.join("cache");
    println!("  驱动: file");
    println!("  缓存目录: {}", cache_dir.display());

    // 统计缓存文件数量
    let mut total_files = 0;
    let mut total_size: u64 = 0;

    if cache_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&cache_dir) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    total_files += 1;
                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }
                }
            }
        }
    }

    println!("  总文件数: {}", total_files);
    println!("  总大小: {}", format_bytes(total_size));

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

/// 格式化字节数为可读格式
fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
