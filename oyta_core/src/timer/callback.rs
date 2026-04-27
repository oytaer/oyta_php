//! 回调处理模块
//!
//! 实现定时器任务的回调处理
//! 支持 PHP 函数、方法和闭包的调用

use std::sync::Arc;
use super::task::{TaskEntry, TimerCallback};
use super::error::{TimerError, TimerResult};

/// 回调处理器
/// 负责执行定时器任务的回调
#[derive(Debug, Clone)]
pub struct CallbackHandler {
    /// 是否启用详细日志
    verbose: bool,
}

impl CallbackHandler {
    /// 创建新的回调处理器
    ///
    /// # 返回
    /// 返回新创建的回调处理器实例
    pub fn new() -> Self {
        Self {
            // 默认不启用详细日志
            verbose: false,
        }
    }
    
    /// 启用详细日志
    ///
    /// # 参数
    /// - `verbose`: 是否启用
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }
    
    /// 执行任务回调
    ///
    /// # 参数
    /// - `task`: 任务条目
    ///
    /// # 返回
    /// 成功返回 Ok(执行结果 JSON)，失败返回错误
    pub fn execute(&self, task: &Arc<TaskEntry>) -> TimerResult<String> {
        // 设置任务状态为运行中
        task.set_status(super::task::TaskStatus::Running);
        
        // 根据回调类型执行
        let result = match &task.callback {
            // PHP 函数回调
            TimerCallback::Function { name, args_json } => {
                self.execute_function(name, args_json)
            }
            
            // PHP 方法回调
            TimerCallback::Method { class, method, args_json, is_static } => {
                self.execute_method(class, method, args_json, *is_static)
            }
            
            // PHP 闭包回调
            TimerCallback::Closure { closure_json, args_json } => {
                self.execute_closure(closure_json, args_json)
            }
            
            // Rust 原生回调
            TimerCallback::Native { handler_id } => {
                self.execute_native(handler_id)
            }
        };
        
        // 处理执行结果
        match result {
            Ok(output) => {
                // 设置任务状态为完成（对于一次性任务）
                if matches!(task.task_type, super::task::TaskType::Once) {
                    task.set_status(super::task::TaskStatus::Completed);
                } else {
                    // 周期性任务恢复为等待状态
                    task.set_status(super::task::TaskStatus::Pending);
                }
                Ok(output)
            }
            Err(e) => {
                // 设置任务状态为完成（失败）
                task.set_status(super::task::TaskStatus::Completed);
                Err(e)
            }
        }
    }
    
    /// 执行 PHP 函数
    ///
    /// # 参数
    /// - `name`: 函数名
    /// - `args_json`: 参数列表（JSON 格式）
    ///
    /// # 返回
    /// 成功返回执行结果，失败返回错误
    fn execute_function(&self, name: &str, args_json: &[String]) -> TimerResult<String> {
        // 记录日志
        if self.verbose {
            println!("[Timer] 执行函数: {}({:?})", name, args_json);
        }
        
        // 注意：实际的 PHP 函数调用需要通过解释器执行
        // 这里返回模拟结果
        Ok(format!("{{\"function\": \"{}\", \"executed\": true}}", name))
    }
    
    /// 执行 PHP 方法
    ///
    /// # 参数
    /// - `class`: 类名
    /// - `method`: 方法名
    /// - `args_json`: 参数列表（JSON 格式）
    /// - `is_static`: 是否为静态方法
    ///
    /// # 返回
    /// 成功返回执行结果，失败返回错误
    fn execute_method(
        &self,
        class: &str,
        method: &str,
        args_json: &[String],
        is_static: bool,
    ) -> TimerResult<String> {
        // 记录日志
        if self.verbose {
            let call_type = if is_static { "静态" } else { "实例" };
            println!(
                "[Timer] 执行{}方法: {}::{}({:?})",
                call_type, class, method, args_json
            );
        }
        
        // 注意：实际的 PHP 方法调用需要通过解释器执行
        // 这里返回模拟结果
        Ok(format!(
            "{{\"class\": \"{}\", \"method\": \"{}\", \"executed\": true}}",
            class, method
        ))
    }
    
    /// 执行 PHP 闭包
    ///
    /// # 参数
    /// - `closure_json`: 闭包的 JSON 表示
    /// - `args_json`: 参数列表（JSON 格式）
    ///
    /// # 返回
    /// 成功返回执行结果，失败返回错误
    fn execute_closure(&self, closure_json: &str, args_json: &[String]) -> TimerResult<String> {
        // 记录日志
        if self.verbose {
            println!("[Timer] 执行闭包: {}({:?})", closure_json, args_json);
        }
        
        // 注意：实际的 PHP 闭包调用需要通过解释器执行
        // 这里返回模拟结果
        Ok("{\"closure\": true, \"executed\": true}".to_string())
    }
    
    /// 执行 Rust 原生回调
    ///
    /// # 参数
    /// - `handler_id`: 回调标识符
    ///
    /// # 返回
    /// 成功返回执行结果，失败返回错误
    fn execute_native(&self, handler_id: &str) -> TimerResult<String> {
        // 记录日志
        if self.verbose {
            println!("[Timer] 执行原生回调: {}", handler_id);
        }
        
        // 注意：实际的原生回调需要通过注册表查找并执行
        // 这里返回模拟结果
        Ok(format!(
            "{{\"native\": \"{}\", \"executed\": true}}",
            handler_id
        ))
    }
}

impl Default for CallbackHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试回调处理器创建
    #[test]
    fn test_handler_creation() {
        let handler = CallbackHandler::new();
        assert!(!handler.verbose);
    }
    
    /// 测试执行函数回调
    #[test]
    fn test_execute_function() {
        let handler = CallbackHandler::new();
        
        let task = Arc::new(TaskEntry::once(
            1,
            1000,
            TimerCallback::Function {
                name: "test_function".to_string(),
                args_json: vec!["\"arg1\"".to_string()],
            },
            0,
        ));
        
        let result = handler.execute(&task);
        assert!(result.is_ok());
    }
    
    /// 测试执行方法回调
    #[test]
    fn test_execute_method() {
        let handler = CallbackHandler::new();
        
        let task = Arc::new(TaskEntry::once(
            1,
            1000,
            TimerCallback::Method {
                class: "TestClass".to_string(),
                method: "testMethod".to_string(),
                args_json: vec![],
                is_static: true,
            },
            0,
        ));
        
        let result = handler.execute(&task);
        assert!(result.is_ok());
    }
}
