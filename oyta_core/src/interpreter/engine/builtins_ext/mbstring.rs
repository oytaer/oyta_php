//! mbstring 多字节字符串内置函数注册模块
//!
//! 本模块实现 PHP mbstring 扩展的内置函数注册
//! 包括：mb_strlen, mb_substr, mb_strpos, mb_strrpos, mb_strtolower, mb_strtoupper,
//! mb_convert_encoding, mb_detect_encoding, mb_ereg, mb_eregi, mb_ereg_replace 等

use std::collections::HashMap;

use super::super::builtins::BuiltinFunction;
use crate::interpreter::value::Value;
use anyhow::Result;

// ============================================================================
// 编码处理辅助函数
// ============================================================================

/// 获取编码名称，如果未指定则返回默认编码 UTF-8
///
/// # 参数
/// - `encoding`: 可选的编码名称
///
/// # 返回
/// 编码名称字符串
fn get_encoding_name(encoding: Option<&str>) -> &'static str {
    match encoding {
        Some(enc) => {
            // 标准化编码名称
            let enc_lower = enc.to_lowercase();
            match enc_lower.as_str() {
                "utf8" | "utf-8" => "UTF-8",
                "gbk" | "gb2312" | "gb18030" => "GBK",
                "big5" => "BIG5",
                "shift_jis" | "shift-jis" | "sjis" => "Shift_JIS",
                "euc-jp" => "EUC-JP",
                "iso-8859-1" | "latin1" => "ISO-8859-1",
                "ascii" => "ASCII",
                _ => "UTF-8",
            }
        }
        None => "UTF-8",
    }
}

/// 检测字符串编码
///
/// # 参数
/// - `input`: 输入字符串
///
/// # 返回
/// 检测到的编码名称
fn detect_encoding(input: &str) -> String {
    // 检查是否为有效的 UTF-8
    if input.is_ascii() {
        return "ASCII".to_string();
    }
    
    // 检查是否为有效的 UTF-8
    if input.chars().all(|c| c.is_ascii() || c.len_utf8() > 1) {
        // 尝试检测是否包含中文字符
        for c in input.chars() {
            // 简体中文范围：U+4E00 到 U+9FFF
            if c >= '\u{4E00}' && c <= '\u{9FFF}' {
                return "UTF-8".to_string();
            }
            // 日文假名范围
            if (c >= '\u{3040}' && c <= '\u{309F}') || (c >= '\u{30A0}' && c <= '\u{30FF}') {
                return "UTF-8".to_string();
            }
            // 韩文字符范围
            if c >= '\u{AC00}' && c <= '\u{D7AF}' {
                return "UTF-8".to_string();
            }
        }
        return "UTF-8".to_string();
    }
    
    // 默认返回 UTF-8
    "UTF-8".to_string()
}

/// 将字符串转换为字符数组
///
/// # 参数
/// - `input`: 输入字符串
///
/// # 返回
/// 字符数组
fn string_to_chars(input: &str) -> Vec<char> {
    input.chars().collect()
}

// ============================================================================
// mbstring 内置函数实现
// ============================================================================

/// mb_strlen — 获取字符串的长度（字符数）
///
/// # PHP 签名
/// mb_strlen(string $string, ?string $encoding = null): int
///
/// # 参数
/// - args[0]: 输入字符串
/// - args[1]: 可选的字符编码
///
/// # 返回
/// 字符串的字符数
pub fn builtin_mb_strlen(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    
    // 获取输入字符串
    let input = match &args[0] {
        Value::String(s) => s.as_str(),
        Value::Null => return Ok(Value::Int(0)),
        _ => return Ok(Value::Int(0)),
    };
    
    // 获取编码（虽然 Rust 字符串是 UTF-8，但我们仍然处理参数）
    let _encoding = if args.len() > 1 {
        match &args[1] {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    } else {
        None
    };
    
    // 计算字符数（Rust 的 chars() 返回 Unicode 字符迭代器）
    let len = input.chars().count();
    
    Ok(Value::Int(len as i64))
}

/// mb_substr — 获取字符串的部分
///
/// # PHP 签名
/// mb_substr(string $string, int $start, ?int $length = null, ?string $encoding = null): string
///
/// # 参数
/// - args[0]: 输入字符串
/// - args[1]: 起始位置
/// - args[2]: 可选的长度
/// - args[3]: 可选的字符编码
///
/// # 返回
/// 子字符串
pub fn builtin_mb_substr(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::String(String::new()));
    }
    
    // 获取输入字符串
    let input = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::String(String::new())),
    };
    
    // 获取起始位置
    let start = match &args[1] {
        Value::Int(i) => *i as isize,
        Value::Float(f) => *f as isize,
        _ => return Ok(Value::String(String::new())),
    };
    
    // 获取长度
    let length = if args.len() > 2 {
        match &args[2] {
            Value::Int(i) => Some(*i as usize),
            Value::Null => None,
            _ => None,
        }
    } else {
        None
    };
    
    // 忽略编码参数（Rust 字符串是 UTF-8）
    let _encoding = if args.len() > 3 {
        match &args[3] {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    } else {
        None
    };
    
    // 转换为字符数组
    let chars: Vec<char> = string_to_chars(input);
    let char_count = chars.len();
    
    // 处理负起始位置
    let start_pos = if start < 0 {
        char_count.saturating_sub(start.abs() as usize)
    } else {
        start as usize
    };
    
    // 确保起始位置不超过字符串长度
    if start_pos >= char_count {
        return Ok(Value::String(String::new()));
    }
    
    // 计算结束位置
    let end_pos = match length {
        Some(len) => (start_pos + len).min(char_count),
        None => char_count,
    };
    
    // 提取子字符串
    let result: String = chars[start_pos..end_pos].iter().collect();
    
    Ok(Value::String(result))
}

