//! 错误和异常处理器模块
//!
//! 提供完整的 PHP 错误和异常处理功能
//! 包括：set_error_handler, set_exception_handler 等

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use super::level::ErrorLevel;
use super::types::{ErrorInfo, ErrorHandlingResult, ExceptionInfo, ExceptionHandlingResult, ValueSnapshot};

/// 可调用值类型
///
/// 表示一个可以被调用的函数或方法
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CallableValue {
    /// 函数名
    Function(String),
    /// 类方法
    Method {
        /// 类名
        class: String,
        /// 方法名
        method: String,
    },
    /// 闭包 ID
    Closure(String),
}

impl CallableValue {
    /// 从函数名创建
    pub fn function(name: &str) -> Self {
        CallableValue::Function(name.to_string())
    }
    
    /// 从类和方法创建
    pub fn method(class: &str, method: &str) -> Self {
        CallableValue::Method {
            class: class.to_string(),
            method: method.to_string(),
        }
    }
    
    /// 从闭包 ID 创建
    pub fn closure(id: &str) -> Self {
        CallableValue::Closure(id.to_string())
    }
}

impl fmt::Display for CallableValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallableValue::Function(name) => write!(f, "{}", name),
            CallableValue::Method { class, method } => write!(f, "{}::{}", class, method),
            CallableValue::Closure(id) => write!(f, "Closure#{}", id),
        }
    }
}

/// 错误处理器
///
/// 管理错误和异常处理器的注册、调用和恢复
#[derive(Debug)]
pub struct ErrorHandler {
    /// 当前错误处理器
    error_handler: Option<CallableValue>,
    /// 错误处理器栈（用于 restore_error_handler）
    error_handler_stack: Vec<Option<CallableValue>>,
    /// 当前异常处理器
    exception_handler: Option<CallableValue>,
    /// 异常处理器栈（用于 restore_exception_handler）
    exception_handler_stack: Vec<Option<CallableValue>>,
    /// 当前错误报告级别
    error_reporting: ErrorLevel,
    /// 是否在错误处理器中（防止递归）
    in_error_handler: bool,
    /// 是否在异常处理器中（防止递归）
    in_exception_handler: bool,
    /// 错误日志
    error_log: Vec<ErrorInfo>,
    /// 最大日志条目数
    max_log_entries: usize,
}

impl ErrorHandler {
    /// 创建新的错误处理器
    pub fn new() -> Self {
        Self {
            error_handler: None,
            error_handler_stack: Vec::new(),
            exception_handler: None,
            exception_handler_stack: Vec::new(),
            error_reporting: ErrorLevel::E_ALL_EXCEPT_NOTICE,
            in_error_handler: false,
            in_exception_handler: false,
            error_log: Vec::new(),
            max_log_entries: 1000,
        }
    }
    
    /// 设置错误处理器
    ///
    /// 对应 PHP 的 set_error_handler() 函数
    ///
    /// # 参数
    /// - `handler`: 错误处理器回调
    /// - `error_types`: 要处理的错误类型（可选）
    ///
    /// # 返回
    /// 之前设置的错误处理器（如果有）
    pub fn set_error_handler(
        &mut self,
        handler: CallableValue,
        error_types: Option<ErrorLevel>,
    ) -> Option<CallableValue> {
        // 保存当前处理器到栈
        self.error_handler_stack.push(self.error_handler.clone());
        
        // 设置新处理器
        let previous = self.error_handler.replace(handler);
        
        // 如果指定了错误类型，更新错误报告级别
        if let Some(types) = error_types {
            self.error_reporting = types;
        }
        
        previous
    }
    
    /// 恢复之前的错误处理器
    ///
    /// 对应 PHP 的 restore_error_handler() 函数
    ///
    /// # 返回
    /// 是否成功恢复
    pub fn restore_error_handler(&mut self) -> bool {
        if let Some(previous) = self.error_handler_stack.pop() {
            self.error_handler = previous;
            true
        } else {
            false
        }
    }
    
