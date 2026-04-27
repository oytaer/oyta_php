//! Generator 生成器模块
//!
//! 实现 PHP 的 Generator/yield 功能支持
//! Generator 是 PHP 5.5+ 引入的特性，用于实现迭代器和协程
//!
//! # 功能特性
//! - yield 关键字支持
//! - yield from 委托生成器
//! - Generator 对象方法：current(), key(), next(), rewind(), valid(), send(), throw(), getReturn()
//! - 生成器状态管理：创建、运行、挂起、关闭
//!
//! # 使用示例
//! ```php
//! function getUsers(): Generator {
//!     foreach (User::chunk(100) as $users) {
//!         foreach ($users as $user) {
//!             yield $user;
//!         }
//!     }
//! }
//! 
//! foreach (getUsers() as $user) {
//!     echo $user->name;
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Generator 生成器状态
/// 表示生成器在生命周期中的不同状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeneratorState {
    /// 已创建但未启动
    /// 当 Generator 被创建但尚未调用 current()/next()/rewind() 时处于此状态
    Created,
    
    /// 正在运行
    /// 当 Generator 正在执行 yield 之间的代码时处于此状态
    Running,
    
    /// 已挂起
    /// 当 Generator 执行到 yield 语句并返回值后处于此状态
    /// 此时可以调用 current() 获取当前值，或 send() 发送新值
    Suspended,
    
    /// 已关闭
    /// 当 Generator 执行完毕（return 或所有代码执行完）后处于此状态
    /// 此时调用 valid() 返回 false
    Closed,
}

/// Generator 生成器内部状态
/// 使用 Arc<Mutex<>> 包装以支持共享访问
#[derive(Debug, Clone)]
pub struct GeneratorInner {
    /// 生成器唯一标识符
    /// 用于区分不同的生成器实例
    pub id: String,
    
    /// 当前状态
    /// 表示生成器当前处于哪个生命周期阶段
    pub state: GeneratorState,
    
    /// 当前 yield 的值（JSON 序列化存储）
    /// 当生成器挂起时，此字段保存 yield 后面的表达式值
    pub current_value_json: Option<String>,
    
    /// 当前 yield 的键（JSON 序列化存储）
    /// yield 可以带键名，如 yield 'key' => $value
    /// 如果没有指定键名，默认为数字索引（从 0 开始）
    pub current_key_json: Option<String>,
    
    /// 当前索引计数器
    /// 用于没有指定键名时自动生成数字索引
    /// 每次 yield 后自动递增
    pub index: i64,
    
    /// 通过 send() 发送的值（JSON 序列化存储）
    /// 当调用 send() 方法时，此值会作为 yield 表达式的返回值
    pub sent_value_json: Option<String>,
    
    /// 通过 throw() 抛出的异常（JSON 序列化存储）
    /// 当调用 throw() 方法时，此异常会在 yield 点被抛出
    pub thrown_exception_json: Option<String>,
    
    /// 生成器函数名
    /// 保存生成器函数的名称
    pub function_name: String,
    
    /// 生成器参数（JSON 序列化存储）
    /// 调用生成器函数时传入的参数列表
    pub args_json: Vec<String>,
    
    /// 返回值（JSON 序列化存储）
    /// 当生成器执行完毕（通过 return 语句）时保存返回值
    pub return_value_json: Option<String>,
    
    /// 当前执行位置标记
    /// 用于记录生成器执行到哪个 yield 语句
    pub execution_pointer: usize,
    
    /// 是否已启动
    /// 标记生成器是否已经执行过第一次 yield
    pub started: bool,
    
    /// 生成器来源文件路径
    /// 用于调试和错误信息
    pub file_path: String,
    
    /// 生成器起始行号
    /// 用于调试和错误信息
    pub start_line: usize,
}

/// Generator 生成器值类型
/// 表示一个 PHP Generator 对象，支持 yield 语法
/// 使用 Arc<Mutex<>> 包装内部状态以支持共享访问
#[derive(Debug, Clone)]
pub struct GeneratorValue {
    /// 内部状态（共享引用）
    /// 使用 Arc<Mutex<>> 支持多线程共享和修改
    inner: Arc<Mutex<GeneratorInner>>,
}

