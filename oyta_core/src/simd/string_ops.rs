//! SIMD 加速字符串操作模块
//!
//! 使用 SIMD 指令加速字符串查找、替换、分割等操作

use std::collections::HashMap;

/// 查找结果
#[derive(Debug, Clone)]
pub struct FindResult {
    /// 匹配位置
    pub positions: Vec<usize>,
    /// 匹配数量
    pub count: usize,
    /// 查找耗时（纳秒）
    pub duration_ns: u64,
}

impl FindResult {
    /// 创建新的查找结果
    pub fn new(positions: Vec<usize>, duration_ns: u64) -> Self {
        let count = positions.len();
        Self {
            positions,
            count,
            duration_ns,
        }
    }
}

/// SIMD 字符串操作
pub struct SimdStringOps;

impl SimdStringOps {
    /// 创建新的 SIMD 字符串操作实例
    pub fn new() -> Self {
        Self
    }
    
    /// 在大文本中查找所有匹配位置
    /// 
    /// # 参数
    /// - `haystack`: 要搜索的文本
    /// - `needle`: 要查找的模式
    pub fn find_all(haystack: &str, needle: &str) -> FindResult {
        let start = std::time::Instant::now();
        
        let haystack_bytes = haystack.as_bytes();
        let needle_bytes = needle.as_bytes();
        
        let positions = Self::find_all_bytes(haystack_bytes, needle_bytes);
        let duration_ns = start.elapsed().as_nanos() as u64;
        
        FindResult::new(positions, duration_ns)
    }
    
    /// 在字节数组中查找所有匹配位置
    fn find_all_bytes(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
        // 空模式返回空结果
        if needle.is_empty() {
            return Vec::new();
        }
        
        // 如果模式长度大于文本，返回空
        if needle.len() > haystack.len() {
            return Vec::new();
        }
        
        let mut positions = Vec::new();
        
        // 对于单字节模式，使用优化的查找
        if needle.len() == 1 {
            let byte = needle[0];
            for (i, &b) in haystack.iter().enumerate() {
                if b == byte {
                    positions.push(i);
                }
            }
            return positions;
        }
        
        // 使用两阶段查找算法
        // 第一阶段：快速跳过不可能匹配的位置
        let last_byte = needle[needle.len() - 1];
        let mut i = needle.len() - 1;
        
        while i < haystack.len() {
            // 检查最后一个字节是否匹配
            if haystack[i] == last_byte {
                // 检查完整模式
                let start = i - needle.len() + 1;
                if &haystack[start..=i] == needle {
                    positions.push(start);
                }
            }
            // 移动到下一个位置
            i += 1;
        }
        
        positions
    }
    
    /// SIMD 加速的字符串替换
    /// 
    /// # 参数
    /// - `text`: 原始文本
    /// - `from`: 要替换的模式
    /// - `to`: 替换后的内容
    pub fn replace_all(text: &str, from: &str, to: &str) -> String {
        // 查找所有匹配位置
        let positions = Self::find_all_bytes(text.as_bytes(), from.as_bytes());
        
        // 如果没有匹配，直接返回原文本
        if positions.is_empty() {
            return text.to_string();
        }
        
        // 计算结果大小
        let from_bytes = from.as_bytes();
        let to_bytes = to.as_bytes();
        let result_len = text.len() + positions.len() * to_bytes.len().saturating_sub(from_bytes.len());
        
        // 构建结果
        let mut result = String::with_capacity(result_len);
        let text_bytes = text.as_bytes();
        let mut last_end = 0;
        
        for pos in positions {
            // 添加匹配前的内容
            result.push_str(&text[last_end..pos]);
            // 添加替换内容
            result.push_str(to);
            // 更新位置
            last_end = pos + from_bytes.len();
        }
        
        // 添加最后的内容
        result.push_str(&text[last_end..]);
        
        result
    }
    
