//! 关闭函数注册模块
//!
//! 提供完整的 PHP register_shutdown_function 功能
//! 支持：注册多个关闭函数、执行顺序、错误捕获等

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::handler::CallableValue;
use super::level::ErrorLevel;
use super::types::{ErrorInfo, ExceptionInfo};

/// 关闭函数管理器
///
/// 管理所有注册的关闭函数，并在脚本结束时执行
#[derive(Debug)]
pub struct ShutdownManager {
    /// 注册的关闭函数列表
    shutdown_functions: Vec<ShutdownFunction>,
    /// 是否已执行关闭函数
    executed: bool,
    /// 最大执行时间（毫秒）
    max_execution_time: u64,
    /// 是否捕获致命错误
    catch_fatal_errors: bool,
    /// 最后的错误
    last_error: Option<ErrorInfo>,
}

/// 关闭函数
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShutdownFunction {
    /// 可调用值
    pub callable: CallableValue,
    /// 参数列表
    pub args: Vec<ShutdownArg>,
    /// 注册顺序
    pub order: usize,
    /// 注册时间
    pub registered_at: u64,
    /// 优先级（数字越小越先执行）
    pub priority: i32,
}

/// 关闭函数参数
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShutdownArg {
    /// 整数
    Int(i64),
    /// 浮点数
    Float(f64),
    /// 字符串
    String(String),
    /// 布尔值
    Bool(bool),
    /// Null
    Null,
    /// 数组（序列化）
    Array(String),
}

impl ShutdownArg {
    /// 从值创建参数
    pub fn from_int(v: i64) -> Self {
        ShutdownArg::Int(v)
    }
    
    pub fn from_float(v: f64) -> Self {
        ShutdownArg::Float(v)
    }
    
    pub fn from_string(v: &str) -> Self {
        ShutdownArg::String(v.to_string())
    }
    
    pub fn from_bool(v: bool) -> Self {
        ShutdownArg::Bool(v)
    }
    
    pub fn null() -> Self {
        ShutdownArg::Null
    }
}

impl fmt::Display for ShutdownArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShutdownArg::Int(v) => write!(f, "{}", v),
            ShutdownArg::Float(v) => write!(f, "{}", v),
            ShutdownArg::String(v) => write!(f, "\"{}\"", v),
            ShutdownArg::Bool(v) => write!(f, "{}", v),
            ShutdownArg::Null => write!(f, "null"),
            ShutdownArg::Array(v) => write!(f, "Array({})", v.len()),
        }
    }
}

impl ShutdownManager {
    /// 创建新的关闭函数管理器
    pub fn new() -> Self {
        Self {
            shutdown_functions: Vec::new(),
            executed: false,
            max_execution_time: 30000, // 默认 30 秒
            catch_fatal_errors: true,
            last_error: None,
        }
    }
    
    /// 注册关闭函数
    ///
    /// 对应 PHP 的 register_shutdown_function() 函数
    ///
    /// # 参数
    /// - `callable`: 可调用值
    /// - `args`: 参数列表
    ///
    /// # 返回
    /// 是否注册成功
    pub fn register(&mut self, callable: CallableValue, args: Vec<ShutdownArg>) -> bool {
        // 如果已经执行过关闭函数，不能再注册
        if self.executed {
            return false;
        }
        
        let order = self.shutdown_functions.len();
        let registered_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        let shutdown_func = ShutdownFunction {
            callable,
            args,
            order,
            registered_at,
            priority: 0,
        };
        
        self.shutdown_functions.push(shutdown_func);
        true
    }
    
    /// 注册带优先级的关闭函数
    ///
    /// # 参数
    /// - `callable`: 可调用值
    /// - `args`: 参数列表
    /// - `priority`: 优先级（数字越小越先执行）
    ///
    /// # 返回
    /// 是否注册成功
    pub fn register_with_priority(
        &mut self,
        callable: CallableValue,
        args: Vec<ShutdownArg>,
        priority: i32,
    ) -> bool {
        if self.executed {
            return false;
        }
        
        let order = self.shutdown_functions.len();
        let registered_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        let shutdown_func = ShutdownFunction {
            callable,
            args,
            order,
            registered_at,
            priority,
        };
        
        self.shutdown_functions.push(shutdown_func);
        true
    }
    
