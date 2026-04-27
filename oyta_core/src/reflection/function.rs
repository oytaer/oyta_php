//! ReflectionFunction 函数反射模块
//!
//! 提供完整的 PHP ReflectionFunction 功能

use serde::{Deserialize, Serialize};
use std::fmt;

use super::metadata::{FunctionMetadata, ParameterMetadata};
use super::parameter::ReflectionParameter;
use super::attribute::ReflectionAttribute;
use super::r#type::ReflectionNamedType;

/// ReflectionFunction 函数反射
///
/// 对应 PHP 的 ReflectionFunction 类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectionFunction {
    /// 函数元数据
    pub metadata: FunctionMetadata,
}

impl ReflectionFunction {
    /// 创建新的函数反射
    pub fn new(metadata: FunctionMetadata) -> Self {
        Self { metadata }
    }
    
    /// 从函数名创建
    pub fn from_name(name: &str) -> Self {
        Self {
            metadata: FunctionMetadata::new(name),
        }
    }
    
    /// 获取函数名
    ///
    /// 对应 PHP 的 ReflectionFunction::getName()
    pub fn get_name(&self) -> &str {
        &self.metadata.name
    }
    
    /// 获取短函数名（不含命名空间）
    ///
    /// 对应 PHP 的 ReflectionFunction::getShortName()
    pub fn get_short_name(&self) -> &str {
        &self.metadata.short_name
    }
    
    /// 获取命名空间
    ///
    /// 对应 PHP 的 ReflectionFunction::getNamespaceName()
    pub fn get_namespace_name(&self) -> Option<&str> {
        self.metadata.namespace.as_deref()
    }
    
    /// 检查是否在命名空间中
    ///
    /// 对应 PHP 的 ReflectionFunction::inNamespace()
    pub fn in_namespace(&self) -> bool {
        self.metadata.namespace.is_some()
    }
    
    /// 获取返回类型
    ///
    /// 对应 PHP 的 ReflectionFunction::getReturnType()
    pub fn get_return_type(&self) -> Option<ReflectionNamedType> {
        self.metadata.return_type.as_ref().map(|t| {
            ReflectionNamedType::new(t.clone())
        })
    }
    
    /// 检查是否有返回类型
    ///
    /// 对应 PHP 的 ReflectionFunction::hasReturnType()
    pub fn has_return_type(&self) -> bool {
        self.metadata.return_type.is_some()
    }
    
    /// 检查是否引用返回
    ///
    /// 对应 PHP 的 ReflectionFunction::returnsReference()
    pub fn returns_reference(&self) -> bool {
        self.metadata.returns_reference
    }
    
    /// 获取参数数量
    ///
    /// 对应 PHP 的 ReflectionFunction::getNumberOfParameters()
    pub fn get_number_of_parameters(&self) -> usize {
        self.metadata.parameters.len()
    }
    
    /// 获取必需参数数量
    ///
    /// 对应 PHP 的 ReflectionFunction::getNumberOfRequiredParameters()
    pub fn get_number_of_required_parameters(&self) -> usize {
        self.metadata.parameters
            .iter()
            .filter(|p| !p.is_optional)
            .count()
    }
    
    /// 获取参数列表
    ///
    /// 对应 PHP 的 ReflectionFunction::getParameters()
    pub fn get_parameters(&self) -> Vec<ReflectionParameter> {
        self.metadata.parameters
            .iter()
            .map(|p| ReflectionParameter::new(p.clone()))
            .collect()
    }
    
    /// 获取指定位置的参数
    pub fn get_parameter(&self, position: usize) -> Option<ReflectionParameter> {
        self.metadata.parameters
            .get(position)
            .map(|p| ReflectionParameter::new(p.clone()))
    }
    
    /// 获取注解列表
    ///
    /// 对应 PHP 的 ReflectionFunction::getAttributes()
    pub fn get_attributes(&self, name: Option<&str>) -> Vec<ReflectionAttribute> {
        self.metadata.attributes
            .iter()
            .filter(|a| name.is_none() || a.name == name.unwrap())
            .map(|a| ReflectionAttribute::new(a.clone()))
            .collect()
    }
    
