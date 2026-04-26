//! PHP 解析器模块
//!
//! 提供 PHP 源码解析、配置文件解析、文件扫描等功能
//! 基于 php-rs-parser 和 php-ast 库实现

pub mod config_parser;
pub mod php_parser;
pub mod scanner;
