//! composer.json 解析器
//!
//! 解析 composer.json 文件，提取依赖、自动加载等配置
//! 支持完整的 composer.json 规范

use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;

use super::manager::{Author, AutoloadConfig, ComposerConfig, ComposerJson, Repository};

/// 解析 composer.json 文件
///
/// # 参数
/// - `path`: composer.json 文件路径
///
/// # 返回
/// 解析后的 ComposerJson 结构
pub async fn parse_composer_json(path: &str) -> Result<ComposerJson> {
    let content = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("无法读取 composer.json: {}", path))?;

    parse_composer_json_content(&content)
}

/// 解析 composer.json 内容
///
/// # 参数
/// - `content`: JSON 内容字符串
///
/// # 返回
/// 解析后的 ComposerJson 结构
pub fn parse_composer_json_content(content: &str) -> Result<ComposerJson> {
    let json: Value = serde_json::from_str(content)
        .with_context(|| "composer.json 格式错误")?;

    Ok(ComposerJson {
        name: json.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),

        description: json.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),

        r#type: json.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("project")
            .to_string(),

        license: parse_string_or_array(json.get("license")),

        authors: parse_authors(json.get("authors")),

        require: parse_require(json.get("require")),

        require_dev: parse_require(json.get("require-dev")),

        autoload: parse_autoload(json.get("autoload")),

        autoload_dev: parse_autoload(json.get("autoload-dev")),

        repositories: parse_repositories(json.get("repositories")),

        minimum_stability: json.get("minimum-stability")
            .and_then(|v| v.as_str())
            .unwrap_or("stable")
            .to_string(),

        prefer_stable: json.get("prefer-stable")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),

        scripts: parse_scripts(json.get("scripts")),

        extra: json.get("extra")
            .cloned()
            .unwrap_or(Value::Null),

        config: parse_config(json.get("config")),
    })
}

/// 解析字符串或字符串数组
fn parse_string_or_array(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    }
}

