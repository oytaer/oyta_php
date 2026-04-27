//! ReflectionClass 类反射模块
//!
//! 提供完整的 PHP ReflectionClass 功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use super::metadata::{
    ClassMetadata, MethodMetadata, PropertyMetadata, ConstantMetadata, Visibility,
};
use super::method::ReflectionMethod;
use super::property::ReflectionProperty;
use super::attribute::ReflectionAttribute;

/// ReflectionClass 类反射
///
/// 对应 PHP 的 ReflectionClass 类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectionClass {
    /// 类元数据
    pub metadata: ClassMetadata,
}

impl ReflectionClass {
    /// 创建新的类反射
    ///
    /// # 参数
    /// - `metadata`: 类元数据
    pub fn new(metadata: ClassMetadata) -> Self {
        Self { metadata }
    }
    
    /// 从类名创建反射
    ///
    /// # 参数
    /// - `class_name`: 类名
    ///
    /// # 返回
    /// ReflectionClass 实例
    pub fn from_name(class_name: &str) -> Self {
        Self {
            metadata: ClassMetadata::new(class_name),
        }
    }
    
    /// 获取类名
    ///
    /// 对应 PHP 的 ReflectionClass::getName()
    pub fn get_name(&self) -> &str {
        &self.metadata.name
    }
    
    /// 获取短类名（不含命名空间）
    ///
    /// 对应 PHP 的 ReflectionClass::getShortName()
    pub fn get_short_name(&self) -> &str {
        &self.metadata.short_name
    }
    
    /// 获取命名空间
    ///
    /// 对应 PHP 的 ReflectionClass::getNamespaceName()
    pub fn get_namespace_name(&self) -> Option<&str> {
        self.metadata.namespace.as_deref()
    }
    
    /// 检查是否在命名空间中
    ///
    /// 对应 PHP 的 ReflectionClass::inNamespace()
    pub fn in_namespace(&self) -> bool {
        self.metadata.namespace.is_some()
    }
    
    /// 检查是否为抽象类
    ///
    /// 对应 PHP 的 ReflectionClass::isAbstract()
    pub fn is_abstract(&self) -> bool {
        self.metadata.is_abstract
    }
    
    /// 检查是否为最终类
    ///
    /// 对应 PHP 的 ReflectionClass::isFinal()
    pub fn is_final(&self) -> bool {
        self.metadata.is_final
    }
    
    /// 检查是否为只读类
    ///
    /// 对应 PHP 的 ReflectionClass::isReadOnly()
    pub fn is_readonly(&self) -> bool {
        self.metadata.is_readonly
    }
    
    /// 检查是否为接口
    ///
    /// 对应 PHP 的 ReflectionClass::isInterface()
    pub fn is_interface(&self) -> bool {
        false // 类反射永远返回 false
    }
    
    /// 检查是否为 Trait
    ///
    /// 对应 PHP 的 ReflectionClass::isTrait()
    pub fn is_trait(&self) -> bool {
        false // 类反射永远返回 false
    }
    
    /// 检查是否为枚举
    ///
    /// 对应 PHP 的 ReflectionClass::isEnum()
    pub fn is_enum(&self) -> bool {
        false // 类反射永远返回 false
    }
    
    /// 检查是否可实例化
    ///
    /// 对应 PHP 的 ReflectionClass::isInstantiable()
    pub fn is_instantiable(&self) -> bool {
        // 抽象类和接口不可实例化
        !self.metadata.is_abstract
    }
    
    /// 检查是否可克隆
    ///
    /// 对应 PHP 的 ReflectionClass::isCloneable()
    pub fn is_cloneable(&self) -> bool {
        // 检查是否有 __clone 方法且为 public
        if let Some(method) = self.get_method("__clone") {
            method.is_public()
        } else {
            true // 没有 __clone 方法时可克隆
        }
    }
    
    /// 获取父类名
    ///
    /// 对应 PHP 的 ReflectionClass::getParentClass()
    pub fn get_parent_class(&self) -> Option<&str> {
        self.metadata.parent_class.as_deref()
    }
    
    /// 检查是否为子类
    ///
    /// 对应 PHP 的 ReflectionClass::isSubclassOf()
    ///
    /// # 参数
    /// - `class_name`: 父类名
    pub fn is_subclass_of(&self, class_name: &str) -> bool {
        // 检查直接父类
        if self.metadata.extends(class_name) {
            return true;
        }
        
        // 在实际实现中，需要递归检查继承链
        false
    }
    
