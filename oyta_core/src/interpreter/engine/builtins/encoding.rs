//! 编码/解码函数模块
//!
//! 包含 json_encode、json_decode、base64_encode、base64_decode 等函数

use std::collections::HashMap;

use anyhow::Result;

use crate::interpreter::value::Value;

use super::types::BuiltinFunction;

/// 注册编码/解码函数到映射表
///
/// # 参数
/// - `map`: 内置函数映射表的可变引用
pub fn register_encoding_functions(map: &mut HashMap<String, BuiltinFunction>) {
    // JSON 编码
    map.insert("json_encode".to_string(), builtin_json_encode);
    // JSON 解码
    map.insert("json_decode".to_string(), builtin_json_decode);
    // Base64 编码
    map.insert("base64_encode".to_string(), builtin_base64_encode);
    // Base64 解码
    map.insert("base64_decode".to_string(), builtin_base64_decode);
    // URL 编码
    map.insert("urlencode".to_string(), builtin_urlencode);
    // URL 解码
    map.insert("urldecode".to_string(), builtin_urldecode);
    // HTML 实体编码
    map.insert("htmlentities".to_string(), builtin_htmlentities);
    // HTML 实体解码
    map.insert("html_entity_decode".to_string(), builtin_html_entity_decode);
    // 序列化
    map.insert("serialize".to_string(), builtin_serialize);
    // 反序列化
    map.insert("unserialize".to_string(), builtin_unserialize);
}

/// json_encode — 对变量进行 JSON 编码
pub fn builtin_json_encode(args: &[Value]) -> Result<Value> {
    let value = args.first().unwrap_or(&Value::Null);

    // 转换为 JSON 字符串
    let json_str = value.to_json_string();

    Ok(Value::String(json_str))
}

/// json_decode — 对 JSON 格式的字符串进行解码
pub fn builtin_json_decode(args: &[Value]) -> Result<Value> {
    let json_str = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let assoc = args.get(1).map(|v| v.is_truthy()).unwrap_or(false);

    // 解析 JSON 字符串
    match serde_json::from_str::<serde_json::Value>(&json_str) {
        Ok(json_value) => {
            let value = json_value_to_value(&json_value, assoc);
            Ok(value)
        }
        Err(e) => {
            tracing::debug!("JSON 解析失败: {}", e);
            Ok(Value::Null)
        }
    }
}

/// 将 serde_json::Value 转换为解释器的 Value
///
/// # 参数
/// - `json`: JSON 值引用
/// - `assoc`: 是否返回关联数组
///
/// # 返回
/// 解释器的 Value
fn json_value_to_value(json: &serde_json::Value, assoc: bool) -> Value {
    match json {
        // JSON null
        serde_json::Value::Null => Value::Null,
        // JSON 布尔值
        serde_json::Value::Bool(b) => Value::Bool(*b),
        // JSON 数字
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::String(n.to_string())
            }
        }
        // JSON 字符串
        serde_json::Value::String(s) => Value::String(s.clone()),
        // JSON 数组
        serde_json::Value::Array(arr) => {
            let values: Vec<Value> = arr.iter().map(|v| json_value_to_value(v, assoc)).collect();
            Value::IndexedArray(values)
        }
        // JSON 对象
        serde_json::Value::Object(obj) => {
            if assoc {
                // 返回关联数组
                let pairs: Vec<(String, Value)> = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), json_value_to_value(v, assoc)))
                    .collect();
                Value::AssociativeArray(pairs)
            } else {
                // 返回对象
                let properties: HashMap<String, Value> = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), json_value_to_value(v, assoc)))
                    .collect();
                Value::Object(crate::interpreter::value::ObjectInstance {
                    class_name: "stdClass".to_string(),
                    properties,
                })
            }
        }
    }
}

/// base64_encode — 使用 MIME base64 对数据进行编码
pub fn builtin_base64_encode(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();

    // Base64 编码
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, s.as_bytes());

    Ok(Value::String(encoded))
}

/// base64_decode — 对使用 MIME base64 编码的数据进行解码
pub fn builtin_base64_decode(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let strict = args.get(1).map(|v| v.is_truthy()).unwrap_or(false);

    // Base64 解码
    match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, s) {
        Ok(bytes) => {
            // 尝试转换为 UTF-8 字符串
            match String::from_utf8(bytes) {
                Ok(decoded) => Ok(Value::String(decoded)),
                Err(_) => {
                    if strict {
                        Ok(Value::Bool(false))
                    } else {
                        // 非严格模式返回空字符串
                        Ok(Value::String(String::new()))
                    }
                }
            }
        }
        Err(_) => {
            if strict {
                Ok(Value::Bool(false))
            } else {
                Ok(Value::String(String::new()))
            }
        }
    }
}

/// urlencode — 编码 URL 字符串
pub fn builtin_urlencode(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();

    // URL 编码
    let encoded = urlencoding::encode(&s);

    Ok(Value::String(encoded.to_string()))
}

/// urldecode — 解码已编码的 URL 字符串
pub fn builtin_urldecode(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();

    // URL 解码
    match urlencoding::decode(&s) {
        Ok(decoded) => Ok(Value::String(decoded.to_string())),
        Err(_) => Ok(Value::String(s)),
    }
}

/// htmlentities — 将字符转换为 HTML 转义字符
pub fn builtin_htmlentities(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();

    // HTML 实体编码
    let encoded = html_escape::encode_text(&s);

    Ok(Value::String(encoded.to_string()))
}

/// html_entity_decode — 将 HTML 实体转换回字符
pub fn builtin_html_entity_decode(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();

    // HTML 实体解码
    let decoded = html_escape::decode_html_entities(&s);

    Ok(Value::String(decoded.to_string()))
}

/// serialize — 产生一个可存储的值的表示
pub fn builtin_serialize(args: &[Value]) -> Result<Value> {
    let value = args.first().unwrap_or(&Value::Null);

    // 简化的序列化实现
    // 实际 PHP 序列化格式更复杂
    let serialized = format!("SERIALIZED:{}", value.to_json_string());

    Ok(Value::String(serialized))
}

/// unserialize — 从已存储的表示中创建 PHP 的值
pub fn builtin_unserialize(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();

    // 简化的反序列化实现
    if let Some(json_str) = s.strip_prefix("SERIALIZED:") {
        match serde_json::from_str::<serde_json::Value>(json_str) {
            Ok(json_value) => Ok(json_value_to_value(&json_value, true)),
            Err(_) => Ok(Value::Bool(false)),
        }
    } else {
        Ok(Value::Bool(false))
    }
}
