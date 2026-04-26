//! 构建命令模块
//!
//! 包含应用目录构建相关命令

use anyhow::Result;

use crate::env_loader;
use crate::project;

/// 处理 build 命令：自动生成应用目录和文件
/// 在 app/ 目录下创建指定名称的应用子目录
/// 包含 controller/、model/、middleware/ 等子目录
///
/// # 参数
/// - `name`: 应用名称
pub async fn handle_build(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建应用目录结构
    let app_dir = project.app_dir.join(name);

    // 需要创建的子目录列表
    let subdirs = ["controller", "model", "middleware"];

    // 逐个创建子目录
    for subdir in subdirs {
        let dir_path = app_dir.join(subdir);
        std::fs::create_dir_all(&dir_path)?;
        tracing::info!("创建目录: {}", dir_path.display());
    }

    // 创建应用公共函数文件（可选）
    let common_file = app_dir.join("common.php");
    if !common_file.exists() {
        // 生成公共函数文件内容
        let content = format!("<?php\n// {} 应用公共函数文件\n", name);
        std::fs::write(&common_file, content)?;
        tracing::info!("创建文件: {}", common_file.display());
    }

    tracing::info!("✓ 应用 [{}] 构建完成", name);
    Ok(())
}
