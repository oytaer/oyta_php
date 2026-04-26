//! 解释器模块
//!
//! 提供 PHP 代码的解释执行能力
//! 基于 php-ast 的 AST 节点进行遍历执行
//! 支持：变量操作、控制流、函数调用、类方法调用等
//!
//! # PHP 8.0+ 特性支持
//! - match 表达式
//! - 命名参数
//! - Nullsafe 运算符 (?->)
//! - Attribute 注解
//! - 构造器属性提升
//! - 联合类型

pub mod engine;
pub mod php8_features;
pub mod value;
