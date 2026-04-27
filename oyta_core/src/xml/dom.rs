//! DOM 文档对象模型模块
//!
//! 提供完整的 PHP DOMDocument 功能

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::io::BufRead;

/// XML 文档
///
/// 对应 PHP 的 DOMDocument 类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XmlDocument {
    /// 文档版本
    pub version: String,
    /// 文档编码
    pub encoding: String,
    /// 是否独立
    pub standalone: bool,
    /// 根元素
    pub root: Option<XmlElement>,
    /// 文档类型定义
    pub doctype: Option<XmlDoctype>,
    /// 格式化输出
    pub format_output: bool,
    /// 保留空白
    pub preserve_whitespace: bool,
    /// 验证 OnParse
    pub validate_on_parse: bool,
}

impl XmlDocument {
    /// 创建新的 XML 文档
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            encoding: "UTF-8".to_string(),
            standalone: false,
            root: None,
            doctype: None,
            format_output: false,
            preserve_whitespace: false,
            validate_on_parse: false,
        }
    }
    
    /// 创建带版本和编码的文档
    pub fn with_version_encoding(version: &str, encoding: &str) -> Self {
        Self {
            version: version.to_string(),
            encoding: encoding.to_string(),
            ..Self::new()
        }
    }
    
    /// 从字符串加载 XML
    ///
    /// 对应 PHP 的 DOMDocument::loadXML()
    pub fn load_xml(source: &str) -> Result<Self, String> {
        let mut doc = Self::new();
        
        // 使用 quick-xml 解析
        let mut reader = Reader::from_str(source);
        // quick-xml 0.31+ 使用 Builder 模式配置
        // trim_text 默认为 false
        
        // 解析 XML 声明
        let mut buf = Vec::new();
        
        // 读取第一个事件
        match reader.read_event_into(&mut buf) {
            Ok(Event::Decl(decl)) => {
                // 解析版本
                if let Ok(version) = decl.version() {
                    doc.version = String::from_utf8_lossy(version.as_ref()).to_string();
                }
                // 解析编码
                if let Some(encoding_result) = decl.encoding() {
                    if let Ok(encoding) = encoding_result {
                        doc.encoding = String::from_utf8_lossy(encoding.as_ref()).to_string();
                    }
                }
                // 解析 standalone
                if let Some(standalone_result) = decl.standalone() {
                    if let Ok(standalone) = standalone_result {
                        doc.standalone = standalone.as_ref() == b"yes";
                    }
                }
            }
            _ => {}
        }
        
        // 解析元素
        doc.root = Self::parse_elements(&mut reader)?;
        
        Ok(doc)
    }
    
    /// 解析元素
    fn parse_elements<R: BufRead>(reader: &mut Reader<R>) -> Result<Option<XmlElement>, String> {
        let mut element_stack: Vec<XmlElement> = Vec::new();
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let mut attributes = HashMap::new();
                    
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let value = String::from_utf8_lossy(&attr.value).to_string();
                            attributes.insert(key, value);
                        }
                    }
                    
                    let element = XmlElement {
                        name,
                        value: String::new(),
                        attributes,
                        children: Vec::new(),
                        node_type: XmlNodeType::Element,
                    };
                    
                    element_stack.push(element);
                }
                Ok(Event::Empty(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let mut attributes = HashMap::new();
                    
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let value = String::from_utf8_lossy(&attr.value).to_string();
                            attributes.insert(key, value);
                        }
                    }
                    
                    let element = XmlElement {
                        name,
                        value: String::new(),
                        attributes,
                        children: Vec::new(),
                        node_type: XmlNodeType::Element,
                    };
                    
                    if let Some(parent) = element_stack.last_mut() {
                        parent.children.push(XmlNode::Element(element));
                    } else {
                        return Ok(Some(element));
                    }
                }
                Ok(Event::End(_)) => {
                    if let Some(completed) = element_stack.pop() {
                        if let Some(parent) = element_stack.last_mut() {
                            parent.children.push(XmlNode::Element(completed));
                        } else {
                            return Ok(Some(completed));
                        }
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().unwrap_or_default().to_string();
                    if !text.trim().is_empty() {
                        if let Some(parent) = element_stack.last_mut() {
                            if parent.children.is_empty() && parent.value.is_empty() {
                                parent.value = text;
                            } else {
                                parent.children.push(XmlNode::Text(text));
                            }
                        }
                    }
                }
                Ok(Event::CData(e)) => {
                    let text = String::from_utf8_lossy(&e.into_inner()).to_string();
                    if let Some(parent) = element_stack.last_mut() {
                        parent.children.push(XmlNode::CData(text));
                    }
                }
                Ok(Event::Comment(e)) => {
                    let text = String::from_utf8_lossy(&e.into_inner()).to_string();
                    if let Some(parent) = element_stack.last_mut() {
                        parent.children.push(XmlNode::Comment(text));
                    }
                }
                Ok(Event::PI(e)) => {
                    let text = String::from_utf8_lossy(&e.into_inner()).to_string();
                    if let Some(parent) = element_stack.last_mut() {
                        parent.children.push(XmlNode::ProcessingInstruction(text));
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(format!("XML 解析错误: {:?}", e)),
                _ => {}
            }
            
            buf.clear();
        }
        
        Ok(element_stack.pop())
    }
    
    /// 从文件加载 XML
    ///
    /// 对应 PHP 的 DOMDocument::load()
    pub fn load(file_path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("无法读取文件: {}", e))?;
        Self::load_xml(&content)
    }
    
    /// 保存 XML 到字符串
    ///
    /// 对应 PHP 的 DOMDocument::saveXML()
    pub fn save_xml(&self) -> String {
        let mut result = String::new();
        
        // XML 声明
        result.push_str(&format!(
            "<?xml version=\"{}\" encoding=\"{}\"?>\n",
            self.version, self.encoding
        ));
        
        // 根元素
        if let Some(root) = &self.root {
            result.push_str(&root.to_xml_string(0, self.format_output));
        }
        
        result
    }
    
    /// 保存 XML 到文件
    ///
    /// 对应 PHP 的 DOMDocument::save()
    pub fn save(&self, file_path: &str) -> Result<(), String> {
        let content = self.save_xml();
        std::fs::write(file_path, content)
            .map_err(|e| format!("无法写入文件: {}", e))
    }
    
    /// 创建元素
    ///
    /// 对应 PHP 的 DOMDocument::createElement()
    pub fn create_element(&self, name: &str) -> XmlElement {
        XmlElement {
            name: name.to_string(),
            value: String::new(),
            attributes: HashMap::new(),
            children: Vec::new(),
            node_type: XmlNodeType::Element,
        }
    }
    
    /// 创建带值的元素
    pub fn create_element_with_value(&self, name: &str, value: &str) -> XmlElement {
        XmlElement {
            name: name.to_string(),
            value: value.to_string(),
            attributes: HashMap::new(),
            children: Vec::new(),
            node_type: XmlNodeType::Element,
        }
    }
    
    /// 创建文本节点
    pub fn create_text_node(&self, content: &str) -> XmlTextNode {
        XmlTextNode {
            content: content.to_string(),
        }
    }
    
    /// 创建属性
    pub fn create_attribute(&self, name: &str, value: &str) -> XmlAttribute {
        XmlAttribute {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
    
    /// 创建 CDATA 节点
    pub fn create_cdata_section(&self, content: &str) -> XmlCData {
        XmlCData {
            content: content.to_string(),
        }
    }
    
    /// 创建注释节点
    pub fn create_comment(&self, content: &str) -> XmlComment {
        XmlComment {
            content: content.to_string(),
        }
    }
    
    /// 获取文档元素（根元素）
    pub fn get_document_element(&self) -> Option<&XmlElement> {
        self.root.as_ref()
    }
    
    /// 获取文档元素（可变）
    pub fn get_document_element_mut(&mut self) -> Option<&mut XmlElement> {
        self.root.as_mut()
    }
    
    /// 设置根元素
    pub fn set_root(&mut self, element: XmlElement) {
        self.root = Some(element);
    }
    
    /// 获取元素通过 ID
    pub fn get_element_by_id(&self, id: &str) -> Option<&XmlElement> {
        self.root.as_ref()?.find_by_attribute("id", id)
    }
    
    /// 获取元素通过标签名
    pub fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<&XmlElement> {
        match &self.root {
            Some(root) => root.find_all_by_tag_name(tag_name),
            None => Vec::new(),
        }
    }
    
    /// 验证 XML
    pub fn validate(&self) -> bool {
        // 简单验证：检查是否有根元素
        self.root.is_some()
    }
}

impl Default for XmlDocument {
    fn default() -> Self {
        Self::new()
    }
}

/// XML 元素
///
/// 对应 PHP 的 DOMElement 类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XmlElement {
    /// 元素名称
    pub name: String,
    /// 元素文本值
    pub value: String,
    /// 属性列表
    pub attributes: HashMap<String, String>,
    /// 子节点列表
    pub children: Vec<XmlNode>,
    /// 节点类型
    pub node_type: XmlNodeType,
}