/// mb_strpos — 查找字符串在另一个字符串中首次出现的位置
///
/// # PHP 签名
/// mb_strpos(string $haystack, string $needle, int $offset = 0, ?string $encoding = null): int|false
///
/// # 参数
/// - args[0]: 主字符串
/// - args[1]: 要查找的子字符串
/// - args[2]: 可选的起始偏移量
/// - args[3]: 可选的字符编码
///
/// # 返回
/// 找到返回位置，未找到返回 false
pub fn builtin_mb_strpos(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取主字符串
    let haystack = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取要查找的字符串
    let needle = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取偏移量
    let offset = if args.len() > 2 {
        match &args[2] {
            Value::Int(i) => (*i).max(0) as usize,
            _ => 0,
        }
    } else {
        0
    };
    
    // 转换为字符数组
    let haystack_chars: Vec<char> = string_to_chars(haystack);
    let needle_chars: Vec<char> = string_to_chars(needle);
    
    // 检查偏移量
    if offset >= haystack_chars.len() {
        return Ok(Value::Bool(false));
    }
    
    // 检查空 needle
    if needle_chars.is_empty() {
        return Ok(Value::Int(offset as i64));
    }
    
    // 从偏移位置开始搜索
    let search_start = offset;
    let search_end = haystack_chars.len().saturating_sub(needle_chars.len()) + 1;
    
    for i in search_start..search_end {
        // 检查是否匹配
        let mut matches = true;
        for (j, &needle_char) in needle_chars.iter().enumerate() {
            if i + j >= haystack_chars.len() || haystack_chars[i + j] != needle_char {
                matches = false;
                break;
            }
        }
        if matches {
            return Ok(Value::Int(i as i64));
        }
    }
    
    Ok(Value::Bool(false))
}

/// mb_strrpos — 查找字符串在另一个字符串中最后一次出现的位置
///
/// # PHP 签名
/// mb_strrpos(string $haystack, string $needle, int $offset = 0, ?string $encoding = null): int|false
///
/// # 参数
/// - args[0]: 主字符串
/// - args[1]: 要查找的子字符串
/// - args[2]: 可选的起始偏移量
/// - args[3]: 可选的字符编码
///
/// # 返回
/// 找到返回位置，未找到返回 false
pub fn builtin_mb_strrpos(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取主字符串
    let haystack = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取要查找的字符串
    let needle = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 转换为字符数组
    let haystack_chars: Vec<char> = string_to_chars(haystack);
    let needle_chars: Vec<char> = string_to_chars(needle);
    
    // 检查空 needle
    if needle_chars.is_empty() {
        return Ok(Value::Int(haystack_chars.len() as i64));
    }
    
    // 从后向前搜索
    let search_end = haystack_chars.len().saturating_sub(needle_chars.len());
    
    for i in (0..=search_end).rev() {
        // 检查是否匹配
        let mut matches = true;
        for (j, &needle_char) in needle_chars.iter().enumerate() {
            if i + j >= haystack_chars.len() || haystack_chars[i + j] != needle_char {
                matches = false;
                break;
            }
        }
        if matches {
            return Ok(Value::Int(i as i64));
        }
    }
    
    Ok(Value::Bool(false))
}

/// mb_stripos — 查找字符串在另一个字符串中首次出现的位置（不区分大小写）
///
/// # PHP 签名
/// mb_stripos(string $haystack, string $needle, int $offset = 0, ?string $encoding = null): int|false
///
/// # 参数
/// - args[0]: 主字符串
/// - args[1]: 要查找的子字符串
/// - args[2]: 可选的起始偏移量
/// - args[3]: 可选的字符编码
///
/// # 返回
/// 找到返回位置，未找到返回 false
pub fn builtin_mb_stripos(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取主字符串
    let haystack = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取要查找的字符串
    let needle = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取偏移量
    let offset = if args.len() > 2 {
        match &args[2] {
            Value::Int(i) => (*i).max(0) as usize,
            _ => 0,
        }
    } else {
        0
    };
    
    // 转换为小写字符数组
    let haystack_lower: Vec<char> = haystack.to_lowercase().chars().collect();
    let needle_lower: Vec<char> = needle.to_lowercase().chars().collect();
    
    // 检查偏移量
    if offset >= haystack_lower.len() {
        return Ok(Value::Bool(false));
    }
    
    // 检查空 needle
    if needle_lower.is_empty() {
        return Ok(Value::Int(offset as i64));
    }
    
    // 从偏移位置开始搜索
    let search_start = offset;
    let search_end = haystack_lower.len().saturating_sub(needle_lower.len()) + 1;
    
    for i in search_start..search_end {
        // 检查是否匹配
        let mut matches = true;
        for (j, &needle_char) in needle_lower.iter().enumerate() {
            if i + j >= haystack_lower.len() || haystack_lower[i + j] != needle_char {
                matches = false;
                break;
            }
        }
        if matches {
            return Ok(Value::Int(i as i64));
        }
    }
    
    Ok(Value::Bool(false))
}

