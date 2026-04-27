//! 文件系统隔离模块
//!
//! 提供沙箱的文件系统访问控制

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// 文件权限
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilePermission {
    /// 只读
    ReadOnly,
    /// 只写
    WriteOnly,
    /// 读写
    ReadWrite,
    /// 无权限
    None,
}

/// 路径规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRule {
    /// 路径模式（支持通配符）
    pub pattern: String,
    /// 权限
    pub permission: FilePermission,
    /// 是否递归
    pub recursive: bool,
}

impl PathRule {
    /// 创建新的路径规则
    pub fn new(pattern: &str, permission: FilePermission, recursive: bool) -> Self {
        Self {
            pattern: pattern.to_string(),
            permission,
            recursive,
        }
    }
    
    /// 检查路径是否匹配此规则
    pub fn matches(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        // 支持通配符匹配
        if self.pattern.contains('*') {
            // 简单的前缀匹配
            let parts: Vec<&str> = self.pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                
                if !prefix.is_empty() && !path_str.starts_with(prefix) {
                    return false;
                }
                if !suffix.is_empty() && !path_str.ends_with(suffix) {
                    return false;
                }
                return true;
            }
        }
        
        // 精确匹配
        path_str == self.pattern || path_str.starts_with(&format!("{}/", self.pattern))
    }
}

/// 文件系统隔离
/// 控制沙箱对文件系统的访问
#[derive(Debug, Clone)]
pub struct FilesystemIsolation {
    /// 路径规则列表
    rules: Vec<PathRule>,
    /// 虚拟根目录
    virtual_root: Option<PathBuf>,
    /// 是否启用隔离
    enabled: bool,
}

impl FilesystemIsolation {
    /// 创建新的文件系统隔离
    /// 
    /// # 参数
    /// - `rules`: 路径规则列表（字符串格式）
    pub fn new(rules: &[String]) -> Self {
        // 解析规则
        let parsed_rules: Vec<PathRule> = rules
            .iter()
            .filter_map(|r| Self::parse_rule(r))
            .collect();
        
        Self {
            rules: parsed_rules,
            virtual_root: None,
            enabled: true,
        }
    }
    
    /// 解析规则字符串
    fn parse_rule(rule: &str) -> Option<PathRule> {
        // 格式: "permission:path" 或 "permission:recursive:path"
        // 例如: "ro:/app/data", "rw:recursive:/app/logs"
        
        let parts: Vec<&str> = rule.split(':').collect();
        
        match parts.len() {
            2 => {
                // 简单格式: "permission:path"
                let permission = Self::parse_permission(parts[0])?;
                let pattern = parts[1].to_string();
                Some(PathRule::new(&pattern, permission, false))
            }
            3 => {
                // 带递归标志: "permission:recursive:path"
                let permission = Self::parse_permission(parts[0])?;
                let recursive = parts[1] == "recursive";
                let pattern = parts[2].to_string();
                Some(PathRule::new(&pattern, permission, recursive))
            }
            _ => None,
        }
    }
    
    /// 解析权限字符串
    fn parse_permission(s: &str) -> Option<FilePermission> {
        match s.to_lowercase().as_str() {
            "ro" | "readonly" | "r" => Some(FilePermission::ReadOnly),
            "wo" | "writeonly" | "w" => Some(FilePermission::WriteOnly),
            "rw" | "readwrite" => Some(FilePermission::ReadWrite),
            "none" | "deny" => Some(FilePermission::None),
            _ => None,
        }
    }
    
    /// 设置虚拟根目录
    pub fn set_virtual_root(&mut self, root: PathBuf) {
        self.virtual_root = Some(root);
    }
    
    /// 启用隔离
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    /// 禁用隔离
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    /// 检查是否允许读取
    /// 
    /// # 参数
    /// - `path`: 文件路径
    pub fn can_read(&self, path: &Path) -> bool {
        // 如果禁用隔离，允许所有操作
        if !self.enabled {
            return true;
        }
        
        // 查找匹配的规则
        for rule in &self.rules {
            if rule.matches(path) {
                return rule.permission == FilePermission::ReadOnly
                    || rule.permission == FilePermission::ReadWrite;
            }
        }
        
        // 默认拒绝
        false
    }
    
    /// 检查是否允许写入
    /// 
    /// # 参数
    /// - `path`: 文件路径
    pub fn can_write(&self, path: &Path) -> bool {
        // 如果禁用隔离，允许所有操作
        if !self.enabled {
            return true;
        }
        
        // 查找匹配的规则
        for rule in &self.rules {
            if rule.matches(path) {
                return rule.permission == FilePermission::WriteOnly
                    || rule.permission == FilePermission::ReadWrite;
            }
        }
        
        // 默认拒绝
        false
    }
    
