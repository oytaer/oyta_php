//! AWS S3 存储驱动
//!
//! 使用 rust-s3 SDK 实现基于 Amazon S3 的文件存储操作
//! 支持 AWS S3、MinIO、Cloudflare R2、Wasabi 等兼容 S3 API 的服务

use anyhow::{Context, Result};
use async_trait::async_trait;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use std::collections::HashMap;

use super::traits::{StorageDriver, StorageConfig, FileInfo, StorageError};

#[derive(Debug, Clone)]
pub struct S3Config {
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub bucket: String,
    pub endpoint: Option<String>,
    pub url: String,
    pub path_style: bool,
    pub use_https: bool,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            access_key: String::new(),
            secret_key: String::new(),
            region: "us-east-1".to_string(),
            bucket: String::new(),
            endpoint: None,
            url: String::new(),
            path_style: false,
            use_https: true,
        }
    }
}

pub struct S3Storage {
    config: StorageConfig,
    s3_config: S3Config,
    bucket: Box<Bucket>,
}

impl S3Storage {
    pub fn new(config: StorageConfig, s3_config: S3Config) -> Result<Self> {
        let bucket = Self::create_bucket(&s3_config)?;

        Ok(Self {
            config,
            s3_config,
            bucket,
        })
    }

    fn create_bucket(s3_config: &S3Config) -> Result<Box<Bucket>> {
        let region = if let Some(ref endpoint) = s3_config.endpoint {
            Region::Custom {
                region: s3_config.region.clone(),
                endpoint: endpoint.clone(),
            }
        } else {
            s3_config.region.parse()
                .map_err(|e| StorageError::ConfigError(format!("S3 区域解析失败: {:?}", e)))?
        };

        let credentials = Credentials::new(
            Some(&s3_config.access_key),
            Some(&s3_config.secret_key),
            None,
            None,
            None,
        ).map_err(|e| StorageError::ConfigError(format!("S3 凭证创建失败: {}", e)))?;

        let bucket = Bucket::new(&s3_config.bucket, region, credentials)
            .map_err(|e| StorageError::ConfigError(format!("S3 Bucket 创建失败: {}", e)))?;

        let bucket = if s3_config.path_style {
            bucket.with_path_style()
        } else {
            bucket
        };

        Ok(bucket)
    }

    fn guess_mime_type(path: &str) -> String {
        let extension = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "ico" => "image/x-icon",
            "bmp" => "image/bmp",
            "pdf" => "application/pdf",
            "doc" => "application/msword",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "xls" => "application/vnd.ms-excel",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "txt" => "text/plain",
            "html" | "htm" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "json" => "application/json",
            "xml" => "application/xml",
            "csv" => "text/csv",
            "zip" => "application/zip",
            "mp3" => "audio/mpeg",
            "mp4" => "video/mp4",
            _ => "application/octet-stream",
        }.to_string()
    }
}

#[async_trait]
impl StorageDriver for S3Storage {
    fn driver_name(&self) -> &str {
        "s3"
    }

    async fn exists(&self, path: &str) -> bool {
        self.bucket.head_object(path).await.is_ok()
    }

    async fn get(&self, path: &str) -> Result<Vec<u8>> {
        let response_data = self.bucket.get_object(path).await
            .map_err(|e| StorageError::ReadError(format!("S3 获取对象失败: {} - {}", path, e)))?;

        Ok(response_data.to_vec())
    }

    async fn put(&self, path: &str, contents: &[u8]) -> Result<bool> {
        let content_type = Self::guess_mime_type(path);

        self.bucket.put_object_with_content_type(path, contents, &content_type).await
            .map_err(|e| StorageError::WriteError(format!("S3 上传对象失败: {} - {}", path, e)))?;

        Ok(true)
    }

    async fn append(&self, path: &str, contents: &[u8]) -> Result<bool> {
        let mut existing = if self.exists(path).await {
            self.get(path).await?
        } else {
            Vec::new()
        };

        existing.extend_from_slice(contents);
        self.put(path, &existing).await
    }

    async fn prepend(&self, path: &str, contents: &[u8]) -> Result<bool> {
        let mut new_contents = contents.to_vec();

        if self.exists(path).await {
            let existing = self.get(path).await?;
            new_contents.extend_from_slice(&existing);
        }

        self.put(path, &new_contents).await
    }

    async fn delete(&self, path: &str) -> Result<bool> {
        self.bucket.delete_object(path).await
            .map_err(|e| StorageError::DeleteError(format!("S3 删除对象失败: {} - {}", path, e)))?;

        Ok(true)
    }

    async fn copy(&self, from: &str, to: &str) -> Result<bool> {
        let copy_source = format!("{}/{}", self.s3_config.bucket, from);

        self.bucket.copy_object_internal(&copy_source, to).await
            .map_err(|e| StorageError::WriteError(format!("S3 复制对象失败: {} -> {} - {}", from, to, e)))?;

        Ok(true)
    }

    async fn move_file(&self, from: &str, to: &str) -> Result<bool> {
        self.copy(from, to).await?;
        self.delete(from).await
    }

    async fn size(&self, path: &str) -> Result<u64> {
        let (head, _status_code) = self.bucket.head_object(path).await
            .map_err(|e| StorageError::FileNotFound(format!("S3 获取对象信息失败: {} - {}", path, e)))?;

        Ok(head.content_length.unwrap_or(0) as u64)
    }

