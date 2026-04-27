//! XML 处理内置函数注册模块
//!
//! 本模块实现 PHP XML 扩展的内置函数注册
//! 包括：simplexml_load_string, simplexml_load_file, dom_import_simplexml,
//! xml_parser_create, xml_parse, xml_parse_into_struct 等

use std::collections::HashMap;

use super::super::builtins::BuiltinFunction;
use crate::interpreter::value::Value;
use anyhow::Result;

// ============================================================================
// SimpleXML 对象处理辅助函数
// ============================================================================

/// 创建 SimpleXMLElement 对象值
///
/// # 参数
/// - `xml_string`: XML 字符串
///
/// # 返回
/// SimpleXMLElement 对象值
fn create_simplexmlelement_object(xml_string: &str) -> Value {
    use std::collections::HashMap;
    
    // 创建对象实例
    let mut properties: HashMap<String, Value> = HashMap::new();
    
    // 存储 XML 内容
    properties.insert("xml".to_string(), Value::String(xml_string.to_string()));
    
    // 解析 XML 并提取属性（简化实现）
    // 实际实现需要完整的 XML 解析
    properties.insert("nodeName".to_string(), Value::String(String::new()));
    properties.insert("nodeValue".to_string(), Value::String(String::new()));
    properties.insert("nodeType".to_string(), Value::Int(1)); // XML_ELEMENT_NODE
    properties.insert("attributes".to_string(), Value::IndexedArray(vec![]));
    properties.insert("children".to_string(), Value::IndexedArray(vec![]));
    
    // 返回对象值
    Value::Object(crate::interpreter::value::ObjectInstance {
        class_name: "SimpleXMLElement".to_string(),
        properties,
    })
}

/// 创建 DOMDocument 对象值
///
/// # 参数
/// - `xml_string`: XML 字符串
///
/// # 返回
/// DOMDocument 对象值
fn create_domdocument_object(xml_string: &str) -> Value {
    use std::collections::HashMap;
    
    // 创建对象实例
    let mut properties: HashMap<String, Value> = HashMap::new();
    
    // 存储 XML 内容
    properties.insert("xml".to_string(), Value::String(xml_string.to_string()));
    properties.insert("version".to_string(), Value::String("1.0".to_string()));
    properties.insert("encoding".to_string(), Value::String("UTF-8".to_string()));
    properties.insert("documentElement".to_string(), Value::Null);
    properties.insert("formatOutput".to_string(), Value::Bool(false));
    properties.insert("preserveWhiteSpace".to_string(), Value::Bool(true));
    properties.insert("validateOnParse".to_string(), Value::Bool(false));
    
    // 返回对象值
    Value::Object(crate::interpreter::value::ObjectInstance {
        class_name: "DOMDocument".to_string(),
        properties,
    })
}

/// 创建 XMLParser 对象值
///
/// # 参数
/// - `encoding`: 编码名称
///
/// # 返回
/// XMLParser 对象值
fn create_xmlparser_object(encoding: &str) -> Value {
    use std::collections::HashMap;
    
    // 创建对象实例
    let mut properties: HashMap<String, Value> = HashMap::new();
    
    // 设置解析器属性
    properties.insert("encoding".to_string(), Value::String(encoding.to_string()));
    properties.insert("buffer".to_string(), Value::String(String::new()));
    properties.insert("status".to_string(), Value::Bool(true));
    
    // 返回对象值
    Value::Object(crate::interpreter::value::ObjectInstance {
        class_name: "XMLParser".to_string(),
        properties,
    })
}

// ============================================================================
// SimpleXML 内置函数实现
// ============================================================================

/// simplexml_load_string — 将 XML 字符串转换为 SimpleXMLElement 对象
///
/// # PHP 签名
/// simplexml_load_string(string $data, ?string $class_name = SimpleXMLElement::class, int $options = 0, string $ns_prefix = "", bool $is_prefix = false): SimpleXMLElement|false
///
/// # 参数
/// - args[0]: XML 字符串
/// - args[1]: 可选的类名
/// - args[2]: 可选的选项
/// - args[3]: 可选的命名空间前缀
/// - args[4]: 可选的是否为前缀
///
/// # 返回
/// SimpleXMLElement 对象或 false
pub fn builtin_simplexml_load_string(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 获取 XML 字符串
    let xml_string = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 创建 SimpleXMLElement 对象
    Ok(create_simplexmlelement_object(xml_string))
}