    /// 设置异常处理器
    ///
    /// 对应 PHP 的 set_exception_handler() 函数
    ///
    /// # 参数
    /// - `handler`: 异常处理器回调
    ///
    /// # 返回
    /// 之前设置的异常处理器（如果有）
    pub fn set_exception_handler(&mut self, handler: CallableValue) -> Option<CallableValue> {
        // 保存当前处理器到栈
        self.exception_handler_stack.push(self.exception_handler.clone());
        
        // 设置新处理器
        self.exception_handler.replace(handler)
    }
    
    /// 恢复之前的异常处理器
    ///
    /// 对应 PHP 的 restore_exception_handler() 函数
    ///
    /// # 返回
    /// 是否成功恢复
    pub fn restore_exception_handler(&mut self) -> bool {
        if let Some(previous) = self.exception_handler_stack.pop() {
            self.exception_handler = previous;
            true
        } else {
            false
        }
    }
    
    /// 获取或设置错误报告级别
    ///
    /// 对应 PHP 的 error_reporting() 函数
    ///
    /// # 参数
    /// - `level`: 新的错误报告级别（可选）
    ///
    /// # 返回
    /// 旧的错误报告级别
    pub fn error_reporting(&mut self, level: Option<ErrorLevel>) -> ErrorLevel {
        let old_level = self.error_reporting;
        if let Some(new_level) = level {
            self.error_reporting = new_level;
        }
        old_level
    }
    
    /// 触发错误
    ///
    /// 对应 PHP 的 trigger_error() 函数
    ///
    /// # 参数
    /// - `message`: 错误消息
    /// - `level`: 错误级别
    /// - `file`: 文件路径
    /// - `line`: 行号
    ///
    /// # 返回
    /// 错误处理结果
    pub fn trigger_error(
        &mut self,
        message: &str,
        level: ErrorLevel,
        file: &str,
        line: usize,
    ) -> ErrorHandlingResult {
        // 检查是否应该报告此错误
        if !self.should_report(level) {
            return ErrorHandlingResult::Continue;
        }
        
        // 创建错误信息
        let error_info = ErrorInfo::new(
            level,
            message.to_string(),
            file.to_string(),
            line,
        );
        
        // 记录到日志
        self.log_error(error_info.clone());
        
        // 如果在错误处理器中，避免递归
        if self.in_error_handler {
            return ErrorHandlingResult::Continue;
        }
        
        // 如果有自定义错误处理器，调用它
        let handler_opt = self.error_handler.clone();
        if let Some(handler) = handler_opt {
            // 检查处理器是否处理此错误级别
            if self.handler_handles_level(&handler, level) {
                self.in_error_handler = true;
                
                // 调用错误处理器
                let result = self.call_error_handler(&handler, &error_info);
                
                self.in_error_handler = false;
                
                return result;
            }
        }
        
        // 使用默认处理
        self.default_error_handler(&error_info)
    }
    
    /// 处理未捕获的异常
    ///
    /// # 参数
    /// - `exception`: 异常信息
    ///
    /// # 返回
    /// 异常处理结果
    pub fn handle_exception(&mut self, exception: ExceptionInfo) -> ExceptionHandlingResult {
        // 如果在异常处理器中，避免递归
        if self.in_exception_handler {
            return ExceptionHandlingResult::Unhandled;
        }
        
        // 如果有自定义异常处理器，调用它
        let handler_opt = self.exception_handler.clone();
        if let Some(handler) = handler_opt {
            self.in_exception_handler = true;
            
            // 调用异常处理器
            let result = self.call_exception_handler(&handler, &exception);
            
            self.in_exception_handler = false;
            
            return result;
        }
        
        // 使用默认处理
        self.default_exception_handler(&exception)
    }
    
    /// 检查是否应该报告此错误
    fn should_report(&self, level: ErrorLevel) -> bool {
        // 使用 @ 运算符时，error_reporting 返回 0
        if self.error_reporting.value() == 0 {
            return false;
        }
        
        self.error_reporting.includes(level)
    }
    
