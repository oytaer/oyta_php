//! mbstring 多字节字符串模块入口
//!
//! 提供完整的 PHP mbstring 扩展功能
//! 包括：字符编码转换、多字节字符串操作、编码检测等
//!
//! # 模块结构
//! - `encoding`: 字符编码定义和检测
//! - `convert`: 编码转换功能
//! - `string`: 多字节字符串操作
//! - `detect`: 编码检测功能
//! - `http`: HTTP 输入输出编码处理
//! - `regex`: 多字节正则表达式

pub mod encoding;
pub mod convert;
pub mod string;
pub mod detect;
pub mod http;
pub mod regex;

// 重新导出主要类型
pub use encoding::{MbEncoding, MbEncodingList};
pub use convert::MbConverter;
pub use string::MbString;
pub use detect::MbDetector;
pub use http::MbHttpHandler;
pub use regex::MbRegex;
