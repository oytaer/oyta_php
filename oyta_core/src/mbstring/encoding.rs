//! 字符编码定义和检测模块
//!
//! 定义所有支持的字符编码，提供编码检测功能

use serde::{Deserialize, Serialize};
use std::fmt;

/// mbstring 支持的字符编码
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MbEncoding {
    /// 编码名称
    pub name: String,
    /// 编码别名列表
    pub aliases: Vec<String>,
    /// 编码类型
    pub encoding_type: EncodingType,
    /// 是否支持多字节
    pub is_multibyte: bool,
}

impl MbEncoding {
    /// 创建新的编码
    pub fn new(name: &str) -> Self {
        let name_upper = name.to_uppercase();
        let is_multibyte = Self::is_multibyte_encoding(&name_upper);
        let encoding_type = Self::get_encoding_type(&name_upper);
        
        Self {
            name: name_upper,
            aliases: Vec::new(),
            encoding_type,
            is_multibyte,
        }
    }
    
    /// 创建带别名的编码
    pub fn with_aliases(name: &str, aliases: &[&str]) -> Self {
        let mut encoding = Self::new(name);
        encoding.aliases = aliases.iter().map(|s| s.to_string()).collect();
        encoding
    }
    
    /// 检查是否为多字节编码
    fn is_multibyte_encoding(name: &str) -> bool {
        matches!(
            name,
            "UTF-8" | "UTF-16" | "UTF-16LE" | "UTF-16BE" |
            "UTF-32" | "UTF-32LE" | "UTF-32BE" |
            "EUC-JP" | "EUC-CN" | "EUC-KR" | "EUC-TW" |
            "SHIFT_JIS" | "SJIS" | "SJIS-WIN" |
            "ISO-2022-JP" | "ISO-2022-KR" |
            "BIG5" | "BIG5-HKSCS" |
            "GB2312" | "GBK" | "GB18030" |
            "HZ" | "CP936" | "CP950" |
            "UHC" | "ISO-2022-CN"
        )
    }
    
    /// 获取编码类型
    fn get_encoding_type(name: &str) -> EncodingType {
        match name {
            "UTF-8" | "UTF-16" | "UTF-16LE" | "UTF-16BE" |
            "UTF-32" | "UTF-32LE" | "UTF-32BE" => EncodingType::Unicode,
            
            "EUC-JP" | "SHIFT_JIS" | "SJIS" | "SJIS-WIN" | "ISO-2022-JP" |
            "EUCJP-WIN" | "CP932" => EncodingType::Japanese,
            
            "EUC-KR" | "ISO-2022-KR" | "UHC" => EncodingType::Korean,
            
            "EUC-CN" | "GB2312" | "GBK" | "GB18030" | "HZ" |
            "CP936" | "ISO-2022-CN" => EncodingType::Chinese,
            
            "BIG5" | "BIG5-HKSCS" | "CP950" => EncodingType::ChineseTraditional,
            
            "EUC-TW" => EncodingType::Taiwanese,
            
            "ISO-8859-1" | "ISO-8859-2" | "ISO-8859-3" | "ISO-8859-4" |
            "ISO-8859-5" | "ISO-8859-6" | "ISO-8859-7" | "ISO-8859-8" |
            "ISO-8859-9" | "ISO-8859-10" | "ISO-8859-13" | "ISO-8859-14" |
            "ISO-8859-15" | "ISO-8859-16" => EncodingType::SingleByte,
            
            "WINDOWS-1250" | "WINDOWS-1251" | "WINDOWS-1252" |
            "WINDOWS-1253" | "WINDOWS-1254" | "WINDOWS-1255" |
            "WINDOWS-1256" | "WINDOWS-1257" | "WINDOWS-1258" |
            "CP1250" | "CP1251" | "CP1252" | "CP1253" | "CP1254" |
            "CP1255" | "CP1256" | "CP1257" | "CP1258" => EncodingType::SingleByte,
            
            "ASCII" | "US-ASCII" => EncodingType::Ascii,
            
            "BASE64" | "QUOTED-PRINTABLE" | "UUENCODE" => EncodingType::Transfer,
            
            _ => EncodingType::Other,
        }
    }
    
    /// 检查编码名称是否匹配（包括别名）
    pub fn matches(&self, name: &str) -> bool {
        let name_upper = name.to_uppercase();
        
        // 检查主名称
        if self.name == name_upper {
            return true;
        }
        
        // 检查别名
        self.aliases.iter().any(|a| a.to_uppercase() == name_upper)
    }
    
    /// 获取编码的最大字节数
    pub fn max_bytes_per_char(&self) -> usize {
        match self.name.as_str() {
            "UTF-8" => 4,
            "UTF-16" | "UTF-16LE" | "UTF-16BE" => 4, // 代理对
            "UTF-32" | "UTF-32LE" | "UTF-32BE" => 4,
            "GBK" | "GB18030" => 4,
            "BIG5" | "BIG5-HKSCS" => 2,
            "SHIFT_JIS" | "SJIS" | "SJIS-WIN" => 2,
            "EUC-JP" | "EUC-CN" | "EUC-KR" | "EUC-TW" => 3,
            "ISO-2022-JP" | "ISO-2022-KR" | "ISO-2022-CN" => 8, // 转义序列
            _ => 1,
        }
    }
    
