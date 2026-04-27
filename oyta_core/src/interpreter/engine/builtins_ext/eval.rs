//! eval 动态执行内置函数注册模块
//!
//! 本模块实现 PHP eval() 函数的内置函数注册
//! 包括：eval, assert, create_function 等

use std::collections::HashMap;

use super::super::builtins::BuiltinFunction;
use crate::interpreter::value::Value;
use anyhow::Result;

// ============================================================================
// eval 内置函数实现
// ============================================================================

/// eval — 把字符串作为 PHP 代码执行
///
/// # PHP 签名
/// eval(string $code): mixed
///
/// # 参数
/// - args[0]: 要执行的 PHP 代码字符串
///
/// # 返回
/// 执行结果，除非在代码中返回了值，否则返回 null
///
/// # 安全说明
/// eval() 是一个语言结构，允许执行任意 PHP 代码
/// 使用时需要特别注意安全性，避免代码注入攻击
pub fn builtin_eval(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Null);
    }
    
    // 获取代码字符串
    let code = match &args[0] {
        Value::String(s) => s.as_str(),
        Value::Null => return Ok(Value::Null),
        _ => return Ok(Value::Null),
    };
    
    // 安全检查：检查是否包含危险代码
    // 这是一个简化的安全检查，实际实现需要更严格的检查
    let dangerous_patterns = [
        "system(",
        "exec(",
        "shell_exec(",
        "passthru(",
        "popen(",
        "proc_open(",
        "pcntl_exec(",
        "file_put_contents(",
        "fwrite(",
        "unlink(",
        "rmdir(",
        "chmod(",
        "chown(",
    ];
    
    // 检查是否包含危险模式
    let code_lower = code.to_lowercase();
    for pattern in dangerous_patterns.iter() {
        if code_lower.contains(pattern) {
            // 返回错误
            return Err(anyhow::anyhow!("eval(): 安全限制：代码包含不允许的函数调用"));
        }
    }
    
    // 简化实现：返回 null
    // 实际实现需要调用解释器执行代码
    // 这需要访问解释器实例，当前简化处理
    Ok(Value::Null)
}

/// assert — 检查一个断言是否为 false
///
/// # PHP 签名
/// assert(mixed $assertion, Throwable|string|null $description = null): bool
///
/// # 参数
/// - args[0]: 断言表达式或字符串
/// - args[1]: 可选的描述信息
///
/// # 返回
/// 断言结果
pub fn builtin_assert(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(true));
    }
    
    // 获取断言值
    let assertion = &args[0];
    
    // 检查断言是否为真
    let result = assertion.is_truthy();
    
    // 如果断言失败，可能需要抛出异常或警告
    // 简化实现：只返回布尔值
    Ok(Value::Bool(result))
}

/// assert_options — 设置/获取各种 assert 标志
///
/// # PHP 签名
/// assert_options(int $what, mixed $value = ?): mixed
///
/// # 参数
/// - args[0]: 选项常量
/// - args[1]: 可选的新值
///
/// # 返回
/// 选项的原始值
pub fn builtin_assert_options(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 获取选项
    let _what = match &args[0] {
        Value::Int(i) => *i,
        _ => return Ok(Value::Bool(false)),
    };
    
    // 简化实现：返回默认值
    if args.len() > 1 {
        // 设置模式
        Ok(Value::Bool(true))
    } else {
        // 获取模式
        Ok(Value::Bool(true))
    }
}

/// create_function — 创建一个匿名函数（已弃用）
///
/// # PHP 签名
/// create_function(string $args, string $code): string
///
/// # 参数
/// - args[0]: 参数列表字符串
/// - args[1]: 函数体代码字符串
///
/// # 返回
/// 函数名称（以 "\0lambda_" 开头）
///
/// # 警告
/// 此函数已弃用，建议使用匿名函数代替
pub fn builtin_create_function(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取参数和代码
    let _params = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    let _code = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 生成唯一的函数名
    // 使用静态计数器生成唯一名称
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let func_name = format!("\0lambda_{}", id);
    
    // 简化实现：返回生成的函数名
    // 实际实现需要注册函数到解释器
    Ok(Value::String(func_name))
}

