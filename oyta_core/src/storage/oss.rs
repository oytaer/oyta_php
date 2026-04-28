//! 阿里云 OSS 存储驱动
//!
//! 使用 aliyun-oss-rust-sdk 实现基于阿里云对象存储服务（OSS）的文件存储操作
//! SDK 不支持的功能（列表、复制、ACL）通过 OSS REST API 手动实现

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::time::SystemTime;

use aliyun_oss_rust_sdk::oss::OSS;
use aliyun_oss_rust_sdk::request::RequestBuilder;
use aliyun_oss_rust_sdk::url::UrlApi;
use aliyun_oss_rust_sdk::metadata::ObjectMetadata;

use super::traits::{StorageDriver, StorageConfig, FileInfo, StorageError};

#[derive(Debug, Clone)]
pub struct OssConfig {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub bucket: String,
    pub endpoint: String,
    pub custom_domain: Option<String>,
    pub use_https: bool,
    pub cname: bool,
}

impl Default for OssConfig {
    fn default() -> Self {
        Self {
            access_key_id: String::new(),
            access_key_secret: String::new(),
            bucket: String::new(),
            endpoint: "oss-cn-hangzhou.aliyuncs.com".to_string(),
            custom_domain: None,
            use_https: true,
            cname: false,
        }
    }
}

pub struct OssStorage {
    config: StorageConfig,
    oss_config: OssConfig,
    oss: OSS,
    client: reqwest::Client,
}

impl OssStorage {
    pub fn new(config: StorageConfig, oss_config: OssConfig) -> Result<Self> {
        let endpoint = if oss_config.endpoint.starts_with("http") {
            oss_config.endpoint.clone()
        } else {
            let scheme = if oss_config.use_https { "https" } else { "http" };
            format!("{}://{}", scheme, oss_config.endpoint)
        };

        let oss = OSS::new(
            &oss_config.access_key_id,
            &oss_config.access_key_secret,
            &endpoint,
            &oss_config.bucket,
        );

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Ok(Self {
            config,
            oss_config,
            oss,
            client,
        })
    }

    fn get_host(&self) -> String {
        if let Some(ref domain) = self.oss_config.custom_domain {
            domain.clone()
        } else if self.oss_config.cname {
            self.oss_config.endpoint.clone()
        } else {
            format!("{}.{}", self.oss_config.bucket, self.oss_config.endpoint)
        }
    }

    fn get_object_url(&self, path: &str) -> String {
        let normalized_path = path.trim_start_matches('/');
        let scheme = if self.oss_config.use_https { "https" } else { "http" };
        format!("{}://{}/{}", scheme, self.get_host(), normalized_path)
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

    fn build_resource(&self, path: &str) -> String {
        format!("/{}/{}", self.oss_config.bucket, path.trim_start_matches('/'))
    }

    fn sign(
        &self,
        method: &str,
        resource: &str,
        headers: &[(String, String)],
        expires: Option<i64>,
    ) -> String {
        use hmac::{Hmac, Mac};
        use sha1::Sha1;
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

        let mut string_to_sign = format!("{}\n\n\n", method);

        if let Some(exp) = expires {
            string_to_sign = format!("{}\n\n\n{}\n", method, exp);
        }

        let mut oss_headers: Vec<_> = headers
            .iter()
            .filter(|(k, _)| k.to_lowercase().starts_with("x-oss-"))
            .collect();
        oss_headers.sort_by(|a, b| a.0.cmp(&b.0));

        if !oss_headers.is_empty() {
            for (k, v) in oss_headers {
                string_to_sign.push_str(&format!("{}:{}\n", k.to_lowercase(), v));
            }
        }

        string_to_sign.push_str(resource);

        let mut mac = Hmac::<Sha1>::new_from_slice(self.oss_config.access_key_secret.as_bytes())
            .unwrap();
        mac.update(string_to_sign.as_bytes());
        let result = mac.finalize();
        BASE64.encode(result.into_bytes())
    }

    fn get_date() -> String {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();

        let datetime = chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now());

        datetime.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
    }
}

#[async_trait]
impl StorageDriver for OssStorage {
    fn driver_name(&self) -> &str {
        "oss"
    }

    async fn exists(&self, path: &str) -> bool {
        let builder = RequestBuilder::new();
        self.oss.get_object_metadata(path, builder).await.is_ok()
    }

