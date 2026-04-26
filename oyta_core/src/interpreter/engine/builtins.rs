//! 内置 PHP 函数模块（基础部分）
//!
//! 实现 PHP 内置函数，覆盖：
//! - 字符串函数：strlen/strtolower/strtoupper/trim/substr/str_replace 等
//! - 编码/解码函数：json_encode/json_decode/base64_encode/base64_decode/urlencode/urldecode
//! - 数组函数：count/array_keys/array_merge/array_push/array_map 等
//!
//! 扩展函数见 builtins_ext 模块

use anyhow::Result;
use std::collections::HashMap;

use crate::interpreter::value::Value;
use super::binary_op::values_equal;
use super::builtins_ext;

/// 内置函数类型定义
///
/// 每个内置函数接收参数切片，返回 Result<Value>
/// 函数签名：fn(&[Value]) -> Result<Value>
pub type BuiltinFunction = fn(&[Value]) -> Result<Value>;

/// 创建内置函数映射表
///
/// 注册所有内置 PHP 函数到 HashMap 中
/// 在 Interpreter::new() 中调用，初始化解释器的内置函数表
///
/// # 返回
/// 函数名 → 函数指针的映射
pub fn create_builtins_map() -> HashMap<String, BuiltinFunction> {
    let mut map: HashMap<String, BuiltinFunction> = HashMap::new();

    map.insert("strlen".to_string(), builtin_strlen);
    map.insert("strtolower".to_string(), builtin_strtolower);
    map.insert("strtoupper".to_string(), builtin_strtoupper);
    map.insert("trim".to_string(), builtin_trim);
    map.insert("ltrim".to_string(), builtin_ltrim);
    map.insert("rtrim".to_string(), builtin_rtrim);
    map.insert("substr".to_string(), builtin_substr);
    map.insert("str_replace".to_string(), builtin_str_replace);
    map.insert("str_contains".to_string(), builtin_str_contains);
    map.insert("str_starts_with".to_string(), builtin_str_starts_with);
    map.insert("str_ends_with".to_string(), builtin_str_ends_with);
    map.insert("strpos".to_string(), builtin_strpos);
    map.insert("str_repeat".to_string(), builtin_str_repeat);
    map.insert("str_pad".to_string(), builtin_str_pad);
    map.insert("explode".to_string(), builtin_explode);
    map.insert("implode".to_string(), builtin_implode);
    map.insert("json_encode".to_string(), builtin_json_encode);
    map.insert("json_decode".to_string(), builtin_json_decode);
    map.insert("count".to_string(), builtin_count);
    map.insert("array_keys".to_string(), builtin_array_keys);
    map.insert("array_values".to_string(), builtin_array_values);
    map.insert("array_merge".to_string(), builtin_array_merge);
    map.insert("array_push".to_string(), builtin_array_push);
    map.insert("array_pop".to_string(), builtin_array_pop);
    map.insert("array_map".to_string(), builtin_array_map);
    map.insert("array_filter".to_string(), builtin_array_filter);
    map.insert("in_array".to_string(), builtin_in_array);
    map.insert("array_key_exists".to_string(), builtin_array_key_exists);
    map.insert("array_slice".to_string(), builtin_array_slice);
    map.insert("array_splice".to_string(), builtin_array_splice);
    map.insert("array_reverse".to_string(), builtin_array_reverse);
    map.insert("sort".to_string(), builtin_sort);
    map.insert("usort".to_string(), builtin_usort);
    map.insert("range".to_string(), builtin_range);
    map.insert("array_column".to_string(), builtin_array_column);
    map.insert("array_unique".to_string(), builtin_array_unique);
    map.insert("array_search".to_string(), builtin_array_search);
    map.insert("compact".to_string(), builtin_compact);
    map.insert("extract".to_string(), builtin_extract);

    builtins_ext::register_ext_builtins(&mut map);
    builtins_ext::register_oyta_helpers(&mut map);

    map
}

// ===== 字符串函数 =====

/// strlen — 获取字符串长度
fn builtin_strlen(args: &[Value]) -> Result<Value> {
    Ok(Value::Int(
        args.first()
            .map(|v| v.to_string_value().len())
            .unwrap_or(0) as i64,
    ))
}

/// strtolower — 将字符串转换为小写
fn builtin_strtolower(args: &[Value]) -> Result<Value> {
    Ok(Value::String(
        args.first()
            .map(|v| v.to_string_value().to_lowercase())
            .unwrap_or_default(),
    ))
}

