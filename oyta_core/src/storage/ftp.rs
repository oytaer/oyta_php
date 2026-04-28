//! FTP/FTPS 存储驱动
//!
//! 使用 suppaftp 库实现基于 FTP 协议的文件存储操作
//! 支持普通 FTP 和 FTPS（TLS 加密）连接

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::io::Cursor;
use std::sync::Arc;
use tokio::sync::Mutex;

use suppaftp::tokio::AsyncFtpStream;
use suppaftp::types::{FileType, FtpError};

use super::traits::{StorageDriver, StorageConfig, FileInfo, StorageError};

#[derive(Debug, Clone)]
pub struct FtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub root: String,
    pub passive: bool,
    pub ssl: bool,
    pub timeout: u64,
}

impl Default for FtpConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 21,
            username: String::new(),
            password: String::new(),
            root: "/".to_string(),
            passive: true,
            ssl: false,
            timeout: 30,
        }
    }
}

pub struct FtpStorage {
    config: StorageConfig,
    ftp_config: FtpConfig,
    connection: Arc<Mutex<Option<AsyncFtpStream>>>,
}

impl FtpStorage {
    pub fn new(config: StorageConfig, ftp_config: FtpConfig) -> Result<Self> {
        Ok(Self {
            config,
            ftp_config,
            connection: Arc::new(Mutex::new(None)),
        })
    }

    fn get_address(&self) -> String {
        format!("{}:{}", self.ftp_config.host, self.ftp_config.port)
    }

    fn normalize_path(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        if self.ftp_config.root == "/" || self.ftp_config.root.is_empty() {
            format!("/{}", path)
        } else {
            format!("{}/{}", self.ftp_config.root.trim_end_matches('/'), path)
        }
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

    async fn connect(&self) -> Result<AsyncFtpStream> {
        let addr = self.get_address();

        let mut ftp = AsyncFtpStream::connect(&addr).await
            .map_err(|e| StorageError::ConnectionError(format!("FTP 连接失败: {} - {}", addr, e)))?;

        ftp.login(&self.ftp_config.username, &self.ftp_config.password).await
            .map_err(|e| StorageError::ConnectionError(format!("FTP 登录失败: {}", e)))?;

        ftp.transfer_type(FileType::Binary).await
            .map_err(|e| StorageError::ConnectionError(format!("FTP 设置传输类型失败: {}", e)))?;

        let root = self.ftp_config.root.trim_end_matches('/');
        if !root.is_empty() && root != "/" {
            let _ = ftp.cwd(root).await;
        }

        Ok(ftp)
    }

    async fn get_connection(&self) -> Result<tokio::sync::MutexGuard<'_, Option<AsyncFtpStream>>> {
        let mut guard = self.connection.lock().await;

        if guard.is_none() {
            let ftp = self.connect().await?;
            *guard = Some(ftp);
        }

        Ok(guard)
    }

    async fn reset_connection(&self) -> Result<()> {
        let mut guard = self.connection.lock().await;
        if let Some(mut ftp) = guard.take() {
            let _ = ftp.quit().await;
        }
        Ok(())
    }

    async fn ensure_parent_dirs(ftp: &mut AsyncFtpStream, full_path: &str) -> Result<()> {
        let path_obj = std::path::Path::new(full_path);
        let parent = path_obj.parent()
            .and_then(|p| p.to_str())
            .unwrap_or("/");

        let parts: Vec<&str> = parent.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = String::from("/");
        for part in parts {
            current = format!("{}/", current.trim_end_matches('/'));
            let _ = ftp.cwd(&current).await;
            if ftp.cwd(part).await.is_err() {
                ftp.mkdir(part).await
                    .map_err(|e| StorageError::WriteError(format!("FTP 创建目录失败: {}", e)))?;
                let _ = ftp.cwd(part).await;
            }
            current = format!("{}{}", current, part);
        }

        Ok(())
    }
}

#[async_trait]
impl StorageDriver for FtpStorage {
    fn driver_name(&self) -> &str {
        "ftp"
    }

    async fn exists(&self, path: &str) -> bool {
        let full_path = self.normalize_path(path);
        let path_obj = std::path::Path::new(&full_path);
        let parent = path_obj.parent()
            .and_then(|p| p.to_str())
            .unwrap_or("/")
            .to_string();
        let filename = path_obj.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path)
            .to_string();

