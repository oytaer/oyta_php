//! 优化相关命令模块
//!
//! 提供应用优化、路由缓存、字段缓存、清除缓存等功能

use anyhow::Result;

use crate::env_loader;
use crate::project;

/// 处理 optimize 命令
///
/// 一键执行所有优化操作
pub async fn handle_optimize() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  优化应用");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 1. 缓存配置
    println!("\n  [1/3] 缓存配置...");
    handle_config_cache_internal(&project).await?;
    println!("  ✓ 配置已缓存");

    // 2. 缓存路由
    println!("\n  [2/3] 缓存路由...");
    handle_route_cache_internal(&project).await?;
    println!("  ✓ 路由已缓存");

    // 3. 生成字段缓存
    println!("\n  [3/3] 生成字段缓存...");
    handle_schema_cache_internal(&project).await?;
    println!("  ✓ 字段缓存已生成");

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✓ 应用优化完成");

    Ok(())
}

/// 处理 optimize:route 命令
///
/// 生成路由缓存
pub async fn handle_optimize_route() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("正在生成路由缓存...");

    handle_route_cache_internal(&project).await?;

    println!("✓ 路由缓存已生成");

    Ok(())
}

/// 处理 optimize:schema 命令
///
/// 生成数据表字段缓存
pub async fn handle_optimize_schema() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("正在生成字段缓存...");

    handle_schema_cache_internal(&project).await?;

    println!("✓ 字段缓存已生成");

    Ok(())
}

/// 处理 clear 命令
///
/// 清除运行时缓存
pub async fn handle_clear() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("正在清除运行时缓存...");

    // 清除缓存目录
    let cache_dir = project.runtime_dir.join("cache");
    if cache_dir.exists() {
        clear_directory(&cache_dir)?;
    }

    // 清除临时文件
    let temp_dir = project.runtime_dir.join("temp");
    if temp_dir.exists() {
        clear_directory(&temp_dir)?;
    }

    // 清除日志文件
    let log_dir = project.runtime_dir.join("log");
    if log_dir.exists() {
        clear_directory(&log_dir)?;
    }

    println!("✓ 运行时缓存已清除");

    Ok(())
}

// ============================================================================
// 内部辅助函数
// ============================================================================

/// 内部：缓存配置
async fn handle_config_cache_internal(project: &project::detector::Project) -> Result<()> {
    // 加载配置
    let config_store = crate::config::store::ConfigStore::new();
    let config_loader = crate::config::loader::ConfigLoader::new(&project.config_dir);
    let mut php_parser = crate::parser::php_parser::PhpParser::new();
    config_loader.load(&config_store, &mut php_parser)?;

    // 生成配置缓存文件
    let cache_file = project.runtime_dir.join("cache/config.php");
    
    if let Some(parent) = cache_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 序列化配置
    let config_data = serialize_config(&config_store);
    std::fs::write(&cache_file, config_data)?;

    Ok(())
}

/// 序列化配置存储为 PHP 格式字符串
fn serialize_config(store: &crate::config::store::ConfigStore) -> String {
    let mut content = String::new();
    content.push_str("<?php\n");
    content.push_str("// 自动生成的配置缓存文件\n\n");
    content.push_str("return [\n");
    
    for key in store.keys() {
        if let Some(value) = store.get(&key) {
            content.push_str(&format!("    '{}' => {},\n", key, format_config_value_php(&value)));
        }
    }
    
    content.push_str("];\n");
    content
}

/// 格式化配置值为 PHP 格式
fn format_config_value_php(value: &crate::symbol_table::types::ConfigValue) -> String {
    use crate::symbol_table::types::ConfigValue;
    
    match value {
        ConfigValue::String(s) => format!("'{}'", s.replace('\'', "\\'")),
        ConfigValue::Int(i) => i.to_string(),
        ConfigValue::Float(f) => f.to_string(),
        ConfigValue::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
        ConfigValue::Null => "null".to_string(),
        ConfigValue::IndexedArray(arr) => {
            let items: Vec<String> = arr.iter()
                .map(|v| format_config_value_php(v))
                .collect();
            format!("[{}]", items.join(", "))
        }
        ConfigValue::AssociativeArray(map) => {
            let items: Vec<String> = map.iter()
                .map(|(k, v)| format!("'{}' => {}", k, format_config_value_php(v)))
                .collect();
            format!("[{}]", items.join(", "))
        }
    }
}

/// 内部：缓存路由
async fn handle_route_cache_internal(project: &project::detector::Project) -> Result<()> {
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
    
    if let Some(parent) = cache_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 使用 all_routes() 方法获取路由
    let routes = route_registry.all_routes();
    let mut content = String::new();
    content.push_str("<?php\n");
    content.push_str("// 自动生成的路由缓存文件\n\n");
    content.push_str("return [\n");
    
    for route in &routes {
        // 获取处理器字符串
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

/// 内部：生成字段缓存
async fn handle_schema_cache_internal(project: &project::detector::Project) -> Result<()> {
    // 生成字段缓存文件
    let cache_file = project.runtime_dir.join("cache/schema.php");
    
    if let Some(parent) = cache_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 这里应该扫描数据库表结构并缓存
    // 简化实现：生成空缓存文件
    let content = r#"<?php
// 自动生成的字段缓存文件
return [];
"#;
    
    std::fs::write(&cache_file, content)?;

    Ok(())
}

/// 清空目录内容
fn clear_directory(dir: &std::path::Path) -> Result<()> {
    if dir.exists() && dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                std::fs::remove_dir_all(&path)?;
            } else {
                std::fs::remove_file(&path)?;
            }
        }
    }
    
    Ok(())
}
