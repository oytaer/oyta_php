//! 数组函数模块
//!
//! 包含 count、array_keys、array_merge、array_push、array_map 等数组处理函数

use std::collections::HashMap;

use anyhow::Result;

use crate::interpreter::value::Value;

use super::types::BuiltinFunction;

/// 注册数组函数到映射表
///
/// # 参数
/// - `map`: 内置函数映射表的可变引用
pub fn register_array_functions(map: &mut HashMap<String, BuiltinFunction>) {
    // 数组计数
    map.insert("count".to_string(), builtin_count);
    map.insert("sizeof".to_string(), builtin_count);
    // 获取键名
    map.insert("array_keys".to_string(), builtin_array_keys);
    // 获取值
    map.insert("array_values".to_string(), builtin_array_values);
    // 合并数组
    map.insert("array_merge".to_string(), builtin_array_merge);
    // 末尾添加
    map.insert("array_push".to_string(), builtin_array_push);
    // 末尾弹出
    map.insert("array_pop".to_string(), builtin_array_pop);
    // 开头添加
    map.insert("array_unshift".to_string(), builtin_array_unshift);
    // 开头弹出
    map.insert("array_shift".to_string(), builtin_array_shift);
    // 映射
    map.insert("array_map".to_string(), builtin_array_map);
    // 过滤
    map.insert("array_filter".to_string(), builtin_array_filter);
    // 检查值是否存在
    map.insert("in_array".to_string(), builtin_in_array);
    // 检查键是否存在
    map.insert("array_key_exists".to_string(), builtin_array_key_exists);
    // 切片
    map.insert("array_slice".to_string(), builtin_array_slice);
    // 替换部分
    map.insert("array_splice".to_string(), builtin_array_splice);
    // 反转
    map.insert("array_reverse".to_string(), builtin_array_reverse);
    // 排序
    map.insert("sort".to_string(), builtin_sort);
    // 自定义排序
    map.insert("usort".to_string(), builtin_usort);
    // 范围
    map.insert("range".to_string(), builtin_range);
    // 提取列
    map.insert("array_column".to_string(), builtin_array_column);
    // 去重
    map.insert("array_unique".to_string(), builtin_array_unique);
    // 搜索
    map.insert("array_search".to_string(), builtin_array_search);
    // 填充
    map.insert("array_fill".to_string(), builtin_array_fill);
    // 合并为键值对
    map.insert("array_combine".to_string(), builtin_array_combine);
    // 翻转键值
    map.insert("array_flip".to_string(), builtin_array_flip);
    // 求和
    map.insert("array_sum".to_string(), builtin_array_sum);
    // 求积
    map.insert("array_product".to_string(), builtin_array_product);
    // 随机取值
    map.insert("array_rand".to_string(), builtin_array_rand);
    // 打乱
    map.insert("shuffle".to_string(), builtin_shuffle);
}

/// count — 计算数组中的单元数目，或对象中的属性个数
pub fn builtin_count(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(arr)) => Ok(Value::Int(arr.len() as i64)),
        Some(Value::AssociativeArray(map)) => Ok(Value::Int(map.len() as i64)),
        Some(Value::Object(obj)) => Ok(Value::Int(obj.properties.len() as i64)),
        Some(Value::String(s)) => Ok(Value::Int(s.len() as i64)),
        Some(_) => Ok(Value::Int(1)),
        None => Ok(Value::Int(0)),
    }
}