/// strtoupper — 将字符串转换为大写
fn builtin_strtoupper(args: &[Value]) -> Result<Value> {
    Ok(Value::String(
        args.first()
            .map(|v| v.to_string_value().to_uppercase())
            .unwrap_or_default(),
    ))
}

/// trim — 去除字符串首尾空白字符
fn builtin_trim(args: &[Value]) -> Result<Value> {
    Ok(Value::String(
        args.first()
            .map(|v| v.to_string_value().trim().to_string())
            .unwrap_or_default(),
    ))
}

/// ltrim — 去除字符串开头空白字符
fn builtin_ltrim(args: &[Value]) -> Result<Value> {
    Ok(Value::String(
        args.first()
            .map(|v| v.to_string_value().trim_start().to_string())
            .unwrap_or_default(),
    ))
}

/// rtrim — 去除字符串末尾空白字符
fn builtin_rtrim(args: &[Value]) -> Result<Value> {
    Ok(Value::String(
        args.first()
            .map(|v| v.to_string_value().trim_end().to_string())
            .unwrap_or_default(),
    ))
}

/// substr — 截取字符串的子串
/// substr(string $string, int $offset, ?int $length = null): string
fn builtin_substr(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let start = args.get(1).map(|v| v.to_int()).unwrap_or(0);
    if start < 0 {
        return Ok(Value::String(String::new()));
    }
    let len = args.get(2).map(|v| v.to_int());
    match len {
        Some(l) if l >= 0 => Ok(Value::String(
            s.chars().skip(start as usize).take(l as usize).collect(),
        )),
        _ => Ok(Value::String(s.chars().skip(start as usize).collect())),
    }
}

/// str_replace — 字符串替换
/// str_replace(string $search, string $replace, string $subject): string
fn builtin_str_replace(args: &[Value]) -> Result<Value> {
    let search = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let replace = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    let subject = args.get(2).map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::String(subject.replace(&search, &replace)))
}

/// str_contains — 判断字符串是否包含子串
fn builtin_str_contains(args: &[Value]) -> Result<Value> {
    let haystack = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let needle = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::Bool(haystack.contains(&needle)))
}

/// str_starts_with — 判断字符串是否以指定前缀开头
fn builtin_str_starts_with(args: &[Value]) -> Result<Value> {
    let haystack = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let needle = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::Bool(haystack.starts_with(&needle)))
}

/// str_ends_with — 判断字符串是否以指定后缀结尾
fn builtin_str_ends_with(args: &[Value]) -> Result<Value> {
    let haystack = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let needle = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::Bool(haystack.ends_with(&needle)))
}

/// strpos — 查找子串首次出现的位置
fn builtin_strpos(args: &[Value]) -> Result<Value> {
    let haystack = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let needle = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::Int(
        haystack.find(&needle).map(|i| i as i64).unwrap_or(-1),
    ))
}

/// str_repeat — 重复字符串
fn builtin_str_repeat(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let times = args.get(1).map(|v| v.to_int()).unwrap_or(0);
    Ok(Value::String(s.repeat(times as usize)))
}

/// str_pad — 使用另一个字符串填充字符串为指定长度
fn builtin_str_pad(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let len = args.get(1).map(|v| v.to_int()).unwrap_or(0) as usize;
    let pad = args
        .get(2)
        .map(|v| v.to_string_value())
        .unwrap_or_else(|| " ".to_string());
    let pad_char = pad.chars().next().unwrap_or(' ');
    let mut result = s.clone();
    while result.chars().count() < len {
        result.push(pad_char);
    }
    Ok(Value::String(result))
}

/// explode — 使用一个字符串分割另一个字符串
fn builtin_explode(args: &[Value]) -> Result<Value> {
    let delim = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let s = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    let limit = args.get(2).map(|v| v.to_int()).unwrap_or(-1);
    if limit > 0 {
        Ok(Value::IndexedArray(
            s.splitn(limit as usize, &delim)
                .map(|p| Value::String(p.to_string()))
                .collect(),
        ))
    } else {
        Ok(Value::IndexedArray(
            s.split(&delim)
                .map(|p| Value::String(p.to_string()))
                .collect(),
        ))
    }
}

/// implode — 使用一个字符串将数组元素连接为字符串
fn builtin_implode(args: &[Value]) -> Result<Value> {
    let glue = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    match args.get(1) {
        Some(Value::IndexedArray(items)) => Ok(Value::String(
            items
                .iter()
                .map(|v| v.to_string_value())
                .collect::<Vec<_>>()
                .join(&glue),
        )),
        _ => Ok(Value::String(String::new())),
    }
}

