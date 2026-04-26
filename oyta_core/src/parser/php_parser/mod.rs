//! PHP 解析器封装模块
//!
//! 封装 php-rs-parser 和 php-ast 库，提供统一的 PHP 源码解析接口
//!
//! # 核心功能
//! 1. 解析 PHP 源码为 AST
//! 2. 遍历 AST 提取类、方法、属性、函数、命名空间等符号信息
//! 3. 将提取的符号信息转换为 OYTAPHP 内部的类型定义
//! 4. 支持解析 PHP 配置文件（return [] 格式）
//!
//! # 模块结构
//! - `parser`: 解析器主体
//! - `collector`: 符号收集器
//! - `extractors`: 符号提取函数
//! - `converters`: 类型转换函数
//! - `config_parser`: 配置文件解析
//! - `expr`: 表达式处理

pub mod collector;
pub mod config_parser;
pub mod converters;
pub mod expr;
pub mod extractors;
pub mod parser;

// 重新导出主要类型
pub use parser::PhpParser;