/// array_keys — 返回数组中部分的或所有的键名
pub fn builtin_array_keys(args: &[Value]) -> Result<Value> {
    let array = args.first();
    let search_value = args.get(1);

    match array {
        Some(Value::IndexedArray(arr)) => {
            let keys: Vec<Value> = if let Some(search) = search_value {
                // 返回匹配值的键
                arr.iter()
                    .enumerate()
                    .filter(|(_, v)| values_equal(v, search))
                    .map(|(i, _)| Value::Int(i as i64))
                    .collect()
            } else {
                // 返回所有键
                (0..arr.len()).map(|i| Value::Int(i as i64)).collect()
            };
            Ok(Value::IndexedArray(keys))
        }
        Some(Value::AssociativeArray(map)) => {
            let keys: Vec<Value> = if let Some(search) = search_value {
                // 返回匹配值的键
                map.iter()
                    .filter(|(_, v)| values_equal(v, search))
                    .map(|(k, _)| Value::String(k.clone()))
                    .collect()
            } else {
                // 返回所有键
                map.iter().map(|(k, _)| Value::String(k.clone())).collect()
            };
            Ok(Value::IndexedArray(keys))
        }
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// array_values — 返回数组中所有的值
pub fn builtin_array_values(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(arr)) => Ok(Value::IndexedArray(arr.clone())),
        Some(Value::AssociativeArray(map)) => {
            let values: Vec<Value> = map.iter().map(|(_, v)| v.clone()).collect();
            Ok(Value::IndexedArray(values))
        }
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// array_merge — 合并一个或多个数组
pub fn builtin_array_merge(args: &[Value]) -> Result<Value> {
    let mut result = Vec::new();

    for arg in args {
        match arg {
            Value::IndexedArray(arr) => result.extend(arr.clone()),
            Value::AssociativeArray(map) => {
                // 关联数组合并时保留键名
                for (_, v) in map {
                    result.push(v.clone());
                }
            }
            _ => {}
        }
    }

    Ok(Value::IndexedArray(result))
}

/// array_push — 将一个或多个单元压入数组的末尾（入栈）
pub fn builtin_array_push(args: &[Value]) -> Result<Value> {
    // 注意：PHP 的 array_push 是修改原数组的
    // 这里简化为返回新数组
    let array = args.first();
    let new_elements = &args[1..];

    match array {
        Some(Value::IndexedArray(arr)) => {
            let mut result = arr.clone();
            result.extend(new_elements.to_vec());
            Ok(Value::Int(result.len() as i64))
        }
        Some(Value::AssociativeArray(map)) => {
            let mut result = map.clone();
            for elem in new_elements {
                // 使用数字键
                let key = result.len().to_string();
                result.push((key, elem.clone()));
            }
            Ok(Value::Int(result.len() as i64))
        }
        _ => Ok(Value::Int(0)),
    }
}

/// array_pop — 弹出数组最后一个单元（出栈）
pub fn builtin_array_pop(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(arr)) => {
            if let Some(last) = arr.last() {
                Ok(last.clone())
            } else {
                Ok(Value::Null)
            }
        }
        Some(Value::AssociativeArray(map)) => {
            if let Some((_, last)) = map.last() {
                Ok(last.clone())
            } else {
                Ok(Value::Null)
            }
        }
        _ => Ok(Value::Null),
    }
}

/// array_unshift — 在数组开头插入一个或多个单元
pub fn builtin_array_unshift(args: &[Value]) -> Result<Value> {
    let array = args.first();
    let new_elements = &args[1..];

    match array {
        Some(Value::IndexedArray(arr)) => {
            let mut result = new_elements.to_vec();
            result.extend(arr.clone());
            Ok(Value::Int(result.len() as i64))
        }
        _ => Ok(Value::Int(0)),
    }
}

/// array_shift — 将数组开头的单元移出数组
pub fn builtin_array_shift(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(arr)) => {
            if let Some(first) = arr.first() {
                Ok(first.clone())
            } else {
                Ok(Value::Null)
            }
        }
        _ => Ok(Value::Null),
    }
}