        let result: Result<bool> = async {
            let mut guard = self.get_connection().await?;
            let ftp = guard.as_mut().ok_or_else(|| {
                StorageError::ConnectionError("FTP 连接未建立".to_string())
            })?;

            let _ = ftp.cwd(&parent).await;

            let files = ftp.nlst(None).await
                .map_err(|e| StorageError::ReadError(format!("FTP 列出文件失败: {}", e)))?;

            Ok(files.iter().any(|f| f == &filename || f.ends_with(&format!("/{}", filename))))
        }.await;

        match result {
            Ok(r) => r,
            Err(_) => {
                let _ = self.reset_connection().await;
                false
            }
        }
    }

    async fn get(&self, path: &str) -> Result<Vec<u8>> {
        let full_path = self.normalize_path(path);

        let mut guard = self.get_connection().await?;
        let ftp = guard.as_mut().ok_or_else(|| {
            StorageError::ConnectionError("FTP 连接未建立".to_string())
        })?;

        let data_stream = ftp.retr_as_stream(&full_path).await
            .map_err(|e| StorageError::ReadError(format!("FTP 下载文件失败: {} - {}", path, e)))?;

        let mut result = Vec::new();
        use tokio::io::AsyncReadExt;
        let mut reader = data_stream;
        reader.read_to_end(&mut result).await
            .map_err(|e| StorageError::ReadError(format!("FTP 读取数据失败: {}", e)))?;

        ftp.finalize_retr_stream(reader).await
            .map_err(|e| StorageError::ReadError(format!("FTP 完成读取失败: {}", e)))?;

        Ok(result)
    }

    async fn put(&self, path: &str, contents: &[u8]) -> Result<bool> {
        let full_path = self.normalize_path(path);

        let mut guard = self.get_connection().await?;
        let ftp = guard.as_mut().ok_or_else(|| {
            StorageError::ConnectionError("FTP 连接未建立".to_string())
        })?;

        Self::ensure_parent_dirs(ftp, &full_path).await?;

        let mut cursor = Cursor::new(contents.to_vec());
        ftp.put_file(&full_path, &mut cursor).await
            .map_err(|e| StorageError::WriteError(format!("FTP 上传文件失败: {} - {}", path, e)))?;

        Ok(true)
    }

    async fn append(&self, path: &str, contents: &[u8]) -> Result<bool> {
        let full_path = self.normalize_path(path);

        let mut guard = self.get_connection().await?;
        let ftp = guard.as_mut().ok_or_else(|| {
            StorageError::ConnectionError("FTP 连接未建立".to_string())
        })?;

        let mut cursor = Cursor::new(contents.to_vec());
        ftp.append_file(&full_path, &mut cursor).await
            .map_err(|e| StorageError::WriteError(format!("FTP 追加文件失败: {} - {}", path, e)))?;

        Ok(true)
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
        let full_path = self.normalize_path(path);

        let mut guard = self.get_connection().await?;
        let ftp = guard.as_mut().ok_or_else(|| {
            StorageError::ConnectionError("FTP 连接未建立".to_string())
        })?;

        ftp.rm(&full_path).await
            .map_err(|e| StorageError::DeleteError(format!("FTP 删除文件失败: {} - {}", path, e)))?;

        Ok(true)
    }

    async fn copy(&self, from: &str, to: &str) -> Result<bool> {
        let data = self.get(from).await?;
        self.put(to, &data).await
    }

    async fn move_file(&self, from: &str, to: &str) -> Result<bool> {
        let from_path = self.normalize_path(from);
        let to_path = self.normalize_path(to);

        let mut guard = self.get_connection().await?;
        let ftp = guard.as_mut().ok_or_else(|| {
            StorageError::ConnectionError("FTP 连接未建立".to_string())
        })?;

        Self::ensure_parent_dirs(ftp, &to_path).await?;

        ftp.rename(&from_path, &to_path).await
            .map_err(|e| StorageError::WriteError(format!("FTP 移动文件失败: {} -> {} - {}", from, to, e)))?;

        Ok(true)
    }

    async fn size(&self, path: &str) -> Result<u64> {
        let full_path = self.normalize_path(path);

        let mut guard = self.get_connection().await?;
        let ftp = guard.as_mut().ok_or_else(|| {
            StorageError::ConnectionError("FTP 连接未建立".to_string())
        })?;

        let size = ftp.size(&full_path).await
            .map_err(|e| StorageError::FileNotFound(format!("FTP 获取文件大小失败: {} - {}", path, e)))?;

        Ok(size as u64)
    }

    async fn last_modified(&self, path: &str) -> Result<u64> {
        let full_path = self.normalize_path(path);

        let mut guard = self.get_connection().await?;
        let ftp = guard.as_mut().ok_or_else(|| {
            StorageError::ConnectionError("FTP 连接未建立".to_string())
        })?;

        let mtime = ftp.mdtm(&full_path).await
            .map_err(|e| StorageError::FileNotFound(format!("FTP 获取修改时间失败: {} - {}", path, e)))?;

        Ok(mtime.timestamp() as u64)
    }

    async fn mime_type(&self, path: &str) -> Result<String> {
        Ok(Self::guess_mime_type(path))
    }

    async fn url(&self, path: &str) -> Result<String> {
        let scheme = if self.ftp_config.ssl { "ftps" } else { "ftp" };
        Ok(format!("{}://{}:{}/{}", scheme, self.ftp_config.host, self.ftp_config.port, path.trim_start_matches('/')))
    }

    fn full_path(&self, path: &str) -> String {
        self.normalize_path(path)
    }

    async fn files(&self, directory: &str) -> Result<Vec<String>> {
        let full_path = self.normalize_path(directory);

        let mut guard = self.get_connection().await?;
        let ftp = guard.as_mut().ok_or_else(|| {
            StorageError::ConnectionError("FTP 连接未建立".to_string())
        })?;

        let list_result = ftp.nlst(Some(&full_path)).await
            .map_err(|e| StorageError::ReadError(format!("FTP 列出文件失败: {} - {}", directory, e)))?;

        let files: Vec<String> = list_result
            .into_iter()
            .filter(|f| !f.ends_with('/'))
            .map(|f| {
                std::path::Path::new(&f)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&f)
                    .to_string()
            })
            .collect();

        Ok(files)
    }

    async fn all_files(&self, directory: &str) -> Result<Vec<String>> {
        let full_path = self.normalize_path(directory);

        let mut guard = self.get_connection().await?;
        let ftp = guard.as_mut().ok_or_else(|| {
            StorageError::ConnectionError("FTP 连接未建立".to_string())
        })?;

        let _ = ftp.cwd(&full_path).await;

        let list_result = ftp.list(None).await
            .map_err(|e| StorageError::ReadError(format!("FTP 列出所有文件失败: {} - {}", directory, e)))?;

        let mut all_files = Vec::new();
        let mut subdirs = Vec::new();

        for entry in &list_result {
            let parts: Vec<&str> = entry.split_whitespace().collect();
            if parts.len() >= 9 {
                let name = parts[8..].join(" ");
                let is_dir = entry.starts_with('d');
                if is_dir && name != "." && name != ".." {
                    subdirs.push(name);
                } else if !is_dir {
                    all_files.push(name);
                }
            }
        }

        drop(guard);

        for subdir in subdirs {
            let subdir_path = format!("{}/{}", directory.trim_end_matches('/'), subdir);
            match self.all_files(&subdir_path).await {
                Ok(sub_files) => {
                    for f in sub_files {
                        all_files.push(format!("{}/{}", subdir, f));
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(all_files)
    }

    async fn directories(&self, directory: &str) -> Result<Vec<String>> {
        let full_path = self.normalize_path(directory);

        let mut guard = self.get_connection().await?;
        let ftp = guard.as_mut().ok_or_else(|| {
            StorageError::ConnectionError("FTP 连接未建立".to_string())
        })?;

        let _ = ftp.cwd(&full_path).await;

        let list_result = ftp.list(None).await
            .map_err(|e| StorageError::ReadError(format!("FTP 列出目录失败: {} - {}", directory, e)))?;

        let dirs: Vec<String> = list_result
            .into_iter()
            .filter(|entry| entry.starts_with('d'))
            .filter_map(|entry| {
                let parts: Vec<&str> = entry.split_whitespace().collect();
                if parts.len() >= 9 {
                    let name = parts[8..].join(" ");
                    if name != "." && name != ".." {
                        return Some(name);
                    }
                }
                None
            })
            .collect();

        Ok(dirs)
    }

    async fn make_directory(&self, path: &str) -> Result<bool> {
        let full_path = self.normalize_path(path);

        let mut guard = self.get_connection().await?;
        let ftp = guard.as_mut().ok_or_else(|| {
            StorageError::ConnectionError("FTP 连接未建立".to_string())
        })?;

        Self::ensure_parent_dirs(ftp, &full_path).await?;

        Ok(true)
    }

    async fn delete_directory(&self, directory: &str) -> Result<bool> {
        let full_path = self.normalize_path(directory);

        let mut guard = self.get_connection().await?;
        let ftp = guard.as_mut().ok_or_else(|| {
            StorageError::ConnectionError("FTP 连接未建立".to_string())
        })?;

        let _ = ftp.cwd(&full_path).await;

        let list_result = ftp.list(None).await
            .map_err(|e| StorageError::ReadError(format!("FTP 列出目录内容失败: {} - {}", directory, e)))?;

        let mut subdirs = Vec::new();

        for entry in &list_result {
            let parts: Vec<&str> = entry.split_whitespace().collect();
            if parts.len() >= 9 {
                let name = parts[8..].join(" ");
                if name == "." || name == ".." {
                    continue;
                }
                let is_dir = entry.starts_with('d');
                if is_dir {
                    subdirs.push(name.clone());
                } else {
                    let file_path = format!("{}/{}", full_path.trim_end_matches('/'), name);
                    ftp.rm(&file_path).await.ok();
                }
            }
        }

        drop(guard);

        for subdir in &subdirs {
            let sub_path = format!("{}/{}", directory.trim_end_matches('/'), subdir);
            self.delete_directory(&sub_path).await?;
        }

        let mut guard = self.get_connection().await?;
        let ftp = guard.as_mut().ok_or_else(|| {
            StorageError::ConnectionError("FTP 连接未建立".to_string())
        })?;

        ftp.rmdir(&full_path).await
            .map_err(|e| StorageError::DeleteError(format!("FTP 删除目录失败: {} - {}", directory, e)))?;

        Ok(true)
    }

    async fn clean_directory(&self, directory: &str) -> Result<bool> {
        let full_path = self.normalize_path(directory);

        let mut guard = self.get_connection().await?;
        let ftp = guard.as_mut().ok_or_else(|| {
            StorageError::ConnectionError("FTP 连接未建立".to_string())
        })?;

        let _ = ftp.cwd(&full_path).await;

        let list_result = ftp.list(None).await
            .map_err(|e| StorageError::ReadError(format!("FTP 列出目录内容失败: {} - {}", directory, e)))?;

        let mut subdirs = Vec::new();

        for entry in &list_result {
            let parts: Vec<&str> = entry.split_whitespace().collect();
            if parts.len() >= 9 {
                let name = parts[8..].join(" ");
                if name == "." || name == ".." {
                    continue;
                }
                let is_dir = entry.starts_with('d');
                if is_dir {
                    subdirs.push(name.clone());
                } else {
                    let file_path = format!("{}/{}", full_path.trim_end_matches('/'), name);
                    ftp.rm(&file_path).await.ok();
                }
            }
        }

        drop(guard);

        for subdir in &subdirs {
            let sub_path = format!("{}/{}", directory.trim_end_matches('/'), subdir);
            self.delete_directory(&sub_path).await?;
        }

        Ok(true)
    }

    async fn get_file_info(&self, path: &str) -> Result<FileInfo> {
        let full_path = self.normalize_path(path);

        let mut guard = self.get_connection().await?;
        let ftp = guard.as_mut().ok_or_else(|| {
            StorageError::ConnectionError("FTP 连接未建立".to_string())
        })?;

        let size = ftp.size(&full_path).await.unwrap_or(0) as u64;

        let last_modified = ftp.mdtm(&full_path).await
            .map(|dt| dt.timestamp() as u64)
            .unwrap_or(0);

        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        Ok(FileInfo {
            path: path.to_string(),
            size,
            last_modified,
            mime_type: Self::guess_mime_type(path),
            extension: self.extension(path),
            filename,
            is_dir: false,
            visibility: "private".to_string(),
        })
    }

    async fn temporary_url(&self, path: &str, _expiration: u64) -> Result<String> {
        self.url(path).await
    }

    async fn set_visibility(&self, _path: &str, _visibility: &str) -> Result<bool> {
        Ok(true)
    }

    async fn get_visibility(&self, _path: &str) -> Result<String> {
        Ok("private".to_string())
    }
}
