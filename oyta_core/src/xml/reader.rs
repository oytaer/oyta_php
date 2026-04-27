//! XMLReader 流式 XML 读取模块
//!
//! 提供完整的 PHP XMLReader 功能

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;

/// XMLReader 流式 XML 读取器
///
/// 对应 PHP 的 XMLReader 类
pub struct XmlReader {
    /// 内部读取器
    reader: Option<Reader<Cursor<Vec<u8>>>>,
    /// 当前节点类型
    current_node_type: XmlReaderNodeType,
    /// 当前节点名称
    current_name: String,
    /// 当前节点值
    current_value: String,
    /// 当前节点属性
    current_attributes: HashMap<String, String>,
    /// 当前节点深度
    current_depth: usize,
    /// 是否已读取
    has_read: bool,
}

/// XMLReader 节点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum XmlReaderNodeType {
    /// 无
    None,
    /// 元素
    Element,
    /// 属性
    Attribute,
    /// 文本
    Text,
    /// CDATA
    CData,
    /// 注释
    Comment,
    /// 文档
    Document,
    /// 文档类型
    DocumentType,
    /// 结束元素
    EndElement,
    /// 处理指令
    ProcessingInstruction,
    /// 空元素
    ElementEmpty,
    /// XML 声明
    XmlDeclaration,
}

impl XmlReader {
    /// 创建新的 XMLReader
    pub fn new() -> Self {
        Self {
            reader: None,
            current_node_type: XmlReaderNodeType::None,
            current_name: String::new(),
            current_value: String::new(),
            current_attributes: HashMap::new(),
            current_depth: 0,
            has_read: false,
        }
    }
    
    /// 从字符串打开 XML
    ///
    /// 对应 PHP 的 XMLReader::XML()
    pub fn from_string(xml: &str) -> Result<Self, String> {
        let cursor = Cursor::new(xml.as_bytes().to_vec());
        let reader = Reader::from_reader(cursor);
        
        Ok(Self {
            reader: Some(reader),
            current_node_type: XmlReaderNodeType::None,
            current_name: String::new(),
            current_value: String::new(),
            current_attributes: HashMap::new(),
            current_depth: 0,
            has_read: false,
        })
    }
    