/// call_user_func — 把第一个参数作为回调函数调用
///
/// # PHP 签名
/// call_user_func(callable $callback, mixed ...$args): mixed
///
/// # 参数
/// - args[0]: 回调函数
/// - args[1..]: 可选的参数
///
/// # 返回
/// 回调函数的返回值
pub fn builtin_call_user_func(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Null);
    }
    
    // 获取回调函数名
    let _callback = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Null),
    };
    
    // 获取参数
    let _call_args = &args[1..];
    
    // 简化实现：返回 null
    // 实际实现需要调用回调函数
    Ok(Value::Null)
}

/// call_user_func_array — 调用回调函数，并把一个数组参数作为回调函数的参数
///
/// # PHP 签名
/// call_user_func_array(callable $callback, array $args): mixed
///
/// # 参数
/// - args[0]: 回调函数
/// - args[1]: 参数数组
///
/// # 返回
/// 回调函数的返回值
pub fn builtin_call_user_func_array(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Null);
    }
    
    // 获取回调函数名
    let _callback = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Null),
    };
    
    // 获取参数数组
    let _call_args = match &args[1] {
        Value::IndexedArray(arr) => arr,
        _ => return Ok(Value::Null),
    };
    
    // 简化实现：返回 null
    Ok(Value::Null)
}

/// forward_static_call — 调用静态方法
///
/// # PHP 签名
/// forward_static_call(callable $callback, mixed ...$args): mixed
///
/// # 参数
/// - args[0]: 回调函数
/// - args[1..]: 可选的参数
///
/// # 返回
/// 回调函数的返回值
pub fn builtin_forward_static_call(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Null);
    }
    
    // 简化实现：返回 null
    Ok(Value::Null)
}

/// forward_static_call_array — 调用静态方法，并把一个数组参数作为回调函数的参数
///
/// # PHP 签名
/// forward_static_call_array(callable $callback, array $args): mixed
///
/// # 参数
/// - args[0]: 回调函数
/// - args[1]: 参数数组
///
/// # 返回
/// 回调函数的返回值
pub fn builtin_forward_static_call_array(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Null);
    }
    
    // 简化实现：返回 null
    Ok(Value::Null)
}

/// register_shutdown_function — 注册一个会在脚本结束时执行的函数
///
/// # PHP 签名
/// register_shutdown_function(callable $callback, mixed ...$args): void
///
/// # 参数
/// - args[0]: 回调函数
/// - args[1..]: 可选的参数
///
/// # 返回
/// 无返回值
pub fn builtin_register_shutdown_function(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Null);
    }
    
    // 简化实现：返回 null
    // 实际实现需要注册回调函数
    Ok(Value::Null)
}

/// register_tick_function — 注册一个在每个 tick 上执行的函数
///
/// # PHP 签名
/// register_tick_function(callable $callback, mixed ...$args): bool
///
/// # 参数
/// - args[0]: 回调函数
/// - args[1..]: 可选的参数
///
/// # 返回
/// 成功返回 true
pub fn builtin_register_tick_function(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// unregister_tick_function — 取消注册 tick 函数
///
/// # PHP 签名
/// unregister_tick_function(callable $callback): void
///
/// # 参数
/// - args[0]: 回调函数
///
/// # 返回
/// 无返回值
pub fn builtin_unregister_tick_function(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Null);
    }
    
    // 简化实现：返回 null
    Ok(Value::Null)
}

/// is_callable — 验证值是否可以在范围内作为函数调用
///
/// # PHP 签名
/// is_callable(mixed $value, bool $syntax_only = false, ?string &$callable_name = null): bool
///
/// # 参数
/// - args[0]: 要检查的值
/// - args[1]: 可选，是否只检查语法
/// - args[2]: 可选，接收可调用名称
///
/// # 返回
/// 是否可调用
pub fn builtin_is_callable(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 检查值是否可调用
    let is_callable = match &args[0] {
        // 字符串函数名
        Value::String(s) => {
            // 检查是否是有效的函数名格式
            !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_')
        }
        // 数组形式的回调 [类名或对象, 方法名]
        Value::IndexedArray(arr) => {
            arr.len() == 2 && 
            matches!(arr.get(0), Some(Value::String(_))) &&
            matches!(arr.get(1), Some(Value::String(_)))
        }
        // 对象（检查是否有 __invoke 方法）
        Value::Object(_) => true,
        _ => false,
    };
    
    Ok(Value::Bool(is_callable))
}

