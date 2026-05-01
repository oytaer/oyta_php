//! Storage 门面类方法实现
//!
//! 提供存储操作的静态方法

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Storage::exists 方法实现
pub fn storage_exists(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let path = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::storage::Storage::exists(&path, None).await
    });
    Ok(Value::Bool(result))
}

/// Storage::get 方法实现
pub fn storage_get(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let path = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::storage::Storage::get_as_string(&path, None).await
    });
    match result {
        Ok(content) => Ok(Value::String(content)),
        Err(e) => {
            tracing::error!("Storage::get 错误: {}", e);
            Ok(Value::String(String::new()))
        }
    }
}

/// Storage::put 方法实现
pub fn storage_put(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let path = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let content = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::storage::Storage::put_string(&path, &content, None).await
    });
    match result {
        Ok(success) => Ok(Value::Bool(success)),
        Err(e) => {
            tracing::error!("Storage::put 错误: {}", e);
            Ok(Value::Bool(false))
        }
    }
}

/// Storage::delete 方法实现
pub fn storage_delete(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let path = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::storage::Storage::delete(&path, None).await
    });
    match result {
        Ok(success) => Ok(Value::Bool(success)),
        Err(e) => {
            tracing::error!("Storage::delete 错误: {}", e);
            Ok(Value::Bool(false))
        }
    }
}

/// Storage::copy 方法实现
pub fn storage_copy(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let from = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let to = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::storage::Storage::copy(&from, &to, None).await
    });
    match result {
        Ok(success) => Ok(Value::Bool(success)),
        Err(e) => {
            tracing::error!("Storage::copy 错误: {}", e);
            Ok(Value::Bool(false))
        }
    }
}

/// Storage::move 方法实现
pub fn storage_move(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let from = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let to = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::storage::Storage::move_file(&from, &to, None).await
    });
    match result {
        Ok(success) => Ok(Value::Bool(success)),
        Err(e) => {
            tracing::error!("Storage::move 错误: {}", e);
            Ok(Value::Bool(false))
        }
    }
}

/// Storage::size 方法实现
pub fn storage_size(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let path = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::storage::Storage::size(&path, None).await
    });
    match result {
        Ok(size) => Ok(Value::Int(size as i64)),
        Err(e) => {
            tracing::error!("Storage::size 错误: {}", e);
            Ok(Value::Int(0))
        }
    }
}

/// Storage::url 方法实现
pub fn storage_url(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let path = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::storage::Storage::url(&path, None).await
    });
    match result {
        Ok(url) => Ok(Value::String(url)),
        Err(e) => {
            tracing::error!("Storage::url 错误: {}", e);
            Ok(Value::String(String::new()))
        }
    }
}

/// Storage::files 方法实现
pub fn storage_files(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let directory = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::storage::Storage::files(&directory, None).await
    });
    match result {
        Ok(files) => Ok(Value::IndexedArray(files.into_iter().map(Value::String).collect())),
        Err(e) => {
            tracing::error!("Storage::files 错误: {}", e);
            Ok(Value::IndexedArray(Vec::new()))
        }
    }
}

/// Storage::makeDirectory 方法实现
pub fn storage_make_directory(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let path = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::storage::Storage::make_directory(&path, None).await
    });
    match result {
        Ok(success) => Ok(Value::Bool(success)),
        Err(e) => {
            tracing::error!("Storage::makeDirectory 错误: {}", e);
            Ok(Value::Bool(false))
        }
    }
}

/// Storage::deleteDirectory 方法实现
pub fn storage_delete_directory(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let path = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        crate::storage::Storage::delete_directory(&path, None).await
    });
    match result {
        Ok(success) => Ok(Value::Bool(success)),
        Err(e) => {
            tracing::error!("Storage::deleteDirectory 错误: {}", e);
            Ok(Value::Bool(false))
        }
    }
}

/// 获取所有 Storage 门面方法
pub fn get_storage_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("exists", storage_exists),
        ("get", storage_get),
        ("put", storage_put),
        ("delete", storage_delete),
        ("copy", storage_copy),
        ("move", storage_move),
        ("size", storage_size),
        ("url", storage_url),
        ("files", storage_files),
        ("makeDirectory", storage_make_directory),
        ("deleteDirectory", storage_delete_directory),
    ]
}
