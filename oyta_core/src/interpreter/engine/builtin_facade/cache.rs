//! Cache 门面类方法实现
//!
//! 提供缓存操作的静态方法
//! 对应 ThinkPHP 8.0 的 Cache 门面功能
//!
//! # 支持的方法
//! - get: 获取缓存
//! - getOr: 获取缓存，带默认值
//! - set: 设置缓存
//! - forever: 永久设置缓存
//! - delete: 删除缓存
//! - has: 检查缓存是否存在
//! - clear: 清空所有缓存
//! - increment: 自增
//! - decrement: 自减
//! - remember: 记住缓存（如果不存在则设置）
//! - pull: 获取并删除缓存
//! - put: 设置缓存（set 的别名）

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Cache::get 方法实现
///
/// 获取缓存值
///
/// # PHP 用法
/// ```php
/// $value = Cache::get('key');
/// ```
///
/// # 参数
/// - key: 缓存键名
///
/// # 返回
/// 缓存值，不存在则返回 null
pub fn cache_get(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    match crate::cache::facade::Cache::get(&key) {
        Some(value) => Ok(Value::String(value)),
        None => Ok(Value::Null),
    }
}

/// Cache::getOr 方法实现
///
/// 获取缓存值，如果不存在则返回默认值
///
/// # PHP 用法
/// ```php
/// $value = Cache::getOr('key', 'default');
/// ```
///
/// # 参数
/// - key: 缓存键名
/// - default: 默认值
///
/// # 返回
/// 缓存值或默认值
pub fn cache_get_or(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let default = args.get(1)
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    Ok(Value::String(crate::cache::facade::Cache::get_or(&key, &default)))
}

