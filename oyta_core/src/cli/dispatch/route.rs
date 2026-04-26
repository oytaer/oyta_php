//! 路由命令模块
//!
//! 包含路由列表和路由缓存相关命令

use anyhow::Result;

use crate::env_loader;
use crate::parser::scanner::Scanner;
use crate::project;
use crate::router::registry::RouteRegistry;
use super::super::optimizer;

/// 处理 route:list 命令：查看路由列表
/// 显示所有已注册的路由信息
pub async fn handle_route_list() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 扫描项目文件
    let mut scanner = Scanner::new();
    scanner.scan_all(&project.root)?;

    // 生成自动路由
    let route_registry = RouteRegistry::new();
    let controllers = scanner.registry().get_controllers();
    route_registry.generate_auto_routes(&controllers, project.is_multi_app(), None);

    // 解析路由定义文件
    let route_dir = project.root.join("route");
    let mut php_parser = crate::parser::php_parser::PhpParser::new();
    let _ = crate::router::route_parser::RouteFileParser::parse_route_dir(
        &route_dir,
        &route_registry,
        &mut php_parser,
    );

    // 获取所有路由
    let routes = route_registry.all_routes();

    // 如果没有路由，输出提示并返回
    if routes.is_empty() {
        println!("没有注册的路由");
        return Ok(());
    }

    // 输出路由列表
    println!("路由列表 (共 {} 条):", routes.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{:<8} {:<10} {:<40} {}", "方法", "类型", "路径", "处理器");
    println!("────────────────────────────────────────────────────────────────────");

    // 遍历并输出每个路由
    for route in &routes {
        let method = route.method.as_str();
        // 格式化处理器字符串
        let handler_str = match &route.handler {
            crate::router::types::RouteHandler::ControllerMethod { controller, method } => {
                format!("{}@{}", controller, method)
            }
            crate::router::types::RouteHandler::Static { content, .. } => {
                format!(
                    "[static] {}",
                    if content.len() > 30 {
                        &content[..30]
                    } else {
                        content
                    }
                )
            }
            crate::router::types::RouteHandler::Redirect { url, .. } => {
                format!("[redirect] {}", url)
            }
            crate::router::types::RouteHandler::Closure { id } => {
                format!("[closure] {}", id)
            }
        };
        let route_type = route.group.as_deref().unwrap_or("auto");
        println!("{:<8} {:<10} {:<40} {}", method, route_type, route.path, handler_str);
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    Ok(())
}

/// 处理 route:cache 命令：缓存路由
///
/// 将路由配置编译为缓存文件，提高路由匹配性能
pub async fn handle_route_cache() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建优化器并执行路由缓存
    let optimizer = optimizer::Optimizer::new(project);
    optimizer.cache_routes().await
}
