//! 错误级别定义模块
//!
//! 定义 PHP 所有的错误级别常量
//! 对应 PHP 的 E_* 常量

use serde::{Deserialize, Serialize};
use std::fmt;

/// 错误级别枚举
///
/// 对应 PHP 的错误级别常量
/// 使用位掩码设计，支持组合多个级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ErrorLevel(pub i32);

impl ErrorLevel {
    // ============ 致命错误 ============
    
    /// E_ERROR - 致命的运行时错误
    /// 这类错误通常是不可恢复的，脚本将终止执行
    pub const E_ERROR: ErrorLevel = ErrorLevel(1);
    
    /// E_CORE_ERROR - PHP 初始化启动时发生的致命错误
    /// 类似 E_ERROR，但由 PHP 核心引擎产生
    pub const E_CORE_ERROR: ErrorLevel = ErrorLevel(16);
    
    /// E_COMPILE_ERROR - 致命的编译时错误
    /// 类似 E_ERROR，由 Zend 脚本引擎产生
    pub const E_COMPILE_ERROR: ErrorLevel = ErrorLevel(64);
    
    /// E_RECOVERABLE_ERROR - 可捕获的致命错误
    /// 表示发生了一个可能很危险的错误，但还没有导致引擎处于不稳定状态
    /// 如果错误未被用户自定义的处理器捕获，则成为 E_ERROR
    pub const E_RECOVERABLE_ERROR: ErrorLevel = ErrorLevel(4096);
    
    /// E_PARSE - 编译时解析错误
    /// 解析错误只由解析器产生
    pub const E_PARSE: ErrorLevel = ErrorLevel(4);
    
    // ============ 警告 ============
    
    /// E_WARNING - 运行时警告（非致命错误）
    /// 脚本会继续执行，但建议修复
    pub const E_WARNING: ErrorLevel = ErrorLevel(2);
    
    /// E_CORE_WARNING - PHP 初始化启动时发生的警告（非致命错误）
    /// 类似 E_WARNING，但由 PHP 核心引擎产生
    pub const E_CORE_WARNING: ErrorLevel = ErrorLevel(32);
    
    /// E_COMPILE_WARNING - 编译时警告（非致命错误）
    /// 类似 E_WARNING，由 Zend 脚本引擎产生
    pub const E_COMPILE_WARNING: ErrorLevel = ErrorLevel(128);
    
    /// E_USER_WARNING - 用户产生的警告信息
    /// 类似 E_WARNING，由用户在代码中使用 trigger_error() 产生
    pub const E_USER_WARNING: ErrorLevel = ErrorLevel(512);
    
    // ============ 通知 ============
    
    /// E_NOTICE - 运行时通知
    /// 表示脚本遇到了可能表现为错误的情况，但在正常运行时也可能发生
    pub const E_NOTICE: ErrorLevel = ErrorLevel(8);
    
    /// E_USER_NOTICE - 用户产生的通知信息
    /// 类似 E_NOTICE，由用户在代码中使用 trigger_error() 产生
    pub const E_USER_NOTICE: ErrorLevel = ErrorLevel(1024);
    
    /// E_STRICT - 运行时通知
    /// 建议修改代码以确保代码具有最佳的互操作性和向前兼容性
    pub const E_STRICT: ErrorLevel = ErrorLevel(2048);
    
    /// E_DEPRECATED - 运行时通知
    /// 表示代码使用了已弃用的特性，未来版本可能移除
    pub const E_DEPRECATED: ErrorLevel = ErrorLevel(8192);
    
    /// E_USER_DEPRECATED - 用户产生的弃用警告
    /// 类似 E_DEPRECATED，由用户在代码中使用 trigger_error() 产生
    pub const E_USER_DEPRECATED: ErrorLevel = ErrorLevel(16384);
    
    // ============ 用户错误 ============
    
    /// E_USER_ERROR - 用户产生的错误信息
    /// 类似 E_ERROR，由用户在代码中使用 trigger_error() 产生
    pub const E_USER_ERROR: ErrorLevel = ErrorLevel(256);
    
    // ============ 组合级别 ============
    
    /// E_ALL - 所有错误和警告
    pub const E_ALL: ErrorLevel = ErrorLevel(32767);
    
    /// E_ERROR | E_WARNING | E_PARSE - 常用的错误级别组合
    pub const E_ERROR_WARNING_PARSE: ErrorLevel = ErrorLevel(1 | 2 | 4);
    
