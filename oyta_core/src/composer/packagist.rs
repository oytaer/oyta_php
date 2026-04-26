//! Packagist API 客户端
//!
//! 与 Packagist.org API 交互，获取包信息、版本列表
//! 支持搜索包、获取包详情、获取版本信息

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Packagist API 基础 URL
const PACKAGIST_API_URL: &str = "https://repo.packagist.org";

/// Packagist API 客户端
pub struct PackagistClient {
    /// HTTP 客户端
    client: reqwest::Client,
    /// API 基础 URL
    base_url: String,
    /// 缓存的包信息
    cache: HashMap<String, PackageInfo>,
}

/// 包信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    /// 包名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 版本列表
    pub versions: HashMap<String, VersionInfo>,
    /// 下载统计
    pub downloads: Downloads,
    /// 许可证
    pub license: Vec<String>,
    /// 维护者
    pub maintainers: Vec<Maintainer>,
    /// 时间
    pub time: String,
}

/// 版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// 版本号
    pub version: String,
    /// 版本规范化号
    pub version_normalized: String,
    /// 源信息
    pub source: SourceInfo,
    /// 依赖
    pub require: HashMap<String, String>,
    /// 开发依赖
    pub require_dev: HashMap<String, String>,
    /// 自动加载
    pub autoload: AutoloadInfo,
    /// 时间
    pub time: String,
    /// 类型
    pub r#type: String,
    /// 许可证
    pub license: Vec<String>,
    /// 描述
    pub description: String,
}

/// 源信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    /// 源类型
    pub r#type: String,
    /// URL
    pub url: String,
    /// 引用
    pub reference: String,
}

/// 下载统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Downloads {
    /// 总下载量
    pub total: u64,
    /// 月下载量
    pub monthly: u64,
    /// 日下载量
    pub daily: u64,
}

/// 维护者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Maintainer {
    /// 名称
    pub name: String,
    /// 头像
    pub avatar_url: Option<String>,
}

/// 自动加载信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoloadInfo {
    /// PSR-4
    #[serde(rename = "psr-4")]
    pub psr_4: Option<HashMap<String, Vec<String>>>,
    /// PSR-0
    #[serde(rename = "psr-0")]
    pub psr_0: Option<HashMap<String, Vec<String>>>,
    /// 类映射
    pub classmap: Option<Vec<String>>,
    /// 文件
    pub files: Option<Vec<String>>,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// 结果列表
    pub results: Vec<SearchItem>,
    /// 总数
    pub total: u32,
    /// 下一页
    pub next: Option<String>,
}

/// 搜索结果项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchItem {
    /// 包名称
    pub name: String,
    /// 描述
    pub description: String,
    /// URL
    pub url: String,
    /// 仓库
    pub repository: String,
    /// 下载量
    pub downloads: u64,
    /// 官方标志
    pub official: Option<bool>,
}