/// array_map — 为数组的每个元素应用回调函数
pub fn builtin_array_map(args: &[Value]) -> Result<Value> {
    // 简化实现：只处理第一个数组
    let array = args.get(1);

    match array {
        Some(Value::IndexedArray(arr)) => {
            // 简化：返回原数组的副本
            // 实际应该调用回调函数处理每个元素
            Ok(Value::IndexedArray(arr.clone()))
        }
        Some(Value::AssociativeArray(map)) => {
            Ok(Value::AssociativeArray(map.clone()))
        }
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// array_filter — 用回调函数过滤数组中的单元
pub fn builtin_array_filter(args: &[Value]) -> Result<Value> {
    let array = args.first();

    match array {
        Some(Value::IndexedArray(arr)) => {
            // 过滤掉 falsy 值
            let filtered: Vec<Value> = arr.iter().filter(|v| v.is_truthy()).cloned().collect();
            Ok(Value::IndexedArray(filtered))
        }
        Some(Value::AssociativeArray(map)) => {
            let filtered: Vec<(String, Value)> = map
                .iter()
                .filter(|(_, v)| v.is_truthy())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            Ok(Value::AssociativeArray(filtered))
        }
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// in_array — 检查数组中是否存在某个值
pub fn builtin_in_array(args: &[Value]) -> Result<Value> {
    let needle = args.first();
    let haystack = args.get(1);
    let strict = args.get(2).map(|v| v.is_truthy()).unwrap_or(false);

    match haystack {
        Some(Value::IndexedArray(arr)) => {
            let found = arr.iter().any(|v| {
                if strict {
                    values_equal_strict(v, needle.unwrap_or(&Value::Null))
                } else {
                    values_equal(v, needle.unwrap_or(&Value::Null))
                }
            });
            Ok(Value::Bool(found))
        }
        Some(Value::AssociativeArray(map)) => {
            let found = map.iter().any(|(_, v)| {
                if strict {
                    values_equal_strict(v, needle.unwrap_or(&Value::Null))
                } else {
                    values_equal(v, needle.unwrap_or(&Value::Null))
                }
            });
            Ok(Value::Bool(found))
        }
        _ => Ok(Value::Bool(false)),
    }
}

/// array_key_exists — 检查数组里是否有指定的键名或索引
pub fn builtin_array_key_exists(args: &[Value]) -> Result<Value> {
    let key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let array = args.get(1);

    match array {
        Some(Value::IndexedArray(arr)) => {
            // 尝试将 key 解析为索引
            if let Ok(index) = key.parse::<usize>() {
                Ok(Value::Bool(index < arr.len()))
            } else {
                Ok(Value::Bool(false))
            }
        }
        Some(Value::AssociativeArray(map)) => {
            let exists = map.iter().any(|(k, _)| k == &key);
            Ok(Value::Bool(exists))
        }
        _ => Ok(Value::Bool(false)),
    }
}

/// array_slice — 从数组中取出一段
pub fn builtin_array_slice(args: &[Value]) -> Result<Value> {
    let array = args.first();
    let offset = args.get(1).map(|v| v.to_int()).unwrap_or(0);
    let length = args.get(2).map(|v| v.to_int());

    match array {
        Some(Value::IndexedArray(arr)) => {
            let start = if offset < 0 {
                (arr.len() as i64 + offset).max(0) as usize
            } else {
                offset.min(arr.len() as i64) as usize
            };

            let end = match length {
                Some(len) if len > 0 => (start + len as usize).min(arr.len()),
                Some(len) if len < 0 => {
                    (arr.len() as i64 + len).max(start as i64) as usize
                }
                _ => arr.len(),
            };

            let slice = arr[start..end].to_vec();
            Ok(Value::IndexedArray(slice))
        }
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// array_splice — 去掉数组中的某一部分并用其它值取代
pub fn builtin_array_splice(args: &[Value]) -> Result<Value> {
    // 简化实现：返回被移除的部分
    let array = args.first();
    let offset = args.get(1).map(|v| v.to_int()).unwrap_or(0);
    let length = args.get(2).map(|v| v.to_int());

    match array {
        Some(Value::IndexedArray(arr)) => {
            let start = if offset < 0 {
                (arr.len() as i64 + offset).max(0) as usize
            } else {
                offset.min(arr.len() as i64) as usize
            };

            let end = match length {
                Some(len) if len > 0 => (start + len as usize).min(arr.len()),
                Some(len) if len < 0 => {
                    (arr.len() as i64 + len).max(start as i64) as usize
                }
                _ => arr.len(),
            };

            let removed = arr[start..end].to_vec();
            Ok(Value::IndexedArray(removed))
        }
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// array_reverse — 返回单元顺序相反的数组
pub fn builtin_array_reverse(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(arr)) => {
            Ok(Value::IndexedArray(arr.iter().rev().cloned().collect()))
        }
        Some(Value::AssociativeArray(map)) => {
            Ok(Value::AssociativeArray(map.iter().rev().map(|(k, v)| (k.clone(), v.clone())).collect()))
        }
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// sort — 对数组排序
pub fn builtin_sort(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(arr)) => {
            let mut sorted = arr.clone();
            sorted.sort_by(|a, b| {
                let a_str = a.to_string_value();
                let b_str = b.to_string_value();
                a_str.cmp(&b_str)
            });
            Ok(Value::Bool(true))
        }
        _ => Ok(Value::Bool(false)),
    }
}

/// usort — 使用用户自定义的比较函数对数组中的值进行排序
pub fn builtin_usort(args: &[Value]) -> Result<Value> {
    // 简化实现：使用默认排序
    builtin_sort(args)
}

/// range — 根据范围创建数组，包含指定的元素
pub fn builtin_range(args: &[Value]) -> Result<Value> {
    let start = args.first().map(|v| v.to_int()).unwrap_or(0);
    let end = args.get(1).map(|v| v.to_int()).unwrap_or(0);
    let step = args.get(2).map(|v| v.to_int()).unwrap_or(1);

    let mut result = Vec::new();

    if step == 0 {
        return Ok(Value::IndexedArray(result));
    }

    if start <= end && step > 0 {
        let mut i = start;
        while i <= end {
            result.push(Value::Int(i));
            i += step;
        }
    } else if start >= end && step < 0 {
        let mut i = start;
        while i >= end {
            result.push(Value::Int(i));
            i += step;
        }
    }

    Ok(Value::IndexedArray(result))
}

/// array_column — 返回数组中指定的一列
pub fn builtin_array_column(args: &[Value]) -> Result<Value> {
    let column_key = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let array = args.get(1);

    match array {
        Some(Value::IndexedArray(arr)) => {
            let column: Vec<Value> = arr
                .iter()
                .filter_map(|row| match row {
                    Value::AssociativeArray(map) => {
                        map.iter().find(|(k, _)| k == &column_key).map(|(_, v)| v.clone())
                    }
                    Value::Object(obj) => obj.properties.get(&column_key).cloned(),
                    _ => None,
                })
                .collect();
            Ok(Value::IndexedArray(column))
        }
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// array_unique — 移除数组中重复的值
pub fn builtin_array_unique(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(arr)) => {
            let mut seen = std::collections::HashSet::new();
            let unique: Vec<Value> = arr
                .iter()
                .filter(|v| {
                    let key = v.to_string_value();
                    seen.insert(key)
                })
                .cloned()
                .collect();
            Ok(Value::IndexedArray(unique))
        }
        Some(Value::AssociativeArray(map)) => {
            let mut seen = std::collections::HashSet::new();
            let unique: Vec<(String, Value)> = map
                .iter()
                .filter(|(_, v)| {
                    let key = v.to_string_value();
                    seen.insert(key)
                })
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            Ok(Value::AssociativeArray(unique))
        }
        _ => Ok(Value::IndexedArray(Vec::new())),
    }
}

/// array_search — 在数组中搜索给定的值，如果成功则返回首个相应的键名
pub fn builtin_array_search(args: &[Value]) -> Result<Value> {
    let needle = args.first();
    let haystack = args.get(1);

    match haystack {
        Some(Value::IndexedArray(arr)) => {
            for (i, v) in arr.iter().enumerate() {
                if values_equal(v, needle.unwrap_or(&Value::Null)) {
                    return Ok(Value::Int(i as i64));
                }
            }
            Ok(Value::Bool(false))
        }
        Some(Value::AssociativeArray(map)) => {
            for (k, v) in map.iter() {
                if values_equal(v, needle.unwrap_or(&Value::Null)) {
                    return Ok(Value::String(k.clone()));
                }
            }
            Ok(Value::Bool(false))
        }
        _ => Ok(Value::Bool(false)),
    }
}

/// array_fill — 用给定的值填充数组
pub fn builtin_array_fill(args: &[Value]) -> Result<Value> {
    let start_index = args.first().map(|v| v.to_int()).unwrap_or(0);
    let count = args.get(1).map(|v| v.to_int()).unwrap_or(0);
    let value = args.get(2).cloned().unwrap_or(Value::Null);

    let mut result = Vec::new();
    for i in 0..count {
        if start_index >= 0 {
            result.push(value.clone());
        }
    }

    Ok(Value::IndexedArray(result))
}

/// array_combine — 创建一个数组，用一个数组的值作为其键名，另一个数组的值作为其值
pub fn builtin_array_combine(args: &[Value]) -> Result<Value> {
    let keys = args.first();
    let values = args.get(1);

    match (keys, values) {
        (Some(Value::IndexedArray(keys_arr)), Some(Value::IndexedArray(values_arr))) => {
            if keys_arr.len() != values_arr.len() {
                return Ok(Value::Bool(false));
            }

            let combined: Vec<(String, Value)> = keys_arr
                .iter()
                .zip(values_arr.iter())
                .map(|(k, v)| (k.to_string_value(), v.clone()))
                .collect();

            Ok(Value::AssociativeArray(combined))
        }
        _ => Ok(Value::Bool(false)),
    }
}

/// array_flip — 交换数组中的键和值
pub fn builtin_array_flip(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::AssociativeArray(map)) => {
            let flipped: Vec<(String, Value)> = map
                .iter()
                .map(|(k, v)| (v.to_string_value(), Value::String(k.clone())))
                .collect();
            Ok(Value::AssociativeArray(flipped))
        }
        Some(Value::IndexedArray(arr)) => {
            let flipped: Vec<(String, Value)> = arr
                .iter()
                .enumerate()
                .map(|(i, v)| (v.to_string_value(), Value::Int(i as i64)))
                .collect();
            Ok(Value::AssociativeArray(flipped))
        }
        _ => Ok(Value::AssociativeArray(Vec::new())),
    }
}

/// array_sum — 计算数组中所有值的和
pub fn builtin_array_sum(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(arr)) => {
            let sum: f64 = arr.iter().map(|v| v.to_float()).sum();
            if sum.fract() == 0.0 {
                Ok(Value::Int(sum as i64))
            } else {
                Ok(Value::Float(sum))
            }
        }
        Some(Value::AssociativeArray(map)) => {
            let sum: f64 = map.iter().map(|(_, v)| v.to_float()).sum();
            if sum.fract() == 0.0 {
                Ok(Value::Int(sum as i64))
            } else {
                Ok(Value::Float(sum))
            }
        }
        _ => Ok(Value::Int(0)),
    }
}

/// array_product — 计算数组中所有值的乘积
pub fn builtin_array_product(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::IndexedArray(arr)) => {
            let product: f64 = arr.iter().map(|v| v.to_float()).product();
            if product.fract() == 0.0 {
                Ok(Value::Int(product as i64))
            } else {
                Ok(Value::Float(product))
            }
        }
        Some(Value::AssociativeArray(map)) => {
            let product: f64 = map.iter().map(|(_, v)| v.to_float()).product();
            if product.fract() == 0.0 {
                Ok(Value::Int(product as i64))
            } else {
                Ok(Value::Float(product))
            }
        }
        _ => Ok(Value::Int(0)),
    }
}