    /// E_ALL & ~E_NOTICE - 除通知外的所有错误
    pub const E_ALL_EXCEPT_NOTICE: ErrorLevel = ErrorLevel(32759);
    
    /// E_ALL & ~E_NOTICE & ~E_STRICT - 除通知和严格模式外的所有错误
    pub const E_ALL_EXCEPT_NOTICE_STRICT: ErrorLevel = ErrorLevel(30711);
    
    /// E_ALL & ~E_DEPRECATED - 除弃用警告外的所有错误
    pub const E_ALL_EXCEPT_DEPRECATED: ErrorLevel = ErrorLevel(24575);
    
    // ============ 方法实现 ============
    
    /// 创建新的错误级别
    pub fn new(level: i32) -> Self {
        ErrorLevel(level)
    }
    
    /// 检查是否包含指定的错误级别
    ///
    /// # 参数
    /// - `other`: 要检查的错误级别
    ///
    /// # 返回
    /// 如果当前级别包含指定级别，返回 true
    pub fn includes(&self, other: ErrorLevel) -> bool {
        (self.0 & other.0) != 0
    }
    
    /// 添加错误级别
    ///
    /// # 参数
    /// - `other`: 要添加的错误级别
    ///
    /// # 返回
    /// 新的错误级别组合
    pub fn add(&self, other: ErrorLevel) -> Self {
        ErrorLevel(self.0 | other.0)
    }
    
    /// 移除错误级别
    ///
    /// # 参数
    /// - `other`: 要移除的错误级别
    ///
    /// # 返回
    /// 新的错误级别组合
    pub fn remove(&self, other: ErrorLevel) -> Self {
        ErrorLevel(self.0 & !other.0)
    }
    