    /// 检查处理器是否处理指定级别的错误
    fn handler_handles_level(&self, _handler: &CallableValue, _level: ErrorLevel) -> bool {
        // 简化实现：假设处理器处理所有级别
        // 实际实现需要检查 set_error_handler 的第二个参数
        true
    }
    
    /// 调用错误处理器
    fn call_error_handler(
        &mut self,
        _handler: &CallableValue,
        error: &ErrorInfo,
    ) -> ErrorHandlingResult {
        // 在实际实现中，这里会调用 PHP 函数
        // 目前返回默认结果
        tracing::warn!(
            "Error handler called: {} in {}:{}",
            error.message,
            error.file,
            error.line
        );
        
        // 如果是致命错误，终止执行
        if error.level.is_fatal() {
            ErrorHandlingResult::Halt
        } else {
            ErrorHandlingResult::Continue
        }
    }
    
    /// 调用异常处理器
    fn call_exception_handler(
        &mut self,
        _handler: &CallableValue,
        exception: &ExceptionInfo,
    ) -> ExceptionHandlingResult {
        // 在实际实现中，这里会调用 PHP 函数
        tracing::error!(
            "Exception handler called: {}",
            exception.to_string_formatted()
        );
        
        ExceptionHandlingResult::Handled
    }
    
    /// 默认错误处理器
    fn default_error_handler(&mut self, error: &ErrorInfo) -> ErrorHandlingResult {
        // 根据错误级别输出错误信息
        let output = error.to_string_formatted();
        
        if error.level.is_fatal() {
            tracing::error!("{}", output);
            ErrorHandlingResult::Halt
        } else if error.level.is_warning() {
            tracing::warn!("{}", output);
            ErrorHandlingResult::Continue
        } else {
            tracing::info!("{}", output);
            ErrorHandlingResult::Continue
        }
    }
    
    /// 默认异常处理器
    fn default_exception_handler(&mut self, exception: &ExceptionInfo) -> ExceptionHandlingResult {
        tracing::error!("{}", exception.to_string_formatted());
        ExceptionHandlingResult::Unhandled
    }
    
    /// 记录错误到日志
    fn log_error(&mut self, error: ErrorInfo) {
        // 如果日志已满，移除最旧的条目
        if self.error_log.len() >= self.max_log_entries {
            self.error_log.remove(0);
        }
        
        self.error_log.push(error);
    }
    
    /// 获取错误日志
    pub fn get_error_log(&self) -> &[ErrorInfo] {
        &self.error_log
    }
    
    /// 清空错误日志
    pub fn clear_error_log(&mut self) {
        self.error_log.clear();
    }
    
    /// 获取最后一个错误
    pub fn get_last_error(&self) -> Option<&ErrorInfo> {
        self.error_log.last()
    }
    
    /// 检查是否有错误处理器
    pub fn has_error_handler(&self) -> bool {
        self.error_handler.is_some()
    }
    
    /// 检查是否有异常处理器
    pub fn has_exception_handler(&self) -> bool {
        self.exception_handler.is_some()
    }
    
    /// 获取当前错误报告级别
    pub fn get_error_reporting(&self) -> ErrorLevel {
        self.error_reporting
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局错误处理器
///
/// 使用 Arc<RwLock> 实现线程安全的全局访问
pub struct GlobalErrorHandler {
    /// 内部错误处理器
    inner: Arc<RwLock<ErrorHandler>>,
}

impl GlobalErrorHandler {
    /// 创建新的全局错误处理器
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ErrorHandler::new())),
        }
    }
    
    /// 获取内部处理器的引用
    pub fn inner(&self) -> Arc<RwLock<ErrorHandler>> {
        self.inner.clone()
    }
    
    /// 设置错误处理器
    pub fn set_error_handler(&self, handler: CallableValue, error_types: Option<ErrorLevel>) -> Option<CallableValue> {
        self.inner.write().set_error_handler(handler, error_types)
    }
    
    /// 恢复错误处理器
    pub fn restore_error_handler(&self) -> bool {
        self.inner.write().restore_error_handler()
    }
    
    /// 设置异常处理器
    pub fn set_exception_handler(&self, handler: CallableValue) -> Option<CallableValue> {
        self.inner.write().set_exception_handler(handler)
    }
    
    /// 恢复异常处理器
    pub fn restore_exception_handler(&self) -> bool {
        self.inner.write().restore_exception_handler()
    }
    
    /// 触发错误
    pub fn trigger_error(
        &self,
        message: &str,
        level: ErrorLevel,
        file: &str,
        line: usize,
    ) -> ErrorHandlingResult {
        self.inner.write().trigger_error(message, level, file, line)
    }
    
    /// 处理异常
    pub fn handle_exception(&self, exception: ExceptionInfo) -> ExceptionHandlingResult {
        self.inner.write().handle_exception(exception)
    }
    
    /// 获取/设置错误报告级别
    pub fn error_reporting(&self, level: Option<ErrorLevel>) -> ErrorLevel {
        self.inner.write().error_reporting(level)
    }
}

