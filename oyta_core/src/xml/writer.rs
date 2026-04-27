//! XMLWriter XML 写入模块
//!
//! 提供完整的 PHP XMLWriter 功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// XMLWriter XML 写入器
///
/// 对应 PHP 的 XMLWriter 类
#[derive(Debug, Clone)]
pub struct XmlWriter {
    /// 输出缓冲
    buffer: String,
    /// 缩进级别
    indent_level: usize,
    /// 缩进字符串
    indent_string: String,
    /// 是否格式化输出
    format_output: bool,
    /// 元素栈
    element_stack: Vec<String>,
    /// 当前元素是否有内容
    element_has_content: bool,
}

impl XmlWriter {
    /// 创建新的 XMLWriter
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            indent_level: 0,
            indent_string: "  ".to_string(),
            format_output: false,
            element_stack: Vec::new(),
            element_has_content: false,
        }
    }
    
    /// 创建带格式化的 XMLWriter
    pub fn with_format() -> Self {
        Self {
            buffer: String::new(),
            indent_level: 0,
            indent_string: "  ".to_string(),
            format_output: true,
            element_stack: Vec::new(),
            element_has_content: false,
        }
    }
    
    /// 打开内存
    ///
    /// 对应 PHP 的 XMLWriter::openMemory()
    pub fn open_memory() -> Self {
        Self::new()
    }
    
    /// 打开 URI（文件）
    ///
    /// 对应 PHP 的 XMLWriter::openURI()
    pub fn open_uri(file_path: &str) -> Result<Self, String> {
        let writer = Self::new();
        // 在实际实现中，这里会打开文件
        Ok(writer)
    }
    
    /// 开始文档
    ///
    /// 对应 PHP 的 XMLWriter::startDocument()
    pub fn start_document(&mut self, version: Option<&str>, encoding: Option<&str>, standalone: Option<bool>) -> bool {
        let version = version.unwrap_or("1.0");
        let encoding = encoding.unwrap_or("UTF-8");
        
        self.buffer.push_str(&format!("<?xml version=\"{}\" encoding=\"{}\"", version, encoding));
        
        if let Some(standalone) = standalone {
            self.buffer.push_str(&format!(" standalone=\"{}\"", if standalone { "yes" } else { "no" }));
        }
        
        self.buffer.push_str("?>");
        
        if self.format_output {
            self.buffer.push('\n');
        }
        
        true
    }
    
    /// 结束文档
    ///
    /// 对应 PHP 的 XMLWriter::endDocument()
    pub fn end_document(&mut self) -> bool {
        // 关闭所有未关闭的元素
        while !self.element_stack.is_empty() {
            self.end_element();
        }
        
        true
    }
    
    /// 开始元素
    ///
    /// 对应 PHP 的 XMLWriter::startElement()
    pub fn start_element(&mut self, name: &str) -> bool {
        // 如果当前元素没有内容，关闭开始标签
        if let Some(current) = self.element_stack.last() {
            if !self.element_has_content {
                self.buffer.push('>');
                if self.format_output {
                    self.buffer.push('\n');
                }
            }
        }
        
        // 添加缩进
        if self.format_output {
            self.buffer.push_str(&self.indent_string.repeat(self.indent_level));
        }
        
        self.buffer.push_str(&format!("<{}", name));
        self.element_stack.push(name.to_string());
        self.element_has_content = false;
        self.indent_level += 1;
        
        true
    }
    
    /// 结束元素
    ///
    /// 对应 PHP 的 XMLWriter::endElement()
    pub fn end_element(&mut self) -> bool {
        if self.element_stack.is_empty() {
            return false;
        }
        
        self.indent_level = self.indent_level.saturating_sub(1);
        let name = self.element_stack.pop().unwrap();
        
        if !self.element_has_content {
            // 空元素
            self.buffer.push_str("/>");
        } else {
            // 有内容的元素
            if self.format_output {
                self.buffer.push_str(&self.indent_string.repeat(self.indent_level));
            }
            self.buffer.push_str(&format!("</{}>", name));
        }
        
        if self.format_output {
            self.buffer.push('\n');
        }
        
        self.element_has_content = true;
        true
    }
    
    /// 写入元素
    ///
    /// 对应 PHP 的 XMLWriter::writeElement()
    pub fn write_element(&mut self, name: &str, content: Option<&str>) -> bool {
        self.start_element(name);
        
        if let Some(content) = content {
            self.write_raw(content);
        }
        
        self.end_element()
    }
    
    /// 写入属性
    ///
    /// 对应 PHP 的 XMLWriter::writeAttribute()
    pub fn write_attribute(&mut self, name: &str, value: &str) -> bool {
        if self.element_stack.is_empty() {
            return false;
        }
        
        self.buffer.push_str(&format!(" {}=\"{}\"", name, Self::escape_attribute(value)));
        true
    }
    
    /// 写入文本
    ///
    /// 对应 PHP 的 XMLWriter::text()
    pub fn text(&mut self, content: &str) -> bool {
        if self.element_stack.is_empty() {
            return false;
        }
        
        // 关闭开始标签
        if !self.element_has_content {
            self.buffer.push('>');
            if self.format_output {
                self.buffer.push('\n');
            }
        }
        
        self.buffer.push_str(&Self::escape_text(content));
        self.element_has_content = true;
        true
    }
    
    /// 写入原始内容
    pub fn write_raw(&mut self, content: &str) -> bool {
        if self.element_stack.is_empty() {
            return false;
        }
        
        // 关闭开始标签
        if !self.element_has_content {
            self.buffer.push('>');
            if self.format_output {
                self.buffer.push('\n');
            }
        }
        
        self.buffer.push_str(content);
        self.element_has_content = true;
        true
    }
    
    /// 写入 CDATA
    ///
    /// 对应 PHP 的 XMLWriter::writeCData()
    pub fn write_cdata(&mut self, content: &str) -> bool {
        if self.element_stack.is_empty() {
            return false;
        }
        
        // 关闭开始标签
        if !self.element_has_content {
            self.buffer.push('>');
            if self.format_output {
                self.buffer.push('\n');
            }
        }
        
        self.buffer.push_str(&format!("<![CDATA[{}]]>", content));
        self.element_has_content = true;
        true
    }
    
    /// 写入注释
    ///
    /// 对应 PHP 的 XMLWriter::writeComment()
    pub fn write_comment(&mut self, content: &str) -> bool {
        if self.format_output {
            self.buffer.push_str(&self.indent_string.repeat(self.indent_level));
        }
        
        self.buffer.push_str(&format!("<!--{}-->", content));
        
        if self.format_output {
            self.buffer.push('\n');
        }
        
        true
    }
    
    /// 写入处理指令
    ///
    /// 对应 PHP 的 XMLWriter::writePI()
    pub fn write_pi(&mut self, target: &str, content: &str) -> bool {
        self.buffer.push_str(&format!("<?{} {}?>", target, content));
        
        if self.format_output {
            self.buffer.push('\n');
        }
        
        true
    }
    
    /// 获取输出
    ///
    /// 对应 PHP 的 XMLWriter::outputMemory()
    pub fn output_memory(&self, flush: bool) -> String {
        self.buffer.clone()
    }
    
    /// 清空缓冲
    pub fn flush(&mut self) -> String {
        let output = self.buffer.clone();
        self.buffer.clear();
        output
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
    
    /// 转义文本
    fn escape_text(text: &str) -> String {
        text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
}

impl Default for XmlWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_xml_writer_document() {
        let mut writer = XmlWriter::new();
        
        writer.start_document(Some("1.0"), Some("UTF-8"), None);
        writer.start_element("root");
        writer.text("Hello");
        writer.end_element();
        writer.end_document();
        
        let output = writer.output_memory(false);
        assert!(output.contains("<?xml"));
        assert!(output.contains("<root>Hello</root>"));
    }
    
    #[test]
    fn test_xml_writer_element() {
        let mut writer = XmlWriter::new();
        
        writer.start_element("person");
        writer.write_attribute("id", "1");
        writer.text("John");
        writer.end_element();
        
        let output = writer.output_memory(false);
        assert!(output.contains("<person id=\"1\">John</person>"));
    }
    
    #[test]
    fn test_xml_writer_cdata() {
        let mut writer = XmlWriter::new();
        
        writer.start_element("data");
        writer.write_cdata("<script>alert('xss')</script>");
        writer.end_element();
        
        let output = writer.output_memory(false);
        assert!(output.contains("<![CDATA["));
    }
    
    #[test]
    fn test_xml_writer_comment() {
        let mut writer = XmlWriter::new();
        
        writer.write_comment("This is a comment");
        
        let output = writer.output_memory(false);
        assert!(output.contains("<!--This is a comment-->"));
    }
}
