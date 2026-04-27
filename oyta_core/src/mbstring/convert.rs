//! 编码转换功能模块
//!
//! 提供完整的字符编码转换功能

use encoding_rs::{UTF_8, GBK, BIG5, EUC_JP, SHIFT_JIS, EUC_KR, WINDOWS_1252};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use super::encoding::{MbEncoding, MbEncodingList};

/// 编码转换器
///
/// 提供字符编码转换功能
#[derive(Debug, Clone)]
pub struct MbConverter {
    /// 源编码
    pub from_encoding: MbEncoding,
    /// 目标编码
    pub to_encoding: MbEncoding,
}

impl MbConverter {
    /// 创建新的转换器
    pub fn new(from_encoding: &str, to_encoding: &str) -> Result<Self, String> {
        let from = MbEncodingList::find(from_encoding)
            .ok_or_else(|| format!("不支持的源编码: {}", from_encoding))?;
        
        let to = MbEncodingList::find(to_encoding)
            .ok_or_else(|| format!("不支持的目标编码: {}", to_encoding))?;
        
        Ok(Self {
            from_encoding: from,
            to_encoding: to,
        })
    }
    
    /// 转换字符串编码
    ///
    /// # 参数
    /// - `input`: 输入字符串
    ///
    /// # 返回
    /// 转换后的字符串
    pub fn convert(&self, input: &str) -> Result<String, String> {
        // 如果源和目标编码相同，直接返回
        if self.from_encoding.name == self.to_encoding.name {
            return Ok(input.to_string());
        }
        
        // 使用 encoding_rs 进行转换
        self.convert_with_encoding_rs(input)
    }
    
    /// 使用 encoding_rs 进行转换
    fn convert_with_encoding_rs(&self, input: &str) -> Result<String, String> {
        // 首先将输入转换为 UTF-8
        let utf8_string = if self.from_encoding.name == "UTF-8" {
            input.to_string()
        } else {
            // 从源编码解码到 UTF-8
            let (decoded, _had_errors) = self.decode_to_utf8(input.as_bytes());
            decoded.to_string()
        };
        
        // 然后从 UTF-8 编码到目标编码
        if self.to_encoding.name == "UTF-8" {
            Ok(utf8_string)
        } else {
            let (encoded, _had_errors) = self.encode_from_utf8(utf8_string.as_bytes());
            String::from_utf8(encoded.to_vec())
                .map_err(|e| format!("目标编码错误: {}", e))
        }
    }
    
    /// 从源编码解码到 UTF-8
    fn decode_to_utf8<'a>(&self, input: &'a [u8]) -> (Cow<'a, str>, bool) {
        match self.from_encoding.name.as_str() {
            "UTF-8" => {
                let (decoded, _, had_errors) = UTF_8.decode(input);
                (decoded, had_errors)
            }
            "GBK" | "GB2312" | "CP936" => {
                let (decoded, _, had_errors) = GBK.decode(input);
                (decoded, had_errors)
            }
            "BIG5" | "BIG5-HKSCS" => {
                let (decoded, _, had_errors) = BIG5.decode(input);
                (decoded, had_errors)
            }
            "EUC-JP" => {
                let (decoded, _, had_errors) = EUC_JP.decode(input);
                (decoded, had_errors)
            }
            "SHIFT_JIS" | "SJIS" | "SJIS-WIN" => {
                let (decoded, _, had_errors) = SHIFT_JIS.decode(input);
                (decoded, had_errors)
            }
            "EUC-KR" => {
                let (decoded, _, had_errors) = EUC_KR.decode(input);
                (decoded, had_errors)
            }
            "ISO-8859-1" | "LATIN-1" | "LATIN1" => {
                let (decoded, _, had_errors) = WINDOWS_1252.decode(input);
                (decoded, had_errors)
            }
            "WINDOWS-1252" | "CP1252" => {
                let (decoded, _, had_errors) = WINDOWS_1252.decode(input);
                (decoded, had_errors)
            }
            _ => {
                let (decoded, _, had_errors) = UTF_8.decode(input);
                (decoded, had_errors)
            }
        }
    }
    
    /// 从 UTF-8 编码到目标编码
    fn encode_from_utf8<'a>(&self, input: &'a [u8]) -> (Cow<'a, [u8]>, bool) {
        // 先转换为字符串
        let input_str = match std::str::from_utf8(input) {
            Ok(s) => s,
            Err(_) => return (Cow::Borrowed(input), true),
        };
        
        match self.to_encoding.name.as_str() {
            "UTF-8" => (Cow::Borrowed(input), false),
            "GBK" | "GB2312" | "CP936" => {
                let (encoded, _, had_errors) = GBK.encode(input_str);
                (encoded, had_errors)
            }
            "BIG5" | "BIG5-HKSCS" => {
                let (encoded, _, had_errors) = BIG5.encode(input_str);
                (encoded, had_errors)
            }
            "EUC-JP" => {
                let (encoded, _, had_errors) = EUC_JP.encode(input_str);
                (encoded, had_errors)
            }
            "SHIFT_JIS" | "SJIS" | "SJIS-WIN" => {
                let (encoded, _, had_errors) = SHIFT_JIS.encode(input_str);
                (encoded, had_errors)
            }
            "EUC-KR" => {
                let (encoded, _, had_errors) = EUC_KR.encode(input_str);
                (encoded, had_errors)
            }
            "ISO-8859-1" | "LATIN-1" | "LATIN1" => {
                let (encoded, _, had_errors) = WINDOWS_1252.encode(input_str);
                (encoded, had_errors)
            }
            "WINDOWS-1252" | "CP1252" => {
                let (encoded, _, had_errors) = WINDOWS_1252.encode(input_str);
                (encoded, had_errors)
            }
            _ => (Cow::Borrowed(input), false), // 默认不转换
        }
    }
    
    /// 转换字节数组
    pub fn convert_bytes(&self, input: &[u8]) -> Result<Vec<u8>, String> {
        // 解码到 UTF-8
        let (utf8_bytes, _had_errors) = self.decode_to_utf8(input);
        
        // 如果目标是 UTF-8，直接返回
        if self.to_encoding.name == "UTF-8" {
            return Ok(utf8_bytes.as_bytes().to_vec());
        }
        
        // 编码到目标编码
        let (encoded, _had_errors) = self.encode_from_utf8(utf8_bytes.as_bytes());
        Ok(encoded.to_vec())
    }
}

