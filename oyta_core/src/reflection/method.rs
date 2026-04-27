//! ReflectionMethod 方法反射模块
//!
//! 提供完整的 PHP ReflectionMethod 功能

use serde::{Deserialize, Serialize};
use std::fmt;

use super::metadata::{MethodMetadata, ParameterMetadata, Visibility};
use super::parameter::ReflectionParameter;
use super::attribute::ReflectionAttribute;
use super::r#type::ReflectionNamedType;

/// ReflectionMethod 方法反射
///
/// 对应 PHP 的 ReflectionMethod 类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectionMethod {
    /// 方法元数据
    pub metadata: MethodMetadata,
}

impl ReflectionMethod {
    /// 创建新的方法反射
    pub fn new(metadata: MethodMetadata) -> Self {
        Self { metadata }
    }
    
    /// 从类名和方法名创建
    pub fn from_name(class_name: &str, method_name: &str) -> Self {
        Self {
            metadata: MethodMetadata::new(method_name, class_name),
        }
    }
    
    /// 获取方法名
    ///
    /// 对应 PHP 的 ReflectionMethod::getName()
    pub fn get_name(&self) -> &str {
        &self.metadata.name
    }
    
    /// 获取所属类名
    ///
    /// 对应 PHP 的 ReflectionMethod::getDeclaringClass()
    pub fn get_declaring_class_name(&self) -> &str {
        &self.metadata.class_name
    }
    
    /// 检查是否为公开方法
    ///
    /// 对应 PHP 的 ReflectionMethod::isPublic()
    pub fn is_public(&self) -> bool {
        self.metadata.visibility == Visibility::Public
    }
    
    /// 检查是否为受保护方法
    ///
    /// 对应 PHP 的 ReflectionMethod::isProtected()
    pub fn is_protected(&self) -> bool {
        self.metadata.visibility == Visibility::Protected
    }
    
    /// 检查是否为私有方法
    ///
    /// 对应 PHP 的 ReflectionMethod::isPrivate()
    pub fn is_private(&self) -> bool {
        self.metadata.visibility == Visibility::Private
    }
    
    /// 检查是否为静态方法
    ///
    /// 对应 PHP 的 ReflectionMethod::isStatic()
    pub fn is_static(&self) -> bool {
        self.metadata.is_static
    }
    
    /// 检查是否为抽象方法
    ///
    /// 对应 PHP 的 ReflectionMethod::isAbstract()
    pub fn is_abstract(&self) -> bool {
        self.metadata.is_abstract
    }
    
    /// 检查是否为最终方法
    ///
    /// 对应 PHP 的 ReflectionMethod::isFinal()
    pub fn is_final(&self) -> bool {
        self.metadata.is_final
    }
    
    /// 检查是否为构造函数
    ///
    /// 对应 PHP 的 ReflectionMethod::isConstructor()
    pub fn is_constructor(&self) -> bool {
        self.metadata.is_constructor()
    }
    
    /// 检查是否为析构函数
    ///
    /// 对应 PHP 的 ReflectionMethod::isDestructor()
    pub fn is_destructor(&self) -> bool {
        self.metadata.is_destructor()
    }
    
    /// 获取可见性
    pub fn get_visibility(&self) -> Visibility {
        self.metadata.visibility
    }
    
    /// 获取返回类型
    ///
    /// 对应 PHP 的 ReflectionMethod::getReturnType()
    pub fn get_return_type(&self) -> Option<ReflectionNamedType> {
        self.metadata.return_type.as_ref().map(|t| {
            ReflectionNamedType::new(t.clone())
        })
    }
    
    /// 检查是否有返回类型
    ///
    /// 对应 PHP 的 ReflectionMethod::hasReturnType()
    pub fn has_return_type(&self) -> bool {
        self.metadata.return_type.is_some()
    }
    
    /// 获取参数数量
    ///
    /// 对应 PHP 的 ReflectionMethod::getNumberOfParameters()
    pub fn get_number_of_parameters(&self) -> usize {
        self.metadata.parameters.len()
    }
    
    /// 获取必需参数数量
    ///
    /// 对应 PHP 的 ReflectionMethod::getNumberOfRequiredParameters()
    pub fn get_number_of_required_parameters(&self) -> usize {
        self.metadata.parameters
            .iter()
            .filter(|p| !p.is_optional)
            .count()
    }
    
    /// 获取参数
    ///
    /// 对应 PHP 的 ReflectionMethod::getParameters()
    pub fn get_parameters(&self) -> Vec<ReflectionParameter> {
        self.metadata.parameters
            .iter()
            .map(|p| ReflectionParameter::new(p.clone()))
            .collect()
    }
    
