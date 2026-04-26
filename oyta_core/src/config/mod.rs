//! 配置系统模块
//!
//! 提供 OYTAPHP 的配置管理功能
//! 支持 PHP 配置文件、环境变量、figment 多源配置
//! 提供 Config 和 Env 门面供全局访问

pub mod facade;
pub mod loader;
pub mod store;
