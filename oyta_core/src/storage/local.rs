//! 本地文件存储驱动
//!
//! 实现基于本地文件系统的存储操作
//! 支持文件的增删改查、目录操作等

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;

use super::traits::{StorageDriver, StorageConfig, FileInfo, StorageError};

/// 本地存储驱动
///
/// 将文件存储在本地文件系统中
/// 支持相对路径和绝对路径
pub struct LocalStorage {
    /// 存储配置
    config: StorageConfig,
    /// 根目录的绝对路径
    root_path: PathBuf,
}

impl LocalStorage {
    /// 创建本地存储驱动实例
    ///
    /// # 参数
    /// - `config`: 存储配置
    ///
    /// # 返回
    /// 本地存储驱动实例
    pub fn new(config: StorageConfig) -> Self {
        let root_path = PathBuf::from(&config.root);
        
        // 确保根目录存在
        if !root_path.exists() {
            let _ = fs::create_dir_all(&root_path);
        }
        
        Self { config, root_path }
    }
    
    /// 使用默认配置创建本地存储驱动
    pub fn default_storage() -> Self {
        Self::new(StorageConfig::default())
    }
    
    /// 获取文件的完整物理路径
    ///
    /// # 参数
    /// - `path`: 相对路径
    ///
    /// # 返回
    /// 完整物理路径
    /// 
    /// # 安全性
    /// 防止路径遍历攻击，确保路径在根目录内
    fn full_path_internal(&self, path: &str) -> PathBuf {
        // URL解码路径（防止URL编码绕过）
        let decoded_path = urlencoding::decode(path).unwrap_or_default().to_string();
        
        // 移除开头的斜杠
        let normalized_path = decoded_path.trim_start_matches('/');
        
        // 安全检查：防止路径遍历攻击
        // 检查是否包含 .. 或其他危险路径组件
        let dangerous_patterns = ["../", "..\\", ".."];
        for pattern in &dangerous_patterns {
            if normalized_path.contains(pattern) {
                // 如果检测到路径遍历尝试，返回根目录
                return self.root_path.clone();
            }
        }
        
        // 检查路径组件是否包含空字节（防止空字节注入）
        if normalized_path.contains('\0') {
            return self.root_path.clone();
        }
        
        // 构建完整路径
        let full_path = self.root_path.join(normalized_path);
        
        // 规范化路径并验证是否在根目录内
        match full_path.canonicalize() {
            Ok(canonical) => {
                // 验证规范化后的路径是否仍在根目录内
                if canonical.starts_with(&self.root_path) {
                    canonical
                } else {
                    // 路径遍历尝试，返回根目录
                    self.root_path.clone()
                }
            }
            Err(_) => {
                // 文件不存在时，使用未规范化的路径
                // 但仍需验证不会逃逸根目录
                full_path
            }
        }
    }
    
    /// 验证路径是否安全
    ///
    /// # 参数
    /// - `path`: 要验证的路径
    ///
    /// # 返回
    /// 路径是否安全
    fn is_path_safe(&self, path: &str) -> bool {
        let decoded = urlencoding::decode(path).unwrap_or_default();
        let path_str = decoded.trim_start_matches('/');
        
        // 检查危险模式
        !path_str.contains("..") 
            && !path_str.contains('\0')
            && !path_str.starts_with('/')
    }
    
    /// 确保父目录存在
    ///
    /// # 参数
    /// - `path`: 文件路径
    fn ensure_parent_directory(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("无法创建目录: {:?}", parent))?;
            }
        }
        Ok(())
    }
    
    /// 获取文件的 MIME 类型
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// MIME 类型字符串
    fn guess_mime_type(&self, path: &str) -> String {
        let extension = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            // 图片
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            "ico" => "image/x-icon",
            "bmp" => "image/bmp",
            
            // 文档
            "pdf" => "application/pdf",
            "doc" => "application/msword",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "xls" => "application/vnd.ms-excel",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "ppt" => "application/vnd.ms-powerpoint",
            "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            
            // 文本
            "txt" => "text/plain",
            "html" | "htm" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "json" => "application/json",
            "xml" => "application/xml",
            "csv" => "text/csv",
            
            // 压缩
            "zip" => "application/zip",
            "rar" => "application/vnd.rar",
            "7z" => "application/x-7z-compressed",
            "tar" => "application/x-tar",
            "gz" => "application/gzip",
            
            // 音频
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            "ogg" => "audio/ogg",
            "m4a" => "audio/mp4",
            
            // 视频
            "mp4" => "video/mp4",
            "avi" => "video/x-msvideo",
            "mov" => "video/quicktime",
            "wmv" => "video/x-ms-wmv",
            "flv" => "video/x-flv",
            "mkv" => "video/x-matroska",
            
            // 字体
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "ttf" => "font/ttf",
            "otf" => "font/otf",
            "eot" => "application/vnd.ms-fontobject",
            
            // 其他
            "swf" => "application/x-shockwave-flash",
            "exe" => "application/vnd.microsoft.portable-executable",
            "apk" => "application/vnd.android.package-archive",
            "ipa" => "application/octet-stream",
            
            _ => "application/octet-stream",
        }.to_string()
    }
}