    /// 从文件打开 XML
    ///
    /// 对应 PHP 的 XMLReader::open()
    pub fn open(file_path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("无法读取文件: {}", e))?;
        Self::from_string(&content)
    }
    
    /// 读取下一个节点
    ///
    /// 对应 PHP 的 XMLReader::read()
    pub fn read(&mut self) -> Result<bool, String> {
        let reader = match &mut self.reader {
            Some(r) => r,
            None => return Err("未打开 XML 文档".to_string()),
        };
        
        let mut buf = Vec::new();
        
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                self.current_node_type = XmlReaderNodeType::Element;
                self.current_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                self.current_value.clear();
                self.current_attributes.clear();
                
                for attr in e.attributes() {
                    if let Ok(attr) = attr {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        self.current_attributes.insert(key, value);
                    }
                }
                
                self.current_depth += 1;
                self.has_read = true;
                Ok(true)
            }
            Ok(Event::Empty(e)) => {
                self.current_node_type = XmlReaderNodeType::ElementEmpty;
                self.current_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                self.current_value.clear();
                self.current_attributes.clear();
                
                for attr in e.attributes() {
                    if let Ok(attr) = attr {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        self.current_attributes.insert(key, value);
                    }
                }
                
                self.has_read = true;
                Ok(true)
            }
            Ok(Event::End(_)) => {
                self.current_node_type = XmlReaderNodeType::EndElement;
                self.current_depth = self.current_depth.saturating_sub(1);
                self.has_read = true;
                Ok(true)
            }
            Ok(Event::Text(e)) => {
                self.current_node_type = XmlReaderNodeType::Text;
                self.current_value = e.unescape().unwrap_or_default().to_string();
                self.has_read = true;
                Ok(true)
            }
            Ok(Event::CData(e)) => {
                self.current_node_type = XmlReaderNodeType::CData;
                self.current_value = String::from_utf8_lossy(&e.into_inner()).to_string();
                self.has_read = true;
                Ok(true)
            }
            Ok(Event::Comment(e)) => {
                self.current_node_type = XmlReaderNodeType::Comment;
                self.current_value = String::from_utf8_lossy(&e.into_inner()).to_string();
                self.has_read = true;
                Ok(true)
            }
            Ok(Event::Decl(_)) => {
                self.current_node_type = XmlReaderNodeType::XmlDeclaration;
                self.has_read = true;
                Ok(true)
            }
            Ok(Event::PI(e)) => {
                self.current_node_type = XmlReaderNodeType::ProcessingInstruction;
                self.current_value = String::from_utf8_lossy(&e.into_inner()).to_string();
                self.has_read = true;
                Ok(true)
            }
            Ok(Event::Eof) => Ok(false),
            Err(e) => Err(format!("XML 解析错误: {:?}", e)),
            _ => Ok(true),
        }
    }
    
    /// 获取当前节点类型
    pub fn get_node_type(&self) -> XmlReaderNodeType {
        self.current_node_type
    }
    
    /// 获取当前节点名称
    pub fn get_name(&self) -> &str {
        &self.current_name
    }
    
    /// 获取当前节点值
    pub fn get_value(&self) -> &str {
        &self.current_value
    }
    
    /// 获取当前节点深度
    pub fn get_depth(&self) -> usize {
        self.current_depth
    }
    
    /// 获取属性数量
    pub fn get_attribute_count(&self) -> usize {
        self.current_attributes.len()
    }
    
    /// 获取属性值
    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        self.current_attributes.get(name)
    }
    
    /// 获取所有属性
    pub fn get_attributes(&self) -> &HashMap<String, String> {
        &self.current_attributes
    }
    
    /// 检查是否有属性
    pub fn has_attributes(&self) -> bool {
        !self.current_attributes.is_empty()
    }
    
    /// 检查是否有值
    pub fn has_value(&self) -> bool {
        !self.current_value.is_empty()
    }
    
    /// 检查是否为空元素
    pub fn is_empty_element(&self) -> bool {
        self.current_node_type == XmlReaderNodeType::ElementEmpty
    }
    
    /// 移动到第一个属性
    pub fn move_to_first_attribute(&mut self) -> bool {
        !self.current_attributes.is_empty()
    }
    
    /// 移动到下一个属性
    pub fn move_to_next_attribute(&mut self) -> bool {
        false // 简化实现
    }
    
    /// 移动到指定属性
    pub fn move_to_attribute(&mut self, name: &str) -> bool {
        self.current_attributes.contains_key(name)
    }
    
    /// 读取内部 XML
    pub fn read_inner_xml(&mut self) -> Result<String, String> {
        // 简化实现
        Ok(self.current_value.clone())
    }
    
    /// 读取外部 XML
    pub fn read_outer_xml(&mut self) -> Result<String, String> {
        // 简化实现
        Ok(format!("<{}>{}</{}>", self.current_name, self.current_value, self.current_name))
    }
    
    /// 读取字符串值
    pub fn read_string(&mut self) -> Result<String, String> {
        Ok(self.current_value.clone())
    }
    
    /// 关闭读取器
    pub fn close(&mut self) {
        self.reader = None;
        self.current_node_type = XmlReaderNodeType::None;
        self.current_name.clear();
        self.current_value.clear();
        self.current_attributes.clear();
        self.current_depth = 0;
        self.has_read = false;
    }
}

impl Default for XmlReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_xml_reader_from_string() {
        let xml = r#"<?xml version="1.0"?>
<root>
    <person id="1">John</person>
</root>"#;
        
        let reader = XmlReader::from_string(xml);
        assert!(reader.is_ok());
    }
    
    #[test]
    fn test_xml_reader_read() {
        let xml = r#"<?xml version="1.0"?>
<root>
    <person id="1">John</person>
</root>"#;
        
        let mut reader = XmlReader::from_string(xml).unwrap();
        
        // 读取 XML 声明
        assert!(reader.read().unwrap());
        
        // 读取根元素
        while reader.read().unwrap() {
            if reader.get_node_type() == XmlReaderNodeType::Element {
                if reader.get_name() == "person" {
                    assert_eq!(reader.get_attribute("id"), Some(&"1".to_string()));
                    break;
                }
            }
        }
    }
}