    /// 检查是否实现了接口
    ///
    /// 对应 PHP 的 ReflectionClass::implementsInterface()
    ///
    /// # 参数
    /// - `interface_name`: 接口名
    pub fn implements_interface(&self, interface_name: &str) -> bool {
        self.metadata.implements(interface_name)
    }
    
    /// 获取实现的接口列表
    ///
    /// 对应 PHP 的 ReflectionClass::getInterfaceNames()
    pub fn get_interface_names(&self) -> &[String] {
        &self.metadata.interfaces
    }
    
    /// 获取使用的 Trait 列表
    ///
    /// 对应 PHP 的 ReflectionClass::getTraitNames()
    pub fn get_trait_names(&self) -> &[String] {
        &self.metadata.traits
    }
    
    /// 获取方法
    ///
    /// 对应 PHP 的 ReflectionClass::getMethod()
    ///
    /// # 参数
    /// - `name`: 方法名
    ///
    /// # 返回
    /// ReflectionMethod 或 None
    pub fn get_method(&self, name: &str) -> Option<ReflectionMethod> {
        self.metadata.find_method(name).map(|m| {
            ReflectionMethod::new(m.clone())
        })
    }
    
    /// 获取所有方法
    ///
    /// 对应 PHP 的 ReflectionClass::getMethods()
    ///
    /// # 参数
    /// - `filter`: 可选的过滤器
    ///
    /// # 返回
    /// 方法列表
    pub fn get_methods(&self, filter: Option<MethodFilter>) -> Vec<ReflectionMethod> {
        self.metadata.methods
            .iter()
            .filter(|m| {
                if let Some(f) = filter {
                    self.method_matches_filter(m, f)
                } else {
                    true
                }
            })
            .map(|m| ReflectionMethod::new(m.clone()))
            .collect()
    }
    
    /// 检查方法是否匹配过滤器
    fn method_matches_filter(&self, method: &MethodMetadata, filter: MethodFilter) -> bool {
        match filter {
            MethodFilter::Public => method.visibility == Visibility::Public,
            MethodFilter::Protected => method.visibility == Visibility::Protected,
            MethodFilter::Private => method.visibility == Visibility::Private,
            MethodFilter::Static => method.is_static,
            MethodFilter::Abstract => method.is_abstract,
            MethodFilter::Final => method.is_final,
        }
    }
    
    /// 检查是否有方法
    ///
    /// 对应 PHP 的 ReflectionClass::hasMethod()
    ///
    /// # 参数
    /// - `name`: 方法名
    pub fn has_method(&self, name: &str) -> bool {
        self.metadata.find_method(name).is_some()
    }
    
    /// 获取属性
    ///
    /// 对应 PHP 的 ReflectionClass::getProperty()
    ///
    /// # 参数
    /// - `name`: 属性名
    ///
    /// # 返回
    /// ReflectionProperty 或 None
    pub fn get_property(&self, name: &str) -> Option<ReflectionProperty> {
        self.metadata.find_property(name).map(|p| {
            ReflectionProperty::new(p.clone())
        })
    }
    
    /// 获取所有属性
    ///
    /// 对应 PHP 的 ReflectionClass::getProperties()
    ///
    /// # 参数
    /// - `filter`: 可选的过滤器
    ///
    /// # 返回
    /// 属性列表
    pub fn get_properties(&self, filter: Option<PropertyFilter>) -> Vec<ReflectionProperty> {
        self.metadata.properties
            .iter()
            .filter(|p| {
                if let Some(f) = filter {
                    self.property_matches_filter(p, f)
                } else {
                    true
                }
            })
            .map(|p| ReflectionProperty::new(p.clone()))
            .collect()
    }
    
    /// 检查属性是否匹配过滤器
    fn property_matches_filter(&self, property: &PropertyMetadata, filter: PropertyFilter) -> bool {
        match filter {
            PropertyFilter::Public => property.visibility == Visibility::Public,
            PropertyFilter::Protected => property.visibility == Visibility::Protected,
            PropertyFilter::Private => property.visibility == Visibility::Private,
            PropertyFilter::Static => property.is_static,
            PropertyFilter::Readonly => property.is_readonly,
        }
    }
    
