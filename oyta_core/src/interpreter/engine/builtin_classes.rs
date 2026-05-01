//! 内置类注册模块
//!
//! 本模块实现 PHP 内置类的注册机制
//! 包括：DateTime, DateTimeImmutable, DateTimeZone, DateInterval, DatePeriod,
//! DOMDocument, SimpleXMLElement, GdImage, Phar, PharData, Model 等
//!
//! # 模块结构
//! - `builtin_class/types`: 类型定义（BuiltinClassDefinition, BuiltinClassRegistry）
//! - `builtin_class/datetime`: DateTime 相关类
//! - `builtin_class/dom`: DOM 相关类
//! - `builtin_class/phar`: Phar 相关类
//! - `builtin_class/reflection`: Reflection 相关类
//! - `builtin_class/request`: Request 类
//! - `builtin_class/model`: Model 基类

// 从 builtin_class 子模块导入类型和方法
pub use super::builtin_class::{
    BuiltinClassDefinition, BuiltinClassRegistry, BuiltinMethod,
    // DateTime 方法
    datetime_format, datetime_modify, datetime_get_timestamp, datetime_set_timestamp,
    datetime_add, datetime_sub, datetime_diff,
    datetime_immutable_format, datetime_immutable_get_timestamp, datetime_immutable_set_timestamp,
    datetimezone_get_name, datetimezone_get_offset,
    dateinterval_format,
    // DOM 方法
    domdocument_save_xml, domdocument_load_xml, domdocument_load_html, domdocument_save_html,
    domdocument_create_element, domdocument_append_child,
    simplexmlelement_as_xml, simplexmlelement_xpath, simplexmlelement_children, simplexmlelement_attributes,
    // Phar 方法
    phar_get_signature, phar_get_metadata, phar_set_metadata, phar_add_file, phar_add_from_string,
    phardata_get_signature, phardata_get_metadata,
    gdimage_get_width, gdimage_get_height, gdimage_crop, gdimage_resize,
    // Reflection 方法
    reflection_class_get_name, reflection_class_get_methods, reflection_class_get_properties,
    reflection_class_has_method, reflection_class_has_property, reflection_class_new_instance,
    reflection_method_get_name, reflection_method_invoke,
    reflection_property_get_name, reflection_property_get_value, reflection_property_set_value,
    reflection_parameter_get_name, reflection_parameter_get_type, reflection_parameter_is_optional,
    reflection_function_get_name, reflection_function_invoke,
    // Request 方法
    request_get, request_post, request_param, request_input,
    request_method, request_is_method, request_ip, request_header, request_all,
    request_is_ajax, request_is_json, request_is_mobile, request_is_cli, request_is_cgi,
    // Model 方法
    model_find, model_find_or_fail, model_first, model_all, model_create, model_count, model_where, model_paginate,
    model_instance_save, model_instance_delete, model_instance_to_array, model_instance_to_json,
};

use std::sync::OnceLock;

use crate::interpreter::value::Value;

/// 全局内置类注册表
///
/// 使用懒加载初始化
pub fn get_builtin_class_registry() -> &'static BuiltinClassRegistry {
    static REGISTRY: OnceLock<BuiltinClassRegistry> = OnceLock::new();
    REGISTRY.get_or_init(create_builtin_class_registry)
}

