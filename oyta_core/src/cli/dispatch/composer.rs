//! Composer 命令模块
//!
//! 包含所有 Composer 相关命令的实现
//! 提供依赖管理、版本检查、配置管理等功能

use anyhow::Result;

use crate::env_loader;
use crate::project;

use super::helpers::parse_package_spec;
use super::types::OutdatedPackageInfo;
use super::version_utils::{compare_versions, fetch_latest_version, normalize_version};

/// 处理 composer install 命令
///
/// 根据 composer.json 或 composer.lock 安装所有依赖包
///
/// # 参数
/// - `no_dev`: 是否跳过开发依赖
/// - `optimize`: 是否优化自动加载
pub async fn handle_composer_install(no_dev: bool, optimize: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("开始安装 Composer 依赖...");

    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());

    // 执行安装
    composer.install().await?;

    // 如果需要优化自动加载
    if optimize {
        tracing::info!("优化自动加载...");
        composer.dump_autoload().await?;
    }

    // 如果跳过开发依赖，输出提示
    if no_dev {
        tracing::info!("已跳过开发依赖 (require-dev)");
    }

    tracing::info!("✓ Composer 依赖安装完成");
    Ok(())
}

/// 处理 composer update 命令
///
/// 更新指定的包到最新版本，如果包列表为空则更新所有依赖
///
/// # 参数
/// - `packages`: 要更新的包列表
/// - `no_dev`: 是否跳过开发依赖
pub async fn handle_composer_update(packages: &[String], no_dev: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("开始更新 Composer 依赖...");

    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());

    // 执行更新
    composer.update(packages).await?;

    // 如果跳过开发依赖，输出提示
    if no_dev {
        tracing::info!("已跳过开发依赖 (require-dev)");
    }

    tracing::info!("✓ Composer 依赖更新完成");
    Ok(())
}

/// 处理 composer require 命令
///
/// 添加新的依赖包到 composer.json 并安装
///
/// # 参数
/// - `packages`: 包列表，格式为 "vendor/package" 或 "vendor/package:^1.0"
/// - `dev`: 是否添加为开发依赖
pub async fn handle_composer_require(packages: &[String], dev: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 检查包列表是否为空
    if packages.is_empty() {
        anyhow::bail!("请指定要添加的包，例如: oyta composer:require vendor/package");
    }

    tracing::info!("开始添加 Composer 依赖...");

    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());

    // 解析并添加每个包
    for package_spec in packages {
        // 解析包名和版本约束
        let (package_name, version) = parse_package_spec(package_spec);

        // 添加依赖
        composer.add_require(&package_name, &version).await?;

        // 如果是开发依赖，输出提示
        if dev {
            tracing::info!("已添加为开发依赖: {}", package_name);
        }
    }

    tracing::info!("✓ Composer 依赖添加完成");
    Ok(())
}

/// 处理 composer remove 命令
///
/// 从 composer.json 中移除依赖包并删除相关文件
///
/// # 参数
/// - `packages`: 要移除的包名列表
pub async fn handle_composer_remove(packages: &[String]) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 检查包列表是否为空
    if packages.is_empty() {
        anyhow::bail!("请指定要移除的包，例如: oyta composer:remove vendor/package");
    }

    tracing::info!("开始移除 Composer 依赖...");

    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());

    // 移除每个包
    for package_name in packages {
        composer.remove_require(package_name).await?;
    }

    tracing::info!("✓ Composer 依赖移除完成");
    Ok(())
}

/// 处理 composer dump-autoload 命令
///
/// 重新生成自动加载文件
///
/// # 参数
/// - `optimize`: 是否优化自动加载（生成完整 classmap）
pub async fn handle_composer_dump_autoload(optimize: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("重新生成自动加载文件...");

    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());

    // 加载配置
    composer.load().await?;

    // 重新生成自动加载
    composer.dump_autoload().await?;

    // 如果需要优化
    if optimize {
        tracing::info!("已优化自动加载（生成完整 classmap）");
    }

    tracing::info!("✓ 自动加载文件已重新生成");
    Ok(())
}

