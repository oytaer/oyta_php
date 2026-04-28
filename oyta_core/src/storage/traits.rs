//! 存储驱动接口定义
//!
//! 定义所有存储驱动必须实现的接口
//! 提供统一的文件操作抽象

use anyhow::Result;
use std::path::Path;
use async_trait::async_trait;

/// 存储驱动接口
///
/// 所有存储驱动必须实现此接口
/// 提供文件的增删改查、复制、移动等操作
#[async_trait]
pub trait StorageDriver: Send + Sync {
    /// 获取驱动名称
    fn driver_name(&self) -> &str;
    
    /// 检查文件是否存在
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// 文件存在返回 true，否则返回 false
    async fn exists(&self, path: &str) -> bool;
    
    /// 读取文件内容
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// 文件内容的字节数组
    async fn get(&self, path: &str) -> Result<Vec<u8>>;
    
    /// 读取文件内容为字符串
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// 文件内容的字符串
    async fn get_as_string(&self, path: &str) -> Result<String> {
        let bytes = self.get(path).await?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }
    
    /// 写入文件内容
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `contents`: 文件内容
    ///
    /// # 返回
    /// 成功返回 true
    async fn put(&self, path: &str, contents: &[u8]) -> Result<bool>;
    
    /// 写入字符串内容到文件
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `contents`: 字符串内容
    ///
    /// # 返回
    /// 成功返回 true
    async fn put_string(&self, path: &str, contents: &str) -> Result<bool> {
        self.put(path, contents.as_bytes()).await
    }
    
    /// 追加内容到文件
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `contents`: 追加内容
    ///
    /// # 返回
    /// 成功返回 true
    async fn append(&self, path: &str, contents: &[u8]) -> Result<bool>;
    
    /// 追加字符串内容到文件
    async fn append_string(&self, path: &str, contents: &str) -> Result<bool> {
        self.append(path, contents.as_bytes()).await
    }
    
    /// 预置内容到文件开头
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `contents`: 预置内容
    ///
    /// # 返回
    /// 成功返回 true
    async fn prepend(&self, path: &str, contents: &[u8]) -> Result<bool>;
    
    /// 删除文件
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// 成功返回 true
    async fn delete(&self, path: &str) -> Result<bool>;
    
    /// 复制文件
    ///
    /// # 参数
    /// - `from`: 源文件路径
    /// - `to`: 目标文件路径
    ///
    /// # 返回
    /// 成功返回 true
    async fn copy(&self, from: &str, to: &str) -> Result<bool>;
    
    /// 移动文件
    ///
    /// # 参数
    /// - `from`: 源文件路径
    /// - `to`: 目标文件路径
    ///
    /// # 返回
    /// 成功返回 true
    async fn move_file(&self, from: &str, to: &str) -> Result<bool>;
    
    /// 获取文件大小
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// 文件大小（字节）
    async fn size(&self, path: &str) -> Result<u64>;
    
    /// 获取文件最后修改时间
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// Unix 时间戳
    async fn last_modified(&self, path: &str) -> Result<u64>;
    
    /// 获取文件 MIME 类型
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// MIME 类型字符串
    async fn mime_type(&self, path: &str) -> Result<String>;
    
    /// 获取文件扩展名
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// 扩展名（不含点号）
    fn extension(&self, path: &str) -> String {
        Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase()
    }
    
    /// 获取文件 URL
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// 文件的公开访问 URL
    async fn url(&self, path: &str) -> Result<String>;
    
    /// 获取文件完整路径
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// 文件的完整物理路径或存储路径
    fn full_path(&self, path: &str) -> String;
    
    /// 列出目录下的所有文件
    ///
    /// # 参数
    /// - `directory`: 目录路径
    ///
    /// # 返回
    /// 文件路径列表
    async fn files(&self, directory: &str) -> Result<Vec<String>>;
    
