//! 服务管理命令模块
//!
//! 提供查看服务列表、查看服务详情等功能

use anyhow::Result;

use crate::env_loader;
use crate::project;
use crate::service;

/// 处理 service:list 命令
///
/// 查看所有服务状态
pub async fn handle_service_list() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  服务列表");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 获取服务启动管理器
    let bootstrap = service::bootstrap::get_service_bootstrap();

    // 初始化服务系统
    bootstrap.initialize().await?;

    // 获取所有服务
    let services = bootstrap.get_all_services().await;
    let running = bootstrap.get_running_services().await;

    if services.is_empty() {
        println!("\n  没有注册的服务");
    } else {
        println!("\n  {:<20} {:<15} {:<10}", "服务名称", "状态", "延迟加载");
        println!("  ────────────────────────────────────────────────");
        
        for name in &services {
            let state = bootstrap.get_state(name).await;
            let state_str = match state {
                Some(service::traits::ServiceState::Running) => "运行中",
                Some(service::traits::ServiceState::Registered) => "已注册",
                Some(service::traits::ServiceState::Stopped) => "已停止",
                Some(service::traits::ServiceState::Starting) => "启动中",
                Some(service::traits::ServiceState::Stopping) => "停止中",
                Some(service::traits::ServiceState::Failed) => "失败",
                Some(service::traits::ServiceState::Unregistered) => "未注册",
                None => "未知",
            };
            
            let is_running = running.contains(name);
            let deferred = !is_running; // 未运行的服务通常是延迟加载的
            
            println!("  {:<20} {:<15} {:<10}", name, state_str, if deferred { "是" } else { "否" });
        }
    }

    println!("\n  已注册: {} 个服务", services.len());
    println!("  运行中: {} 个服务", running.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

/// 处理 service:show 命令
///
/// 查看指定服务详情
///
/// # 参数
/// - `name`: 服务名称
pub async fn handle_service_show(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  服务详情: {}", name);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 获取服务启动管理器
    let bootstrap = service::bootstrap::get_service_bootstrap();

    // 初始化服务系统
    bootstrap.initialize().await?;

    // 检查服务是否存在
    let all_services = bootstrap.get_all_services().await;
    
    if !all_services.contains(&name.to_string()) {
        println!("\n  ⚠ 服务 '{}' 不存在", name);
        println!("\n  可用服务:");
        for service_name in &all_services {
            println!("    - {}", service_name);
        }
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        return Ok(());
    }

    // 获取服务状态
    let state = bootstrap.get_state(name).await;
    let state_str = match state {
        Some(service::traits::ServiceState::Running) => "运行中",
        Some(service::traits::ServiceState::Registered) => "已注册",
        Some(service::traits::ServiceState::Stopped) => "已停止",
        Some(service::traits::ServiceState::Starting) => "启动中",
        Some(service::traits::ServiceState::Stopping) => "停止中",
        Some(service::traits::ServiceState::Failed) => "失败",
        Some(service::traits::ServiceState::Unregistered) => "未注册",
        None => "未知",
    };

    // 获取服务注册表
    let registry = bootstrap.registry();
    let metadata = registry.get_metadata(name).await;

    println!("\n  基本信息:");
    println!("  ────────────────────────────────────────");
    println!("  名称: {}", name);
    println!("  状态: {}", state_str);

    if let Some(meta) = metadata {
        println!("  类型: {}", meta.service_type);
        if !meta.description.is_empty() {
            println!("  描述: {}", meta.description);
        }
        println!("  版本: {}", meta.version);
        if !meta.author.is_empty() {
            println!("  作者: {}", meta.author);
        }
        println!("  后台服务: {}", if meta.is_background { "是" } else { "否" });
        
        if !meta.dependencies.is_empty() {
            println!("\n  依赖服务:");
            for dep in &meta.dependencies {
                println!("    - {}", dep);
            }
        }
        
        if !meta.tags.is_empty() {
            println!("\n  标签: {}", meta.tags.join(", "));
        }
    }

    // 显示配置文件路径
    let config_file = project.config_dir.join(format!("{}.php", name));
    if config_file.exists() {
        println!("\n  配置文件: {}", config_file.display());
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

/// 处理 service:discover 命令
///
/// 自动发现并注册扩展包的服务提供者
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

/// 服务提供者信息
#[derive(Debug, Clone)]
struct ServiceProviderInfo {
    name: String,
    package: String,
    path: std::path::PathBuf,
    deferred: bool,
    provides: Vec<String>,
}

/// 生成服务注册文件
fn generate_service_file(
    services: &[ServiceProviderInfo],
    output_path: &std::path::Path,
) -> Result<()> {
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

/// 处理 vendor:publish 命令
///
/// 发布扩展包的配置文件或资源文件到应用目录
///
/// # 参数
/// - `package`: 指定要发布的包名
/// - `force`: 是否强制覆盖
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

/// 可发布资源项
#[derive(Debug, Clone)]
struct PublishableItem {
    package: String,
    source: std::path::PathBuf,
    destination: String,
}

/// 获取可发布的资源列表
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