/// 处理 composer show 命令
///
/// 显示已安装的包信息
///
/// # 参数
/// - `package`: 指定包名，不指定则显示所有包
/// - `direct`: 是否只显示直接依赖
pub async fn handle_composer_show(package: &Option<String>, direct: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("查看已安装的包...");

    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());

    // 加载配置
    composer.load().await?;

    // 如果指定了包名，显示详细信息
    if let Some(pkg_name) = package {
        if let Some(version) = composer.get_package_version(pkg_name) {
            println!("包名: {}", pkg_name);
            println!("版本: {}", version);

            // 获取配置中的包信息
            if let Some(config) = composer.config() {
                if let Some(version_constraint) = config.require.get(pkg_name) {
                    println!("版本约束: {}", version_constraint);
                }
            }
        } else {
            tracing::warn!("包 {} 未安装", pkg_name);
        }
    } else {
        // 显示所有已安装的包
        let installed = composer.get_installed_packages();

        // 如果没有安装的包，输出提示并返回
        if installed.is_empty() {
            println!("没有已安装的包");
            return Ok(());
        }

        println!("已安装的包 (共 {} 个):", installed.len());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // 遍历并输出每个包
        for pkg_name in &installed {
            if let Some(version) = composer.get_package_version(pkg_name) {
                println!("  {} {}", pkg_name, version);
            }
        }

        // 如果只显示直接依赖
        if direct {
            println!("\n直接依赖:");
            if let Some(config) = composer.config() {
                for (name, version) in &config.require {
                    println!("  {} {}", name, version);
                }
            }
        }
    }

    Ok(())
}

/// 处理 composer outdated 命令
///
/// 检查已安装的包是否有新版本可用
pub async fn handle_composer_outdated() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("检查过时的包...");

    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());

    // 加载配置
    composer.load().await?;

    // 获取已安装的包
    let installed = composer.get_installed_packages();

    // 如果没有安装的包，输出提示并返回
    if installed.is_empty() {
        println!("没有已安装的包");
        return Ok(());
    }

    // 创建 Packagist API 客户端
    let mut packagist = crate::composer::packagist::PackagistClient::new();

    println!("检查包更新状态（连接 Packagist API）...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "{:<40} {:<15} {:<15} {:<10}",
        "包名", "当前版本", "最新版本", "状态"
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 统计信息
    let mut outdated_count = 0;
    let mut up_to_date_count = 0;
    let mut error_count = 0;
    let mut outdated_packages: Vec<OutdatedPackageInfo> = Vec::new();

    // 遍历每个已安装的包，检查更新
    for pkg_name in &installed {
        // 获取当前安装的版本
        let current_version = match composer.get_package_version(pkg_name) {
            Some(v) => v,
            None => {
                println!(
                    "{:<40} {:<15} {:<15} {:<10}",
                    pkg_name, "未知", "-", "⚠ 无法获取"
                );
                error_count += 1;
                continue;
            }
        };

        // 从 Packagist API 获取最新版本信息
        match fetch_latest_version(&mut packagist, pkg_name).await {
            Ok(latest_info) => {
                let latest_version = latest_info.latest_version;
                let is_outdated = latest_info.is_outdated;

                // 规范化版本号用于比较
                let current_normalized = normalize_version(&current_version);
                let latest_normalized = normalize_version(&latest_version);

                // 比较版本
                let needs_update = is_outdated || compare_versions(&latest_normalized, &current_normalized) > 0;

                if needs_update {
                    let status = "🔴 需更新";
                    outdated_count += 1;

                    // 记录过时包的详细信息
                    outdated_packages.push(OutdatedPackageInfo {
                        name: pkg_name.clone(),
                        current_version: current_version.clone(),
                        latest_version: latest_version.clone(),
                        description: latest_info.description,
                    });

                    println!(
                        "{:<40} {:<15} {:<15} {:<10}",
                        pkg_name, current_version, latest_version, status
                    );
                } else {
                    let status = "✓ 最新";
                    up_to_date_count += 1;

                    println!(
                        "{:<40} {:<15} {:<15} {:<10}",
                        pkg_name, current_version, latest_version, status
                    );
                }
            }
            Err(e) => {
                // API 请求失败，显示当前版本并标记错误
                println!(
                    "{:<40} {:<15} {:<15} {:<10}",
                    pkg_name, current_version, "-", "⚠ API错误"
                );
                tracing::debug!("获取 {} 版本信息失败: {}", pkg_name, e);
                error_count += 1;
            }
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 显示统计摘要
    println!();
    println!("统计摘要:");
    println!("  总包数: {}", installed.len());
    println!("  ✓ 最新: {} 个", up_to_date_count);
    println!("  🔴 需更新: {} 个", outdated_count);
    if error_count > 0 {
        println!("  ⚠ 检查失败: {} 个", error_count);
    }

    // 如果有过时的包，显示更新建议
    if !outdated_packages.is_empty() {
        println!();
        println!("更新建议:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for pkg in &outdated_packages {
            println!();
            println!(
                "📦 {} ({} → {})",
                pkg.name, pkg.current_version, pkg.latest_version
            );
            if !pkg.description.is_empty() {
                println!("   描述: {}", pkg.description);
            }
            println!(
                "   更新命令: oyta composer require {}:{}",
                pkg.name, pkg.latest_version
            );
        }

        println!();
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("提示: 使用 'oyta composer update' 更新所有包");
        println!("      使用 'oyta composer require <package>:<version>' 更新指定包");
    } else if outdated_count == 0 && error_count == 0 {
        println!();
        println!("✓ 所有包都是最新版本！");
    }

    Ok(())
}

/// 处理 composer validate 命令
///
/// 验证 composer.json 格式是否正确
pub async fn handle_composer_validate() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("验证 composer.json...");

    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());

    // 加载配置
    composer.load().await?;

    // 执行验证
    let warnings = composer.validate()?;

    // 根据验证结果输出
    if warnings.is_empty() {
        println!("✓ composer.json 格式正确");
    } else {
        println!("composer.json 验证结果:");
        for warning in &warnings {
            println!("  ⚠ {}", warning);
        }
    }

    Ok(())
}

