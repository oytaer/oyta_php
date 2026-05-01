//! XMLReader 和 XMLWriter 内置类模块
//!
//! 实现 PHP XMLReader、XMLWriter 类
//! 对应 PHP 的 XML 扩展功能
//!
//! # XMLReader 类
//! 流式 XML 读取器，提供向前只读的 XML 解析
//!
//! # XMLWriter 类
//! XML 写入器，用于生成 XML 文档

use std::collections::HashMap;

use crate::interpreter::value::{ObjectInstance, Value};

// ============================================================================
// XMLReader 类方法实现
// ============================================================================

/// XMLReader::new 创建新的 XMLReader 实例
///
/// # PHP 用法
/// ```php
/// $reader = new XMLReader();
/// ```
///
/// # 返回
/// XMLReader 对象实例
pub fn xmlreader_new(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let mut properties = HashMap::new();
    properties.insert("xml".to_string(), Value::String(String::new()));
    properties.insert("nodeType".to_string(), Value::Int(0));
    properties.insert("name".to_string(), Value::String(String::new()));
    properties.insert("value".to_string(), Value::String(String::new()));
    properties.insert("depth".to_string(), Value::Int(0));
    properties.insert("isEmptyElement".to_string(), Value::Bool(false));
    properties.insert("hasAttributes".to_string(), Value::Bool(false));
    properties.insert("hasValue".to_string(), Value::Bool(false));
    properties.insert("attributeCount".to_string(), Value::Int(0));

    Ok(Value::Object(ObjectInstance {
        class_name: "XMLReader".to_string(),
        properties,
    }))
}