    /// 检查是否有属性
    ///
    /// 对应 PHP 的 ReflectionClass::hasProperty()
    ///
    /// # 参数
    /// - `name`: 属性名
    pub fn has_property(&self, name: &str) -> bool {
        self.metadata.find_property(name).is_some()
    }
    
    /// 获取常量
    ///
    /// 对应 PHP 的 ReflectionClass::getConstant()
    ///
    /// # 参数
    /// - `name`: 常量名
    pub fn get_constant(&self, name: &str) -> Option<&ConstantMetadata> {
        self.metadata.find_constant(name)
    }
    
    /// 获取所有常量
    ///
    /// 对应 PHP 的 ReflectionClass::getConstants()
    pub fn get_constants(&self) -> &[ConstantMetadata] {
        &self.metadata.constants
    }
    
    /// 检查是否有常量
    ///
    /// 对应 PHP 的 ReflectionClass::hasConstant()
    ///
    /// # 参数
    /// - `name`: 常量名
    pub fn has_constant(&self, name: &str) -> bool {
        self.metadata.find_constant(name).is_some()
    }
    
    /// 获取注解列表
    ///
    /// 对应 PHP 的 ReflectionClass::getAttributes()
    ///
    /// # 参数
    /// - `name`: 可选的注解类名过滤
    pub fn get_attributes(&self, name: Option<&str>) -> Vec<ReflectionAttribute> {
        self.metadata.attributes
            .iter()
            .filter(|a| name.is_none() || a.name == name.unwrap())
            .map(|a| ReflectionAttribute::new(a.clone()))
            .collect()
    }
    
    /// 获取 DocBlock 注释
    ///
    /// 对应 PHP 的 ReflectionClass::getDocComment()
    pub fn get_doc_comment(&self) -> Option<&str> {
        self.metadata.doc_comment.as_deref()
    }
    
    /// 获取源文件路径
    ///
    /// 对应 PHP 的 ReflectionClass::getFileName()
    pub fn get_file_name(&self) -> Option<&str> {
        self.metadata.file_name.as_deref()
    }
    
    /// 获取起始行号
    ///
    /// 对应 PHP 的 ReflectionClass::getStartLine()
    pub fn get_start_line(&self) -> Option<usize> {
        self.metadata.start_line
    }
    
    /// 获取结束行号
    ///
    /// 对应 PHP 的 ReflectionClass::getEndLine()
    pub fn get_end_line(&self) -> Option<usize> {
        self.metadata.end_line
    }
    
    /// 获取构造函数
    ///
    /// 对应 PHP 的 ReflectionClass::getConstructor()
    pub fn get_constructor(&self) -> Option<ReflectionMethod> {
        self.get_method("__construct")
    }
    
    /// 检查是否有构造函数
    pub fn has_constructor(&self) -> bool {
        self.has_method("__construct")
    }
    
    /// 获取默认属性值
    ///
    /// 对应 PHP 的 ReflectionClass::getDefaultProperties()
    pub fn get_default_properties(&self) -> HashMap<String, Option<String>> {
        self.metadata.properties
            .iter()
            .map(|p| (p.name.clone(), p.default_value.clone()))
            .collect()
    }
    
    /// 获取静态属性值
    ///
    /// 对应 PHP 的 ReflectionClass::getStaticPropertyValue()
    ///
    /// # 参数
    /// - `name`: 属性名
    pub fn get_static_property_value(&self, name: &str) -> Option<&str> {
        self.metadata.properties
            .iter()
            .find(|p| p.is_static && p.name_without_dollar() == name.trim_start_matches('$'))
            .and_then(|p| p.default_value.as_deref())
    }
    
    /// 获取修改器
    ///
    /// 对应 PHP 的 ReflectionClass::getModifiers()
    pub fn get_modifiers(&self) -> ClassModifiers {
        let mut modifiers = ClassModifiers::empty();
        
        if self.metadata.is_abstract {
            modifiers |= ClassModifiers::ABSTRACT;
        }
        if self.metadata.is_final {
            modifiers |= ClassModifiers::FINAL;
        }
        if self.metadata.is_readonly {
            modifiers |= ClassModifiers::READONLY;
        }
        
        modifiers
    }
    
    /// 检查是否为匿名类
    ///
    /// 对应 PHP 的 ReflectionClass::isAnonymous()
    pub fn is_anonymous(&self) -> bool {
        // 匿名类的名称以 "class@anonymous" 开头
        self.metadata.name.starts_with("class@anonymous")
    }
    
