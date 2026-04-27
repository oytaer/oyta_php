//! 多字节字符串操作模块
//!
//! 提供完整的 PHP mbstring 字符串操作功能

use serde::{Deserialize, Serialize};
use std::fmt;

/// 多字节字符串操作
///
/// 提供多字节安全的字符串操作函数
pub struct MbString;

impl MbString {
    /// 获取字符串长度（字符数）
    ///
    /// 对应 PHP 的 mb_strlen() 函数
    ///
    /// # 参数
    /// - `input`: 输入字符串
    /// - `encoding`: 字符编码（默认 UTF-8）
    ///
    /// # 返回
    /// 字符串的字符数
    pub fn strlen(input: &str, _encoding: Option<&str>) -> usize {
        // Rust 的字符串已经是 UTF-8，直接计算字符数
        input.chars().count()
    }
    
    /// 获取子字符串
    ///
    /// 对应 PHP 的 mb_substr() 函数
    ///
    /// # 参数
    /// - `input`: 输入字符串
    /// - `start`: 起始位置（可以为负数）
    /// - `length`: 长度（可选）
    /// - `encoding`: 字符编码
    ///
    /// # 返回
    /// 子字符串
    pub fn substr(input: &str, start: i32, length: Option<usize>, _encoding: Option<&str>) -> String {
        let chars: Vec<char> = input.chars().collect();
        let char_count = chars.len();
        
        // 计算起始位置
        let start_pos = if start < 0 {
            char_count.saturating_sub(start.abs() as usize)
        } else {
            start as usize
        };
        
        // 确保起始位置不超过字符串长度
        if start_pos >= char_count {
            return String::new();
        }
        
        // 计算结束位置
        let end_pos = match length {
            Some(len) => (start_pos + len).min(char_count),
            None => char_count,
        };
        
        // 提取子字符串
        chars[start_pos..end_pos].iter().collect()
    }
    
    /// 查找子字符串位置
    ///
    /// 对应 PHP 的 mb_strpos() 函数
    ///
    /// # 参数
    /// - `haystack`: 主字符串
    /// - `needle`: 要查找的子字符串
    /// - `offset`: 起始偏移量
    /// - `encoding`: 字符编码
    ///
    /// # 返回
    /// 找到返回位置，未找到返回 None
    pub fn strpos(haystack: &str, needle: &str, offset: usize, _encoding: Option<&str>) -> Option<usize> {
        // 转换为字符数组
        let haystack_chars: Vec<char> = haystack.chars().collect();
        let needle_chars: Vec<char> = needle.chars().collect();
        
        // 检查偏移量
        if offset >= haystack_chars.len() {
            return None;
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
                return Some(i);
            }
        }
        
