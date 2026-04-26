//! 类型定义模块
//!
//! 包含命令处理中使用的各种类型定义

use serde::{Deserialize, Serialize};

/// 过时包信息
///
/// 用于存储检查更新时发现的过时包的详细信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedPackageInfo {
    /// 包名
    pub name: String,
    /// 当前安装的版本
    pub current_version: String,
    /// 最新可用版本
    pub latest_version: String,
    /// 包描述
    #[serde(default)]
    pub description: String,
}

/// 最新版本信息
///
/// 从 Packagist API 获取的最新版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestVersionInfo {
    /// 最新版本号
    pub latest_version: String,
    /// 是否已过时
    pub is_outdated: bool,
    /// 包描述
    #[serde(default)]
    pub description: String,
}

/// 可发布资源项
///
/// 用于 vendor:publish 命令的可发布资源信息
#[derive(Debug, Clone)]
pub struct PublishableItem {
    /// 所属包名
    pub package: String,
    /// 源文件路径
    pub source: std::path::PathBuf,
    /// 目标相对路径
    pub destination: String,
    /// 资源类型
    pub resource_type: PublishableType,
}

/// 可发布资源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishableType {
    /// 配置文件
    Config,
    /// 视图文件
    Views,
    /// 语言文件
    Lang,
    /// 数据库迁移
    Migrations,
    /// 其他资源
    Assets,
}

/// 服务提供者信息
///
/// 用于 service:discover 命令发现的服务提供者信息
#[derive(Debug, Clone)]
pub struct ServiceProviderInfo {
    /// 服务提供者类名
    pub name: String,
    /// 所属包名
    pub package: String,
    /// 服务提供者文件路径
    pub path: std::path::PathBuf,
    /// 是否延迟加载
    pub deferred: bool,
    /// 提供的服务列表
    pub provides: Vec<String>,
}