    async fn get(&self, path: &str) -> Result<Vec<u8>> {
        let builder = RequestBuilder::new();
        self.oss.get_object(path, builder).await
            .map_err(|e| StorageError::ReadError(format!("OSS 获取对象失败: {} - {}", path, e)).into())
    }

    async fn put(&self, path: &str, contents: &[u8]) -> Result<bool> {
        let content_type = Self::guess_mime_type(path);
        let builder = RequestBuilder::new()
            .with_content_type(&content_type);

        self.oss.pub_object_from_buffer(path, contents, builder).await
            .map_err(|e| anyhow::anyhow!("OSS 上传对象失败: {} - {}", path, e))?;

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
        let builder = RequestBuilder::new();
        self.oss.delete_object(path, builder).await
            .map_err(|e| anyhow::anyhow!("OSS 删除对象失败: {} - {}", path, e))?;

        Ok(true)
    }

    async fn copy(&self, from: &str, to: &str) -> Result<bool> {
        let to_url = self.get_object_url(to);
        let resource = self.build_resource(to);
        let date = Self::get_date();

        let copy_source = format!("/{}/{}", self.oss_config.bucket, from.trim_start_matches('/'));

        let headers = vec![
            ("x-oss-copy-source".to_string(), copy_source.clone()),
        ];

        let signature = self.sign("PUT", &resource, &headers, None);
        let authorization = format!("OSS {}:{}", self.oss_config.access_key_id, signature);

        let response = self.client
            .put(&to_url)
            .header("Date", &date)
            .header("Authorization", &authorization)
            .header("x-oss-copy-source", &copy_source)
            .send()
            .await
            .with_context(|| format!("无法复制 OSS 对象: {} -> {}", from, to))?;

        if !response.status().is_success() {
            return Err(StorageError::WriteError(format!("OSS 复制失败: {}", response.status())).into());
        }

        Ok(true)
    }

    async fn move_file(&self, from: &str, to: &str) -> Result<bool> {
        self.copy(from, to).await?;
        self.delete(from).await
    }

    async fn size(&self, path: &str) -> Result<u64> {
        let builder = RequestBuilder::new();
        let metadata = self.oss.get_object_metadata(path, builder).await
            .map_err(|e| StorageError::FileNotFound(format!("OSS 获取对象信息失败: {} - {}", path, e)))?;

        let size = metadata.content_length()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        Ok(size)
    }

    async fn last_modified(&self, path: &str) -> Result<u64> {
        let builder = RequestBuilder::new();
        let metadata = self.oss.get_object_metadata(path, builder).await
            .map_err(|e| StorageError::FileNotFound(format!("OSS 获取对象信息失败: {} - {}", path, e)))?;

        let last_modified = metadata.last_modified()
            .map(|dt| dt.timestamp() as u64)
            .unwrap_or(0);

        Ok(last_modified)
    }

    async fn mime_type(&self, path: &str) -> Result<String> {
        let builder = RequestBuilder::new();
        let metadata = self.oss.get_object_metadata(path, builder).await
            .map_err(|e| StorageError::FileNotFound(format!("OSS 获取对象信息失败: {} - {}", path, e)))?;

        Ok(metadata.content_type().unwrap_or_else(|| Self::guess_mime_type(path)))
    }

    async fn url(&self, path: &str) -> Result<String> {
        Ok(self.get_object_url(path))
    }

    fn full_path(&self, path: &str) -> String {
        format!("oss://{}/{}", self.oss_config.bucket, path.trim_start_matches('/'))
    }