/// 为 GeneratorValue 实现 PartialEq trait
/// 通过比较唯一标识符来判断两个 Generator 是否相等
impl PartialEq for GeneratorValue {
    fn eq(&self, other: &Self) -> bool {
        // 获取两个 Generator 的内部状态锁
        let self_inner = self.inner.lock().unwrap();
        let other_inner = other.inner.lock().unwrap();
        // 比较唯一标识符
        self_inner.id == other_inner.id
    }
}

/// Generator 实现的方法列表
/// 这些方法可以通过 Generator 对象调用
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratorMethod {
    /// current() - 获取当前值
    /// 返回当前 yield 的值，不移动指针
    Current,
    
    /// key() - 获取当前键
    /// 返回当前 yield 的键名
    Key,
    
    /// next() - 前进到下一个值
    /// 继续执行生成器直到下一个 yield
    Next,
    
    /// rewind() - 重置生成器
    /// 将生成器重置到初始状态（PHP 生成器不支持真正的 rewind）
    Rewind,
    
    /// valid() - 检查是否有效
    /// 如果生成器未关闭则返回 true
    Valid,
    
    /// send($value) - 发送值到生成器
    /// 发送一个值到生成器，作为 yield 表达式的返回值
    Send,
    
    /// throw($exception) - 向生成器抛出异常
    /// 在当前 yield 点抛出一个异常
    Throw,
    
    /// getReturn() - 获取返回值
    /// 获取生成器的 return 语句返回的值
    GetReturn,
    
    /// __wakeup() - 反序列化时抛出异常
    /// Generator 不支持序列化
    Wakeup,
}

