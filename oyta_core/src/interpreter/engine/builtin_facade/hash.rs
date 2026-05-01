//! Hash 门面类方法实现
//!
//! 提供哈希操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Hash::make 方法实现
pub fn hash_make(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let password = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let hashed = crate::security::Hash::make(&password);
    Ok(Value::String(hashed))
}

/// Hash::check 方法实现
pub fn hash_check(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let password = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let hashed = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    let result = crate::security::Hash::check(&password, &hashed);
    Ok(Value::Bool(result))
}

/// Hash::needsRehash 方法实现
pub fn hash_needs_rehash(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(false))
}

/// 获取所有 Hash 门面方法
pub fn get_hash_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("make", hash_make),
        ("check", hash_check),
        ("needsRehash", hash_needs_rehash),
    ]
}
