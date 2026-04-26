//! 依赖解析引擎
//!
//! 解析包依赖关系，解决版本冲突
//! 支持语义化版本约束

use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;

use super::packagist::{PackagistClient, VersionConstraint};
use super::manager::PackageSource;

/// 解析后的依赖包
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    /// 包名称
    pub name: String,
    /// 版本号
    pub version: String,
    /// 下载 URL
    pub download_url: String,
    /// 校验和
    pub checksum: String,
    /// 源信息
    pub source: PackageSource,
    /// 依赖
    pub require: HashMap<String, String>,
}

/// 依赖解析器
pub struct DependencyResolver {
    /// Packagist 客户端
    client: PackagistClient,
    /// 已解析的包
    resolved: HashMap<String, ResolvedPackage>,
    /// 解析栈（用于检测循环依赖）
    stack: HashSet<String>,
}

impl DependencyResolver {
    /// 创建新的依赖解析器
    pub fn new() -> Self {
        Self {
            client: PackagistClient::new(),
            resolved: HashMap::new(),
            stack: HashSet::new(),
        }
    }

    /// 解析依赖
    ///
    /// # 参数
    /// - `requirements`: 依赖映射（包名 → 版本约束）
    ///
    /// # 返回
    /// 解析后的包列表
    pub async fn resolve(&mut self, requirements: &HashMap<String, String>) -> Result<Vec<ResolvedPackage>> {
        tracing::info!("开始解析依赖，共 {} 个包", requirements.len());

        // 清空之前的结果
        self.resolved.clear();
        self.stack.clear();

        // 递归解析每个依赖
        for (package_name, constraint) in requirements {
            self.resolve_package(package_name, constraint).await?;
        }

        // 返回解析结果
        Ok(self.resolved.values().cloned().collect())
    }

    /// 解析单个包（使用 Box::pin 处理递归）
    fn resolve_package<'a>(
        &'a mut self,
        package_name: &'a str,
        constraint: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // 检查循环依赖
            if self.stack.contains(package_name) {
                tracing::warn!("检测到循环依赖: {}", package_name);
                return Ok(());
            }

            // 检查是否已解析
            if self.resolved.contains_key(package_name) {
                return Ok(());
            }

            tracing::debug!("解析包: {} ({})", package_name, constraint);

            // 添加到解析栈
            self.stack.insert(package_name.to_string());

            // 获取包信息
            let package_info = self.client.get_package(package_name).await
                .with_context(|| format!("无法获取包信息: {}", package_name))?;

            // 找到满足约束的最新版本
            let best_version = self.find_best_version(&package_info.versions, constraint)?;

            tracing::debug!("选择版本: {}@{}", package_name, best_version.version);

            // 获取下载信息
            let (download_url, checksum) = self.client
                .get_download_info(package_name, &best_version.version)
                .await?;

            // 创建解析后的包
            let resolved = ResolvedPackage {
                name: package_name.to_string(),
                version: best_version.version.clone(),
                download_url,
                checksum: checksum.clone(),
                source: PackageSource {
                    r#type: "dist".to_string(),
                    url: package_info.name.clone(),
                    reference: checksum,
                },
                require: best_version.require.clone(),
            };

            // 添加到已解析列表
            self.resolved.insert(package_name.to_string(), resolved.clone());

            // 收集需要解析的依赖
            let deps: Vec<(String, String)> = best_version.require
                .iter()
                .filter(|(dep_name, _)| {
                    // 跳过 PHP 和扩展依赖
                    !dep_name.starts_with("php") && !dep_name.starts_with("ext-")
                })
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            // 递归解析依赖
            for (dep_name, dep_constraint) in deps {
                self.resolve_package(&dep_name, &dep_constraint).await?;
            }

            // 从解析栈移除
            self.stack.remove(package_name);

            Ok(())
        })
    }

    /// 找到最佳版本
    fn find_best_version(
        &self,
        versions: &HashMap<String, super::packagist::VersionInfo>,
        constraint: &str,
    ) -> Result<super::packagist::VersionInfo> {
        // 过滤满足约束的版本
        let mut matching_versions: Vec<_> = versions
            .values()
            .filter(|v| {
                // 跳过开发版本
                if v.version.contains("dev-") {
                    return false;
                }
                VersionConstraint::satisfies(&v.version, constraint)
            })
            .collect();

        // 按版本号排序（降序）
        matching_versions.sort_by(|a, b| {
            let a_parts = parse_version_parts(&a.version);
            let b_parts = parse_version_parts(&b.version);
            b_parts.cmp(&a_parts)
        });

        matching_versions
            .first()
            .cloned()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("没有找到满足约束 {} 的版本", constraint))
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// 解析版本号为元组
fn parse_version_parts(version: &str) -> (u32, u32, u32, u32) {
    let version = version.trim_start_matches('v');

    let parts: Vec<&str> = version.split('.')
        .flat_map(|p| p.split('-'))
        .collect();

    let major = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    let build = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);

    (major, minor, patch, build)
}

/// 解析依赖（简化入口）
pub async fn resolve(requirements: &HashMap<String, String>) -> Result<Vec<ResolvedPackage>> {
    let mut resolver = DependencyResolver::new();
    resolver.resolve(requirements).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_parts() {
        assert_eq!(parse_version_parts("1.2.3"), (1, 2, 3, 0));
        assert_eq!(parse_version_parts("2.0.0-beta"), (2, 0, 0, 0));
        assert_eq!(parse_version_parts("v1.0.0"), (1, 0, 0, 0));
    }
}
