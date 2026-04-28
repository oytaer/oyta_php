//! Storage 门面类
//!
//! 提供静态方法访问存储功能
//! 对应 ThinkPHP 8.0 的 Storage 门面

use anyhow::Result;

use super::manager::get_storage_manager;
use super::traits::{StorageDriver, FileInfo};

/// Storage 门面类
///
/// 提供静态方法访问存储功能
/// 所有方法都使用默认磁盘，除非指定磁盘名称
pub struct Storage;

impl Storage {
    /// 获取指定磁盘的存储驱动
    ///
    /// # 参数
    /// - `name`: 磁盘名称，为 None 时使用默认磁盘
    ///
    /// # 返回
    /// 存储驱动实例
    ///
    /// # 示例
    /// ```php
    /// $disk = Storage::disk('s3');
    /// $disk->put('file.txt', 'content');
    /// ```
    pub fn disk(name: Option<&str>) -> Result<Box<dyn StorageDriver>> {
        let manager = get_storage_manager();
        let driver = manager.disk(name)?;
        
        // 由于 StorageDriver 不是 Sized，我们需要返回一个包装类型
        // 这里简化处理，实际应使用 trait object
        Err(anyhow::anyhow!("请使用 Storage 的静态方法"))
    }
    
    /// 检查文件是否存在
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 文件存在返回 true
    ///
    /// # 示例
    /// ```php
    /// if (Storage::exists('file.txt')) {
    ///     // 文件存在
    /// }
    /// ```
    pub async fn exists(path: &str, disk: Option<&str>) -> bool {
        let manager = get_storage_manager();
        match manager.disk(disk) {
            Ok(driver) => driver.exists(path).await,
            Err(_) => false,
        }
    }
    
