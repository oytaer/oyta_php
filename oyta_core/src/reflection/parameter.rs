//! ReflectionParameter 参数反射模块
//!
//! 提供完整的 PHP ReflectionParameter 功能

use serde::{Deserialize, Serialize};
use std::fmt;

use super::metadata::ParameterMetadata;
use super::attribute::ReflectionAttribute;
use super::r#type::ReflectionNamedType;

/// ReflectionParameter 参数反射
///
/// 对应 PHP 的 ReflectionParameter 类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectionParameter {
    /// 参数元数据
    pub metadata: ParameterMetadata,
}

impl ReflectionParameter {
    /// 创建新的参数反射
    pub fn new(metadata: ParameterMetadata) -> Self {
        Self { metadata }
    }
    
    /// 从名称创建
    pub fn from_name(name: &str, position: usize) -> Self {
        Self {
            metadata: ParameterMetadata::new(name, position),
        }
    }
    
    /// 获取参数名
    ///
    /// 对应 PHP 的 ReflectionParameter::getName()
    pub fn get_name(&self) -> &str {
        &self.metadata.name
    }
    
    /// 获取参数名（不含 $ 前缀）
    pub fn get_name_without_dollar(&self) -> &str {
        self.metadata.name_without_dollar()
    }
    
    /// 获取参数位置
    ///
    /// 对应 PHP 的 ReflectionParameter::getPosition()
    pub fn get_position(&self) -> usize {
        self.metadata.position
    }
    
    /// 获取参数类型
    ///
    /// 对应 PHP 的 ReflectionParameter::getType()
    pub fn get_type(&self) -> Option<ReflectionNamedType> {
        self.metadata.parameter_type.as_ref().map(|t| {
            ReflectionNamedType::new(t.clone())
        })
    }
    
    /// 检查是否有类型声明
    ///
    /// 对应 PHP 的 ReflectionParameter::hasType()
    pub fn has_type(&self) -> bool {
        self.metadata.parameter_type.is_some()
    }
    
    /// 检查是否可选
    ///
    /// 对应 PHP 的 ReflectionParameter::isOptional()
    pub fn is_optional(&self) -> bool {
        self.metadata.is_optional
    }
    
    /// 检查是否必需
    ///
    /// 对应 PHP 的 ReflectionParameter::isRequired()
    pub fn is_required(&self) -> bool {
        !self.metadata.is_optional
    }
    
    /// 检查是否引用传递
    ///
    /// 对应 PHP 的 ReflectionParameter::isPassedByReference()
    pub fn is_passed_by_reference(&self) -> bool {
        self.metadata.is_passed_by_reference
    }
    
    /// 检查是否可变参数
    ///
    /// 对应 PHP 的 ReflectionParameter::isVariadic()
    pub fn is_variadic(&self) -> bool {
        self.metadata.is_variadic
    }
    
    /// 获取默认值
    ///
    /// 对应 PHP 的 ReflectionParameter::getDefaultValue()
    pub fn get_default_value(&self) -> Option<&str> {
        self.metadata.default_value.as_deref()
    }
    
    /// 检查是否有默认值
    ///
    /// 对应 PHP 的 ReflectionParameter::isDefaultValueAvailable()
    pub fn has_default_value(&self) -> bool {
        self.metadata.default_value.is_some()
    }
    
    /// 检查默认值是否为常量
    ///
    /// 对应 PHP 的 ReflectionParameter::isDefaultValueConstant()
    pub fn is_default_value_constant(&self) -> bool {
        // 检查默认值是否是常量引用（如 self::CONSTANT）
        self.metadata.default_value
            .as_ref()
            .map(|v| v.contains("::") || v.chars().next().map_or(false, |c| c.is_uppercase()))
            .unwrap_or(false)
    }
    
    /// 获取默认值常量名
    ///
    /// 对应 PHP 的 ReflectionParameter::getDefaultValueConstantName()
    pub fn get_default_value_constant_name(&self) -> Option<&str> {
        if self.is_default_value_constant() {
            self.metadata.default_value.as_deref()
        } else {
            None
        }
    }
    
