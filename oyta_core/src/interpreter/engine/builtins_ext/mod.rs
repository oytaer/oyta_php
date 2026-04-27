//! 内置 PHP 函数扩展模块
//!
//! 包含类型检测、类型转换、数学、调试、哈希、时间、类/对象等函数
//! 以及 DateTime、mbstring、GD、XML、Phar、eval 等扩展函数
//! 这些函数通过 register_ext_builtins() 注册到主内置函数映射表中
//!
//! # 模块结构
//! - `type_check`: 类型检测函数
//! - `type_cast`: 类型转换函数
//! - `math`: 数学函数
//! - `debug`: 调试函数
//! - `string_ext`: 字符串扩展函数
//! - `hash`: 哈希函数
//! - `time`: 时间函数
//! - `class_object`: 类/对象函数
//! - `validation`: 验证函数
//! - `helpers`: OYTAPHP 助手函数
//! - `datetime`: DateTime 日期时间函数
//! - `mbstring`: 多字节字符串函数
//! - `gd`: GD 图像处理函数
//! - `xml`: XML 处理函数
//! - `phar`: Phar 归档函数
//! - `eval`: 动态执行函数

pub mod class_object;
pub mod debug;
pub mod hash;
pub mod helpers;
pub mod math;
pub mod string_ext;
pub mod time;
pub mod type_cast;
pub mod type_check;
pub mod validation;

// 新增扩展模块
pub mod datetime;
pub mod mbstring;
pub mod gd;
pub mod xml;
pub mod phar;
pub mod eval;

use std::collections::HashMap;

use super::builtins::BuiltinFunction;
use class_object::{
    builtin_call_user_func, builtin_call_user_func_array, builtin_class_exists,
    builtin_func_num_args, builtin_get_class, builtin_method_exists, builtin_property_exists,
};
use debug::{builtin_dd, builtin_dump, builtin_json, builtin_print_r, builtin_var_dump};
use hash::{builtin_md5, builtin_sha1};
pub use helpers::register_oyta_helpers;
use math::{
    builtin_abs, builtin_ceil, builtin_floor, builtin_max, builtin_min, builtin_round,
};
use string_ext::{
    builtin_chr, builtin_htmlspecialchars, builtin_lcfirst, builtin_nl2br, builtin_number_format,
    builtin_ord, builtin_printf, builtin_sprintf, builtin_str_split, builtin_strip_tags,
    builtin_ucfirst, builtin_ucwords,
};
use time::{builtin_date, builtin_microtime, builtin_strtotime, builtin_time};
use type_cast::{builtin_boolval, builtin_floatval, builtin_intval, builtin_strval};
use type_check::{
    builtin_is_array, builtin_is_bool, builtin_is_callable, builtin_is_empty, builtin_is_float,
    builtin_is_int, builtin_is_null, builtin_is_numeric, builtin_is_object, builtin_is_string,
};
use validation::{
    builtin_filter_input, builtin_filter_var, builtin_validate_email, builtin_validate_ip,
    builtin_validate_length, builtin_validate_range, builtin_validate_regex,
    builtin_validate_required, builtin_validate_url,
};