/// 编码转换函数
///
/// 对应 PHP 的 mb_convert_encoding() 函数
pub fn mb_convert_encoding(input: &str, to_encoding: &str, from_encoding: Option<&str>) -> Result<String, String> {
    let from = from_encoding.unwrap_or("UTF-8");
    let converter = MbConverter::new(from, to_encoding)?;
    converter.convert(input)
}

/// 编码转换函数（支持多种源编码）
///
/// 对应 PHP 的 mb_convert_encoding() 函数（多编码版本）
pub fn mb_convert_encoding_multi(
    input: &str,
    to_encoding: &str,
    from_encodings: &[&str],
) -> Result<String, String> {
    // 尝试每个源编码
    for from_encoding in from_encodings {
        if let Ok(converter) = MbConverter::new(from_encoding, to_encoding) {
            if let Ok(result) = converter.convert(input) {
                // 检查转换是否成功（没有乱码）
                if is_valid_string(&result) {
                    return Ok(result);
                }
            }
        }
    }
    
    // 所有编码都失败，使用 UTF-8 作为默认
    mb_convert_encoding(input, to_encoding, Some("UTF-8"))
}

/// 检查字符串是否有效（没有乱码）
fn is_valid_string(s: &str) -> bool {
    // 检查是否包含过多的替换字符
    let replacement_count = s.chars().filter(|&c| c == '\u{FFFD}').count();
    let total_chars = s.chars().count();
    
    // 如果替换字符超过 10%，认为转换失败
    if total_chars > 0 && replacement_count as f32 / total_chars as f32 > 0.1 {
        return false;
    }
    
    true
}

/// 检测并转换编码
///
/// 对应 PHP 的 mb_convert_variables() 函数
pub fn mb_convert_variables(
    input: &str,
    to_encoding: &str,
    from_encodings: &[&str],
) -> Result<(String, String), String> {
    // 尝试检测编码
    for from_encoding in from_encodings {
        if let Ok(converter) = MbConverter::new(from_encoding, to_encoding) {
            if let Ok(result) = converter.convert(input) {
                if is_valid_string(&result) {
                    return Ok((result, from_encoding.to_string()));
                }
            }
        }
    }
    
    Err("无法确定输入编码".to_string())
}

/// 获取编码别名
///
/// 对应 PHP 的 mb_encoding_aliases() 函数
pub fn mb_encoding_aliases(encoding: &str) -> Vec<String> {
    if let Some(enc) = MbEncodingList::find(encoding) {
        enc.aliases
    } else {
        Vec::new()
    }
}

/// 检查编码是否支持
pub fn mb_check_encoding(input: &str, encoding: &str) -> bool {
    // 尝试将输入从指定编码解码
    if let Ok(converter) = MbConverter::new(encoding, "UTF-8") {
        if let Ok(result) = converter.convert(input) {
            return is_valid_string(&result);
        }
    }
    false
}

/// 获取支持的编码列表
pub fn mb_list_encodings() -> Vec<String> {
    MbEncodingList::names()
}

/// 获取内部编码
pub fn mb_internal_encoding() -> String {
    "UTF-8".to_string()
}

/// 设置内部编码
pub fn mb_internal_encoding_set(_encoding: &str) -> bool {
    // 在 Rust 实现中，内部编码始终为 UTF-8
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_converter_creation() {
        let converter = MbConverter::new("UTF-8", "GBK");
        assert!(converter.is_ok());
        
        let converter = MbConverter::new("INVALID", "UTF-8");
        assert!(converter.is_err());
    }
    
    #[test]
    fn test_convert_utf8_to_utf8() {
        let converter = MbConverter::new("UTF-8", "UTF-8").unwrap();
        let result = converter.convert("Hello, 世界!").unwrap();
        assert_eq!(result, "Hello, 世界!");
    }
    
    #[test]
    fn test_mb_convert_encoding() {
        let result = mb_convert_encoding("Hello", "UTF-8", Some("UTF-8"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello");
    }
    
    #[test]
    fn test_mb_list_encodings() {
        let encodings = mb_list_encodings();
        assert!(!encodings.is_empty());
        assert!(encodings.contains(&"UTF-8".to_string()));
    }
    
    #[test]
    fn test_mb_check_encoding() {
        assert!(mb_check_encoding("Hello", "UTF-8"));
        assert!(mb_check_encoding("你好", "UTF-8"));
    }
}
