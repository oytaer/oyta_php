//! 类型反射模块
//!
//! 提供 ReflectionNamedType, ReflectionUnionType, ReflectionIntersectionType 功能

use serde::{Deserialize, Serialize};
use std::fmt;

use super::metadata::TypeMetadata;

/// ReflectionNamedType 命名类型反射
///
/// 对应 PHP 的 ReflectionNamedType 类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectionNamedType {
    /// 类型元数据
    pub metadata: TypeMetadata,
}

impl ReflectionNamedType {
    /// 创建新的命名类型反射
    pub fn new(metadata: TypeMetadata) -> Self {
        Self { metadata }
    }
    
    /// 从名称创建
    pub fn from_name(name: &str) -> Self {
        Self {
            metadata: TypeMetadata::new(name),
        }
    }
    
    /// 创建可空类型
    pub fn nullable(name: &str) -> Self {
        Self {
            metadata: TypeMetadata::nullable(name),
        }
    }
    
    /// 获取类型名称
    ///
    /// 对应 PHP 的 ReflectionNamedType::getName()
    pub fn get_name(&self) -> &str {
        &self.metadata.name
    }
    
    /// 检查是否允许 null
    ///
    /// 对应 PHP 的 ReflectionType::allowsNull()
    pub fn allows_null(&self) -> bool {
        self.metadata.allows_null
    }
    
    /// 检查是否为内置类型
    ///
    /// 对应 PHP 的 ReflectionType::isBuiltin()
    pub fn is_builtin(&self) -> bool {
        self.metadata.is_builtin
    }
    
    /// 获取类型字符串表示
    ///
    /// 对应 PHP 的 ReflectionType::__toString()
    pub fn to_type_string(&self) -> String {
        if self.metadata.allows_null {
            format!("?{}", self.metadata.name)
        } else {
            self.metadata.name.clone()
        }
    }
    
    /// 检查是否为标量类型
    pub fn is_scalar(&self) -> bool {
        matches!(
            self.metadata.name.to_lowercase().as_str(),
            "int" | "integer" | "float" | "double" | "string" | "bool" | "boolean"
        )
    }
    
    /// 检查是否为数组类型
    pub fn is_array(&self) -> bool {
        self.metadata.name.eq_ignore_ascii_case("array")
    }
    
    /// 检查是否为对象类型
    pub fn is_object(&self) -> bool {
        self.metadata.name.eq_ignore_ascii_case("object") || !self.metadata.is_builtin
    }
    
    /// 检查是否为可调用类型
    pub fn is_callable(&self) -> bool {
        self.metadata.name.eq_ignore_ascii_case("callable")
    }
    
    /// 检查是否为 void 类型
    pub fn is_void(&self) -> bool {
        self.metadata.name.eq_ignore_ascii_case("void")
    }
    
    /// 检查是否为 mixed 类型
    pub fn is_mixed(&self) -> bool {
        self.metadata.name.eq_ignore_ascii_case("mixed")
    }
    
    /// 检查是否为 never 类型
    pub fn is_never(&self) -> bool {
        self.metadata.name.eq_ignore_ascii_case("never")
    }
    
    /// 检查是否为 iterable 类型
    pub fn is_iterable(&self) -> bool {
        self.metadata.name.eq_ignore_ascii_case("iterable")
    }
}

impl fmt::Display for ReflectionNamedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_type_string())
    }
}

/// ReflectionUnionType 联合类型反射
///
/// 对应 PHP 的 ReflectionUnionType 类
/// 表示联合类型，如 int|string
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectionUnionType {
    /// 联合类型列表
    pub types: Vec<ReflectionNamedType>,
}

impl ReflectionUnionType {
    /// 创建新的联合类型
    pub fn new(types: Vec<ReflectionNamedType>) -> Self {
        Self { types }
    }
    
    /// 从类型名称列表创建
    pub fn from_names(names: &[&str]) -> Self {
        Self {
            types: names.iter().map(|n| ReflectionNamedType::from_name(n)).collect(),
        }
    }
    
    /// 获取所有类型
    ///
    /// 对应 PHP 的 ReflectionUnionType::getTypes()
    pub fn get_types(&self) -> &[ReflectionNamedType] {
        &self.types
    }
    
    /// 检查是否允许 null
    ///
    /// 对应 PHP 的 ReflectionType::allowsNull()
    pub fn allows_null(&self) -> bool {
        self.types.iter().any(|t| t.metadata.name.eq_ignore_ascii_case("null"))
    }
    
    /// 获取类型数量
    pub fn count(&self) -> usize {
        self.types.len()
    }
    
    /// 检查是否包含指定类型
    pub fn contains(&self, type_name: &str) -> bool {
        self.types.iter().any(|t| t.metadata.name.eq_ignore_ascii_case(type_name))
    }
    
    /// 获取类型字符串表示
    pub fn to_type_string(&self) -> String {
        self.types
            .iter()
            .map(|t| t.metadata.name.clone())
            .collect::<Vec<_>>()
            .join("|")
    }
}

impl fmt::Display for ReflectionUnionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_type_string())
    }
}

/// ReflectionIntersectionType 交集类型反射
///
/// 对应 PHP 的 ReflectionIntersectionType 类
/// 表示交集类型，如 InterfaceA&InterfaceB
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectionIntersectionType {
    /// 交集类型列表
    pub types: Vec<ReflectionNamedType>,
}

impl ReflectionIntersectionType {
    /// 创建新的交集类型
    pub fn new(types: Vec<ReflectionNamedType>) -> Self {
        Self { types }
    }
    