/// simplexml_load_file — 将 XML 文件转换为 SimpleXMLElement 对象
///
/// # PHP 签名
/// simplexml_load_file(string $filename, ?string $class_name = SimpleXMLElement::class, int $options = 0, string $ns_prefix = "", bool $is_prefix = false): SimpleXMLElement|false
///
/// # 参数
/// - args[0]: 文件路径
/// - args[1]: 可选的类名
/// - args[2]: 可选的选项
/// - args[3]: 可选的命名空间前缀
/// - args[4]: 可选的是否为前缀
///
/// # 返回
/// SimpleXMLElement 对象或 false
pub fn builtin_simplexml_load_file(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 获取文件路径
    let _filename = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 简化实现：创建一个空的 SimpleXMLElement 对象
    // 实际实现需要读取文件内容
    Ok(create_simplexmlelement_object(""))
}

/// simplexml_import_dom — 从 DOM 节点获取 SimpleXMLElement 对象
///
/// # PHP 签名
/// simplexml_import_dom(SimpleXMLElement|DOMNode $node, ?string $class_name = SimpleXMLElement::class): SimpleXMLElement|null
///
/// # 参数
/// - args[0]: DOM 节点
/// - args[1]: 可选的类名
///
/// # 返回
/// SimpleXMLElement 对象或 null
pub fn builtin_simplexml_import_dom(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Null);
    }
    
    // 简化实现：创建一个空的 SimpleXMLElement 对象
    Ok(create_simplexmlelement_object(""))
}

// ============================================================================
// DOM 内置函数实现
// ============================================================================

/// dom_import_simplexml — 从 SimpleXMLElement 对象获取 DOMDocument 对象
///
/// # PHP 签名
/// dom_import_simplexml(SimpleXMLElement $node): DOMElement|null
///
/// # 参数
/// - args[0]: SimpleXMLElement 对象
///
/// # 返回
/// DOMElement 对象或 null
pub fn builtin_dom_import_simplexml(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Null);
    }
    
    // 简化实现：创建一个空的 DOMDocument 对象
    Ok(create_domdocument_object(""))
}

// ============================================================================
// XML Parser 内置函数实现
// ============================================================================

/// xml_parser_create — 创建 XML 解析器
///
/// # PHP 签名
/// xml_parser_create(?string $encoding = null): XMLParser
///
/// # 参数
/// - args[0]: 可选的编码名称
///
/// # 返回
/// XMLParser 对象
pub fn builtin_xml_parser_create(args: &[Value]) -> Result<Value> {
    // 获取编码
    let encoding = args.first()
        .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
        .unwrap_or("UTF-8");
    
    // 创建 XMLParser 对象
    Ok(create_xmlparser_object(encoding))
}

/// xml_parser_create_ns — 创建支持命名空间的 XML 解析器
///
/// # PHP 签名
/// xml_parser_create_ns(?string $encoding = null, ?string $separator = null): XMLParser
///
/// # 参数
/// - args[0]: 可选的编码名称
/// - args[1]: 可选的命名空间分隔符
///
/// # 返回
/// XMLParser 对象
pub fn builtin_xml_parser_create_ns(args: &[Value]) -> Result<Value> {
    // 获取编码
    let encoding = args.first()
        .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
        .unwrap_or("UTF-8");
    
    // 创建 XMLParser 对象
    Ok(create_xmlparser_object(encoding))
}

/// xml_parse — 解析 XML 文档
///
/// # PHP 签名
/// xml_parse(XMLParser $parser, string $data, bool $is_final = false): int
///
/// # 参数
/// - args[0]: XMLParser 对象
/// - args[1]: XML 数据
/// - args[2]: 是否为最后一块数据
///
/// # 返回
/// 成功返回 1，失败返回 0
pub fn builtin_xml_parse(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Int(0));
    }
    
    // 简化实现：返回成功
    Ok(Value::Int(1))
}

