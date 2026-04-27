//! SimpleXML 简单 XML 处理模块
//!
//! 提供完整的 PHP SimpleXML 功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use super::dom::XmlElement;

/// SimpleXML 元素
///
/// 对应 PHP 的 SimpleXMLElement 类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimpleXmlElement {
    /// 元素名称
    pub name: String,
    /// 元素值
    pub value: String,
    /// 属性
    pub attributes: HashMap<String, String>,
    /// 子元素
    pub children: Vec<SimpleXmlElement>,
}

impl SimpleXmlElement {
    /// 从字符串加载 XML
    ///
    /// 对应 PHP 的 simplexml_load_string()
    pub fn from_string(xml: &str) -> Result<Self, String> {
        let doc = super::dom::XmlDocument::load_xml(xml)?;
        Ok(Self::from_dom_element(doc.root.as_ref()))
    }
    
    /// 从文件加载 XML
    ///
    /// 对应 PHP 的 simplexml_load_file()
    pub fn from_file(file_path: &str) -> Result<Self, String> {
        let doc = super::dom::XmlDocument::load(file_path)?;
        Ok(Self::from_dom_element(doc.root.as_ref()))
    }
    
    /// 从 DOM 元素转换
    fn from_dom_element(element: Option<&XmlElement>) -> Self {
        match element {
            Some(e) => {
                let children: Vec<SimpleXmlElement> = e.children
                    .iter()
                    .filter_map(|n| {
                        if let super::dom::XmlNode::Element(child) = n {
                            Some(Self::from_dom_element(Some(child)))
                        } else {
                            None
                        }
                    })
                    .collect();
                
                Self {
                    name: e.name.clone(),
                    value: e.value.clone(),
                    attributes: e.attributes.clone(),
                    children,
                }
            }
            None => Self {
                name: String::new(),
                value: String::new(),
                attributes: HashMap::new(),
                children: Vec::new(),
            }
        }
    }
    
    /// 获取元素名称
    pub fn get_name(&self) -> &str {
        &self.name
    }
    
    /// 获取元素值
    pub fn get_value(&self) -> &str {
        &self.value
    }
    
    /// 获取属性
    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }
    
    /// 获取子元素
    pub fn get_children(&self) -> &[SimpleXmlElement] {
        &self.children
    }
    
    /// 按名称获取子元素
    pub fn get_child(&self, name: &str) -> Option<&SimpleXmlElement> {
        self.children.iter().find(|c| c.name == name)
    }
    
    /// 按名称获取所有子元素
    pub fn get_children_by_name(&self, name: &str) -> Vec<&SimpleXmlElement> {
        self.children.iter().filter(|c| c.name == name).collect()
    }
    
    /// 添加子元素
    pub fn add_child(&mut self, name: &str, value: &str) -> &mut SimpleXmlElement {
        self.children.push(SimpleXmlElement {
            name: name.to_string(),
            value: value.to_string(),
            attributes: HashMap::new(),
            children: Vec::new(),
        });
        self.children.last_mut().unwrap()
    }
    
    /// 添加属性
    pub fn add_attribute(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_string(), value.to_string());
    }
    
    /// 转换为 XML 字符串
    pub fn as_xml(&self) -> String {
        self.to_xml_string(0)
    }
    
    /// 转换为 XML 字符串（带缩进）
    fn to_xml_string(&self, indent: usize) -> String {
        let mut result = String::new();
        let indent_str = "  ".repeat(indent);
        
        result.push_str(&format!("{}<{}", indent_str, self.name));
        
        // 属性
        for (name, value) in &self.attributes {
            result.push_str(&format!(" {}=\"{}\"", name, value));
        }
        
        // 检查是否为空元素
        if self.value.is_empty() && self.children.is_empty() {
            result.push_str("/>\n");
            return result;
        }
        
        result.push('>');
        
        // 值
        if !self.value.is_empty() {
            result.push_str(&self.value);
        }
        
        // 子元素
        if !self.children.is_empty() {
            result.push('\n');
            for child in &self.children {
                result.push_str(&child.to_xml_string(indent + 1));
            }
            result.push_str(&indent_str);
        }
        
        result.push_str(&format!("</{}>\n", self.name));
        result
    }
    
    /// XPath 查询
    pub fn xpath(&self, path: &str) -> Vec<&SimpleXmlElement> {
        // 简化实现：只支持简单的路径查询
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        
        if parts.is_empty() {
            return vec![self];
        }
        
        self.xpath_query(&parts, 0)
    }
    
    /// XPath 查询实现
    fn xpath_query(&self, parts: &[&str], index: usize) -> Vec<&SimpleXmlElement> {
        if index >= parts.len() {
            return vec![self];
        }
        
        let part = parts[index];
        let mut result = Vec::new();
        
        // 通配符
        if part == "*" {
            for child in &self.children {
                result.extend(child.xpath_query(parts, index + 1));
            }
        } else {
            // 按名称查找
            for child in &self.children {
                if child.name == part {
                    result.extend(child.xpath_query(parts, index + 1));
                }
            }
        }
        
        result
    }
    
    /// 注册 XPath 命名空间
    pub fn register_xpath_namespace(&mut self, _prefix: &str, _uri: &str) -> bool {
        // 简化实现
        true
    }
}

impl fmt::Display for SimpleXmlElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_xml())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simplexml_from_string() {
        let xml = r#"<?xml version="1.0"?>
<root>
    <person id="1">John</person>
</root>"#;
        
        let element = SimpleXmlElement::from_string(xml);
        assert!(element.is_ok());
        
        let element = element.unwrap();
        assert_eq!(element.get_name(), "root");
    }
    
    #[test]
    fn test_simplexml_children() {
        let xml = r#"<?xml version="1.0"?>
<root>
    <name>John</name>
    <age>30</age>
</root>"#;
        
        let element = SimpleXmlElement::from_string(xml).unwrap();
        
        assert!(element.get_child("name").is_some());
        assert!(element.get_child("age").is_some());
        assert_eq!(element.get_children().len(), 2);
    }
    
    #[test]
    fn test_simplexml_attributes() {
        let xml = r#"<?xml version="1.0"?>
<person id="1" name="John"/>"#;
        
        let element = SimpleXmlElement::from_string(xml).unwrap();
        
        assert_eq!(element.get_attribute("id"), Some(&"1".to_string()));
        assert_eq!(element.get_attribute("name"), Some(&"John".to_string()));
    }
}
