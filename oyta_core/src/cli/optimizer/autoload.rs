//! 自动加载优化模块
//!
//! 提供类映射的扫描和缓存生成功能

use std::collections::HashMap;
use std::path::Path;

/// 自动加载优化器
pub struct AutoloadOptimizer {
    /// 项目根目录
    project_root: std::path::PathBuf,
}

impl AutoloadOptimizer {
    /// 创建新的自动加载优化器
    ///
    /// # 参数
    /// - `project_root`: 项目根目录
    ///
    /// # 返回值
    /// 新的 AutoloadOptimizer 实例
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
        }
    }

    /// 扫描类文件
    ///
    /// # 参数
    /// - `dir`: 扫描目录
    /// - `class_map`: 类映射输出
    pub fn scan_classes(&self, dir: &Path, class_map: &mut HashMap<String, String>) -> anyhow::Result<()> {
        // 遍历目录
        for entry in std::fs::read_dir(dir)
            .map_err(|e| anyhow::anyhow!("无法读取目录: {} - {}", dir.display(), e))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // 递归扫描子目录
                self.scan_classes(&path, class_map)?;
            } else if path.extension().map(|e| e == "php").unwrap_or(false) {
                // 解析 PHP 文件提取类名
                if let Some(class_name) = self.extract_class_name(&path)? {
                    // 计算相对路径
                    let relative_path = path
                        .strip_prefix(&self.project_root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();

                    class_map.insert(class_name, relative_path);
                }
            }
        }

        Ok(())
    }

    /// 从 PHP 文件提取类名
    ///
    /// # 参数
    /// - `path`: PHP 文件路径
    ///
    /// # 返回
    /// 类名（包含命名空间）
    fn extract_class_name(&self, path: &Path) -> anyhow::Result<Option<String>> {
        // 读取文件内容
        let content = std::fs::read_to_string(path)?;

        let mut namespace = None;
        let mut class_name = None;

        // 遍历每一行
        for line in content.lines() {
            let line = line.trim();

            // 提取命名空间
            if line.starts_with("namespace ") {
                namespace = line
                    .trim_start_matches("namespace ")
                    .trim_end_matches(';')
                    .trim()
                    .to_string()
                    .into();
            }

            // 提取类名
            if line.starts_with("class ")
                || line.starts_with("interface ")
                || line.starts_with("trait ")
            {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    class_name = parts[1].to_string().into();
                    break;
                }
            }
        }

        // 组合完整类名
        if let Some(name) = class_name {
            if let Some(ns) = namespace {
                return Ok(Some(format!("{}\\{}", ns, name)));
            }
            return Ok(Some(name));
        }

        Ok(None)
    }

    /// 生成类映射缓存内容
    ///
    /// # 参数
    /// - `class_map`: 类映射
    ///
    /// # 返回
    /// PHP 缓存文件内容
    pub fn generate_cache_content(class_map: &HashMap<String, String>) -> String {
        let mut content = String::new();

        content.push_str("<?php\n");
        content.push_str("/**\n");
        content.push_str(" * 类映射缓存文件\n");
        content.push_str(" * 由 OYTAPHP 自动生成\n");
        content.push_str(" * 优化自动加载性能\n");
        content.push_str(" */\n\n");
        content.push_str("return [\n");

        for (class, path) in class_map {
            content.push_str(&format!("    '{}' => '{}',\n", class, path));
        }

        content.push_str("];\n");

        content
    }
}