impl GeneratorValue {
    /// 创建新的 Generator 实例
    ///
    /// # 参数
    /// - `id`: 生成器唯一标识符
    /// - `function_name`: 生成器函数名
    /// - `file_path`: 生成器所在文件路径
    /// - `start_line`: 生成器起始行号
    ///
    /// # 返回
    /// 返回新创建的 Generator 实例，初始状态为 Created
    pub fn new(
        id: String,
        function_name: String,
        file_path: String,
        start_line: usize,
    ) -> Self {
        // 创建内部状态
        let inner = GeneratorInner {
            // 设置唯一标识符
            id,
            // 初始状态为已创建
            state: GeneratorState::Created,
            // 当前值初始为空
            current_value_json: None,
            // 当前键初始为空
            current_key_json: None,
            // 索引从 0 开始
            index: 0,
            // 发送的值初始为空
            sent_value_json: None,
            // 抛出的异常初始为空
            thrown_exception_json: None,
            // 保存生成器函数名
            function_name,
            // 参数初始为空
            args_json: Vec::new(),
            // 返回值初始为空
            return_value_json: None,
            // 执行指针从 0 开始
            execution_pointer: 0,
            // 尚未启动
            started: false,
            // 保存文件路径
            file_path,
            // 保存起始行号
            start_line,
        };
        
        // 返回 Generator 实例
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
    
    /// 获取当前值（JSON 字符串）
    /// 对应 PHP 的 Generator::current() 方法
    ///
    /// # 返回
    /// 返回当前 yield 值的 JSON 字符串表示
    pub fn current_json(&self) -> Option<String> {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        
        // 检查生成器是否已关闭
        if inner.state == GeneratorState::Closed {
            return None;
        }
        
        // 检查是否未启动
        if inner.state == GeneratorState::Created {
            return None;
        }
        
        // 返回当前值的 JSON 字符串
        inner.current_value_json.clone()
    }
    
    /// 获取当前键（JSON 字符串）
    /// 对应 PHP 的 Generator::key() 方法
    ///
    /// # 返回
    /// 返回当前 yield 键的 JSON 字符串表示
    pub fn key_json(&self) -> Option<String> {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        
        // 检查生成器是否已关闭
        if inner.state == GeneratorState::Closed {
            return None;
        }
        
        // 检查是否未启动
        if inner.state == GeneratorState::Created {
            return None;
        }
        
        // 如果有显式键名，返回键名的 JSON 字符串
        if let Some(ref key_json) = inner.current_key_json {
            return Some(key_json.clone());
        }
        
        // 否则返回自动生成的数字索引
        Some(format!("{}", inner.index - 1))
    }
    
    /// 前进到下一个值
    /// 对应 PHP 的 Generator::next() 方法
    pub fn next(&mut self) -> GeneratorState {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        
        // 检查生成器是否已关闭
        if inner.state == GeneratorState::Closed {
            return GeneratorState::Closed;
        }
        
        // 清除之前发送的值
        inner.sent_value_json = None;
        
        // 清除之前抛出的异常
        inner.thrown_exception_json = None;
        
        // 设置状态为运行中
        inner.state = GeneratorState::Running;
        
        // 标记为已启动
        inner.started = true;
        
        // 返回当前状态
        inner.state
    }
    
    /// 重置生成器
    /// 对应 PHP 的 Generator::rewind() 方法
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误信息
    pub fn rewind(&mut self) -> Result<(), String> {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        
        // 检查生成器是否已启动
        if inner.started {
            // PHP 行为：已启动的生成器不能 rewind
            return Err("Cannot rewind a generator that was already run".to_string());
        }
        
        // 检查是否已关闭
        if inner.state == GeneratorState::Closed {
            // 已关闭的生成器不能 rewind
            return Err("Cannot rewind a closed generator".to_string());
        }
        
        // 设置状态为运行中
        inner.state = GeneratorState::Running;
        
        // 标记为已启动
        inner.started = true;
        
        // 返回成功
        Ok(())
    }
    
    /// 检查生成器是否有效
    /// 对应 PHP 的 Generator::valid() 方法
    ///
    /// # 返回
    /// 如果生成器未关闭返回 true，否则返回 false
    pub fn valid(&self) -> bool {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        
        // 检查状态是否为已关闭
        inner.state != GeneratorState::Closed
    }
    
    /// 发送值到生成器
    /// 对应 PHP 的 Generator::send() 方法
    ///
    /// # 参数
    /// - `value_json`: 要发送到生成器的值的 JSON 字符串
    pub fn send_json(&mut self, value_json: String) {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        
        // 检查生成器是否已关闭
        if inner.state == GeneratorState::Closed {
            return;
        }
        
        // 保存发送的值
        inner.sent_value_json = Some(value_json);
        
        // 设置状态为运行中
        inner.state = GeneratorState::Running;
    }
    
    /// 向生成器抛出异常
    /// 对应 PHP 的 Generator::throw() 方法
    ///
    /// # 参数
    /// - `exception_json`: 要抛出的异常的 JSON 字符串
    pub fn throw_json(&mut self, exception_json: String) {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        
        // 检查生成器是否已关闭
        if inner.state == GeneratorState::Closed {
            return;
        }
        
        // 保存要抛出的异常
        inner.thrown_exception_json = Some(exception_json);
        
        // 设置状态为运行中
        inner.state = GeneratorState::Running;
    }
    
    /// 获取生成器的返回值（JSON 字符串）
    /// 对应 PHP 的 Generator::getReturn() 方法
    ///
    /// # 返回
    /// 成功返回返回值的 JSON 字符串，失败返回错误信息
    pub fn get_return_json(&self) -> Result<Option<String>, String> {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        
        // 检查生成器是否已关闭
        if inner.state != GeneratorState::Closed {
            // 未关闭的生成器不能获取返回值
            return Err("Cannot get return value of a generator that hasn't returned".to_string());
        }
        
        // 返回保存的返回值
        Ok(inner.return_value_json.clone())
    }
    
    /// 设置当前 yield 的值和键
    /// 由解释器在执行 yield 语句时调用
    ///
    /// # 参数
    /// - `value_json`: yield 的值的 JSON 字符串
    /// - `key_json`: yield 的键的 JSON 字符串（可选）
    pub fn set_yield_json(&mut self, value_json: String, key_json: Option<String>) {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        
        // 设置当前值
        inner.current_value_json = Some(value_json);
        
        // 设置当前键（如果提供）
        inner.current_key_json = key_json;
        
        // 递增索引计数器
        inner.index += 1;
        
        // 设置状态为已挂起
        inner.state = GeneratorState::Suspended;
    }
    
    /// 设置返回值并关闭生成器
    /// 由解释器在生成器执行 return 语句或结束时调用
    ///
    /// # 参数
    /// - `value_json`: 返回值的 JSON 字符串
    pub fn set_return_json(&mut self, value_json: String) {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        
        // 保存返回值
        inner.return_value_json = Some(value_json);
        
        // 清空当前值
        inner.current_value_json = None;
        
        // 清空当前键
        inner.current_key_json = None;
        
        // 设置状态为已关闭
        inner.state = GeneratorState::Closed;
    }
    
    /// 关闭生成器
    /// 用于强制关闭生成器（如循环中断时）
    pub fn close(&mut self) {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        
        // 清空当前值
        inner.current_value_json = None;
        
        // 清空当前键
        inner.current_key_json = None;
        
        // 设置状态为已关闭
        inner.state = GeneratorState::Closed;
    }
    
    /// 获取发送的值（JSON 字符串）
    /// 由解释器在执行 yield 表达式时调用
    pub fn get_sent_value_json(&self) -> Option<String> {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        
        // 返回发送的值
        inner.sent_value_json.clone()
    }
    
    /// 获取要抛出的异常（JSON 字符串）
    /// 由解释器在 yield 点检查时调用
    pub fn get_thrown_exception_json(&self) -> Option<String> {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        
        // 返回要抛出的异常
        inner.thrown_exception_json.clone()
    }
    
    /// 清除发送的值
    pub fn clear_sent_value(&mut self) {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        
        // 清空发送的值
        inner.sent_value_json = None;
    }
    
    /// 清除抛出的异常
    pub fn clear_thrown_exception(&mut self) {
        // 获取内部状态的可变锁
        let mut inner = self.inner.lock().unwrap();
        
        // 清空抛出的异常
        inner.thrown_exception_json = None;
    }
    
    /// 检查生成器是否已启动
    pub fn is_started(&self) -> bool {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        inner.started
    }
    
    /// 获取生成器状态
    pub fn get_state(&self) -> GeneratorState {
        // 获取内部状态的锁
        let inner = self.inner.lock().unwrap();
        inner.state
    }
    
    /// 获取生成器唯一标识符
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
    
    /// 获取生成器函数名
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
}

/// Yield 表达式类型
/// 用于区分不同的 yield 语法形式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YieldKind {
    /// 简单 yield：yield $value
    /// 只返回值，键自动为数字索引
    Simple,
    