/// array_rand — 从数组中随机取出一个或多个单元
pub fn builtin_array_rand(args: &[Value]) -> Result<Value> {
    use rand::Rng;
    use rand::seq::SliceRandom;

    let array = args.first();
    let num = args.get(1).map(|v| v.to_int()).unwrap_or(1);

    match array {
        Some(Value::IndexedArray(arr)) => {
            if arr.is_empty() {
                return Ok(Value::Null);
            }

            let mut rng = rand::rng();

            if num == 1 {
                let index = rng.random_range(0..arr.len());
                Ok(Value::Int(index as i64))
            } else {
                let count = num.min(arr.len() as i64) as usize;
                let mut indices: Vec<usize> = (0..arr.len()).collect();
                indices.shuffle(&mut rng);
                let selected: Vec<Value> = indices[..count]
                    .iter()
                    .map(|&i| Value::Int(i as i64))
                    .collect();
                Ok(Value::IndexedArray(selected))
            }
        }
        Some(Value::AssociativeArray(map)) => {
            if map.is_empty() {
                return Ok(Value::Null);
            }

            let keys: Vec<&String> = map.iter().map(|(k, _)| k).collect();
            let mut rng = rand::rng();

            if num == 1 {
                let index = rng.random_range(0..keys.len());
                Ok(Value::String(keys[index].clone()))
            } else {
                let count = num.min(keys.len() as i64) as usize;
                let mut keys_owned: Vec<String> = keys.into_iter().cloned().collect();
                keys_owned.shuffle(&mut rng);
                let selected: Vec<Value> = keys_owned[..count]
                    .iter()
                    .map(|k| Value::String(k.clone()))
                    .collect();
                Ok(Value::IndexedArray(selected))
            }
        }
        _ => Ok(Value::Null),
    }
}