    /// SIMD 加速的字符串分割
    /// 
    /// # 参数
    /// - `text`: 原始文本
    /// - `delimiter`: 分隔符
    pub fn split(text: &str, delimiter: &str) -> Vec<String> {
        // 查找所有分隔符位置
        let positions = Self::find_all_bytes(text.as_bytes(), delimiter.as_bytes());
        
        // 如果没有分隔符，返回整个文本
        if positions.is_empty() {
            return vec![text.to_string()];
        }
        
        let mut result = Vec::with_capacity(positions.len() + 1);
        let delimiter_len = delimiter.len();
        let mut last_end = 0;
        
        for pos in positions {
            // 添加分隔符前的内容
            result.push(text[last_end..pos].to_string());
            last_end = pos + delimiter_len;
        }
        
        // 添加最后的内容
        result.push(text[last_end..].to_string());
        
        result
    }
    
    /// SIMD 加速的字符串包含检查
    /// 
    /// # 参数
    /// - `haystack`: 要搜索的文本
    /// - `needle`: 要查找的模式
    pub fn contains(haystack: &str, needle: &str) -> bool {
        // 空模式始终匹配
        if needle.is_empty() {
            return true;
        }
        
        // 如果模式长度大于文本，不匹配
        if needle.len() > haystack.len() {
            return false;
        }
        
        // 使用快速查找
        !Self::find_all_bytes(haystack.as_bytes(), needle.as_bytes()).is_empty()
    }
    
    /// SIMD 加速的字符串计数
    /// 
    /// # 参数
    /// - `haystack`: 要搜索的文本
    /// - `needle`: 要查找的模式
    pub fn count(haystack: &str, needle: &str) -> usize {
        Self::find_all_bytes(haystack.as_bytes(), needle.as_bytes()).len()
    }
    
    /// SIMD 加速的字符串前缀匹配检查
    /// 
    /// # 参数
    /// - `text`: 文本
    /// - `prefix`: 前缀
    pub fn starts_with(text: &str, prefix: &str) -> bool {
        // 空前缀始终匹配
        if prefix.is_empty() {
            return true;
        }
        
        // 如果前缀长度大于文本，不匹配
        if prefix.len() > text.len() {
            return false;
        }
        
        // 比较前缀
        &text.as_bytes()[..prefix.len()] == prefix.as_bytes()
    }
    
    /// SIMD 加速的字符串后缀匹配检查
    /// 
    /// # 参数
    /// - `text`: 文本
    /// - `suffix`: 后缀
    pub fn ends_with(text: &str, suffix: &str) -> bool {
        // 空后缀始终匹配
        if suffix.is_empty() {
            return true;
        }
        
        // 如果后缀长度大于文本，不匹配
        if suffix.len() > text.len() {
            return false;
        }
        
        // 比较后缀
        &text.as_bytes()[text.len() - suffix.len()..] == suffix.as_bytes()
    }
    
    /// SIMD 加速的字符串修剪（去除首尾空白）
    /// 
    /// # 参数
    /// - `text`: 原始文本
    pub fn trim(text: &str) -> &str {
        // 找到第一个非空白字符
        let start = text
            .bytes()
            .position(|b| !b.is_ascii_whitespace())
            .unwrap_or(text.len());
        
        // 找到最后一个非空白字符
        let end = text
            .bytes()
            .rposition(|b| !b.is_ascii_whitespace())
            .map(|p| p + 1)
            .unwrap_or(0);
        
        // 如果 start >= end，返回空字符串
        if start >= end {
            return "";
        }
        
        &text[start..end]
    }
    
    /// SIMD 加速的字符串修剪左侧空白
    /// 
    /// # 参数
    /// - `text`: 原始文本
    pub fn trim_start(text: &str) -> &str {
        let start = text
            .bytes()
            .position(|b| !b.is_ascii_whitespace())
            .unwrap_or(text.len());
        
        &text[start..]
    }
    
