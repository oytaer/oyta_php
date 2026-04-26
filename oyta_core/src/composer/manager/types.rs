//! Composer 类型定义模块
//!
//! 包含 composer.json 和 composer.lock 相关的数据结构定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// composer.json 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposerJson {
    /// 包名称
    pub name: String,
    /// 包描述
    pub description: String,
    /// 包类型
    #[serde(default)]
    pub r#type: String,
    /// 许可证
    #[serde(default)]
    pub license: Vec<String>,
    /// 作者列表
    #[serde(default)]
    pub authors: Vec<Author>,
    /// 需要的依赖
    #[serde(default)]
    pub require: HashMap<String, String>,
    /// 开发依赖
    #[serde(default)]
    pub require_dev: HashMap<String, String>,
    /// 自动加载配置
    #[serde(default)]
    pub autoload: AutoloadConfig,
    /// 开发环境自动加载配置
    #[serde(default)]
    pub autoload_dev: AutoloadConfig,
    /// 仓库配置
    #[serde(default)]
    pub repositories: Vec<Repository>,
    /// 最小稳定性
    #[serde(default = "default_stability")]
    pub minimum_stability: String,
    /// 优先稳定版本
    #[serde(default = "default_true")]
    pub prefer_stable: bool,
    /// 脚本配置
    #[serde(default)]
    pub scripts: HashMap<String, Vec<String>>,
    /// 额外配置
    #[serde(default)]
    pub extra: serde_json::Value,
    /// 配置选项
    #[serde(default)]
    pub config: ComposerConfig,
}

/// 为 ComposerJson 实现自定义 Default
/// 确保默认值符合预期
impl Default for ComposerJson {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            r#type: String::new(),
            license: Vec::new(),
            authors: Vec::new(),
            require: HashMap::new(),
            require_dev: HashMap::new(),
            autoload: AutoloadConfig::default(),
            autoload_dev: AutoloadConfig::default(),
            repositories: Vec::new(),
            minimum_stability: default_stability(),
            prefer_stable: default_true(),
            scripts: HashMap::new(),
            extra: serde_json::Value::Null,
            config: ComposerConfig::default(),
        }
    }
}

/// 默认稳定性
pub fn default_stability() -> String {
    "stable".to_string()
}

/// 默认 true
pub fn default_true() -> bool {
    true
}

/// 作者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// 作者名称
    pub name: String,
    /// 邮箱地址
    pub email: String,
    /// 主页
    #[serde(default)]
    pub homepage: Option<String>,
    /// 角色
    #[serde(default)]
    pub role: Option<String>,
}

/// 自动加载配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutoloadConfig {
    /// PSR-4 自动加载
    #[serde(default, rename = "psr-4")]
    pub psr_4: HashMap<String, Vec<String>>,
    /// PSR-0 自动加载
    #[serde(default, rename = "psr-0")]
    pub psr_0: HashMap<String, Vec<String>>,
    /// 类映射
    #[serde(default)]
    pub classmap: Vec<String>,
    /// 文件映射
    #[serde(default)]
    pub files: Vec<String>,
}

/// 仓库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// 仓库类型
    pub r#type: String,
    /// 仓库 URL
    pub url: String,
    /// 仓库名称
    #[serde(default)]
    pub name: Option<String>,
}

/// Composer 配置选项
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComposerConfig {
    /// 平台配置（覆盖 PHP 版本等）
    #[serde(default)]
    pub platform: HashMap<String, String>,
    /// 优化自动加载
    #[serde(default)]
    pub optimize_autoloader: bool,
    /// 类映射权威模式
    #[serde(default)]
    pub classmap_authoritative: bool,
    /// APCu 缓存
    #[serde(default)]
    pub apcu_autoloader: bool,
}

/// composer.lock 结构
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComposerLock {
    /// 锁文件版本
    #[serde(default)]
    pub version: i32,
    /// 内容哈希
    #[serde(default)]
    pub content_hash: String,
    /// 已安装的包
    #[serde(default)]
    pub packages: Vec<LockedPackage>,
    /// 开发环境包
    #[serde(default)]
    pub packages_dev: Vec<LockedPackage>,
    /// 平台要求
    #[serde(default)]
    pub platform: HashMap<String, String>,
    /// 平台开发要求
    #[serde(default)]
    pub platform_dev: HashMap<String, String>,
}

/// 已锁定的包信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    /// 包名称
    pub name: String,
    /// 版本号
    pub version: String,
    /// 源信息
    pub source: PackageSource,
    /// 安装路径
    #[serde(default)]
    pub install_path: String,
    /// 自动加载配置
    #[serde(default)]
    pub autoload: AutoloadConfig,
    /// 许可证
    #[serde(default)]
    pub license: Vec<String>,
    /// 依赖
    #[serde(default)]
    pub require: HashMap<String, String>,
}

/// 包源信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSource {
    /// 源类型（git/dist）
    pub r#type: String,
    /// 源 URL
    pub url: String,
    /// 引用（commit hash 或 dist checksum）
    pub reference: String,
}
