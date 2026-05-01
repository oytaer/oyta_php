//! 内置类模块
//!
//! 本模块实现 PHP 内置类的注册机制
//! 包括：DateTime, DateTimeImmutable, DateTimeZone, DateInterval, DatePeriod,
//! DOMDocument, SimpleXMLElement, GdImage, Phar, PharData, Model, XMLReader, XMLWriter, XPath 等

// 子模块声明
pub mod types;
pub mod datetime;
pub mod dom;
pub mod gdimage;
pub mod phar;
pub mod reflection;
pub mod request;
pub mod model;
pub mod xml_reader_writer;
pub mod xpath;

// 重新导出主要类型
pub use types::{BuiltinClassDefinition, BuiltinClassRegistry, BuiltinMethod};

// ============================================================================
// DateTime 类方法导出
// ============================================================================
pub use datetime::{
    // DateTime 方法
    datetime_format, datetime_modify, datetime_get_timestamp, datetime_set_timestamp,
    datetime_add, datetime_sub, datetime_diff,
    // DateTimeImmutable 方法
    datetime_immutable_format, datetime_immutable_get_timestamp, datetime_immutable_set_timestamp,
    // DateTimeZone 方法
    datetimezone_get_name, datetimezone_get_offset,
    // DateInterval 方法
    dateinterval_format,
};

// ============================================================================
// DOM 类方法导出
// ============================================================================
pub use dom::{
    // DOMDocument 方法
    domdocument_save_xml, domdocument_load_xml, domdocument_load_html, domdocument_save_html,
    domdocument_create_element, domdocument_append_child,
    // SimpleXMLElement 方法
    simplexmlelement_as_xml, simplexmlelement_xpath, simplexmlelement_children, simplexmlelement_attributes,
};

// ============================================================================
// Phar 类方法导出
// ============================================================================
pub use phar::{
    // Phar 方法
    phar_get_signature, phar_get_metadata, phar_set_metadata, phar_add_file, phar_add_from_string,
    // PharData 方法
    phardata_get_signature, phardata_get_metadata,
};

// ============================================================================
// Reflection 类方法导出
// ============================================================================
pub use reflection::{
    // ReflectionClass 方法
    reflection_class_get_name, reflection_class_get_methods, reflection_class_get_properties,
    reflection_class_has_method, reflection_class_has_property, reflection_class_new_instance,
    // ReflectionMethod 方法
    reflection_method_get_name, reflection_method_invoke,
    // ReflectionProperty 方法
    reflection_property_get_name, reflection_property_get_value, reflection_property_set_value,
    // ReflectionParameter 方法
    reflection_parameter_get_name, reflection_parameter_get_type, reflection_parameter_is_optional,
    // ReflectionFunction 方法
    reflection_function_get_name, reflection_function_invoke,
};

// ============================================================================
// Request 类方法导出
// ============================================================================
pub use request::{
    request_get, request_post, request_param, request_input,
    request_method, request_is_method, request_ip, request_header, request_all,
    request_is_ajax, request_is_json, request_is_mobile, request_is_cli, request_is_cgi,
};

// ============================================================================
// Model 类方法导出
// ============================================================================
pub use model::{
    // 静态方法
    model_find, model_find_or_fail, model_first, model_all, model_create, model_count, model_where, model_paginate,
    // 实例方法
    model_instance_save, model_instance_delete, model_instance_to_array, model_instance_to_json,
};

// ============================================================================
// XMLReader/XMLWriter 类方法导出
// ============================================================================
pub use xml_reader_writer::{
    // XMLReader 方法
    xmlreader_new, xmlreader_open, xmlreader_xml, xmlreader_read, xmlreader_close,
    xmlreader_get_name, xmlreader_get_node_type, xmlreader_get_attribute,
    // XMLWriter 方法
    xmlwriter_new, xmlwriter_open_memory, xmlwriter_open_uri,
    xmlwriter_start_document, xmlwriter_end_document,
    xmlwriter_start_element, xmlwriter_end_element, xmlwriter_write_element,
    xmlwriter_text, xmlwriter_write_attribute, xmlwriter_output_memory,
    // 类定义获取函数
    get_xmlreader_class_definition, get_xmlwriter_class_definition,
};

// ============================================================================
// XPath 类方法导出
// ============================================================================
pub use xpath::{
    // XPath 方法
    xpath_new, xpath_evaluate, xpath_select_nodes, xpath_select_single_node,
    xpath_set_variable, xpath_set_namespace, xpath_get_result,
    xpath_evaluate_string, xpath_evaluate_boolean, xpath_evaluate_number,
    // 类定义获取函数
    get_xpath_class_definition,
};

// ============================================================================
// GdImage 类方法导出
// ============================================================================
pub use gdimage::{
    // GdImage 方法
    gdimage_new, gdimage_load, gdimage_save, gdimage_resize, gdimage_crop,
    gdimage_rotate, gdimage_flip, gdimage_filter, gdimage_text, gdimage_line,
    gdimage_rectangle, gdimage_ellipse, gdimage_get_width, gdimage_get_height,
    gdimage_get_type, gdimage_destroy, gdimage_output, gdimage_to_base64,
    // 类定义获取函数
    get_gdimage_class_definition,
};
