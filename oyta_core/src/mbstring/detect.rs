//! 编码检测功能模块
//!
//! 提供字符编码自动检测功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::encoding::{MbEncoding, MbEncodingList, MbEncodingOrder};

/// 编码检测器
///
/// 自动检测字符串的字符编码
#[derive(Debug, Clone)]
pub struct MbDetector {
    /// 检测顺序
    pub order: MbEncodingOrder,
    /// 严格模式
    pub strict: bool,
}

impl MbDetector {
    /// 创建新的检测器
    pub fn new() -> Self {
        Self {
            order: MbEncodingOrder::default_order(),
            strict: false,
        }
    }
    
    /// 使用指定编码顺序创建检测器
    pub fn with_order(order: MbEncodingOrder) -> Self {
        Self {
            order,
            strict: false,
        }
    }
    
    /// 设置严格模式
    pub fn set_strict(&mut self, strict: bool) {
        self.strict = strict;
    }
    
    /// 检测字符串编码
    ///
    /// 对应 PHP 的 mb_detect_encoding() 函数
    ///
    /// # 参数
    /// - `input`: 输入字符串
    ///
    /// # 返回
    /// 检测到的编码名称
    pub fn detect(&self, input: &str) -> Option<String> {
        // 如果输入为空，无法检测
        if input.is_empty() {
            return None;
        }
        
        let bytes = input.as_bytes();
        
        // 按顺序尝试每个编码
        for encoding_name in &self.order.encodings {
            if self.try_encoding(bytes, encoding_name) {
                return Some(encoding_name.clone());
            }
        }
        
        None
    }
    
    /// 检测编码（返回所有可能的编码）
    ///
    /// 对应 PHP 的 mb_detect_order() 相关功能
    pub fn detect_all(&self, input: &str) -> Vec<String> {
        let mut results = Vec::new();
        
        if input.is_empty() {
            return results;
        }
        
        let bytes = input.as_bytes();
        
        for encoding_name in &self.order.encodings {
            if self.try_encoding(bytes, encoding_name) {
                results.push(encoding_name.clone());
            }
        }
        
        results
    }
    
    /// 尝试使用指定编码解码
    fn try_encoding(&self, bytes: &[u8], encoding_name: &str) -> bool {
        match encoding_name.to_uppercase().as_str() {
            "UTF-8" => self.try_utf8(bytes),
            "GBK" | "GB2312" => self.try_gbk(bytes),
            "BIG5" => self.try_big5(bytes),
            "EUC-JP" => self.try_euc_jp(bytes),
            "SHIFT_JIS" | "SJIS" => self.try_shift_jis(bytes),
            "EUC-KR" => self.try_euc_kr(bytes),
            "ISO-8859-1" => true, // ISO-8859-1 总是有效
            "ASCII" => self.try_ascii(bytes),
            _ => false,
        }
    }
    
    /// 尝试 UTF-8 编码
    fn try_utf8(&self, bytes: &[u8]) -> bool {
        // 使用 std::str::from_utf8 验证
        match std::str::from_utf8(bytes) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
    
    /// 尝试 ASCII 编码
    fn try_ascii(&self, bytes: &[u8]) -> bool {
        bytes.iter().all(|&b| b < 128)
    }
    
    /// 尝试 GBK 编码
    fn try_gbk(&self, bytes: &[u8]) -> bool {
        // 简单验证：检查是否有 GBK 特征字节
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            
            // ASCII 范围
            if b < 0x80 {
                i += 1;
                continue;
            }
            
            // GBK 双字节范围
            if i + 1 < bytes.len() {
                let b2 = bytes[i + 1];
                
                // 第一字节：0x81-0xFE
                // 第二字节：0x40-0xFE（排除 0x7F）
                if (b >= 0x81 && b <= 0xFE) && 
                   ((b2 >= 0x40 && b2 <= 0x7E) || (b2 >= 0x80 && b2 <= 0xFE)) {
                    i += 2;
                    continue;
                }
            }
            
            // 无效的 GBK 字节序列
            if self.strict {
                return false;
            }
            i += 1;
        }
        
        true
    }
    
    /// 尝试 Big5 编码
    fn try_big5(&self, bytes: &[u8]) -> bool {
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            
            // ASCII 范围
            if b < 0x80 {
                i += 1;
                continue;
            }
            
            // Big5 双字节范围
            if i + 1 < bytes.len() {
                let b2 = bytes[i + 1];
                
                // 第一字节：0x81-0xFE
                // 第二字节：0x40-0x7E 或 0xA1-0xFE
                if (b >= 0x81 && b <= 0xFE) && 
                   ((b2 >= 0x40 && b2 <= 0x7E) || (b2 >= 0xA1 && b2 <= 0xFE)) {
                    i += 2;
                    continue;
                }
            }
            
            if self.strict {
                return false;
            }
            i += 1;
        }
        
