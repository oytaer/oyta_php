//! 多字节正则表达式模块
//!
//! 提供完整的 PHP mbstring 正则表达式功能

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 多字节正则表达式
///
/// 提供多字节安全的正则表达式操作
pub struct MbRegex;

impl MbRegex {
    /// 正则匹配
    ///
    /// 对应 PHP 的 mb_ereg() 函数
    ///
    /// # 参数
    /// - `pattern`: 正则表达式模式
    /// - `string`: 要匹配的字符串
    /// - `options`: 可选的匹配选项
    ///
    /// # 返回
    /// 匹配结果
    pub fn ereg(pattern: &str, string: &str, options: Option<&str>) -> Option<MatchResult> {
        // 转换 PHP 正则语法到 Rust 正则语法
        let rust_pattern = Self::convert_pattern(pattern, options);
        
        // 编译正则表达式
        let re = match Regex::new(&rust_pattern) {
            Ok(re) => re,
            Err(_) => return None,
        };
        
        // 执行匹配
        if let Some(caps) = re.captures(string) {
            let mut matches = Vec::new();
            
            // 获取所有捕获组
            for (i, cap) in caps.iter().enumerate() {
                if let Some(m) = cap {
                    matches.push(CaptureGroup {
                        index: i,
                        text: m.as_str().to_string(),
                        start: m.start(),
                        end: m.end(),
                    });
                }
            }
            
            Some(MatchResult {
                full_match: caps.get(0).map(|m| m.as_str().to_string()).unwrap_or_default(),
                groups: matches,
            })
        } else {
            None
        }
    }
    
    /// 正则匹配（不区分大小写）
    ///
    /// 对应 PHP 的 mb_eregi() 函数
    pub fn eregi(pattern: &str, string: &str) -> Option<MatchResult> {
        Self::ereg(pattern, string, Some("i"))
    }
    
    /// 正则替换
    ///
    /// 对应 PHP 的 mb_ereg_replace() 函数
    ///
    /// # 参数
    /// - `pattern`: 正则表达式模式
    /// - `replacement`: 替换字符串
    /// - `string`: 要处理的字符串
    /// - `options`: 可选的匹配选项
    ///
    /// # 返回
    /// 替换后的字符串
    pub fn ereg_replace(pattern: &str, replacement: &str, string: &str, options: Option<&str>) -> String {
        let rust_pattern = Self::convert_pattern(pattern, options);
        
        let re = match Regex::new(&rust_pattern) {
            Ok(re) => re,
            Err(_) => return string.to_string(),
        };
        
        // 执行替换
        re.replace_all(string, replacement).to_string()
    }
    
    /// 正则替换（不区分大小写）
    ///
    /// 对应 PHP 的 mb_eregi_replace() 函数
    pub fn eregi_replace(pattern: &str, replacement: &str, string: &str) -> String {
        Self::ereg_replace(pattern, replacement, string, Some("i"))
    }
    
    /// 正则替换（使用回调函数）
    ///
    /// 对应 PHP 的 mb_ereg_replace_callback() 函数
    pub fn ereg_replace_callback<F>(
        pattern: &str,
        string: &str,
        callback: F,
        options: Option<&str>,
    ) -> String
    where
        F: Fn(&MatchResult) -> String,
    {
        let rust_pattern = Self::convert_pattern(pattern, options);
        
        let re = match Regex::new(&rust_pattern) {
            Ok(re) => re,
            Err(_) => return string.to_string(),
        };
        
        // 执行替换
        re.replace_all(string, |caps: &regex::Captures| {
            let mut matches = Vec::new();
            for (i, cap) in caps.iter().enumerate() {
                if let Some(m) = cap {
                    matches.push(CaptureGroup {
                        index: i,
                        text: m.as_str().to_string(),
                        start: m.start(),
                        end: m.end(),
                    });
                }
            }
            
            let result = MatchResult {
                full_match: caps.get(0).map(|m| m.as_str().to_string()).unwrap_or_default(),
                groups: matches,
            };
            
            callback(&result)
        }).to_string()
    }
    
    /// 正则分割
    ///
    /// 对应 PHP 的 mb_split() 函数
    ///
    /// # 参数
    /// - `pattern`: 正则表达式模式
    /// - `string`: 要分割的字符串
    /// - `limit`: 最大分割数量（-1 表示无限制）
    ///
    /// # 返回
    /// 分割后的字符串数组
    pub fn split(pattern: &str, string: &str, limit: i32) -> Vec<String> {
        let rust_pattern = Self::convert_pattern(pattern, None);
        
        let re = match Regex::new(&rust_pattern) {
            Ok(re) => re,
            Err(_) => return vec![string.to_string()],
        };
        
        if limit > 0 {
            re.splitn(string, limit as usize)
                .map(|s| s.to_string())
                .collect()
        } else {
            re.split(string)
                .map(|s| s.to_string())
                .collect()
        }
    }
    
    /// 查找所有匹配
    ///
    /// 对应 PHP 的 mb_ereg_match() 函数
    pub fn ereg_match(pattern: &str, string: &str) -> bool {
        let rust_pattern = Self::convert_pattern(pattern, None);
        
        let re = match Regex::new(&rust_pattern) {
            Ok(re) => re,
            Err(_) => return false,
        };
        
        re.is_match(string)
    }
    