// ===== 编码/解码函数 =====

/// json_encode — 对变量进行 JSON 编码
fn builtin_json_encode(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(data) => Ok(Value::String(data.to_json_string())),
        None => Ok(Value::Bool(false)),
    }
}

/// json_decode — 对 JSON 格式的字符串进行解码
fn builtin_json_decode(args: &[Value]) -> Result<Value> {
    match serde_json::from_str::<Value>(
        &args.first()
            .map(|v| v.to_string_value())
            .unwrap_or_default(),
    ) {
        Ok(v) => Ok(v),
        Err(_) => Ok(Value::Null),
    }
}

/// base64_encode — 使用 MIME base64 对数据进行编码
fn builtin_base64_encode(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    use base64::Engine;
    Ok(Value::String(
        base64::engine::general_purpose::STANDARD.encode(s.as_bytes()),
    ))
}

/// base64_decode — 对使用 MIME base64 编码的数据进行解码
fn builtin_base64_decode(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    use base64::Engine;
    match base64::engine::general_purpose::STANDARD.decode(s) {
        Ok(bytes) => Ok(Value::String(String::from_utf8_lossy(&bytes).to_string())),
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// urlencode — 编码 URL 字符串
fn builtin_urlencode(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::String(urlencoding::encode(&s).to_string()))
}

/// urldecode — 解码已编码的 URL 字符串
fn builtin_urldecode(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let decoded = urlencoding::decode(&s).unwrap_or_else(|_| std::borrow::Cow::Borrowed(&s));
    Ok(Value::String(decoded.to_string()))
}

// ===== 数组函数 =====

/// count — 计算数组中的元素数目
fn builtin_count(args: &[Value]) -> Result<Value> {
    Ok(Value::Int(
        args.first().map(|v| v.count()).unwrap_or(0) as i64,
    ))
}

/// array_keys — 返回数组中所有的键名
fn builtin_array_keys(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::AssociativeArray(map)) => Ok(Value::IndexedArray(
            map.iter().map(|(k, _)| Value::String(k.clone())).collect(),
        )),
        Some(Value::IndexedArray(arr)) => Ok(Value::IndexedArray(
            (0..arr.len()).map(|i| Value::Int(i as i64)).collect(),
        )),
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// array_values — 返回数组中所有的值
fn builtin_array_values(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::AssociativeArray(map)) => {
            Ok(Value::IndexedArray(map.iter().map(|(_, v)| v.clone()).collect()))
        }
        Some(Value::IndexedArray(arr)) => Ok(Value::IndexedArray(arr.clone())),
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// array_merge — 合并一个或多个数组
fn builtin_array_merge(args: &[Value]) -> Result<Value> {
    let mut indexed = Vec::new();
    let mut assoc: Vec<(String, Value)> = Vec::new();
    let mut has_string_key = false;
    for arg in args {
        match arg {
            Value::IndexedArray(items) => {
                for v in items {
                    indexed.push(v.clone());
                }
            }
            Value::AssociativeArray(map) => {
                has_string_key = true;
                for (k, v) in map {
                    assoc.push((k.clone(), v.clone()));
                }
            }
            _ => {}
        }
    }
    if has_string_key {
        for v in indexed {
            assoc.push((v.to_string_value(), v));
        }
        Ok(Value::AssociativeArray(assoc))
    } else {
        Ok(Value::IndexedArray(indexed))
    }
}

/// array_push — 将一个或多个单元压入数组的末尾
fn builtin_array_push(args: &[Value]) -> Result<Value> {
    Ok(Value::Int(args.len() as i64))
}

/// array_pop — 弹出数组最后一个单元
fn builtin_array_pop(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(items)) => Ok(items.last().cloned().unwrap_or(Value::Null)),
        _ => Ok(Value::Null),
    }
}