impl XmlElement {
    /// 创建新元素
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: String::new(),
            attributes: HashMap::new(),
            children: Vec::new(),
            node_type: XmlNodeType::Element,
        }
    }
    
    /// 创建带值的元素
    pub fn with_value(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            attributes: HashMap::new(),
            children: Vec::new(),
            node_type: XmlNodeType::Element,
        }
    }
    
    /// 获取标签名
    pub fn get_tag_name(&self) -> &str {
        &self.name
    }
    
    /// 获取文本内容
    pub fn get_text_content(&self) -> String {
        if !self.value.is_empty() {
            return self.value.clone();
        }
        
        // 合并所有文本子节点
        let mut text = String::new();
        for child in &self.children {
            if let XmlNode::Text(t) = child {
                text.push_str(t);
            } else if let XmlNode::Element(e) = child {
                text.push_str(&e.get_text_content());
            }
        }
        text
    }
    
    /// 设置文本内容
    pub fn set_text_content(&mut self, content: &str) {
        self.value = content.to_string();
        self.children.clear();
    }
    
    /// 获取属性
    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }
    
    /// 设置属性
    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_string(), value.to_string());
    }
    
    /// 移除属性
    pub fn remove_attribute(&mut self, name: &str) -> Option<String> {
        self.attributes.remove(name)
    }
    
    /// 检查是否有属性
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }
    
    /// 添加子元素
    pub fn append_child(&mut self, child: XmlElement) {
        self.children.push(XmlNode::Element(child));
    }
    
    /// 添加子节点
    pub fn append_node(&mut self, node: XmlNode) {
        self.children.push(node);
    }
    
    /// 获取子元素列表
    pub fn get_child_elements(&self) -> Vec<&XmlElement> {
        self.children
            .iter()
            .filter_map(|n| {
                if let XmlNode::Element(e) = n {
                    Some(e)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// 获取第一个子元素
    pub fn get_first_child(&self) -> Option<&XmlNode> {
        self.children.first()
    }
    
    /// 获取最后一个子元素
    pub fn get_last_child(&self) -> Option<&XmlNode> {
        self.children.last()
    }
    
    /// 查找子元素通过属性
    pub fn find_by_attribute(&self, attr_name: &str, attr_value: &str) -> Option<&XmlElement> {
        if self.get_attribute(attr_name).map(|v| v == attr_value).unwrap_or(false) {
            return Some(self);
        }
        
        for child in &self.children {
            if let XmlNode::Element(e) = child {
                if let Some(found) = e.find_by_attribute(attr_name, attr_value) {
                    return Some(found);
                }
            }
        }
        
        None
    }
    
    /// 查找所有元素通过标签名
    pub fn find_all_by_tag_name(&self, tag_name: &str) -> Vec<&XmlElement> {
        let mut result = Vec::new();
        
        if self.name == tag_name {
            result.push(self);
        }
        
        for child in &self.children {
            if let XmlNode::Element(e) = child {
                result.extend(e.find_all_by_tag_name(tag_name));
            }
        }
        
        result
    }
    
    /// 转换为 XML 字符串
    pub fn to_xml_string(&self, indent: usize, format: bool) -> String {
        let mut result = String::new();
        let indent_str = if format { "  ".repeat(indent) } else { "".to_string() };
        
        // 开始标签
        result.push_str(&indent_str);
        result.push('<');
        result.push_str(&self.name);
        
        // 属性
        for (name, value) in &self.attributes {
            result.push_str(&format!(" {}=\"{}\"", name, Self::escape_attribute(value)));
        }
        
        // 检查是否为空元素
        if self.value.is_empty() && self.children.is_empty() {
            result.push_str("/>");
            if format {
                result.push('\n');
            }
            return result;
        }
        
        result.push('>');
        
        // 内容
        if !self.value.is_empty() {
            result.push_str(&Self::escape_text(&self.value));
        }
        
        // 子节点
        for child in &self.children {
            match child {
                XmlNode::Element(e) => {
                    if format {
                        result.push('\n');
                    }
                    result.push_str(&e.to_xml_string(indent + 1, format));
                }
                XmlNode::Text(t) => {
                    result.push_str(&Self::escape_text(t));
                }
                XmlNode::CData(c) => {
                    result.push_str(&format!("<![CDATA[{}]]>", c));
                }
                XmlNode::Comment(c) => {
                    result.push_str(&format!("<!--{}-->", c));
                }
                XmlNode::ProcessingInstruction(pi) => {
                    result.push_str(&format!("<?{}?>", pi));
                }
            }
        }
        
        // 结束标签
        if format && !self.children.is_empty() {
            result.push('\n');
            result.push_str(&indent_str);
        }
        result.push_str(&format!("</{}>", self.name));
        if format {
            result.push('\n');
        }
        
        result
    }
    
    /// 转义属性值
    fn escape_attribute(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
    
    /// 转义文本内容
    fn escape_text(text: &str) -> String {
        text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
}

impl fmt::Display for XmlElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_xml_string(0, false))
    }
}

/// XML 节点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum XmlNodeType {
    /// 元素节点
    Element,
    /// 属性节点
    Attribute,
    /// 文本节点
    Text,
    /// CDATA 节点
    CData,
    /// 注释节点
    Comment,
    /// 文档节点
    Document,
    /// 文档类型节点
    DocumentType,
    /// 处理指令节点
    ProcessingInstruction,
}

/// XML 节点
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum XmlNode {
    /// 元素
    Element(XmlElement),
    /// 文本
    Text(String),
    /// CDATA
    CData(String),
    /// 注释
    Comment(String),
    /// 处理指令
    ProcessingInstruction(String),
}

/// XML 文本节点
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XmlTextNode {
    /// 文本内容
    pub content: String,
}

/// XML 属性
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XmlAttribute {
    /// 属性名
    pub name: String,
    /// 属性值
    pub value: String,
}

/// XML CDATA 节点
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XmlCData {
    /// CDATA 内容
    pub content: String,
}

/// XML 注释
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XmlComment {
    /// 注释内容
    pub content: String,
}

/// XML 文档类型定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XmlDoctype {
    /// 名称
    pub name: String,
    /// 公共标识符
    pub public_id: Option<String>,
    /// 系统标识符
    pub system_id: Option<String>,
    /// 内部子集
    pub internal_subset: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_xml_document_creation() {
        let doc = XmlDocument::new();
        
        assert_eq!(doc.version, "1.0");
        assert_eq!(doc.encoding, "UTF-8");
        assert!(doc.root.is_none());
    }
    
    #[test]
    fn test_xml_element_creation() {
        let element = XmlElement::with_value("name", "John");
        
        assert_eq!(element.get_tag_name(), "name");
        assert_eq!(element.get_text_content(), "John");
    }
    
    #[test]
    fn test_xml_element_attributes() {
        let mut element = XmlElement::new("person");
        
        element.set_attribute("id", "1");
        element.set_attribute("name", "John");
        
        assert_eq!(element.get_attribute("id"), Some(&"1".to_string()));
        assert!(element.has_attribute("name"));
        
        element.remove_attribute("name");
        assert!(!element.has_attribute("name"));
    }
    
    #[test]
    fn test_xml_element_children() {
        let mut parent = XmlElement::new("parent");
        let child = XmlElement::with_value("child", "value");
        
        parent.append_child(child);
        
        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.get_child_elements().len(), 1);
    }
    
    #[test]
    fn test_xml_load() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
    <person id="1">John</person>
</root>"#;
        
        let doc = XmlDocument::load_xml(xml);
        assert!(doc.is_ok());
        
        let doc = doc.unwrap();
        assert_eq!(doc.version, "1.0");
        assert_eq!(doc.encoding, "UTF-8");
        assert!(doc.root.is_some());
    }
    
    #[test]
    fn test_xml_save() {
        let mut doc = XmlDocument::new();
        let mut root = XmlElement::new("root");
        let child = XmlElement::with_value("name", "John");
        root.append_child(child);
        doc.set_root(root);
        
        let xml = doc.save_xml();
        assert!(xml.contains("<root>"));
        assert!(xml.contains("<name>John</name>"));
    }
}
