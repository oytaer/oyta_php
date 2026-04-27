//! 输出缓冲类型定义模块
//!
//! 定义输出缓冲相关的核心类型

use serde::{Deserialize, Serialize};
use std::fmt;

use super::callback::OutputCallback;
use super::handler::OutputHandlerFlags;

/// 缓冲状态
///
/// 表示单个输出缓冲区的状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BufferState {
    /// 缓冲区内容
    pub content: String,
    /// 缓冲区大小（字节）
    pub size: usize,
    /// 块大小（用于回调触发）
    pub chunk_size: usize,
    /// 处理器标志
    pub flags: OutputHandlerFlags,
    /// 回调函数
    pub callback: Option<OutputCallback>,
    /// 缓冲区名称（用于 ob_list_handlers）
    pub handler_name: String,
    /// 是否已删除
    pub deleted: bool,
    /// 创建时间戳
    pub created_at: u64,
}

impl BufferState {
    /// 创建新的缓冲状态
    pub fn new() -> Self {
        Self {
            content: String::new(),
            size: 0,
            chunk_size: 0,
            flags: OutputHandlerFlags::default(),
            callback: None,
            handler_name: "default output handler".to_string(),
            deleted: false,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
    
    /// 创建带回调的缓冲状态
    pub fn with_callback(callback: OutputCallback, chunk_size: usize, flags: OutputHandlerFlags) -> Self {
        let handler_name = callback.name().to_string();
        Self {
            content: String::new(),
            size: 0,
            chunk_size,
            flags,
            callback: Some(callback),
            handler_name,
            deleted: false,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
    
    /// 追加内容到缓冲区
    pub fn append(&mut self, data: &str) {
        self.content.push_str(data);
        self.size = self.content.len();
    }
    
    /// 清空缓冲区内容
    pub fn clear(&mut self) {
        self.content.clear();
        self.size = 0;
    }
    
    /// 获取缓冲区内容
    pub fn get_content(&self) -> &str {
        &self.content
    }
    
    /// 获取缓冲区长度
    pub fn len(&self) -> usize {
        self.size
    }
    
    /// 检查缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
    
    /// 提取并清空缓冲区内容
    pub fn take_content(&mut self) -> String {
        let content = std::mem::take(&mut self.content);
        self.size = 0;
        content
    }
    
    /// 检查是否需要触发回调
    pub fn should_trigger_callback(&self) -> bool {
        // 如果设置了块大小，检查是否达到
        if self.chunk_size > 0 && self.size >= self.chunk_size {
            return true;
        }
        
        false
    }
    
    /// 检查是否可删除
    pub fn can_delete(&self) -> bool {
        !self.deleted && self.flags.contains(OutputHandlerFlags::REMOVABLE)
    }
    
    /// 检查是否可清空
    pub fn can_clean(&self) -> bool {
        !self.deleted && self.flags.contains(OutputHandlerFlags::CLEANABLE)
    }
    
    /// 标记为已删除
    pub fn mark_deleted(&mut self) {
        self.deleted = true;
    }
}

impl Default for BufferState {
    fn default() -> Self {
        Self::new()
    }
}

/// 输出处理器
///
/// 表示一个输出处理器的信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputHandler {
    /// 处理器名称
    pub name: String,
    /// 处理器类型
    pub handler_type: OutputHandlerType,
    /// 处理器标志
    pub flags: OutputHandlerFlags,
}

impl OutputHandler {
    /// 创建新的输出处理器
    pub fn new(name: &str, handler_type: OutputHandlerType) -> Self {
        Self {
            name: name.to_string(),
            handler_type,
            flags: OutputHandlerFlags::default(),
        }
    }
    
    /// 创建带标志的输出处理器
    pub fn with_flags(name: &str, handler_type: OutputHandlerType, flags: OutputHandlerFlags) -> Self {
        Self {
            name: name.to_string(),
            handler_type,
            flags,
        }
    }
}

impl fmt::Display for OutputHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// 输出处理器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutputHandlerType {
    /// 默认处理器
    Default,
    /// 用户自定义回调处理器
    Callback,
    /// 内置处理器（如 mb_output_handler）
    Builtin,
    /// URL 重写处理器
    UrlRewrite,
    /// 压缩处理器
    Compression,
}

impl fmt::Display for OutputHandlerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputHandlerType::Default => write!(f, "default"),
            OutputHandlerType::Callback => write!(f, "callback"),
            OutputHandlerType::Builtin => write!(f, "builtin"),
            OutputHandlerType::UrlRewrite => write!(f, "url_rewrite"),
            OutputHandlerType::Compression => write!(f, "compression"),
        }
    }
}

