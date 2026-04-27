//! ReflectionEnum 枚举反射模块
//!
//! 提供完整的 PHP ReflectionEnum 功能

use serde::{Deserialize, Serialize};
use std::fmt;

use super::metadata::{EnumMetadata, EnumCaseMetadata, MethodMetadata, ConstantMetadata};
use super::method::ReflectionMethod;
use super::attribute::ReflectionAttribute;

/// ReflectionEnum 枚举反射
///
/// 对应 PHP 的 ReflectionEnum 类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectionEnum {
    /// 枚举元数据
    pub metadata: EnumMetadata,
}

impl ReflectionEnum {
    /// 创建新的枚举反射
    pub fn new(metadata: EnumMetadata) -> Self {
        Self { metadata }
    }
    
    /// 从名称创建
    pub fn from_name(name: &str) -> Self {
        Self {
            metadata: EnumMetadata::new(name),
        }
    }
    
    /// 获取枚举名
    ///
    /// 对应 PHP 的 ReflectionEnum::getName()
    pub fn get_name(&self) -> &str {
        &self.metadata.name
    }
    
    /// 获取短名称
    pub fn get_short_name(&self) -> &str {
        &self.metadata.short_name
    }
    
    /// 获取命名空间
    pub fn get_namespace_name(&self) -> Option<&str> {
        self.metadata.namespace.as_deref()
    }
    
    /// 检查是否在命名空间中
    pub fn in_namespace(&self) -> bool {
        self.metadata.namespace.is_some()
    }
    
    /// 检查是否为 Backed Enum
    ///
    /// 对应 PHP 的 ReflectionEnum::isBacked()
    pub fn is_backed(&self) -> bool {
        self.metadata.is_backed()
    }
    
    /// 获取底层类型
    ///
    /// 对应 PHP 的 ReflectionEnum::getBackingType()
    pub fn get_backing_type(&self) -> Option<&str> {
        self.metadata.backing_type.as_deref()
    }
    
    /// 获取 Case
    ///
    /// 对应 PHP 的 ReflectionEnum::getCase()
    ///
    /// # 参数
    /// - `name`: Case 名称
    pub fn get_case(&self, name: &str) -> Option<ReflectionEnumCase> {
        self.metadata.find_case(name).map(|c| {
            ReflectionEnumCase::new(c.clone(), self.metadata.name.clone())
        })
    }
    
    /// 获取所有 Case
    ///
    /// 对应 PHP 的 ReflectionEnum::getCases()
    pub fn get_cases(&self) -> Vec<ReflectionEnumCase> {
        self.metadata.cases
            .iter()
            .map(|c| ReflectionEnumCase::new(c.clone(), self.metadata.name.clone()))
            .collect()
    }
    
    /// 检查是否有 Case
    ///
    /// 对应 PHP 的 ReflectionEnum::hasCase()
    pub fn has_case(&self, name: &str) -> bool {
        self.metadata.find_case(name).is_some()
    }
    
    /// 获取 Case 数量
    pub fn get_case_count(&self) -> usize {
        self.metadata.cases.len()
    }
    