    /// 列出目录下的所有文件（包括子目录）
    ///
    /// # 参数
    /// - `directory`: 目录路径
    ///
    /// # 返回
    /// 文件路径列表
    async fn all_files(&self, directory: &str) -> Result<Vec<String>>;
    
    /// 列出目录下的所有目录
    ///
    /// # 参数
    /// - `directory`: 目录路径
    ///
    /// # 返回
    /// 目录路径列表
    async fn directories(&self, directory: &str) -> Result<Vec<String>>;
    
    /// 创建目录
    ///
    /// # 参数
    /// - `path`: 目录路径
    ///
    /// # 返回
    /// 成功返回 true
    async fn make_directory(&self, path: &str) -> Result<bool>;
    
    /// 删除目录
    ///
    /// # 参数
    /// - `directory`: 目录路径
    ///
    /// # 返回
    /// 成功返回 true
    async fn delete_directory(&self, directory: &str) -> Result<bool>;
    
    /// 清空目录内容
    ///
    /// # 参数
    /// - `directory`: 目录路径
    ///
    /// # 返回
    /// 成功返回 true
    async fn clean_directory(&self, directory: &str) -> Result<bool>;
    
    /// 获取文件信息
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// 文件信息结构体
    async fn get_file_info(&self, path: &str) -> Result<FileInfo>;
    
    /// 生成临时访问 URL（用于私有文件）
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `expiration`: 过期时间（秒）
    ///
    /// # 返回
    /// 临时访问 URL
    async fn temporary_url(&self, path: &str, expiration: u64) -> Result<String>;
    
    /// 设置文件可见性（公开/私有）
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `visibility`: 可见性（public/private）
    ///
    /// # 返回
    /// 成功返回 true
    async fn set_visibility(&self, path: &str, visibility: &str) -> Result<bool>;
    
    /// 获取文件可见性
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// 可见性字符串
    async fn get_visibility(&self, path: &str) -> Result<String>;
}

/// 存储配置结构体
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// 驱动类型
    pub driver: String,
    /// 根目录
    pub root: String,
    /// URL 前缀
    pub url: String,
    /// 可见性（public/private）
    pub visibility: String,
    /// 是否抛出异常
    pub throw: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            driver: "local".to_string(),
            root: "storage/app".to_string(),
            url: "/storage".to_string(),
            visibility: "public".to_string(),
            throw: false,
        }
    }
}

/// 文件信息结构体
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// 文件路径
    pub path: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 最后修改时间（Unix 时间戳）
    pub last_modified: u64,
    /// MIME 类型
    pub mime_type: String,
    /// 扩展名
    pub extension: String,
    /// 文件名
    pub filename: String,
    /// 是否为目录
    pub is_dir: bool,
    /// 可见性
    pub visibility: String,
}

/// 存储操作结果
pub type StorageResult<T> = Result<T, StorageError>;

/// 存储错误类型
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// 文件不存在
    #[error("文件不存在: {0}")]
    FileNotFound(String),
    
    /// 文件已存在
    #[error("文件已存在: {0}")]
    FileAlreadyExists(String),
    
    /// 目录不存在
    #[error("目录不存在: {0}")]
    DirectoryNotFound(String),
    
    /// 权限不足
    #[error("权限不足: {0}")]
    PermissionDenied(String),
    
    /// 读取错误
    #[error("读取文件失败: {0}")]
    ReadError(String),
    
    /// 写入错误
    #[error("写入文件失败: {0}")]
    WriteError(String),
    
    /// 删除错误
    #[error("删除文件失败: {0}")]
    DeleteError(String),
    
    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    /// 连接错误
    #[error("连接错误: {0}")]
    ConnectionError(String),
    
    /// 未知错误
    #[error("未知错误: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => StorageError::FileNotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => StorageError::PermissionDenied(err.to_string()),
            std::io::ErrorKind::AlreadyExists => StorageError::FileAlreadyExists(err.to_string()),
            _ => StorageError::Unknown(err.to_string()),
        }
    }
}
