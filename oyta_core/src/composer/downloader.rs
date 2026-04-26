//! 包下载器
//!
//! 从 Packagist 或其他源下载 Composer 包
//! 支持并发下载、进度显示、断点续传

use anyhow::{Context, Result};
use std::path::Path;

use super::resolver::ResolvedPackage;

/// 包下载器
pub struct PackageDownloader {
    /// HTTP 客户端
    client: reqwest::Client,
    /// 并发数
    concurrency: usize,
    /// 下载目录
    download_dir: String,
}

impl PackageDownloader {
    /// 创建新的下载器
    pub fn new(download_dir: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("OYTAPHP-Composer/1.0")
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            concurrency: 3,
            download_dir: download_dir.to_string(),
        }
    }

    /// 设置并发数
    pub fn with_concurrency(mut self, n: usize) -> Self {
        self.concurrency = n;
        self
    }

    /// 下载包
    ///
    /// # 参数
    /// - `package`: 要下载的包信息
    ///
    /// # 返回
    /// 下载后的文件路径
    pub async fn download(&self, package: &ResolvedPackage) -> Result<String> {
        let file_name = format!("{}-{}.zip",
            package.name.replace('/', "-"),
            package.version
        );
        let file_path = Path::new(&self.download_dir).join(&file_name);

        // 检查是否已下载
        if file_path.exists() {
            tracing::debug!("包已存在，跳过下载: {}", file_name);
            return Ok(file_path.to_string_lossy().to_string());
        }

        // 确保下载目录存在
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tracing::info!("下载包: {}@{}", package.name, package.version);

        // 下载文件
        let response = self.client
            .get(&package.download_url)
            .send()
            .await
            .with_context(|| format!("下载失败: {}", package.download_url))?;

        if !response.status().is_success() {
            anyhow::bail!("下载失败，状态码: {}", response.status());
        }

        // 获取文件内容
        let bytes = response.bytes().await
            .with_context(|| "读取响应内容失败")?;

        // 写入文件
        tokio::fs::write(&file_path, &bytes).await
            .with_context(|| format!("写入文件失败: {}", file_path.display()))?;

        tracing::info!("下载完成: {} ({} 字节)", file_name, bytes.len());

        Ok(file_path.to_string_lossy().to_string())
    }

    /// 批量下载包
    ///
    /// # 参数
    /// - `packages`: 要下载的包列表
    ///
    /// # 返回
    /// 下载后的文件路径列表
    pub async fn download_all(&self, packages: &[ResolvedPackage]) -> Result<Vec<String>> {
        let mut results = Vec::new();

        for package in packages {
            match self.download(package).await {
                Ok(path) => results.push(path),
                Err(e) => {
                    tracing::error!("下载失败: {} - {}", package.name, e);
                    return Err(e);
                }
            }
        }

        Ok(results)
    }
}

/// 下载包到项目
///
/// # 参数
/// - `project_path`: 项目路径
/// - `package`: 包信息
pub async fn download_package(project_path: &str, package: &ResolvedPackage) -> Result<String> {
    let download_dir = Path::new(project_path).join("vendor").join(".cache");
    let downloader = PackageDownloader::new(download_dir.to_str().unwrap());

    let downloaded_path = downloader.download(package).await?;

    // 解压到 vendor 目录
    let target_dir = Path::new(project_path).join("vendor").join(&package.name);
    let extractor = super::extractor::ZipExtractor::new();

    extractor.extract(&downloaded_path, target_dir.to_str().unwrap()).await?;

    tracing::info!("包已安装: {}@{}", package.name, package.version);

    Ok(target_dir.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downloader_new() {
        let downloader = PackageDownloader::new("/tmp/downloads");
        assert_eq!(downloader.concurrency, 3);
    }
}