    /// 从类型名称列表创建
    pub fn from_names(names: &[&str]) -> Self {
        Self {
            types: names.iter().map(|n| ReflectionNamedType::from_name(n)).collect(),
        }
    }
    
    /// 获取所有类型
    ///
    /// 对应 PHP 的 ReflectionIntersectionType::getTypes()
    pub fn get_types(&self) -> &[ReflectionNamedType] {
        &self.types
    }
    
    /// 检查是否允许 null
    ///
    /// 对应 PHP 的 ReflectionType::allowsNull()
    pub fn allows_null(&self) -> bool {
        false // 交集类型不允许 null
    }
    
    /// 获取类型数量
    pub fn count(&self) -> usize {
        self.types.len()
    }
    
    /// 检查是否包含指定类型
    pub fn contains(&self, type_name: &str) -> bool {
        self.types.iter().any(|t| t.metadata.name.eq_ignore_ascii_case(type_name))
    }
    
    /// 获取类型字符串表示
    pub fn to_type_string(&self) -> String {
        self.types
            .iter()
            .map(|t| t.metadata.name.clone())
            .collect::<Vec<_>>()
            .join("&")
    }
}

impl fmt::Display for ReflectionIntersectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_type_string())
    }
}

/// 类型反射枚举
///
/// 统一表示各种类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReflectionType {
    /// 命名类型
    Named(ReflectionNamedType),
    /// 联合类型
    Union(ReflectionUnionType),
    /// 交集类型
    Intersection(ReflectionIntersectionType),
}

impl ReflectionType {
    /// 从命名类型创建
    pub fn named(name: &str) -> Self {
        ReflectionType::Named(ReflectionNamedType::from_name(name))
    }
    
    /// 从联合类型创建
    pub fn union(names: &[&str]) -> Self {
        ReflectionType::Union(ReflectionUnionType::from_names(names))
    }
    
    /// 从交集类型创建
    pub fn intersection(names: &[&str]) -> Self {
        ReflectionType::Intersection(ReflectionIntersectionType::from_names(names))
    }
    
    /// 检查是否允许 null
    pub fn allows_null(&self) -> bool {
        match self {
            ReflectionType::Named(t) => t.allows_null(),
            ReflectionType::Union(t) => t.allows_null(),
            ReflectionType::Intersection(t) => t.allows_null(),
        }
    }
    
    /// 获取类型字符串
    pub fn to_type_string(&self) -> String {
        match self {
            ReflectionType::Named(t) => t.to_type_string(),
            ReflectionType::Union(t) => t.to_type_string(),
            ReflectionType::Intersection(t) => t.to_type_string(),
        }
    }
    
    /// 检查是否为命名类型
    pub fn is_named(&self) -> bool {
        matches!(self, ReflectionType::Named(_))
    }
    
    /// 检查是否为联合类型
    pub fn is_union(&self) -> bool {
        matches!(self, ReflectionType::Union(_))
    }
    
    /// 检查是否为交集类型
    pub fn is_intersection(&self) -> bool {
        matches!(self, ReflectionType::Intersection(_))
    }
    
    /// 获取命名类型（如果是）
    pub fn as_named(&self) -> Option<&ReflectionNamedType> {
        match self {
            ReflectionType::Named(t) => Some(t),
            _ => None,
        }
    }
    
    /// 获取联合类型（如果是）
    pub fn as_union(&self) -> Option<&ReflectionUnionType> {
        match self {
            ReflectionType::Union(t) => Some(t),
            _ => None,
        }
    }
    
    /// 获取交集类型（如果是）
    pub fn as_intersection(&self) -> Option<&ReflectionIntersectionType> {
        match self {
            ReflectionType::Intersection(t) => Some(t),
            _ => None,
        }
    }
}

impl fmt::Display for ReflectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_type_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_named_type() {
        let int_type = ReflectionNamedType::from_name("int");
        
        assert_eq!(int_type.get_name(), "int");
        assert!(int_type.is_builtin());
        assert!(int_type.is_scalar());
        assert!(!int_type.allows_null());
    }
    
    #[test]
    fn test_nullable_type() {
        let nullable_string = ReflectionNamedType::nullable("string");
        
        assert_eq!(nullable_string.get_name(), "string");
        assert!(nullable_string.allows_null());
        assert_eq!(nullable_string.to_type_string(), "?string");
    }
    
    #[test]
    fn test_union_type() {
        let union = ReflectionUnionType::from_names(&["int", "string", "null"]);
        
        assert_eq!(union.count(), 3);
        assert!(union.allows_null());
        assert!(union.contains("int"));
        assert!(union.contains("string"));
        assert_eq!(union.to_type_string(), "int|string|null");
    }
    
    #[test]
    fn test_intersection_type() {
        let intersection = ReflectionIntersectionType::from_names(&["Countable", "Iterator"]);
        
        assert_eq!(intersection.count(), 2);
        assert!(!intersection.allows_null());
        assert!(intersection.contains("Countable"));
        assert_eq!(intersection.to_type_string(), "Countable&Iterator");
    }
    
    #[test]
    fn test_reflection_type_enum() {
        let named = ReflectionType::named("int");
        assert!(named.is_named());
        assert_eq!(named.to_type_string(), "int");
        
        let union = ReflectionType::union(&["int", "string"]);
        assert!(union.is_union());
        assert_eq!(union.to_type_string(), "int|string");
        
        let intersection = ReflectionType::intersection(&["A", "B"]);
        assert!(intersection.is_intersection());
        assert_eq!(intersection.to_type_string(), "A&B");
    }
}
