//! 优化命令模块
//!
//! 包含应用优化相关命令的实现

use anyhow::Result;

use crate::env_loader;
use crate::project;
use super::super::optimizer;

/// 处理 optimize 命令：优化应用
///
/// 执行所有优化操作：路由缓存、配置缓存、类映射优化
pub async fn handle_optimize() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建优化器并执行所有优化
    let optimizer = optimizer::Optimizer::new(project);
    optimizer.optimize_all().await
}

/// 处理 optimize:route 命令：生成路由缓存
pub async fn handle_optimize_route() -> Result<()> {
    // 路由缓存与 route:cache 命令相同
    super::route::handle_route_cache().await
}

/// 处理 optimize:schema 命令：生成数据表字段缓存
///
/// 扫描数据库表结构，生成字段映射缓存
pub async fn handle_optimize_schema() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建优化器并执行字段缓存
    let optimizer = optimizer::Optimizer::new(project);
    optimizer.cache_schema().await
}

/// 处理 clear 命令：清除运行时缓存
/// 删除 runtime/ 目录下的所有缓存文件
pub async fn handle_clear() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建优化器并清除缓存
    let optimizer = optimizer::Optimizer::new(project);
    optimizer.clear_cache().await
}

/// 处理 config:cache 命令：缓存配置
///
/// 将所有配置文件合并为一个缓存文件，提高配置读取性能
pub async fn handle_config_cache() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建优化器并执行配置缓存
    let optimizer = optimizer::Optimizer::new(project);
    optimizer.cache_config().await
}
