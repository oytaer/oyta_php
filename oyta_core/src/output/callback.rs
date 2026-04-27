//! 缓冲回调处理模块
//!
//! 定义输出缓冲回调函数的类型和处理逻辑

use serde::{Deserialize, Serialize};
use std::fmt;

use super::handler::OutputHandlerStatus;

/// 输出回调函数
///
/// 表示一个输出缓冲回调函数
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputCallback {
    /// 回调类型
    pub callback_type: CallbackType,
    /// 回调名称
    pub name: String,
}

impl OutputCallback {
    /// 从函数名创建回调
    pub fn function(name: &str) -> Self {
        Self {
            callback_type: CallbackType::Function,
            name: name.to_string(),
        }
    }
    
    /// 从类方法创建回调
    pub fn method(class: &str, method: &str) -> Self {
        Self {
            callback_type: CallbackType::Method {
                class: class.to_string(),
                method: method.to_string(),
            },
            name: format!("{}::{}", class, method),
        }
    }
    
    /// 从闭包 ID 创建回调
    pub fn closure(id: &str) -> Self {
        Self {
            callback_type: CallbackType::Closure(id.to_string()),
            name: format!("Closure#{}", id),
        }
    }
    
    /// 创建内置回调
    pub fn builtin(name: &str) -> Self {
        Self {
            callback_type: CallbackType::Builtin,
            name: name.to_string(),
        }
    }
    
    /// 获取回调名称
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// 检查是否为函数
    pub fn is_function(&self) -> bool {
        matches!(self.callback_type, CallbackType::Function)
    }
    
    /// 检查是否为方法
    pub fn is_method(&self) -> bool {
        matches!(self.callback_type, CallbackType::Method { .. })
    }
    
    /// 检查是否为闭包
    pub fn is_closure(&self) -> bool {
        matches!(self.callback_type, CallbackType::Closure(_))
    }
    
    /// 检查是否为内置回调
    pub fn is_builtin(&self) -> bool {
        matches!(self.callback_type, CallbackType::Builtin)
    }
    
    /// 执行回调
    ///
    /// # 参数
    /// - `buffer`: 缓冲区内容
    /// - `status`: 处理器状态
    ///
    /// # 返回
    /// 处理后的内容
    pub fn execute(&self, buffer: &str, status: OutputHandlerStatus) -> CallbackResult {
        // 在实际实现中，这里会调用 PHP 函数
        // 目前返回默认结果
        match self.callback_type {
            CallbackType::Builtin => {
                // 内置回调处理
                self.execute_builtin(buffer, status)
            }
            _ => {
                // 用户自定义回调
                // 在实际实现中，这里会调用解释器执行回调
                CallbackResult::Processed(buffer.to_string())
            }
        }
    }
    
    /// 执行内置回调
    fn execute_builtin(&self, buffer: &str, _status: OutputHandlerStatus) -> CallbackResult {
        match self.name.as_str() {
            // mb_output_handler - 多字节编码输出处理
            "mb_output_handler" => {
                // 在实际实现中，这里会进行编码转换
                CallbackResult::Processed(buffer.to_string())
            }
            // ob_gzhandler - GZIP 压缩输出处理
            "ob_gzhandler" => {
                // 在实际实现中，这里会进行 GZIP 压缩
                CallbackResult::Processed(buffer.to_string())
            }
            // ob_iconv_handler - 字符编码转换
            "ob_iconv_handler" => {
                CallbackResult::Processed(buffer.to_string())
            }
            // 其他内置处理器
            _ => CallbackResult::Processed(buffer.to_string())
        }
    }
}

impl fmt::Display for OutputCallback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// 回调类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CallbackType {
    /// 函数名
    Function,
    /// 类方法
    Method {
        /// 类名
        class: String,
        /// 方法名
        method: String,
    },
    /// 闭包
    Closure(String),
    /// 内置处理器
    Builtin,
}

/// 回调执行结果
#[derive(Debug, Clone, PartialEq)]
pub enum CallbackResult {
    /// 处理成功，返回处理后的内容
    Processed(String),
    /// 不做处理，返回原始内容
    Unchanged,
    /// 处理失败，返回错误信息
    Failed(String),
    /// 禁用输出
    Suppress,
}

impl CallbackResult {
    /// 检查是否处理成功
    pub fn is_processed(&self) -> bool {
        matches!(self, CallbackResult::Processed(_))
    }
    
    /// 检查是否失败
    pub fn is_failed(&self) -> bool {
        matches!(self, CallbackResult::Failed(_))
    }
    
