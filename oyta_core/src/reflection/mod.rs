//! Reflection 反射系统模块入口
//!
//! 提供完整的 PHP Reflection 功能
//! 包括：ReflectionClass, ReflectionMethod, ReflectionProperty 等
//!
//! # 模块结构
//! - `class`: ReflectionClass 类反射
//! - `method`: ReflectionMethod 方法反射
//! - `property`: ReflectionProperty 属性反射
//! - `parameter`: ReflectionParameter 参数反射
//! - `function`: ReflectionFunction 函数反射
//! - `attribute`: ReflectionAttribute 注解反射
//! - `type`: 类型反射（ReflectionNamedType, ReflectionUnionType 等）
//! - `enum_ref`: 枚举反射
//! - `metadata`: 元数据管理

pub mod class;
pub mod method;
pub mod property;
pub mod parameter;
pub mod function;
pub mod attribute;
pub mod r#type;
pub mod enum_ref;
pub mod metadata;

// 重新导出主要类型
pub use class::ReflectionClass;
pub use method::ReflectionMethod;
pub use property::ReflectionProperty;
pub use parameter::ReflectionParameter;
pub use function::{ReflectionFunction, ReflectionFunctionAbstract};
pub use attribute::ReflectionAttribute;
pub use r#type::{ReflectionNamedType, ReflectionUnionType, ReflectionIntersectionType};
pub use enum_ref::ReflectionEnum;
pub use metadata::ReflectionMetadata;
