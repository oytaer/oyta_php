//! 定时器错误定义模块
//!
//! 定义定时器模块中使用的所有错误类型
//! 提供详细的错误信息以便调试和问题定位

use std::fmt;

/// 定时器错误类型
/// 定义定时器操作中可能发生的各种错误
#[derive(Debug, Clone)]
pub enum TimerError {
    /// 超时时间超出最大值
    /// 当请求的超时时间超过定时器支持的最大范围时返回
    TimeoutTooLarge {
        /// 请求的超时时间（毫秒）
        requested: u64,
        /// 支持的最大超时时间（毫秒）
        max_allowed: u64,
    },
    
    /// 任务不存在
    /// 当尝试操作一个不存在的任务时返回
    TaskNotFound {
        /// 任务 ID
        task_id: u64,
    },
    
    /// 任务已取消
    /// 当尝试操作一个已取消的任务时返回
    TaskAlreadyCancelled {
        /// 任务 ID
        task_id: u64,
    },
    
    /// 任务已完成
    /// 当尝试操作一个已完成的任务时返回
    TaskAlreadyCompleted {
        /// 任务 ID
        task_id: u64,
    },
    
    /// 时间轮已满
    /// 当时间轮中的任务数量达到上限时返回
    WheelFull {
        /// 当前层级
        level: u8,
        /// 当前槽位
        slot: usize,
    },
    
    /// 无效的层级配置
    /// 当时间轮的层级配置不正确时返回
    InvalidLevelConfig {
        /// 错误描述
        message: String,
    },
    
    /// 定时器未运行
    /// 当尝试在未启动的定时器上执行操作时返回
    TimerNotRunning,
    
    /// 定时器已停止
    /// 当尝试在已停止的定时器上执行操作时返回
    TimerAlreadyStopped,
    
    /// 回调执行错误
    /// 当任务回调执行过程中发生错误时返回
    CallbackError {
        /// 任务 ID
        task_id: u64,
        /// 错误信息
        message: String,
    },
    
    /// 无效的任务参数
    /// 当任务参数不正确时返回
    InvalidTaskParams {
        /// 错误描述
        message: String,
    },
    
    /// 内存分配错误
    /// 当内存分配失败时返回
    MemoryAllocationError {
        /// 错误描述
        message: String,
    },
    
    /// 并发访问错误
    /// 当发生并发访问冲突时返回
    ConcurrentAccessError {
        /// 错误描述
        message: String,
    },
}

/// 定时器操作结果类型
/// 使用 Result<T, TimerError> 作为定时器模块的标准返回类型
pub type TimerResult<T> = Result<T, TimerError>;

/// 实现 Display trait 以支持错误信息格式化
impl fmt::Display for TimerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // 超时时间超出最大值
            TimerError::TimeoutTooLarge { requested, max_allowed } => {
                write!(
                    f,
                    "超时时间 {}ms 超出最大允许值 {}ms",
                    requested, max_allowed
                )
            }
            
            // 任务不存在
            TimerError::TaskNotFound { task_id } => {
                write!(f, "任务 {} 不存在", task_id)
            }
            
            // 任务已取消
            TimerError::TaskAlreadyCancelled { task_id } => {
                write!(f, "任务 {} 已被取消", task_id)
            }
            
            // 任务已完成
            TimerError::TaskAlreadyCompleted { task_id } => {
                write!(f, "任务 {} 已完成", task_id)
            }
            
            // 时间轮已满
            TimerError::WheelFull { level, slot } => {
                write!(f, "时间轮层级 {} 槽位 {} 已满", level, slot)
            }
            
            // 无效的层级配置
            TimerError::InvalidLevelConfig { message } => {
                write!(f, "无效的层级配置: {}", message)
            }
            
            // 定时器未运行
            TimerError::TimerNotRunning => {
                write!(f, "定时器未运行")
            }
            
            // 定时器已停止
            TimerError::TimerAlreadyStopped => {
                write!(f, "定时器已停止")
            }
            
            // 回调执行错误
            TimerError::CallbackError { task_id, message } => {
                write!(f, "任务 {} 回调执行错误: {}", task_id, message)
            }
            
            // 无效的任务参数
            TimerError::InvalidTaskParams { message } => {
                write!(f, "无效的任务参数: {}", message)
            }
            
            // 内存分配错误
            TimerError::MemoryAllocationError { message } => {
                write!(f, "内存分配错误: {}", message)
            }
            
            // 并发访问错误
            TimerError::ConcurrentAccessError { message } => {
                write!(f, "并发访问错误: {}", message)
            }
        }
    }
}

/// 实现 std::error::Error trait
impl std::error::Error for TimerError {}