    /// 获取指定位置的参数
    ///
    /// # 参数
    /// - `position`: 参数位置（从 0 开始）
    pub fn get_parameter(&self, position: usize) -> Option<ReflectionParameter> {
        self.metadata.parameters
            .get(position)
            .map(|p| ReflectionParameter::new(p.clone()))
    }
    
    /// 按名称获取参数
    pub fn get_parameter_by_name(&self, name: &str) -> Option<ReflectionParameter> {
        let name = name.trim_start_matches('$');
        self.metadata.parameters
            .iter()
            .find(|p| p.name_without_dollar() == name)
            .map(|p| ReflectionParameter::new(p.clone()))
    }
    
    /// 获取注解列表
    ///
    /// 对应 PHP 的 ReflectionMethod::getAttributes()
    pub fn get_attributes(&self, name: Option<&str>) -> Vec<ReflectionAttribute> {
        self.metadata.attributes
            .iter()
            .filter(|a| name.is_none() || a.name == name.unwrap())
            .map(|a| ReflectionAttribute::new(a.clone()))
            .collect()
    }
    
    /// 获取 DocBlock 注释
    ///
    /// 对应 PHP 的 ReflectionMethod::getDocComment()
    pub fn get_doc_comment(&self) -> Option<&str> {
        self.metadata.doc_comment.as_deref()
    }
    
    /// 获取起始行号
    ///
    /// 对应 PHP 的 ReflectionMethod::getStartLine()
    pub fn get_start_line(&self) -> Option<usize> {
        self.metadata.start_line
    }
    
    /// 获取结束行号
    ///
    /// 对应 PHP 的 ReflectionMethod::getEndLine()
    pub fn get_end_line(&self) -> Option<usize> {
        self.metadata.end_line
    }
    
    /// 获取修改器
    ///
    /// 对应 PHP 的 ReflectionMethod::getModifiers()
    pub fn get_modifiers(&self) -> MethodModifiers {
        let mut modifiers = MethodModifiers::empty();
        
        match self.metadata.visibility {
            Visibility::Public => modifiers |= MethodModifiers::PUBLIC,
            Visibility::Protected => modifiers |= MethodModifiers::PROTECTED,
            Visibility::Private => modifiers |= MethodModifiers::PRIVATE,
        }
        
        if self.metadata.is_static {
            modifiers |= MethodModifiers::STATIC;
        }
        if self.metadata.is_abstract {
            modifiers |= MethodModifiers::ABSTRACT;
        }
        if self.metadata.is_final {
            modifiers |= MethodModifiers::FINAL;
        }
        
        modifiers
    }
    
    /// 检查是否为内部方法
    ///
    /// 对应 PHP 的 ReflectionMethod::isInternal()
    pub fn is_internal(&self) -> bool {
        self.metadata.start_line.is_none()
    }
    
    /// 检查是否为用户定义方法
    ///
    /// 对应 PHP 的 ReflectionMethod::isUserDefined()
    pub fn is_user_defined(&self) -> bool {
        self.metadata.start_line.is_some()
    }
    
    /// 检查方法是否可以设置访问控制
    ///
    /// 对应 PHP 的 ReflectionMethod::setAccessible()
    pub fn set_accessible(&mut self, _accessible: bool) {
        // 在 Rust 实现中，这个方法主要用于标记
        // 实际访问控制在执行时处理
    }
    
    /// 获取原型方法
    ///
    /// 对应 PHP 的 ReflectionMethod::getPrototype()
    pub fn get_prototype(&self) -> Option<ReflectionMethod> {
        // 在实际实现中，需要遍历继承链查找原型
        None
    }
    
    /// 检查是否为魔术方法
    pub fn is_magic_method(&self) -> bool {
        matches!(
            self.metadata.name.to_lowercase().as_str(),
            "__construct" | "__destruct" | "__call" | "__callstatic" |
            "__get" | "__set" | "__isset" | "__unset" |
            "__sleep" | "__wakeup" | "__tostring" | "__invoke" |
            "__set_state" | "__clone" | "__debuginfo" |
            "__serialize" | "__unserialize"
        )
    }
    
