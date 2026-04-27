//! ReflectionProperty 属性反射模块
//!
//! 提供完整的 PHP ReflectionProperty 功能

use serde::{Deserialize, Serialize};
use std::fmt;

use super::metadata::{PropertyMetadata, Visibility};
use super::attribute::ReflectionAttribute;
use super::r#type::ReflectionNamedType;

/// ReflectionProperty 属性反射
///
/// 对应 PHP 的 ReflectionProperty 类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectionProperty {
    /// 属性元数据
    pub metadata: PropertyMetadata,
}

impl ReflectionProperty {
    /// 创建新的属性反射
    pub fn new(metadata: PropertyMetadata) -> Self {
        Self { metadata }
    }
    
    /// 从类名和属性名创建
    pub fn from_name(class_name: &str, property_name: &str) -> Self {
        Self {
            metadata: PropertyMetadata::new(property_name, class_name),
        }
    }
    
    /// 获取属性名
    ///
    /// 对应 PHP 的 ReflectionProperty::getName()
    pub fn get_name(&self) -> &str {
        &self.metadata.name
    }
    
    /// 获取属性名（不含 $ 前缀）
    pub fn get_name_without_dollar(&self) -> &str {
        self.metadata.name_without_dollar()
    }
    
    /// 获取所属类名
    ///
    /// 对应 PHP 的 ReflectionProperty::getDeclaringClass()
    pub fn get_declaring_class_name(&self) -> &str {
        &self.metadata.class_name
    }
    
    /// 检查是否为公开属性
    ///
    /// 对应 PHP 的 ReflectionProperty::isPublic()
    pub fn is_public(&self) -> bool {
        self.metadata.visibility == Visibility::Public
    }
    
    /// 检查是否为受保护属性
    ///
    /// 对应 PHP 的 ReflectionProperty::isProtected()
    pub fn is_protected(&self) -> bool {
        self.metadata.visibility == Visibility::Protected
    }
    
    /// 检查是否为私有属性
    ///
    /// 对应 PHP 的 ReflectionProperty::isPrivate()
    pub fn is_private(&self) -> bool {
        self.metadata.visibility == Visibility::Private
    }
    
    /// 检查是否为静态属性
    ///
    /// 对应 PHP 的 ReflectionProperty::isStatic()
    pub fn is_static(&self) -> bool {
        self.metadata.is_static
    }
    
    /// 检查是否为只读属性
    ///
    /// 对应 PHP 的 ReflectionProperty::isReadOnly()
    pub fn is_readonly(&self) -> bool {
        self.metadata.is_readonly
    }
    
    /// 获取可见性
    pub fn get_visibility(&self) -> Visibility {
        self.metadata.visibility
    }
    
    /// 获取属性类型
    ///
    /// 对应 PHP 的 ReflectionProperty::getType()
    pub fn get_type(&self) -> Option<ReflectionNamedType> {
        self.metadata.property_type.as_ref().map(|t| {
            ReflectionNamedType::new(t.clone())
        })
    }
    
    /// 检查是否有类型声明
    ///
    /// 对应 PHP 的 ReflectionProperty::hasType()
    pub fn has_type(&self) -> bool {
        self.metadata.property_type.is_some()
    }
    
    /// 获取默认值
    ///
    /// 对应 PHP 的 ReflectionProperty::getDefaultValue()
    pub fn get_default_value(&self) -> Option<&str> {
        self.metadata.default_value.as_deref()
    }
    
    /// 检查是否有默认值
    ///
    /// 对应 PHP 的 ReflectionProperty::hasDefaultValue()
    pub fn has_default_value(&self) -> bool {
        self.metadata.default_value.is_some()
    }
    
    /// 获取注解列表
    ///
    /// 对应 PHP 的 ReflectionProperty::getAttributes()
    pub fn get_attributes(&self, name: Option<&str>) -> Vec<ReflectionAttribute> {
        self.metadata.attributes
            .iter()
            .filter(|a| name.is_none() || a.name == name.unwrap())
            .map(|a| ReflectionAttribute::new(a.clone()))
            .collect()
    }
    
    /// 获取 DocBlock 注释
    ///
    /// 对应 PHP 的 ReflectionProperty::getDocComment()
    pub fn get_doc_comment(&self) -> Option<&str> {
        self.metadata.doc_comment.as_deref()
    }
    
