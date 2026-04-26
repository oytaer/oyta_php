//! 分布式支持模块
//!
//! 提供分布式部署所需的功能：
//! - 分布式 Session
//! - 分布式缓存
//! - 分布式锁
//! - 服务发现与注册
//!
//! # 功能特性
//! - Redis Session 共享
//! - 分布式锁（Redlock）
//! - 缓存一致性
//! - 服务健康检查

pub mod distributed_cache;
pub mod distributed_lock;
pub mod distributed_session;
pub mod service_discovery;
