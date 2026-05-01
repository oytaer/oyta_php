//! Upload 门面类方法实现
//!
//! 提供上传操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Upload::validate 方法实现
pub fn upload_validate(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(true))
}

/// Upload::move 方法实现
pub fn upload_move(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let directory = args.get(1)
        .map(|v| v.to_string_value())
        .unwrap_or_else(|| "uploads".to_string());
    let name = args.get(2)
        .map(|v| v.to_string_value());
    let filename = format!("{}.txt", uuid::Uuid::new_v4());
    tracing::debug!("Upload::move - 目录: {}, 文件名: {:?}", directory, name);
    Ok(Value::String(filename))
}

/// Upload::exists 方法实现
pub fn upload_exists(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Bool(false))
}

/// Upload::delete 方法实现
pub fn upload_delete(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let path = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    tracing::debug!("Upload::delete - 路径: {}", path);
    Ok(Value::Bool(true))
}

/// 获取所有 Upload 门面方法
pub fn get_upload_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("validate", upload_validate),
        ("move", upload_move),
        ("exists", upload_exists),
        ("delete", upload_delete),
    ]
}
