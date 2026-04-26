//! composer.lock 文件处理
//!
//! 读写 composer.lock 文件
//! 锁定依赖版本，确保可重复安装

use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;

use super::manager::{ComposerLock, LockedPackage, PackageSource};

/// 解析 composer.lock 文件
///
/// # 参数
/// - `path`: composer.lock 文件路径
///
/// # 返回
/// 解析后的 ComposerLock 结构
pub async fn parse_composer_lock(path: &str) -> Result<ComposerLock> {
    let content = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("无法读取 composer.lock: {}", path))?;

    parse_composer_lock_content(&content)
}

/// 解析 composer.lock 内容
pub fn parse_composer_lock_content(content: &str) -> Result<ComposerLock> {
    let json: Value = serde_json::from_str(content)
        .with_context(|| "composer.lock 格式错误")?;

    Ok(ComposerLock {
        version: json.get("_read_version")
            .and_then(|v| v.as_i64())
            .unwrap_or(2) as i32,

        content_hash: json.get("content-hash")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),

        packages: parse_packages(json.get("packages")),

        packages_dev: parse_packages(json.get("packages-dev")),

        platform: parse_platform(json.get("platform")),

        platform_dev: parse_platform(json.get("platform-dev")),
    })
}

/// 解析包列表
fn parse_packages(value: Option<&Value>) -> Vec<LockedPackage> {
    match value {
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| {
                Some(LockedPackage {
                    name: v.get("name")?.as_str()?.to_string(),
                    version: v.get("version")?.as_str()?.to_string(),
                    source: PackageSource {
                        r#type: v.get("source")
                            .and_then(|s| s.get("type"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("git")
                            .to_string(),
                        url: v.get("source")
                            .and_then(|s| s.get("url"))
                            .and_then(|u| u.as_str())
                            .unwrap_or("")
                            .to_string(),
                        reference: v.get("source")
                            .and_then(|s| s.get("reference"))
                            .and_then(|r| r.as_str())
                            .unwrap_or("")
                            .to_string(),
                    },
                    install_path: v.get("install-path")
                        .and_then(|p| p.as_str())
                        .unwrap_or("")
                        .to_string(),
                    autoload: super::AutoloadConfig::default(),
                    license: v.get("license")
                        .and_then(|l| l.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default(),
                    require: v.get("require")
                        .and_then(|r| r.as_object())
                        .map(|obj| obj.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect())
                        .unwrap_or_default(),
                })
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// 解析平台要求
fn parse_platform(value: Option<&Value>) -> HashMap<String, String> {
    match value {
        Some(Value::Object(obj)) => obj
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect(),
        _ => HashMap::new(),
    }
}

/// 写入 composer.lock 文件
///
/// # 参数
/// - `project_path`: 项目路径
/// - `lock`: 锁文件内容
pub async fn write_composer_lock(project_path: &str, lock: &ComposerLock) -> Result<()> {
    let mut obj = serde_json::Map::new();

    obj.insert("_read_version".to_string(), Value::Number(lock.version.into()));
    obj.insert("content-hash".to_string(), Value::String(lock.content_hash.clone()));

    // 写入包列表
    obj.insert("packages".to_string(), Value::Array(
        lock.packages.iter().map(serialize_package).collect()
    ));

    obj.insert("packages-dev".to_string(), Value::Array(
        lock.packages_dev.iter().map(serialize_package).collect()
    ));

    // 写入平台要求
    obj.insert("platform".to_string(), Value::Object(
        lock.platform.iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect()
    ));

    obj.insert("platform-dev".to_string(), Value::Object(
        lock.platform_dev.iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect()
    ));

    let json = Value::Object(obj);
    let content = serde_json::to_string_pretty(&json)
        .with_context(|| "无法序列化 composer.lock")?;

    let lock_path = format!("{}/composer.lock", project_path);
    tokio::fs::write(&lock_path, content).await
        .with_context(|| format!("无法写入 composer.lock: {}", lock_path))?;

    Ok(())
}

/// 序列化单个包
fn serialize_package(pkg: &LockedPackage) -> Value {
    let mut obj = serde_json::Map::new();

    obj.insert("name".to_string(), Value::String(pkg.name.clone()));
    obj.insert("version".to_string(), Value::String(pkg.version.clone()));

    let mut source = serde_json::Map::new();
    source.insert("type".to_string(), Value::String(pkg.source.r#type.clone()));
    source.insert("url".to_string(), Value::String(pkg.source.url.clone()));
    source.insert("reference".to_string(), Value::String(pkg.source.reference.clone()));
    obj.insert("source".to_string(), Value::Object(source));

    obj.insert("install-path".to_string(), Value::String(pkg.install_path.clone()));

    if !pkg.license.is_empty() {
        obj.insert("license".to_string(), Value::Array(
            pkg.license.iter().map(|s| Value::String(s.clone())).collect()
        ));
    }

    if !pkg.require.is_empty() {
        obj.insert("require".to_string(), Value::Object(
            pkg.require.iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect()
        ));
    }

    Value::Object(obj)
}

/// 生成锁文件
///
/// # 参数
/// - `packages`: 已解析的包列表
///
/// # 返回
/// 生成的锁文件结构
pub fn generate_lock(packages: &[super::resolver::ResolvedPackage]) -> ComposerLock {
    let locked_packages: Vec<LockedPackage> = packages
        .iter()
        .map(|p| LockedPackage::from(p))
        .collect();

    // 计算内容哈希
    let content_hash = calculate_content_hash(&locked_packages);

    ComposerLock {
        version: 2,
        content_hash,
        packages: locked_packages,
        packages_dev: Vec::new(),
        platform: HashMap::new(),
        platform_dev: HashMap::new(),
    }
}

/// 计算内容哈希
fn calculate_content_hash(packages: &[LockedPackage]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    for pkg in packages {
        pkg.name.hash(&mut hasher);
        pkg.version.hash(&mut hasher);
        pkg.source.reference.hash(&mut hasher);
    }

    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_composer_lock_content() {
        let content = r#"{
            "_read_version": 2,
            "content-hash": "abc123",
            "packages": [
                {
                    "name": "vendor/package",
                    "version": "1.0.0",
                    "source": {
                        "type": "git",
                        "url": "https://github.com/vendor/package.git",
                        "reference": "abc123"
                    },
                    "install-path": "../vendor/package"
                }
            ],
            "platform": {
                "php": ">=8.0"
            }
        }"#;

        let lock = parse_composer_lock_content(content).unwrap();

        assert_eq!(lock.version, 2);
        assert_eq!(lock.content_hash, "abc123");
        assert_eq!(lock.packages.len(), 1);
        assert_eq!(lock.packages[0].name, "vendor/package");
    }
}