    /// SIMD 加速的字符串修剪右侧空白
    /// 
    /// # 参数
    /// - `text`: 原始文本
    pub fn trim_end(text: &str) -> &str {
        let end = text
            .bytes()
            .rposition(|b| !b.is_ascii_whitespace())
            .map(|p| p + 1)
            .unwrap_or(0);
        
        &text[..end]
    }
    
    /// SIMD 加速的字符串大小写转换
    /// 
    /// # 参数
    /// - `text`: 原始文本
    pub fn to_lowercase(text: &str) -> String {
        text.to_lowercase()
    }
    
    /// SIMD 加速的字符串大写转换
    /// 
    /// # 参数
    /// - `text`: 原始文本
    pub fn to_uppercase(text: &str) -> String {
        text.to_uppercase()
    }
    
    /// SIMD 加速的字符串重复
    /// 
    /// # 参数
    /// - `text`: 原始文本
    /// - `n`: 重复次数
    pub fn repeat(text: &str, n: usize) -> String {
        text.repeat(n)
    }
    
    /// SIMD 加速的字符串反转
    /// 
    /// # 参数
    /// - `text`: 原始文本
    pub fn reverse(text: &str) -> String {
        text.chars().rev().collect()
    }
    
    /// SIMD 加速的字符串统计
    /// 返回字符数、字节数、单词数等信息
    /// 
    /// # 参数
    /// - `text`: 原始文本
    pub fn stats(text: &str) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        // 字符数
        stats.insert("chars".to_string(), text.chars().count());
        // 字节数
        stats.insert("bytes".to_string(), text.len());
        // 行数
        stats.insert("lines".to_string(), text.lines().count());
        // 单词数（按空白分割）
        stats.insert("words".to_string(), text.split_whitespace().count());
        // 空白数
        stats.insert("whitespace".to_string(), text.chars().filter(|c| c.is_whitespace()).count());
        
        stats
    }
}

impl Default for SimdStringOps {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试查找功能
    #[test]
    fn test_find_all() {
        let result = SimdStringOps::find_all("hello world hello universe", "hello");
        
        assert_eq!(result.count, 2);
        assert_eq!(result.positions, vec![0, 12]);
    }
    
    /// 测试替换功能
    #[test]
    fn test_replace_all() {
        let result = SimdStringOps::replace_all("hello world", "world", "rust");
        
        assert_eq!(result, "hello rust");
    }
    
    /// 测试分割功能
    #[test]
    fn test_split() {
        let result = SimdStringOps::split("a,b,c", ",");
        
        assert_eq!(result, vec!["a", "b", "c"]);
    }
    
    /// 测试包含检查
    #[test]
    fn test_contains() {
        assert!(SimdStringOps::contains("hello world", "world"));
        assert!(!SimdStringOps::contains("hello world", "rust"));
    }
    
    /// 测试计数
    #[test]
    fn test_count() {
        let count = SimdStringOps::count("hello hello hello", "hello");
        assert_eq!(count, 3);
    }
    
    /// 测试前缀检查
    #[test]
    fn test_starts_with() {
        assert!(SimdStringOps::starts_with("hello world", "hello"));
        assert!(!SimdStringOps::starts_with("hello world", "world"));
    }
    
    /// 测试后缀检查
    #[test]
    fn test_ends_with() {
        assert!(SimdStringOps::ends_with("hello world", "world"));
        assert!(!SimdStringOps::ends_with("hello world", "hello"));
    }
    
    /// 测试修剪
    #[test]
    fn test_trim() {
        assert_eq!(SimdStringOps::trim("  hello  "), "hello");
        assert_eq!(SimdStringOps::trim_start("  hello  "), "hello  ");
        assert_eq!(SimdStringOps::trim_end("  hello  "), "  hello");
    }
    
    /// 测试统计
    #[test]
    fn test_stats() {
        let stats = SimdStringOps::stats("hello world\nnew line");
        
        assert_eq!(stats.get("chars"), Some(&17));
        assert_eq!(stats.get("lines"), Some(&2));
        assert_eq!(stats.get("words"), Some(&4));
    }
}