    /// 检查是否为 Unicode 编码
    pub fn is_unicode(&self) -> bool {
        self.encoding_type == EncodingType::Unicode
    }
    
    /// 检查是否为 ASCII 兼容编码
    pub fn is_ascii_compatible(&self) -> bool {
        matches!(
            self.name.as_str(),
            "UTF-8" | "ASCII" | "ISO-8859-1" | "WINDOWS-1252" |
            "EUC-JP" | "EUC-CN" | "EUC-KR" | "GBK" | "GB2312"
        )
    }
}

impl fmt::Display for MbEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// 编码类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EncodingType {
    /// Unicode 编码
    Unicode,
    /// 日语编码
    Japanese,
    /// 韩语编码
    Korean,
    /// 简体中文编码
    Chinese,
    /// 繁体中文编码
    ChineseTraditional,
    /// 台湾编码
    Taiwanese,
    /// 单字节编码
    SingleByte,
    /// ASCII 编码
    Ascii,
    /// 传输编码
    Transfer,
    /// 其他编码
    Other,
}

/// 编码列表
///
/// 提供所有支持的编码列表
pub struct MbEncodingList;

impl MbEncodingList {
    /// 获取所有支持的编码列表
    pub fn all() -> Vec<MbEncoding> {
        vec![
            // Unicode 编码
            MbEncoding::with_aliases("UTF-8", &["UTF8"]),
            MbEncoding::with_aliases("UTF-16", &["UTF16"]),
            MbEncoding::with_aliases("UTF-16LE", &["UTF16LE"]),
            MbEncoding::with_aliases("UTF-16BE", &["UTF16BE"]),
            MbEncoding::with_aliases("UTF-32", &["UTF32"]),
            MbEncoding::with_aliases("UTF-32LE", &["UTF32LE"]),
            MbEncoding::with_aliases("UTF-32BE", &["UTF32BE"]),
            
            // 日语编码
            MbEncoding::with_aliases("EUC-JP", &["EUCJP"]),
            MbEncoding::with_aliases("SHIFT_JIS", &["SJIS", "SHIFT-JIS"]),
            MbEncoding::with_aliases("SJIS-WIN", &["SJISWIN", "CP932"]),
            MbEncoding::with_aliases("ISO-2022-JP", &["ISO2022JP"]),
            MbEncoding::with_aliases("EUCJP-WIN", &["EUCJPWIN"]),
            
            // 中文编码
            MbEncoding::with_aliases("EUC-CN", &["EUCCN"]),
            MbEncoding::with_aliases("GB2312", &["GB-2312"]),
            MbEncoding::with_aliases("GBK", &["CP936"]),
            MbEncoding::with_aliases("GB18030", &["GB-18030"]),
            MbEncoding::with_aliases("BIG5", &["BIG-5", "BIG-FIVE"]),
            MbEncoding::with_aliases("BIG5-HKSCS", &["BIG5HKSCS"]),
            MbEncoding::with_aliases("HZ", &["HZ-GB-2312"]),
            MbEncoding::with_aliases("ISO-2022-CN", &["ISO2022CN"]),
            
            // 韩语编码
            MbEncoding::with_aliases("EUC-KR", &["EUCKR"]),
            MbEncoding::with_aliases("ISO-2022-KR", &["ISO2022KR"]),
            MbEncoding::with_aliases("UHC", &["CP949"]),
            
            // 台湾编码
            MbEncoding::with_aliases("EUC-TW", &["EUCTW"]),
            
            // ISO-8859 系列
            MbEncoding::new("ISO-8859-1"),
            MbEncoding::new("ISO-8859-2"),
            MbEncoding::new("ISO-8859-3"),
            MbEncoding::new("ISO-8859-4"),
            MbEncoding::new("ISO-8859-5"),
            MbEncoding::new("ISO-8859-6"),
            MbEncoding::new("ISO-8859-7"),
            MbEncoding::new("ISO-8859-8"),
            MbEncoding::new("ISO-8859-9"),
            MbEncoding::new("ISO-8859-10"),
            MbEncoding::new("ISO-8859-13"),
            MbEncoding::new("ISO-8859-14"),
            MbEncoding::new("ISO-8859-15"),
            MbEncoding::new("ISO-8859-16"),
            
            // Windows 编码
            MbEncoding::with_aliases("WINDOWS-1250", &["CP1250"]),
            MbEncoding::with_aliases("WINDOWS-1251", &["CP1251"]),
            MbEncoding::with_aliases("WINDOWS-1252", &["CP1252"]),
            MbEncoding::with_aliases("WINDOWS-1253", &["CP1253"]),
            MbEncoding::with_aliases("WINDOWS-1254", &["CP1254"]),
            MbEncoding::with_aliases("WINDOWS-1255", &["CP1255"]),
            MbEncoding::with_aliases("WINDOWS-1256", &["CP1256"]),
            MbEncoding::with_aliases("WINDOWS-1257", &["CP1257"]),
            MbEncoding::with_aliases("WINDOWS-1258", &["CP1258"]),
            
            // ASCII
            MbEncoding::with_aliases("ASCII", &["US-ASCII"]),
            
            // 传输编码
            MbEncoding::new("BASE64"),
            MbEncoding::with_aliases("QUOTED-PRINTABLE", &["QP"]),
            MbEncoding::new("UUENCODE"),
        ]
    }
    
