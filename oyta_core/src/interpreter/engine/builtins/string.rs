//! 字符串函数模块
//!
//! 包含 strlen、strtolower、strtoupper、trim、substr 等字符串处理函数

use std::collections::HashMap;

use anyhow::Result;

use crate::interpreter::value::Value;

use super::types::BuiltinFunction;

/// 注册字符串函数到映射表
///
/// # 参数
/// - `map`: 内置函数映射表的可变引用
pub fn register_string_functions(map: &mut HashMap<String, BuiltinFunction>) {
    // 字符串长度
    map.insert("strlen".to_string(), builtin_strlen);
    // 转小写
    map.insert("strtolower".to_string(), builtin_strtolower);
    // 转大写
    map.insert("strtoupper".to_string(), builtin_strtoupper);
    // 去除两端空白
    map.insert("trim".to_string(), builtin_trim);
    // 去除左侧空白
    map.insert("ltrim".to_string(), builtin_ltrim);
    // 去除右侧空白
    map.insert("rtrim".to_string(), builtin_rtrim);
    // 子字符串
    map.insert("substr".to_string(), builtin_substr);
    // 字符串替换
    map.insert("str_replace".to_string(), builtin_str_replace);
    // 字符串包含
    map.insert("str_contains".to_string(), builtin_str_contains);
    // 字符串开头
    map.insert("str_starts_with".to_string(), builtin_str_starts_with);
    // 字符串结尾
    map.insert("str_ends_with".to_string(), builtin_str_ends_with);
    // 查找位置
    map.insert("strpos".to_string(), builtin_strpos);
    // 重复字符串
    map.insert("str_repeat".to_string(), builtin_str_repeat);
    // 填充字符串
    map.insert("str_pad".to_string(), builtin_str_pad);
    // 分割字符串
    map.insert("explode".to_string(), builtin_explode);
    // 连接字符串
    map.insert("implode".to_string(), builtin_implode);
    map.insert("join".to_string(), builtin_implode);
    // 反转字符串
    map.insert("strrev".to_string(), builtin_strrev);
    // 随机打乱
    map.insert("str_shuffle".to_string(), builtin_str_shuffle);
    // 单词计数
    map.insert("str_word_count".to_string(), builtin_str_word_count);
    // 查找最后一次出现位置
    map.insert("strrpos".to_string(), builtin_strrpos);
}

/// strlen — 获取字符串长度
pub fn builtin_strlen(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::Int(s.chars().count() as i64))
}

/// strtolower — 将字符串转换为小写
pub fn builtin_strtolower(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::String(s.to_lowercase()))
}

/// strtoupper — 将字符串转换为大写
pub fn builtin_strtoupper(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    Ok(Value::String(s.to_uppercase()))
}

/// trim — 去除字符串首尾处的空白字符（或者其他字符）
pub fn builtin_trim(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();

    // 检查是否有第二个参数（自定义字符）
    let trimmed = if let Some(chars) = args.get(1) {
        let chars_str = chars.to_string_value();
        s.trim_matches(|c| chars_str.contains(c))
    } else {
        s.trim()
    };

    Ok(Value::String(trimmed.to_string()))
}

/// ltrim — 删除字符串开头的空白字符（或其他字符）
pub fn builtin_ltrim(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();

    let trimmed = if let Some(chars) = args.get(1) {
        let chars_str = chars.to_string_value();
        s.trim_start_matches(|c| chars_str.contains(c))
    } else {
        s.trim_start()
    };

    Ok(Value::String(trimmed.to_string()))
}

/// rtrim — 删除字符串末端的空白字符（或者其他字符）
pub fn builtin_rtrim(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();

    let trimmed = if let Some(chars) = args.get(1) {
        let chars_str = chars.to_string_value();
        s.trim_end_matches(|c| chars_str.contains(c))
    } else {
        s.trim_end()
    };

    Ok(Value::String(trimmed.to_string()))
}

/// substr — 返回字符串的子串
pub fn builtin_substr(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let start = args.get(1).map(|v| v.to_int()).unwrap_or(0);

    // 将字符串转为字符数组以正确处理 UTF-8
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;

    // 处理负数 start
    let start_idx = if start < 0 {
        (len + start).max(0) as usize
    } else {
        start.min(len) as usize
    };

    // 处理长度参数
    let result = if let Some(length) = args.get(2) {
        let length = length.to_int();
        if length < 0 {
            // 负长度表示从末尾截取
            let end_idx = (len + length).max(start_idx as i64) as usize;
            chars[start_idx..end_idx].iter().collect()
        } else {
            let end_idx = (start_idx + length as usize).min(chars.len());
            chars[start_idx..end_idx].iter().collect()
        }
    } else {
        chars[start_idx..].iter().collect()
    };

    Ok(Value::String(result))
}