/// Cache::set 方法实现
///
/// 设置缓存值
///
/// # PHP 用法
/// ```php
/// Cache::set('key', 'value', 3600);
/// ```
///
/// # 参数
/// - key: 缓存键名
/// - value: 缓存值
/// - ttl: 过期时间（秒），0 表示永不过期
///
/// # 返回
/// 成功返回 true，失败返回 false
pub fn cache_set(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let value = args.get(1)
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let ttl = args.get(2)
        .map(|v| v.to_int() as u64)
        .unwrap_or(0);

    match crate::cache::facade::Cache::set(&key, &value, ttl) {
        Ok(()) => Ok(Value::Bool(true)),
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// Cache::forever 方法实现
///
/// 永久设置缓存（不会过期）
///
/// # PHP 用法
/// ```php
/// Cache::forever('key', 'value');
/// ```
///
/// # 参数
/// - key: 缓存键名
/// - value: 缓存值
///
/// # 返回
/// 成功返回 true，失败返回 false
pub fn cache_forever(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let value = args.get(1)
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    match crate::cache::facade::Cache::forever(&key, &value) {
        Ok(()) => Ok(Value::Bool(true)),
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// Cache::delete 方法实现
///
/// 删除缓存
///
/// # PHP 用法
/// ```php
/// Cache::delete('key');
/// ```
///
/// # 参数
/// - key: 缓存键名
///
/// # 返回
/// 成功返回 true，失败返回 false
pub fn cache_delete(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    match crate::cache::facade::Cache::delete(&key) {
        Ok(()) => Ok(Value::Bool(true)),
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// Cache::has 方法实现
///
/// 检查缓存是否存在
///
/// # PHP 用法
/// ```php
/// if (Cache::has('key')) { ... }
/// ```
///
/// # 参数
/// - key: 缓存键名
///
/// # 返回
/// 存在返回 true，不存在返回 false
pub fn cache_has(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    Ok(Value::Bool(crate::cache::facade::Cache::has(&key)))
}

/// Cache::clear 方法实现
///
/// 清空所有缓存
///
/// # PHP 用法
/// ```php
/// Cache::clear();
/// ```
///
/// # 返回
/// 成功返回 true，失败返回 false
pub fn cache_clear(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    match crate::cache::facade::Cache::clear() {
        Ok(()) => Ok(Value::Bool(true)),
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// Cache::increment 方法实现
///
/// 缓存值自增
///
/// # PHP 用法
/// ```php
/// Cache::increment('counter', 1);
/// ```
///
/// # 参数
/// - key: 缓存键名
/// - step: 自增步长，默认为 1
///
/// # 返回
/// 自增后的值，失败返回 false
pub fn cache_increment(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let step = args.get(1)
        .map(|v| v.to_int())
        .unwrap_or(1);

    match crate::cache::facade::Cache::increment(&key, step) {
        Ok(result) => Ok(Value::Int(result)),
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// Cache::decrement 方法实现
///
/// 缓存值自减
///
/// # PHP 用法
/// ```php
/// Cache::decrement('counter', 1);
/// ```
///
/// # 参数
/// - key: 缓存键名
/// - step: 自减步长，默认为 1
///
/// # 返回
/// 自减后的值，失败返回 false
pub fn cache_decrement(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let step = args.get(1)
        .map(|v| v.to_int())
        .unwrap_or(1);

    match crate::cache::facade::Cache::decrement(&key, step) {
        Ok(result) => Ok(Value::Int(result)),
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// Cache::remember 方法实现
///
/// 记住缓存（如果不存在则设置）
///
/// # PHP 用法
/// ```php
/// $value = Cache::remember('key', 3600, 'default_value');
/// ```
///
/// # 参数
/// - key: 缓存键名
/// - ttl: 过期时间（秒）
/// - value: 缓存值（不存在时设置）
///
/// # 返回
/// 缓存值
pub fn cache_remember(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let ttl = args.get(1)
        .map(|v| v.to_int() as u64)
        .unwrap_or(0);
    let value = args.get(2)
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    match crate::cache::facade::Cache::remember(&key, ttl, &value) {
        Ok(result) => Ok(Value::String(result)),
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// Cache::pull 方法实现
///
/// 获取并删除缓存
///
/// # PHP 用法
/// ```php
/// $value = Cache::pull('key');
/// ```
///
/// # 参数
/// - key: 缓存键名
///
/// # 返回
/// 缓存值，获取后删除
pub fn cache_pull(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    // 先获取值
    let value = crate::cache::facade::Cache::get(&key);
    // 然后删除
    let _ = crate::cache::facade::Cache::delete(&key);

    match value {
        Some(v) => Ok(Value::String(v)),
        None => Ok(Value::Null),
    }
}

/// Cache::put 方法实现
///
/// 设置缓存（set 的别名）
///
/// # PHP 用法
/// ```php
/// Cache::put('key', 'value', 3600);
/// ```
///
/// # 参数
/// - key: 缓存键名
/// - value: 缓存值
/// - ttl: 过期时间（秒）
///
/// # 返回
/// 成功返回 true，失败返回 false
pub fn cache_put(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    cache_set(_instance, args)
}

/// Cache::tags 方法实现
///
/// 设置缓存标签
///
/// # PHP 用法
/// ```php
/// Cache::tags(['user', 'profile'])->set('key', 'value');
/// ```
///
/// # 参数
/// - tags: 标签数组
///
/// # 返回
/// 返回 TaggedCache 对象用于链式调用
pub fn cache_tags(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let tags = args.first()
        .map(|v| {
            match v {
                Value::IndexedArray(arr) => arr.iter()
                    .map(|item| item.to_string_value())
                    .collect::<Vec<_>>(),
                Value::AssociativeArray(map) => map.iter()
                    .map(|(_, item)| item.to_string_value())
                    .collect::<Vec<_>>(),
                Value::String(s) => vec![s.clone()],
                _ => vec![],
            }
        })
        .unwrap_or_default();

    // 创建 TaggedCache 对象
    let mut properties = std::collections::HashMap::new();
    properties.insert("tags".to_string(), Value::IndexedArray(
        tags.iter().map(|t| Value::String(t.clone())).collect()
    ));

    Ok(Value::Object(ObjectInstance {
        class_name: "TaggedCache".to_string(),
        properties,
    }))
}

/// Cache::tag 方法实现
///
/// 设置单个缓存标签
///
/// # PHP 用法
/// ```php
/// Cache::tag('user')->set('key', 'value');
/// ```
///
/// # 参数
/// - tag: 标签名
///
/// # 返回
/// 返回 TaggedCache 对象用于链式调用
pub fn cache_tag(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let tag = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    // 创建 TaggedCache 对象
    let mut properties = std::collections::HashMap::new();
    properties.insert("tags".to_string(), Value::IndexedArray(
        vec![Value::String(tag)]
    ));

    Ok(Value::Object(ObjectInstance {
        class_name: "TaggedCache".to_string(),
        properties,
    }))
}

/// Cache::forget 方法实现
///
/// 删除缓存（delete 的别名）
///
/// # PHP 用法
/// ```php
/// Cache::forget('key');
/// ```
///
/// # 参数
/// - key: 缓存键名
///
/// # 返回
/// 成功返回 true，失败返回 false
pub fn cache_forget(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    cache_delete(_instance, args)
}

/// Cache::add 方法实现
///
/// 仅当缓存不存在时设置
///
/// # PHP 用法
/// ```php
/// Cache::add('key', 'value', 3600);
/// ```
///
/// # 参数
/// - key: 缓存键名
/// - value: 缓存值
/// - ttl: 过期时间（秒）
///
/// # 返回
/// 设置成功返回 true，已存在返回 false
pub fn cache_add(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    // 检查是否已存在
    if crate::cache::facade::Cache::has(&key) {
        return Ok(Value::Bool(false));
    }

    // 不存在则设置
    cache_set(_instance, args)
}

/// 获取所有 Cache 门面方法
///
/// 返回方法名和对应的函数指针
/// 用于注册到门面类注册表中
pub fn get_cache_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        // 基础操作
        ("get", cache_get),
        ("getOr", cache_get_or),
        ("set", cache_set),
        ("put", cache_put),
        ("forever", cache_forever),
        ("delete", cache_delete),
        ("forget", cache_forget),
        ("has", cache_has),
        ("clear", cache_clear),
        // 数值操作
        ("increment", cache_increment),
        ("decrement", cache_decrement),
        // 高级操作
        ("remember", cache_remember),
        ("pull", cache_pull),
        ("add", cache_add),
        // 标签操作
        ("tag", cache_tag),
        ("tags", cache_tags),
    ]
}