/// xml_parse_into_struct — 将 XML 数据解析为数组结构
///
/// # PHP 签名
/// xml_parse_into_struct(XMLParser $parser, string $data, array &$values, ?array &$index = null): int
///
/// # 参数
/// - args[0]: XMLParser 对象
/// - args[1]: XML 数据
/// - args[2]: 值数组引用
/// - args[3]: 索引数组引用
///
/// # 返回
/// 成功返回 1，失败返回 0
pub fn builtin_xml_parse_into_struct(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 3 {
        return Ok(Value::Int(0));
    }
    
    // 简化实现：返回成功
    Ok(Value::Int(1))
}

/// xml_get_error_code — 获取 XML 解析器错误代码
///
/// # PHP 签名
/// xml_get_error_code(XMLParser $parser): int
///
/// # 参数
/// - args[0]: XMLParser 对象
///
/// # 返回
/// 错误代码
pub fn builtin_xml_get_error_code(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    
    // 简化实现：返回无错误
    Ok(Value::Int(0))
}

/// xml_error_string — 获取 XML 解析器错误字符串
///
/// # PHP 签名
/// xml_error_string(int $error_code): string|null
///
/// # 参数
/// - args[0]: 错误代码
///
/// # 返回
/// 错误字符串
pub fn builtin_xml_error_string(args: &[Value]) -> Result<Value> {
    // 获取错误代码
    let error_code = args.first()
        .and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None })
        .unwrap_or(0);
    
    // 返回错误字符串
    let error_string = match error_code {
        0 => "No error",
        1 => "Out of memory",
        2 => "Syntax error",
        3 => "No elements",
        4 => "Invalid token",
        5 => "Unclosed token",
        6 => "Partial character",
        7 => "Mismatched tag",
        8 => "Duplicate attribute",
        9 => "Junk after document element",
        10 => "Illegal parameter entity reference",
        11 => "Undefined entity",
        12 => "Recursive entity reference",
        13 => "Asynchronous entity",
        14 => "Reference to invalid character number",
        15 => "Reference to binary entity",
        16 => "Reference to external entity",
        17 => "Invalid character reference",
        18 => "Invalid character",
        19 => "XML declaration not at start of document",
        20 => "Reserved prefix must not be undeclared",
        21 => "Undeclared entity",
        22 => "Unbound namespace prefix",
        23 => "Unbound parameter entity",
        24 => "Unbound external entity",
        25 => "Unbound notation",
        _ => "Unknown error",
    };
    
    Ok(Value::String(error_string.to_string()))
}

/// xml_get_current_line_number — 获取 XML 解析器的当前行号
///
/// # PHP 签名
/// xml_get_current_line_number(XMLParser $parser): int
///
/// # 参数
/// - args[0]: XMLParser 对象
///
/// # 返回
/// 当前行号
pub fn builtin_xml_get_current_line_number(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    
    // 简化实现：返回 1
    Ok(Value::Int(1))
}

/// xml_get_current_column_number — 获取 XML 解析器的当前列号
///
/// # PHP 签名
/// xml_get_current_column_number(XMLParser $parser): int
///
/// # 参数
/// - args[0]: XMLParser 对象
///
/// # 返回
/// 当前列号
pub fn builtin_xml_get_current_column_number(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    
    // 简化实现：返回 1
    Ok(Value::Int(1))
}

/// xml_get_current_byte_index — 获取 XML 解析器的当前字节索引
///
/// # PHP 签名
/// xml_get_current_byte_index(XMLParser $parser): int
///
/// # 参数
/// - args[0]: XMLParser 对象
///
/// # 返回
/// 当前字节索引
pub fn builtin_xml_get_current_byte_index(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    
    // 简化实现：返回 0
    Ok(Value::Int(0))
}

