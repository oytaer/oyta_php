//! Fiber 协程模块
//!
//! 实现 PHP 8.1+ 的 Fiber 协程功能支持
//! Fiber 是 PHP 8.1 引入的特性，用于实现协程和异步编程
//!
//! # 功能特性
//! - Fiber 创建和启动
//! - Fiber 挂起和恢复
//! - Fiber 状态管理
//! - Fiber 异常处理
//!
//! # 使用示例
//! ```php
//! $fiber = new Fiber(function() {
//!     $value = Fiber::suspend('suspended');
//!     return "received: " . $value;
//! });
//! 
//! $fiber->start();  // 返回 'suspended'
//! $fiber->resume('resumed');  // 返回 'received: resumed'
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Fiber 协程状态
/// 表示 Fiber 在生命周期中的不同状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FiberState {
    /// 已创建但未启动
    /// 当 Fiber 被创建但尚未调用 start() 时处于此状态
    Created,
    
    /// 已启动
    /// 当 Fiber 正在执行时处于此状态
    Started,
    
    /// 已挂起
    /// 当 Fiber 执行 Fiber::suspend() 后处于此状态
    /// 此时可以调用 resume() 恢复执行
    Suspended,
    
    /// 已终止
    /// 当 Fiber 执行完毕（return 或异常）后处于此状态
    /// 此时可以调用 getReturn() 获取返回值
    Terminated,
}

/// Fiber 协程内部状态
/// 使用 Arc<Mutex<>> 包装以支持共享访问
#[derive(Debug, Clone)]
pub struct FiberInner {
    /// Fiber 唯一标识符
    /// 用于区分不同的 Fiber 实例
    pub id: String,
    
    /// 当前状态
    /// 表示 Fiber 当前处于哪个生命周期阶段
    pub state: FiberState,
    
    /// 挂起时传递的值（JSON 序列化存储）
    /// 当 Fiber 调用 Fiber::suspend() 时保存的值
    /// 可以通过 start() 或 resume() 的返回值获取
    pub suspended_value_json: Option<String>,
    
    /// 返回值（JSON 序列化存储）
    /// 当 Fiber 执行完毕（通过 return 语句）时保存返回值
    /// 可以通过 getReturn() 方法获取
    pub return_value_json: Option<String>,
    
    /// 异常值（JSON 序列化存储）
    /// 当 Fiber 因异常终止时保存异常对象
    /// 可以通过 throw() 方法抛出异常
    pub exception_json: Option<String>,
    
    /// Fiber 函数名
    /// 保存 Fiber 构造函数传入的可调用对象名称
    pub function_name: String,
    
    /// 当前执行位置标记
    /// 用于记录 Fiber 执行到哪个位置
    /// 以便下次 resume 时从正确位置继续执行
    pub execution_pointer: usize,
    
    /// Fiber 来源文件路径
    /// 用于调试和错误信息
    pub file_path: String,
    
    /// Fiber 起始行号
    /// 用于调试和错误信息
    pub start_line: usize,
    
    /// 是否是主 Fiber
    /// PHP 中每个线程都有一个隐式的主 Fiber
    pub is_main: bool,
}

/// Fiber 协程值类型
/// 表示一个 PHP Fiber 对象，支持协程编程
/// 使用 Arc<Mutex<>> 包装内部状态以支持共享访问
#[derive(Debug, Clone)]
pub struct FiberValue {
    /// 内部状态（共享引用）
    /// 使用 Arc<Mutex<>> 支持多线程共享和修改
    inner: Arc<Mutex<FiberInner>>,
}

/// 为 FiberValue 实现 PartialEq trait
/// 通过比较唯一标识符来判断两个 Fiber 是否相等
impl PartialEq for FiberValue {
    fn eq(&self, other: &Self) -> bool {
        // 获取两个 Fiber 的内部状态锁
        let self_inner = self.inner.lock().unwrap();
        let other_inner = other.inner.lock().unwrap();
        // 比较唯一标识符
        self_inner.id == other_inner.id
    }
}