    /// 检查是否为内部类
    ///
    /// 对应 PHP 的 ReflectionClass::isInternal()
    pub fn is_internal(&self) -> bool {
        // 内置类没有文件名
        self.metadata.file_name.is_none()
    }
    
    /// 检查是否为用户定义类
    ///
    /// 对应 PHP 的 ReflectionClass::isUserDefined()
    pub fn is_user_defined(&self) -> bool {
        self.metadata.file_name.is_some()
    }
}

impl fmt::Display for ReflectionClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Class [ {} ", self.metadata.name)?;
        
        if self.metadata.is_abstract {
            write!(f, "abstract ")?;
        }
        if self.metadata.is_final {
            write!(f, "final ")?;
        }
        if self.metadata.is_readonly {
            write!(f, "readonly ")?;
        }
        
        write!(f, "]")?;
        
        if let Some(parent) = &self.metadata.parent_class {
            write!(f, " extends {}", parent)?;
        }
        
        if !self.metadata.interfaces.is_empty() {
            write!(f, " implements {}", self.metadata.interfaces.join(", "))?;
        }
        
        Ok(())
    }
}

/// 方法过滤器
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MethodFilter {
    /// 公开方法
    Public,
    /// 受保护方法
    Protected,
    /// 私有方法
    Private,
    /// 静态方法
    Static,
    /// 抽象方法
    Abstract,
    /// 最终方法
    Final,
}

/// 属性过滤器
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyFilter {
    /// 公开属性
    Public,
    /// 受保护属性
    Protected,
    /// 私有属性
    Private,
    /// 静态属性
    Static,
    /// 只读属性
    Readonly,
}

/// 类修改器标志
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ClassModifiers: u32 {
        /// 抽象类
        const ABSTRACT = 1;
        /// 最终类
        const FINAL = 2;
        /// 只读类
        const READONLY = 4;
    }
}

impl Default for ClassModifiers {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_class() -> ReflectionClass {
        let mut metadata = ClassMetadata::new("App\\Service\\UserService");
        metadata.is_final = true;
        metadata.parent_class = Some("App\\Service\\BaseService".to_string());
        metadata.interfaces.push("App\\Contract\\UserInterface".to_string());
        
        // 添加方法
        let mut method = MethodMetadata::new("getName", &metadata.name);
        method.visibility = Visibility::Public;
        metadata.methods.push(method);
        
        // 添加属性
        let mut property = PropertyMetadata::new("name", &metadata.name);
        property.visibility = Visibility::Private;
        metadata.properties.push(property);
        
        ReflectionClass::new(metadata)
    }
    
    #[test]
    fn test_reflection_class_name() {
        let class = create_test_class();
        
        assert_eq!(class.get_name(), "App\\Service\\UserService");
        assert_eq!(class.get_short_name(), "UserService");
        assert_eq!(class.get_namespace_name(), Some("App\\Service"));
        assert!(class.in_namespace());
    }
    
    #[test]
    fn test_reflection_class_modifiers() {
        let class = create_test_class();
        
        assert!(class.is_final());
        assert!(!class.is_abstract());
        assert!(!class.is_readonly());
    }
    
    #[test]
    fn test_reflection_class_inheritance() {
        let class = create_test_class();
        
        assert_eq!(class.get_parent_class(), Some("App\\Service\\BaseService"));
        assert!(class.implements_interface("App\\Contract\\UserInterface"));
    }
    
    #[test]
    fn test_reflection_class_methods() {
        let class = create_test_class();
        
        assert!(class.has_method("getName"));
        assert!(class.get_method("getName").is_some());
        
        let methods = class.get_methods(None);
        assert_eq!(methods.len(), 1);
    }
    
    #[test]
    fn test_reflection_class_properties() {
        let class = create_test_class();
        
        assert!(class.has_property("name"));
        assert!(class.get_property("name").is_some());
        
        let properties = class.get_properties(None);
        assert_eq!(properties.len(), 1);
    }
    
    #[test]
    fn test_reflection_class_instantiable() {
        let class = create_test_class();
        assert!(class.is_instantiable());
        
        // 抽象类不可实例化
        let mut abstract_metadata = ClassMetadata::new("AbstractClass");
        abstract_metadata.is_abstract = true;
        let abstract_class = ReflectionClass::new(abstract_metadata);
        assert!(!abstract_class.is_instantiable());
    }
}
