//! Composer 管理器模块
//!
//! 统一管理 Composer 相关操作
//! 提供 PHP Composer 兼容的依赖管理功能
//!
//! # 功能特性
//! - 依赖安装 (install)
//! - 依赖更新 (update)
//! - 添加依赖 (require)
//! - 移除依赖 (remove)
//! - 自动加载生成 (dump-autoload)
//!
//! # 模块结构
//! - `types`: 类型定义
//! - `composer`: Composer 管理器主体

pub mod composer;
pub mod types;

// 重新导出主要类型
pub use composer::Composer;
pub use types::{Author, AutoloadConfig, ComposerConfig, ComposerJson, ComposerLock, LockedPackage, PackageSource, Repository};