    /// 获取修改器
    ///
    /// 对应 PHP 的 ReflectionProperty::getModifiers()
    pub fn get_modifiers(&self) -> PropertyModifiers {
        let mut modifiers = PropertyModifiers::empty();
        
        match self.metadata.visibility {
            Visibility::Public => modifiers |= PropertyModifiers::PUBLIC,
            Visibility::Protected => modifiers |= PropertyModifiers::PROTECTED,
            Visibility::Private => modifiers |= PropertyModifiers::PRIVATE,
        }
        
        if self.metadata.is_static {
            modifiers |= PropertyModifiers::STATIC;
        }
        if self.metadata.is_readonly {
            modifiers |= PropertyModifiers::READONLY;
        }
        
        modifiers
    }
    
    /// 检查属性是否已初始化
    ///
    /// 对应 PHP 的 ReflectionProperty::isInitialized()
    pub fn is_initialized(&self) -> bool {
        // 如果有类型声明但没有默认值，可能未初始化
        self.metadata.property_type.is_none() || self.metadata.default_value.is_some()
    }
    
    /// 检查是否可以从声明类外部访问
    pub fn is_accessible_from(&self, class_name: &str) -> bool {
        match self.metadata.visibility {
            Visibility::Public => true,
            Visibility::Protected => {
                // 受保护属性可从子类访问
                // 在实际实现中需要检查继承关系
                self.metadata.class_name == class_name
            }
            Visibility::Private => {
                // 私有属性只能从声明类访问
                self.metadata.class_name == class_name
            }
        }
    }
    
    /// 设置属性可访问
    ///
    /// 对应 PHP 的 ReflectionProperty::setAccessible()
    pub fn set_accessible(&mut self, _accessible: bool) {
        // 在 Rust 实现中，这个方法主要用于标记
    }
}

impl fmt::Display for ReflectionProperty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Property [ ")?;
        
        if self.metadata.is_static {
            write!(f, "static ")?;
        }
        if self.metadata.is_readonly {
            write!(f, "readonly ")?;
        }
        
        write!(f, "{} ", self.metadata.visibility)?;
        
        if let Some(t) = &self.metadata.property_type {
            write!(f, "{} ", t.name)?;
        }
        
        write!(f, "{} ]", self.metadata.name)?;
        
        if let Some(default) = &self.metadata.default_value {
            write!(f, " = {}", default)?;
        }
        
        Ok(())
    }
}

/// 属性修改器标志
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PropertyModifiers: u32 {
        /// 公开属性
        const PUBLIC = 1;
        /// 受保护属性
        const PROTECTED = 2;
        /// 私有属性
        const PRIVATE = 4;
        /// 静态属性
        const STATIC = 8;
        /// 只读属性
        const READONLY = 16;
    }
}

impl Default for PropertyModifiers {
    fn default() -> Self {
        Self::PUBLIC
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_property() -> ReflectionProperty {
        let mut metadata = PropertyMetadata::new("name", "User");
        metadata.visibility = Visibility::Private;
        metadata.is_readonly = true;
        metadata.default_value = Some("'default'".to_string());
        
        ReflectionProperty::new(metadata)
    }
    
    #[test]
    fn test_property_name() {
        let property = create_test_property();
        
        assert_eq!(property.get_name(), "$name");
        assert_eq!(property.get_name_without_dollar(), "name");
        assert_eq!(property.get_declaring_class_name(), "User");
    }
    
    #[test]
    fn test_property_visibility() {
        let property = create_test_property();
        
        assert!(!property.is_public());
        assert!(!property.is_protected());
        assert!(property.is_private());
    }
    
    #[test]
    fn test_property_modifiers() {
        let property = create_test_property();
        
        assert!(property.is_readonly());
        assert!(!property.is_static());
    }
    
    #[test]
    fn test_property_default_value() {
        let property = create_test_property();
        
        assert!(property.has_default_value());
        assert_eq!(property.get_default_value(), Some("'default'"));
    }
    
    #[test]
    fn test_property_modifiers_flags() {
        let property = create_test_property();
        let modifiers = property.get_modifiers();
        
        assert!(modifiers.contains(PropertyModifiers::PRIVATE));
        assert!(modifiers.contains(PropertyModifiers::READONLY));
        assert!(!modifiers.contains(PropertyModifiers::STATIC));
    }
}