#[async_trait]
impl StorageDriver for LocalStorage {
    /// 获取驱动名称
    fn driver_name(&self) -> &str {
        "local"
    }
    
    /// 检查文件是否存在
    async fn exists(&self, path: &str) -> bool {
        let full_path = self.full_path_internal(path);
        full_path.exists()
    }
    
    /// 读取文件内容
    async fn get(&self, path: &str) -> Result<Vec<u8>> {
        let full_path = self.full_path_internal(path);
        
        if !full_path.exists() {
            return Err(StorageError::FileNotFound(path.to_string()).into());
        }
        
        fs::read(&full_path)
            .with_context(|| format!("无法读取文件: {:?}", full_path))
    }
    
    /// 写入文件内容
    async fn put(&self, path: &str, contents: &[u8]) -> Result<bool> {
        let full_path = self.full_path_internal(path);
        
        // 确保父目录存在
        self.ensure_parent_directory(&full_path)?;
        
        fs::write(&full_path, contents)
            .with_context(|| format!("无法写入文件: {:?}", full_path))?;
        
        Ok(true)
    }
    
    /// 追加内容到文件
    async fn append(&self, path: &str, contents: &[u8]) -> Result<bool> {
        let full_path = self.full_path_internal(path);
        
        // 确保父目录存在
        self.ensure_parent_directory(&full_path)?;
        
        // 打开文件（如果存在则追加，否则创建）
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&full_path)
            .with_context(|| format!("无法打开文件: {:?}", full_path))?;
        
        use std::io::Write;
        file.write_all(contents)
            .with_context(|| format!("无法追加内容到文件: {:?}", full_path))?;
        