/// mb_strripos — 查找字符串在另一个字符串中最后一次出现的位置（不区分大小写）
///
/// # PHP 签名
/// mb_strripos(string $haystack, string $needle, int $offset = 0, ?string $encoding = null): int|false
///
/// # 参数
/// - args[0]: 主字符串
/// - args[1]: 要查找的子字符串
/// - args[2]: 可选的起始偏移量
/// - args[3]: 可选的字符编码
///
/// # 返回
/// 找到返回位置，未找到返回 false
pub fn builtin_mb_strripos(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取主字符串
    let haystack = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取要查找的字符串
    let needle = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 转换为小写字符数组
    let haystack_lower: Vec<char> = haystack.to_lowercase().chars().collect();
    let needle_lower: Vec<char> = needle.to_lowercase().chars().collect();
    
    // 检查空 needle
    if needle_lower.is_empty() {
        return Ok(Value::Int(haystack_lower.len() as i64));
    }
    
    // 从后向前搜索
    let search_end = haystack_lower.len().saturating_sub(needle_lower.len());
    
    for i in (0..=search_end).rev() {
        // 检查是否匹配
        let mut matches = true;
        for (j, &needle_char) in needle_lower.iter().enumerate() {
            if i + j >= haystack_lower.len() || haystack_lower[i + j] != needle_char {
                matches = false;
                break;
            }
        }
        if matches {
            return Ok(Value::Int(i as i64));
        }
    }
    
    Ok(Value::Bool(false))
}

/// mb_strtolower — 将字符串转换为小写
///
/// # PHP 签名
/// mb_strtolower(string $string, ?string $encoding = null): string
///
/// # 参数
/// - args[0]: 输入字符串
/// - args[1]: 可选的字符编码
///
/// # 返回
/// 小写字符串
pub fn builtin_mb_strtolower(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::String(String::new()));
    }
    
    // 获取输入字符串
    let input = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::String(String::new())),
    };
    
    // 转换为小写（Rust 的 to_lowercase 支持 Unicode）
    let result = input.to_lowercase();
    
    Ok(Value::String(result))
}

/// mb_strtoupper — 将字符串转换为大写
///
/// # PHP 签名
/// mb_strtoupper(string $string, ?string $encoding = null): string
///
/// # 参数
/// - args[0]: 输入字符串
/// - args[1]: 可选的字符编码
///
/// # 返回
/// 大写字符串
pub fn builtin_mb_strtoupper(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::String(String::new()));
    }
    
    // 获取输入字符串
    let input = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::String(String::new())),
    };
    
    // 转换为大写（Rust 的 to_uppercase 支持 Unicode）
    let result = input.to_uppercase();
    
    Ok(Value::String(result))
}

/// mb_strwidth — 返回字符串的宽度
///
/// # PHP 签名
/// mb_strwidth(string $string, ?string $encoding = null): int
///
/// # 参数
/// - args[0]: 输入字符串
/// - args[1]: 可选的字符编码
///
/// # 返回
/// 字符串宽度（全角字符宽度为 2，半角字符宽度为 1）
pub fn builtin_mb_strwidth(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    
    // 获取输入字符串
    let input = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Int(0)),
    };
    
    // 计算字符串宽度
    // 全角字符（中日韩文字等）宽度为 2，半角字符宽度为 1
    let mut width = 0;
    for c in input.chars() {
        // 判断是否为全角字符
        let is_fullwidth = match c {
            // CJK 统一汉字
            '\u{4E00}'..='\u{9FFF}' => true,
            // CJK 扩展 A
            '\u{3400}'..='\u{4DBF}' => true,
            // CJK 扩展 B-F
            '\u{20000}'..='\u{2CEAF}' => true,
            // 日文假名
            '\u{3040}'..='\u{309F}' | '\u{30A0}'..='\u{30FF}' => true,
            // 韩文字符
            '\u{AC00}'..='\u{D7AF}' => true,
            // 全角 ASCII 和标点
            '\u{FF00}'..='\u{FFEF}' => true,
            _ => false,
        };
        
        width += if is_fullwidth { 2 } else { 1 };
    }
    
    Ok(Value::Int(width as i64))
}

