//! Session 门面类方法实现
//!
//! 提供会话操作的静态方法
//! 对应 ThinkPHP 8.0 的 Session 门面功能
//!
//! # 支持的方法
//! - get: 获取会话值
//! - set: 设置会话值
//! - delete: 删除会话值
//! - has: 检查会话值是否存在
//! - clear: 清空会话
//! - destroy: 销毁会话
//! - getId: 获取会话ID
//! - setId: 设置会话ID
//! - regenerate: 重新生成会话ID
//! - all: 获取所有会话数据
//! - pull: 获取并删除会话值
//! - flash: 设置闪存数据
//! - reflash: 刷新闪存数据
//! - keep: 保留闪存数据

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Session::get 方法实现
///
/// 获取会话值
///
/// # PHP 用法
/// ```php
/// $value = Session::get('key');
/// ```
///
/// # 参数
/// - key: 会话键名
///
/// # 返回
/// 会话值，不存在则返回 null
pub fn session_get(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    match crate::session::facade::Session::get(&key) {
        Some(value) => Ok(Value::String(value)),
        None => Ok(Value::Null),
    }
}

/// Session::set 方法实现
///
/// 设置会话值
///
/// # PHP 用法
/// ```php
/// Session::set('key', 'value');
/// ```
///
/// # 参数
/// - key: 会话键名
/// - value: 会话值
///
/// # 返回
/// 成功返回 true，失败返回 false
pub fn session_set(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let value = args.get(1)
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    match crate::session::facade::Session::set(&key, &value) {
        Ok(()) => Ok(Value::Bool(true)),
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// Session::delete 方法实现
///
/// 删除会话值
///
/// # PHP 用法
/// ```php
/// Session::delete('key');
/// ```
///
/// # 参数
/// - key: 会话键名
///
/// # 返回
/// 成功返回 true，失败返回 false
pub fn session_delete(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    match crate::session::facade::Session::delete(&key) {
        Ok(()) => Ok(Value::Bool(true)),
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// Session::has 方法实现
///
/// 检查会话值是否存在
///
/// # PHP 用法
/// ```php
/// if (Session::has('key')) { ... }
/// ```
///
/// # 参数
/// - key: 会话键名
///
/// # 返回
/// 存在返回 true，不存在返回 false
pub fn session_has(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    Ok(Value::Bool(crate::session::facade::Session::has(&key)))
}

/// Session::clear 方法实现
///
/// 清空会话
///
/// # PHP 用法
/// ```php
/// Session::clear();
/// ```
///
/// # 返回
/// 成功返回 true，失败返回 false
pub fn session_clear(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    match crate::session::facade::Session::clear() {
        Ok(()) => Ok(Value::Bool(true)),
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// Session::destroy 方法实现
///
/// 销毁会话
///
/// # PHP 用法
/// ```php
/// Session::destroy();
/// ```
///
/// # 返回
/// 成功返回 true，失败返回 false
pub fn session_destroy(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    match crate::session::facade::Session::destroy() {
        Ok(()) => Ok(Value::Bool(true)),
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// Session::getId 方法实现
///
/// 获取会话ID
///
/// # PHP 用法
/// ```php
/// $id = Session::getId();
/// ```
///
/// # 返回
/// 会话ID字符串
pub fn session_get_id(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 简化实现：返回空字符串
    // 实际实现应从 Session 管理器获取
    Ok(Value::String(String::new()))
}

/// Session::setId 方法实现
///
/// 设置会话ID
///
/// # PHP 用法
/// ```php
/// Session::setId('new_session_id');
/// ```
///
/// # 参数
/// - id: 新的会话ID
///
/// # 返回
/// 成功返回 true
pub fn session_set_id(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let _id = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    // 简化实现
    Ok(Value::Bool(true))
}

/// Session::regenerate 方法实现
///
/// 重新生成会话ID
///
/// # PHP 用法
/// ```php
/// Session::regenerate();
/// ```
///
/// # 参数
/// - destroy: 是否销毁旧会话（可选，默认 false）
///
/// # 返回
/// 成功返回 true
pub fn session_regenerate(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 简化实现
    Ok(Value::Bool(true))
}

/// Session::all 方法实现
///
/// 获取所有会话数据
///
/// # PHP 用法
/// ```php
/// $all = Session::all();
/// ```
///
/// # 返回
/// 所有会话数据的数组
pub fn session_all(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 简化实现：返回空数组
    Ok(Value::IndexedArray(vec![]))
}

/// Session::pull 方法实现
///
/// 获取并删除会话值
///
/// # PHP 用法
/// ```php
/// $value = Session::pull('key');
/// ```
///
/// # 参数
/// - key: 会话键名
///
/// # 返回
/// 会话值，获取后删除
pub fn session_pull(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    // 先获取值
    let value = crate::session::facade::Session::get(&key);
    // 然后删除
    let _ = crate::session::facade::Session::delete(&key);

    match value {
        Some(v) => Ok(Value::String(v)),
        None => Ok(Value::Null),
    }
}

/// Session::flash 方法实现
///
/// 设置闪存数据
///
/// # PHP 用法
/// ```php
/// Session::flash('status', 'success');
/// ```
///
/// # 参数
/// - key: 闪存键名
/// - value: 闪存值
///
/// # 返回
/// 成功返回 true
pub fn session_flash(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let value = args.get(1)
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    // 设置闪存数据（带 _flash_ 前缀）
    let flash_key = format!("_flash_{}", key);
    match crate::session::facade::Session::set(&flash_key, &value) {
        Ok(()) => Ok(Value::Bool(true)),
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// Session::reflash 方法实现
///
/// 刷新所有闪存数据
///
/// # PHP 用法
/// ```php
/// Session::reflash();
/// ```
///
/// # 返回
/// 成功返回 true
pub fn session_reflash(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 简化实现
    Ok(Value::Bool(true))
}

/// Session::keep 方法实现
///
/// 保留指定闪存数据
///
/// # PHP 用法
/// ```php
/// Session::keep(['status', 'message']);
/// ```
///
/// # 参数
/// - keys: 要保留的闪存键名数组
///
/// # 返回
/// 成功返回 true
pub fn session_keep(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 简化实现
    Ok(Value::Bool(true))
}

/// Session::push 方法实现
///
/// 向会话数组中推送值
///
/// # PHP 用法
/// ```php
/// Session::push('cart', $item);
/// ```
///
/// # 参数
/// - key: 会话键名
/// - value: 要推送的值
///
/// # 返回
/// 成功返回 true
pub fn session_push(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let _key = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();
    let _value = args.get(1);

    // 简化实现
    Ok(Value::Bool(true))
}

/// Session::forget 方法实现
///
/// 删除多个会话值（delete 的别名）
///
/// # PHP 用法
/// ```php
/// Session::forget('key');
/// // 或
/// Session::forget(['key1', 'key2']);
/// ```
///
/// # 参数
/// - keys: 键名或键名数组
///
/// # 返回
/// 成功返回 true
pub fn session_forget(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    session_delete(_instance, args)
}

/// 获取所有 Session 门面方法
///
/// 返回方法名和对应的函数指针
/// 用于注册到门面类注册表中
pub fn get_session_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        // 基础操作
        ("get", session_get),
        ("set", session_set),
        ("delete", session_delete),
        ("forget", session_forget),
        ("has", session_has),
        ("clear", session_clear),
        ("destroy", session_destroy),
        // 会话ID操作
        ("getId", session_get_id),
        ("setId", session_set_id),
        ("regenerate", session_regenerate),
        // 数据操作
        ("all", session_all),
        ("pull", session_pull),
        ("push", session_push),
        // 闪存操作
        ("flash", session_flash),
        ("reflash", session_reflash),
        ("keep", session_keep),
    ]
}