/// 解析作者列表
fn parse_authors(value: Option<&Value>) -> Vec<Author> {
    match value {
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| {
                Some(Author {
                    name: v.get("name")?.as_str()?.to_string(),
                    email: v.get("email")
                        .and_then(|e| e.as_str())
                        .unwrap_or("")
                        .to_string(),
                    homepage: v.get("homepage").and_then(|h| h.as_str()).map(|s| s.to_string()),
                    role: v.get("role").and_then(|r| r.as_str()).map(|s| s.to_string()),
                })
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// 解析依赖映射
fn parse_require(value: Option<&Value>) -> HashMap<String, String> {
    match value {
        Some(Value::Object(obj)) => obj
            .iter()
            .filter_map(|(k, v)| {
                v.as_str().map(|s| (k.clone(), s.to_string()))
            })
            .collect(),
        _ => HashMap::new(),
    }
}

/// 解析自动加载配置
fn parse_autoload(value: Option<&Value>) -> AutoloadConfig {
    match value {
        Some(Value::Object(obj)) => AutoloadConfig {
            psr_4: parse_autoload_mapping(obj.get("psr-4")),
            psr_0: parse_autoload_mapping(obj.get("psr-0")),
            classmap: parse_string_array(obj.get("classmap")),
            files: parse_string_array(obj.get("files")),
        },
        _ => AutoloadConfig::default(),
    }
}

/// 解析自动加载映射
fn parse_autoload_mapping(value: Option<&Value>) -> HashMap<String, Vec<String>> {
    match value {
        Some(Value::Object(obj)) => obj
            .iter()
            .filter_map(|(k, v)| {
                let paths = match v {
                    Value::String(s) => vec![s.clone()],
                    Value::Array(arr) => arr
                        .iter()
                        .filter_map(|item| item.as_str().map(|s| s.to_string()))
                        .collect(),
                    _ => return None,
                };
                Some((k.clone(), paths))
            })
            .collect(),
        _ => HashMap::new(),
    }
}

/// 解析字符串数组
fn parse_string_array(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    }
}

/// 解析仓库列表
fn parse_repositories(value: Option<&Value>) -> Vec<Repository> {
    match value {
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| {
                Some(Repository {
                    r#type: v.get("type")?.as_str()?.to_string(),
                    url: v.get("url")
                        .and_then(|u| u.as_str())
                        .unwrap_or("")
                        .to_string(),
                    name: v.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()),
                })
            })
            .collect(),
        Some(Value::Object(obj)) => {
            // 支持对象格式的仓库配置
            obj.iter()
                .filter_map(|(name, v)| {
                    Some(Repository {
                        r#type: v.get("type")?.as_str()?.to_string(),
                        url: v.get("url")
                            .and_then(|u| u.as_str())
                            .unwrap_or("")
                            .to_string(),
                        name: Some(name.clone()),
                    })
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

/// 解析脚本配置
fn parse_scripts(value: Option<&Value>) -> HashMap<String, Vec<String>> {
    match value {
        Some(Value::Object(obj)) => obj
            .iter()
            .filter_map(|(k, v)| {
                let commands = match v {
                    Value::String(s) => vec![s.clone()],
                    Value::Array(arr) => arr
                        .iter()
                        .filter_map(|item| item.as_str().map(|s| s.to_string()))
                        .collect(),
                    _ => return None,
                };
                Some((k.clone(), commands))
            })
            .collect(),
        _ => HashMap::new(),
    }
}

/// 解析 Composer 配置选项
fn parse_config(value: Option<&Value>) -> ComposerConfig {
    match value {
        Some(Value::Object(obj)) => ComposerConfig {
            platform: parse_require(obj.get("platform")),
            optimize_autoloader: obj.get("optimize-autoloader")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            classmap_authoritative: obj.get("classmap-authoritative")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            apcu_autoloader: obj.get("apcu-autoloader")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        },
        _ => ComposerConfig::default(),
    }
}

/// 写入 composer.json 文件
///
/// # 参数
/// - `path`: 文件路径
/// - `config`: 配置结构
pub async fn write_composer_json(path: &str, config: &ComposerJson) -> Result<()> {
    let mut obj = serde_json::Map::new();

    if !config.name.is_empty() {
        obj.insert("name".to_string(), Value::String(config.name.clone()));
    }

    if !config.description.is_empty() {
        obj.insert("description".to_string(), Value::String(config.description.clone()));
    }

    if !config.r#type.is_empty() {
        obj.insert("type".to_string(), Value::String(config.r#type.clone()));
    }

    if !config.license.is_empty() {
        if config.license.len() == 1 {
            obj.insert("license".to_string(), Value::String(config.license[0].clone()));
        } else {
            obj.insert("license".to_string(), Value::Array(
                config.license.iter().map(|s| Value::String(s.clone())).collect()
            ));
        }
    }

    if !config.authors.is_empty() {
        obj.insert("authors".to_string(), Value::Array(
            config.authors.iter().map(|a| {
                let mut author_obj = serde_json::Map::new();
                author_obj.insert("name".to_string(), Value::String(a.name.clone()));
                author_obj.insert("email".to_string(), Value::String(a.email.clone()));
                if let Some(ref homepage) = a.homepage {
                    author_obj.insert("homepage".to_string(), Value::String(homepage.clone()));
                }
                if let Some(ref role) = a.role {
                    author_obj.insert("role".to_string(), Value::String(role.clone()));
                }
                Value::Object(author_obj)
            }).collect()
        ));
    }

    if !config.require.is_empty() {
        obj.insert("require".to_string(), Value::Object(
            config.require.iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect()
        ));
    }

    if !config.require_dev.is_empty() {
        obj.insert("require-dev".to_string(), Value::Object(
            config.require_dev.iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect()
        ));
    }

    if !config.autoload.psr_4.is_empty() || !config.autoload.files.is_empty() {
        obj.insert("autoload".to_string(), serialize_autoload(&config.autoload));
    }

    if !config.minimum_stability.is_empty() && config.minimum_stability != "stable" {
        obj.insert("minimum-stability".to_string(), Value::String(config.minimum_stability.clone()));
    }

    if !config.prefer_stable {
        obj.insert("prefer-stable".to_string(), Value::Bool(config.prefer_stable));
    }

    if !config.scripts.is_empty() {
        obj.insert("scripts".to_string(), Value::Object(
            config.scripts.iter()
                .map(|(k, v)| {
                    if v.len() == 1 {
                        (k.clone(), Value::String(v[0].clone()))
                    } else {
                        (k.clone(), Value::Array(v.iter().map(|s| Value::String(s.clone())).collect()))
                    }
                })
                .collect()
        ));
    }

    let json = Value::Object(obj);
    let content = serde_json::to_string_pretty(&json)
        .with_context(|| "无法序列化 composer.json")?;

    tokio::fs::write(path, content)
        .await
        .with_context(|| format!("无法写入 composer.json: {}", path))?;

    Ok(())
}

/// 序列化自动加载配置
fn serialize_autoload(autoload: &AutoloadConfig) -> Value {
    let mut obj = serde_json::Map::new();

    if !autoload.psr_4.is_empty() {
        obj.insert("psr-4".to_string(), Value::Object(
            autoload.psr_4.iter()
                .map(|(k, v)| {
                    if v.len() == 1 {
                        (k.clone(), Value::String(v[0].clone()))
                    } else {
                        (k.clone(), Value::Array(v.iter().map(|s| Value::String(s.clone())).collect()))
                    }
                })
                .collect()
        ));
    }

    if !autoload.psr_0.is_empty() {
        obj.insert("psr-0".to_string(), Value::Object(
            autoload.psr_0.iter()
                .map(|(k, v)| {
                    if v.len() == 1 {
                        (k.clone(), Value::String(v[0].clone()))
                    } else {
                        (k.clone(), Value::Array(v.iter().map(|s| Value::String(s.clone())).collect()))
                    }
                })
                .collect()
        ));
    }

    if !autoload.classmap.is_empty() {
        obj.insert("classmap".to_string(), Value::Array(
            autoload.classmap.iter().map(|s| Value::String(s.clone())).collect()
        ));
    }

    if !autoload.files.is_empty() {
        obj.insert("files".to_string(), Value::Array(
            autoload.files.iter().map(|s| Value::String(s.clone())).collect()
        ));
    }

    Value::Object(obj)
}

/// 创建默认的 composer.json
///
/// # 参数
/// - `name`: 包名称
/// - `description`: 描述
///
/// # 返回
/// 默认配置的 ComposerJson
pub fn create_default_composer_json(name: &str, description: &str) -> ComposerJson {
    let mut autoload = AutoloadConfig::default();
    autoload.psr_4.insert(
        "app\\".to_string(),
        vec!["app".to_string()],
    );

    ComposerJson {
        name: name.to_string(),
        description: description.to_string(),
        r#type: "project".to_string(),
        license: vec!["MIT".to_string()],
        autoload,
        prefer_stable: true,
        ..ComposerJson::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_composer_json_content() {
        let content = r#"{
            "name": "test/project",
            "description": "Test project",
            "type": "project",
            "license": "MIT",
            "require": {
                "php": ">=8.0"
            },
            "autoload": {
                "psr-4": {
                    "app\\": "app/"
                }
            }
        }"#;

        let config = parse_composer_json_content(content).unwrap();

        assert_eq!(config.name, "test/project");
        assert_eq!(config.description, "Test project");
        assert_eq!(config.license, vec!["MIT"]);
        assert!(config.require.contains_key("php"));
        assert!(config.autoload.psr_4.contains_key("app\\"));
    }

    #[test]
    fn test_create_default_composer_json() {
        let config = create_default_composer_json("test/app", "Test application");

        assert_eq!(config.name, "test/app");
        assert_eq!(config.description, "Test application");
        assert!(config.autoload.psr_4.contains_key("app\\"));
    }
}
