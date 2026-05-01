//! 解释器引擎模块
//!
//! 将引擎功能按职责拆分为多个子模块：
//! - interpreter: 解释器核心结构体和主要入口方法
//! - stmt: 语句执行（if/while/for/switch/try等）
//! - expr: 表达式求值（算术/比较/函数调用/对象操作等）
//! - assign: 赋值操作（变量/属性/数组/静态属性赋值）
//! - method_call: 字符串和数组的链式方法调用
//! - builtins: 90+ 内置 PHP 函数实现
//! - builtins_ext: 扩展内置函数（DateTime、mbstring、GD、XML、Phar、eval等）
//! - builtin_class: 内置类子模块（DateTime、DOMDocument、GdImage、Phar等）
//! - builtin_classes: 内置类注册模块
//! - builtin_facade: 门面类子模块（Cache、Db、Session、Config、Log等）
//! - builtin_facades: 门面类注册模块
//! - binary_op: 二元运算符求值
//! - helpers: 通用辅助函数

pub mod assign;
pub mod binary_op;
pub mod builtins;
pub mod builtins_ext;
pub mod builtin_class;
pub mod builtin_classes;
pub mod builtin_facade;
pub mod builtin_facades;
pub mod expr;
pub mod helpers;
pub mod interpreter;
pub mod method_call;
pub mod stmt;

pub use interpreter::Interpreter;
pub use builtin_classes::{BuiltinClassRegistry, BuiltinClassDefinition, get_builtin_class_registry};
pub use builtin_facades::{FacadeClassRegistry, FacadeClassDefinition, get_facade_class_registry};