        true
    }
    
    /// 尝试 EUC-JP 编码
    fn try_euc_jp(&self, bytes: &[u8]) -> bool {
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            
            // ASCII 范围
            if b < 0x80 {
                i += 1;
                continue;
            }
            
            // EUC-JP 双字节范围
            if i + 1 < bytes.len() {
                let b2 = bytes[i + 1];
                
                // 第一字节：0xA1-0xFE
                // 第二字节：0xA1-0xFE
                if (b >= 0xA1 && b <= 0xFE) && (b2 >= 0xA1 && b2 <= 0xFE) {
                    i += 2;
                    continue;
                }
                
                // JIS X 0212（三字节）
                if b == 0x8F && i + 2 < bytes.len() {
                    let b3 = bytes[i + 2];
                    if (b2 >= 0xA1 && b2 <= 0xFE) && (b3 >= 0xA1 && b3 <= 0xFE) {
                        i += 3;
                        continue;
                    }
                }
            }
            
            if self.strict {
                return false;
            }
            i += 1;
        }
        
        true
    }
    
    /// 尝试 Shift_JIS 编码
    fn try_shift_jis(&self, bytes: &[u8]) -> bool {
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            
            // ASCII 范围
            if b < 0x80 || (b >= 0xA1 && b <= 0xDF) {
                i += 1;
                continue;
            }
            
            // Shift_JIS 双字节范围
            if i + 1 < bytes.len() {
                let b2 = bytes[i + 1];
                
                // 第一字节：0x81-0x9F 或 0xE0-0xEF
                // 第二字节：0x40-0x7E 或 0x80-0xFC
                if ((b >= 0x81 && b <= 0x9F) || (b >= 0xE0 && b <= 0xEF)) &&
                   ((b2 >= 0x40 && b2 <= 0x7E) || (b2 >= 0x80 && b2 <= 0xFC)) {
                    i += 2;
                    continue;
                }
            }
            
            if self.strict {
                return false;
            }
            i += 1;
        }
        
        true
    }
    
    /// 尝试 EUC-KR 编码
    fn try_euc_kr(&self, bytes: &[u8]) -> bool {
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            
            // ASCII 范围
            if b < 0x80 {
                i += 1;
                continue;
            }
            
            // EUC-KR 双字节范围
            if i + 1 < bytes.len() {
                let b2 = bytes[i + 1];
                
                // 第一字节：0xA1-0xFE
                // 第二字节：0xA1-0xFE
                if (b >= 0xA1 && b <= 0xFE) && (b2 >= 0xA1 && b2 <= 0xFE) {
                    i += 2;
                    continue;
                }
            }
            
            if self.strict {
                return false;
            }
            i += 1;
        }
        
        true
    }
}

impl Default for MbDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// 检测字符串编码
///
/// 对应 PHP 的 mb_detect_encoding() 函数
pub fn mb_detect_encoding(input: &str, encoding_list: Option<&[&str]>, strict: bool) -> Option<String> {
    let order = match encoding_list {
        Some(list) => MbEncodingOrder::new(list.to_vec()),
        None => MbEncodingOrder::default_order(),
    };
    
    let mut detector = MbDetector::with_order(order);
    detector.set_strict(strict);
    
    detector.detect(input)
}

/// 获取/设置编码检测顺序
///
/// 对应 PHP 的 mb_detect_order() 函数
pub fn mb_detect_order(encoding_list: Option<&[&str]>) -> Vec<String> {
    match encoding_list {
        Some(list) => {
            let order = MbEncodingOrder::new(list.to_vec());
            order.encodings.clone()
        }
        None => MbEncodingOrder::default_order().encodings.clone(),
    }
}

/// 检测所有可能的编码
///
/// 对应 PHP 的 mb_list_encodings() 函数
pub fn mb_list_encodings() -> Vec<String> {
    MbEncodingList::names()
}

/// 检查字符串是否为指定编码
///
/// 对应 PHP 的 mb_check_encoding() 函数
pub fn mb_check_encoding(input: &str, encoding: Option<&str>) -> bool {
    let detector = MbDetector::new();
    
    match encoding {
        Some(enc) => {
            if let Some(detected) = detector.detect(input) {
                detected.eq_ignore_ascii_case(enc)
            } else {
                false
            }
        }
        None => {
            // 检查是否为有效的 UTF-8
            std::str::from_utf8(input.as_bytes()).is_ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_utf8() {
        let detector = MbDetector::new();
        
        let result = detector.detect("Hello, 世界!");
        assert_eq!(result, Some("UTF-8".to_string()));
    }
    
    #[test]
    fn test_detect_ascii() {
        let detector = MbDetector::new();
        
        let result = detector.detect("Hello World");
        assert_eq!(result, Some("UTF-8".to_string())); // ASCII 也是有效的 UTF-8
    }
    
    #[test]
    fn test_mb_detect_encoding() {
        let result = mb_detect_encoding("Hello", None, false);
        assert!(result.is_some());
        
        let result = mb_detect_encoding("你好", Some(&["UTF-8", "GBK"]), false);
        assert_eq!(result, Some("UTF-8".to_string()));
    }
    
    #[test]
    fn test_mb_check_encoding() {
        assert!(mb_check_encoding("Hello", Some("UTF-8")));
        assert!(mb_check_encoding("你好", Some("UTF-8")));
    }
    
    #[test]
    fn test_detect_order() {
        let order = mb_detect_order(Some(&["UTF-8", "GBK", "BIG5"]));
        assert_eq!(order, vec!["UTF-8", "GBK", "BIG5"]);
        
        let default_order = mb_detect_order(None);
        assert!(!default_order.is_empty());
    }
}