        None
    }
    
    /// 从后向前查找子字符串位置
    ///
    /// 对应 PHP 的 mb_strrpos() 函数
    pub fn strrpos(haystack: &str, needle: &str, offset: Option<usize>, _encoding: Option<&str>) -> Option<usize> {
        let haystack_chars: Vec<char> = haystack.chars().collect();
        let needle_chars: Vec<char> = needle.chars().collect();
        
        if needle_chars.is_empty() {
            return None;
        }
        
        // 计算搜索范围
        let start_pos = offset.unwrap_or(0);
        let search_end = haystack_chars.len().saturating_sub(needle_chars.len());
        
        // 从后向前搜索
        for i in (start_pos..=search_end).rev() {
            let mut matches = true;
            for (j, &needle_char) in needle_chars.iter().enumerate() {
                if haystack_chars[i + j] != needle_char {
                    matches = false;
                    break;
                }
            }
            
            if matches {
                return Some(i);
            }
        }
        
        None
    }
    
    /// 查找子字符串首次出现的位置（不区分大小写）
    ///
    /// 对应 PHP 的 mb_stripos() 函数
    pub fn stripos(haystack: &str, needle: &str, offset: usize, _encoding: Option<&str>) -> Option<usize> {
        let haystack_lower: String = haystack.to_lowercase();
        let needle_lower: String = needle.to_lowercase();
        
        Self::strpos(&haystack_lower, &needle_lower, offset, None)
    }
    
    /// 查找子字符串最后出现的位置（不区分大小写）
    ///
    /// 对应 PHP 的 mb_strripos() 函数
    pub fn strripos(haystack: &str, needle: &str, offset: Option<usize>, _encoding: Option<&str>) -> Option<usize> {
        let haystack_lower: String = haystack.to_lowercase();
        let needle_lower: String = needle.to_lowercase();
        
        Self::strrpos(&haystack_lower, &needle_lower, offset, None)
    }
    
    /// 转换为小写
    ///
    /// 对应 PHP 的 mb_strtolower() 函数
    pub fn strtolower(input: &str, _encoding: Option<&str>) -> String {
        input.to_lowercase()
    }
    
    /// 转换为大写
    ///
    /// 对应 PHP 的 mb_strtoupper() 函数
    pub fn strtoupper(input: &str, _encoding: Option<&str>) -> String {
        input.to_uppercase()
    }
    
    /// 首字母大写
    ///
    /// 对应 PHP 的 mb_convert_case() 函数的 MB_CASE_TITLE
    pub fn titlecase(input: &str, _encoding: Option<&str>) -> String {
        let mut result = String::new();
        let mut prev_is_space = true;
        
        for c in input.chars() {
            if c.is_whitespace() {
                result.push(c);
                prev_is_space = true;
            } else if prev_is_space {
                for uc in c.to_uppercase() {
                    result.push(uc);
                }
                prev_is_space = false;
            } else {
                for lc in c.to_lowercase() {
                    result.push(lc);
                }
            }
        }
        
        result
    }
    
    /// 大小写转换
    ///
    /// 对应 PHP 的 mb_convert_case() 函数
    pub fn convert_case(input: &str, mode: CaseMode, encoding: Option<&str>) -> String {
        match mode {
            CaseMode::Upper => Self::strtoupper(input, encoding),
            CaseMode::Lower => Self::strtolower(input, encoding),
            CaseMode::Title => Self::titlecase(input, encoding),
            CaseMode::Fold => input.to_lowercase(), // 简化实现
            CaseMode::UpperSimple => input.to_uppercase(),
            CaseMode::LowerSimple => input.to_lowercase(),
            CaseMode::TitleSimple => Self::titlecase(input, encoding),
            CaseMode::FoldSimple => input.to_lowercase(),
        }
    }
    
    /// 统计子字符串出现次数
    ///
    /// 对应 PHP 的 mb_substr_count() 函数
    pub fn substr_count(haystack: &str, needle: &str, _encoding: Option<&str>) -> usize {
        if needle.is_empty() {
            return 0;
        }
        
        let mut count = 0;
        let mut pos = 0;
        
        while let Some(p) = Self::strpos(haystack, needle, pos, None) {
            count += 1;
            pos = p + 1;
        }
        
        count
    }
    
    /// 分割字符串
    ///
    /// 对应 PHP 的 mb_split() 函数
    pub fn split(pattern: &str, string: &str, limit: i32) -> Vec<String> {
        // 简化实现：使用固定分隔符
        let max_parts = if limit > 0 { limit as usize } else { usize::MAX };
        
        let parts: Vec<&str> = string.splitn(max_parts, pattern).collect();
        parts.into_iter().map(|s| s.to_string()).collect()
    }
    
    /// 字符串填充
    ///
    /// 对应 PHP 的 mb_str_pad() 函数
    pub fn str_pad(
        input: &str,
        pad_length: usize,
        pad_string: &str,
        pad_type: PadType,
        _encoding: Option<&str>,
    ) -> String {
        let input_len = input.chars().count();
        
        if input_len >= pad_length {
            return input.to_string();
        }
        
        let pad_chars: Vec<char> = pad_string.chars().collect();
        if pad_chars.is_empty() {
            return input.to_string();
        }
        
        let total_pad = pad_length - input_len;
        
        match pad_type {
            PadType::Right => {
                let mut result = input.to_string();
                let mut pad_count = 0;
                while pad_count < total_pad {
                    for &c in &pad_chars {
                        if pad_count >= total_pad {
                            break;
                        }
                        result.push(c);
                        pad_count += 1;
                    }
                }
                result
            }
            PadType::Left => {
                let mut pad = String::new();
                let mut pad_count = 0;
                while pad_count < total_pad {
                    for &c in &pad_chars {
                        if pad_count >= total_pad {
                            break;
                        }
                        pad.push(c);
                        pad_count += 1;
                    }
                }
                format!("{}{}", pad, input)
            }
            PadType::Both => {
                let left_pad = total_pad / 2;
                let right_pad = total_pad - left_pad;
                
                let mut left = String::new();
                let mut pad_count = 0;
                while pad_count < left_pad {
                    for &c in &pad_chars {
                        if pad_count >= left_pad {
                            break;
                        }
                        left.push(c);
                        pad_count += 1;
                    }
                }
                
                let mut right = String::new();
                pad_count = 0;
                while pad_count < right_pad {
                    for &c in &pad_chars {
                        if pad_count >= right_pad {
                            break;
                        }
                        right.push(c);
                        pad_count += 1;
                    }
                }
                
                format!("{}{}{}", left, input, right)
            }
        }
    }
    
    /// 截取字符串（指定宽度）
    ///
    /// 对应 PHP 的 mb_strimwidth() 函数
    pub fn strimwidth(
        input: &str,
        start: i32,
        width: usize,
        trim_marker: &str,
        _encoding: Option<&str>,
    ) -> String {
        let chars: Vec<char> = input.chars().collect();
        let char_count = chars.len();
        
        // 计算起始位置
        let start_pos = if start < 0 {
            char_count.saturating_sub(start.abs() as usize)
        } else {
            start as usize
        };
        
        if start_pos >= char_count {
            return String::new();
        }
        
        // 计算截取宽度
        let trim_marker_len = trim_marker.chars().count();
        let available_width = width.saturating_sub(trim_marker_len);
        
        // 截取字符串
        let end_pos = (start_pos + available_width).min(char_count);
        let mut result: String = chars[start_pos..end_pos].iter().collect();
        
        // 如果截取后字符串变短，添加截断标记
        if end_pos < char_count {
            result.push_str(trim_marker);
        }
        
        result
    }
    
    /// 获取字符宽度
    ///
    /// 对应 PHP 的 mb_strwidth() 函数
    pub fn strwidth(input: &str, _encoding: Option<&str>) -> usize {
        let mut width = 0;
        
        for c in input.chars() {
            // 东亚字符宽度为 2
            if is_east_asian_width(c) {
                width += 2;
            } else {
                width += 1;
            }
        }
        
        width
    }
}