    /// 获取方法
    pub fn get_method(&self, name: &str) -> Option<ReflectionMethod> {
        self.metadata.methods
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case(name))
            .map(|m| ReflectionMethod::new(m.clone()))
    }
    
    /// 获取所有方法
    pub fn get_methods(&self) -> Vec<ReflectionMethod> {
        self.metadata.methods
            .iter()
            .map(|m| ReflectionMethod::new(m.clone()))
            .collect()
    }
    
    /// 检查是否有方法
    pub fn has_method(&self, name: &str) -> bool {
        self.metadata.methods
            .iter()
            .any(|m| m.name.eq_ignore_ascii_case(name))
    }
    
    /// 获取常量
    pub fn get_constant(&self, name: &str) -> Option<&ConstantMetadata> {
        self.metadata.constants.iter().find(|c| c.name == name)
    }
    
    /// 获取所有常量
    pub fn get_constants(&self) -> &[ConstantMetadata] {
        &self.metadata.constants
    }
    
    /// 检查是否有常量
    pub fn has_constant(&self, name: &str) -> bool {
        self.metadata.constants.iter().any(|c| c.name == name)
    }
    
    /// 获取实现的接口
    pub fn get_interface_names(&self) -> &[String] {
        &self.metadata.interfaces
    }
    
    /// 检查是否实现接口
    pub fn implements_interface(&self, interface_name: &str) -> bool {
        self.metadata.interfaces
            .iter()
            .any(|i| i.eq_ignore_ascii_case(interface_name))
    }
    
    /// 获取使用的 Trait
    pub fn get_trait_names(&self) -> &[String] {
        &self.metadata.traits
    }
    
    /// 检查是否使用 Trait
    pub fn uses_trait(&self, trait_name: &str) -> bool {
        self.metadata.traits
            .iter()
            .any(|t| t.eq_ignore_ascii_case(trait_name))
    }
    
    /// 获取注解列表
    pub fn get_attributes(&self, name: Option<&str>) -> Vec<ReflectionAttribute> {
        self.metadata.attributes
            .iter()
            .filter(|a| name.is_none() || a.name == name.unwrap())
            .map(|a| ReflectionAttribute::new(a.clone()))
            .collect()
    }
    
    /// 获取 DocBlock 注释
    pub fn get_doc_comment(&self) -> Option<&str> {
        self.metadata.doc_comment.as_deref()
    }
    
    /// 获取源文件路径
    pub fn get_file_name(&self) -> Option<&str> {
        self.metadata.file_name.as_deref()
    }
    
    /// 检查是否为内部枚举
    pub fn is_internal(&self) -> bool {
        self.metadata.file_name.is_none()
    }
    
    /// 检查是否为用户定义枚举
    pub fn is_user_defined(&self) -> bool {
        self.metadata.file_name.is_some()
    }
}

impl fmt::Display for ReflectionEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Enum [ {}", self.metadata.name)?;
        
        if let Some(backing_type) = &self.metadata.backing_type {
            write!(f, " : {}", backing_type)?;
        }
        
        write!(f, " ]")?;
        
        if !self.metadata.interfaces.is_empty() {
            write!(f, " implements {}", self.metadata.interfaces.join(", "))?;
        }
        
        Ok(())
    }
}

/// ReflectionEnumCase 枚举 Case 反射
///
/// 对应 PHP 的 ReflectionEnumUnitCase 或 ReflectionEnumBackedCase
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectionEnumCase {
    /// Case 元数据
    pub metadata: EnumCaseMetadata,
    /// 所属枚举名
    pub enum_name: String,
}

impl ReflectionEnumCase {
    /// 创建新的 Case 反射
    pub fn new(metadata: EnumCaseMetadata, enum_name: String) -> Self {
        Self { metadata, enum_name }
    }
    
    /// 获取 Case 名称
    ///
    /// 对应 PHP 的 ReflectionEnumCase::getName()
    pub fn get_name(&self) -> &str {
        &self.metadata.name
    }
    
    /// 获取所属枚举名
    ///
    /// 对应 PHP 的 ReflectionEnumCase::getEnum()
    pub fn get_enum_name(&self) -> &str {
        &self.enum_name
    }
    
    /// 获取 Case 值
    ///
    /// 对应 PHP 的 ReflectionEnumBackedCase::getBackingValue()
    pub fn get_value(&self) -> Option<&str> {
        self.metadata.value.as_deref()
    }
    
    /// 检查是否有值
    pub fn has_value(&self) -> bool {
        self.metadata.value.is_some()
    }
    
    /// 获取注解列表
    pub fn get_attributes(&self, name: Option<&str>) -> Vec<ReflectionAttribute> {
        self.metadata.attributes
            .iter()
            .filter(|a| name.is_none() || a.name == name.unwrap())
            .map(|a| ReflectionAttribute::new(a.clone()))
            .collect()
    }
    
    /// 获取完整名称（枚举名::Case名）
    pub fn get_full_name(&self) -> String {
        format!("{}::{}", self.enum_name, self.metadata.name)
    }
}

