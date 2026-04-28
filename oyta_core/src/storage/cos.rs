//! 腾讯云 COS 存储驱动
//!
//! 使用 qcos SDK 实现基于腾讯云对象存储服务（COS）的文件存储操作
//! SDK 不支持的功能（HEAD、复制、ACL）通过 COS REST API 手动实现

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::time::SystemTime;

use qcos::client::Client;
use qcos::request::{ErrNo, Response};

use super::traits::{StorageDriver, StorageConfig, FileInfo, StorageError};

#[derive(Debug, Clone)]
pub struct CosConfig {
    pub secret_id: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
    pub custom_domain: Option<String>,
    pub use_https: bool,
}

impl Default for CosConfig {
    fn default() -> Self {
        Self {
            secret_id: String::new(),
            secret_key: String::new(),
            bucket: String::new(),
            region: "ap-guangzhou".to_string(),
            custom_domain: None,
            use_https: true,
        }
    }
}

pub struct CosStorage {
    config: StorageConfig,
    cos_config: CosConfig,
    client: Client,
    http_client: reqwest::Client,
    host: String,
}

impl CosStorage {
    pub fn new(config: StorageConfig, cos_config: CosConfig) -> Result<Self> {
        let client = Client::new(
            &cos_config.secret_id,
            &cos_config.secret_key,
            &cos_config.bucket,
            &cos_config.region,
        );

        let host = if let Some(ref domain) = cos_config.custom_domain {
            domain.clone()
        } else {
            format!("{}.cos.{}.myqcloud.com", cos_config.bucket, cos_config.region)
        };

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Ok(Self {
            config,
            cos_config,
            client,
            http_client,
            host,
        })
    }

    fn get_host(&self) -> &str {
        &self.host
    }

    fn get_object_url(&self, path: &str) -> String {
        let normalized_path = path.trim_start_matches('/');
        let scheme = if self.cos_config.use_https { "https" } else { "http" };
        format!("{}://{}/{}", scheme, self.host, normalized_path)
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
            "pdf" => "application/pdf",
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

    fn check_response(resp: Response, context: &str) -> Result<Response> {
        if resp.error_no != ErrNo::SUCCESS {
            let msg = if resp.error_message.is_empty() {
                String::from_utf8_lossy(&resp.result).to_string()
            } else {
                resp.error_message.clone()
            };
            return Err(StorageError::ReadError(format!("{}: {}", context, msg)).into());
        }
        Ok(resp)
    }
}

#[async_trait]
impl StorageDriver for CosStorage {
    fn driver_name(&self) -> &str {
        "cos"
    }

    async fn exists(&self, path: &str) -> bool {
        let size = self.client.get_object_size(path).await;
        size >= 0
    }

    async fn get(&self, path: &str) -> Result<Vec<u8>> {
        let resp = self.client.get_object_binary(path, None).await;
        let resp = Self::check_response(resp, &format!("COS 获取对象失败: {}", path))?;
        Ok(resp.result)
    }

    async fn put(&self, path: &str, contents: &[u8]) -> Result<bool> {
        let content_type: mime::Mime = Self::guess_mime_type(path)
            .parse()
            .unwrap_or(mime::APPLICATION_OCTET_STREAM);

        let resp = self.client.put_object_binary(
            contents.to_vec(),
            path,
            Some(content_type),
            None,
        ).await;

        if resp.error_no != ErrNo::SUCCESS {
            let msg = if resp.error_message.is_empty() {
                String::from_utf8_lossy(&resp.result).to_string()
            } else {
                resp.error_message.clone()
            };
            return Err(StorageError::WriteError(format!("COS 上传对象失败: {} - {}", path, msg)).into());
        }

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
        let resp = self.client.delete_object(path).await;
        if resp.error_no != ErrNo::SUCCESS {
            let msg = String::from_utf8_lossy(&resp.result).to_string();
            if msg.contains("404") || msg.contains("NoSuchKey") {
                return Ok(true);
            }
            return Err(StorageError::DeleteError(format!("COS 删除对象失败: {} - {}", path, msg)).into());
        }

        Ok(true)
    }

    async fn copy(&self, from: &str, to: &str) -> Result<bool> {
        let to_url = self.get_object_url(to);
        let url_path = self.client.get_path_from_object_key(to);
        let host = self.get_host();

        let copy_source = format!("{}.cos.{}.myqcloud.com/{}",
            self.cos_config.bucket,
            self.cos_config.region,
            from.trim_start_matches('/')
        );

        let headers = self.client.get_headers_with_auth(
            "put",
            &url_path,
            None,
            None,
            None,
        );

        let response = self.http_client
            .put(&to_url)
            .headers(headers)
            .header("x-cos-copy-source", &copy_source)
            .send()
            .await
            .with_context(|| format!("无法复制 COS 对象: {} -> {}", from, to))?;

        if !response.status().is_success() {
            return Err(StorageError::WriteError(format!("COS 复制失败: {}", response.status())).into());
        }

        Ok(true)
    }

    async fn move_file(&self, from: &str, to: &str) -> Result<bool> {
        self.copy(from, to).await?;
        self.delete(from).await
    }

    async fn size(&self, path: &str) -> Result<u64> {
        let size = self.client.get_object_size(path).await;
        if size < 0 {
            return Err(StorageError::FileNotFound(path.to_string()).into());
        }
        Ok(size as u64)
    }