/// str_replace — 子字符串替换
pub fn builtin_str_replace(args: &[Value]) -> Result<Value> {
    let search = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let replace = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    let subject = args.get(2).map(|v| v.to_string_value()).unwrap_or_default();

    // 简单字符串替换
    let result = subject.replace(&search, &replace);

    // 如果有第四个参数，返回替换次数
    if args.len() > 3 {
        // 计算替换次数
        let count = subject.matches(&search).count() as i64;
        // 注意：这里简化处理，实际 PHP 会通过引用修改第四个参数
        tracing::debug!("str_replace 替换次数: {}", count);
    }

    Ok(Value::String(result))
}

/// str_contains — 判断字符串是否包含另一个字符串（PHP 8.0+）
pub fn builtin_str_contains(args: &[Value]) -> Result<Value> {
    let haystack = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let needle = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();

    Ok(Value::Bool(haystack.contains(&needle)))
}

/// str_starts_with — 检查字符串是否以指定字符串开头（PHP 8.0+）
pub fn builtin_str_starts_with(args: &[Value]) -> Result<Value> {
    let haystack = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let needle = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();

    Ok(Value::Bool(haystack.starts_with(&needle)))
}

/// str_ends_with — 检查字符串是否以指定字符串结尾（PHP 8.0+）
pub fn builtin_str_ends_with(args: &[Value]) -> Result<Value> {
    let haystack = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let needle = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();

    Ok(Value::Bool(haystack.ends_with(&needle)))
}

/// strpos — 查找字符串首次出现的位置
pub fn builtin_strpos(args: &[Value]) -> Result<Value> {
    let haystack = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let needle = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    let offset = args.get(2).map(|v| v.to_int()).unwrap_or(0);

    // 处理偏移量
    let search_start = if offset < 0 {
        (haystack.len() as i64 + offset).max(0) as usize
    } else {
        offset as usize
    };

    // 从偏移位置开始搜索
    if search_start > haystack.len() {
        return Ok(Value::Bool(false));
    }

    let search_str = &haystack[search_start..];

    match search_str.find(&needle) {
        Some(pos) => {
            // 返回字节位置（PHP strpos 返回字节位置）
            let byte_pos = haystack[..search_start + pos].len();
            Ok(Value::Int(byte_pos as i64))
        }
        None => Ok(Value::Bool(false)),
    }
}

/// str_repeat — 重复一个字符串
pub fn builtin_str_repeat(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let times = args.get(1).map(|v| v.to_int()).unwrap_or(0);

    if times <= 0 {
        return Ok(Value::String(String::new()));
    }

    Ok(Value::String(s.repeat(times as usize)))
}

/// str_pad — 使用另一个字符串填充字符串为指定长度
pub fn builtin_str_pad(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let length = args.get(1).map(|v| v.to_int()).unwrap_or(0) as usize;
    let pad_string = args.get(2).map(|v| v.to_string_value()).unwrap_or_else(|| " ".to_string());
    let pad_type = args.get(3).map(|v| v.to_int()).unwrap_or(1); // 1=right, 0=left, 2=both

    let current_len = s.chars().count();
    if current_len >= length {
        return Ok(Value::String(s));
    }

    let pad_len = length - current_len;
    let pad_chars: Vec<char> = pad_string.chars().collect();

    if pad_chars.is_empty() {
        return Ok(Value::String(s));
    }

    // 生成填充字符串
    let mut pad = String::new();
    let mut i = 0;
    while pad.chars().count() < pad_len {
        pad.push(pad_chars[i % pad_chars.len()]);
        i += 1;
    }
    let pad = pad.chars().take(pad_len).collect::<String>();

    let result = match pad_type {
        0 => format!("{}{}", pad, s), // STR_PAD_LEFT
        2 => {
            // STR_PAD_BOTH
            let left_len = pad_len / 2;
            let right_len = pad_len - left_len;
            let left_pad: String = pad.chars().take(left_len).collect();
            let right_pad: String = pad.chars().take(right_len).collect();
            format!("{}{}{}", left_pad, s, right_pad)
        }
        _ => format!("{}{}", s, pad), // STR_PAD_RIGHT (default)
    };

    Ok(Value::String(result))
}