    /// 带键 yield：yield $key => $value
    /// 返回键值对
    WithKey,
    
    /// yield from：yield from $iterable
    /// 委托给另一个可迭代对象或生成器
    YieldFrom,
}

/// Yield 表达式信息
/// 保存 yield 语句的完整信息
#[derive(Debug, Clone)]
pub struct YieldInfo {
    /// yield 类型
    pub kind: YieldKind,
    
    /// yield 的值（JSON 字符串）
    pub value_json: String,
    
    /// yield 的键（JSON 字符串，可选）
    pub key_json: Option<String>,
    
    /// yield from 的可迭代对象（JSON 字符串，可选）
    pub from_iterable_json: Option<String>,
    
    /// yield 语句所在文件路径
    pub file_path: String,
    
    /// yield 语句所在行号
    pub line: usize,
}

impl YieldInfo {
    /// 创建简单的 yield 信息
    pub fn simple(value_json: String, file_path: String, line: usize) -> Self {
        Self {
            kind: YieldKind::Simple,
            value_json,
            key_json: None,
            from_iterable_json: None,
            file_path,
            line,
        }
    }
    
    /// 创建带键的 yield 信息
    pub fn with_key(key_json: String, value_json: String, file_path: String, line: usize) -> Self {
        Self {
            kind: YieldKind::WithKey,
            value_json,
            key_json: Some(key_json),
            from_iterable_json: None,
            file_path,
            line,
        }
    }
    
    /// 创建 yield from 信息
    pub fn yield_from(iterable_json: String, file_path: String, line: usize) -> Self {
        Self {
            kind: YieldKind::YieldFrom,
            value_json: String::new(),
            key_json: None,
            from_iterable_json: Some(iterable_json),
            file_path,
            line,
        }
    }
}

/// Generator 工厂
/// 用于创建和管理 Generator 实例
pub struct GeneratorFactory {
    /// 计数器，用于生成唯一 ID
    counter: u64,
}