impl PackagistClient {
    /// 创建新的 Packagist 客户端
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("OYTAPHP-Composer/1.0")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            base_url: PACKAGIST_API_URL.to_string(),
            cache: HashMap::new(),
        }
    }

    /// 设置自定义 API URL
    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }

    /// 获取包信息
    ///
    /// # 参数
    /// - `package_name`: 包名称（如 laravel/framework）
    ///
    /// # 返回
    /// 包详细信息
    pub async fn get_package(&mut self, package_name: &str) -> Result<PackageInfo> {
        // 检查缓存
        if let Some(cached) = self.cache.get(package_name) {
            return Ok(cached.clone());
        }

        let url = format!("{}/p2/{}.json", self.base_url, package_name);

        tracing::debug!("请求 Packagist API: {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("无法连接 Packagist: {}", url))?;

        if !response.status().is_success() {
            anyhow::bail!("Packagist 返回错误: {}", response.status());
        }

        let json: serde_json::Value = response
            .json()
            .await
            .with_context(|| "无法解析 Packagist 响应")?;

        // 解析包信息
        let package = self.parse_package_response(&json, package_name)?;

        // 缓存结果
        self.cache.insert(package_name.to_string(), package.clone());

        Ok(package)
    }

    /// 解析包响应
    fn parse_package_response(&self, json: &serde_json::Value, package_name: &str) -> Result<PackageInfo> {
        let packages = json.get("packages")
            .and_then(|p| p.get(package_name))
            .and_then(|p| p.as_array())
            .context("无效的包响应格式")?;

        let mut versions = HashMap::new();
        let mut description = String::new();
        let mut license = Vec::new();

        for version_json in packages {
            if let Ok(version) = serde_json::from_value::<VersionInfo>(version_json.clone()) {
                if description.is_empty() {
                    description = version.description.clone();
                }
                if license.is_empty() {
                    license = version.license.clone();
                }
                versions.insert(version.version.clone(), version);
            }
        }

        Ok(PackageInfo {
            name: package_name.to_string(),
            description,
            versions,
            downloads: Downloads {
                total: 0,
                monthly: 0,
                daily: 0,
            },
            license,
            maintainers: Vec::new(),
            time: String::new(),
        })
    }

    /// 搜索包
    ///
    /// # 参数
    /// - `query`: 搜索关键词
    /// - `page`: 页码
    ///
    /// # 返回
    /// 搜索结果
    pub async fn search(&self, query: &str, page: u32) -> Result<SearchResult> {
        let url = format!(
            "{}/search.json?q={}&page={}",
            self.base_url,
            urlencoding::encode(query),
            page
        );

        tracing::debug!("搜索包: {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .with_context(|| "搜索请求失败")?;

        if !response.status().is_success() {
            anyhow::bail!("搜索失败: {}", response.status());
        }

        let result: SearchResult = response
            .json()
            .await
            .with_context(|| "无法解析搜索结果")?;

        Ok(result)
    }

    /// 获取包的所有版本
    ///
    /// # 参数
    /// - `package_name`: 包名称
    ///
    /// # 返回
    /// 版本列表
    pub async fn get_versions(&mut self, package_name: &str) -> Result<Vec<String>> {
        let package = self.get_package(package_name).await?;
        Ok(package.versions.keys().cloned().collect())
    }

    /// 获取最新稳定版本
    ///
    /// # 参数
    /// - `package_name`: 包名称
    ///
    /// # 返回
    /// 最新稳定版本号
    pub async fn get_latest_stable_version(&mut self, package_name: &str) -> Result<String> {
        let package = self.get_package(package_name).await?;

        let mut stable_versions: Vec<String> = package.versions
            .keys()
            .filter(|v| !v.contains("dev") && !v.contains("alpha") && !v.contains("beta") && !v.contains("RC"))
            .cloned()
            .collect();

        stable_versions.sort_by(|a, b| {
            version_compare(b).cmp(&version_compare(a))
        });

        stable_versions
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("没有找到稳定版本"))
    }

    /// 获取指定版本的下载信息
    ///
    /// # 参数
    /// - `package_name`: 包名称
    /// - `version`: 版本号
    ///
    /// # 返回
    /// 下载信息（dist URL 和 checksum）
    pub async fn get_download_info(&mut self, package_name: &str, version: &str) -> Result<(String, String)> {
        let package = self.get_package(package_name).await?;

        let version_info = package.versions
            .get(version)
            .with_context(|| format!("版本 {} 不存在", version))?;

        // 构建 dist URL
        let dist_url = format!(
            "https://repo.packagist.org/{}/{}-{}.zip",
            package_name.replace('/', "/"),
            package_name.replace('/', "-"),
            version_info.source.reference
        );

        Ok((dist_url, version_info.source.reference.clone()))
    }

    /// 清除缓存
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for PackagistClient {
    fn default() -> Self {
        Self::new()
    }
}

/// 版本比较辅助函数
///
/// 将版本字符串转换为可比较的元组
fn version_compare(version: &str) -> (u32, u32, u32, u32) {
    let parts: Vec<&str> = version.split('.').collect();
    let major = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.get(2)
        .and_then(|s| s.split('-').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let build = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch, build)
}

/// 版本约束解析
pub struct VersionConstraint;

impl VersionConstraint {
    /// 检查版本是否满足约束
    ///
    /// # 参数
    /// - `version`: 版本号
    /// - `constraint`: 约束条件（如 ^1.0, >=1.0,<2.0）
    ///
    /// # 返回
    /// 是否满足约束
    pub fn satisfies(version: &str, constraint: &str) -> bool {
        // 简化实现，支持常见的约束格式
        let constraint = constraint.trim();

        // 精确版本
        if !constraint.contains('^') && !constraint.contains('~') && !constraint.contains('>') && !constraint.contains('<') && !constraint.contains('*') {
            return version == constraint;
        }

        // 通配符
        if constraint.ends_with('*') {
            let prefix = constraint.trim_end_matches('*').trim_end_matches('.');
            return version.starts_with(prefix);
        }

        // 插入符约束 (^1.2.3 表示 >=1.2.3,<2.0.0)
        if constraint.starts_with('^') {
            let target = constraint.trim_start_matches('^');
            let parts: Vec<u32> = target.split('.').filter_map(|s| s.parse().ok()).collect();
            if parts.is_empty() {
                return false;
            }

            let major = parts[0];
            let minor = parts.get(1).copied().unwrap_or(0);
            let patch = parts.get(2).copied().unwrap_or(0);

            let v_parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();
            if v_parts.is_empty() {
                return false;
            }

            let v_major = v_parts[0];
            let v_minor = v_parts.get(1).copied().unwrap_or(0);
            let v_patch = v_parts.get(2).copied().unwrap_or(0);

            // ^1.2.3 表示 >=1.2.3,<2.0.0
            // ^0.3 表示 >=0.3.0,<0.4.0
            if major == 0 {
                return v_major == 0 && v_minor == minor && v_patch >= patch;
            } else {
                return v_major == major && (v_minor > minor || (v_minor == minor && v_patch >= patch));
            }
        }

        // 波浪号约束 (~1.2.3 表示 >=1.2.3,<1.3.0)
        if constraint.starts_with('~') {
            let target = constraint.trim_start_matches('~');
            let parts: Vec<u32> = target.split('.').filter_map(|s| s.parse().ok()).collect();
            if parts.len() < 2 {
                return false;
            }

            let major = parts[0];
            let minor = parts[1];
            let patch = parts.get(2).copied().unwrap_or(0);

            let v_parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();
            if v_parts.is_empty() {
                return false;
            }

            let v_major = v_parts[0];
            let v_minor = v_parts.get(1).copied().unwrap_or(0);
            let v_patch = v_parts.get(2).copied().unwrap_or(0);

            return v_major == major && v_minor == minor && v_patch >= patch;
        }

        // 复合约束（如 >=1.0,<2.0）
        if constraint.contains(',') {
            let constraints: Vec<&str> = constraint.split(',').collect();
            return constraints.iter().all(|c| VersionConstraint::satisfies(version, c.trim()));
        }

        // 大于等于
        if constraint.starts_with(">=") {
            let target = constraint.trim_start_matches(">=");
            return version_compare(version) >= version_compare(target);
        }

        // 大于
        if constraint.starts_with('>') {
            let target = constraint.trim_start_matches('>');
            return version_compare(version) > version_compare(target);
        }

        // 小于等于
        if constraint.starts_with("<=") {
            let target = constraint.trim_start_matches("<=");
            return version_compare(version) <= version_compare(target);
        }

        // 小于
        if constraint.starts_with('<') {
            let target = constraint.trim_start_matches('<');
            return version_compare(version) < version_compare(target);
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constraint_exact() {
        assert!(VersionConstraint::satisfies("1.0.0", "1.0.0"));
        assert!(!VersionConstraint::satisfies("1.0.0", "1.0.1"));
    }

    #[test]
    fn test_version_constraint_caret() {
        assert!(VersionConstraint::satisfies("1.2.5", "^1.2.3"));
        assert!(VersionConstraint::satisfies("1.2.3", "^1.2.3"));
        assert!(!VersionConstraint::satisfies("2.0.0", "^1.2.3"));
    }

    #[test]
    fn test_version_constraint_tilde() {
        assert!(VersionConstraint::satisfies("1.2.5", "~1.2.3"));
        assert!(!VersionConstraint::satisfies("1.3.0", "~1.2.3"));
    }

    #[test]
    fn test_version_constraint_wildcard() {
        assert!(VersionConstraint::satisfies("1.2.3", "1.2.*"));
        assert!(VersionConstraint::satisfies("1.2.10", "1.*"));
        assert!(!VersionConstraint::satisfies("2.0.0", "1.*"));
    }

    #[test]
    fn test_version_compare() {
        assert!(version_compare("2.0.0") > version_compare("1.9.9"));
        assert!(version_compare("1.2.3") < version_compare("1.2.4"));
    }
}
