//! XML 处理模块入口
//!
//! 提供完整的 PHP XML 处理功能
//! 包括：DOM、SimpleXML、XMLReader、XMLWriter、SimplePie 等
//!
//! # 模块结构
//! - `dom`: DOMDocument 文档对象模型
//! - `simplexml`: SimpleXML 简单 XML 处理
//! - `reader`: XMLReader 流式 XML 读取（通过内置类暴露）
//! - `writer`: XMLWriter XML 写入（通过内置类暴露）
//! - `parser`: XML 解析器（内部实现）
//! - `xpath`: XPath 查询（通过内置类暴露）
//!
//! # 内部实现说明
//! - parser: XML 解析器内部实现
//! - dom/simplexml: DOM 和 SimpleXML 内部实现细节

// 允许内部实现未使用警告
#![allow(dead_code)]

pub mod dom;
pub mod simplexml;
pub mod reader;
pub mod writer;
pub mod parser;
pub mod xpath;

// 重新导出主要类型
pub use dom::XmlDocument;
pub use simplexml::SimpleXmlElement;
pub use reader::XmlReader;
pub use writer::XmlWriter;
pub use parser::XmlParser;
pub use xpath::XPath;