impl GeneratorFactory {
    /// 创建新的 Generator 工厂
    pub fn new() -> Self {
        Self {
            counter: 0,
        }
    }
    
    /// 创建新的 Generator 实例
    ///
    /// # 参数
    /// - `function_name`: 生成器函数名
    /// - `file_path`: 文件路径
    /// - `start_line`: 起始行号
    ///
    /// # 返回
    /// 返回新创建的 Generator 实例
    pub fn create(
        &mut self,
        function_name: String,
        file_path: String,
        start_line: usize,
    ) -> GeneratorValue {
        // 递增计数器
        self.counter += 1;
        
        // 生成唯一 ID
        let id = format!("generator_{}", self.counter);
        
        // 创建并返回 Generator 实例
        GeneratorValue::new(id, function_name, file_path, start_line)
    }
    
    /// 获取已创建的 Generator 数量
    pub fn count(&self) -> u64 {
        self.counter
    }
}

impl Default for GeneratorFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试 Generator 创建
    #[test]
    fn test_generator_creation() {
        // 创建 Generator 工厂
        let mut factory = GeneratorFactory::new();
        
        // 创建 Generator
        let generator = factory.create(
            "test".to_string(),
            "test.php".to_string(),
            1,
        );
        
        // 验证初始状态
        assert_eq!(generator.get_state(), GeneratorState::Created);
        assert!(!generator.is_started());
    }
    
    /// 测试 Generator 状态转换
    #[test]
    fn test_generator_state() {
        // 创建 Generator
        let mut generator = GeneratorValue::new(
            "test".to_string(),
            "test".to_string(),
            "test.php".to_string(),
            1,
        );
        
        // 初始状态为 Created
        assert_eq!(generator.get_state(), GeneratorState::Created);
        
        // rewind 后变为 Running
        generator.rewind().unwrap();
        assert_eq!(generator.get_state(), GeneratorState::Running);
        assert!(generator.is_started());
        
        // 设置 yield 后变为 Suspended
        generator.set_yield_json("\"value\"".to_string(), None);
        assert_eq!(generator.get_state(), GeneratorState::Suspended);
        
        // 关闭后变为 Closed
        generator.close();
        assert_eq!(generator.get_state(), GeneratorState::Closed);
    }
    
    /// 测试 Generator 的 valid 方法
    #[test]
    fn test_generator_valid() {
        // 创建 Generator
        let mut generator = GeneratorValue::new(
            "test".to_string(),
            "test".to_string(),
            "test.php".to_string(),
            1,
        );
        
        // 未关闭时 valid 为 true
        assert!(generator.valid());
        
        // 关闭后 valid 为 false
        generator.close();
        assert!(!generator.valid());
    }
    
    /// 测试 Generator 的 get_return 方法
    #[test]
    fn test_generator_get_return() {
        // 创建 Generator
        let mut generator = GeneratorValue::new(
            "test".to_string(),
            "test".to_string(),
            "test.php".to_string(),
            1,
        );
        
        // 未关闭时获取返回值会报错
        assert!(generator.get_return_json().is_err());
        
        // 关闭并设置返回值
        generator.set_return_json("\"done\"".to_string());
        
        // 验证返回值
        assert_eq!(generator.get_return_json().unwrap(), Some("\"done\"".to_string()));
    }
    
    /// 测试 YieldInfo 创建
    #[test]
    fn test_yield_info() {
        // 测试简单 yield
        let info = YieldInfo::simple(
            "1".to_string(),
            "test.php".to_string(),
            10,
        );
        assert_eq!(info.kind, YieldKind::Simple);
        
        // 测试带键 yield
        let info = YieldInfo::with_key(
            "\"key\"".to_string(),
            "1".to_string(),
            "test.php".to_string(),
            10,
        );
        assert_eq!(info.kind, YieldKind::WithKey);
        assert_eq!(info.key_json, Some("\"key\"".to_string()));
        
        // 测试 yield from
        let info = YieldInfo::yield_from(
            "[1,2]".to_string(),
            "test.php".to_string(),
            10,
        );
        assert_eq!(info.kind, YieldKind::YieldFrom);
        assert!(info.from_iterable_json.is_some());
    }
}