    async fn last_modified(&self, path: &str) -> Result<u64> {
        let (head, _status_code) = self.bucket.head_object(path).await
            .map_err(|e| StorageError::FileNotFound(format!("S3 获取对象信息失败: {} - {}", path, e)))?;

        let last_modified = head.last_modified
            .and_then(|dt| dt.parse::<chrono::DateTime<chrono::Utc>>().ok())
            .map(|dt| dt.timestamp() as u64)
            .unwrap_or(0);

        Ok(last_modified)
    }

    async fn mime_type(&self, path: &str) -> Result<String> {
        let (head, _status_code) = self.bucket.head_object(path).await
            .map_err(|e| StorageError::FileNotFound(format!("S3 获取对象信息失败: {} - {}", path, e)))?;

        Ok(head.content_type.unwrap_or_else(|| Self::guess_mime_type(path)))
    }

    async fn url(&self, path: &str) -> Result<String> {
        if !self.s3_config.url.is_empty() {
            return Ok(format!("{}/{}", self.s3_config.url, path));
        }

        let base_url = self.bucket.url();
        Ok(format!("{}/{}", base_url, path))
    }

    fn full_path(&self, path: &str) -> String {
        format!("s3://{}/{}", self.s3_config.bucket, path)
    }

    async fn files(&self, directory: &str) -> Result<Vec<String>> {
        let prefix = if directory.ends_with('/') {
            directory.to_string()
        } else {
            format!("{}/", directory)
        };

        let results = self.bucket.list(prefix.clone(), Some("/".to_string())).await
            .map_err(|e| StorageError::ReadError(format!("S3 列出文件失败: {} - {}", directory, e)))?;

        let mut files = Vec::new();

        for result in results {
            for object in result.contents {
                if !object.key.ends_with('/') {
                    files.push(object.key);
                }
            }
        }

        Ok(files)
    }

    async fn all_files(&self, directory: &str) -> Result<Vec<String>> {
        let prefix = if directory.ends_with('/') {
            directory.to_string()
        } else {
            format!("{}/", directory)
        };

        let results = self.bucket.list(prefix.clone(), None).await
            .map_err(|e| StorageError::ReadError(format!("S3 列出所有文件失败: {} - {}", directory, e)))?;

        let mut files = Vec::new();

        for result in results {
            for object in result.contents {
                if !object.key.ends_with('/') {
                    files.push(object.key);
                }
            }
        }

        Ok(files)
    }

    async fn directories(&self, directory: &str) -> Result<Vec<String>> {
        let prefix = if directory.ends_with('/') {
            directory.to_string()
        } else {
            format!("{}/", directory)
        };

        let results = self.bucket.list(prefix.clone(), Some("/".to_string())).await
            .map_err(|e| StorageError::ReadError(format!("S3 列出目录失败: {} - {}", directory, e)))?;

        let mut dirs = Vec::new();

        for result in results {
            if let Some(common_prefixes) = result.common_prefixes {
                for cp in common_prefixes {
                    dirs.push(cp.prefix);
                }
            }
        }

        Ok(dirs)
    }

    async fn make_directory(&self, _path: &str) -> Result<bool> {
        Ok(true)
    }

    async fn delete_directory(&self, directory: &str) -> Result<bool> {
        let files = self.all_files(directory).await?;

        for file in &files {
            self.delete(file).await?;
        }

        Ok(true)
    }

    async fn clean_directory(&self, directory: &str) -> Result<bool> {
        self.delete_directory(directory).await
    }

    async fn get_file_info(&self, path: &str) -> Result<FileInfo> {
        let (head, _status_code) = self.bucket.head_object(path).await
            .map_err(|e| StorageError::FileNotFound(format!("S3 获取对象信息失败: {} - {}", path, e)))?;

        let size = head.content_length.unwrap_or(0) as u64;

        let last_modified = head.last_modified
            .and_then(|dt| dt.parse::<chrono::DateTime<chrono::Utc>>().ok())
            .map(|dt| dt.timestamp() as u64)
            .unwrap_or(0);

        let mime_type = head.content_type.unwrap_or_else(|| Self::guess_mime_type(path));

        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        Ok(FileInfo {
            path: path.to_string(),
            size,
            last_modified,
            mime_type,
            extension: self.extension(path),
            filename,
            is_dir: false,
            visibility: "private".to_string(),
        })
    }

    async fn temporary_url(&self, path: &str, expiration: u64) -> Result<String> {
        let url = self.bucket.presign_get(
            path,
            expiration as u32,
            None,
        ).await
            .map_err(|e| StorageError::ReadError(format!("S3 生成临时 URL 失败: {} - {}", path, e)))?;

        Ok(url)
    }

    async fn set_visibility(&self, path: &str, visibility: &str) -> Result<bool> {
        let acl = if visibility == "public" {
            "public-read"
        } else {
            "private"
        };

        self.bucket.put_object_with_content_type(
            path,
            &[],
            acl,
        ).await
            .map_err(|e| StorageError::WriteError(format!("S3 设置可见性失败: {} - {}", path, e)))?;

        Ok(true)
    }

    async fn get_visibility(&self, path: &str) -> Result<String> {
        let (head, _status_code) = self.bucket.head_object(path).await
            .map_err(|e| StorageError::ReadError(format!("S3 获取可见性失败: {} - {}", path, e)))?;

        Ok(head.content_type.unwrap_or_else(|| "private".to_string()))
    }
}