/// array_map — 为数组的每个元素应用回调函数
/// array_map(?callable $callback, array $array, array ...$arrays): array
/// 当回调为 null 时，合并多个数组
fn builtin_array_map(args: &[Value]) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Null);
    let array = args.get(1).cloned().unwrap_or(Value::Null);

    // 如果回调为 null，合并多个数组
    if matches!(callback, Value::Null) {
        return builtin_array_merge(&args[1..]);
    }

    // 处理单个数组
    match &array {
        Value::IndexedArray(items) => {
            let mapped: Vec<Value> = items.iter().map(|item| {
                // 尝试调用回调函数
                match &callback {
                    Value::Callable(callable) => {
                        // 简化实现：对于内置函数名，直接调用
                        if let crate::interpreter::value::CallableValue::Function { name } = callable {
                            match name.as_str() {
                                "strlen" => Value::Int(item.to_string_value().len() as i64),
                                "strtolower" => Value::String(item.to_string_value().to_lowercase()),
                                "strtoupper" => Value::String(item.to_string_value().to_uppercase()),
                                "trim" => Value::String(item.to_string_value().trim().to_string()),
                                "intval" => Value::Int(item.to_int()),
                                "strval" => Value::String(item.to_string_value()),
                                "floatval" => Value::Float(item.to_float()),
                                _ => item.clone(),
                            }
                        } else {
                            item.clone()
                        }
                    }
                    Value::String(func_name) => {
                        match func_name.as_str() {
                            "strlen" => Value::Int(item.to_string_value().len() as i64),
                            "strtolower" => Value::String(item.to_string_value().to_lowercase()),
                            "strtoupper" => Value::String(item.to_string_value().to_uppercase()),
                            "trim" => Value::String(item.to_string_value().trim().to_string()),
                            "intval" => Value::Int(item.to_int()),
                            "strval" => Value::String(item.to_string_value()),
                            "floatval" => Value::Float(item.to_float()),
                            _ => item.clone(),
                        }
                    }
                    _ => item.clone(),
                }
            }).collect();
            Ok(Value::IndexedArray(mapped))
        }
        Value::AssociativeArray(pairs) => {
            let mapped: Vec<(String, Value)> = pairs.iter().map(|(key, item)| {
                let new_val = match &callback {
                    Value::Callable(callable) => {
                        if let crate::interpreter::value::CallableValue::Function { name } = callable {
                            match name.as_str() {
                                "strlen" => Value::Int(item.to_string_value().len() as i64),
                                "strtolower" => Value::String(item.to_string_value().to_lowercase()),
                                "strtoupper" => Value::String(item.to_string_value().to_uppercase()),
                                _ => item.clone(),
                            }
                        } else {
                            item.clone()
                        }
                    }
                    Value::String(func_name) => {
                        match func_name.as_str() {
                            "strlen" => Value::Int(item.to_string_value().len() as i64),
                            "strtolower" => Value::String(item.to_string_value().to_lowercase()),
                            "strtoupper" => Value::String(item.to_string_value().to_uppercase()),
                            _ => item.clone(),
                        }
                    }
                    _ => item.clone(),
                };
                (key.clone(), new_val)
            }).collect();
            Ok(Value::AssociativeArray(mapped))
        }
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// array_filter — 使用回调函数过滤数组的元素
/// array_filter(array $array, ?callable $callback = null, int $mode = 0): array
/// 当回调为 null 时，过滤掉等价为 false 的值
fn builtin_array_filter(args: &[Value]) -> Result<Value> {
    let array = args.first().cloned().unwrap_or(Value::Null);
    let callback = args.get(1).cloned().unwrap_or(Value::Null);

    match &array {
        Value::IndexedArray(items) => {
            let filtered: Vec<Value> = if matches!(callback, Value::Null) {
                // 无回调：过滤掉 falsy 值
                items.iter().filter(|v| v.is_truthy()).cloned().collect()
            } else {
                // 有回调：保留回调返回 true 的元素
                items.iter().filter(|item| {
                    match &callback {
                        Value::String(func_name) => {
                            match func_name.as_str() {
                                "is_null" => !matches!(item, Value::Null),
                                "is_string" => matches!(item, Value::String(_)),
                                "is_int" | "is_integer" => matches!(item, Value::Int(_)),
                                "is_array" => matches!(item, Value::IndexedArray(_) | Value::AssociativeArray(_)),
                                "is_numeric" => {
                                    match item {
                                        Value::Int(_) | Value::Float(_) => true,
                                        Value::String(s) => s.parse::<f64>().is_ok(),
                                        _ => false,
                                    }
                                }
                                _ => true,
                            }
                        }
                        _ => true,
                    }
                }).cloned().collect()
            };
            Ok(Value::IndexedArray(filtered))
        }
        Value::AssociativeArray(pairs) => {
            let filtered: Vec<(String, Value)> = if matches!(callback, Value::Null) {
                pairs.iter().filter(|(_, v)| v.is_truthy()).cloned().collect()
            } else {
                pairs.iter().filter(|(_, item)| {
                    match &callback {
                        Value::String(func_name) => {
                            match func_name.as_str() {
                                "is_null" => !matches!(item, Value::Null),
                                "is_string" => matches!(item, Value::String(_)),
                                "is_int" | "is_integer" => matches!(item, Value::Int(_)),
                                "is_array" => matches!(item, Value::IndexedArray(_) | Value::AssociativeArray(_)),
                                _ => true,
                            }
                        }
                        _ => true,
                    }
                }).cloned().collect()
            };
            Ok(Value::AssociativeArray(filtered))
        }
        _ => Ok(args.first().cloned().unwrap_or(Value::Null)),
    }
}

/// in_array — 检查数组中是否存在某个值
fn builtin_in_array(args: &[Value]) -> Result<Value> {
    let needle = args.first().cloned().unwrap_or(Value::Null);
    match args.get(1) {
        Some(Value::IndexedArray(arr)) => {
            Ok(Value::Bool(arr.iter().any(|v| values_equal(v, &needle))))
        }
        Some(Value::AssociativeArray(map)) => {
            Ok(Value::Bool(map.iter().any(|(_, v)| values_equal(v, &needle))))
        }
        _ => Ok(Value::Bool(false)),
    }
}

/// array_key_exists — 检查数组里是否有指定的键名或索引
fn builtin_array_key_exists(args: &[Value]) -> Result<Value> {
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    match args.get(1) {
        Some(Value::AssociativeArray(map)) => Ok(Value::Bool(map.iter().any(|(k, _)| k == &key))),
        Some(Value::IndexedArray(arr)) => {
            let idx = key.parse::<usize>().unwrap_or(usize::MAX);
            Ok(Value::Bool(idx < arr.len()))
        }
        _ => Ok(Value::Bool(false)),
    }
}

/// array_slice — 从数组中取出一段
fn builtin_array_slice(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(items)) => {
            let offset = args.get(1).map(|v| v.to_int()).unwrap_or(0) as usize;
            let len = args.get(2).map(|v| v.to_int());
            match len {
                Some(l) => Ok(Value::IndexedArray(
                    items.iter().skip(offset).take(l as usize).cloned().collect(),
                )),
                None => Ok(Value::IndexedArray(items.iter().skip(offset).cloned().collect())),
            }
        }
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// array_splice — 把数组中的一部分去掉并用其它值取代
/// array_splice(array &$array, int $offset, ?int $length = null, mixed $replacement = []): array
fn builtin_array_splice(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(items)) => {
            let offset = args.get(1).map(|v| v.to_int()).unwrap_or(0);
            let offset_usize = if offset < 0 {
                (items.len() as i64 + offset).max(0) as usize
            } else {
                offset as usize
            };
            let length = args.get(2).map(|v| v.to_int() as usize).unwrap_or(items.len());
            let _replacement = args.get(3).cloned().unwrap_or(Value::IndexedArray(Vec::new()));

            // 提取被移除的元素
            let removed: Vec<Value> = items.iter()
                .skip(offset_usize)
                .take(length)
                .cloned()
                .collect();

            Ok(Value::IndexedArray(removed))
        }
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// array_reverse — 返回单元顺序相反的数组
fn builtin_array_reverse(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(items)) => {
            Ok(Value::IndexedArray(items.iter().rev().cloned().collect()))
        }
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// sort — 对数组排序
/// sort(array &$array, int $flags = SORT_REGULAR): bool
/// 支持数值排序和字符串排序
fn builtin_sort(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(items)) => {
            let mut sorted = items.clone();
            let flags = args.get(1).map(|v| v.to_int()).unwrap_or(0);

            match flags {
                1 => {
                    // SORT_NUMERIC
                    sorted.sort_by(|a, b| {
                        a.to_float().partial_cmp(&b.to_float()).unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
                2 => {
                    // SORT_STRING
                    sorted.sort_by(|a, b| {
                        a.to_string_value().cmp(&b.to_string_value())
                    });
                }
                _ => {
                    // SORT_REGULAR
                    sorted.sort_by(|a, b| {
                        compare_values(a, b)
                    });
                }
            }

            Ok(Value::IndexedArray(sorted))
        }
        Some(Value::AssociativeArray(pairs)) => {
            let mut sorted = pairs.clone();
            sorted.sort_by(|a, b| compare_values(&a.1, &b.1));
            Ok(Value::IndexedArray(sorted.into_iter().map(|(_, v)| v).collect()))
        }
        _ => Ok(Value::Bool(true)),
    }
}

/// usort — 使用用户自定义的比较函数对数组排序
/// usort(array &$array, callable $callback): bool
fn builtin_usort(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(items)) => {
            let mut sorted = items.clone();
            // 简化实现：按默认排序
            // 实际的回调比较需要通过解释器执行
            sorted.sort_by(|a, b| compare_values(a, b));
            Ok(Value::IndexedArray(sorted))
        }
        _ => Ok(Value::Bool(true)),
    }
}