    /// 获取编码名称列表
    pub fn names() -> Vec<String> {
        Self::all().iter().map(|e| e.name.clone()).collect()
    }
    
    /// 根据名称查找编码
    pub fn find(name: &str) -> Option<MbEncoding> {
        let encodings = Self::all();
        encodings.into_iter().find(|e| e.matches(name))
    }
    
    /// 检查编码是否支持
    pub fn is_supported(name: &str) -> bool {
        Self::find(name).is_some()
    }
    
    /// 获取内部编码（默认 UTF-8）
    pub fn internal_encoding() -> MbEncoding {
        MbEncoding::new("UTF-8")
    }
    
    /// 获取 HTTP 输入编码
    pub fn http_input_encoding() -> MbEncoding {
        MbEncoding::new("UTF-8")
    }
    
    /// 获取 HTTP 输出编码
    pub fn http_output_encoding() -> MbEncoding {
        MbEncoding::new("UTF-8")
    }
}

/// 编码顺序
///
/// 用于编码检测时的优先级顺序
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MbEncodingOrder {
    /// 编码列表
    pub encodings: Vec<String>,
}

impl MbEncodingOrder {
    /// 创建新的编码顺序
    pub fn new(encodings: Vec<&str>) -> Self {
        Self {
            encodings: encodings.iter().map(|s| s.to_string()).collect(),
        }
    }
    
    /// 获取默认编码顺序
    pub fn default_order() -> Self {
        Self::new(vec![
            "UTF-8",
            "EUC-JP",
            "SJIS",
            "EUC-CN",
            "GBK",
            "BIG5",
            "EUC-KR",
            "ISO-8859-1",
            "ASCII",
        ])
    }
    
    /// 获取日语编码顺序
    pub fn japanese() -> Self {
        Self::new(vec!["UTF-8", "EUC-JP", "SJIS", "ISO-2022-JP", "ASCII"])
    }
    
    /// 获取中文编码顺序
    pub fn chinese() -> Self {
        Self::new(vec!["UTF-8", "GBK", "GB2312", "BIG5", "ASCII"])
    }
    
    /// 获取韩语编码顺序
    pub fn korean() -> Self {
        Self::new(vec!["UTF-8", "EUC-KR", "UHC", "ASCII"])
    }
    
    /// 添加编码到顺序
    pub fn add(&mut self, encoding: &str) {
        self.encodings.push(encoding.to_string());
    }
    
    /// 移除编码
    pub fn remove(&mut self, encoding: &str) {
        self.encodings.retain(|e| e != encoding);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encoding_creation() {
        let utf8 = MbEncoding::new("utf-8");
        
        assert_eq!(utf8.name, "UTF-8");
        assert!(utf8.is_multibyte);
        assert!(utf8.is_unicode());
    }
    
    #[test]
    fn test_encoding_aliases() {
        let utf8 = MbEncoding::with_aliases("UTF-8", &["UTF8"]);
        
        assert!(utf8.matches("UTF-8"));
        assert!(utf8.matches("utf8"));
        assert!(utf8.matches("UTF8"));
    }
    
    #[test]
    fn test_encoding_max_bytes() {
        let utf8 = MbEncoding::new("UTF-8");
        assert_eq!(utf8.max_bytes_per_char(), 4);
        
        let gbk = MbEncoding::new("GBK");
        assert_eq!(gbk.max_bytes_per_char(), 4);
        
        let ascii = MbEncoding::new("ASCII");
        assert_eq!(ascii.max_bytes_per_char(), 1);
    }
    
    #[test]
    fn test_encoding_list() {
        let encodings = MbEncodingList::all();
        
        assert!(!encodings.is_empty());
        assert!(MbEncodingList::is_supported("UTF-8"));
        assert!(MbEncodingList::is_supported("GBK"));
    }
    
    #[test]
    fn test_encoding_find() {
        let encoding = MbEncodingList::find("utf-8");
        assert!(encoding.is_some());
        assert_eq!(encoding.unwrap().name, "UTF-8");
    }
    
    #[test]
    fn test_encoding_order() {
        let order = MbEncodingOrder::default_order();
        
        assert!(!order.encodings.is_empty());
        assert!(order.encodings.contains(&"UTF-8".to_string()));
    }
}