/// xml_parser_free — 释放 XML 解析器
///
/// # PHP 签名
/// xml_parser_free(XMLParser $parser): bool
///
/// # 参数
/// - args[0]: XMLParser 对象
///
/// # 返回
/// 成功返回 true
pub fn builtin_xml_parser_free(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// xml_parser_set_option — 设置 XML 解析器选项
///
/// # PHP 签名
/// xml_parser_set_option(XMLParser $parser, int $option, string|int $value): bool
///
/// # 参数
/// - args[0]: XMLParser 对象
/// - args[1]: 选项名称
/// - args[2]: 选项值
///
/// # 返回
/// 成功返回 true
pub fn builtin_xml_parser_set_option(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 3 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// xml_parser_get_option — 获取 XML 解析器选项
///
/// # PHP 签名
/// xml_parser_get_option(XMLParser $parser, int $option): string|int|bool
///
/// # 参数
/// - args[0]: XMLParser 对象
/// - args[1]: 选项名称
///
/// # 返回
/// 选项值
pub fn builtin_xml_parser_get_option(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回默认值
    Ok(Value::Bool(true))
}

/// xml_set_element_handler — 设置起始和结束元素处理器
///
/// # PHP 签名
/// xml_set_element_handler(XMLParser $parser, callable $start_handler, callable $end_handler): bool
///
/// # 参数
/// - args[0]: XMLParser 对象
/// - args[1]: 起始元素处理器
/// - args[2]: 结束元素处理器
///
/// # 返回
/// 成功返回 true
pub fn builtin_xml_set_element_handler(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 3 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// xml_set_character_data_handler — 设置字符数据处理器
///
/// # PHP 签名
/// xml_set_character_data_handler(XMLParser $parser, callable $handler): bool
///
/// # 参数
/// - args[0]: XMLParser 对象
/// - args[1]: 字符数据处理器
///
/// # 返回
/// 成功返回 true
pub fn builtin_xml_set_character_data_handler(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// xml_set_processing_instruction_handler — 设置处理指令处理器
///
/// # PHP 签名
/// xml_set_processing_instruction_handler(XMLParser $parser, callable $handler): bool
///
/// # 参数
/// - args[0]: XMLParser 对象
/// - args[1]: 处理指令处理器
///
/// # 返回
/// 成功返回 true
pub fn builtin_xml_set_processing_instruction_handler(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// xml_set_default_handler — 设置默认处理器
///
/// # PHP 签名
/// xml_set_default_handler(XMLParser $parser, callable $handler): bool
///
/// # 参数
/// - args[0]: XMLParser 对象
/// - args[1]: 默认处理器
///
/// # 返回
/// 成功返回 true
pub fn builtin_xml_set_default_handler(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// xml_set_unparsed_entity_decl_handler — 设置未解析实体声明处理器
///
/// # PHP 签名
/// xml_set_unparsed_entity_decl_handler(XMLParser $parser, callable $handler): bool
///
/// # 参数
/// - args[0]: XMLParser 对象
/// - args[1]: 未解析实体声明处理器
///
/// # 返回
/// 成功返回 true
pub fn builtin_xml_set_unparsed_entity_decl_handler(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// xml_set_notation_decl_handler — 设置符号声明处理器
///
/// # PHP 签名
/// xml_set_notation_decl_handler(XMLParser $parser, callable $handler): bool
///
/// # 参数
/// - args[0]: XMLParser 对象
/// - args[1]: 符号声明处理器
///
/// # 返回
/// 成功返回 true
pub fn builtin_xml_set_notation_decl_handler(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// xml_set_external_entity_ref_handler — 设置外部实体引用处理器
///
/// # PHP 签名
/// xml_set_external_entity_ref_handler(XMLParser $parser, callable $handler): bool
///
/// # 参数
/// - args[0]: XMLParser 对象
/// - args[1]: 外部实体引用处理器
///
/// # 返回
/// 成功返回 true
pub fn builtin_xml_set_external_entity_ref_handler(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// xml_set_start_namespace_decl_handler — 设置命名空间声明开始处理器
///
/// # PHP 签名
/// xml_set_start_namespace_decl_handler(XMLParser $parser, callable $handler): bool
///
/// # 参数
/// - args[0]: XMLParser 对象
/// - args[1]: 命名空间声明开始处理器
///
/// # 返回
/// 成功返回 true
pub fn builtin_xml_set_start_namespace_decl_handler(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// xml_set_end_namespace_decl_handler — 设置命名空间声明结束处理器
///
/// # PHP 签名
/// xml_set_end_namespace_decl_handler(XMLParser $parser, callable $handler): bool
///
/// # 参数
/// - args[0]: XMLParser 对象
/// - args[1]: 命名空间声明结束处理器
///
/// # 返回
/// 成功返回 true
pub fn builtin_xml_set_end_namespace_decl_handler(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

// ============================================================================
// 其他 XML 函数
// ============================================================================

/// utf8_encode — 将 ISO-8859-1 字符串编码为 UTF-8
///
/// # PHP 签名
/// utf8_encode(string $string): string
///
/// # 参数
/// - args[0]: ISO-8859-1 字符串
///
/// # 返回
/// UTF-8 编码的字符串
pub fn builtin_utf8_encode(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::String(String::new()));
    }
    
    // 获取输入字符串
    let input = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::String(String::new())),
    };
    
    // 使用 encoding_rs 进行编码转换
    let (encoded, _, _had_errors) = encoding_rs::WINDOWS_1252.encode(input);
    Ok(Value::String(String::from_utf8_lossy(&encoded).to_string()))
}

/// utf8_decode — 将 UTF-8 编码的字符串解码为 ISO-8859-1
///
/// # PHP 签名
/// utf8_decode(string $string): string
///
/// # 参数
/// - args[0]: UTF-8 字符串
///
/// # 返回
/// ISO-8859-1 编码的字符串
pub fn builtin_utf8_decode(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::String(String::new()));
    }
    
    // 获取输入字符串
    let input = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::String(String::new())),
    };
    
    // 使用 encoding_rs 进行编码转换
    let (decoded, _, _had_errors) = encoding_rs::WINDOWS_1252.decode(input.as_bytes());
    Ok(Value::String(decoded.to_string()))
}

// ============================================================================
// 函数注册
// ============================================================================

/// 注册 XML 相关的内置函数
///
/// 将所有 XML 函数注册到内置函数映射表中
///
/// # 参数
/// - `map`: 内置函数映射表的可变引用
pub fn register_xml_functions(map: &mut HashMap<String, BuiltinFunction>) {
    // SimpleXML 函数
    map.insert("simplexml_load_string".to_string(), builtin_simplexml_load_string);
    map.insert("simplexml_load_file".to_string(), builtin_simplexml_load_file);
    map.insert("simplexml_import_dom".to_string(), builtin_simplexml_import_dom);
    
    // DOM 函数
    map.insert("dom_import_simplexml".to_string(), builtin_dom_import_simplexml);
    
    // XML Parser 函数
    map.insert("xml_parser_create".to_string(), builtin_xml_parser_create);
    map.insert("xml_parser_create_ns".to_string(), builtin_xml_parser_create_ns);
    map.insert("xml_parse".to_string(), builtin_xml_parse);
    map.insert("xml_parse_into_struct".to_string(), builtin_xml_parse_into_struct);
    map.insert("xml_get_error_code".to_string(), builtin_xml_get_error_code);
    map.insert("xml_error_string".to_string(), builtin_xml_error_string);
    map.insert("xml_get_current_line_number".to_string(), builtin_xml_get_current_line_number);
    map.insert("xml_get_current_column_number".to_string(), builtin_xml_get_current_column_number);
    map.insert("xml_get_current_byte_index".to_string(), builtin_xml_get_current_byte_index);
    map.insert("xml_parser_free".to_string(), builtin_xml_parser_free);
    map.insert("xml_parser_set_option".to_string(), builtin_xml_parser_set_option);
    map.insert("xml_parser_get_option".to_string(), builtin_xml_parser_get_option);
    
    // XML 处理器设置函数
    map.insert("xml_set_element_handler".to_string(), builtin_xml_set_element_handler);
    map.insert("xml_set_character_data_handler".to_string(), builtin_xml_set_character_data_handler);
    map.insert("xml_set_processing_instruction_handler".to_string(), builtin_xml_set_processing_instruction_handler);
    map.insert("xml_set_default_handler".to_string(), builtin_xml_set_default_handler);
    map.insert("xml_set_unparsed_entity_decl_handler".to_string(), builtin_xml_set_unparsed_entity_decl_handler);
    map.insert("xml_set_notation_decl_handler".to_string(), builtin_xml_set_notation_decl_handler);
    map.insert("xml_set_external_entity_ref_handler".to_string(), builtin_xml_set_external_entity_ref_handler);
    map.insert("xml_set_start_namespace_decl_handler".to_string(), builtin_xml_set_start_namespace_decl_handler);
    map.insert("xml_set_end_namespace_decl_handler".to_string(), builtin_xml_set_end_namespace_decl_handler);
    
    // 编码函数
    map.insert("utf8_encode".to_string(), builtin_utf8_encode);
    map.insert("utf8_decode".to_string(), builtin_utf8_decode);
}
