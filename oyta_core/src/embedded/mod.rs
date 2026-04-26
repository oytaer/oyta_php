//! 内嵌服务模块
//!
//! 提供 WebSocket、定时器、队列、文件上传等内嵌服务
//! 在 public/index.php 中声明，随应用一起启动
//!
//! # 功能特性
//! - WebSocket 服务：房间管理、心跳检测、消息广播
//! - 定时器服务：every/once/daily/cron 定时任务
//! - 队列服务：异步任务处理、重试机制、Redis 后端
//! - 文件上传：文件验证、存储管理

pub mod cron;
pub mod queue;
pub mod queue_redis;
pub mod timer;
pub mod upload;
pub mod websocket;
