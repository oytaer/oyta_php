//! 内置 PHP 函数模块（基础部分）
//!
//! 实现 PHP 内置函数，覆盖：
//! - 字符串函数：strlen/strtolower/strtoupper/trim/substr/str_replace 等
//! - 编码/解码函数：json_encode/json_decode/base64_encode/base64_decode/urlencode/urldecode
//! - 数组函数：count/array_keys/array_merge/array_push/array_map 等
//!
//! 扩展函数见 builtins_ext 模块
//!
//! # 模块结构
//! - `types`: 类型定义
//! - `string`: 字符串函数
//! - `encoding`: 编码/解码函数
//! - `array`: 数组函数
//! - `helpers`: 辅助函数

pub mod array;
pub mod encoding;
pub mod helpers;
pub mod string;
pub mod types;

use std::collections::HashMap;

use super::builtins_ext;
pub use types::BuiltinFunction;

/// 创建内置函数映射表
///
/// 注册所有内置 PHP 函数到 HashMap 中
/// 在 Interpreter::new() 中调用，初始化解释器的内置函数表
///
/// # 返回
/// 函数名 → 函数指针的映射
pub fn create_builtins_map() -> HashMap<String, BuiltinFunction> {
    let mut map: HashMap<String, BuiltinFunction> = HashMap::new();

    // 注册字符串函数
    string::register_string_functions(&mut map);

    // 注册编码/解码函数
    encoding::register_encoding_functions(&mut map);

    // 注册数组函数
    array::register_array_functions(&mut map);

    // 注册扩展函数
    builtins_ext::register_ext_builtins(&mut map);
    builtins_ext::register_oyta_helpers(&mut map);

    map
}