    /// 获取 DocBlock 注释
    ///
    /// 对应 PHP 的 ReflectionFunction::getDocComment()
    pub fn get_doc_comment(&self) -> Option<&str> {
        self.metadata.doc_comment.as_deref()
    }
    
    /// 获取源文件路径
    ///
    /// 对应 PHP 的 ReflectionFunction::getFileName()
    pub fn get_file_name(&self) -> Option<&str> {
        self.metadata.file_name.as_deref()
    }
    
    /// 获取起始行号
    ///
    /// 对应 PHP 的 ReflectionFunction::getStartLine()
    pub fn get_start_line(&self) -> Option<usize> {
        self.metadata.start_line
    }
    
    /// 获取结束行号
    ///
    /// 对应 PHP 的 ReflectionFunction::getEndLine()
    pub fn get_end_line(&self) -> Option<usize> {
        self.metadata.end_line
    }
    
    /// 检查是否为内部函数
    ///
    /// 对应 PHP 的 ReflectionFunction::isInternal()
    pub fn is_internal(&self) -> bool {
        self.metadata.file_name.is_none()
    }
    
    /// 检查是否为用户定义函数
    ///
    /// 对应 PHP 的 ReflectionFunction::isUserDefined()
    pub fn is_user_defined(&self) -> bool {
        self.metadata.file_name.is_some()
    }
    
    /// 检查是否为闭包
    ///
    /// 对应 PHP 的 ReflectionFunction::isClosure()
    pub fn is_closure(&self) -> bool {
        self.metadata.name.starts_with("{closure}")
            || self.metadata.name.starts_with("Closure")
    }
    
    /// 检查是否已废弃
    ///
    /// 对应 PHP 的 ReflectionFunction::isDeprecated()
    pub fn is_deprecated(&self) -> bool {
        // 检查 DocBlock 中是否有 @deprecated 标记
        self.metadata.doc_comment
            .as_ref()
            .map(|d| d.contains("@deprecated"))
            .unwrap_or(false)
    }
    
    /// 检查是否为生成器函数
    ///
    /// 对应 PHP 的 ReflectionFunction::isGenerator()
    pub fn is_generator(&self) -> bool {
        // 检查返回类型是否为 Generator
        self.metadata.return_type
            .as_ref()
            .map(|t| t.name == "Generator")
            .unwrap_or(false)
    }
    
    /// 检查是否可变参数
    ///
    /// 对应 PHP 的 ReflectionFunction::isVariadic()
    pub fn is_variadic(&self) -> bool {
        self.metadata.parameters.iter().any(|p| p.is_variadic)
    }
}

impl fmt::Display for ReflectionFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Function [ ")?;
        
        if self.metadata.returns_reference {
            write!(f, "&")?;
        }
        
        write!(f, "function {}(", self.metadata.name)?;
        
        let params: Vec<String> = self.metadata.parameters
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let mut s = String::new();
                if let Some(t) = &p.parameter_type {
                    s.push_str(&t.name);
                    s.push(' ');
                }
                if p.is_passed_by_reference {
                    s.push('&');
                }
                if p.is_variadic {
                    s.push_str("...");
                }
                s.push_str(&p.name);
                if p.is_optional {
                    if let Some(default) = &p.default_value {
                        s.push_str(&format!(" = {}", default));
                    }
                }
                s
            })
            .collect();
        
        write!(f, "{})", params.join(", "))?;
        
        if let Some(return_type) = &self.metadata.return_type {
            write!(f, " : {}", return_type.name)?;
        }
        
        write!(f, " ]")
    }
}

