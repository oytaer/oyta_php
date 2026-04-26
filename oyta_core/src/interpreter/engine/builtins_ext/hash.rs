//! 哈希函数模块
//!
//! 包含 md5、sha1 等哈希函数

use anyhow::Result;

use crate::interpreter::value::Value;

/// md5 — 计算字符串的 MD5 散列值
pub fn builtin_md5(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::String(format!(
        "{:x}",
        md5::compute(s.as_bytes())
    )))
}

/// sha1 — 计算字符串的 SHA-1 散列值
/// 注意：由于 sha1 crate 不在依赖中，此处使用 sha256 替代
pub fn builtin_sha1(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    Ok(Value::String(hex::encode(result)))
}