/// mb_strimwidth — 获取指定宽度的字符串
///
/// # PHP 签名
/// mb_strimwidth(string $str, int $start, int $width, ?string $trim_marker = "", ?string $encoding = null): string
///
/// # 参数
/// - args[0]: 输入字符串
/// - args[1]: 起始位置
/// - args[2]: 宽度
/// - args[3]: 可选的截断标记
/// - args[4]: 可选的字符编码
///
/// # 返回
/// 截断后的字符串
pub fn builtin_mb_strimwidth(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 3 {
        return Ok(Value::String(String::new()));
    }
    
    // 获取输入字符串
    let input = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::String(String::new())),
    };
    
    // 获取起始位置
    let start = match &args[1] {
        Value::Int(i) => *i as usize,
        _ => 0,
    };
    
    // 获取宽度
    let width = match &args[2] {
        Value::Int(i) => *i as usize,
        _ => return Ok(Value::String(String::new())),
    };
    
    // 获取截断标记
    let trim_marker = if args.len() > 3 {
        match &args[3] {
            Value::String(s) => s.as_str(),
            _ => "",
        }
    } else {
        ""
    };
    
    // 转换为字符数组
    let chars: Vec<char> = string_to_chars(input);
    
    // 计算起始位置
    let start_pos = start.min(chars.len());
    
    // 计算截断位置
    let mut current_width = 0;
    let mut end_pos = start_pos;
    
    for i in start_pos..chars.len() {
        let c = chars[i];
        // 判断是否为全角字符
        let char_width = match c {
            '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}' |
            '\u{3040}'..='\u{309F}' | '\u{30A0}'..='\u{30FF}' |
            '\u{AC00}'..='\u{D7AF}' | '\u{FF00}'..='\u{FFEF}' => 2,
            _ => 1,
        };
        
        if current_width + char_width > width {
            break;
        }
        
        current_width += char_width;
        end_pos = i + 1;
    }
    
    // 构建结果字符串
    let mut result: String = chars[start_pos..end_pos].iter().collect();
    
    // 如果截断了，添加截断标记
    if end_pos < chars.len() && !trim_marker.is_empty() {
        result.push_str(trim_marker);
    }
    
    Ok(Value::String(result))
}

/// mb_convert_encoding — 转换字符编码
///
/// # PHP 签名
/// mb_convert_encoding(string|array $string, string $to_encoding, array|string|null $from_encoding = null): string|array
///
/// # 参数
/// - args[0]: 输入字符串
/// - args[1]: 目标编码
/// - args[2]: 可选的源编码
///
/// # 返回
/// 转换后的字符串
pub fn builtin_mb_convert_encoding(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::String(String::new()));
    }
    
    // 获取输入字符串
    let input = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::String(String::new())),
    };
    
    // 获取目标编码
    let to_encoding = match &args[1] {
        Value::String(s) => get_encoding_name(Some(s.as_str())),
        _ => "UTF-8",
    };
    
    // 获取源编码
    let from_encoding = if args.len() > 2 {
        match &args[2] {
            Value::String(s) => Some(s.as_str()),
            Value::IndexedArray(arr) => arr.first().and_then(|v| {
                if let Value::String(s) = v { Some(s.as_str()) } else { None }
            }),
            _ => None,
        }
    } else {
        None
    };
    
    // 执行编码转换
    let result = if to_encoding == "UTF-8" || to_encoding == "ASCII" {
        // 目标是 UTF-8，直接返回（Rust 字符串是 UTF-8）
        input.to_string()
    } else if to_encoding == "GBK" || to_encoding == "GB2312" {
        // 转换为 GBK
        // 使用 encoding_rs 库进行转换
        let (encoded, _, _had_errors) = encoding_rs::GBK.encode(input);
        String::from_utf8_lossy(&encoded).to_string()
    } else if to_encoding == "BIG5" {
        // 转换为 BIG5
        let (encoded, _, _had_errors) = encoding_rs::BIG5.encode(input);
        String::from_utf8_lossy(&encoded).to_string()
    } else if to_encoding == "Shift_JIS" {
        // 转换为 Shift_JIS
        let (encoded, _, _had_errors) = encoding_rs::SHIFT_JIS.encode(input);
        String::from_utf8_lossy(&encoded).to_string()
    } else if to_encoding == "ISO-8859-1" {
        // 转换为 ISO-8859-1
        let (encoded, _, _had_errors) = encoding_rs::WINDOWS_1252.encode(input);
        String::from_utf8_lossy(&encoded).to_string()
    } else {
        // 不支持的编码，返回原字符串
        input.to_string()
    };
    
    // 忽略 from_encoding 参数（Rust 字符串已经是 UTF-8）
    let _ = from_encoding;
    
    Ok(Value::String(result))
}

/// mb_detect_encoding — 检测字符编码
///
/// # PHP 签名
/// mb_detect_encoding(string $string, array|string|null $encodings = null, bool $strict = false): string|false
///
/// # 参数
/// - args[0]: 输入字符串
/// - args[1]: 可选的编码列表
/// - args[2]: 是否严格模式
///
/// # 返回
/// 检测到的编码名称，失败返回 false
pub fn builtin_mb_detect_encoding(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 获取输入字符串
    let input = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 检测编码
    let detected = detect_encoding(input);
    
    Ok(Value::String(detected))
}