/// explode — 使用一个字符串分割另一个字符串
pub fn builtin_explode(args: &[Value]) -> Result<Value> {
    let delimiter = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let s = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();
    let limit = args.get(2).map(|v| v.to_int());

    // 空分隔符特殊处理
    if delimiter.is_empty() {
        return Ok(Value::Bool(false));
    }

    let parts: Vec<Value> = match limit {
        Some(l) if l > 0 => {
            // 正限制：最多分割成 l 个部分
            let split: Vec<&str> = s.splitn(l as usize, &delimiter).collect();
            split.into_iter().map(|p| Value::String(p.to_string())).collect()
        }
        Some(l) if l < 0 => {
            // 负限制：返回除最后 |l| 个元素外的所有部分
            let split: Vec<&str> = s.split(&delimiter).collect();
            let take = (split.len() as i64 + l).max(0) as usize;
            split.into_iter().take(take).map(|p| Value::String(p.to_string())).collect()
        }
        _ => {
            // 无限制或限制为 0
            s.split(&delimiter).map(|p| Value::String(p.to_string())).collect()
        }
    };

    Ok(Value::IndexedArray(parts))
}

/// implode — 将一个一维数组的值转化为字符串
pub fn builtin_implode(args: &[Value]) -> Result<Value> {
    // 根据参数数量确定分隔符和数组
    let (separator, array) = if args.len() == 1 {
        // 只有一个参数，默认分隔符为空字符串
        ("", args.first().unwrap())
    } else {
        // 两个参数：第一个是分隔符，第二个是数组
        let sep = args.first().map(|v| v.to_string_value()).unwrap_or_default();
        let arr = args.get(1).unwrap();
        // 直接在这里处理，避免生命周期问题
        let parts: Vec<String> = match arr {
            Value::IndexedArray(arr) => arr.iter().map(|v| v.to_string_value()).collect(),
            Value::AssociativeArray(map) => map.iter().map(|(_, v)| v.to_string_value()).collect(),
            _ => return Ok(Value::String(String::new())),
        };
        return Ok(Value::String(parts.join(&sep)));
    };

    let parts: Vec<String> = match array {
        Value::IndexedArray(arr) => arr.iter().map(|v| v.to_string_value()).collect(),
        Value::AssociativeArray(map) => map.iter().map(|(_, v)| v.to_string_value()).collect(),
        _ => return Ok(Value::String(String::new())),
    };

    Ok(Value::String(parts.join(separator)))
}

/// strrev — 反转字符串
pub fn builtin_strrev(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let reversed: String = s.chars().rev().collect();
    Ok(Value::String(reversed))
}

/// str_shuffle — 随机打乱一个字符串
pub fn builtin_str_shuffle(args: &[Value]) -> Result<Value> {
    use rand::seq::SliceRandom;

    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let mut chars: Vec<char> = s.chars().collect();

    // 使用 rand 0.9.x 的 rng() 函数
    let mut rng = rand::rng();
    chars.shuffle(&mut rng);

    Ok(Value::String(chars.into_iter().collect()))
}

/// str_word_count — 返回字符串中单词的使用情况
pub fn builtin_str_word_count(args: &[Value]) -> Result<Value> {
    let s = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let format = args.get(1).map(|v| v.to_int()).unwrap_or(0);

    // 简单的单词分割（按空白字符）
    let words: Vec<&str> = s.split_whitespace().collect();

    match format {
        0 => Ok(Value::Int(words.len() as i64)),
        1 => Ok(Value::IndexedArray(
            words.into_iter().map(|w| Value::String(w.to_string())).collect()
        )),
        2 => {
            // 返回关联数组（单词 => 位置）
            let mut result = Vec::new();
            let mut pos = 0;
            for word in s.split_whitespace() {
                if let Some(byte_pos) = s[pos..].find(word) {
                    let actual_pos = pos + byte_pos;
                    result.push((actual_pos as i64, Value::String(word.to_string())));
                    pos = actual_pos + word.len();
                }
            }
            Ok(Value::AssociativeArray(
                result.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
            ))
        }
        _ => Ok(Value::Int(words.len() as i64)),
    }
}

/// strrpos — 计算指定字符串在目标字符串中最后一次出现的位置
pub fn builtin_strrpos(args: &[Value]) -> Result<Value> {
    let haystack = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let needle = args.get(1).map(|v| v.to_string_value()).unwrap_or_default();

    match haystack.rfind(&needle) {
        Some(pos) => Ok(Value::Int(pos as i64)),
        None => Ok(Value::Bool(false)),
    }
}