impl fmt::Display for ReflectionEnumCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EnumCase [ {}::{}", self.enum_name, self.metadata.name)?;
        
        if let Some(value) = &self.metadata.value {
            write!(f, " = {}", value)?;
        }
        
        write!(f, " ]")
    }
}

/// ReflectionEnumBackedCase Backed Enum Case 反射
///
/// 对应 PHP 的 ReflectionEnumBackedCase 类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectionEnumBackedCase {
    /// 内部 Case 反射
    pub inner: ReflectionEnumCase,
}

impl ReflectionEnumBackedCase {
    /// 创建新的 Backed Case 反射
    pub fn new(case: ReflectionEnumCase) -> Self {
        Self { inner: case }
    }
    
    /// 获取 Case 名称
    pub fn get_name(&self) -> &str {
        self.inner.get_name()
    }
    
    /// 获取所属枚举名
    pub fn get_enum_name(&self) -> &str {
        self.inner.get_enum_name()
    }
    
    /// 获取后备值
    ///
    /// 对应 PHP 的 ReflectionEnumBackedCase::getBackingValue()
    pub fn get_backing_value(&self) -> Option<&str> {
        self.inner.get_value()
    }
    
    /// 获取注解列表
    pub fn get_attributes(&self, name: Option<&str>) -> Vec<ReflectionAttribute> {
        self.inner.get_attributes(name)
    }
}

impl fmt::Display for ReflectionEnumBackedCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_enum() -> ReflectionEnum {
        let mut metadata = EnumMetadata::new("App\\Status");
        metadata.backing_type = Some("int".to_string());
        
        // 添加 Case
        let mut pending = EnumCaseMetadata::new("Pending");
        pending.value = Some("0".to_string());
        metadata.cases.push(pending);
        
        let mut active = EnumCaseMetadata::new("Active");
        active.value = Some("1".to_string());
        metadata.cases.push(active);
        
        // 添加接口
        metadata.interfaces.push("UnitEnum".to_string());
        
        ReflectionEnum::new(metadata)
    }
    
    #[test]
    fn test_enum_name() {
        let enum_ref = create_test_enum();
        
        assert_eq!(enum_ref.get_name(), "App\\Status");
        assert_eq!(enum_ref.get_short_name(), "Status");
        assert_eq!(enum_ref.get_namespace_name(), Some("App"));
        assert!(enum_ref.in_namespace());
    }
    
    #[test]
    fn test_enum_backed() {
        let enum_ref = create_test_enum();
        
        assert!(enum_ref.is_backed());
        assert_eq!(enum_ref.get_backing_type(), Some("int"));
    }
    
    #[test]
    fn test_enum_cases() {
        let enum_ref = create_test_enum();
        
        assert_eq!(enum_ref.get_case_count(), 2);
        assert!(enum_ref.has_case("Pending"));
        assert!(enum_ref.has_case("Active"));
        
        let pending = enum_ref.get_case("Pending");
        assert!(pending.is_some());
        assert_eq!(pending.unwrap().get_value(), Some("0"));
    }
    
    #[test]
    fn test_enum_interfaces() {
        let enum_ref = create_test_enum();
        
        assert!(enum_ref.implements_interface("UnitEnum"));
        assert_eq!(enum_ref.get_interface_names().len(), 1);
    }
    
    #[test]
    fn test_enum_case() {
        let enum_ref = create_test_enum();
        let case = enum_ref.get_case("Active").unwrap();
        
        assert_eq!(case.get_name(), "Active");
        assert_eq!(case.get_enum_name(), "App\\Status");
        assert_eq!(case.get_value(), Some("1"));
        assert_eq!(case.get_full_name(), "App\\Status::Active");
    }
    
    #[test]
    fn test_backed_case() {
        let enum_ref = create_test_enum();
        let case = enum_ref.get_case("Active").unwrap();
        let backed = ReflectionEnumBackedCase::new(case);
        
        assert_eq!(backed.get_name(), "Active");
        assert_eq!(backed.get_backing_value(), Some("1"));
    }
}