impl TimerError {
    /// 创建超时时间超出最大值错误
    ///
    /// # 参数
    /// - `requested`: 请求的超时时间（毫秒）
    /// - `max_allowed`: 支持的最大超时时间（毫秒）
    ///
    /// # 返回
    /// 返回 TimeoutTooLarge 错误实例
    pub fn timeout_too_large(requested: u64, max_allowed: u64) -> Self {
        TimerError::TimeoutTooLarge {
            requested,
            max_allowed,
        }
    }
    
    /// 创建任务不存在错误
    ///
    /// # 参数
    /// - `task_id`: 任务 ID
    ///
    /// # 返回
    /// 返回 TaskNotFound 错误实例
    pub fn task_not_found(task_id: u64) -> Self {
        TimerError::TaskNotFound { task_id }
    }
    
    /// 创建任务已取消错误
    ///
    /// # 参数
    /// - `task_id`: 任务 ID
    ///
    /// # 返回
    /// 返回 TaskAlreadyCancelled 错误实例
    pub fn task_already_cancelled(task_id: u64) -> Self {
        TimerError::TaskAlreadyCancelled { task_id }
    }
    
    /// 创建回调执行错误
    ///
    /// # 参数
    /// - `task_id`: 任务 ID
    /// - `message`: 错误信息
    ///
    /// # 返回
    /// 返回 CallbackError 错误实例
    pub fn callback_error(task_id: u64, message: impl Into<String>) -> Self {
        TimerError::CallbackError {
            task_id,
            message: message.into(),
        }
    }
    
    /// 创建无效任务参数错误
    ///
    /// # 参数
    /// - `message`: 错误描述
    ///
    /// # 返回
    /// 返回 InvalidTaskParams 错误实例
    pub fn invalid_task_params(message: impl Into<String>) -> Self {
        TimerError::InvalidTaskParams {
            message: message.into(),
        }
    }
    
    /// 检查是否为可恢复错误
    ///
    /// # 返回
    /// 如果错误可以恢复返回 true，否则返回 false
    pub fn is_recoverable(&self) -> bool {
        match self {
            // 这些错误可以恢复
            TimerError::TaskNotFound { .. } => true,
            TimerError::TaskAlreadyCancelled { .. } => true,
            TimerError::TaskAlreadyCompleted { .. } => true,
            TimerError::CallbackError { .. } => true,
            
            // 这些错误不可恢复
            TimerError::TimeoutTooLarge { .. } => false,
            TimerError::WheelFull { .. } => false,
            TimerError::InvalidLevelConfig { .. } => false,
            TimerError::TimerNotRunning => false,
            TimerError::TimerAlreadyStopped => false,
            TimerError::InvalidTaskParams { .. } => false,
            TimerError::MemoryAllocationError { .. } => false,
            TimerError::ConcurrentAccessError { .. } => false,
        }
    }
    
    /// 获取错误代码
    ///
    /// # 返回
    /// 返回错误的数字代码
    pub fn error_code(&self) -> u32 {
        match self {
            TimerError::TimeoutTooLarge { .. } => 1001,
            TimerError::TaskNotFound { .. } => 1002,
            TimerError::TaskAlreadyCancelled { .. } => 1003,
            TimerError::TaskAlreadyCompleted { .. } => 1004,
            TimerError::WheelFull { .. } => 1005,
            TimerError::InvalidLevelConfig { .. } => 1006,
            TimerError::TimerNotRunning => 1007,
            TimerError::TimerAlreadyStopped => 1008,
            TimerError::CallbackError { .. } => 1009,
            TimerError::InvalidTaskParams { .. } => 1010,
            TimerError::MemoryAllocationError { .. } => 1011,
            TimerError::ConcurrentAccessError { .. } => 1012,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试错误显示
    #[test]
    fn test_error_display() {
        // 测试超时时间超出最大值错误
        let error = TimerError::timeout_too_large(100000, 86400000);
        assert!(error.to_string().contains("100000"));
        assert!(error.to_string().contains("86400000"));
        
        // 测试任务不存在错误
        let error = TimerError::task_not_found(123);
        assert!(error.to_string().contains("123"));
    }
    
    /// 测试错误代码
    #[test]
    fn test_error_code() {
        let error = TimerError::timeout_too_large(0, 0);
        assert_eq!(error.error_code(), 1001);
        
        let error = TimerError::task_not_found(0);
        assert_eq!(error.error_code(), 1002);
    }
    
    /// 测试可恢复性检查
    #[test]
    fn test_is_recoverable() {
        // 可恢复错误
        assert!(TimerError::task_not_found(0).is_recoverable());
        assert!(TimerError::task_already_cancelled(0).is_recoverable());
        
        // 不可恢复错误
        assert!(!TimerError::timeout_too_large(0, 0).is_recoverable());
        assert!(!TimerError::TimerNotRunning.is_recoverable());
    }
}