/// 处理 composer config 命令
///
/// 读取或修改 composer.json 中的配置项
///
/// # 参数
/// - `args`: 配置参数
/// - `list`: 是否列出所有配置项
pub async fn handle_composer_config(args: &[String], list: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("Composer 配置管理...");

    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());

    // 加载配置
    composer.load().await?;

    // 如果列出所有配置项
    if list {
        if let Some(config) = composer.config() {
            println!("composer.json 配置项:");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("名称: {}", config.name);
            println!("描述: {}", config.description);
            println!("类型: {}", config.r#type);
            println!("许可证: {:?}", config.license);
            println!("最小稳定性: {}", config.minimum_stability);
            println!("优先稳定版本: {}", config.prefer_stable);

            // 输出依赖列表
            if !config.require.is_empty() {
                println!("\n依赖 (require):");
                for (name, version) in &config.require {
                    println!("  {} = {}", name, version);
                }
            }

            // 输出开发依赖列表
            if !config.require_dev.is_empty() {
                println!("\n开发依赖 (require-dev):");
                for (name, version) in &config.require_dev {
                    println!("  {} = {}", name, version);
                }
            }

            // 输出 PSR-4 自动加载配置
            if !config.autoload.psr_4.is_empty() {
                println!("\nPSR-4 自动加载:");
                for (namespace, paths) in &config.autoload.psr_4 {
                    println!("  {} => {:?}", namespace, paths);
                }
            }

            // 输出仓库配置
            if !config.repositories.is_empty() {
                println!("\n仓库配置:");
                for repo in &config.repositories {
                    println!("  {} ({})", repo.url, repo.r#type);
                }
            }
        }
        return Ok(());
    }

    // 处理配置读取/设置
    match args.len() {
        0 => {
            anyhow::bail!("请指定配置项名称，或使用 --list 查看所有配置");
        }
        1 => {
            // 读取配置值
            let key = &args[0];
            if let Some(config) = composer.config() {
                let value = get_config_value(config, key);
                if let Some(v) = value {
                    println!("{} = {}", key, v);
                } else {
                    println!("配置项 {} 不存在", key);
                }
            }
        }
        _ => {
            // 设置配置值
            tracing::info!("设置配置项需要修改 composer.json，请手动编辑文件");
            println!("提示: 请直接编辑 composer.json 文件来修改配置项");
        }
    }

    Ok(())
}

/// 处理 composer clear-cache 命令
///
/// 清除本地缓存的包元数据和下载文件
pub async fn handle_composer_clear_cache() -> Result<()> {
    tracing::info!("清除 Composer 缓存...");

    // 获取缓存目录
    let cache_dir = std::env::var("COMPOSER_CACHE_DIR").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        format!("{}/.composer/cache", home)
    });

    let cache_path = std::path::Path::new(&cache_dir);

    // 如果缓存目录存在，删除它
    if cache_path.exists() {
        std::fs::remove_dir_all(cache_path)?;
        println!("✓ 已清除 Composer 缓存: {}", cache_dir);
    } else {
        println!("缓存目录不存在: {}", cache_dir);
    }

    Ok(())
}

