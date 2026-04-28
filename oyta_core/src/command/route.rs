//! 路由相关命令模块
//!
//! 提供查看路由列表、缓存路由、清除路由缓存等功能

use anyhow::Result;

use crate::env_loader;
use crate::project;

/// 处理 route:list 命令
///
/// 查看所有已注册的路由
pub async fn handle_route_list() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  路由列表");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 扫描项目文件
    let mut scanner = crate::parser::scanner::Scanner::new();
    let _ = scanner.scan_all(&project.root)?;

    // 构建路由表
    let route_registry = crate::router::registry::RouteRegistry::new();
    let controllers = scanner.registry().get_controllers();
    route_registry.generate_auto_routes(&controllers, project.is_multi_app(), None);

    // 解析路由定义文件
    let route_dir = project.root.join("route");
    let mut php_parser = crate::parser::php_parser::PhpParser::new();
    let _ = crate::router::route_parser::RouteFileParser::parse_route_dir(
        &route_dir,
        &route_registry,
        &mut php_parser,
    )?;

    // 获取所有路由
    let routes = route_registry.all_routes();

    if routes.is_empty() {
        println!("\n  没有注册的路由");
    } else {
        println!("\n  {:<10} {:<40} {:<30}", "方法", "路径", "处理器");
        println!("  ────────────────────────────────────────────────────────────────");
        
        for route in &routes {
            // 获取处理器字符串
            let handler_str = format_handler(&route.handler);
            println!("  {:<10} {:<40} {:<30}", 
                route.method.to_string(),
                route.path,
                handler_str
            );
        }
    }

    println!("\n  共 {} 条路由", routes.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

/// 处理 route:cache 命令
///
/// 将路由表编译缓存
pub async fn handle_route_cache() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("正在缓存路由...");

    // 扫描项目文件
    let mut scanner = crate::parser::scanner::Scanner::new();
    let _ = scanner.scan_all(&project.root)?;

    // 构建路由表
    let route_registry = crate::router::registry::RouteRegistry::new();
    let controllers = scanner.registry().get_controllers();
    route_registry.generate_auto_routes(&controllers, project.is_multi_app(), None);

    // 解析路由定义文件
    let route_dir = project.root.join("route");
    let mut php_parser = crate::parser::php_parser::PhpParser::new();
    let _ = crate::router::route_parser::RouteFileParser::parse_route_dir(
        &route_dir,
        &route_registry,
        &mut php_parser,
    )?;

    // 生成路由缓存文件
    let cache_file = project.runtime_dir.join("cache/routes.php");
    
    // 确保目录存在
    if let Some(parent) = cache_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 序列化路由到缓存文件
    let routes = route_registry.all_routes();
    let mut content = String::new();
    content.push_str("<?php\n");
    content.push_str("// 自动生成的路由缓存文件\n\n");
    content.push_str("return [\n");
    
    for route in &routes {
        let handler_str = format_handler(&route.handler);
        content.push_str(&format!(
            "    ['{}', '{}', '{}'],\n",
            route.method.to_string(),
            route.path,
            handler_str
        ));
    }
    
    content.push_str("];\n");
    
    std::fs::write(&cache_file, content)?;

    println!("✓ 路由已缓存到: {}", cache_file.display());

    Ok(())
}

/// 处理 route:clear 命令
///
/// 清除路由缓存
pub async fn handle_route_clear() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("正在清除路由缓存...");

    // 删除路由缓存文件
    let cache_file = project.runtime_dir.join("cache/routes.php");
    
    if cache_file.exists() {
        std::fs::remove_file(&cache_file)?;
        println!("✓ 路由缓存已清除");
    } else {
        println!("⚠ 路由缓存文件不存在");
    }

    Ok(())
}

/// 格式化路由处理器为字符串
fn format_handler(handler: &crate::router::types::RouteHandler) -> String {
    use crate::router::types::RouteHandler;
    
    match handler {
        RouteHandler::ControllerMethod { controller, method } => {
            format!("{}@{}", controller, method)
        }
        RouteHandler::Closure { id } => {
            format!("Closure({})", id)
        }
        RouteHandler::Static { content_type, status, .. } => {
            format!("Static({}/{})", status, content_type)
        }
        RouteHandler::Redirect { url, status } => {
            format!("Redirect({} -> {})", status, url)
        }
    }
}