/// ReflectionFunctionAbstract 抽象函数反射基类
///
/// 对应 PHP 的 ReflectionFunctionAbstract 类
/// 提供函数和方法共有的功能
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectionFunctionAbstract {
    /// 函数名
    pub name: String,
    /// 返回类型
    pub return_type: Option<String>,
    /// 参数列表
    pub parameters: Vec<ParameterMetadata>,
    /// 是否引用返回
    pub returns_reference: bool,
    /// DocBlock 注释
    pub doc_comment: Option<String>,
    /// 源文件路径
    pub file_name: Option<String>,
    /// 起始行号
    pub start_line: Option<usize>,
    /// 结束行号
    pub end_line: Option<usize>,
}

impl ReflectionFunctionAbstract {
    /// 创建新的抽象函数反射
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            return_type: None,
            parameters: Vec::new(),
            returns_reference: false,
            doc_comment: None,
            file_name: None,
            start_line: None,
            end_line: None,
        }
    }
    
    /// 获取名称
    pub fn get_name(&self) -> &str {
        &self.name
    }
    
    /// 获取返回类型
    pub fn get_return_type(&self) -> Option<&str> {
        self.return_type.as_deref()
    }
    
    /// 检查是否有返回类型
    pub fn has_return_type(&self) -> bool {
        self.return_type.is_some()
    }
    
    /// 获取参数数量
    pub fn get_number_of_parameters(&self) -> usize {
        self.parameters.len()
    }
    
    /// 获取必需参数数量
    pub fn get_number_of_required_parameters(&self) -> usize {
        self.parameters.iter().filter(|p| !p.is_optional).count()
    }
    
    /// 检查是否引用返回
    pub fn returns_reference(&self) -> bool {
        self.returns_reference
    }
    
    /// 获取 DocBlock 注释
    pub fn get_doc_comment(&self) -> Option<&str> {
        self.doc_comment.as_deref()
    }
    
    /// 获取源文件路径
    pub fn get_file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }
    
    /// 获取起始行号
    pub fn get_start_line(&self) -> Option<usize> {
        self.start_line
    }
    
    /// 获取结束行号
    pub fn get_end_line(&self) -> Option<usize> {
        self.end_line
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::metadata::TypeMetadata;
    
    fn create_test_function() -> ReflectionFunction {
        let mut metadata = FunctionMetadata::new("App\\Utils\\formatDate");
        
        // 设置返回类型
        metadata.return_type = Some(TypeMetadata::new("string"));
        
        // 添加参数
        let mut param = ParameterMetadata::new("timestamp", 0);
        param.is_optional = false;
        metadata.parameters.push(param);
        
        let mut param2 = ParameterMetadata::new("format", 1);
        param2.is_optional = true;
        param2.default_value = Some("'Y-m-d'".to_string());
        metadata.parameters.push(param2);
        
        ReflectionFunction::new(metadata)
    }
    
    #[test]
    fn test_function_name() {
        let func = create_test_function();
        
        assert_eq!(func.get_name(), "App\\Utils\\formatDate");
        assert_eq!(func.get_short_name(), "formatDate");
        assert_eq!(func.get_namespace_name(), Some("App\\Utils"));
        assert!(func.in_namespace());
    }
    
    #[test]
    fn test_function_return_type() {
        let func = create_test_function();
        
        assert!(func.has_return_type());
        assert!(func.get_return_type().is_some());
    }
    
    #[test]
    fn test_function_parameters() {
        let func = create_test_function();
        
        assert_eq!(func.get_number_of_parameters(), 2);
        assert_eq!(func.get_number_of_required_parameters(), 1);
        
        let params = func.get_parameters();
        assert_eq!(params.len(), 2);
    }
    
    #[test]
    fn test_function_closure() {
        let closure = ReflectionFunction::from_name("{closure}");
        assert!(closure.is_closure());
        
        let func = ReflectionFunction::from_name("normalFunction");
        assert!(!func.is_closure());
    }
    
    #[test]
    fn test_function_display() {
        let func = create_test_function();
        let display = format!("{}", func);
        
        assert!(display.contains("formatDate"));
        assert!(display.contains("string"));
    }
}
