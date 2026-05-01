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
//!
//! # 内部实现说明
//! - distributed_cache: 分布式缓存实现（内部实现）
//! - distributed_lock: 分布式锁实现（通过 Lock 门面暴露）
//! - distributed_session: 分布式会话实现（通过 Session 门面暴露）
//! - service_discovery: 服务发现（内部实现）

// 允许内部实现未使用警告
#![allow(dead_code)]

pub mod distributed_cache;
pub mod distributed_lock;
pub mod distributed_session;
pub mod service_discovery;
