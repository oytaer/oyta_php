//! 文件上传处理模块
//!
//! 实现 HTTP 文件上传功能
//! 支持：单文件上传、多文件上传、文件验证、文件保存
//! 对应 ThinkPHP 8.0 的 UploadFile 类
//!
//! # 使用方式
//! ```php
//! $file = request()->file('avatar');
//! $file->validate(['size' => 1024*1024*2, 'ext' => 'jpg,png']);
//! $path = $file->move('uploads/avatar');
//! ```

use anyhow::{Context, Result};
use std::path::Path;

/// 上传文件信息
///
/// 包含上传文件的所有元数据和内容
#[derive(Debug, Clone)]
pub struct UploadedFile {
    /// 表单字段名
    pub field_name: String,
    /// 原始文件名（客户端提供的文件名）
    pub original_name: String,
    /// MIME 类型
    pub mime_type: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 临时文件路径
    pub temp_path: String,
    /// 上传错误码（0 表示无错误）
    pub error_code: u32,
    /// 文件扩展名（从原始文件名提取）
    pub extension: String,
}

impl UploadedFile {
    /// 从 multipart 表单数据创建上传文件
    pub fn new(
        field_name: &str,
        original_name: &str,
        mime_type: &str,
        size: u64,
        temp_path: &str,
    ) -> Self {
        let extension = Path::new(original_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        Self {
            field_name: field_name.to_string(),
            original_name: original_name.to_string(),
            mime_type: mime_type.to_string(),
            size,
            temp_path: temp_path.to_string(),
            error_code: 0,
            extension,
        }
    }

    /// 检查上传是否成功
    pub fn is_valid(&self) -> bool {
        self.error_code == 0 && !self.temp_path.is_empty()
    }

    /// 获取文件扩展名
    pub fn extension(&self) -> &str {
        &self.extension
    }

    /// 获取文件名（不含扩展名）
    pub fn filename(&self) -> &str {
        if let Some(pos) = self.original_name.rfind('.') {
            &self.original_name[..pos]
        } else {
            &self.original_name
        }
    }

    /// 验证文件
    ///
    /// # 参数
    /// - `rules`: 验证规则
    ///
    /// # 返回
    /// 验证结果
    pub fn validate(&self, rules: &UploadRules) -> Result<()> {
        // 检查文件大小
        if let Some(max_size) = rules.max_size {
            if self.size > max_size {
                return Err(anyhow::anyhow!(
                    "文件大小超过限制: {} > {}",
                    self.size,
                    max_size
                ));
            }
        }

        // 检查文件扩展名
        if !rules.allowed_exts.is_empty() {
            if !rules.allowed_exts.contains(&self.extension) {
                return Err(anyhow::anyhow!(
                    "文件类型不支持: {}",
                    self.extension
                ));
            }
        }

        // 检查 MIME 类型
        if !rules.allowed_mimes.is_empty() {
            if !rules.allowed_mimes.contains(&self.mime_type) {
                return Err(anyhow::anyhow!(
                    "MIME类型不支持: {}",
                    self.mime_type
                ));
            }
        }

        Ok(())
    }

    /// 移动上传文件到目标路径
    ///
    /// # 参数
    /// - `directory`: 目标目录
    /// - `name`: 目标文件名（不含扩展名），为空则使用随机名称
    pub fn move_to(&self, directory: &str, name: Option<&str>) -> Result<String> {
        let dir = Path::new(directory);
        if !dir.exists() {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("无法创建目录: {}", directory))?;
        }

        let file_name = match name {
            Some(n) => {
                if n.contains('.') {
                    n.to_string()
                } else {
                    format!("{}.{}", n, self.extension)
                }
            }
            None => {
                let unique_name = generate_unique_name();
                format!("{}.{}", unique_name, self.extension)
            }
        };

        let dest_path = dir.join(&file_name);
        let src_path = Path::new(&self.temp_path);

        if src_path.exists() {
            std::fs::copy(src_path, &dest_path)
                .with_context(|| format!("无法复制文件: {:?} → {:?}", src_path, dest_path))?;

            // 删除临时文件
            let _ = std::fs::remove_file(src_path);
        }

        Ok(file_name)
    }

    /// 获取文件内容
    pub fn content(&self) -> Result<Vec<u8>> {
        std::fs::read(&self.temp_path)
            .with_context(|| format!("无法读取文件: {}", self.temp_path))
    }

    /// 判断是否为图片
    pub fn is_image(&self) -> bool {
        let image_mimes = [
            "image/jpeg",
            "image/png",
            "image/gif",
            "image/bmp",
            "image/webp",
            "image/svg+xml",
        ];
        image_mimes.contains(&self.mime_type.as_str())
    }

    /// 获取文件大小的可读格式
    pub fn human_size(&self) -> String {
        if self.size < 1024 {
            format!("{} B", self.size)
        } else if self.size < 1024 * 1024 {
            format!("{:.1} KB", self.size as f64 / 1024.0)
        } else if self.size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", self.size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", self.size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

/// 上传验证规则
#[derive(Debug, Clone, Default)]
pub struct UploadRules {
    /// 最大文件大小（字节）
    pub max_size: Option<u64>,
    /// 允许的扩展名列表
    pub allowed_exts: Vec<String>,
    /// 允许的 MIME 类型列表
    pub allowed_mimes: Vec<String>,
}

impl UploadRules {
    /// 创建图片上传规则
    pub fn image() -> Self {
        Self {
            max_size: Some(2 * 1024 * 1024), // 2MB
            allowed_exts: vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "webp".to_string(),
            ],
            allowed_mimes: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
            ],
        }
    }
}

/// 生成唯一文件名
fn generate_unique_name() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let random: u32 = rand::random();
    format!("{:x}{:x}", now, random)
}
