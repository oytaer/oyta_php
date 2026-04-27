//! 内存安全沙箱隔离模块
//!
//! 提供安全的沙箱环境，用于执行不可信代码、插件系统、用户自定义逻辑等场景
//! 支持内存限制、时间限制、CPU限制、文件系统隔离、网络隔离等功能

// 引入子模块
pub mod executor;
pub mod resource;
pub mod filesystem;
pub mod network;
pub mod monitor;

// 重导出主要类型
pub use executor::{SandboxExecutor, SandboxConfig, SandboxResult};
pub use resource::{ResourceManager, ResourceLimits, ResourceUsage};
pub use filesystem::{FilesystemIsolation, FilePermission, PathRule};
pub use network::{NetworkIsolation, NetworkRule, NetworkPermission};
pub use monitor::{SandboxMonitor, MonitorStats, ViolationEvent};
