//! Phar 归档模块
//! 
//! 本模块实现 PHP Phar 归档功能，包括：
//! - Phar 文件创建和读取
//! - Phar 清单解析和生成
//! - Phar Stub 处理
//! - 文件压缩支持（Gzip、Bzip2）
//! - Phar 数据签名

pub mod compression;
pub mod manifest;
pub mod phar;
pub mod stub;

pub use compression::{PharCompression, PharCompressor};
pub use manifest::{PharManifest, PharEntry, PharEntryFlags};
pub use phar::{Phar, PharData, PharFileInfo};
pub use stub::{PharStub, PharStubBuilder};