    /// 获取注解列表
    ///
    /// 对应 PHP 的 ReflectionParameter::getAttributes()
    pub fn get_attributes(&self, name: Option<&str>) -> Vec<ReflectionAttribute> {
        self.metadata.attributes
            .iter()
            .filter(|a| name.is_none() || a.name == name.unwrap())
            .map(|a| ReflectionAttribute::new(a.clone()))
            .collect()
    }
    
    /// 检查是否允许 null
    ///
    /// 对应 PHP 的 ReflectionParameter::allowsNull()
    pub fn allows_null(&self) -> bool {
        self.metadata.parameter_type
            .as_ref()
            .map(|t| t.allows_null)
            .unwrap_or(true) // 没有类型声明时允许 null
    }
    
    /// 检查参数是否可以传递指定类型的值
    ///
    /// # 参数
    /// - `type_name`: 类型名
    pub fn can_pass_type(&self, type_name: &str) -> bool {
        if let Some(param_type) = &self.metadata.parameter_type {
            // 检查类型是否兼容
            param_type.name.eq_ignore_ascii_case(type_name)
                || param_type.name == "mixed"
                || (param_type.allows_null && type_name == "null")
        } else {
            // 没有类型声明，可以传递任何类型
            true
        }
    }
    
    /// 获取参数的字符串表示
    pub fn to_parameter_string(&self) -> String {
        let mut result = String::new();
        
        // 类型
        if let Some(t) = &self.metadata.parameter_type {
            if t.allows_null {
                result.push('?');
            }
            result.push_str(&t.name);
            result.push(' ');
        }
        
        // 引用
        if self.metadata.is_passed_by_reference {
            result.push('&');
        }
        
        // 可变参数
        if self.metadata.is_variadic {
            result.push_str("...");
        }
        
        // 参数名
        result.push_str(&self.metadata.name);
        
        // 默认值
        if self.metadata.is_optional {
            if let Some(default) = &self.metadata.default_value {
                result.push_str(&format!(" = {}", default));
            }
        }
        
        result
    }
}

impl fmt::Display for ReflectionParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parameter #{} [ ", self.metadata.position)?;
        
        if self.metadata.is_optional {
            write!(f, "<optional> ")?;
        } else {
            write!(f, "<required> ")?;
        }
        
        write!(f, "{} ]", self.to_parameter_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::metadata::TypeMetadata;
    
    fn create_test_parameter() -> ReflectionParameter {
        let mut metadata = ParameterMetadata::new("name", 0);
        metadata.is_optional = true;
        metadata.default_value = Some("'default'".to_string());
        metadata.parameter_type = Some(TypeMetadata::new("string"));
        
        ReflectionParameter::new(metadata)
    }
    
    #[test]
    fn test_parameter_name() {
        let param = create_test_parameter();
        
        assert_eq!(param.get_name(), "$name");
        assert_eq!(param.get_name_without_dollar(), "name");
        assert_eq!(param.get_position(), 0);
    }
    
    #[test]
    fn test_parameter_type() {
        let param = create_test_parameter();
        
        assert!(param.has_type());
        assert!(param.get_type().is_some());
    }
    
    #[test]
    fn test_parameter_optional() {
        let param = create_test_parameter();
        
        assert!(param.is_optional());
        assert!(!param.is_required());
    }
    
    #[test]
    fn test_parameter_default_value() {
        let param = create_test_parameter();
        
        assert!(param.has_default_value());
        assert_eq!(param.get_default_value(), Some("'default'"));
    }
    
    #[test]
    fn test_parameter_to_string() {
        let param = create_test_parameter();
        let str = param.to_parameter_string();
        
        assert!(str.contains("string"));
        assert!(str.contains("$name"));
        assert!(str.contains("'default'"));
    }
    
    #[test]
    fn test_parameter_variadic() {
        let mut param = ReflectionParameter::from_name("args", 0);
        param.metadata.is_variadic = true;
        
        assert!(param.is_variadic());
    }
    
    #[test]
    fn test_parameter_reference() {
        let mut param = ReflectionParameter::from_name("value", 0);
        param.metadata.is_passed_by_reference = true;
        
        assert!(param.is_passed_by_reference());
    }
}