/// mb_internal_encoding — 设置/获取内部字符编码
///
/// # PHP 签名
/// mb_internal_encoding(?string $encoding = null): string|bool
///
/// # 参数
/// - args[0]: 可选的编码名称
///
/// # 返回
/// 设置时返回成功/失败，获取时返回当前编码
pub fn builtin_mb_internal_encoding(args: &[Value]) -> Result<Value> {
    // 检查是否设置了编码
    if args.is_empty() {
        // 获取当前编码
        return Ok(Value::String("UTF-8".to_string()));
    }
    
    // 设置编码
    let _encoding = match &args[0] {
        Value::String(s) => s.as_str(),
        Value::Null => return Ok(Value::String("UTF-8".to_string())),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 简化实现：总是返回 true
    Ok(Value::Bool(true))
}

/// mb_http_output — 设置/获取 HTTP 输出字符编码
///
/// # PHP 签名
/// mb_http_output(?string $encoding = null): string|bool
///
/// # 参数
/// - args[0]: 可选的编码名称
///
/// # 返回
/// 设置时返回成功/失败，获取时返回当前编码
pub fn builtin_mb_http_output(args: &[Value]) -> Result<Value> {
    // 检查是否设置了编码
    if args.is_empty() {
        // 获取当前编码
        return Ok(Value::String("UTF-8".to_string()));
    }
    
    // 设置编码
    Ok(Value::Bool(true))
}

/// mb_http_input — 检测 HTTP 输入字符编码
///
/// # PHP 签名
/// mb_http_input(?string $type = null): string|array|false
///
/// # 参数
/// - args[0]: 可选的输入类型
///
/// # 返回
/// 检测到的编码
pub fn builtin_mb_http_input(args: &[Value]) -> Result<Value> {
    // 忽略参数
    let _ = args;
    
    // 返回默认编码
    Ok(Value::String("UTF-8".to_string()))
}

/// mb_language — 设置/获取当前语言
///
/// # PHP 签名
/// mb_language(?string $language = null): string|bool
///
/// # 参数
/// - args[0]: 可选的语言名称
///
/// # 返回
/// 设置时返回成功/失败，获取时返回当前语言
pub fn builtin_mb_language(args: &[Value]) -> Result<Value> {
    // 检查是否设置了语言
    if args.is_empty() {
        // 获取当前语言
        return Ok(Value::String("neutral".to_string()));
    }
    
    // 设置语言
    Ok(Value::Bool(true))
}

/// mb_substitute_character — 设置/获取替代字符
///
/// # PHP 签名
/// mb_substitute_character(string|int|null $substitute_character = null): string|int|bool
///
/// # 参数
/// - args[0]: 可选的替代字符
///
/// # 返回
/// 设置时返回成功/失败，获取时返回当前替代字符
pub fn builtin_mb_substitute_character(args: &[Value]) -> Result<Value> {
    // 检查是否设置了替代字符
    if args.is_empty() {
        // 获取当前替代字符
        return Ok(Value::String("none".to_string()));
    }
    
    // 设置替代字符
    Ok(Value::Bool(true))
}

/// mb_strstr — 查找字符串的首次出现
///
/// # PHP 签名
/// mb_strstr(string $haystack, string $needle, bool $before_needle = false, ?string $encoding = null): string|false
///
/// # 参数
/// - args[0]: 主字符串
/// - args[1]: 要查找的字符串
/// - args[2]: 可选，是否返回 needle 之前的部分
/// - args[3]: 可选的字符编码
///
/// # 返回
/// 找到的字符串部分或 false
pub fn builtin_mb_strstr(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取主字符串
    let haystack = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取要查找的字符串
    let needle = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取 before_needle 选项
    let before_needle = args.len() > 2 && args[2].is_truthy();
    
    // 查找字符串
    if let Some(pos) = haystack.find(needle) {
        if before_needle {
            // 返回 needle 之前的部分
            Ok(Value::String(haystack[..pos].to_string()))
        } else {
            // 返回从 needle 开始的部分
            Ok(Value::String(haystack[pos..].to_string()))
        }
    } else {
        Ok(Value::Bool(false))
    }
}

/// mb_stristr — 查找字符串的首次出现（不区分大小写）
///
/// # PHP 签名
/// mb_stristr(string $haystack, string $needle, bool $before_needle = false, ?string $encoding = null): string|false
///
/// # 参数
/// - args[0]: 主字符串
/// - args[1]: 要查找的字符串
/// - args[2]: 可选，是否返回 needle 之前的部分
/// - args[3]: 可选的字符编码
///
/// # 返回
/// 找到的字符串部分或 false
pub fn builtin_mb_stristr(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取主字符串
    let haystack = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取要查找的字符串
    let needle = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取 before_needle 选项
    let before_needle = args.len() > 2 && args[2].is_truthy();
    
    // 不区分大小写查找
    let haystack_lower = haystack.to_lowercase();
    let needle_lower = needle.to_lowercase();
    
    if let Some(pos) = haystack_lower.find(&needle_lower) {
        if before_needle {
            Ok(Value::String(haystack[..pos].to_string()))
        } else {
            Ok(Value::String(haystack[pos..].to_string()))
        }
    } else {
        Ok(Value::Bool(false))
    }
}

/// mb_strrchr — 查找字符串的最后一次出现
///
/// # PHP 签名
/// mb_strrchr(string $haystack, string $needle, bool $before_needle = false, ?string $encoding = null): string|false
///
/// # 参数
/// - args[0]: 主字符串
/// - args[1]: 要查找的字符串
/// - args[2]: 可选，是否返回 needle 之前的部分
/// - args[3]: 可选的字符编码
///
/// # 返回
/// 找到的字符串部分或 false
pub fn builtin_mb_strrchr(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取主字符串
    let haystack = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取要查找的字符串
    let needle = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取 before_needle 选项
    let before_needle = args.len() > 2 && args[2].is_truthy();
    
    // 从后向前查找
    if let Some(pos) = haystack.rfind(needle) {
        if before_needle {
            Ok(Value::String(haystack[..pos].to_string()))
        } else {
            Ok(Value::String(haystack[pos..].to_string()))
        }
    } else {
        Ok(Value::Bool(false))
    }
}

/// mb_substr_count — 统计字符串出现的次数
///
/// # PHP 签名
/// mb_substr_count(string $haystack, string $needle, ?string $encoding = null): int
///
/// # 参数
/// - args[0]: 主字符串
/// - args[1]: 要统计的字符串
/// - args[2]: 可选的字符编码
///
/// # 返回
/// 出现的次数
pub fn builtin_mb_substr_count(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Int(0));
    }
    
    // 获取主字符串
    let haystack = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Int(0)),
    };
    
    // 获取要统计的字符串
    let needle = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Int(0)),
    };
    
    // 统计出现次数
    let count = haystack.matches(needle).count();
    
    Ok(Value::Int(count as i64))
}