    /// 获取错误级别的名称
    ///
    /// # 返回
    /// 错误级别的字符串名称
    pub fn name(&self) -> &'static str {
        match *self {
            Self::E_ERROR => "E_ERROR",
            Self::E_WARNING => "E_WARNING",
            Self::E_PARSE => "E_PARSE",
            Self::E_NOTICE => "E_NOTICE",
            Self::E_CORE_ERROR => "E_CORE_ERROR",
            Self::E_CORE_WARNING => "E_CORE_WARNING",
            Self::E_COMPILE_ERROR => "E_COMPILE_ERROR",
            Self::E_COMPILE_WARNING => "E_COMPILE_WARNING",
            Self::E_USER_ERROR => "E_USER_ERROR",
            Self::E_USER_WARNING => "E_USER_WARNING",
            Self::E_USER_NOTICE => "E_USER_NOTICE",
            Self::E_STRICT => "E_STRICT",
            Self::E_RECOVERABLE_ERROR => "E_RECOVERABLE_ERROR",
            Self::E_DEPRECATED => "E_DEPRECATED",
            Self::E_USER_DEPRECATED => "E_USER_DEPRECATED",
            Self::E_ALL => "E_ALL",
            _ => "E_UNKNOWN",
        }
    }
    
    /// 获取错误级别的描述
    ///
    /// # 返回
    /// 错误级别的详细描述
    pub fn description(&self) -> &'static str {
        match *self {
            Self::E_ERROR => "Fatal run-time error",
            Self::E_WARNING => "Run-time warning (non-fatal error)",
            Self::E_PARSE => "Compile-time parse error",
            Self::E_NOTICE => "Run-time notice",
            Self::E_CORE_ERROR => "Fatal error that occurred during PHP's initial startup",
            Self::E_CORE_WARNING => "Warning that occurred during PHP's initial startup",
            Self::E_COMPILE_ERROR => "Fatal compile-time error",
            Self::E_COMPILE_WARNING => "Compile-time warning",
            Self::E_USER_ERROR => "User-generated error message",
            Self::E_USER_WARNING => "User-generated warning message",
            Self::E_USER_NOTICE => "User-generated notice message",
            Self::E_STRICT => "Suggestion to change code for interoperability",
            Self::E_RECOVERABLE_ERROR => "Catchable fatal error",
            Self::E_DEPRECATED => "Warning about code that will not work in future versions",
            Self::E_USER_DEPRECATED => "User-generated deprecated warning",
            Self::E_ALL => "All errors and warnings",
            _ => "Unknown error level",
        }
    }
    
    /// 判断是否为致命错误
    ///
    /// # 返回
    /// 如果是致命错误，返回 true
    pub fn is_fatal(&self) -> bool {
        matches!(
            *self,
            Self::E_ERROR 
            | Self::E_CORE_ERROR 
            | Self::E_COMPILE_ERROR 
            | Self::E_RECOVERABLE_ERROR
            | Self::E_PARSE
        )
    }
    
    /// 判断是否为用户产生的错误
    ///
    /// # 返回
    /// 如果是用户产生的错误，返回 true
    pub fn is_user_error(&self) -> bool {
        matches!(
            *self,
            Self::E_USER_ERROR 
            | Self::E_USER_WARNING 
            | Self::E_USER_NOTICE 
            | Self::E_USER_DEPRECATED
        )
    }
    
    /// 判断是否为警告
    ///
    /// # 返回
    /// 如果是警告级别，返回 true
    pub fn is_warning(&self) -> bool {
        matches!(
            *self,
            Self::E_WARNING 
            | Self::E_CORE_WARNING 
            | Self::E_COMPILE_WARNING 
            | Self::E_USER_WARNING
        )
    }
    
    /// 判断是否为通知
    ///
    /// # 返回
    /// 如果是通知级别，返回 true
    pub fn is_notice(&self) -> bool {
        matches!(
            *self,
            Self::E_NOTICE 
            | Self::E_USER_NOTICE 
            | Self::E_STRICT 
            | Self::E_DEPRECATED 
            | Self::E_USER_DEPRECATED
        )
    }
    
    /// 获取原始值
    pub fn value(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for ErrorLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Default for ErrorLevel {
    fn default() -> Self {
        // 默认报告除通知外的所有错误
        Self::E_ALL_EXCEPT_NOTICE
    }
}

impl From<i32> for ErrorLevel {
    fn from(value: i32) -> Self {
        ErrorLevel(value)
    }
}

impl From<ErrorLevel> for i32 {
    fn from(level: ErrorLevel) -> Self {
        level.0
    }
}

/// 错误级别工具函数
pub struct ErrorLevelUtils;

impl ErrorLevelUtils {
    /// 从字符串解析错误级别
    ///
    /// # 参数
    /// - `name`: 错误级别名称字符串
    ///
    /// # 返回
    /// 成功返回对应的错误级别，失败返回 None
    pub fn from_name(name: &str) -> Option<ErrorLevel> {
        match name.to_uppercase().as_str() {
            "E_ERROR" => Some(ErrorLevel::E_ERROR),
            "E_WARNING" => Some(ErrorLevel::E_WARNING),
            "E_PARSE" => Some(ErrorLevel::E_PARSE),
            "E_NOTICE" => Some(ErrorLevel::E_NOTICE),
            "E_CORE_ERROR" => Some(ErrorLevel::E_CORE_ERROR),
            "E_CORE_WARNING" => Some(ErrorLevel::E_CORE_WARNING),
            "E_COMPILE_ERROR" => Some(ErrorLevel::E_COMPILE_ERROR),
            "E_COMPILE_WARNING" => Some(ErrorLevel::E_COMPILE_WARNING),
            "E_USER_ERROR" => Some(ErrorLevel::E_USER_ERROR),
            "E_USER_WARNING" => Some(ErrorLevel::E_USER_WARNING),
            "E_USER_NOTICE" => Some(ErrorLevel::E_USER_NOTICE),
            "E_STRICT" => Some(ErrorLevel::E_STRICT),
            "E_RECOVERABLE_ERROR" => Some(ErrorLevel::E_RECOVERABLE_ERROR),
            "E_DEPRECATED" => Some(ErrorLevel::E_DEPRECATED),
            "E_USER_DEPRECATED" => Some(ErrorLevel::E_USER_DEPRECATED),
            "E_ALL" => Some(ErrorLevel::E_ALL),
            _ => None,
        }
    }
    
    /// 获取所有错误级别的列表
    ///
    /// # 返回
    /// 所有错误级别的向量
    pub fn all_levels() -> Vec<ErrorLevel> {
        vec![
            ErrorLevel::E_ERROR,
            ErrorLevel::E_WARNING,
            ErrorLevel::E_PARSE,
            ErrorLevel::E_NOTICE,
            ErrorLevel::E_CORE_ERROR,
            ErrorLevel::E_CORE_WARNING,
            ErrorLevel::E_COMPILE_ERROR,
            ErrorLevel::E_COMPILE_WARNING,
            ErrorLevel::E_USER_ERROR,
            ErrorLevel::E_USER_WARNING,
            ErrorLevel::E_USER_NOTICE,
            ErrorLevel::E_STRICT,
            ErrorLevel::E_RECOVERABLE_ERROR,
            ErrorLevel::E_DEPRECATED,
            ErrorLevel::E_USER_DEPRECATED,
        ]
    }
    
    /// 解析错误级别字符串
    ///
    /// 支持格式：
    /// - "E_ALL"
    /// - "E_ALL & ~E_NOTICE"
    /// - "E_ERROR | E_WARNING"
    /// - 数字值
    ///
    /// # 参数
    /// - `s`: 错误级别字符串
    ///
    /// # 返回
    /// 成功返回错误级别，失败返回 None
    pub fn parse(s: &str) -> Option<ErrorLevel> {
        let s = s.trim();
        
        // 尝试解析为数字
        if let Ok(num) = s.parse::<i32>() {
            return Some(ErrorLevel(num));
        }
        
        // 处理组合表达式
        let mut result = ErrorLevel(0);
        let parts: Vec<&str> = s.split('|').map(|p| p.trim()).collect();
        
        for part in parts {
            // 处理 & ~ 操作
            if part.contains('&') && part.contains('~') {
                let sub_parts: Vec<&str> = part.split('&').map(|p| p.trim()).collect();
                if sub_parts.len() == 2 {
                    let base = Self::from_name(sub_parts[0])?;
                    let exclude = sub_parts[1].trim_start_matches('~').trim();
                    let exclude_level = Self::from_name(exclude)?;
                    result = result.add(base).remove(exclude_level);
                    continue;
                }
            }
            
            // 处理单个级别
            if let Some(level) = Self::from_name(part) {
                result = result.add(level);
            }
        }
        
        if result.0 != 0 {
            Some(result)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_level_includes() {
        // 测试 E_ALL 包含 E_ERROR
        assert!(ErrorLevel::E_ALL.includes(ErrorLevel::E_ERROR));
        
        // 测试 E_ALL 不包含 0
        assert!(!ErrorLevel::E_ERROR.includes(ErrorLevel::E_WARNING));
    }
    
    #[test]
    fn test_error_level_add_remove() {
        // 测试添加级别
        let level = ErrorLevel::E_ERROR.add(ErrorLevel::E_WARNING);
        assert!(level.includes(ErrorLevel::E_ERROR));
        assert!(level.includes(ErrorLevel::E_WARNING));
        
        // 测试移除级别
        let level = level.remove(ErrorLevel::E_WARNING);
        assert!(level.includes(ErrorLevel::E_ERROR));
        assert!(!level.includes(ErrorLevel::E_WARNING));
    }
    
    #[test]
    fn test_error_level_is_fatal() {
        assert!(ErrorLevel::E_ERROR.is_fatal());
        assert!(ErrorLevel::E_PARSE.is_fatal());
        assert!(!ErrorLevel::E_WARNING.is_fatal());
        assert!(!ErrorLevel::E_NOTICE.is_fatal());
    }
    
    #[test]
    fn test_error_level_is_user_error() {
        assert!(ErrorLevel::E_USER_ERROR.is_user_error());
        assert!(ErrorLevel::E_USER_WARNING.is_user_error());
        assert!(!ErrorLevel::E_ERROR.is_user_error());
    }
    
    #[test]
    fn test_error_level_parse() {
        // 测试解析单个级别
        let level = ErrorLevelUtils::parse("E_ERROR").unwrap();
        assert_eq!(level, ErrorLevel::E_ERROR);
        
        // 测试解析组合级别
        let level = ErrorLevelUtils::parse("E_ERROR | E_WARNING").unwrap();
        assert!(level.includes(ErrorLevel::E_ERROR));
        assert!(level.includes(ErrorLevel::E_WARNING));
        
        // 测试解析带排除的组合
        let level = ErrorLevelUtils::parse("E_ALL & ~E_NOTICE").unwrap();
        assert!(level.includes(ErrorLevel::E_ERROR));
        assert!(!level.includes(ErrorLevel::E_NOTICE));
    }
    
    #[test]
    fn test_error_level_from_name() {
        assert_eq!(ErrorLevelUtils::from_name("E_ERROR"), Some(ErrorLevel::E_ERROR));
        assert_eq!(ErrorLevelUtils::from_name("e_error"), Some(ErrorLevel::E_ERROR));
        assert_eq!(ErrorLevelUtils::from_name("UNKNOWN"), None);
    }
}
