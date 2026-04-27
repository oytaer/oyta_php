//! ReflectionAttribute 注解反射模块
//!
//! 提供完整的 PHP ReflectionAttribute 功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use super::metadata::{AttributeMetadata, AttributeTarget};

/// ReflectionAttribute 注解反射
///
/// 对应 PHP 的 ReflectionAttribute 类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectionAttribute {
    /// 注解元数据
    pub metadata: AttributeMetadata,
}

impl ReflectionAttribute {
    /// 创建新的注解反射
    pub fn new(metadata: AttributeMetadata) -> Self {
        Self { metadata }
    }
    
    /// 从名称创建
    pub fn from_name(name: &str) -> Self {
        Self {
            metadata: AttributeMetadata::new(name),
        }
    }
    
    /// 获取注解类名
    ///
    /// 对应 PHP 的 ReflectionAttribute::getName()
    pub fn get_name(&self) -> &str {
        &self.metadata.name
    }
    
    /// 获取目标类型
    ///
    /// 对应 PHP 的 ReflectionAttribute::getTarget()
    pub fn get_target(&self) -> AttributeTarget {
        self.metadata.target
    }
    
    /// 获取位置参数
    ///
    /// 对应 PHP 的 ReflectionAttribute::getArguments()
    pub fn get_arguments(&self) -> &[String] {
        &self.metadata.arguments
    }
    
    /// 获取命名参数
    ///
    /// 对应 PHP 的 ReflectionAttribute::getNamedArguments()
    pub fn get_named_arguments(&self) -> &HashMap<String, String> {
        &self.metadata.named_arguments
    }
    
    /// 获取指定位置参数
    ///
    /// # 参数
    /// - `index`: 参数位置
    pub fn get_argument(&self, index: usize) -> Option<&str> {
        self.metadata.arguments.get(index).map(|s| s.as_str())
    }
    
    /// 获取命名参数值
    ///
    /// # 参数
    /// - `name`: 参数名
    pub fn get_named_argument(&self, name: &str) -> Option<&str> {
        self.metadata.named_arguments.get(name).map(|s| s.as_str())
    }
    
    /// 检查是否有位置参数
    pub fn has_arguments(&self) -> bool {
        !self.metadata.arguments.is_empty()
    }
    
    /// 检查是否有命名参数
    pub fn has_named_arguments(&self) -> bool {
        !self.metadata.named_arguments.is_empty()
    }
    
    /// 获取参数数量
    pub fn get_argument_count(&self) -> usize {
        self.metadata.arguments.len() + self.metadata.named_arguments.len()
    }
    
    /// 检查是否可重复
    ///
    /// 对应 PHP 的 ReflectionAttribute::isRepeated()
    pub fn is_repeated(&self) -> bool {
        // 在实际实现中，需要检查注解类的 Attribute 声明
        false
    }
    
    /// 获取注解实例
    ///
    /// 对应 PHP 的 ReflectionAttribute::newInstance()
    /// 在 Rust 实现中返回元数据
    pub fn get_instance(&self) -> &AttributeMetadata {
        &self.metadata
    }
    
    /// 添加位置参数
    pub fn add_argument(&mut self, argument: &str) {
        self.metadata.arguments.push(argument.to_string());
    }
    
    /// 添加命名参数
    pub fn add_named_argument(&mut self, name: &str, value: &str) {
        self.metadata.named_arguments.insert(name.to_string(), value.to_string());
    }
    
    /// 设置目标
    pub fn set_target(&mut self, target: AttributeTarget) {
        self.metadata.target = target;
    }
}

impl fmt::Display for ReflectionAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Attribute: {}", self.metadata.name)?;
        
        if !self.metadata.arguments.is_empty() || !self.metadata.named_arguments.is_empty() {
            write!(f, "(")?;
            
            let mut parts = Vec::new();
            
            // 位置参数
            for arg in &self.metadata.arguments {
                parts.push(arg.clone());
            }
            
            // 命名参数
            for (name, value) in &self.metadata.named_arguments {
                parts.push(format!("{}: {}", name, value));
            }
            
            write!(f, "{})", parts.join(", "))?;
        }
        
        Ok(())
    }
}

/// 注解构建器
///
/// 提供链式 API 来构建注解
pub struct AttributeBuilder {
    name: String,
    arguments: Vec<String>,
    named_arguments: HashMap<String, String>,
    target: AttributeTarget,
}

impl AttributeBuilder {
    /// 创建新的构建器
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            arguments: Vec::new(),
            named_arguments: HashMap::new(),
            target: AttributeTarget::Class,
        }
    }
    
    /// 添加位置参数
    pub fn argument(mut self, value: &str) -> Self {
        self.arguments.push(value.to_string());
        self
    }
    
    /// 添加命名参数
    pub fn named_argument(mut self, name: &str, value: &str) -> Self {
        self.named_arguments.insert(name.to_string(), value.to_string());
        self
    }
    
    /// 设置目标
    pub fn target(mut self, target: AttributeTarget) -> Self {
        self.target = target;
        self
    }
    
    /// 构建 ReflectionAttribute
    pub fn build(self) -> ReflectionAttribute {
        ReflectionAttribute {
            metadata: AttributeMetadata {
                name: self.name,
                arguments: self.arguments,
                named_arguments: self.named_arguments,
                target: self.target,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_attribute_name() {
        let attr = ReflectionAttribute::from_name("Attribute");
        
        assert_eq!(attr.get_name(), "Attribute");
    }
    
    #[test]
    fn test_attribute_arguments() {
        let mut attr = ReflectionAttribute::from_name("Route");
        
        attr.add_argument("'/api/users'");
        attr.add_named_argument("methods", "['GET', 'POST']");
        
        assert_eq!(attr.get_argument_count(), 2);
        assert_eq!(attr.get_argument(0), Some("'/api/users'"));
        assert_eq!(attr.get_named_argument("methods"), Some("['GET', 'POST']"));
    }
    
    #[test]
    fn test_attribute_builder() {
        let attr = AttributeBuilder::new("Route")
            .argument("'/api/users'")
            .named_argument("methods", "['GET']")
            .target(AttributeTarget::Method)
            .build();
        
        assert_eq!(attr.get_name(), "Route");
        assert_eq!(attr.get_target(), AttributeTarget::Method);
        assert_eq!(attr.get_argument_count(), 2);
    }
    
    #[test]
    fn test_attribute_display() {
        let attr = AttributeBuilder::new("Route")
            .argument("'/api/users'")
            .named_argument("methods", "['GET']")
            .build();
        
        let display = format!("{}", attr);
        assert!(display.contains("Route"));
        assert!(display.contains("'/api/users'"));
    }
}