/// 输出缓冲级别信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BufferLevelInfo {
    /// 缓冲级别
    pub level: usize,
    /// 处理器名称
    pub handler_name: String,
    /// 缓冲区大小
    pub size: usize,
    /// 块大小
    pub chunk_size: usize,
    /// 标志
    pub flags: OutputHandlerFlags,
}

impl BufferLevelInfo {
    /// 从缓冲状态创建
    pub fn from_state(level: usize, state: &BufferState) -> Self {
        Self {
            level,
            handler_name: state.handler_name.clone(),
            size: state.size,
            chunk_size: state.chunk_size,
            flags: state.flags,
        }
    }
}

/// 输出状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutputStatus {
    /// 正常输出
    Normal,
    /// 输出已开始
    Started,
    /// 输出已发送
    Sent,
    /// 输出已禁用
    Disabled,
}

impl Default for OutputStatus {
    fn default() -> Self {
        OutputStatus::Normal
    }
}

/// 输出缓冲操作结果
#[derive(Debug, Clone, PartialEq)]
pub enum BufferResult {
    /// 操作成功
    Success,
    /// 操作失败
    Failed(String),
    /// 没有活动的缓冲区
    NoBuffer,
    /// 缓冲区为空
    Empty,
}

impl BufferResult {
    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        matches!(self, BufferResult::Success)
    }
    
    /// 检查是否失败
    pub fn is_failed(&self) -> bool {
        matches!(self, BufferResult::Failed(_))
    }
    
    /// 获取错误消息
    pub fn error_message(&self) -> Option<&str> {
        match self {
            BufferResult::Failed(msg) => Some(msg),
            _ => None,
        }
    }
}

/// 输出刷新结果
#[derive(Debug, Clone, PartialEq)]
pub struct FlushResult {
    /// 刷新的内容
    pub content: String,
    /// 是否执行了回调
    pub callback_executed: bool,
    /// 是否成功
    pub success: bool,
}

impl FlushResult {
    /// 创建成功的刷新结果
    pub fn success(content: String, callback_executed: bool) -> Self {
        Self {
            content,
            callback_executed,
            success: true,
        }
    }
    
    /// 创建失败的刷新结果
    pub fn failed() -> Self {
        Self {
            content: String::new(),
            callback_executed: false,
            success: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_buffer_state_creation() {
        let state = BufferState::new();
        
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
        assert_eq!(state.handler_name, "default output handler");
    }
    
    #[test]
    fn test_buffer_state_append() {
        let mut state = BufferState::new();
        
        state.append("Hello");
        assert_eq!(state.len(), 5);
        assert_eq!(state.get_content(), "Hello");
        
        state.append(" World");
        assert_eq!(state.len(), 11);
        assert_eq!(state.get_content(), "Hello World");
    }
    
    #[test]
    fn test_buffer_state_clear() {
        let mut state = BufferState::new();
        state.append("Test content");
        
        state.clear();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
    }
    
    #[test]
    fn test_buffer_state_take_content() {
        let mut state = BufferState::new();
        state.append("Content to take");
        
        let content = state.take_content();
        assert_eq!(content, "Content to take");
        assert!(state.is_empty());
    }
    
    #[test]
    fn test_output_handler() {
        let handler = OutputHandler::new("my_handler", OutputHandlerType::Callback);
        
        assert_eq!(handler.name, "my_handler");
        assert_eq!(handler.handler_type, OutputHandlerType::Callback);
    }
    
    #[test]
    fn test_buffer_level_info() {
        let state = BufferState::new();
        let info = BufferLevelInfo::from_state(1, &state);
        
        assert_eq!(info.level, 1);
        assert_eq!(info.handler_name, "default output handler");
        assert_eq!(info.size, 0);
    }
    
    #[test]
    fn test_buffer_result() {
        let success = BufferResult::Success;
        assert!(success.is_success());
        assert!(!success.is_failed());
        
        let failed = BufferResult::Failed("Error message".to_string());
        assert!(!failed.is_success());
        assert!(failed.is_failed());
        assert_eq!(failed.error_message(), Some("Error message"));
    }
}