/// function_exists — 如果给定的函数已经被定义就返回 true
///
/// # PHP 签名
/// function_exists(string $function_name): bool
///
/// # 参数
/// - args[0]: 函数名
///
/// # 返回
/// 函数是否存在
pub fn builtin_function_exists(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 获取函数名
    let _func_name = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 简化实现：返回 true
    // 实际实现需要检查函数表
    Ok(Value::Bool(true))
}

/// class_exists — 检查类是否已定义
///
/// # PHP 签名
/// class_exists(string $class, bool $autoload = true): bool
///
/// # 参数
/// - args[0]: 类名
/// - args[1]: 可选，是否自动加载
///
/// # 返回
/// 类是否存在
pub fn builtin_class_exists(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 获取类名
    let _class_name = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// method_exists — 检查类的方法是否存在
///
/// # PHP 签名
/// method_exists(object|string $object_or_class, string $method): bool
///
/// # 参数
/// - args[0]: 对象或类名
/// - args[1]: 方法名
///
/// # 返回
/// 方法是否存在
pub fn builtin_method_exists(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取方法名
    let _method_name = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// property_exists — 检查对象或类是否具有该属性
///
/// # PHP 签名
/// property_exists(object|string $object_or_class, string $property): bool
///
/// # 参数
/// - args[0]: 对象或类名
/// - args[1]: 属性名
///
/// # 返回
/// 属性是否存在
pub fn builtin_property_exists(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取属性名
    let _property_name = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 简化实现：返回 true
    Ok(Value::Bool(true))
}

/// trait_exists — 检查 trait 是否存在
///
/// # PHP 签名
/// trait_exists(string $trait, bool $autoload = true): bool
///
/// # 参数
/// - args[0]: trait 名称
/// - args[1]: 可选，是否自动加载
///
/// # 返回
/// trait 是否存在
pub fn builtin_trait_exists(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 false
    Ok(Value::Bool(false))
}

/// interface_exists — 检查接口是否已被定义
///
/// # PHP 签名
/// interface_exists(string $interface_name, bool $autoload = true): bool
///
/// # 参数
/// - args[0]: 接口名称
/// - args[1]: 可选，是否自动加载
///
/// # 返回
/// 接口是否存在
pub fn builtin_interface_exists(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 简化实现：返回 false
    Ok(Value::Bool(false))
}

// ============================================================================
// 函数注册
// ============================================================================

/// 注册 eval 相关的内置函数
///
/// 将所有 eval 相关函数注册到内置函数映射表中
///
/// # 参数
/// - `map`: 内置函数映射表的可变引用
pub fn register_eval_functions(map: &mut HashMap<String, BuiltinFunction>) {
    // 动态执行函数
    map.insert("eval".to_string(), builtin_eval);
    map.insert("assert".to_string(), builtin_assert);
    map.insert("assert_options".to_string(), builtin_assert_options);
    map.insert("create_function".to_string(), builtin_create_function);
    
    // 回调函数
    map.insert("call_user_func".to_string(), builtin_call_user_func);
    map.insert("call_user_func_array".to_string(), builtin_call_user_func_array);
    map.insert("forward_static_call".to_string(), builtin_forward_static_call);
    map.insert("forward_static_call_array".to_string(), builtin_forward_static_call_array);
    
    // 注册函数
    map.insert("register_shutdown_function".to_string(), builtin_register_shutdown_function);
    map.insert("register_tick_function".to_string(), builtin_register_tick_function);
    map.insert("unregister_tick_function".to_string(), builtin_unregister_tick_function);
    
    // 检查函数
    map.insert("is_callable".to_string(), builtin_is_callable);
    map.insert("function_exists".to_string(), builtin_function_exists);
    map.insert("class_exists".to_string(), builtin_class_exists);
    map.insert("method_exists".to_string(), builtin_method_exists);
    map.insert("property_exists".to_string(), builtin_property_exists);
    map.insert("trait_exists".to_string(), builtin_trait_exists);
    map.insert("interface_exists".to_string(), builtin_interface_exists);
}
