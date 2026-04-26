//! OYTAPHP 助手函数模块
//!
//! 包含所有以 oyta_ 前缀命名的助手函数

use std::collections::HashMap;

use super::super::builtins::BuiltinFunction;
use super::class_object::{
    builtin_call_user_func, builtin_call_user_func_array, builtin_class_exists,
    builtin_func_num_args, builtin_get_class, builtin_method_exists, builtin_property_exists,
};
use super::debug::{builtin_dd, builtin_dump, builtin_json};
use super::hash::{builtin_md5, builtin_sha1};
use super::math::{builtin_abs, builtin_ceil, builtin_floor, builtin_max, builtin_min, builtin_round};
use super::string_ext::{
    builtin_htmlspecialchars, builtin_lcfirst, builtin_nl2br, builtin_number_format,
    builtin_sprintf, builtin_strip_tags, builtin_ucfirst, builtin_ucwords,
};
use super::time::{builtin_date, builtin_microtime, builtin_strtotime, builtin_time};
use super::type_cast::{builtin_boolval, builtin_floatval, builtin_intval, builtin_strval};
use super::type_check::{
    builtin_is_array, builtin_is_bool, builtin_is_float, builtin_is_int, builtin_is_null,
    builtin_is_numeric, builtin_is_object, builtin_is_string,
};

/// 注册 OYTAPHP 专用助手函数
/// 所有函数以 oyta_ 前缀命名，避免与其他框架冲突
pub fn register_oyta_helpers(map: &mut HashMap<String, BuiltinFunction>) {
    // 调试助手函数
    map.insert("oyta_dd".to_string(), builtin_dd);
    map.insert("oyta_dump".to_string(), builtin_dump);
    map.insert("oyta_json".to_string(), builtin_json);

    // 哈希助手函数
    map.insert("oyta_md5".to_string(), builtin_md5);
    map.insert("oyta_sha1".to_string(), builtin_sha1);

    // 时间助手函数
    map.insert("oyta_time".to_string(), builtin_time);
    map.insert("oyta_date".to_string(), builtin_date);
    map.insert("oyta_strtotime".to_string(), builtin_strtotime);
    map.insert("oyta_microtime".to_string(), builtin_microtime);

    // 类型检测助手函数
    map.insert("oyta_is_null".to_string(), builtin_is_null);
    map.insert("oyta_is_array".to_string(), builtin_is_array);
    map.insert("oyta_is_string".to_string(), builtin_is_string);
    map.insert("oyta_is_int".to_string(), builtin_is_int);
    map.insert("oyta_is_float".to_string(), builtin_is_float);
    map.insert("oyta_is_bool".to_string(), builtin_is_bool);
    map.insert("oyta_is_object".to_string(), builtin_is_object);
    map.insert("oyta_is_numeric".to_string(), builtin_is_numeric);

    // 类型转换助手函数
    map.insert("oyta_intval".to_string(), builtin_intval);
    map.insert("oyta_floatval".to_string(), builtin_floatval);
    map.insert("oyta_strval".to_string(), builtin_strval);
    map.insert("oyta_boolval".to_string(), builtin_boolval);

    // 数学助手函数
    map.insert("oyta_abs".to_string(), builtin_abs);
    map.insert("oyta_max".to_string(), builtin_max);
    map.insert("oyta_min".to_string(), builtin_min);
    map.insert("oyta_floor".to_string(), builtin_floor);
    map.insert("oyta_ceil".to_string(), builtin_ceil);
    map.insert("oyta_round".to_string(), builtin_round);

    // 字符串助手函数
    map.insert("oyta_sprintf".to_string(), builtin_sprintf);
    map.insert("oyta_number_format".to_string(), builtin_number_format);
    map.insert("oyta_htmlspecialchars".to_string(), builtin_htmlspecialchars);
    map.insert("oyta_strip_tags".to_string(), builtin_strip_tags);
    map.insert("oyta_ucfirst".to_string(), builtin_ucfirst);
    map.insert("oyta_lcfirst".to_string(), builtin_lcfirst);
    map.insert("oyta_ucwords".to_string(), builtin_ucwords);
    map.insert("oyta_nl2br".to_string(), builtin_nl2br);

    // 类/对象助手函数
    map.insert("oyta_class_exists".to_string(), builtin_class_exists);
    map.insert("oyta_method_exists".to_string(), builtin_method_exists);
    map.insert("oyta_property_exists".to_string(), builtin_property_exists);
    map.insert("oyta_get_class".to_string(), builtin_get_class);
    map.insert("oyta_call_user_func".to_string(), builtin_call_user_func);
    map.insert("oyta_call_user_func_array".to_string(), builtin_call_user_func_array);
    map.insert("oyta_func_num_args".to_string(), builtin_func_num_args);
}
