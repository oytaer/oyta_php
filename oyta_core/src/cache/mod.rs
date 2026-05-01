//! 缓存模块
//!
//! 提供 OYTAPHP 的缓存功能
//! 支持：内存缓存（moka）、文件缓存、Redis 缓存、多级缓存、缓存标签、智能缓存预热
//!
//! # 内部实现说明
//! - driver: 缓存驱动接口（内部实现）
//! - multi_level: 多级缓存实现（内部实现）
//! - redis_driver: Redis 驱动（内部实现）
//! - warmup: 缓存预热（内部实现）
//! - facade: 门面接口（对外暴露）

// 允许内部实现未使用警告
#![allow(dead_code)]

pub mod driver;
pub mod facade;
pub mod manager;
pub mod multi_level;
pub mod redis_driver;
pub mod tag;
pub mod warmup;
