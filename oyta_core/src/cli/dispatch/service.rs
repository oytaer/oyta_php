//! 服务发现命令模块
//!
//! 包含服务发现和发布相关命令的实现

use anyhow::Result;

use crate::env_loader;
use crate::project;

/// 处理 service:discover 命令
///
/// 自动发现并注册扩展包的服务提供者
/// 扫描 vendor 目录下的所有包，查找 ServiceProvider 类
pub async fn handle_service_discover() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("开始服务发现...");

    // 获取 vendor 目录路径
    let vendor_dir = project.root.join("vendor");

    // 检查 vendor 目录是否存在
    if !vendor_dir.exists() {
        println!("vendor 目录不存在，请先运行 composer install");
        return Ok(());
    }

    // 扫描 vendor 目录下的所有包
    let mut services: Vec<ServiceProviderInfo> = Vec::new();

    // 遍历 vendor 目录
    if let Ok(entries) = std::fs::read_dir(&vendor_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 遍历供应商目录下的包目录
                if let Ok(package_entries) = std::fs::read_dir(&path) {
                    for package_entry in package_entries.flatten() {
                        let package_path = package_entry.path();
                        if package_path.is_dir() {
                            // 检查是否存在 ServiceProvider
                            let provider_path = package_path.join("src/ServiceProvider.php");
                            if provider_path.exists() {
                                let package_name = format!(
                                    "{}/{}",
                                    path.file_name().unwrap().to_string_lossy(),
                                    package_path.file_name().unwrap().to_string_lossy()
                                );

                                services.push(ServiceProviderInfo {
                                    name: "ServiceProvider".to_string(),
                                    package: package_name,
                                    path: provider_path,
                                    deferred: false,
                                    provides: vec![],
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // 如果没有发现服务，输出提示
    if services.is_empty() {
        println!("没有发现可注册的服务提供者");
        return Ok(());
    }

    // 输出发现的服务
    println!("发现的服务提供者 (共 {} 个):", services.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for service in &services {
        println!("  {} ({})", service.name, service.package);
    }

    // 生成服务注册文件
    let output_path = project.runtime_dir.join("services.php");
    generate_service_file(&services, &output_path)?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✓ 服务发现完成，已生成: {}", output_path.display());

    Ok(())
}

/// 处理 vendor:publish 命令
///
/// 发布扩展包的配置文件或资源文件到应用目录
///
/// # 参数
/// - `package`: 指定要发布的包名，不指定则发布所有可发布的资源
/// - `force`: 是否强制覆盖已存在的文件
pub async fn handle_vendor_publish(package: &Option<String>, force: bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("开始发布扩展资源...");

    // 获取 vendor 目录路径
    let vendor_dir = project.root.join("vendor");

    // 检查 vendor 目录是否存在
    if !vendor_dir.exists() {
        println!("vendor 目录不存在，请先运行 composer install");
        return Ok(());
    }

    // 获取可发布的资源列表
    let publishable = get_publishable_resources(&vendor_dir, &project);

    // 如果没有可发布的资源，输出提示
    if publishable.is_empty() {
        println!("没有可发布的资源");
        return Ok(());
    }

    // 如果指定了包名，只发布该包的资源
    let to_publish = if let Some(pkg) = package {
        publishable
            .into_iter()
            .filter(|p| &p.package == pkg)
            .collect::<Vec<_>>()
    } else {
        publishable
    };

    // 如果过滤后没有资源，输出提示
    if to_publish.is_empty() {
        if let Some(pkg) = package {
            println!("包 {} 没有可发布的资源", pkg);
        } else {
            println!("没有可发布的资源");
        }
        return Ok(());
    }

    println!("可发布的资源 (共 {} 个):", to_publish.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 发布每个资源
    let mut published_count = 0;
    let mut skipped_count = 0;

    for item in &to_publish {
        let dest_path = project.root.join(&item.destination);

        // 检查目标文件是否存在
        if dest_path.exists() && !force {
            println!("  ⚠ 跳过 (已存在): {}", item.destination);
            skipped_count += 1;
            continue;
        }

        // 确保目标目录存在
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 复制文件
        std::fs::copy(&item.source, &dest_path)?;
        println!("  ✓ 已发布: {} → {}", item.source.display(), item.destination);
        published_count += 1;
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "✓ 发布完成: {} 个文件已发布, {} 个文件已跳过",
        published_count, skipped_count
    );

    Ok(())
}

/// 服务提供者信息
///
/// 用于 service:discover 命令发现的服务提供者信息
#[derive(Debug, Clone)]
struct ServiceProviderInfo {
    /// 服务提供者类名
    name: String,
    /// 所属包名
    package: String,
    /// 服务提供者文件路径
    path: std::path::PathBuf,
    /// 是否延迟加载
    deferred: bool,
    /// 提供的服务列表
    provides: Vec<String>,
}

/// 可发布资源项
///
/// 用于 vendor:publish 命令的可发布资源信息
#[derive(Debug, Clone)]
struct PublishableItem {
    /// 所属包名
    package: String,
    /// 源文件路径
    source: std::path::PathBuf,
    /// 目标相对路径
    destination: String,
}

/// 生成服务注册文件
///
/// 将发现的服务提供者写入 PHP 文件
///
/// # 参数
/// - `services`: 服务提供者列表
/// - `output_path`: 输出文件路径
fn generate_service_file(services: &[ServiceProviderInfo], output_path: &std::path::Path) -> Result<()> {
    // 确保目录存在
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 生成 PHP 文件内容
    let mut content = String::new();
    content.push_str("<?php\n");
    content.push_str("// 自动生成的服务提供者注册文件\n");
    content.push_str("// 由 oyta service:discover 命令生成\n\n");
    content.push_str("return [\n");

    for service in services {
        content.push_str(&format!(
            "    // {}\n",
            service.package
        ));
        content.push_str(&format!(
            "    // {}\n",
            service.path.display()
        ));
    }

    content.push_str("];\n");

    // 写入文件
    std::fs::write(output_path, content)?;

    Ok(())
}

/// 获取可发布的资源列表
///
/// 扫描 vendor 目录下的包，查找可发布的配置文件
///
/// # 参数
/// - `vendor_dir`: vendor 目录路径
/// - `project`: 项目信息
///
/// # 返回值
/// 可发布资源列表
fn get_publishable_resources(
    vendor_dir: &std::path::Path,
    project: &project::detector::Project,
) -> Vec<PublishableItem> {
    let mut items = Vec::new();

    // 遍历 vendor 目录
    if let Ok(entries) = std::fs::read_dir(vendor_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 遍历供应商目录下的包目录
                if let Ok(package_entries) = std::fs::read_dir(&path) {
                    for package_entry in package_entries.flatten() {
                        let package_path = package_entry.path();
                        if package_path.is_dir() {
                            let package_name = format!(
                                "{}/{}",
                                path.file_name().unwrap().to_string_lossy(),
                                package_path.file_name().unwrap().to_string_lossy()
                            );

                            // 检查配置文件目录
                            let config_src = package_path.join("config");
                            if config_src.exists() && config_src.is_dir() {
                                if let Ok(config_files) = std::fs::read_dir(&config_src) {
                                    for config_file in config_files.flatten() {
                                        if config_file.path().extension().map(|e| e == "php").unwrap_or(false) {
                                            let file_name = config_file.file_name().to_string_lossy().to_string();
                                            items.push(PublishableItem {
                                                package: package_name.clone(),
                                                source: config_file.path(),
                                                destination: format!("config/{}", file_name),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    items
}