/// 处理 composer diagnose 命令
///
/// 检查 Composer 运行环境是否正常
pub async fn handle_composer_diagnose() -> Result<()> {
    tracing::info!("Composer 环境诊断...");

    println!("Composer 环境诊断结果:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 检查 composer.json 是否存在
    let project = project::detector::Project::detect_from_cwd();
    match project {
        Ok(p) => {
            // 检查 composer.json
            let json_path = p.root.join("composer.json");
            if json_path.exists() {
                println!("✓ composer.json 存在: {}", json_path.display());
            } else {
                println!("⚠ composer.json 不存在");
            }

            // 检查 composer.lock
            let lock_path = p.root.join("composer.lock");
            if lock_path.exists() {
                println!("✓ composer.lock 存在: {}", lock_path.display());
            } else {
                println!("⚠ composer.lock 不存在");
            }

            // 检查 vendor 目录
            let vendor_path = p.root.join("vendor");
            if vendor_path.exists() {
                println!("✓ vendor 目录存在: {}", vendor_path.display());
            } else {
                println!("⚠ vendor 目录不存在，请运行 composer install");
            }
        }
        Err(e) => {
            println!("⚠ 无法检测项目目录: {}", e);
        }
    }

    // 检查网络连接
    println!("\n网络检测:");
    println!("  Packagist: https://repo.packagist.org");
    println!("  中国镜像: https://mirrors.aliyun.com/composer/");

    // 检查环境变量
    println!("\n环境变量:");
    if let Ok(cache_dir) = std::env::var("COMPOSER_CACHE_DIR") {
        println!("  COMPOSER_CACHE_DIR: {}", cache_dir);
    } else {
        println!("  COMPOSER_CACHE_DIR: 未设置 (使用默认值)");
    }

    if let Ok(home) = std::env::var("HOME") {
        println!("  HOME: {}", home);
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✓ 诊断完成");

    Ok(())
}

/// 获取配置值
///
/// 根据键名从 ComposerJson 中获取配置值
///
/// # 参数
/// - `config`: ComposerJson 配置引用
/// - `key`: 配置键名
///
/// # 返回值
/// 配置值的字符串表示，如果不存在返回 None
fn get_config_value(config: &crate::composer::ComposerJson, key: &str) -> Option<String> {
    match key {
        "name" => Some(config.name.clone()),
        "description" => Some(config.description.clone()),
        "type" => Some(config.r#type.clone()),
        "license" => Some(format!("{:?}", config.license)),
        "minimum-stability" => Some(config.minimum_stability.clone()),
        "prefer-stable" => Some(config.prefer_stable.to_string()),
        _ => {
            // 尝试从 require 中获取
            if let Some(version) = config.require.get(key) {
                return Some(version.clone());
            }
            // 尝试从 require_dev 中获取
            if let Some(version) = config.require_dev.get(key) {
                return Some(version.clone());
            }
            None
        }
    }
}
