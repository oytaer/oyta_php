//! 版本比较工具模块
//!
//! 包含版本号解析、比较和规范化相关的工具函数

use anyhow::Result;

use super::types::LatestVersionInfo;

/// 从 Packagist API 获取最新版本信息
///
/// # 参数
/// - `packagist`: Packagist 客户端引用
/// - `package_name`: 包名
///
/// # 返回值
/// 返回最新版本信息
pub async fn fetch_latest_version(
    packagist: &mut crate::composer::packagist::PackagistClient,
    package_name: &str,
) -> Result<LatestVersionInfo> {
    // 从 Packagist 获取包信息
    let package_info = packagist.get_package(package_name).await?;

    // 获取所有版本（versions 是 HashMap，获取其 keys）
    let version_keys: Vec<&str> = package_info.versions.keys().map(|s| s.as_str()).collect();

    // 如果没有版本，返回错误
    if version_keys.is_empty() {
        anyhow::bail!("包 {} 没有可用版本", package_name);
    }

    // 找出最新的稳定版本
    let mut latest_stable: Option<&str> = None;
    let mut latest_overall: Option<&str> = None;

    // 遍历所有版本
    for version in version_keys {
        // 更新最新版本（包括预发布版本）
        if latest_overall.is_none()
            || compare_versions(version, latest_overall.unwrap()) > 0
        {
            latest_overall = Some(version);
        }

        // 更新最新稳定版本（排除预发布版本）
        if !is_unstable_version(version) {
            if latest_stable.is_none()
                || compare_versions(version, latest_stable.unwrap()) > 0
            {
                latest_stable = Some(version);
            }
        }
    }

    // 优先使用稳定版本
    let latest_version = latest_stable
        .or(latest_overall)
        .unwrap_or("unknown")
        .to_string();

    Ok(LatestVersionInfo {
        latest_version: latest_version.clone(),
        is_outdated: false,
        // description 是 String 字段，直接访问
        description: package_info.description.clone(),
    })
}

/// 判断版本是否为不稳定版本
///
/// 不稳定版本包括：alpha、beta、RC、dev 等
///
/// # 参数
/// - `version`: 版本号字符串
///
/// # 返回值
/// 如果是不稳定版本返回 true
pub fn is_unstable_version(version: &str) -> bool {
    let lower = version.to_lowercase();

    // 检查常见的不稳定版本标识
    lower.contains("alpha")
        || lower.contains("beta")
        || lower.contains("rc")
        || lower.contains("dev")
        || lower.contains("snapshot")
        || lower.contains("preview")
        || lower.contains("pre")
        || lower.contains("unstable")
}

/// 规范化版本号
///
/// 移除版本号前缀（如 v1.0.0 → 1.0.0）
/// 并处理特殊的版本格式
///
/// # 参数
/// - `version`: 原始版本号
///
/// # 返回值
/// 规范化后的版本号
pub fn normalize_version(version: &str) -> String {
    let mut v = version.trim();

    // 移除 'v' 或 'V' 前缀
    if v.starts_with('v') || v.starts_with('V') {
        v = &v[1..];
    }

    // 移除不稳定版本后缀（用于比较）
    if let Some(pos) = v.find('-') {
        v = &v[..pos];
    }

    v.to_string()
}

/// 解析版本号为元组
///
/// 将版本号字符串解析为数字元组
/// 例如：1.2.3 → (1, 2, 3)
///
/// # 参数
/// - `version`: 版本号字符串
///
/// # 返回值
/// 版本号元组，最多 4 个部分
pub fn parse_version_tuple(version: &str) -> (u32, u32, u32, u32) {
    let normalized = normalize_version(version);

    // 分割版本号各部分
    let parts: Vec<&str> = normalized.split('.').collect();

    // 解析各部分为数字
    let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    let build = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);

    (major, minor, patch, build)
}

/// 比较两个版本号
///
/// # 参数
/// - `v1`: 第一个版本号
/// - `v2`: 第二个版本号
///
/// # 返回值
/// - 正数：v1 > v2
/// - 0：v1 == v2
/// - 负数：v1 < v2
pub fn compare_versions(v1: &str, v2: &str) -> i32 {
    // 解析版本号
    let t1 = parse_version_tuple(v1);
    let t2 = parse_version_tuple(v2);

    // 比较各部分
    if t1.0 != t2.0 {
        return (t1.0 as i32) - (t2.0 as i32);
    }
    if t1.1 != t2.1 {
        return (t1.1 as i32) - (t2.1 as i32);
    }
    if t1.2 != t2.2 {
        return (t1.2 as i32) - (t2.2 as i32);
    }
    if t1.3 != t2.3 {
        return (t1.3 as i32) - (t2.3 as i32);
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试不稳定版本判断
    #[test]
    fn test_is_unstable_version() {
        assert!(is_unstable_version("1.0.0-alpha"));
        assert!(is_unstable_version("1.0.0-beta"));
        assert!(is_unstable_version("1.0.0-RC1"));
        assert!(is_unstable_version("1.0.0-dev"));
        assert!(!is_unstable_version("1.0.0"));
        assert!(!is_unstable_version("2.1.3"));
    }

    /// 测试版本号规范化
    #[test]
    fn test_normalize_version() {
        assert_eq!(normalize_version("v1.0.0"), "1.0.0");
        assert_eq!(normalize_version("V2.1.3"), "2.1.3");
        assert_eq!(normalize_version("1.0.0-beta"), "1.0.0");
        assert_eq!(normalize_version("1.0.0"), "1.0.0");
    }

    /// 测试版本号解析
    #[test]
    fn test_parse_version_tuple() {
        assert_eq!(parse_version_tuple("1.2.3"), (1, 2, 3, 0));
        assert_eq!(parse_version_tuple("1.2.3.4"), (1, 2, 3, 4));
        assert_eq!(parse_version_tuple("1.2"), (1, 2, 0, 0));
        assert_eq!(parse_version_tuple("1"), (1, 0, 0, 0));
    }

    /// 测试版本比较
    #[test]
    fn test_compare_versions() {
        assert!(compare_versions("1.0.0", "1.0.0") == 0);
        assert!(compare_versions("2.0.0", "1.0.0") > 0);
        assert!(compare_versions("1.0.0", "2.0.0") < 0);
        assert!(compare_versions("1.2.0", "1.1.0") > 0);
        assert!(compare_versions("1.0.1", "1.0.0") > 0);
        assert!(compare_versions("v1.0.0", "1.0.0") == 0);
    }
}