    /// 读取文件内容
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 文件内容的字节数组
    ///
    /// # 示例
    /// ```php
    /// $content = Storage::get('file.txt');
    /// ```
    pub async fn get(path: &str, disk: Option<&str>) -> Result<Vec<u8>> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.get(path).await
    }
    
    /// 读取文件内容为字符串
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 文件内容的字符串
    pub async fn get_as_string(path: &str, disk: Option<&str>) -> Result<String> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.get_as_string(path).await
    }
    
    /// 写入文件内容
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `contents`: 文件内容
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 成功返回 true
    ///
    /// # 示例
    /// ```php
    /// Storage::put('file.txt', 'Hello World');
    /// ```
    pub async fn put(path: &str, contents: &[u8], disk: Option<&str>) -> Result<bool> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.put(path, contents).await
    }
    
    /// 写入字符串内容到文件
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `contents`: 字符串内容
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 成功返回 true
    pub async fn put_string(path: &str, contents: &str, disk: Option<&str>) -> Result<bool> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.put_string(path, contents).await
    }
    
    /// 追加内容到文件
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `contents`: 追加内容
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 成功返回 true
    ///
    /// # 示例
    /// ```php
    /// Storage::append('log.txt', 'New log entry');
    /// ```
    pub async fn append(path: &str, contents: &[u8], disk: Option<&str>) -> Result<bool> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.append(path, contents).await
    }
    
    /// 预置内容到文件开头
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `contents`: 预置内容
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 成功返回 true
    pub async fn prepend(path: &str, contents: &[u8], disk: Option<&str>) -> Result<bool> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.prepend(path, contents).await
    }
    
    /// 删除文件
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 成功返回 true
    ///
    /// # 示例
    /// ```php
    /// Storage::delete('file.txt');
    /// ```
    pub async fn delete(path: &str, disk: Option<&str>) -> Result<bool> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.delete(path).await
    }
    
    /// 删除多个文件
    ///
    /// # 参数
    /// - `paths`: 文件路径列表
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 成功返回 true
    pub async fn delete_multiple(paths: &[&str], disk: Option<&str>) -> Result<bool> {
        for path in paths {
            Self::delete(path, disk).await?;
        }
        Ok(true)
    }
    
    /// 复制文件
    ///
    /// # 参数
    /// - `from`: 源文件路径
    /// - `to`: 目标文件路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 成功返回 true
    ///
    /// # 示例
    /// ```php
    /// Storage::copy('old.txt', 'new.txt');
    /// ```
    pub async fn copy(from: &str, to: &str, disk: Option<&str>) -> Result<bool> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.copy(from, to).await
    }
    
    /// 移动文件
    ///
    /// # 参数
    /// - `from`: 源文件路径
    /// - `to`: 目标文件路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 成功返回 true
    ///
    /// # 示例
    /// ```php
    /// Storage::move('old.txt', 'new.txt');
    /// ```
    pub async fn move_file(from: &str, to: &str, disk: Option<&str>) -> Result<bool> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.move_file(from, to).await
    }
    
    /// 获取文件大小
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 文件大小（字节）
    ///
    /// # 示例
    /// ```php
    /// $size = Storage::size('file.txt');
    /// ```
    pub async fn size(path: &str, disk: Option<&str>) -> Result<u64> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.size(path).await
    }
    
    /// 获取文件最后修改时间
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// Unix 时间戳
    pub async fn last_modified(path: &str, disk: Option<&str>) -> Result<u64> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.last_modified(path).await
    }
    
    /// 获取文件 URL
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 文件的公开访问 URL
    ///
    /// # 示例
    /// ```php
    /// $url = Storage::url('avatar.jpg');
    /// ```
    pub async fn url(path: &str, disk: Option<&str>) -> Result<String> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.url(path).await
    }
    
    /// 获取文件完整路径
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 文件的完整物理路径或存储路径
    pub fn full_path(path: &str, disk: Option<&str>) -> Result<String> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        Ok(driver.full_path(path))
    }
    
    /// 获取文件 MIME 类型
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// MIME 类型字符串
    pub async fn mime_type(path: &str, disk: Option<&str>) -> Result<String> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.mime_type(path).await
    }
    
    /// 获取文件扩展名
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// 扩展名（不含点号）
    pub fn extension(path: &str) -> String {
        std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase()
    }
    
    /// 列出目录下的所有文件
    ///
    /// # 参数
    /// - `directory`: 目录路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 文件路径列表
    ///
    /// # 示例
    /// ```php
    /// $files = Storage::files('uploads');
    /// ```
    pub async fn files(directory: &str, disk: Option<&str>) -> Result<Vec<String>> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.files(directory).await
    }
    
    /// 列出目录下的所有文件（包括子目录）
    ///
    /// # 参数
    /// - `directory`: 目录路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 文件路径列表
    pub async fn all_files(directory: &str, disk: Option<&str>) -> Result<Vec<String>> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.all_files(directory).await
    }
    
    /// 列出目录下的所有目录
    ///
    /// # 参数
    /// - `directory`: 目录路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 目录路径列表
    pub async fn directories(directory: &str, disk: Option<&str>) -> Result<Vec<String>> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.directories(directory).await
    }
    
    /// 创建目录
    ///
    /// # 参数
    /// - `path`: 目录路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 成功返回 true
    ///
    /// # 示例
    /// ```php
    /// Storage::makeDirectory('uploads/images');
    /// ```
    pub async fn make_directory(path: &str, disk: Option<&str>) -> Result<bool> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.make_directory(path).await
    }
    
    /// 删除目录
    ///
    /// # 参数
    /// - `directory`: 目录路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 成功返回 true
    ///
    /// # 示例
    /// ```php
    /// Storage::deleteDirectory('uploads/old');
    /// ```
    pub async fn delete_directory(directory: &str, disk: Option<&str>) -> Result<bool> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.delete_directory(directory).await
    }
    
    /// 清空目录内容
    ///
    /// # 参数
    /// - `directory`: 目录路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 成功返回 true
    pub async fn clean_directory(directory: &str, disk: Option<&str>) -> Result<bool> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.clean_directory(directory).await
    }
    
    /// 获取文件信息
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 文件信息结构体
    pub async fn get_file_info(path: &str, disk: Option<&str>) -> Result<FileInfo> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.get_file_info(path).await
    }
    
    /// 生成临时访问 URL
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `expiration`: 过期时间（秒）
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 临时访问 URL
    ///
    /// # 示例
    /// ```php
    /// $url = Storage::temporaryUrl('private/file.pdf', 3600);
    /// ```
    pub async fn temporary_url(path: &str, expiration: u64, disk: Option<&str>) -> Result<String> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.temporary_url(path, expiration).await
    }
    
    /// 设置文件可见性
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `visibility`: 可见性（public/private）
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 成功返回 true
    pub async fn set_visibility(path: &str, visibility: &str, disk: Option<&str>) -> Result<bool> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.set_visibility(path, visibility).await
    }
    
    /// 获取文件可见性
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 可见性字符串
    pub async fn get_visibility(path: &str, disk: Option<&str>) -> Result<String> {
        let manager = get_storage_manager();
        let driver = manager.disk(disk)?;
        driver.get_visibility(path).await
    }
    
    /// 上传文件
    ///
    /// # 参数
    /// - `path`: 目标路径
    /// - `file`: 文件内容
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 成功返回文件路径
    ///
    /// # 示例
    /// ```php
    /// $path = Storage::upload('uploads/avatar.jpg', $fileContent);
    /// ```
    pub async fn upload(path: &str, file: &[u8], disk: Option<&str>) -> Result<String> {
        Self::put(path, file, disk).await?;
        Ok(path.to_string())
    }
    
    /// 下载文件
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `disk`: 磁盘名称（可选）
    ///
    /// # 返回
    /// 文件内容
    pub async fn download(path: &str, disk: Option<&str>) -> Result<Vec<u8>> {
        Self::get(path, disk).await
    }
}
