//! ZIP/TAR 解压器
//!
//! 解压下载的 Composer 包
//! 支持 ZIP 和 TAR.GZ 格式

use anyhow::{Context, Result};
use std::fs::File;
use std::path::Path;

/// ZIP 解压器
pub struct ZipExtractor {
    /// 是否覆盖已存在的文件
    overwrite: bool,
}

impl ZipExtractor {
    /// 创建新的 ZIP 解压器
    pub fn new() -> Self {
        Self {
            overwrite: true,
        }
    }

    /// 设置是否覆盖已存在的文件
    pub fn overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }

    /// 解压 ZIP 文件
    ///
    /// # 参数
    /// - `zip_path`: ZIP 文件路径
    /// - `target_dir`: 目标目录
    pub async fn extract(&self, zip_path: &str, target_dir: &str) -> Result<()> {
        tracing::debug!("解压文件: {} → {}", zip_path, target_dir);

        // 确保目标目录存在
        tokio::fs::create_dir_all(target_dir).await?;

        // 使用 zip 库解压
        let file = File::open(zip_path)
            .with_context(|| format!("无法打开 ZIP 文件: {}", zip_path))?;

        let mut archive = zip::ZipArchive::new(file)
            .with_context(|| format!("无效的 ZIP 文件: {}", zip_path))?;

        let target_path = Path::new(target_dir);

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;

            let outpath = match file.enclosed_name() {
                Some(path) => target_path.join(path),
                None => continue,
            };

            // 跳过 macOS 特殊文件
            if outpath.to_string_lossy().contains("__MACOSX") {
                continue;
            }

            if file.name().ends_with('/') {
                // 创建目录
                tokio::fs::create_dir_all(&outpath).await?;
            } else {
                // 创建父目录
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        tokio::fs::create_dir_all(p).await?;
                    }
                }

                // 写入文件
                let mut outfile = File::create(&outpath)
                    .with_context(|| format!("无法创建文件: {}", outpath.display()))?;

                std::io::copy(&mut file, &mut outfile)
                    .with_context(|| format!("写入文件失败: {}", outpath.display()))?;
            }
        }

        tracing::debug!("解压完成: {} 个文件", archive.len());

        Ok(())
    }
}

impl Default for ZipExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// TAR.GZ 解压器
pub struct TarGzExtractor {
    /// 是否覆盖已存在的文件
    overwrite: bool,
}

impl TarGzExtractor {
    /// 创建新的 TAR.GZ 解压器
    pub fn new() -> Self {
        Self {
            overwrite: true,
        }
    }

    /// 设置是否覆盖已存在的文件
    pub fn overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }

    /// 解压 TAR.GZ 文件
    ///
    /// # 参数
    /// - `tar_path`: TAR.GZ 文件路径
    /// - `target_dir`: 目标目录
    pub async fn extract(&self, tar_path: &str, target_dir: &str) -> Result<()> {
        tracing::debug!("解压 TAR.GZ 文件: {} → {}", tar_path, target_dir);

        // 确保目标目录存在
        tokio::fs::create_dir_all(target_dir).await?;

        let file = File::open(tar_path)
            .with_context(|| format!("无法打开 TAR.GZ 文件: {}", tar_path))?;

        let gz_decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(gz_decoder);

        let target_path = Path::new(target_dir);

        archive.unpack(target_path)
            .with_context(|| format!("解压 TAR.GZ 失败: {}", tar_path))?;

        tracing::debug!("TAR.GZ 解压完成");

        Ok(())
    }
}

impl Default for TarGzExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// 自动检测格式并解压
///
/// # 参数
/// - `file_path`: 压缩文件路径
/// - `target_dir`: 目标目录
pub async fn extract_auto(file_path: &str, target_dir: &str) -> Result<()> {
    let path = Path::new(file_path);
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match extension {
        "zip" => {
            ZipExtractor::new().extract(file_path, target_dir).await
        }
        "gz" => {
            TarGzExtractor::new().extract(file_path, target_dir).await
        }
        _ => {
            anyhow::bail!("不支持的压缩格式: {}", extension);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zip_extractor_new() {
        let extractor = ZipExtractor::new();
        assert!(extractor.overwrite);
    }

    #[test]
    fn test_tar_gz_extractor_new() {
        let extractor = TarGzExtractor::new();
        assert!(extractor.overwrite);
    }
}