/// shuffle — 打乱数组
pub fn builtin_shuffle(args: &[Value]) -> Result<Value> {
    use rand::seq::SliceRandom;

    match args.first() {
        Some(Value::IndexedArray(arr)) => {
            let mut shuffled = arr.clone();
            let mut rng = rand::rng();
            shuffled.shuffle(&mut rng);
            Ok(Value::Bool(true))
        }
        _ => Ok(Value::Bool(false)),
    }
}

/// 比较两个值是否相等（弱类型比较）
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(i1), Value::Int(i2)) => i1 == i2,
        (Value::Float(f1), Value::Float(f2)) => f1 == f2,
        (Value::String(s1), Value::String(s2)) => s1 == s2,
        (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
        (Value::Null, Value::Null) => true,
        (Value::Int(i), Value::Float(f)) | (Value::Float(f), Value::Int(i)) => (*i as f64) == *f,
        (Value::String(s), Value::Int(i)) => s.parse::<i64>().map(|parsed| parsed == *i).unwrap_or(false),
        (Value::Int(i), Value::String(s)) => s.parse::<i64>().map(|parsed| parsed == *i).unwrap_or(false),
        (Value::String(s), Value::Float(f)) => s.parse::<f64>().map(|parsed| parsed == *f).unwrap_or(false),
        (Value::Float(f), Value::String(s)) => s.parse::<f64>().map(|parsed| parsed == *f).unwrap_or(false),
        _ => false,
    }
}

/// 比较两个值是否严格相等（强类型比较）
fn values_equal_strict(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(i1), Value::Int(i2)) => i1 == i2,
        (Value::Float(f1), Value::Float(f2)) => f1 == f2,
        (Value::String(s1), Value::String(s2)) => s1 == s2,
        (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
        (Value::Null, Value::Null) => true,
        _ => false,
    }
}