    /// 获取魔术方法类型
    pub fn get_magic_method_type(&self) -> Option<MagicMethodType> {
        match self.metadata.name.to_lowercase().as_str() {
            "__construct" => Some(MagicMethodType::Construct),
            "__destruct" => Some(MagicMethodType::Destruct),
            "__call" => Some(MagicMethodType::Call),
            "__callstatic" => Some(MagicMethodType::CallStatic),
            "__get" => Some(MagicMethodType::Get),
            "__set" => Some(MagicMethodType::Set),
            "__isset" => Some(MagicMethodType::Isset),
            "__unset" => Some(MagicMethodType::Unset),
            "__sleep" => Some(MagicMethodType::Sleep),
            "__wakeup" => Some(MagicMethodType::Wakeup),
            "__tostring" => Some(MagicMethodType::ToString),
            "__invoke" => Some(MagicMethodType::Invoke),
            "__set_state" => Some(MagicMethodType::SetState),
            "__clone" => Some(MagicMethodType::Clone),
            "__debuginfo" => Some(MagicMethodType::DebugInfo),
            "__serialize" => Some(MagicMethodType::Serialize),
            "__unserialize" => Some(MagicMethodType::Unserialize),
            _ => None,
        }
    }
}

impl fmt::Display for ReflectionMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Method [ ")?;
        
        if self.metadata.is_static {
            write!(f, "static ")?;
        }
        if self.metadata.is_abstract {
            write!(f, "abstract ")?;
        }
        if self.metadata.is_final {
            write!(f, "final ")?;
        }
        
        write!(f, "{} ", self.metadata.visibility)?;
        write!(f, "{} ]", self.metadata.name)?;
        
        if let Some(return_type) = &self.metadata.return_type {
            write!(f, " : {}", return_type.name)?;
        }
        
        Ok(())
    }
}

/// 方法修改器标志
bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MethodModifiers: u32 {
        /// 公开方法
        const PUBLIC = 1;
        /// 受保护方法
        const PROTECTED = 2;
        /// 私有方法
        const PRIVATE = 4;
        /// 静态方法
        const STATIC = 8;
        /// 抽象方法
        const ABSTRACT = 16;
        /// 最终方法
        const FINAL = 32;
    }
}

impl Default for MethodModifiers {
    fn default() -> Self {
        Self::PUBLIC
    }
}

/// 魔术方法类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MagicMethodType {
    /// 构造函数
    Construct,
    /// 析构函数
    Destruct,
    /// 动态方法调用
    Call,
    /// 动态静态方法调用
    CallStatic,
    /// 属性获取
    Get,
    /// 属性设置
    Set,
    /// 属性检查
    Isset,
    /// 属性删除
    Unset,
    /// 序列化前
    Sleep,
    /// 反序列化后
    Wakeup,
    /// 字符串转换
    ToString,
    /// 可调用
    Invoke,
    /// var_export 导出
    SetState,
    /// 克隆
    Clone,
    /// 调试信息
    DebugInfo,
    /// 序列化
    Serialize,
    /// 反序列化
    Unserialize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_method() -> ReflectionMethod {
        let mut metadata = MethodMetadata::new("getUser", "UserService");
        metadata.visibility = Visibility::Public;
        metadata.is_final = true;
        
        // 添加参数
        let mut param = ParameterMetadata::new("id", 0);
        param.is_optional = false;
        metadata.parameters.push(param);
        
        let mut param2 = ParameterMetadata::new("format", 1);
        param2.is_optional = true;
        param2.default_value = Some("'json'".to_string());
        metadata.parameters.push(param2);
        
        ReflectionMethod::new(metadata)
    }
    
    #[test]
    fn test_method_name() {
        let method = create_test_method();
        
        assert_eq!(method.get_name(), "getUser");
        assert_eq!(method.get_declaring_class_name(), "UserService");
    }
    
    #[test]
    fn test_method_visibility() {
        let method = create_test_method();
        
        assert!(method.is_public());
        assert!(!method.is_protected());
        assert!(!method.is_private());
    }
    
    #[test]
    fn test_method_modifiers() {
        let method = create_test_method();
        
        assert!(method.is_final());
        assert!(!method.is_static());
        assert!(!method.is_abstract());
    }
    
    #[test]
    fn test_method_parameters() {
        let method = create_test_method();
        
        assert_eq!(method.get_number_of_parameters(), 2);
        assert_eq!(method.get_number_of_required_parameters(), 1);
        
        let params = method.get_parameters();
        assert_eq!(params.len(), 2);
    }
    
    #[test]
    fn test_method_get_parameter() {
        let method = create_test_method();
        
        let param = method.get_parameter(0);
        assert!(param.is_some());
        assert_eq!(param.unwrap().get_name(), "$id");
        
        let param = method.get_parameter_by_name("format");
        assert!(param.is_some());
    }
    
    #[test]
    fn test_magic_method() {
        let method = ReflectionMethod::from_name("TestClass", "__construct");
        
        assert!(method.is_constructor());
        assert!(method.is_magic_method());
        assert_eq!(method.get_magic_method_type(), Some(MagicMethodType::Construct));
    }
}