/// mb_split — 使用正则表达式分割多字节字符串
///
/// # PHP 签名
/// mb_split(string $pattern, string $string, int $limit = -1): array|false
///
/// # 参数
/// - args[0]: 正则表达式模式
/// - args[1]: 输入字符串
/// - args[2]: 可选的限制数量
///
/// # 返回
/// 分割后的数组
pub fn builtin_mb_split(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取正则表达式模式
    let pattern = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取输入字符串
    let input = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取限制数量
    let limit = if args.len() > 2 {
        match &args[2] {
            Value::Int(i) => *i as usize,
            _ => 0,
        }
    } else {
        0
    };
    
    // 编译正则表达式
    match regex::Regex::new(pattern) {
        Ok(re) => {
            // 分割字符串
            let parts: Vec<Value> = if limit > 0 {
                re.splitn(input, limit)
                    .map(|s| Value::String(s.to_string()))
                    .collect()
            } else {
                re.split(input)
                    .map(|s| Value::String(s.to_string()))
                    .collect()
            };
            Ok(Value::IndexedArray(parts))
        }
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// mb_ereg — 正则表达式匹配
///
/// # PHP 签名
/// mb_ereg(string $pattern, string $string, ?array &$matches = null): bool
///
/// # 参数
/// - args[0]: 正则表达式模式
/// - args[1]: 输入字符串
/// - args[2]: 可选的匹配结果数组引用
///
/// # 返回
/// 是否匹配成功
pub fn builtin_mb_ereg(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取正则表达式模式
    let pattern = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    }
    .to_string();
    
    // 获取输入字符串
    let input = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 编译正则表达式
    match regex::Regex::new(&pattern) {
        Ok(re) => {
            // 执行匹配
            if re.is_match(input) {
                Ok(Value::Bool(true))
            } else {
                Ok(Value::Bool(false))
            }
        }
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// mb_eregi — 正则表达式匹配（不区分大小写）
///
/// # PHP 签名
/// mb_eregi(string $pattern, string $string, ?array &$matches = null): bool
///
/// # 参数
/// - args[0]: 正则表达式模式
/// - args[1]: 输入字符串
/// - args[2]: 可选的匹配结果数组引用
///
/// # 返回
/// 是否匹配成功
pub fn builtin_mb_eregi(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取正则表达式模式
    let pattern = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取输入字符串
    let input = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 编译不区分大小写的正则表达式
    match regex::RegexBuilder::new(pattern).case_insensitive(true).build() {
        Ok(re) => {
            if re.is_match(input) {
                Ok(Value::Bool(true))
            } else {
                Ok(Value::Bool(false))
            }
        }
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// mb_ereg_replace — 正则表达式替换
///
/// # PHP 签名
/// mb_ereg_replace(string $pattern, string $replacement, string $string, ?string $options = null): string|false|null
///
/// # 参数
/// - args[0]: 正则表达式模式
/// - args[1]: 替换字符串
/// - args[2]: 输入字符串
/// - args[3]: 可选的选项
///
/// # 返回
/// 替换后的字符串
pub fn builtin_mb_ereg_replace(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 3 {
        return Ok(Value::Bool(false));
    }
    
    // 获取正则表达式模式
    let pattern = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取替换字符串
    let replacement = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取输入字符串
    let input = match &args[2] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 编译正则表达式
    match regex::Regex::new(pattern) {
        Ok(re) => {
            // 执行替换
            let result = re.replace_all(input, replacement).to_string();
            Ok(Value::String(result))
        }
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// mb_eregi_replace — 正则表达式替换（不区分大小写）
///
/// # PHP 签名
/// mb_eregi_replace(string $pattern, string $replacement, string $string, ?string $options = null): string|false|null
///
/// # 参数
/// - args[0]: 正则表达式模式
/// - args[1]: 替换字符串
/// - args[2]: 输入字符串
/// - args[3]: 可选的选项
///
/// # 返回
/// 替换后的字符串
pub fn builtin_mb_eregi_replace(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 3 {
        return Ok(Value::Bool(false));
    }
    
    // 获取正则表达式模式
    let pattern = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取替换字符串
    let replacement = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取输入字符串
    let input = match &args[2] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 编译不区分大小写的正则表达式
    match regex::RegexBuilder::new(pattern).case_insensitive(true).build() {
        Ok(re) => {
            let result = re.replace_all(input, replacement).to_string();
            Ok(Value::String(result))
        }
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// mb_regex_encoding — 设置/获取多字节正则表达式的当前编码
///
/// # PHP 签名
/// mb_regex_encoding(?string $encoding = null): string|bool
///
/// # 参数
/// - args[0]: 可选的编码名称
///
/// # 返回
/// 设置时返回成功/失败，获取时返回当前编码
pub fn builtin_mb_regex_encoding(args: &[Value]) -> Result<Value> {
    // 检查是否设置了编码
    if args.is_empty() {
        return Ok(Value::String("UTF-8".to_string()));
    }
    
    // 设置编码
    Ok(Value::Bool(true))
}

/// mb_check_encoding — 检查字符串是否为有效的编码
///
/// # PHP 签名
/// mb_check_encoding(?string $value = null, ?string $encoding = null): bool
///
/// # 参数
/// - args[0]: 可选的字符串值
/// - args[1]: 可选的编码名称
///
/// # 返回
/// 是否为有效的编码
pub fn builtin_mb_check_encoding(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(true));
    }
    
    // 获取字符串
    let input = match &args[0] {
        Value::String(s) => s.as_str(),
        Value::Null => return Ok(Value::Bool(true)),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 检查是否为有效的 UTF-8
    // Rust 字符串保证是有效的 UTF-8
    let is_valid = input.is_ascii() || input.chars().all(|c| c.len_utf8() > 0);
    
    Ok(Value::Bool(is_valid))
}

/// mb_convert_case — 对字符串进行大小写转换
///
/// # PHP 签名
/// mb_convert_case(string $string, int $mode, ?string $encoding = null): string
///
/// # 参数
/// - args[0]: 输入字符串
/// - args[1]: 转换模式（MB_CASE_UPPER, MB_CASE_LOWER, MB_CASE_TITLE）
/// - args[2]: 可选的编码名称
///
/// # 返回
/// 转换后的字符串
pub fn builtin_mb_convert_case(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::String(String::new()));
    }
    
    // 获取输入字符串
    let input = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::String(String::new())),
    };
    
    // 获取转换模式
    let mode = match &args[1] {
        Value::Int(i) => *i,
        _ => 0,
    };
    
    // 根据模式转换
    let result = match mode {
        0 => input.to_uppercase(),      // MB_CASE_UPPER
        1 => input.to_lowercase(),      // MB_CASE_LOWER
        2 => {                          // MB_CASE_TITLE
            // 将每个单词的首字母大写
            let mut result = String::new();
            let mut capitalize_next = true;
            for c in input.chars() {
                if c.is_whitespace() {
                    capitalize_next = true;
                    result.push(c);
                } else if capitalize_next {
                    for uc in c.to_uppercase() {
                        result.push(uc);
                    }
                    capitalize_next = false;
                } else {
                    for lc in c.to_lowercase() {
                        result.push(lc);
                    }
                }
            }
            result
        }
        _ => input.to_string(),
    };
    
    Ok(Value::String(result))
}

/// mb_convert_kana — 将假名转换为其他假名（"zen-kaku"、"han-kaku"等）
///
/// # PHP 签名
/// mb_convert_kana(string $string, string $mode = "KV", ?string $encoding = null): string
///
/// # 参数
/// - args[0]: 输入字符串
/// - args[1]: 可选的转换模式
/// - args[2]: 可选的编码名称
///
/// # 返回
/// 转换后的字符串
pub fn builtin_mb_convert_kana(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::String(String::new()));
    }
    
    // 获取输入字符串
    let input = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::String(String::new())),
    };
    
    // 获取转换模式
    let _mode = if args.len() > 1 {
        match &args[1] {
            Value::String(s) => s.as_str(),
            _ => "KV",
        }
    } else {
        "KV"
    };
    
    // 简化实现：直接返回原字符串
    // 完整实现需要处理日文假名转换
    Ok(Value::String(input.to_string()))
}