/// XMLReader::open 打开 XML 文件
///
/// # PHP 用法
/// ```php
/// $reader = new XMLReader();
/// $reader->open('file.xml');
/// ```
///
/// # 参数
/// - instance: XMLReader 对象实例
/// - args[0]: 文件路径
///
/// # 返回
/// 成功返回 true，失败返回 false
pub fn xmlreader_open(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let file_path = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    // 读取文件内容
    match std::fs::read_to_string(&file_path) {
        Ok(xml_content) => {
            // 更新实例属性
            let mut new_instance = instance.clone();
            new_instance.properties.insert("xml".to_string(), Value::String(xml_content));
            Ok(Value::Object(new_instance))
        }
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// XMLReader::xml 从字符串加载 XML
///
/// # PHP 用法
/// ```php
/// $reader = new XMLReader();
/// $reader->xml('<root><element/></root>');
/// ```
///
/// # 参数
/// - instance: XMLReader 对象实例
/// - args[0]: XML 字符串
///
/// # 返回
/// 成功返回 true，失败返回 false
pub fn xmlreader_xml(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let xml_content = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    // 更新实例属性
    let mut new_instance = instance.clone();
    new_instance.properties.insert("xml".to_string(), Value::String(xml_content));
    Ok(Value::Object(new_instance))
}

/// XMLReader::read 读取下一个节点
///
/// # PHP 用法
/// ```php
/// while ($reader->read()) {
///     // 处理节点
/// }
/// ```
///
/// # 参数
/// - instance: XMLReader 对象实例
///
/// # 返回
/// 成功返回 true，没有更多节点返回 false
pub fn xmlreader_read(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 获取 XML 内容
    let xml = instance.properties.get("xml")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();

    // 使用 crate 的 XML 模块解析
    match crate::xml::reader::XmlReader::from_string(&xml) {
        Ok(mut reader) => {
            match reader.read() {
                Ok(has_more) => Ok(Value::Bool(has_more)),
                Err(_) => Ok(Value::Bool(false)),
            }
        }
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// XMLReader::close 关闭读取器
///
/// # PHP 用法
/// ```php
/// $reader->close();
/// ```
///
/// # 参数
/// - instance: XMLReader 对象实例
///
/// # 返回
/// 成功返回 true
pub fn xmlreader_close(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 清空 XML 内容
    let mut new_instance = instance.clone();
    new_instance.properties.insert("xml".to_string(), Value::String(String::new()));
    Ok(Value::Object(new_instance))
}

/// XMLReader::getName 获取当前节点名称
///
/// # PHP 用法
/// ```php
/// $name = $reader->name;
/// // 或
/// $name = $reader->getName();
/// ```
///
/// # 参数
/// - instance: XMLReader 对象实例
///
/// # 返回
/// 节点名称
pub fn xmlreader_get_name(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let name = instance.properties.get("name")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();
    Ok(Value::String(name))
}

/// XMLReader::nodeType 获取当前节点类型
///
/// # PHP 用法
/// ```php
/// $type = $reader->nodeType;
/// ```
///
/// # 参数
/// - instance: XMLReader 对象实例
///
/// # 返回
/// 节点类型常量
pub fn xmlreader_get_node_type(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let node_type = instance.properties.get("nodeType")
        .and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None })
        .unwrap_or(0);
    Ok(Value::Int(node_type))
}

/// XMLReader::getAttribute 获取属性值
///
/// # PHP 用法
/// ```php
/// $value = $reader->getAttribute('name');
/// ```
///
/// # 参数
/// - instance: XMLReader 对象实例
/// - args[0]: 属性名
///
/// # 返回
/// 属性值，不存在返回 null
pub fn xmlreader_get_attribute(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let attr_name = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    // 简化实现：从属性中获取
    let key = format!("attr_{}", attr_name);
    match instance.properties.get(&key) {
        Some(Value::String(s)) => Ok(Value::String(s.clone())),
        _ => Ok(Value::Null),
    }
}

// ============================================================================
// XMLWriter 类方法实现
// ============================================================================

/// XMLWriter::new 创建新的 XMLWriter 实例
///
/// # PHP 用法
/// ```php
/// $writer = new XMLWriter();
/// ```
///
/// # 返回
/// XMLWriter 对象实例
pub fn xmlwriter_new(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let mut properties = HashMap::new();
    properties.insert("xml".to_string(), Value::String(String::new()));
    properties.insert("indent".to_string(), Value::Bool(false));
    properties.insert("indentString".to_string(), Value::String("  ".to_string()));

    Ok(Value::Object(ObjectInstance {
        class_name: "XMLWriter".to_string(),
        properties,
    }))
}

/// XMLWriter::openMemory 打开内存缓冲区
///
/// # PHP 用法
/// ```php
/// $writer = new XMLWriter();
/// $writer->openMemory();
/// ```
///
/// # 参数
/// - instance: XMLWriter 对象实例
///
/// # 返回
/// 成功返回 true
pub fn xmlwriter_open_memory(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let mut new_instance = instance.clone();
    new_instance.properties.insert("xml".to_string(), Value::String(String::new()));
    Ok(Value::Object(new_instance))
}

/// XMLWriter::openURI 打开文件 URI
///
/// # PHP 用法
/// ```php
/// $writer = new XMLWriter();
/// $writer->openURI('output.xml');
/// ```
///
/// # 参数
/// - instance: XMLWriter 对象实例
/// - args[0]: 文件路径
///
/// # 返回
/// 成功返回 true，失败返回 false
pub fn xmlwriter_open_uri(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let file_path = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    // 检查文件是否可写
    let parent = std::path::Path::new(&file_path).parent();
    if let Some(dir) = parent {
        if !dir.exists() {
            return Ok(Value::Bool(false));
        }
    }

    let mut new_instance = instance.clone();
    new_instance.properties.insert("uri".to_string(), Value::String(file_path));
    Ok(Value::Object(new_instance))
}

/// XMLWriter::startDocument 开始文档
///
/// # PHP 用法
/// ```php
/// $writer->startDocument('1.0', 'UTF-8');
/// ```
///
/// # 参数
/// - instance: XMLWriter 对象实例
/// - args[0]: 版本号（可选，默认 1.0）
/// - args[1]: 编码（可选，默认 UTF-8）
///
/// # 返回
/// 成功返回 true
pub fn xmlwriter_start_document(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let version = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_else(|| "1.0".to_string());
    let encoding = args.get(1)
        .map(|v| v.to_string_value())
        .unwrap_or_else(|| "UTF-8".to_string());

    let declaration = format!("<?xml version=\"{}\" encoding=\"{}\"?>\n", version, encoding);

    let mut new_instance = instance.clone();
    let current_xml = new_instance.properties.get("xml")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();
    new_instance.properties.insert("xml".to_string(), Value::String(current_xml + &declaration));
    Ok(Value::Object(new_instance))
}

/// XMLWriter::endDocument 结束文档
///
/// # PHP 用法
/// ```php
/// $writer->endDocument();
/// ```
///
/// # 参数
/// - instance: XMLWriter 对象实例
///
/// # 返回
/// 成功返回 true
pub fn xmlwriter_end_document(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 检查是否有文件路径，如果有则写入文件
    if let Some(Value::String(uri)) = instance.properties.get("uri") {
        if let Some(Value::String(xml)) = instance.properties.get("xml") {
            let _ = std::fs::write(uri, xml);
        }
    }
    Ok(Value::Bool(true))
}

/// XMLWriter::startElement 开始元素
///
/// # PHP 用法
/// ```php
/// $writer->startElement('root');
/// ```
///
/// # 参数
/// - instance: XMLWriter 对象实例
/// - args[0]: 元素名称
///
/// # 返回
/// 成功返回 true
pub fn xmlwriter_start_element(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let name = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_else(|| "element".to_string());

    let mut new_instance = instance.clone();
    let current_xml = new_instance.properties.get("xml")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();

    // 获取缩进设置
    let indent = new_instance.properties.get("indent")
        .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
        .unwrap_or(false);

    let element_str = if indent {
        format!("<{}>\n", name)
    } else {
        format!("<{}>", name)
    };

    new_instance.properties.insert("xml".to_string(), Value::String(current_xml + &element_str));
    // 记录当前元素名用于闭合
    new_instance.properties.insert("_currentElement".to_string(), Value::String(name));
    Ok(Value::Object(new_instance))
}

/// XMLWriter::endElement 结束元素
///
/// # PHP 用法
/// ```php
/// $writer->endElement();
/// ```
///
/// # 参数
/// - instance: XMLWriter 对象实例
///
/// # 返回
/// 成功返回 true
pub fn xmlwriter_end_element(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let mut new_instance = instance.clone();
    let current_xml = new_instance.properties.get("xml")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();

    // 获取当前元素名
    let element_name = new_instance.properties.get("_currentElement")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_else(|| "element".to_string());

    let end_tag = format!("</{}>", element_name);
    new_instance.properties.insert("xml".to_string(), Value::String(current_xml + &end_tag));
    new_instance.properties.remove("_currentElement");
    Ok(Value::Object(new_instance))
}

/// XMLWriter::writeElement 写入完整元素
///
/// # PHP 用法
/// ```php
/// $writer->writeElement('name', 'value');
/// ```
///
/// # 参数
/// - instance: XMLWriter 对象实例
/// - args[0]: 元素名称
/// - args[1]: 元素内容（可选）
///
/// # 返回
/// 成功返回 true
pub fn xmlwriter_write_element(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let name = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_else(|| "element".to_string());
    let content = args.get(1)
        .map(|v| v.to_string_value());

    let mut new_instance = instance.clone();
    let current_xml = new_instance.properties.get("xml")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();

    let element_str = match content {
        Some(c) => format!("<{}>{}</{}>", name, escape_xml(&c), name),
        None => format!("<{}/>", name),
    };

    new_instance.properties.insert("xml".to_string(), Value::String(current_xml + &element_str));
    Ok(Value::Object(new_instance))
}

/// XMLWriter::text 写入文本内容
///
/// # PHP 用法
/// ```php
/// $writer->text('Hello World');
/// ```
///
/// # 参数
/// - instance: XMLWriter 对象实例
/// - args[0]: 文本内容
///
/// # 返回
/// 成功返回 true
pub fn xmlwriter_text(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let content = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    let mut new_instance = instance.clone();
    let current_xml = new_instance.properties.get("xml")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();

    new_instance.properties.insert("xml".to_string(), Value::String(current_xml + &escape_xml(&content)));
    Ok(Value::Object(new_instance))
}

/// XMLWriter::writeAttribute 写入属性
///
/// # PHP 用法
/// ```php
/// $writer->writeAttribute('name', 'value');
/// ```
///
/// # 参数
/// - instance: XMLWriter 对象实例
/// - args[0]: 属性名
/// - args[1]: 属性值
///
/// # 返回
/// 成功返回 true
pub fn xmlwriter_write_attribute(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let name = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let value = args.get(1)
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    let mut new_instance = instance.clone();
    let current_xml = new_instance.properties.get("xml")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();

    // 在最后一个 > 前插入属性
    if let Some(pos) = current_xml.rfind('>') {
        let (before, after) = current_xml.split_at(pos);
        let attr_str = format!(" {}=\"{}\"", name, escape_attribute(&value));
        new_instance.properties.insert("xml".to_string(), Value::String(before.to_string() + &attr_str + after));
    }

    Ok(Value::Object(new_instance))
}

/// XMLWriter::outputMemory 输出内存内容
///
/// # PHP 用法
/// ```php
/// $xml = $writer->outputMemory();
/// ```
///
/// # 参数
/// - instance: XMLWriter 对象实例
/// - args[0]: 是否清空缓冲区（可选，默认 true）
///
/// # 返回
/// XML 字符串
pub fn xmlwriter_output_memory(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let flush = args.first()
        .map(|v| v.to_int() != 0)
        .unwrap_or(true);

    let xml = instance.properties.get("xml")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();

    if flush {
        // 清空缓冲区
        let mut new_instance = instance.clone();
        new_instance.properties.insert("xml".to_string(), Value::String(String::new()));
        // 不返回修改后的实例，因为这是获取输出
    }

    Ok(Value::String(xml))
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 转义 XML 特殊字符
///
/// # 参数
/// - text: 原始文本
///
/// # 返回
/// 转义后的文本
fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// 转义属性值
///
/// # 参数
/// - value: 原始属性值
///
/// # 返回
/// 转义后的属性值
fn escape_attribute(value: &str) -> String {
    escape_xml(value).replace('"', "&quot;")
}

// ============================================================================
// 类注册函数
// ============================================================================

/// 获取 XMLReader 类定义
///
/// 返回 XMLReader 类的所有方法定义
/// 用于注册到内置类注册表中
pub fn get_xmlreader_class_definition() -> super::types::BuiltinClassDefinition {
    let mut definition = super::types::BuiltinClassDefinition::new("XMLReader");

    // 添加实例方法
    definition = definition
        .add_method("open", xmlreader_open)
        .add_method("xml", xmlreader_xml)
        .add_method("read", xmlreader_read)
        .add_method("close", xmlreader_close)
        .add_method("getName", xmlreader_get_name)
        .add_method("nodeType", xmlreader_get_node_type)
        .add_method("getAttribute", xmlreader_get_attribute);

    // 添加静态方法（构造函数）
    definition = definition.add_static_method("__construct", xmlreader_new);

    definition
}

/// 获取 XMLWriter 类定义
///
/// 返回 XMLWriter 类的所有方法定义
/// 用于注册到内置类注册表中
pub fn get_xmlwriter_class_definition() -> super::types::BuiltinClassDefinition {
    let mut definition = super::types::BuiltinClassDefinition::new("XMLWriter");

    // 添加实例方法
    definition = definition
        .add_method("openMemory", xmlwriter_open_memory)
        .add_method("openURI", xmlwriter_open_uri)
        .add_method("startDocument", xmlwriter_start_document)
        .add_method("endDocument", xmlwriter_end_document)
        .add_method("startElement", xmlwriter_start_element)
        .add_method("endElement", xmlwriter_end_element)
        .add_method("writeElement", xmlwriter_write_element)
        .add_method("text", xmlwriter_text)
        .add_method("writeAttribute", xmlwriter_write_attribute)
        .add_method("outputMemory", xmlwriter_output_memory);

    // 添加静态方法（构造函数）
    definition = definition.add_static_method("__construct", xmlwriter_new);

    definition
}