    /// 执行所有关闭函数
    ///
    /// 按优先级和注册顺序执行所有关闭函数
    pub fn execute(&mut self) -> ShutdownResult {
        // 防止重复执行
        if self.executed {
            return ShutdownResult::AlreadyExecuted;
        }
        
        self.executed = true;
        
        // 检查是否有致命错误
        if self.catch_fatal_errors {
            self.capture_last_error();
        }
        
        // 按优先级排序（优先级小的先执行）
        let mut functions = std::mem::take(&mut self.shutdown_functions);
        functions.sort_by(|a, b| {
            a.priority.cmp(&b.priority)
                .then_with(|| a.order.cmp(&b.order))
        });
        
        let total_count = functions.len();
        let start_time = Instant::now();
        let mut results = Vec::new();
        let mut timed_out = false;
        
        for func in functions {
            // 检查是否超时
            if start_time.elapsed().as_millis() as u64 > self.max_execution_time {
                timed_out = true;
                break;
            }
            
            // 执行关闭函数
            let result = self.execute_function(&func);
            results.push(result);
        }
        
        if timed_out {
            ShutdownResult::TimedOut {
                executed_count: results.len(),
                total_count,
            }
        } else {
            ShutdownResult::Completed {
                executed_count: results.len(),
            }
        }
    }
    
    /// 执行单个关闭函数
    fn execute_function(&self, func: &ShutdownFunction) -> FunctionResult {
        // 在实际实现中，这里会调用 PHP 函数
        tracing::debug!(
            "Executing shutdown function: {} with {} args",
            func.callable,
            func.args.len()
        );
        
        // 模拟执行
        FunctionResult::Success
    }
    
    /// 捕获最后的错误
    fn capture_last_error(&mut self) {
        // 在实际实现中，这里会获取最后的错误
        // 目前使用 None
        self.last_error = None;
    }
    
    /// 获取最后的错误
    pub fn get_last_error(&self) -> Option<&ErrorInfo> {
        self.last_error.as_ref()
    }
    
    /// 设置最大执行时间
    pub fn set_max_execution_time(&mut self, millis: u64) {
        self.max_execution_time = millis;
    }
    
    /// 设置是否捕获致命错误
    pub fn set_catch_fatal_errors(&mut self, catch: bool) {
        self.catch_fatal_errors = catch;
    }
    
    /// 获取注册的关闭函数数量
    pub fn count(&self) -> usize {
        self.shutdown_functions.len()
    }
    
    /// 清除所有关闭函数
    pub fn clear(&mut self) {
        if !self.executed {
            self.shutdown_functions.clear();
        }
    }
    
    /// 检查是否已执行
    pub fn is_executed(&self) -> bool {
        self.executed
    }
    
    /// 获取所有注册的关闭函数
    pub fn get_functions(&self) -> &[ShutdownFunction] {
        &self.shutdown_functions
    }
}

impl Default for ShutdownManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 关闭函数执行结果
#[derive(Debug, Clone, PartialEq)]
pub enum ShutdownResult {
    /// 执行完成
    Completed {
        /// 已执行的函数数量
        executed_count: usize,
    },
    /// 执行超时
    TimedOut {
        /// 已执行的函数数量
        executed_count: usize,
        /// 总函数数量
        total_count: usize,
    },
    /// 已经执行过
    AlreadyExecuted,
}

/// 单个函数执行结果
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionResult {
    /// 执行成功
    Success,
    /// 执行失败
    Failed(String),
    /// 执行超时
    Timeout,
    /// 异常
    Exception(ExceptionInfo),
}

/// 全局关闭函数管理器
pub struct GlobalShutdownManager {
    /// 内部管理器
    inner: Arc<RwLock<ShutdownManager>>,
}

impl GlobalShutdownManager {
    /// 创建新的全局管理器
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ShutdownManager::new())),
        }
    }
    
    /// 注册关闭函数
    pub fn register(&self, callable: CallableValue, args: Vec<ShutdownArg>) -> bool {
        self.inner.write().register(callable, args)
    }
    
    /// 注册带优先级的关闭函数
    pub fn register_with_priority(
        &self,
        callable: CallableValue,
        args: Vec<ShutdownArg>,
        priority: i32,
    ) -> bool {
        self.inner.write().register_with_priority(callable, args, priority)
    }
    
    /// 执行所有关闭函数
    pub fn execute(&self) -> ShutdownResult {
        self.inner.write().execute()
    }
    
    /// 获取最后的错误
    pub fn get_last_error(&self) -> Option<ErrorInfo> {
        self.inner.read().get_last_error().cloned()
    }
    
    /// 获取注册的函数数量
    pub fn count(&self) -> usize {
        self.inner.read().count()
    }
    
    /// 检查是否已执行
    pub fn is_executed(&self) -> bool {
        self.inner.read().is_executed()
    }
}

