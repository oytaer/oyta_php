//! Composer 相关命令模块
//!
//! 提供 Composer 依赖管理功能

use anyhow::Result;

use crate::env_loader;
use crate::project;

/// 处理 composer:install 命令
///
/// 安装依赖
///
/// # 参数
/// - `no_dev`: 不安装开发依赖
/// - `optimize`: 优化自动加载
pub async fn handle_composer_install(no_dev: bool, optimize: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    println!("正在安装 Composer 依赖...");

    // 构建 composer 命令
    let mut args = vec!["install"];
    
    if no_dev {
        args.push("--no-dev");
    }
    
    if optimize {
        args.push("--optimize-autoloader");
    }

    // 执行 composer 命令
    let status = std::process::Command::new("composer")
        .args(&args)
        .current_dir(&project.root)
        .status()?;

    if status.success() {
        println!("✓ Composer 依赖安装完成");
    } else {
        println!("⚠ Composer 依赖安装失败");
    }

    Ok(())
}

/// 处理 composer:update 命令
///
/// 更新依赖
///
/// # 参数
/// - `packages`: 要更新的包列表
/// - `no_dev`: 不更新开发依赖
pub async fn handle_composer_update(packages: &[String], no_dev: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    println!("正在更新 Composer 依赖...");

    // 构建 composer 命令
    let mut args = vec!["update"];
    
    for pkg in packages {
        args.push(pkg);
    }
    
    if no_dev {
        args.push("--no-dev");
    }

    // 执行 composer 命令
    let status = std::process::Command::new("composer")
        .args(&args)
        .current_dir(&project.root)
        .status()?;

    if status.success() {
        println!("✓ Composer 依赖更新完成");
    } else {
        println!("⚠ Composer 依赖更新失败");
    }

    Ok(())
}

/// 处理 composer:require 命令
///
/// 添加依赖
///
/// # 参数
/// - `packages`: 要添加的包列表
/// - `dev`: 添加为开发依赖
pub async fn handle_composer_require(packages: &[String], dev: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    println!("正在添加 Composer 依赖...");

    // 构建 composer 命令
    let mut args = vec!["require"];
    
    for pkg in packages {
        args.push(pkg);
    }
    
    if dev {
        args.push("--dev");
    }

    // 执行 composer 命令
    let status = std::process::Command::new("composer")
        .args(&args)
        .current_dir(&project.root)
        .status()?;

    if status.success() {
        println!("✓ Composer 依赖添加完成");
    } else {
        println!("⚠ Composer 依赖添加失败");
    }

    Ok(())
}

/// 处理 composer:remove 命令
///
/// 移除依赖
///
/// # 参数
/// - `packages`: 要移除的包列表
pub async fn handle_composer_remove(packages: &[String]) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    println!("正在移除 Composer 依赖...");

    // 构建 composer 命令
    let mut args = vec!["remove"];
    
    for pkg in packages {
        args.push(pkg);
    }

    // 执行 composer 命令
    let status = std::process::Command::new("composer")
        .args(&args)
        .current_dir(&project.root)
        .status()?;

    if status.success() {
        println!("✓ Composer 依赖移除完成");
    } else {
        println!("⚠ Composer 依赖移除失败");
    }

    Ok(())
}

/// 处理 composer:dump-autoload 命令
///
/// 重新生成自动加载
///
/// # 参数
/// - `optimize`: 优化自动加载
pub async fn handle_composer_dump_autoload(optimize: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    println!("正在重新生成自动加载...");

    // 构建 composer 命令
    let mut args = vec!["dump-autoload"];
    
    if optimize {
        args.push("--optimize");
    }

    // 执行 composer 命令
    let status = std::process::Command::new("composer")
        .args(&args)
        .current_dir(&project.root)
        .status()?;

    if status.success() {
        println!("✓ 自动加载重新生成完成");
    } else {
        println!("⚠ 自动加载重新生成失败");
    }

    Ok(())
}

/// 处理 composer:show 命令
///
/// 查看已安装的包
///
/// # 参数
/// - `package`: 指定包名
/// - `direct`: 只显示直接依赖
pub async fn handle_composer_show(package: &Option<String>, direct: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 构建 composer 命令
    let mut args = vec!["show"];
    
    if let Some(pkg) = package {
        args.push(pkg);
    }
    
    if direct {
        args.push("--direct");
    }

    // 执行 composer 命令
    let status = std::process::Command::new("composer")
        .args(&args)
        .current_dir(&project.root)
        .status()?;

    Ok(())
}

/// 处理 composer:outdated 命令
///
/// 查看过时的包
pub async fn handle_composer_outdated() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 执行 composer 命令
    let status = std::process::Command::new("composer")
        .args(&["outdated"])
        .current_dir(&project.root)
        .status()?;

    Ok(())
}

/// 处理 composer:validate 命令
///
/// 验证 composer.json
pub async fn handle_composer_validate() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 执行 composer 命令
    let status = std::process::Command::new("composer")
        .args(&["validate"])
        .current_dir(&project.root)
        .status()?;

    if status.success() {
        println!("✓ composer.json 格式正确");
    } else {
        println!("⚠ composer.json 格式有误");
    }

    Ok(())
}

/// 处理 composer:config 命令
///
/// 配置 Composer
///
/// # 参数
/// - `args`: 配置参数
/// - `list`: 列出所有配置
pub async fn handle_composer_config(args: &[String], list: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 构建 composer 命令
    let mut cmd_args = vec!["config"];
    
    for arg in args {
        cmd_args.push(arg);
    }
    
    if list {
        cmd_args.push("--list");
    }

    // 执行 composer 命令
    let status = std::process::Command::new("composer")
        .args(&cmd_args)
        .current_dir(&project.root)
        .status()?;

    Ok(())
}

/// 处理 composer:clear-cache 命令
///
/// 清除 Composer 缓存
pub async fn handle_composer_clear_cache() -> Result<()> {
    println!("正在清除 Composer 缓存...");

    // 执行 composer 命令
    let status = std::process::Command::new("composer")
        .args(&["clear-cache"])
        .status()?;

    if status.success() {
        println!("✓ Composer 缓存已清除");
    } else {
        println!("⚠ Composer 缓存清除失败");
    }

    Ok(())
}

/// 处理 composer:diagnose 命令
///
/// 诊断 Composer 问题
pub async fn handle_composer_diagnose() -> Result<()> {
    println!("正在诊断 Composer...");

    // 执行 composer 命令
    let status = std::process::Command::new("composer")
        .args(&["diagnose"])
        .status()?;

    Ok(())
}