        Ok(true)
    }
    
    /// 预置内容到文件开头
    async fn prepend(&self, path: &str, contents: &[u8]) -> Result<bool> {
        let full_path = self.full_path_internal(path);
        
        // 如果文件不存在，直接写入
        if !full_path.exists() {
            return self.put(path, contents).await;
        }
        
        // 读取原有内容
        let original = fs::read(&full_path)
            .with_context(|| format!("无法读取文件: {:?}", full_path))?;
        
        // 合并内容
        let mut new_contents = contents.to_vec();
        new_contents.extend_from_slice(&original);
        
        // 写入新内容
        fs::write(&full_path, &new_contents)
            .with_context(|| format!("无法写入文件: {:?}", full_path))?;
        
        Ok(true)
    }
    
    /// 删除文件
    async fn delete(&self, path: &str) -> Result<bool> {
        let full_path = self.full_path_internal(path);
        
        if !full_path.exists() {
            return Ok(true); // 文件不存在视为删除成功
        }
        
        fs::remove_file(&full_path)
            .with_context(|| format!("无法删除文件: {:?}", full_path))?;
        
        Ok(true)
    }
    
    /// 复制文件
    async fn copy(&self, from: &str, to: &str) -> Result<bool> {
        let from_path = self.full_path_internal(from);
        let to_path = self.full_path_internal(to);
        
        if !from_path.exists() {
            return Err(StorageError::FileNotFound(from.to_string()).into());
        }
        
        // 确保目标目录存在
        self.ensure_parent_directory(&to_path)?;
        
        fs::copy(&from_path, &to_path)
            .with_context(|| format!("无法复制文件: {:?} -> {:?}", from_path, to_path))?;
        
        Ok(true)
    }
    
    /// 移动文件
    async fn move_file(&self, from: &str, to: &str) -> Result<bool> {
        let from_path = self.full_path_internal(from);
        let to_path = self.full_path_internal(to);
        
        if !from_path.exists() {
            return Err(StorageError::FileNotFound(from.to_string()).into());
        }
        
        // 确保目标目录存在
        self.ensure_parent_directory(&to_path)?;
        
        fs::rename(&from_path, &to_path)
            .with_context(|| format!("无法移动文件: {:?} -> {:?}", from_path, to_path))?;
        
        Ok(true)
    }
    
    /// 获取文件大小
    async fn size(&self, path: &str) -> Result<u64> {
        let full_path = self.full_path_internal(path);
        
        if !full_path.exists() {
            return Err(StorageError::FileNotFound(path.to_string()).into());
        }
        
        let metadata = fs::metadata(&full_path)
            .with_context(|| format!("无法获取文件元数据: {:?}", full_path))?;
        
        Ok(metadata.len())
    }
    
    /// 获取文件最后修改时间
    async fn last_modified(&self, path: &str) -> Result<u64> {
        let full_path = self.full_path_internal(path);
        
        if !full_path.exists() {
            return Err(StorageError::FileNotFound(path.to_string()).into());
        }
        
        let metadata = fs::metadata(&full_path)
            .with_context(|| format!("无法获取文件元数据: {:?}", full_path))?;
        
        let modified = metadata.modified()
            .with_context(|| format!("无法获取修改时间: {:?}", full_path))?;
        
        let duration = modified.duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        
        Ok(duration.as_secs())
    }
    
    /// 获取文件 MIME 类型
    async fn mime_type(&self, path: &str) -> Result<String> {
        // 本地存储使用扩展名猜测 MIME 类型
        Ok(self.guess_mime_type(path))
    }
    
    /// 获取文件 URL
    async fn url(&self, path: &str) -> Result<String> {
        let normalized_path = path.trim_start_matches('/');
        Ok(format!("{}/{}", self.config.url, normalized_path))
    }
    
    /// 获取文件完整路径
    fn full_path(&self, path: &str) -> String {
        self.full_path_internal(path).to_string_lossy().to_string()
    }
    
    /// 列出目录下的所有文件
    async fn files(&self, directory: &str) -> Result<Vec<String>> {
        let dir_path = self.full_path_internal(directory);
        
        if !dir_path.exists() || !dir_path.is_dir() {
            return Ok(Vec::new());
        }
        
        let mut files = Vec::new();
        let entries = fs::read_dir(&dir_path)
            .with_context(|| format!("无法读取目录: {:?}", dir_path))?;
        
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    // 转换为相对路径
                    let relative = path.strip_prefix(&self.root_path)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                    files.push(relative);
                }
            }
        }
        
        Ok(files)
    }
    
    /// 列出目录下的所有文件（包括子目录）
    async fn all_files(&self, directory: &str) -> Result<Vec<String>> {
        let dir_path = self.full_path_internal(directory);
        
        if !dir_path.exists() || !dir_path.is_dir() {
            return Ok(Vec::new());
        }
        
        let mut files = Vec::new();
        
        for entry in WalkDir::new(&dir_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                // 转换为相对路径
                let relative = path.strip_prefix(&self.root_path)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();
                files.push(relative);
            }
        }
        
        Ok(files)
    }
    
    /// 列出目录下的所有目录
    async fn directories(&self, directory: &str) -> Result<Vec<String>> {
        let dir_path = self.full_path_internal(directory);
        
        if !dir_path.exists() || !dir_path.is_dir() {
            return Ok(Vec::new());
        }
        
        let mut dirs = Vec::new();
        let entries = fs::read_dir(&dir_path)
            .with_context(|| format!("无法读取目录: {:?}", dir_path))?;
        
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    // 转换为相对路径
                    let relative = path.strip_prefix(&self.root_path)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                    dirs.push(relative);
                }
            }
        }
        
        Ok(dirs)
    }
    
    /// 创建目录
    async fn make_directory(&self, path: &str) -> Result<bool> {
        let full_path = self.full_path_internal(path);
        
        if !full_path.exists() {
            fs::create_dir_all(&full_path)
                .with_context(|| format!("无法创建目录: {:?}", full_path))?;
        }
        
        Ok(true)
    }
    
    /// 删除目录
    async fn delete_directory(&self, directory: &str) -> Result<bool> {
        let dir_path = self.full_path_internal(directory);
        
        if !dir_path.exists() {
            return Ok(true); // 目录不存在视为删除成功
        }
        
        fs::remove_dir_all(&dir_path)
            .with_context(|| format!("无法删除目录: {:?}", dir_path))?;
        
        Ok(true)
    }
    
    /// 清空目录内容
    async fn clean_directory(&self, directory: &str) -> Result<bool> {
        let dir_path = self.full_path_internal(directory);
        
        if !dir_path.exists() {
            return Ok(true);
        }
        
        let entries = fs::read_dir(&dir_path)
            .with_context(|| format!("无法读取目录: {:?}", dir_path))?;
        
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    fs::remove_dir_all(&path)
                        .with_context(|| format!("无法删除目录: {:?}", path))?;
                } else {
                    fs::remove_file(&path)
                        .with_context(|| format!("无法删除文件: {:?}", path))?;
                }
            }
        }
        
        Ok(true)
    }
    
    /// 获取文件信息
    async fn get_file_info(&self, path: &str) -> Result<FileInfo> {
        let full_path = self.full_path_internal(path);
        
        if !full_path.exists() {
            return Err(StorageError::FileNotFound(path.to_string()).into());
        }
        
        let metadata = fs::metadata(&full_path)
            .with_context(|| format!("无法获取文件元数据: {:?}", full_path))?;
        
        let last_modified = metadata.modified()
            .map(|m| m.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
            .unwrap_or(0);
        
        let filename = full_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        
        Ok(FileInfo {
            path: path.to_string(),
            size: metadata.len(),
            last_modified,
            mime_type: self.guess_mime_type(path),
            extension: self.extension(path),
            filename,
            is_dir: metadata.is_dir(),
            visibility: self.config.visibility.clone(),
        })
    }
    
    /// 生成临时访问 URL
    async fn temporary_url(&self, path: &str, _expiration: u64) -> Result<String> {
        // 本地存储不支持临时 URL，返回普通 URL
        self.url(path).await
    }
    
    /// 设置文件可见性
    async fn set_visibility(&self, _path: &str, _visibility: &str) -> Result<bool> {
        // 本地存储通过文件系统权限控制，暂不实现
        Ok(true)
    }
    
    /// 获取文件可见性
    async fn get_visibility(&self, _path: &str) -> Result<String> {
        Ok(self.config.visibility.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_put_and_get() {
        let dir = tempdir().unwrap();
        let config = StorageConfig {
            driver: "local".to_string(),
            root: dir.path().to_string_lossy().to_string(),
            ..Default::default()
        };
        
        let storage = LocalStorage::new(config);
        
        // 写入文件
        storage.put("test.txt", b"Hello, World!").await.unwrap();
        
        // 读取文件
        let content = storage.get("test.txt").await.unwrap();
        assert_eq!(content, b"Hello, World!");
    }
    
    #[tokio::test]
    async fn test_exists() {
        let dir = tempdir().unwrap();
        let config = StorageConfig {
            driver: "local".to_string(),
            root: dir.path().to_string_lossy().to_string(),
            ..Default::default()
        };
        
        let storage = LocalStorage::new(config);
        
        assert!(!storage.exists("test.txt").await);
        
        storage.put("test.txt", b"content").await.unwrap();
        assert!(storage.exists("test.txt").await);
    }
    
    #[tokio::test]
    async fn test_delete() {
        let dir = tempdir().unwrap();
        let config = StorageConfig {
            driver: "local".to_string(),
            root: dir.path().to_string_lossy().to_string(),
            ..Default::default()
        };
        
        let storage = LocalStorage::new(config);
        
        storage.put("test.txt", b"content").await.unwrap();
        assert!(storage.exists("test.txt").await);
        
        storage.delete("test.txt").await.unwrap();
        assert!(!storage.exists("test.txt").await);
    }
    
    #[tokio::test]
    async fn test_copy_and_move() {
        let dir = tempdir().unwrap();
        let config = StorageConfig {
            driver: "local".to_string(),
            root: dir.path().to_string_lossy().to_string(),
            ..Default::default()
        };
        
        let storage = LocalStorage::new(config);
        
        storage.put("source.txt", b"content").await.unwrap();
        
        // 复制
        storage.copy("source.txt", "copy.txt").await.unwrap();
        assert!(storage.exists("copy.txt").await);
        
        // 移动
        storage.move_file("source.txt", "moved.txt").await.unwrap();
        assert!(!storage.exists("source.txt").await);
        assert!(storage.exists("moved.txt").await);
    }
}