/// 创建并填充内置类注册表
///
/// # 返回
/// 包含所有内置类的注册表
pub fn create_builtin_class_registry() -> BuiltinClassRegistry {
    let mut registry = BuiltinClassRegistry::new();
    
    // 注册 DateTime 类
    let datetime_class = BuiltinClassDefinition::new("DateTime")
        .add_default_property("timestamp", Value::Int(0))
        .add_default_property("ustime", Value::Int(0))
        .add_default_property("timezone", Value::String("UTC".to_string()))
        .add_default_property("year", Value::Int(0))
        .add_default_property("month", Value::Int(0))
        .add_default_property("day", Value::Int(0))
        .add_default_property("hour", Value::Int(0))
        .add_default_property("minute", Value::Int(0))
        .add_default_property("second", Value::Int(0))
        .add_method("format", datetime_format)
        .add_method("modify", datetime_modify)
        .add_method("getTimestamp", datetime_get_timestamp)
        .add_method("setTimestamp", datetime_set_timestamp)
        .add_method("add", datetime_add)
        .add_method("sub", datetime_sub)
        .add_method("diff", datetime_diff)
        .add_constant("ATOM", Value::String("Y-m-d\\TH:i:sP".to_string()))
        .add_constant("COOKIE", Value::String("l, d-M-Y H:i:s T".to_string()))
        .add_constant("ISO8601", Value::String("Y-m-d\\TH:i:sO".to_string()))
        .add_constant("RFC822", Value::String("D, d M y H:i:s O".to_string()))
        .add_constant("RFC2822", Value::String("D, d M Y H:i:s O".to_string()))
        .add_constant("RFC3339", Value::String("Y-m-d\\TH:i:sP".to_string()))
        .add_constant("RSS", Value::String("D, d M Y H:i:s O".to_string()))
        .add_constant("W3C", Value::String("Y-m-d\\TH:i:sP".to_string()));
    registry.register(datetime_class);
    
    // 注册 DateTimeImmutable 类
    let datetime_immutable_class = BuiltinClassDefinition::new("DateTimeImmutable")
        .set_parent("DateTime")
        .add_default_property("timestamp", Value::Int(0))
        .add_default_property("ustime", Value::Int(0))
        .add_default_property("timezone", Value::String("UTC".to_string()))
        .add_method("format", datetime_immutable_format)
        .add_method("getTimestamp", datetime_immutable_get_timestamp)
        .add_method("setTimestamp", datetime_immutable_set_timestamp);
    registry.register(datetime_immutable_class);
    
    // 注册 DateTimeZone 类
    let datetimezone_class = BuiltinClassDefinition::new("DateTimeZone")
        .add_default_property("name", Value::String("UTC".to_string()))
        .add_method("getName", datetimezone_get_name)
        .add_method("getOffset", datetimezone_get_offset)
        .add_constant("AFRICA", Value::Int(1))
        .add_constant("AMERICA", Value::Int(2))
        .add_constant("ANTARCTICA", Value::Int(4))
        .add_constant("ARCTIC", Value::Int(8))
        .add_constant("ASIA", Value::Int(16))
        .add_constant("ATLANTIC", Value::Int(32))
        .add_constant("AUSTRALIA", Value::Int(64))
        .add_constant("EUROPE", Value::Int(128))
        .add_constant("INDIAN", Value::Int(256))
        .add_constant("PACIFIC", Value::Int(512))
        .add_constant("UTC", Value::Int(1024))
        .add_constant("ALL", Value::Int(2047))
        .add_constant("ALL_WITH_BC", Value::Int(4095));
    registry.register(datetimezone_class);
    
    // 注册 DateInterval 类
    let dateinterval_class = BuiltinClassDefinition::new("DateInterval")
        .add_default_property("y", Value::Int(0))
        .add_default_property("m", Value::Int(0))
        .add_default_property("d", Value::Int(0))
        .add_default_property("h", Value::Int(0))
        .add_default_property("i", Value::Int(0))
        .add_default_property("s", Value::Int(0))
        .add_default_property("invert", Value::Bool(false))
        .add_default_property("days", Value::Bool(false))
        .add_method("format", dateinterval_format);
    registry.register(dateinterval_class);
    
    // 注册 DatePeriod 类
    let dateperiod_class = BuiltinClassDefinition::new("DatePeriod")
        .add_default_property("start", Value::Null)
        .add_default_property("end", Value::Null)
        .add_default_property("interval", Value::Null)
        .add_default_property("include_start_date", Value::Bool(true))
        .add_default_property("recurrences", Value::Int(0));
    registry.register(dateperiod_class);
    
    // 注册 DOMDocument 类
    let domdocument_class = BuiltinClassDefinition::new("DOMDocument")
        .add_default_property("version", Value::String("1.0".to_string()))
        .add_default_property("encoding", Value::String("UTF-8".to_string()))
        .add_default_property("documentElement", Value::Null)
        .add_default_property("formatOutput", Value::Bool(false))
        .add_default_property("preserveWhiteSpace", Value::Bool(true))
        .add_default_property("validateOnParse", Value::Bool(false))
        .add_method("saveXML", domdocument_save_xml)
        .add_method("loadXML", domdocument_load_xml)
        .add_method("loadHTML", domdocument_load_html)
        .add_method("saveHTML", domdocument_save_html)
        .add_method("createElement", domdocument_create_element)
        .add_method("appendChild", domdocument_append_child);
    registry.register(domdocument_class);
    
    // 注册 SimpleXMLElement 类
    let simplexmlelement_class = BuiltinClassDefinition::new("SimpleXMLElement")
        .add_default_property("xml", Value::String(String::new()))
        .add_default_property("nodeName", Value::String(String::new()))
        .add_default_property("nodeValue", Value::String(String::new()))
        .add_default_property("nodeType", Value::Int(1))
        .add_default_property("attributes", Value::IndexedArray(vec![]))
        .add_default_property("children", Value::IndexedArray(vec![]))
        .add_method("asXML", simplexmlelement_as_xml)
        .add_method("xpath", simplexmlelement_xpath)
        .add_method("children", simplexmlelement_children)
        .add_method("attributes", simplexmlelement_attributes);
    registry.register(simplexmlelement_class);
    
    // 注册 GdImage 类
    let gdimage_class = BuiltinClassDefinition::new("GdImage")
        .add_default_property("width", Value::Int(0))
        .add_default_property("height", Value::Int(0))
        .add_default_property("trueColor", Value::Bool(true))
        .add_default_property("pixels", Value::String(String::new()))
        .add_default_property("colors", Value::IndexedArray(vec![]))
        .add_default_property("transparent", Value::Int(-1))
        .add_method("getWidth", gdimage_get_width)
        .add_method("getHeight", gdimage_get_height)
        .add_method("crop", gdimage_crop)
        .add_method("resize", gdimage_resize);
    registry.register(gdimage_class);
    
    // 注册 Phar 类
    let phar_class = BuiltinClassDefinition::new("Phar")
        .add_default_property("path", Value::String(String::new()))
        .add_default_property("alias", Value::String(String::new()))
        .add_default_property("version", Value::String("1.0.0".to_string()))
        .add_default_property("signature", Value::Int(0))
        .add_default_property("hasSignature", Value::Bool(false))
        .add_default_property("isCompressed", Value::Bool(false))
        .add_default_property("isWritable", Value::Bool(false))
        .add_default_property("entries", Value::IndexedArray(vec![]))
        .add_method("getSignature", phar_get_signature)
        .add_method("getMetadata", phar_get_metadata)
        .add_method("setMetadata", phar_set_metadata)
        .add_method("addFile", phar_add_file)
        .add_method("addFromString", phar_add_from_string)
        .add_constant("MD5", Value::Int(1))
        .add_constant("SHA1", Value::Int(2))
        .add_constant("SHA256", Value::Int(3))
        .add_constant("SHA512", Value::Int(4))
        .add_constant("OPENSSL", Value::Int(16));
    registry.register(phar_class);
    
    // 注册 PharData 类
    let phardata_class = BuiltinClassDefinition::new("PharData")
        .add_default_property("path", Value::String(String::new()))
        .add_default_property("alias", Value::String(String::new()))
        .add_default_property("entries", Value::IndexedArray(vec![]))
        .add_method("getSignature", phardata_get_signature)
        .add_method("getMetadata", phardata_get_metadata);
    registry.register(phardata_class);
    
    // 注册 PharException 类
    let pharexception_class = BuiltinClassDefinition::new("PharException")
        .set_parent("Exception")
        .add_default_property("message", Value::String(String::new()))
        .add_default_property("code", Value::Int(0))
        .add_default_property("file", Value::String(String::new()))
        .add_default_property("line", Value::Int(0));
    registry.register(pharexception_class);
    
    // 注册 XMLParser 类
    let xmlparser_class = BuiltinClassDefinition::new("XMLParser")
        .add_default_property("encoding", Value::String("UTF-8".to_string()))
        .add_default_property("buffer", Value::String(String::new()))
        .add_default_property("status", Value::Bool(true));
    registry.register(xmlparser_class);
    
    // 注册 ReflectionClass 类
    let reflectionclass_class = BuiltinClassDefinition::new("ReflectionClass")
        .add_default_property("name", Value::String(String::new()))
        .add_default_property("isInternal", Value::Bool(false))
        .add_default_property("isUserDefined", Value::Bool(true))
        .add_default_property("isInstantiable", Value::Bool(true))
        .add_default_property("isCloneable", Value::Bool(true))
        .add_default_property("isFinal", Value::Bool(false))
        .add_default_property("isAbstract", Value::Bool(false))
        .add_default_property("isInterface", Value::Bool(false))
        .add_default_property("isTrait", Value::Bool(false))
        .add_method("getName", reflection_class_get_name)
        .add_method("getMethods", reflection_class_get_methods)
        .add_method("getProperties", reflection_class_get_properties)
        .add_method("hasMethod", reflection_class_has_method)
        .add_method("hasProperty", reflection_class_has_property)
        .add_method("newInstance", reflection_class_new_instance);
    registry.register(reflectionclass_class);
    
    // 注册 ReflectionMethod 类
    let reflectionmethod_class = BuiltinClassDefinition::new("ReflectionMethod")
        .add_default_property("name", Value::String(String::new()))
        .add_default_property("class", Value::String(String::new()))
        .add_default_property("isPublic", Value::Bool(true))
        .add_default_property("isPrivate", Value::Bool(false))
        .add_default_property("isProtected", Value::Bool(false))
        .add_default_property("isStatic", Value::Bool(false))
        .add_default_property("isFinal", Value::Bool(false))
        .add_default_property("isAbstract", Value::Bool(false))
        .add_method("getName", reflection_method_get_name)
        .add_method("invoke", reflection_method_invoke);
    registry.register(reflectionmethod_class);
    
    // 注册 ReflectionProperty 类
    let reflectionproperty_class = BuiltinClassDefinition::new("ReflectionProperty")
        .add_default_property("name", Value::String(String::new()))
        .add_default_property("class", Value::String(String::new()))
        .add_default_property("isPublic", Value::Bool(true))
        .add_default_property("isPrivate", Value::Bool(false))
        .add_default_property("isProtected", Value::Bool(false))
        .add_default_property("isStatic", Value::Bool(false))
        .add_default_property("isDefault", Value::Bool(true))
        .add_method("getName", reflection_property_get_name)
        .add_method("getValue", reflection_property_get_value)
        .add_method("setValue", reflection_property_set_value);
    registry.register(reflectionproperty_class);
    
    // 注册 ReflectionParameter 类
    let reflectionparameter_class = BuiltinClassDefinition::new("ReflectionParameter")
        .add_default_property("name", Value::String(String::new()))
        .add_default_property("position", Value::Int(0))
        .add_default_property("isOptional", Value::Bool(false))
        .add_default_property("isPassedByReference", Value::Bool(false))
        .add_default_property("isVariadic", Value::Bool(false))
        .add_default_property("defaultValue", Value::Null)
        .add_method("getName", reflection_parameter_get_name)
        .add_method("getType", reflection_parameter_get_type)
        .add_method("isOptional", reflection_parameter_is_optional);
    registry.register(reflectionparameter_class);
    
    // 注册 ReflectionFunction 类
    let reflectionfunction_class = BuiltinClassDefinition::new("ReflectionFunction")
        .add_default_property("name", Value::String(String::new()))
        .add_default_property("isClosure", Value::Bool(false))
        .add_default_property("isDeprecated", Value::Bool(false))
        .add_default_property("isGenerator", Value::Bool(false))
        .add_default_property("isInternal", Value::Bool(false))
        .add_default_property("isUserDefined", Value::Bool(true))
        .add_default_property("isVariadic", Value::Bool(false))
        .add_method("getName", reflection_function_get_name)
        .add_method("invoke", reflection_function_invoke);
    registry.register(reflectionfunction_class);
    
    // 注册 Request 类
    let request_class = BuiltinClassDefinition::new("Request")
        .add_default_property("method", Value::String("GET".to_string()))
        .add_default_property("path", Value::String(String::new()))
        .add_default_property("host", Value::String(String::new()))
        .add_default_property("ip", Value::String("127.0.0.1".to_string()))
        .add_default_property("headers", Value::AssociativeArray(vec![]))
        .add_default_property("params", Value::AssociativeArray(vec![]))
        .add_method("get", request_get)
        .add_method("post", request_post)
        .add_method("param", request_param)
        .add_method("input", request_input)
        .add_method("method", request_method)
        .add_method("isMethod", request_is_method)
        .add_method("ip", request_ip)
        .add_method("header", request_header)
        .add_method("all", request_all)
        .add_method("isAjax", request_is_ajax)
        .add_method("isJson", request_is_json)
        .add_method("isMobile", request_is_mobile)
        .add_method("isCli", request_is_cli)
        .add_method("isCgi", request_is_cgi);
    registry.register(request_class);
    
    // 注册 Model 基类 - ThinkPHP 8.0 ORM 模型基类
    // PHP 代码可以通过 class User extends Model 来继承此基类
    let model_class = BuiltinClassDefinition::new("Model")
        // 默认属性
        .add_default_property("table", Value::String(String::new()))
        .add_default_property("pk", Value::String("id".to_string()))
        .add_default_property("connection", Value::String(String::new()))
        .add_default_property("autoWriteTimestamp", Value::Bool(true))
        .add_default_property("createTime", Value::String("create_time".to_string()))
        .add_default_property("updateTime", Value::String("update_time".to_string()))
        .add_default_property("deleteTime", Value::String("delete_time".to_string()))
        .add_default_property("defaultSoftDelete", Value::Null)
        .add_default_property("exists", Value::Bool(false))
        .add_default_property("data", Value::AssociativeArray(vec![]))
        .add_default_property("origin", Value::AssociativeArray(vec![]))
        .add_default_property("relation", Value::AssociativeArray(vec![]))
        // 静态方法 - 查询
        .add_static_method("find", model_find)
        .add_static_method("findOrFail", model_find_or_fail)
        .add_static_method("first", model_first)
        .add_static_method("all", model_all)
        .add_static_method("create", model_create)
        .add_static_method("count", model_count)
        .add_static_method("where", model_where)
        .add_static_method("paginate", model_paginate)
        // 实例方法
        .add_method("save", model_instance_save)
        .add_method("delete", model_instance_delete)
        .add_method("toArray", model_instance_to_array)
        .add_method("toJson", model_instance_to_json);
    registry.register(model_class);
    
    registry
}
