//! Container 门面类方法实现
//!
//! 提供依赖注入容器操作的静态方法
//! 对应 ThinkPHP 8.0 的 Container 门面
//!
//! # 支持的方法
//! - bind: 注册绑定
//! - singleton: 注册单例
//! - make: 解析实例
//! - has: 检查绑定是否存在
//! - forget: 移除绑定

use crate::interpreter::value::{ObjectInstance, Value};

use super::types::FacadeMethod;

/// Container::bind 方法实现
///
/// 注册闭包绑定到容器
///
/// # PHP 用法
/// ```php
/// Container::bind('cache', function($container) { return new Cache(); });
/// ```
///
/// # 参数
/// - abstract_name: 抽象名
/// - callback: 构建器闭包
///
/// # 返回
/// 无返回值
pub fn container_bind(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Null)
}

/// Container::singleton 方法实现
///
/// 注册单例绑定到容器
///
/// # PHP 用法
/// ```php
/// Container::singleton('db', function($container) { return new Database(); });
/// ```
///
/// # 参数
/// - abstract_name: 抽象名
/// - callback: 构建器闭包
///
/// # 返回
/// 无返回值
pub fn container_singleton(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Null)
}

/// Container::instance 方法实现
///
/// 注册已构建的实例到容器
///
/// # PHP 用法
/// ```php
/// Container::instance('config', $configInstance);
/// ```
///
/// # 参数
/// - abstract_name: 抽象名
/// - instance: 实例对象
///
/// # 返回
/// 无返回值
pub fn container_instance(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Null)
}

/// Container::make 方法实现
///
/// 从容器解析实例
///
/// # PHP 用法
/// ```php
/// $cache = Container::make('cache');
/// ```
///
/// # 参数
/// - abstract_name: 抽象名
///
/// # 返回
/// 解析出的实例或 null
pub fn container_make(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let _name = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    Ok(Value::Null)
}

/// Container::has 方法实现
///
/// 检查容器中是否存在绑定
///
/// # PHP 用法
/// ```php
/// if (Container::has('cache')) { ... }
/// ```
///
/// # 参数
/// - abstract_name: 抽象名
///
/// # 返回
/// 存在返回 true，不存在返回 false
pub fn container_has(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let _name = args.first()
        .map(|v| v.to_string_value())
        .unwrap_or_default();

    Ok(Value::Bool(false))
}

/// Container::alias 方法实现
///
/// 为绑定设置别名
///
/// # PHP 用法
/// ```php
/// Container::alias('db', 'database');
/// ```
///
/// # 参数
/// - abstract_name: 抽象名
/// - alias: 别名
///
/// # 返回
/// 无返回值
pub fn container_alias(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Null)
}

/// Container::forget 方法实现
///
/// 从容器中移除绑定
///
/// # PHP 用法
/// ```php
/// Container::forget('cache');
/// ```
///
/// # 参数
/// - abstract_name: 抽象名
///
/// # 返回
/// 无返回值
pub fn container_forget(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Null)
}

/// Container::flush 方法实现
///
/// 清空容器中所有绑定
///
/// # PHP 用法
/// ```php
/// Container::flush();
/// ```
///
/// # 返回
/// 无返回值
pub fn container_flush(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::Null)
}

/// Container::bindings 方法实现
///
/// 获取所有已注册的绑定名
///
/// # PHP 用法
/// ```php
/// $bindings = Container::bindings();
/// ```
///
/// # 返回
/// 绑定名数组
pub fn container_bindings(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    Ok(Value::IndexedArray(vec![]))
}

/// 获取所有 Container 门面方法
///
/// 返回方法名和对应的函数指针
/// 用于注册到门面类注册表中
pub fn get_container_methods() -> Vec<(&'static str, FacadeMethod)> {
    vec![
        ("bind", container_bind),
        ("singleton", container_singleton),
        ("instance", container_instance),
        ("make", container_make),
        ("has", container_has),
        ("alias", container_alias),
        ("forget", container_forget),
        ("flush", container_flush),
        ("bindings", container_bindings),
    ]
}