/// mb_list_encodings — 返回支持的所有编码
///
/// # PHP 签名
/// mb_list_encodings(): array
///
/// # 返回
/// 支持的编码列表
pub fn builtin_mb_list_encodings(args: &[Value]) -> Result<Value> {
    // 忽略参数
    let _ = args;
    
    // 返回支持的编码列表
    let encodings = vec![
        Value::String("UTF-8".to_string()),
        Value::String("ASCII".to_string()),
        Value::String("ISO-8859-1".to_string()),
        Value::String("GBK".to_string()),
        Value::String("GB2312".to_string()),
        Value::String("GB18030".to_string()),
        Value::String("BIG5".to_string()),
        Value::String("Shift_JIS".to_string()),
        Value::String("EUC-JP".to_string()),
        Value::String("UTF-16".to_string()),
        Value::String("UTF-32".to_string()),
    ];
    
    Ok(Value::IndexedArray(encodings))
}

/// mb_encoding_aliases — 获取编码的别名
///
/// # PHP 签名
/// mb_encoding_aliases(string $encoding): array
///
/// # 参数
/// - args[0]: 编码名称
///
/// # 返回
/// 编码别名数组
pub fn builtin_mb_encoding_aliases(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::IndexedArray(vec![]));
    }
    
    // 获取编码名称
    let encoding = match &args[0] {
        Value::String(s) => s.to_lowercase(),
        _ => return Ok(Value::IndexedArray(vec![])),
    };
    
    // 返回编码别名
    let aliases = match encoding.as_str() {
        "utf-8" | "utf8" => vec![
            Value::String("utf8".to_string()),
            Value::String("UTF8".to_string()),
        ],
        "gbk" | "gb2312" => vec![
            Value::String("GB2312".to_string()),
            Value::String("gb2312".to_string()),
            Value::String("GB18030".to_string()),
        ],
        "big5" => vec![
            Value::String("Big5".to_string()),
            Value::String("BIG5".to_string()),
        ],
        "shift_jis" | "shift-jis" | "sjis" => vec![
            Value::String("Shift_JIS".to_string()),
            Value::String("SJIS".to_string()),
            Value::String("shift-jis".to_string()),
        ],
        _ => vec![],
    };
    
    Ok(Value::IndexedArray(aliases))
}