    /// 查找所有匹配（返回所有结果）
    ///
    /// 对应 PHP 的 mb_ereg_search() 系列函数
    pub fn find_all(pattern: &str, string: &str) -> Vec<MatchResult> {
        let rust_pattern = Self::convert_pattern(pattern, None);
        
        let re = match Regex::new(&rust_pattern) {
            Ok(re) => re,
            Err(_) => return Vec::new(),
        };
        
        let mut results = Vec::new();
        
        for caps in re.captures_iter(string) {
            let mut matches = Vec::new();
            
            for (i, cap) in caps.iter().enumerate() {
                if let Some(m) = cap {
                    matches.push(CaptureGroup {
                        index: i,
                        text: m.as_str().to_string(),
                        start: m.start(),
                        end: m.end(),
                    });
                }
            }
            
            results.push(MatchResult {
                full_match: caps.get(0).map(|m| m.as_str().to_string()).unwrap_or_default(),
                groups: matches,
            });
        }
        
        results
    }
    
    /// 转换 PHP 正则语法到 Rust 正则语法
    fn convert_pattern(pattern: &str, options: Option<&str>) -> String {
        let mut result = String::new();
        
        // 处理选项
        if let Some(opts) = options {
            if opts.contains('i') {
                result.push_str("(?i)");
            }
            if opts.contains('m') {
                result.push_str("(?m)");
            }
            if opts.contains('s') {
                result.push_str("(?s)");
            }
            if opts.contains('x') {
                result.push_str("(?x)");
            }
        }
        
        // 转换 PHP 特有的正则语法
        let converted = pattern
            // PHP 的 \d 等在 Rust 中相同
            // 转换 PHP 的字符类
            .replace("[:digit:]", "\\d")
            .replace("[:alpha:]", "[a-zA-Z]")
            .replace("[:alnum:]", "[a-zA-Z0-9]")
            .replace("[:space:]", "\\s")
            .replace("[:upper:]", "[A-Z]")
            .replace("[:lower:]", "[a-z]");
        
        result.push_str(&converted);
        result
    }
    
    /// 获取正则表达式选项
    pub fn get_options(options: &str) -> RegexOptions {
        let mut opts = RegexOptions::default();
        
        if options.contains('i') {
            opts.case_insensitive = true;
        }
        if options.contains('m') {
            opts.multiline = true;
        }
        if options.contains('s') {
            opts.dotall = true;
        }
        if options.contains('x') {
            opts.extended = true;
        }
        if options.contains('g') {
            opts.global = true;
        }
        
        opts
    }
}

/// 匹配结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchResult {
    /// 完整匹配的文本
    pub full_match: String,
    /// 捕获组列表
    pub groups: Vec<CaptureGroup>,
}

impl MatchResult {
    /// 获取完整匹配文本
    pub fn get_match(&self) -> &str {
        &self.full_match
    }
    
    /// 获取指定索引的捕获组
    pub fn get_group(&self, index: usize) -> Option<&CaptureGroup> {
        self.groups.get(index)
    }
    
    /// 获取捕获组数量
    pub fn group_count(&self) -> usize {
        self.groups.len().saturating_sub(1) // 排除完整匹配
    }
}

/// 捕获组
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureGroup {
    /// 组索引
    pub index: usize,
    /// 匹配文本
    pub text: String,
    /// 起始位置
    pub start: usize,
    /// 结束位置
    pub end: usize,
}

impl CaptureGroup {
    /// 获取匹配文本
    pub fn as_str(&self) -> &str {
        &self.text
    }
    
    /// 获取匹配长度
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// 正则表达式选项
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegexOptions {
    /// 不区分大小写
    pub case_insensitive: bool,
    /// 多行模式
    pub multiline: bool,
    /// 点匹配换行
    pub dotall: bool,
    /// 扩展模式
    pub extended: bool,
    /// 全局匹配
    pub global: bool,
}

impl Default for RegexOptions {
    fn default() -> Self {
        Self {
            case_insensitive: false,
            multiline: false,
            dotall: false,
            extended: false,
            global: false,
        }
    }
}

/// 设置正则编码
///
/// 对应 PHP 的 mb_regex_set_options() 函数
pub fn mb_regex_set_options(options: &str) -> RegexOptions {
    MbRegex::get_options(options)
}

/// 获取正则编码
///
/// 对应 PHP 的 mb_regex_encoding() 函数
pub fn mb_regex_encoding() -> String {
    "UTF-8".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mb_ereg() {
        let result = MbRegex::ereg("world", "Hello, world!", None);
        assert!(result.is_some());
        
        let match_result = result.unwrap();
        assert_eq!(match_result.full_match, "world");
    }
    
    #[test]
    fn test_mb_eregi() {
        let result = MbRegex::eregi("WORLD", "Hello, world!");
        assert!(result.is_some());
    }
    
    #[test]
    fn test_mb_ereg_replace() {
        let result = MbRegex::ereg_replace("world", "Rust", "Hello, world!", None);
        assert_eq!(result, "Hello, Rust!");
    }
    
    #[test]
    fn test_mb_split() {
        let result = MbRegex::split(r"\s+", "Hello World Test", -1);
        assert_eq!(result, vec!["Hello", "World", "Test"]);
    }
    
    #[test]
    fn test_mb_ereg_match() {
        assert!(MbRegex::ereg_match(r"\d+", "Hello 123 World"));
        assert!(!MbRegex::ereg_match(r"\d+", "Hello World"));
    }
    
    #[test]
    fn test_find_all() {
        let results = MbRegex::find_all(r"\d+", "a1b2c3");
        assert_eq!(results.len(), 3);
    }
    
    #[test]
    fn test_convert_pattern() {
        let converted = MbRegex::convert_pattern("[:digit:]+", None);
        assert_eq!(converted, r"\d+");
    }
}