/// 比较两个 Value 的大小
fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => a.cmp(b),
        (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
        (Value::String(a), Value::String(b)) => a.cmp(b),
        (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)).unwrap_or(std::cmp::Ordering::Equal),
        _ => a.to_string_value().cmp(&b.to_string_value()),
    }
}

/// range — 根据范围创建数组
/// range(mixed $start, mixed $end, int|float $step = 1): array
fn builtin_range(args: &[Value]) -> Result<Value> {
    let start = args.first().map(|v| v.to_int()).unwrap_or(0);
    let end = args.get(1).map(|v| v.to_int()).unwrap_or(0);
    let step = args.get(2).map(|v| v.to_int()).unwrap_or(1);
    let mut result = Vec::new();
    if start <= end {
        let mut i = start;
        while i <= end {
            result.push(Value::Int(i));
            i += step.max(1);
        }
    } else {
        let mut i = start;
        while i >= end {
            result.push(Value::Int(i));
            i -= step.max(1);
        }
    }
    Ok(Value::IndexedArray(result))
}

/// array_column — 返回数组中指定的一列
fn builtin_array_column(args: &[Value]) -> Result<Value> {
    let col_key = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    match args.first() {
        Some(Value::IndexedArray(items)) => {
            let result: Vec<Value> = items
                .iter()
                .filter_map(|item| match item {
                    Value::AssociativeArray(map) => map
                        .iter()
                        .find(|(k, _)| k == &col_key)
                        .map(|(_, v)| v.clone()),
                    Value::Object(obj) => obj.properties.get(&col_key).cloned(),
                    _ => None,
                })
                .collect();
            Ok(Value::IndexedArray(result))
        }
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// array_unique — 移除数组中重复的值
fn builtin_array_unique(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(items)) => {
            let mut seen = Vec::new();
            let mut result = Vec::new();
            for item in items {
                if !seen.iter().any(|v: &Value| values_equal(v, item)) {
                    seen.push(item.clone());
                    result.push(item.clone());
                }
            }
            Ok(Value::IndexedArray(result))
        }
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// array_search — 在数组中搜索给定的值，返回对应的键名
fn builtin_array_search(args: &[Value]) -> Result<Value> {
    let needle = args.first().cloned().unwrap_or(Value::Null);
    match args.get(1) {
        Some(Value::IndexedArray(items)) => {
            for (i, v) in items.iter().enumerate() {
                if values_equal(v, &needle) {
                    return Ok(Value::Int(i as i64));
                }
            }
            Ok(Value::Bool(false))
        }
        Some(Value::AssociativeArray(map)) => {
            for (k, v) in map {
                if values_equal(v, &needle) {
                    return Ok(Value::String(k.clone()));
                }
            }
            Ok(Value::Bool(false))
        }
        _ => Ok(Value::Bool(false)),
    }
}

/// compact — 建立一个数组，包括变量名和它们的值
/// compact(string|array $var_name, string|array ...$var_names): array
fn builtin_compact(args: &[Value]) -> Result<Value> {
    let mut result = Vec::new();
    for arg in args {
        match arg {
            Value::String(name) => {
                // 变量名作为键，值为 null（实际值需要从执行上下文获取）
                result.push((name.clone(), Value::Null));
            }
            Value::IndexedArray(names) => {
                for name in names {
                    let key = name.to_string_value();
                    result.push((key, Value::Null));
                }
            }
            _ => {}
        }
    }
    Ok(Value::AssociativeArray(result))
}

/// extract — 从数组中将变量导入到当前的符号表
/// extract(array &$array, int $flags = EXTR_OVERWRITE, string $prefix = ""): int
fn builtin_extract(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::AssociativeArray(pairs)) => {
            // 返回导入的变量数量
            // 实际的变量导入需要在解释器执行上下文中处理
            Ok(Value::Int(pairs.len() as i64))
        }
        _ => Ok(Value::Int(0)),
    }
}
