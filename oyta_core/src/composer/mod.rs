//! Composer 依赖管理模块
//!
//! 提供 PHP Composer 兼容的依赖管理功能
//! 支持 composer.json 解析、依赖解析、包下载、自动加载生成
//!
//! # 功能特性
//! - composer.json/composer.lock 解析
//! - Packagist API 集成
//! - 依赖版本解析（语义化版本）
//! - 包下载与解压
//! - PSR-4 自动加载生成
//!
//! # 命令支持
//! - `oyta composer install` - 安装依赖
//! - `oyta composer update` - 更新依赖
//! - `oyta composer require vendor/package` - 添加依赖
//! - `oyta composer remove vendor/package` - 移除依赖
//! - `oyta composer dump-autoload` - 重新生成自动加载

pub mod autoload;
pub mod downloader;
pub mod extractor;
pub mod lock;
pub mod manager;
pub mod packagist;
pub mod parser;
pub mod resolver;

// 重新导出常用类型
pub use manager::{
    AutoloadConfig,
    Composer,
    ComposerJson,
    LockedPackage,
};