    /// 获取处理后的内容
    pub fn content(&self) -> Option<&str> {
        match self {
            CallbackResult::Processed(content) => Some(content),
            CallbackResult::Unchanged => None,
            CallbackResult::Failed(_) => None,
            CallbackResult::Suppress => Some(""),
        }
    }
}

/// 回调上下文
///
/// 传递给回调函数的上下文信息
#[derive(Debug, Clone, PartialEq)]
pub struct CallbackContext {
    /// 缓冲区级别
    pub buffer_level: usize,
    /// 处理器状态
    pub status: OutputHandlerStatus,
    /// 原始内容长度
    pub content_length: usize,
    /// 是否为最终调用
    pub is_final: bool,
}

impl CallbackContext {
    /// 创建新的回调上下文
    pub fn new(buffer_level: usize, status: OutputHandlerStatus, content_length: usize) -> Self {
        Self {
            buffer_level,
            status,
            content_length,
            is_final: false,
        }
    }
    
    /// 设置为最终调用
    pub fn set_final(&mut self) {
        self.is_final = true;
    }
}

/// 预定义回调工厂
///
/// 提供创建常用回调的便捷方法
pub struct CallbackFactory;

impl CallbackFactory {
    /// 创建 mb_output_handler 回调
    pub fn mb_output_handler() -> OutputCallback {
        OutputCallback::builtin("mb_output_handler")
    }
    
    /// 创建 ob_gzhandler 回调
    pub fn ob_gzhandler() -> OutputCallback {
        OutputCallback::builtin("ob_gzhandler")
    }
    
    /// 创建 ob_iconv_handler 回调
    pub fn ob_iconv_handler() -> OutputCallback {
        OutputCallback::builtin("ob_iconv_handler")
    }
    
    /// 创建 URL 重写回调
    pub fn url_rewriter() -> OutputCallback {
        OutputCallback::builtin("URL-Rewriter")
    }
    
    /// 创建用户函数回调
    pub fn user_function(name: &str) -> OutputCallback {
        OutputCallback::function(name)
    }
    
    /// 创建类方法回调
    pub fn class_method(class: &str, method: &str) -> OutputCallback {
        OutputCallback::method(class, method)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_callback_function() {
        let callback = OutputCallback::function("my_callback");
        
        assert!(callback.is_function());
        assert!(!callback.is_method());
        assert_eq!(callback.name(), "my_callback");
    }
    
    #[test]
    fn test_callback_method() {
        let callback = OutputCallback::method("MyClass", "myMethod");
        
        assert!(callback.is_method());
        assert!(!callback.is_function());
        assert_eq!(callback.name(), "MyClass::myMethod");
    }
    
    #[test]
    fn test_callback_closure() {
        let callback = OutputCallback::closure("abc123");
        
        assert!(callback.is_closure());
        assert_eq!(callback.name(), "Closure#abc123");
    }
    
    #[test]
    fn test_callback_builtin() {
        let callback = OutputCallback::builtin("ob_gzhandler");
        
        assert!(callback.is_builtin());
        assert_eq!(callback.name(), "ob_gzhandler");
    }
    
    #[test]
    fn test_callback_execute() {
        let callback = OutputCallback::builtin("ob_gzhandler");
        let result = callback.execute("test content", OutputHandlerStatus::FLUSH);
        
        assert!(result.is_processed());
    }
    
    #[test]
    fn test_callback_result() {
        let result = CallbackResult::Processed("processed content".to_string());
        assert!(result.is_processed());
        assert_eq!(result.content(), Some("processed content"));
        
        let result = CallbackResult::Unchanged;
        assert!(!result.is_processed());
        assert_eq!(result.content(), None);
        
        let result = CallbackResult::Failed("error".to_string());
        assert!(result.is_failed());
    }
    
    #[test]
    fn test_callback_context() {
        let mut ctx = CallbackContext::new(1, OutputHandlerStatus::FLUSH, 100);
        
        assert_eq!(ctx.buffer_level, 1);
        assert!(!ctx.is_final);
        
        ctx.set_final();
        assert!(ctx.is_final);
    }
    
    #[test]
    fn test_callback_factory() {
        let callback = CallbackFactory::mb_output_handler();
        assert!(callback.is_builtin());
        
        let callback = CallbackFactory::ob_gzhandler();
        assert!(callback.is_builtin());
        
        let callback = CallbackFactory::user_function("my_func");
        assert!(callback.is_function());
    }
}
