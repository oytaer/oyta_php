//! 文件存储模块入口
//!
//! 提供统一的文件存储抽象层
//! 支持多种存储驱动：本地、S3、OSS、COS、FTP 等
//! 对应 ThinkPHP 8.0 的 Storage 门面功能

pub mod traits;
pub mod local;
// pub mod s3;
// pub mod oss;
// pub mod cos;
pub mod ftp;
pub mod manager;
pub mod facade;

// 重导出主要类型
pub use traits::{StorageDriver, StorageConfig, StorageResult, FileInfo};
pub use local::LocalStorage;
// pub use s3::S3Storage;
// pub use oss::OssStorage;
// pub use cos::CosStorage;
pub use ftp::FtpStorage;
pub use manager::StorageManager;
pub use facade::Storage;
