//! 安全模块
//!
//! 提供 OYTAPHP 的安全功能
//! 包括：加密解密、哈希、CSRF 防护、XSS 防护、SQL 注入防护

pub mod crypto;
pub mod csrf;
pub mod hash;
pub mod xss;
pub mod sql_injection;
pub mod crypt_facade;
pub mod hash_facade;
pub mod csrf_facade;

pub use crypt_facade::Crypt;
pub use hash_facade::Hash;
pub use csrf_facade::Csrf;