    /// 检查是否允许访问
    /// 
    /// # 参数
    /// - `path`: 文件路径
    pub fn can_access(&self, path: &Path) -> bool {
        self.can_read(path) || self.can_write(path)
    }
    
    /// 获取路径权限
    /// 
    /// # 参数
    /// - `path`: 文件路径
    pub fn get_permission(&self, path: &Path) -> FilePermission {
        // 如果禁用隔离，返回读写权限
        if !self.enabled {
            return FilePermission::ReadWrite;
        }
        
        // 查找匹配的规则
        for rule in &self.rules {
            if rule.matches(path) {
                return rule.permission;
            }
        }
        
        // 默认无权限
        FilePermission::None
    }
    
    /// 添加路径规则
    pub fn add_rule(&mut self, rule: PathRule) {
        self.rules.push(rule);
    }
    
    /// 移除路径规则
    pub fn remove_rule(&mut self, pattern: &str) {
        self.rules.retain(|r| r.pattern != pattern);
    }
    
    /// 获取所有规则
    pub fn get_rules(&self) -> &[PathRule] {
        &self.rules
    }
    
    /// 将虚拟路径映射到真实路径
    /// 
    /// # 参数
    /// - `virtual_path`: 虚拟路径
    pub fn map_to_real(&self, virtual_path: &Path) -> PathBuf {
        if let Some(ref root) = self.virtual_root {
            // 将虚拟路径映射到真实根目录下
            root.join(virtual_path.strip_prefix("/").unwrap_or(virtual_path))
        } else {
            virtual_path.to_path_buf()
        }
    }
    
    /// 将真实路径映射到虚拟路径
    /// 
    /// # 参数
    /// - `real_path`: 真实路径
    pub fn map_to_virtual(&self, real_path: &Path) -> PathBuf {
        if let Some(ref root) = self.virtual_root {
            // 将真实路径映射回虚拟路径
            if let Ok(relative) = real_path.strip_prefix(root) {
                PathBuf::from("/").join(relative)
            } else {
                real_path.to_path_buf()
            }
        } else {
            real_path.to_path_buf()
        }
    }
    
    /// 验证路径安全性
    /// 检查路径是否包含目录遍历攻击
    /// 
    /// # 参数
    /// - `path`: 文件路径
    pub fn is_path_safe(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        // 检查目录遍历
        if path_str.contains("..") {
            return false;
        }
        
        // 检查绝对路径（如果设置了虚拟根目录）
        if self.virtual_root.is_some() && path.is_absolute() {
            // 只允许虚拟根目录下的路径
            let real_path = self.map_to_real(path);
            if let Some(ref root) = self.virtual_root {
                if !real_path.starts_with(root) {
                    return false;
                }
            }
        }
        
        true
    }
}

impl Default for FilesystemIsolation {
    fn default() -> Self {
        Self::new(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试路径规则匹配
    #[test]
    fn test_path_rule() {
        let rule = PathRule::new("/app/data/*", FilePermission::ReadOnly, true);
        
        assert!(rule.matches(Path::new("/app/data/file.txt")));
        assert!(rule.matches(Path::new("/app/data/subdir/file.txt")));
        assert!(!rule.matches(Path::new("/app/other/file.txt")));
    }
    
    /// 测试文件系统隔离
    #[test]
    fn test_isolation() {
        let rules = vec![
            "ro:/app/data".to_string(),
            "rw:/app/logs".to_string(),
        ];
        
        let isolation = FilesystemIsolation::new(&rules);
        
        // 只读路径
        assert!(isolation.can_read(Path::new("/app/data/file.txt")));
        assert!(!isolation.can_write(Path::new("/app/data/file.txt")));
        
        // 读写路径
        assert!(isolation.can_read(Path::new("/app/logs/app.log")));
        assert!(isolation.can_write(Path::new("/app/logs/app.log")));
        
        // 未定义路径
        assert!(!isolation.can_read(Path::new("/app/other/file.txt")));
    }
    
    /// 测试路径安全检查
    #[test]
    fn test_path_safety() {
        let isolation = FilesystemIsolation::new(&[]);
        
        // 安全路径
        assert!(isolation.is_path_safe(Path::new("/app/data/file.txt")));
        
        // 目录遍历攻击
        assert!(!isolation.is_path_safe(Path::new("/app/../etc/passwd")));
    }
    
    /// 测试虚拟根目录
    #[test]
    fn test_virtual_root() {
        let mut isolation = FilesystemIsolation::new(&[]);
        isolation.set_virtual_root(PathBuf::from("/sandbox/root"));
        
        let virtual_path = Path::new("/data/file.txt");
        let real_path = isolation.map_to_real(virtual_path);
        
        assert_eq!(real_path, PathBuf::from("/sandbox/root/data/file.txt"));
    }
}