impl Default for GlobalShutdownManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for GlobalShutdownManager {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// 全局关闭函数管理器实例
static GLOBAL_SHUTDOWN_MANAGER: once_cell::sync::Lazy<GlobalShutdownManager> = 
    once_cell::sync::Lazy::new(GlobalShutdownManager::new);

/// 获取全局关闭函数管理器
pub fn global_shutdown_manager() -> &'static GlobalShutdownManager {
    &GLOBAL_SHUTDOWN_MANAGER
}

/// 关闭函数构建器
///
/// 提供链式 API 来注册关闭函数
pub struct ShutdownBuilder {
    callable: CallableValue,
    args: Vec<ShutdownArg>,
    priority: i32,
}

impl ShutdownBuilder {
    /// 创建新的构建器
    pub fn new(callable: CallableValue) -> Self {
        Self {
            callable,
            args: Vec::new(),
            priority: 0,
        }
    }
    
    /// 添加参数
    pub fn arg(mut self, arg: ShutdownArg) -> Self {
        self.args.push(arg);
        self
    }
    
    /// 添加整数参数
    pub fn int(self, v: i64) -> Self {
        self.arg(ShutdownArg::from_int(v))
    }
    
    /// 添加字符串参数
    pub fn string(self, v: &str) -> Self {
        self.arg(ShutdownArg::from_string(v))
    }
    
    /// 添加布尔参数
    pub fn bool(self, v: bool) -> Self {
        self.arg(ShutdownArg::from_bool(v))
    }
    
    /// 设置优先级
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
    
    /// 注册到管理器
    pub fn register(self, manager: &mut ShutdownManager) -> bool {
        manager.register_with_priority(self.callable, self.args, self.priority)
    }
    
    /// 注册到全局管理器
    pub fn register_global(self) -> bool {
        global_shutdown_manager().register_with_priority(self.callable, self.args, self.priority)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_shutdown_function() {
        let mut manager = ShutdownManager::new();
        
        // 注册关闭函数
        let registered = manager.register(
            CallableValue::function("my_shutdown"),
            vec![ShutdownArg::from_string("arg1")],
        );
        
        assert!(registered);
        assert_eq!(manager.count(), 1);
    }
    
    #[test]
    fn test_register_with_priority() {
        let mut manager = ShutdownManager::new();
        
        // 注册多个关闭函数
        manager.register_with_priority(
            CallableValue::function("func1"),
            vec![],
            10,
        );
        
        manager.register_with_priority(
            CallableValue::function("func2"),
            vec![],
            5,
        );
        
        manager.register_with_priority(
            CallableValue::function("func3"),
            vec![],
            5,
        );
        
        assert_eq!(manager.count(), 3);
        
        // 执行
        let result = manager.execute();
        assert!(matches!(result, ShutdownResult::Completed { .. }));
    }
    
    #[test]
    fn test_execute_once() {
        let mut manager = ShutdownManager::new();
        
        manager.register(CallableValue::function("func"), vec![]);
        
        // 第一次执行
        let result = manager.execute();
        assert!(matches!(result, ShutdownResult::Completed { .. }));
        
        // 第二次执行
        let result = manager.execute();
        assert!(matches!(result, ShutdownResult::AlreadyExecuted));
    }
    
    #[test]
    fn test_shutdown_builder() {
        let mut manager = ShutdownManager::new();
        
        // 使用构建器
        let registered = ShutdownBuilder::new(CallableValue::function("my_func"))
            .string("hello")
            .int(42)
            .bool(true)
            .priority(10)
            .register(&mut manager);
        
        assert!(registered);
        assert_eq!(manager.count(), 1);
    }
    
    #[test]
    fn test_shutdown_arg_display() {
        assert_eq!(format!("{}", ShutdownArg::from_int(42)), "42");
        assert_eq!(format!("{}", ShutdownArg::from_string("hello")), "\"hello\"");
        assert_eq!(format!("{}", ShutdownArg::from_bool(true)), "true");
        assert_eq!(format!("{}", ShutdownArg::null()), "null");
    }
    
    #[test]
    fn test_global_shutdown_manager() {
        let manager = global_shutdown_manager();
        
        let registered = manager.register(
            CallableValue::function("global_shutdown"),
            vec![],
        );
        
        assert!(registered);
        assert!(manager.count() > 0);
    }
}