    async fn files(&self, directory: &str) -> Result<Vec<String>> {
        let prefix = if directory.ends_with('/') {
            directory.to_string()
        } else {
            format!("{}/", directory)
        };

        let host = self.get_host();
        let scheme = if self.oss_config.use_https { "https" } else { "http" };
        let url = format!("{}://{}?prefix={}&delimiter=/&max-keys=1000",
            scheme, host, urlencoding::encode(&prefix));

        let resource = format!("/{}/?prefix={}&delimiter=/&max-keys=1000",
            self.oss_config.bucket, urlencoding::encode(&prefix));
        let date = Self::get_date();
        let signature = self.sign("GET", &resource, &[], None);
        let authorization = format!("OSS {}:{}", self.oss_config.access_key_id, signature);

        let response = self.client
            .get(&url)
            .header("Date", &date)
            .header("Authorization", &authorization)
            .send()
            .await
            .with_context(|| format!("无法列出 OSS 文件: {}", directory))?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let body = response.text().await.unwrap_or_default();
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

        let host = self.get_host();
        let scheme = if self.oss_config.use_https { "https" } else { "http" };
        let url = format!("{}://{}?prefix={}&max-keys=1000",
            scheme, host, urlencoding::encode(&prefix));

        let resource = format!("/{}/?prefix={}&max-keys=1000",
            self.oss_config.bucket, urlencoding::encode(&prefix));
        let date = Self::get_date();
        let signature = self.sign("GET", &resource, &[], None);
        let authorization = format!("OSS {}:{}", self.oss_config.access_key_id, signature);

        let response = self.client
            .get(&url)
            .header("Date", &date)
            .header("Authorization", &authorization)
            .send()
            .await
            .with_context(|| format!("无法列出 OSS 所有文件: {}", directory))?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let body = response.text().await.unwrap_or_default();
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

        let host = self.get_host();
        let scheme = if self.oss_config.use_https { "https" } else { "http" };
        let url = format!("{}://{}?prefix={}&delimiter=/&max-keys=1000",
            scheme, host, urlencoding::encode(&prefix));

        let resource = format!("/{}/?prefix={}&delimiter=/&max-keys=1000",
            self.oss_config.bucket, urlencoding::encode(&prefix));
        let date = Self::get_date();
        let signature = self.sign("GET", &resource, &[], None);
        let authorization = format!("OSS {}:{}", self.oss_config.access_key_id, signature);

        let response = self.client
            .get(&url)
            .header("Date", &date)
            .header("Authorization", &authorization)
            .send()
            .await
            .with_context(|| format!("无法列出 OSS 目录: {}", directory))?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let body = response.text().await.unwrap_or_default();
        let mut dirs = Vec::new();

        for prefix_tag in body.split("<Prefix>").skip(1) {
            if let Some(end) = prefix_tag.find("</Prefix>") {
                let dir_prefix = &prefix_tag[..end];
                if dir_prefix.ends_with('/') && dir_prefix != prefix {
                    dirs.push(dir_prefix.to_string());
                }
            }
        }

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
        let builder = RequestBuilder::new();
        let metadata = self.oss.get_object_metadata(path, builder).await
            .map_err(|e| StorageError::FileNotFound(format!("OSS 获取对象信息失败: {} - {}", path, e)))?;

        let size = metadata.content_length()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let last_modified = metadata.last_modified()
            .map(|dt| dt.timestamp() as u64)
            .unwrap_or(0);

        let mime_type = metadata.content_type()
            .unwrap_or_else(|| Self::guess_mime_type(path));

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
        let builder = RequestBuilder::new()
            .with_expire(expiration as i64);

        if let Some(ref cdn) = self.oss_config.custom_domain {
            let builder = builder.with_cdn(cdn);
            let url = self.oss.sign_download_url(path, &builder);
            Ok(url)
        } else {
            let url = self.oss.sign_download_url(path, &builder);
            Ok(url)
        }
    }

    async fn set_visibility(&self, path: &str, visibility: &str) -> Result<bool> {
        let url = format!("{}?acl", self.get_object_url(path));
        let resource = format!("{}?acl", self.build_resource(path));
        let date = Self::get_date();

        let acl = if visibility == "public" {
            "public-read"
        } else {
            "private"
        };

        let headers = vec![
            ("x-oss-object-acl".to_string(), acl.to_string()),
        ];

        let signature = self.sign("PUT", &resource, &headers, None);
        let authorization = format!("OSS {}:{}", self.oss_config.access_key_id, signature);

        let response = self.client
            .put(&url)
            .header("Date", &date)
            .header("Authorization", &authorization)
            .header("x-oss-object-acl", acl)
            .send()
            .await
            .with_context(|| format!("无法设置 OSS 对象可见性: {}", path))?;

        Ok(response.status().is_success())
    }

    async fn get_visibility(&self, path: &str) -> Result<String> {
        let url = format!("{}?acl", self.get_object_url(path));
        let resource = format!("{}?acl", self.build_resource(path));
        let date = Self::get_date();

        let signature = self.sign("GET", &resource, &[], None);
        let authorization = format!("OSS {}:{}", self.oss_config.access_key_id, signature);

        let response = self.client
            .get(&url)
            .header("Date", &date)
            .header("Authorization", &authorization)
            .send()
            .await
            .with_context(|| format!("无法获取 OSS 对象可见性: {}", path))?;

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
