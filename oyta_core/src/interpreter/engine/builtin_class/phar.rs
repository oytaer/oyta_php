//! Phar 内置类模块
//!
//! 实现 PHP Phar、PharData、GdImage 类

use crate::interpreter::value::{ObjectInstance, Value};

// ============================================================================
// Phar 类方法实现
// ============================================================================

/// Phar::getSignature 方法实现
///
/// 获取 Phar 签名
///
/// # 参数
/// - `instance`: Phar 对象实例
///
/// # 返回
/// 签名信息数组或 false
pub fn phar_get_signature(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let has_signature = instance.properties.get("hasSignature")
        .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
        .unwrap_or(false);
    
    if has_signature {
        Ok(Value::IndexedArray(vec![
            Value::String("hash".to_string()),
            Value::String(String::new()),
        ]))
    } else {
        Ok(Value::Bool(false))
    }
}

/// Phar::getMetadata 方法实现
///
/// 获取 Phar 元数据
///
/// # 参数
/// - `instance`: Phar 对象实例
///
/// # 返回
/// 元数据
pub fn phar_get_metadata(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let metadata = instance.properties.get("metadata")
        .cloned()
        .unwrap_or(Value::Null);
    
    Ok(metadata)
}

/// Phar::setMetadata 方法实现
///
/// 设置 Phar 元数据
///
/// # 参数
/// - `instance`: Phar 对象实例
/// - `args`: 元数据参数
///
/// # 返回
/// void
pub fn phar_set_metadata(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let metadata = args.first().cloned().unwrap_or(Value::Null);
    
    // 创建新实例
    let mut new_instance = instance.clone();
    new_instance.properties.insert("metadata".to_string(), metadata);
    
    Ok(Value::Object(new_instance))
}

/// Phar::addFile 方法实现
///
/// 添加文件到 Phar
///
/// # 参数
/// - `instance`: Phar 对象实例
/// - `args`: 文件路径参数
///
/// # 返回
/// void
pub fn phar_add_file(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 简化实现：返回当前实例
    Ok(Value::Object(instance.clone()))
}

/// Phar::addFromString 方法实现
///
/// 从字符串添加文件到 Phar
///
/// # 参数
/// - `instance`: Phar 对象实例
/// - `args`: 文件名和内容参数
///
/// # 返回
/// void
pub fn phar_add_from_string(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 简化实现：返回当前实例
    Ok(Value::Object(instance.clone()))
}

// ============================================================================
// PharData 类方法实现
// ============================================================================

/// PharData::getSignature 方法实现
pub fn phardata_get_signature(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    phar_get_signature(instance, args)
}

/// PharData::getMetadata 方法实现
pub fn phardata_get_metadata(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    phar_get_metadata(instance, args)
}