/// 判断字符是否为东亚宽字符
fn is_east_asian_width(c: char) -> bool {
    // CJK 统一汉字范围
    (c >= '\u{4E00}' && c <= '\u{9FFF}') ||
    // CJK 扩展 A
    (c >= '\u{3400}' && c <= '\u{4DBF}') ||
    // CJK 扩展 B-F
    (c >= '\u{20000}' && c <= '\u{2CEAF}') ||
    // 日文假名
    (c >= '\u{3040}' && c <= '\u{309F}') || // 平假名
    (c >= '\u{30A0}' && c <= '\u{30FF}') || // 片假名
    // 韩文字母
    (c >= '\u{AC00}' && c <= '\u{D7AF}') || // 韩文音节
    (c >= '\u{1100}' && c <= '\u{11FF}')    // 韩文字母
}

/// 大小写转换模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CaseMode {
    /// 转换为大写
    Upper,
    /// 转换为小写
    Lower,
    /// 首字母大写
    Title,
    /// 折叠大小写
    Fold,
    /// 简单大写
    UpperSimple,
    /// 简单小写
    LowerSimple,
    /// 简单首字母大写
    TitleSimple,
    /// 简单折叠
    FoldSimple,
}

/// 填充类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PadType {
    /// 右侧填充
    Right,
    /// 左侧填充
    Left,
    /// 两侧填充
    Both,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mb_strlen() {
        assert_eq!(MbString::strlen("Hello", None), 5);
        assert_eq!(MbString::strlen("你好世界", None), 4);
        assert_eq!(MbString::strlen("Hello 世界", None), 8);
    }
    
    #[test]
    fn test_mb_substr() {
        assert_eq!(MbString::substr("Hello World", 0, Some(5), None), "Hello");
        assert_eq!(MbString::substr("你好世界", 1, Some(2), None), "好世");
        assert_eq!(MbString::substr("Hello", -3, None, None), "llo");
    }
    
    #[test]
    fn test_mb_strpos() {
        assert_eq!(MbString::strpos("Hello World", "World", 0, None), Some(6));
        assert_eq!(MbString::strpos("你好世界", "世界", 0, None), Some(2));
        assert_eq!(MbString::strpos("Hello", "xyz", 0, None), None);
    }
    
    #[test]
    fn test_mb_strrpos() {
        assert_eq!(MbString::strrpos("Hello Hello", "Hello", None, None), Some(6));
    }
    
    #[test]
    fn test_mb_strtolower() {
        assert_eq!(MbString::strtolower("HELLO", None), "hello");
        assert_eq!(MbString::strtolower("HeLLo", None), "hello");
    }
    
    #[test]
    fn test_mb_strtoupper() {
        assert_eq!(MbString::strtoupper("hello", None), "HELLO");
        assert_eq!(MbString::strtoupper("HeLLo", None), "HELLO");
    }
    
    #[test]
    fn test_mb_substr_count() {
        assert_eq!(MbString::substr_count("Hello Hello Hello", "Hello", None), 3);
        assert_eq!(MbString::substr_count("aaaa", "aa", None), 2);
    }
    
    #[test]
    fn test_mb_str_pad() {
        let result = MbString::str_pad("Hello", 10, " ", PadType::Right, None);
        assert_eq!(result, "Hello     ");
        
        let result = MbString::str_pad("Hello", 10, " ", PadType::Left, None);
        assert_eq!(result, "     Hello");
    }
    
    #[test]
    fn test_mb_strwidth() {
        assert_eq!(MbString::strwidth("Hello", None), 5);
        assert_eq!(MbString::strwidth("你好", None), 4); // 每个中文字符宽度为 2
    }
}
