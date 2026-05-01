//! Crypt 门面类方法实现
//!
//! 提供加密操作的静态方法

use base64::{Engine as _, engine::general_purpose};

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Crypt::encrypt 方法实现
pub fn crypt_encrypt(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let data = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let encoded = general_purpose::STANDARD.encode(data.as_bytes());
    Ok(Value::String(encoded))
}

/// Crypt::decrypt 方法实现
pub fn crypt_decrypt(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let data = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    match general_purpose::STANDARD.decode(data) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(decoded) => Ok(Value::String(decoded)),
            Err(_) => Ok(Value::Bool(false)),
        },
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// 获取所有 Crypt 门面方法
pub fn get_crypt_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("encrypt", crypt_encrypt),
        ("decrypt", crypt_decrypt),
    ]
}