// ============================================================================
// 函数注册
// ============================================================================

/// 注册 mbstring 相关的内置函数
///
/// 将所有 mbstring 函数注册到内置函数映射表中
///
/// # 参数
/// - `map`: 内置函数映射表的可变引用
pub fn register_mbstring_functions(map: &mut HashMap<String, BuiltinFunction>) {
    // 字符串长度和子串函数
    map.insert("mb_strlen".to_string(), builtin_mb_strlen);
    map.insert("mb_substr".to_string(), builtin_mb_substr);
    map.insert("mb_strcut".to_string(), builtin_mb_substr); // 别名
    
    // 字符串查找函数
    map.insert("mb_strpos".to_string(), builtin_mb_strpos);
    map.insert("mb_strrpos".to_string(), builtin_mb_strrpos);
    map.insert("mb_stripos".to_string(), builtin_mb_stripos);
    map.insert("mb_strripos".to_string(), builtin_mb_strripos);
    map.insert("mb_strstr".to_string(), builtin_mb_strstr);
    map.insert("mb_stristr".to_string(), builtin_mb_stristr);
    map.insert("mb_strrchr".to_string(), builtin_mb_strrchr);
    map.insert("mb_strrichr".to_string(), builtin_mb_strrchr); // 简化实现
    map.insert("mb_substr_count".to_string(), builtin_mb_substr_count);
    
    // 大小写转换函数
    map.insert("mb_strtolower".to_string(), builtin_mb_strtolower);
    map.insert("mb_strtoupper".to_string(), builtin_mb_strtoupper);
    map.insert("mb_convert_case".to_string(), builtin_mb_convert_case);
    map.insert("mb_convert_kana".to_string(), builtin_mb_convert_kana);
    
    // 宽度相关函数
    map.insert("mb_strwidth".to_string(), builtin_mb_strwidth);
    map.insert("mb_strimwidth".to_string(), builtin_mb_strimwidth);
    
    // 编码转换函数
    map.insert("mb_convert_encoding".to_string(), builtin_mb_convert_encoding);
    map.insert("mb_detect_encoding".to_string(), builtin_mb_detect_encoding);
    map.insert("mb_check_encoding".to_string(), builtin_mb_check_encoding);
    map.insert("mb_list_encodings".to_string(), builtin_mb_list_encodings);
    map.insert("mb_encoding_aliases".to_string(), builtin_mb_encoding_aliases);
    
    // 编码设置函数
    map.insert("mb_internal_encoding".to_string(), builtin_mb_internal_encoding);
    map.insert("mb_http_output".to_string(), builtin_mb_http_output);
    map.insert("mb_http_input".to_string(), builtin_mb_http_input);
    map.insert("mb_language".to_string(), builtin_mb_language);
    map.insert("mb_substitute_character".to_string(), builtin_mb_substitute_character);
    map.insert("mb_regex_encoding".to_string(), builtin_mb_regex_encoding);
    
    // 正则表达式函数
    map.insert("mb_split".to_string(), builtin_mb_split);
    map.insert("mb_ereg".to_string(), builtin_mb_ereg);
    map.insert("mb_eregi".to_string(), builtin_mb_eregi);
    map.insert("mb_ereg_replace".to_string(), builtin_mb_ereg_replace);
    map.insert("mb_eregi_replace".to_string(), builtin_mb_eregi_replace);
}