impl Default for GlobalErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for GlobalErrorHandler {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// 全局错误处理器实例
///
/// 使用 lazy_static 或 once_cell 初始化
static GLOBAL_ERROR_HANDLER: once_cell::sync::Lazy<GlobalErrorHandler> = 
    once_cell::sync::Lazy::new(GlobalErrorHandler::new);

/// 获取全局错误处理器
pub fn global_error_handler() -> &'static GlobalErrorHandler {
    &GLOBAL_ERROR_HANDLER
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_set_error_handler() {
        let mut handler = ErrorHandler::new();
        
        // 设置错误处理器
        let previous = handler.set_error_handler(
            CallableValue::function("my_error_handler"),
            None,
        );
        assert!(previous.is_none());
        assert!(handler.has_error_handler());
        
        // 恢复错误处理器
        let restored = handler.restore_error_handler();
        assert!(restored);
        assert!(!handler.has_error_handler());
    }
    
    #[test]
    fn test_set_exception_handler() {
        let mut handler = ErrorHandler::new();
        
        // 设置异常处理器
        let previous = handler.set_exception_handler(
            CallableValue::function("my_exception_handler"),
        );
        assert!(previous.is_none());
        assert!(handler.has_exception_handler());
        
        // 恢复异常处理器
        let restored = handler.restore_exception_handler();
        assert!(restored);
        assert!(!handler.has_exception_handler());
    }
    
    #[test]
    fn test_error_reporting() {
        let mut handler = ErrorHandler::new();
        
        // 获取当前级别
        let old_level = handler.error_reporting(None);
        assert_eq!(old_level, ErrorLevel::E_ALL_EXCEPT_NOTICE);
        
        // 设置新级别
        let old_level = handler.error_reporting(Some(ErrorLevel::E_ALL));
        assert_eq!(old_level, ErrorLevel::E_ALL_EXCEPT_NOTICE);
        
        // 验证新级别
        assert_eq!(handler.get_error_reporting(), ErrorLevel::E_ALL);
    }
    
    #[test]
    fn test_trigger_error() {
        let mut handler = ErrorHandler::new();
        
        // 触发警告
        let result = handler.trigger_error(
            "Test warning",
            ErrorLevel::E_WARNING,
            "test.php",
            10,
        );
        
        assert_eq!(result, ErrorHandlingResult::Continue);
        
        // 验证日志
        assert!(!handler.get_error_log().is_empty());
    }
    
    #[test]
    fn test_should_report() {
        let handler = ErrorHandler::new();
        
        // 默认配置应该报告警告
        assert!(handler.should_report(ErrorLevel::E_WARNING));
        
        // 默认配置不应该报告通知
        assert!(!handler.should_report(ErrorLevel::E_NOTICE));
    }
    
    #[test]
    fn test_callable_value() {
        let func = CallableValue::function("my_func");
        assert_eq!(format!("{}", func), "my_func");
        
        let method = CallableValue::method("MyClass", "myMethod");
        assert_eq!(format!("{}", method), "MyClass::myMethod");
        
        let closure = CallableValue::closure("abc123");
        assert_eq!(format!("{}", closure), "Closure#abc123");
    }
}