/// 注册扩展内置函数到映射表
///
/// 将类型检测、类型转换、数学、调试、其他、哈希、时间、类/对象函数
/// 注册到主映射表中，由 create_builtins_map() 调用
///
/// # 参数
/// - `map`: 内置函数映射表的可变引用
pub fn register_ext_builtins(map: &mut HashMap<String, BuiltinFunction>) {
    // 注册类型检测函数
    map.insert("is_null".to_string(), builtin_is_null);
    map.insert("is_array".to_string(), builtin_is_array);
    map.insert("is_string".to_string(), builtin_is_string);
    map.insert("is_int".to_string(), builtin_is_int);
    map.insert("is_float".to_string(), builtin_is_float);
    map.insert("is_bool".to_string(), builtin_is_bool);
    map.insert("is_object".to_string(), builtin_is_object);
    map.insert("is_callable".to_string(), builtin_is_callable);
    map.insert("is_numeric".to_string(), builtin_is_numeric);
    map.insert("is_empty".to_string(), builtin_is_empty);

    // 注册类型转换函数
    map.insert("intval".to_string(), builtin_intval);
    map.insert("floatval".to_string(), builtin_floatval);
    map.insert("strval".to_string(), builtin_strval);
    map.insert("boolval".to_string(), builtin_boolval);

    // 注册数学函数
    map.insert("abs".to_string(), builtin_abs);
    map.insert("max".to_string(), builtin_max);
    map.insert("min".to_string(), builtin_min);
    map.insert("floor".to_string(), builtin_floor);
    map.insert("ceil".to_string(), builtin_ceil);
    map.insert("round".to_string(), builtin_round);

    // 注册调试函数
    map.insert("var_dump".to_string(), builtin_var_dump);
    map.insert("print_r".to_string(), builtin_print_r);
    map.insert("json".to_string(), builtin_json);
    map.insert("dd".to_string(), builtin_dd);
    map.insert("dump".to_string(), builtin_dump);

    // 注册字符串扩展函数
    map.insert("sprintf".to_string(), builtin_sprintf);
    map.insert("printf".to_string(), builtin_printf);
    map.insert("number_format".to_string(), builtin_number_format);
    map.insert("chr".to_string(), builtin_chr);
    map.insert("ord".to_string(), builtin_ord);
    map.insert("str_split".to_string(), builtin_str_split);
    map.insert("ucfirst".to_string(), builtin_ucfirst);
    map.insert("lcfirst".to_string(), builtin_lcfirst);
    map.insert("ucwords".to_string(), builtin_ucwords);
    map.insert("nl2br".to_string(), builtin_nl2br);
    map.insert("htmlspecialchars".to_string(), builtin_htmlspecialchars);
    map.insert("htmlentities".to_string(), builtin_htmlspecialchars);
    map.insert("strip_tags".to_string(), builtin_strip_tags);

    // 注册哈希函数
    map.insert("md5".to_string(), builtin_md5);
    map.insert("sha1".to_string(), builtin_sha1);

    // 注册时间函数
    map.insert("time".to_string(), builtin_time);
    map.insert("date".to_string(), builtin_date);
    map.insert("microtime".to_string(), builtin_microtime);
    map.insert("strtotime".to_string(), builtin_strtotime);

    // 注册类/对象函数
    map.insert("class_exists".to_string(), builtin_class_exists);
    map.insert("method_exists".to_string(), builtin_method_exists);
    map.insert("property_exists".to_string(), builtin_property_exists);
    map.insert("get_class".to_string(), builtin_get_class);
    map.insert("call_user_func".to_string(), builtin_call_user_func);
    map.insert("call_user_func_array".to_string(), builtin_call_user_func_array);
    map.insert("func_num_args".to_string(), builtin_func_num_args);

    // 注册验证函数
    map.insert("filter_var".to_string(), builtin_filter_var);
    map.insert("filter_input".to_string(), builtin_filter_input);
    map.insert("validate_email".to_string(), builtin_validate_email);
    map.insert("validate_url".to_string(), builtin_validate_url);
    map.insert("validate_ip".to_string(), builtin_validate_ip);
    map.insert("validate_regex".to_string(), builtin_validate_regex);
    map.insert("validate_length".to_string(), builtin_validate_length);
    map.insert("validate_range".to_string(), builtin_validate_range);
    map.insert("validate_required".to_string(), builtin_validate_required);

    // 注册 OYTAPHP 助手函数
    register_oyta_helpers(map);
    
    // 注册 DateTime 函数
    datetime::register_datetime_functions(map);
    
    // 注册 mbstring 函数
    mbstring::register_mbstring_functions(map);
    
    // 注册 GD 函数
    gd::register_gd_functions(map);
    
    // 注册 XML 函数
    xml::register_xml_functions(map);
    
    // 注册 Phar 函数
    phar::register_phar_functions(map);
    
    // 注册 eval 函数
    eval::register_eval_functions(map);
}

/// isset — 检测变量是否已设置并且非 null
pub fn builtin_isset(args: &[crate::interpreter::value::Value]) -> anyhow::Result<crate::interpreter::value::Value> {
    use crate::interpreter::value::Value;
    Ok(Value::Bool(
        !args.first().map(|v| v.is_null()).unwrap_or(true),
    ))
}

/// unset — 释放给定的变量
pub fn builtin_unset(args: &[crate::interpreter::value::Value]) -> anyhow::Result<crate::interpreter::value::Value> {
    use crate::interpreter::value::Value;
    let _ = args;
    Ok(Value::Null)
}

/// gettype — 获取变量的类型
pub fn builtin_gettype(args: &[crate::interpreter::value::Value]) -> anyhow::Result<crate::interpreter::value::Value> {
    use crate::interpreter::value::Value;
    Ok(Value::String(
        args.first()
            .map(|v| v.type_name().to_string())
            .unwrap_or("NULL".to_string()),
    ))
}

/// settype — 设置变量的类型
pub fn builtin_settype(args: &[crate::interpreter::value::Value]) -> anyhow::Result<crate::interpreter::value::Value> {
    use crate::interpreter::value::Value;
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    let type_name = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    let _value = args.first();
    match type_name.as_str() {
        "int" | "integer" => Ok(Value::Bool(true)),
        "float" | "double" => Ok(Value::Bool(true)),
        "string" => Ok(Value::Bool(true)),
        "bool" | "boolean" => Ok(Value::Bool(true)),
        "array" => Ok(Value::Bool(true)),
        "null" => Ok(Value::Bool(true)),
        _ => Ok(Value::Bool(false)),
    }
}
