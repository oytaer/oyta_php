//! PHP 8.0+ 语法特性支持
//!
//! 实现 PHP 8.0 及以上版本的新特性：
//! - match 表达式
//! - 命名参数
//! - Nullsafe 运算符 (?->)
//! - Attribute 注解
//! - 构造器属性提升
//! - readonly 属性
//! - 联合类型
//! - 匿名类
//!
//! # 模块结构
//! - `value_ext`: Value 辅助方法
//! - `match_expr`: Match 表达式
//! - `named_args`: 命名参数
//! - `nullsafe`: Nullsafe 运算符
//! - `attribute`: Attribute 注解
//! - `promoted_property`: 构造器属性提升
//! - `union_type`: 联合类型

pub mod attribute;
pub mod match_expr;
pub mod named_args;
pub mod nullsafe;
pub mod promoted_property;
pub mod union_type;
pub mod value_ext;

// 重新导出主要类型，方便外部使用
pub use attribute::{Attribute, AttributeParser, AttributeTarget};
pub use match_expr::{MatchArm, MatchBody, MatchExpression, MatchStatement};
pub use named_args::NamedArguments;
pub use nullsafe::{ChainCall, MethodCallContext, NullsafeOperator};
pub use promoted_property::{PromotedProperty, PropertyVisibility};
pub use union_type::UnionType;