/// Fiber 实现的方法列表
/// 这些方法可以通过 Fiber 类调用
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiberMethod {
    /// start() - 启动 Fiber
    /// 开始执行 Fiber，直到 suspend 或终止
    Start,
    
    /// resume($value) - 恢复 Fiber
    /// 恢复挂起的 Fiber，传递值作为 suspend 的返回值
    Resume,
    
    /// throw($exception) - 向 Fiber 抛出异常
    /// 在当前挂起点抛出异常
    Throw,
    
    /// getReturn() - 获取返回值
    /// 获取 Fiber 的 return 语句返回的值
    GetReturn,
    
    /// isStarted() - 检查是否已启动
    /// 如果 Fiber 已启动返回 true
    IsStarted,
    
    /// isSuspended() - 检查是否已挂起
    /// 如果 Fiber 已挂起返回 true
    IsSuspended,
    
    /// isTerminated() - 检查是否已终止
    /// 如果 Fiber 已终止返回 true
    IsTerminated,
    
    /// getCurrent() - 获取当前 Fiber
    /// 静态方法，获取当前正在执行的 Fiber
    GetCurrent,
    
    /// suspend($value) - 挂起当前 Fiber
    /// 静态方法，挂起当前 Fiber 并返回值
    Suspend,
}

impl FiberValue {
    /// 创建新的 Fiber 实例
    ///
    /// # 参数
    /// - `id`: Fiber 唯一标识符
    /// - `function_name`: Fiber 函数名
    /// - `file_path`: Fiber 所在文件路径
    /// - `start_line`: Fiber 起始行号
    ///
    /// # 返回
    /// 返回新创建的 Fiber 实例，初始状态为 Created
    pub fn new(
        id: String,
        function_name: String,
        file_path: String,
        start_line: usize,
    ) -> Self {
        // 创建内部状态
        let inner = FiberInner {
            // 设置唯一标识符
            id,
            // 初始状态为已创建
            state: FiberState::Created,
            // 挂起值初始为空
            suspended_value_json: None,
            // 返回值初始为空
            return_value_json: None,
            // 异常值初始为空
            exception_json: None,
            // 保存 Fiber 函数名
            function_name,
            // 执行指针从 0 开始
            execution_pointer: 0,
            // 保存文件路径
            file_path,
            // 保存起始行号
            start_line,
            // 默认不是主 Fiber
            is_main: false,
        };
        
        // 返回 Fiber 实例
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
    
    /// 创建主 Fiber
    /// PHP 中每个线程都有一个隐式的主 Fiber
    ///
    /// # 返回
    /// 返回主 Fiber 实例
    pub fn create_main_fiber() -> Self {
        // 创建内部状态
        let inner = FiberInner {
            // 主 Fiber 的 ID 为 "main"
            id: "main".to_string(),
            // 主 Fiber 初始状态为已启动
            state: FiberState::Started,
            // 挂起值初始为空
            suspended_value_json: None,
            // 返回值初始为空
            return_value_json: None,
            // 异常值初始为空
            exception_json: None,
            // 主 Fiber 没有函数名
            function_name: String::new(),
            // 执行指针为 0
            execution_pointer: 0,
            // 文件路径为空
            file_path: String::new(),
            // 起始行号为 0
            start_line: 0,
            // 标记为主 Fiber
            is_main: true,
        };
        
        // 返回 Fiber 实例
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
    
    /// 启动 Fiber
    /// 对应 PHP 的 Fiber::start() 方法
    ///
    /// # 返回
    /// 成功返回挂起值的 JSON 字符串，失败返回错误信息
    pub fn start(&mut self) -> Result<Option<String>, String> {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        
        // 检查 Fiber 是否已经启动
        if inner.state != FiberState::Created {
            // 已启动的 Fiber 不能再次启动
            return Err("Cannot start a fiber that has already been started".to_string());
        }
        
        // 设置状态为已启动
        inner.state = FiberState::Started;
        
        // 返回当前挂起值（如果有）
        Ok(inner.suspended_value_json.clone())
    }
    
    /// 恢复 Fiber
    /// 对应 PHP 的 Fiber::resume() 方法
    ///
    /// # 参数
    /// - `value_json`: 要传递给 Fiber 的值的 JSON 字符串
    ///
    /// # 返回
    /// 成功返回挂起值的 JSON 字符串，失败返回错误信息
    pub fn resume_json(&mut self, value_json: String) -> Result<Option<String>, String> {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        
        // 检查 Fiber 是否处于挂起状态
        if inner.state != FiberState::Suspended {
            // 未挂起的 Fiber 不能恢复
            return Err("Cannot resume a fiber that is not suspended".to_string());
        }
        
        // 设置状态为已启动（继续执行）
        inner.state = FiberState::Started;
        
        // 保存传入的值，作为 suspend 的返回值
        inner.suspended_value_json = Some(value_json);
        
        // 返回当前挂起值
        Ok(inner.suspended_value_json.clone())
    }
    
    /// 向 Fiber 抛出异常
    /// 对应 PHP 的 Fiber::throw() 方法
    ///
    /// # 参数
    /// - `exception_json`: 要抛出的异常的 JSON 字符串
    ///
    /// # 返回
    /// 成功返回挂起值的 JSON 字符串，失败返回错误信息
    pub fn throw_json(&mut self, exception_json: String) -> Result<Option<String>, String> {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        
        // 检查 Fiber 是否处于挂起状态
        if inner.state != FiberState::Suspended {
            // 未挂起的 Fiber 不能抛出异常
            return Err("Cannot throw into a fiber that is not suspended".to_string());
        }
        
        // 设置状态为已启动（继续执行）
        inner.state = FiberState::Started;
        
        // 保存异常
        inner.exception_json = Some(exception_json);
        
        // 返回当前挂起值
        Ok(inner.suspended_value_json.clone())
    }
    
    /// 获取 Fiber 的返回值（JSON 字符串）
    /// 对应 PHP 的 Fiber::getReturn() 方法
    ///
    /// # 返回
    /// 成功返回返回值的 JSON 字符串，失败返回错误信息
    pub fn get_return_json(&self) -> Result<Option<String>, String> {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        
        // 检查 Fiber 是否已终止
        if inner.state != FiberState::Terminated {
            // 未终止的 Fiber 不能获取返回值
            return Err("Cannot get return value of a fiber that has not terminated".to_string());
        }
        
        // 返回保存的返回值
        Ok(inner.return_value_json.clone())
    }
    
    /// 检查 Fiber 是否已启动
    /// 对应 PHP 的 Fiber::isStarted() 方法
    ///
    /// # 返回
    /// 如果 Fiber 已启动返回 true，否则返回 false
    pub fn is_started(&self) -> bool {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        
        // 检查状态是否为已启动
        matches!(inner.state, FiberState::Started | FiberState::Suspended | FiberState::Terminated)
    }
    
    /// 检查 Fiber 是否已挂起
    /// 对应 PHP 的 Fiber::isSuspended() 方法
    ///
    /// # 返回
    /// 如果 Fiber 已挂起返回 true，否则返回 false
    pub fn is_suspended(&self) -> bool {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        
        // 检查状态是否为已挂起
        inner.state == FiberState::Suspended
    }
    
    /// 检查 Fiber 是否已终止
    /// 对应 PHP 的 Fiber::isTerminated() 方法
    ///
    /// # 返回
    /// 如果 Fiber 已终止返回 true，否则返回 false
    pub fn is_terminated(&self) -> bool {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        
        // 检查状态是否为已终止
        inner.state == FiberState::Terminated
    }
    
    /// 挂起当前 Fiber
    /// 对应 PHP 的 Fiber::suspend() 静态方法
    ///
    /// # 参数
    /// - `value_json`: 挂起时返回给调用者的值的 JSON 字符串
    pub fn suspend_json(&mut self, value_json: String) {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        
        // 设置状态为已挂起
        inner.state = FiberState::Suspended;
        
        // 保存挂起值
        inner.suspended_value_json = Some(value_json);
    }
    
    /// 设置返回值并终止 Fiber
    /// 由解释器在 Fiber 执行 return 语句或结束时调用
    ///
    /// # 参数
    /// - `value_json`: 返回值的 JSON 字符串
    pub fn set_return_json(&mut self, value_json: String) {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        
        // 保存返回值
        inner.return_value_json = Some(value_json);
        
        // 设置状态为已终止
        inner.state = FiberState::Terminated;
    }
    
    /// 设置异常并终止 Fiber
    /// 由解释器在 Fiber 因异常终止时调用
    ///
    /// # 参数
    /// - `exception_json`: 异常值的 JSON 字符串
    pub fn set_exception_json(&mut self, exception_json: String) {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        
        // 保存异常
        inner.exception_json = Some(exception_json);
        
        // 设置状态为已终止
        inner.state = FiberState::Terminated;
    }
    
    /// 获取挂起值（JSON 字符串）
    /// 由解释器在恢复执行时调用
    pub fn get_resume_value_json(&self) -> Option<String> {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        
        // 返回挂起值
        inner.suspended_value_json.clone()
    }
    
    /// 获取要抛出的异常（JSON 字符串）
    /// 由解释器在挂起点检查时调用
    pub fn get_thrown_exception_json(&self) -> Option<String> {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        
        // 返回要抛出的异常
        inner.exception_json.clone()
    }
    
    /// 清除异常
    /// 在异常被处理后调用
    pub fn clear_exception(&mut self) {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        
        // 清空异常
        inner.exception_json = None;
    }
    
    /// 获取 Fiber 状态
    pub fn get_state(&self) -> FiberState {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        inner.state
    }
    
    /// 获取 Fiber 唯一标识符
    pub fn get_id(&self) -> String {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        inner.id.clone()
    }
    
    /// 设置执行指针
    pub fn set_execution_pointer(&mut self, pointer: usize) {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        inner.execution_pointer = pointer;
    }
    
    /// 获取执行指针
    pub fn get_execution_pointer(&self) -> usize {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        inner.execution_pointer
    }
    
    /// 获取 Fiber 函数名
    pub fn get_function_name(&self) -> String {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        inner.function_name.clone()
    }
    
    /// 获取文件路径
    pub fn get_file_path(&self) -> String {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        inner.file_path.clone()
    }
    
    /// 获取起始行号
    pub fn get_start_line(&self) -> usize {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        inner.start_line
    }
    
    /// 检查是否是主 Fiber
    pub fn is_main_fiber(&self) -> bool {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        inner.is_main
    }
}

/// Fiber 工厂
/// 用于创建和管理 Fiber 实例
pub struct FiberFactory {
    /// 计数器，用于生成唯一 ID
    counter: u64,
    
    /// 当前正在执行的 Fiber ID（用于 getCurrent）
    current_fiber_id: Option<String>,
}

impl FiberFactory {
    /// 创建新的 Fiber 工厂
    pub fn new() -> Self {
        Self {
            counter: 0,
            current_fiber_id: None,
        }
    }
    
    /// 创建新的 Fiber 实例
    ///
    /// # 参数
    /// - `function_name`: Fiber 函数名
    /// - `file_path`: 文件路径
    /// - `start_line`: 起始行号
    ///
    /// # 返回
    /// 返回新创建的 Fiber 实例
    pub fn create(
        &mut self,
        function_name: String,
        file_path: String,
        start_line: usize,
    ) -> FiberValue {
        // 递增计数器
        self.counter += 1;
        
        // 生成唯一 ID
        let id = format!("fiber_{}", self.counter);
        
        // 创建并返回 Fiber 实例
        FiberValue::new(id, function_name, file_path, start_line)
    }
    
    /// 获取已创建的 Fiber 数量
    pub fn count(&self) -> u64 {
        self.counter
    }
    
    /// 设置当前正在执行的 Fiber
    pub fn set_current_fiber(&mut self, fiber_id: Option<String>) {
        self.current_fiber_id = fiber_id;
    }
    
    /// 获取当前正在执行的 Fiber ID
    pub fn get_current_fiber_id(&self) -> Option<&str> {
        self.current_fiber_id.as_deref()
    }
}

impl Default for FiberFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试 Fiber 创建
    #[test]
    fn test_fiber_creation() {
        // 创建 Fiber 工厂
        let mut factory = FiberFactory::new();
        
        // 创建 Fiber
        let fiber = factory.create(
            "test".to_string(),
            "test.php".to_string(),
            1,
        );
        
        // 验证初始状态
        assert_eq!(fiber.get_state(), FiberState::Created);
    }
    
    /// 测试 Fiber 状态转换
    #[test]
    fn test_fiber_state() {
        // 创建 Fiber
        let mut fiber = FiberValue::new(
            "test".to_string(),
            "test".to_string(),
            "test.php".to_string(),
            1,
        );
        
        // 初始状态为 Created
        assert_eq!(fiber.get_state(), FiberState::Created);
        assert!(!fiber.is_started());
        
        // start 后变为 Started
        fiber.start().unwrap();
        assert_eq!(fiber.get_state(), FiberState::Started);
        assert!(fiber.is_started());
        
        // suspend 后变为 Suspended
        fiber.suspend_json("\"suspended\"".to_string());
        assert_eq!(fiber.get_state(), FiberState::Suspended);
        assert!(fiber.is_suspended());
        
        // set_return 后变为 Terminated
        fiber.set_return_json("\"done\"".to_string());
        assert_eq!(fiber.get_state(), FiberState::Terminated);
        assert!(fiber.is_terminated());
    }
    
    /// 测试 Fiber 的 start 和 resume
    #[test]
    fn test_fiber_start_resume() {
        // 创建 Fiber
        let mut fiber = FiberValue::new(
            "test".to_string(),
            "test".to_string(),
            "test.php".to_string(),
            1,
        );
        
        // 启动 Fiber
        let result = fiber.start();
        assert!(result.is_ok());
        
        // 再次启动应该失败
        let result = fiber.start();
        assert!(result.is_err());
        
        // 挂起 Fiber
        fiber.suspend_json("\"suspended\"".to_string());
        
        // 恢复 Fiber
        let result = fiber.resume_json("\"resumed\"".to_string());
        assert!(result.is_ok());
        
        // 未挂起时恢复应该失败
        fiber.set_state_for_test(FiberState::Started);
        let result = fiber.resume_json("\"value\"".to_string());
        assert!(result.is_err());
    }
    
    /// 测试 Fiber 的 get_return
    #[test]
    fn test_fiber_get_return() {
        // 创建 Fiber
        let mut fiber = FiberValue::new(
            "test".to_string(),
            "test".to_string(),
            "test.php".to_string(),
            1,
        );
        
        // 未终止时获取返回值会报错
        assert!(fiber.get_return_json().is_err());
        
        // 终止并设置返回值
        fiber.set_return_json("\"done\"".to_string());
        
        // 验证返回值
        assert_eq!(fiber.get_return_json().unwrap(), Some("\"done\"".to_string()));
    }
    
    /// 测试 Fiber 的 throw
    #[test]
    fn test_fiber_throw() {
        // 创建 Fiber
        let mut fiber = FiberValue::new(
            "test".to_string(),
            "test".to_string(),
            "test.php".to_string(),
            1,
        );
        
        // 未挂起时 throw 应该失败
        let result = fiber.throw_json("\"exception\"".to_string());
        assert!(result.is_err());
        
        // 挂起 Fiber
        fiber.suspend_json("null".to_string());
        
        // throw 应该成功
        let result = fiber.throw_json("\"exception\"".to_string());
        assert!(result.is_ok());
    }
    
    /// 测试主 Fiber
    #[test]
    fn test_main_fiber() {
        // 创建主 Fiber
        let main_fiber = FiberValue::create_main_fiber();
        
        // 验证主 Fiber 属性
        assert!(main_fiber.is_main_fiber());
        assert_eq!(main_fiber.get_id(), "main");
        assert_eq!(main_fiber.get_state(), FiberState::Started);
    }
    
    /// 测试 FiberFactory
    #[test]
    fn test_fiber_factory() {
        // 创建 Fiber 工厂
        let mut factory = FiberFactory::new();
        
        // 创建多个 Fiber
        let fiber1 = factory.create(
            "test1".to_string(),
            "test.php".to_string(),
            1,
        );
        let fiber2 = factory.create(
            "test2".to_string(),
            "test.php".to_string(),
            2,
        );
        
        // 验证 ID 唯一性
        assert_ne!(fiber1.get_id(), fiber2.get_id());
        
        // 验证计数
        assert_eq!(factory.count(), 2);
    }
}

impl FiberValue {
    /// 测试辅助方法：设置状态
    #[cfg(test)]
    fn set_state_for_test(&mut self, state: FiberState) {
        let mut inner = self.inner.lock().unwrap();
        inner.state = state;
    }
}