    async fn last_modified(&self, path: &str) -> Result<u64> {
        let url_path = self.client.get_path_from_object_key(path);
        let url = self.client.get_full_url_from_path(&url_path);
        let headers = self.client.get_headers_with_auth("head", &url_path, None, None, None);

        let response = self.http_client
            .head(&url)
            .headers(headers)
            .send()
            .await
            .with_context(|| format!("无法获取 COS 对象信息: {}", path))?;

        if !response.status().is_success() {
            return Err(StorageError::FileNotFound(path.to_string()).into());
        }

        let last_modified = response.headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| chrono::DateTime::parse_from_rfc2822(v).ok())
            .map(|dt| dt.timestamp() as u64)
            .unwrap_or(0);

        Ok(last_modified)
    }

    async fn mime_type(&self, path: &str) -> Result<String> {
        let url_path = self.client.get_path_from_object_key(path);
        let url = self.client.get_full_url_from_path(&url_path);
        let headers = self.client.get_headers_with_auth("head", &url_path, None, None, None);

        let response = self.http_client
            .head(&url)
            .headers(headers)
            .send()
            .await
            .with_context(|| format!("无法获取 COS 对象 MIME 类型: {}", path))?;

        if !response.status().is_success() {
            return Ok(Self::guess_mime_type(path));
        }

        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        Ok(content_type)
    }

    async fn url(&self, path: &str) -> Result<String> {
        Ok(self.get_object_url(path))
    }

    fn full_path(&self, path: &str) -> String {
        format!("cos://{}/{}", self.cos_config.bucket, path.trim_start_matches('/'))
    }

    async fn files(&self, directory: &str) -> Result<Vec<String>> {
        let prefix = if directory.ends_with('/') {
            directory.to_string()
        } else {
            format!("{}/", directory)
        };

        let resp = self.client.list_objects(&prefix, "/", "", "", 1000).await;
        let resp = Self::check_response(resp, &format!("COS 列出文件失败: {}", directory))?;

        let body = String::from_utf8_lossy(&resp.result).to_string();
        let mut files = Vec::new();

        for key in body.split("<Key>").skip(1) {
            if let Some(end) = key.find("</Key>") {
                let file_key = &key[..end];
                if !file_key.ends_with('/') {
                    files.push(file_key.to_string());
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

        let resp = self.client.list_objects(&prefix, "", "", "", 1000).await;
        let resp = Self::check_response(resp, &format!("COS 列出所有文件失败: {}", directory))?;

        let body = String::from_utf8_lossy(&resp.result).to_string();
        let mut files = Vec::new();

        for key in body.split("<Key>").skip(1) {
            if let Some(end) = key.find("</Key>") {
                let file_key = &key[..end];
                if !file_key.ends_with('/') {
                    files.push(file_key.to_string());
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

        let resp = self.client.list_objects(&prefix, "/", "", "", 1000).await;
        let resp = Self::check_response(resp, &format!("COS 列出目录失败: {}", directory))?;

        let body = String::from_utf8_lossy(&resp.result).to_string();
        let mut dirs = Vec::new();

        for cp in body.split("<CommonPrefixes>").skip(1) {
            if let Some(prefix_start) = cp.find("<Prefix>") {
                let remaining = &cp[prefix_start + 8..];
                if let Some(end) = remaining.find("</Prefix>") {
                    let dir_prefix = &remaining[..end];
                    if dir_prefix.ends_with('/') {
                        dirs.push(dir_prefix.to_string());
                    }
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
        let size = self.size(path).await?;

        let url_path = self.client.get_path_from_object_key(path);
        let url = self.client.get_full_url_from_path(&url_path);
        let headers = self.client.get_headers_with_auth("head", &url_path, None, None, None);

        let response = self.http_client
            .head(&url)
            .headers(headers)
            .send()
            .await
            .with_context(|| format!("无法获取 COS 对象信息: {}", path))?;

        let last_modified = response.headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| chrono::DateTime::parse_from_rfc2822(v).ok())
            .map(|dt| dt.timestamp() as u64)
            .unwrap_or(0);

        let mime_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

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
        let url = self.client.get_presigned_download_url(path, expiration as u32);
        Ok(url)
    }

    async fn set_visibility(&self, path: &str, visibility: &str) -> Result<bool> {
        use qcos::acl::{AclHeader, ObjectAcl};

        let mut acl_header = AclHeader::new();
        if visibility == "public" {
            acl_header.insert_object_x_cos_acl(ObjectAcl::PublicRead);
        } else {
            acl_header.insert_object_x_cos_acl(ObjectAcl::PRIVATE);
        }

        let url_path = self.client.get_path_from_object_key(path);
        let url = self.client.get_full_url_from_path(&url_path);
        let headers = self.client.get_headers_with_auth(
            "put",
            &url_path,
            Some(acl_header),
            None,
            None,
        );

        let response = self.http_client
            .put(&url)
            .headers(headers)
            .send()
            .await
            .with_context(|| format!("无法设置 COS 对象可见性: {}", path))?;

        Ok(response.status().is_success())
    }

    async fn get_visibility(&self, path: &str) -> Result<String> {
        let url_path = self.client.get_path_from_object_key(path);
        let mut query = std::collections::HashMap::new();
        query.insert("acl".to_string(), String::new());

        let url = format!("{}?acl", self.client.get_full_url_from_path(&url_path));
        let headers = self.client.get_headers_with_auth(
            "get",
            &format!("{}?acl", url_path),
            None,
            None,
            Some(query),
        );

        let response = self.http_client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .with_context(|| format!("无法获取 COS 对象可见性: {}", path))?;

        if !response.status().is_success() {
            return Ok("private".to_string());
        }

        let body = response.text().await.unwrap_or_default();
        if body.contains("public-read") {
            Ok("public".to_string())
        } else {
            Ok("private".to_string())
        }
    }
}
