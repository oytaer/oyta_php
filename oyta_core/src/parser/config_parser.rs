//! PHP 配置文件解析模块
//!
//! 专门用于解析 ThinkPHP 风格的 PHP 配置文件
//! 配置文件格式为 `return [...]`，返回关联数组
//! 支持嵌套数组、字面量值、常量引用等

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::php_parser::PhpParser;
use crate::symbol_table::types::ConfigValue;

/// 配置文件解析器
/// 负责扫描和解析 config/ 目录下的所有 PHP 配置文件
/// 将配置文件名作为键名前缀，如 config/app.php → "app" 前缀
pub struct ConfigParser {
    /// 配置目录路径
    config_dir: PathBuf,
}

impl ConfigParser {
    /// 创建新的配置解析器
    ///
    /// # 参数
    /// - `config_dir`: 配置文件目录路径（通常是项目根目录下的 config/）
    pub fn new(config_dir: &Path) -> Self {
        Self {
            config_dir: config_dir.to_path_buf(),
        }
    }

    /// 解析所有配置文件
    /// 扫描 config/ 目录下的所有 .php 文件并解析
    /// 返回以文件名（不含扩展名）为键的配置映射
    ///
    /// # 返回
    /// HashMap，键为配置文件名（如 "app"、"database"），值为配置内容
    pub fn parse_all(&self, parser: &mut PhpParser) -> Result<HashMap<String, ConfigValue>> {
        let mut configs = HashMap::new();

        // 检查配置目录是否存在
        if !self.config_dir.is_dir() {
            tracing::warn!("配置目录不存在: {}", self.config_dir.display());
            return Ok(configs);
        }

        // 遍历配置目录下的所有 .php 文件
        let entries = std::fs::read_dir(&self.config_dir)
            .with_context(|| format!("无法读取配置目录: {}", self.config_dir.display()))?;

        for entry in entries.flatten() {
            let path = entry.path();

            // 只处理 .php 文件
            if path.extension().and_then(|e| e.to_str()) != Some("php") {
                continue;
            }

            // 获取文件名（不含扩展名）作为配置键名
            let config_name = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            // 解析配置文件
            match parser.parse_config_file(&path) {
                Ok(value) => {
                    tracing::debug!("配置文件解析成功: {} → {:?}", config_name, value);
                    configs.insert(config_name.clone(), value);
                }
                Err(e) => {
                    tracing::warn!("配置文件解析失败: {} - {}", path.display(), e);
                }
            }
        }

        tracing::debug!("已加载 {} 个配置文件", configs.len());
        Ok(configs)
    }

    /// 解析单个配置文件
    ///
    /// # 参数
    /// - `name`: 配置文件名（不含扩展名），如 "app"、"database"
    /// - `parser`: PHP 解析器实例
    pub fn parse_one(&self, name: &str, parser: &mut PhpParser) -> Result<ConfigValue> {
        let path = self.config_dir.join(format!("{}.php", name));
        parser.parse_config_file(&path)
    }

    /// 获取所有配置文件名列表
    pub fn config_names(&self) -> Vec<String> {
        let mut names = Vec::new();

        if !self.config_dir.is_dir() {
            return names;
        }

        if let Ok(entries) = std::fs::read_dir(&self.config_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("php") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        names.push(name.to_string());
                    }
                }
            }
        }

        // 按字母排序
        names.sort();
        names
    }
}

/// 配置值便捷访问方法
/// 提供从 ConfigValue 中安全获取嵌套值的方法
impl ConfigValue {
    /// 使用点号路径获取嵌套配置值
    /// 如 "database.connections.mysql.host"
    ///
    /// # 参数
    /// - `path`: 点号分隔的配置路径
    ///
    /// # 返回
    /// 如果路径存在则返回对应的配置值，否则返回 None
    pub fn get_by_path(&self, path: &str) -> Option<&ConfigValue> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        let mut current = self;
        for part in parts {
            current = current.get(part)?;
        }
        Some(current)
    }

    /// 使用点号路径获取字符串值
    pub fn get_string(&self, path: &str) -> Option<String> {
        self.get_by_path(path).and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    /// 使用点号路径获取整数值
    pub fn get_int(&self, path: &str) -> Option<i64> {
        self.get_by_path(path).and_then(|v| v.as_i64())
    }

    /// 使用点号路径获取布尔值
    pub fn get_bool(&self, path: &str) -> Option<bool> {
        self.get_by_path(path).and_then(|v| v.as_bool())
    }
}
