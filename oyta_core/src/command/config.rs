//! 配置管理命令模块
//!
//! 提供配置缓存、清除配置缓存、查看配置值、设置配置值等功能

use anyhow::Result;

use crate::config;
use crate::env_loader;
use crate::project;

/// 处理 config:cache 命令
///
/// 将所有配置文件合并编译为缓存
pub async fn handle_config_cache() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("正在缓存配置...");

    // 加载配置
    let config_store = config::store::ConfigStore::new();
    let config_loader = config::loader::ConfigLoader::new(&project.config_dir);
    let mut php_parser = crate::parser::php_parser::PhpParser::new();
    config_loader.load(&config_store, &mut php_parser)?;

    // 生成配置缓存文件
    let cache_file = project.runtime_dir.join("cache/config.php");
    
    // 确保目录存在
    if let Some(parent) = cache_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 序列化配置到缓存文件
    let config_data = serialize_config(&config_store);
    std::fs::write(&cache_file, config_data)?;

    println!("✓ 配置已缓存到: {}", cache_file.display());

    Ok(())
}

/// 处理 config:clear 命令
///
/// 清除配置缓存
pub async fn handle_config_clear() -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    tracing::info!("正在清除配置缓存...");

    // 删除配置缓存文件
    let cache_file = project.runtime_dir.join("cache/config.php");
    
    if cache_file.exists() {
        std::fs::remove_file(&cache_file)?;
        println!("✓ 配置缓存已清除");
    } else {
        println!("⚠ 配置缓存文件不存在");
    }

    Ok(())
}

/// 处理 config:get 命令
///
/// 查看配置值
///
/// # 参数
/// - `key`: 配置键名，如 app.debug
pub async fn handle_config_get(key: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 加载配置
    let config_store = config::store::ConfigStore::new();
    let config_loader = config::loader::ConfigLoader::new(&project.config_dir);
    let mut php_parser = crate::parser::php_parser::PhpParser::new();
    config_loader.load(&config_store, &mut php_parser)?;
    config::facade::Config::init_from_store(&config_store);

    // 获取配置值
    match config::facade::Config::get(key) {
        Some(value) => {
            println!("{} = {}", key, format_config_value(&value));
        }
        None => {
            println!("⚠ 配置键 '{}' 不存在", key);
        }
    }

    Ok(())
}

/// 处理 config:set 命令
///
/// 设置配置值
///
/// # 参数
/// - `key`: 配置键名
/// - `value`: 配置值
pub async fn handle_config_set(key: &str, value: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 加载配置
    let config_store = config::store::ConfigStore::new();
    let config_loader = config::loader::ConfigLoader::new(&project.config_dir);
    let mut php_parser = crate::parser::php_parser::PhpParser::new();
    config_loader.load(&config_store, &mut php_parser)?;
    config::facade::Config::init_from_store(&config_store);

    // 解析值
    let config_value = parse_config_value(value);

    // 设置配置值
    config::facade::Config::set(key, config_value);

    println!("✓ 已设置配置: {} = {}", key, value);

    Ok(())
}

/// 序列化配置存储为 PHP 格式字符串
fn serialize_config(store: &config::store::ConfigStore) -> String {
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

/// 格式化配置值为显示格式
fn format_config_value(value: &crate::symbol_table::types::ConfigValue) -> String {
    use crate::symbol_table::types::ConfigValue;
    
    match value {
        ConfigValue::String(s) => format!("\"{}\"", s),
        ConfigValue::Int(i) => i.to_string(),
        ConfigValue::Float(f) => f.to_string(),
        ConfigValue::Bool(b) => b.to_string(),
        ConfigValue::Null => "null".to_string(),
        ConfigValue::IndexedArray(arr) => {
            let items: Vec<String> = arr.iter()
                .map(|v| format_config_value(v))
                .collect();
            format!("[{}]", items.join(", "))
        }
        ConfigValue::AssociativeArray(map) => {
            let items: Vec<String> = map.iter()
                .map(|(k, v)| format!("\"{}\" => {}", k, format_config_value(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
    }
}

/// 解析配置值
fn parse_config_value(value: &str) -> crate::symbol_table::types::ConfigValue {
    use crate::symbol_table::types::ConfigValue;
    
    // 尝试解析布尔值
    if value == "true" {
        return ConfigValue::Bool(true);
    }
    if value == "false" {
        return ConfigValue::Bool(false);
    }
    
    // 尝试解析 null
    if value == "null" {
        return ConfigValue::Null;
    }
    
    // 尝试解析整数
    if let Ok(i) = value.parse::<i64>() {
        return ConfigValue::Int(i);
    }
    
    // 尝试解析浮点数
    if let Ok(f) = value.parse::<f64>() {
        return ConfigValue::Float(f);
    }
    
    // 默认作为字符串
    // 去除引号
    if (value.starts_with('"') && value.ends_with('"')) 
        || (value.starts_with('\'') && value.ends_with('\'')) {
        ConfigValue::String(value[1..value.len()-1].to_string())
    } else {
        ConfigValue::String(value.to_string())
    }
}
