//! 输出缓冲栈管理模块
//!
//! 提供完整的 PHP 输出缓冲控制功能
//! 包括：ob_start, ob_get_contents, ob_end_flush 等

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;

use super::callback::{CallbackContext, CallbackResult, OutputCallback};
use super::handler::OutputHandlerFlags;
use super::types::{BufferLevelInfo, BufferResult, BufferState, FlushResult, OutputHandler, OutputHandlerType};

/// 输出缓冲管理器
///
/// 管理输出缓冲栈，实现 PHP 的输出缓冲控制功能
#[derive(Debug)]
pub struct OutputBuffer {
    /// 缓冲栈（使用 VecDeque 实现栈操作）
    buffers: VecDeque<BufferState>,
    /// 输出内容（已刷新的内容）
    output: String,
    /// 是否启用输出缓冲
    enabled: bool,
    /// 输出回调
    output_callback: Option<OutputCallback>,
}

impl OutputBuffer {
    /// 创建新的输出缓冲管理器
    pub fn new() -> Self {
        Self {
            buffers: VecDeque::new(),
            output: String::new(),
            enabled: true,
            output_callback: None,
        }
    }
    
    /// 启动输出缓冲
    ///
    /// 对应 PHP 的 ob_start() 函数
    ///
    /// # 参数
    /// - `callback`: 可选的回调函数
    /// - `chunk_size`: 块大小（0 表示不自动刷新）
    /// - `flags`: 处理器标志
    ///
    /// # 返回
    /// 是否成功启动
    pub fn start(
        &mut self,
        callback: Option<OutputCallback>,
        chunk_size: usize,
        flags: OutputHandlerFlags,
    ) -> bool {
        // 创建新的缓冲状态
        let state = match callback {
            Some(cb) => BufferState::with_callback(cb, chunk_size, flags),
            None => {
                let mut state = BufferState::new();
                state.flags = flags;
                state.chunk_size = chunk_size;
                state
            }
        };
        
        // 添加到栈顶
        self.buffers.push_front(state);
        
        true
    }
    
    /// 获取当前缓冲区内容
    ///
    /// 对应 PHP 的 ob_get_contents() 函数
    ///
    /// # 返回
    /// 当前缓冲区内容，如果没有缓冲区返回 None
    pub fn get_contents(&self) -> Option<&str> {
        self.buffers.front().map(|s| s.get_content())
    }
    
    /// 获取当前缓冲区内容并清空
    ///
    /// 对应 PHP 的 ob_get_clean() 函数
    ///
    /// # 返回
    /// 当前缓冲区内容，如果没有缓冲区返回 None
    pub fn get_clean(&mut self) -> Option<String> {
        if let Some(state) = self.buffers.front_mut() {
            let content = state.take_content();
            Some(content)
        } else {
            None
        }
    }
    
    /// 获取当前缓冲区内容并刷新
    ///
    /// 对应 PHP 的 ob_get_flush() 函数
    ///
    /// # 返回
    /// 当前缓冲区内容，如果没有缓冲区返回 None
    pub fn get_flush(&mut self) -> Option<String> {
        if let Some(state) = self.buffers.front_mut() {
            let content = state.take_content();
            
            // 执行回调（如果有）
            if let Some(callback) = &state.callback {
                let result = callback.execute(&content, super::handler::OutputHandlerStatus::FLUSH);
                if let CallbackResult::Processed(processed) = result {
                    self.output.push_str(&processed);
                } else {
                    self.output.push_str(&content);
                }
            } else {
                self.output.push_str(&content);
            }
            
            Some(content)
        } else {
            None
        }
    }
    
    /// 刷新当前缓冲区
    ///
    /// 对应 PHP 的 ob_flush() 函数
    ///
    /// # 返回
    /// 操作结果
    pub fn flush(&mut self) -> BufferResult {
        if self.buffers.is_empty() {
            return BufferResult::NoBuffer;
        }
        
        if let Some(state) = self.buffers.front_mut() {
            if state.is_empty() {
                return BufferResult::Empty;
            }
            
            let content = state.take_content();
            
            // 执行回调（如果有）
            if let Some(callback) = &state.callback {
                let result = callback.execute(&content, super::handler::OutputHandlerStatus::FLUSH);
                if let CallbackResult::Processed(processed) = result {
                    self.output.push_str(&processed);
                } else {
                    self.output.push_str(&content);
                }
            } else {
                self.output.push_str(&content);
            }
            
            BufferResult::Success
        } else {
            BufferResult::NoBuffer
        }
    }
    
    /// 清空当前缓冲区
    ///
    /// 对应 PHP 的 ob_clean() 函数
    ///
    /// # 返回
    /// 操作结果
    pub fn clean(&mut self) -> BufferResult {
        if self.buffers.is_empty() {
            return BufferResult::NoBuffer;
        }
        
        if let Some(state) = self.buffers.front_mut() {
            state.clear();
            BufferResult::Success
        } else {
            BufferResult::NoBuffer
        }
    }
    
    /// 刷新并关闭当前缓冲区
    ///
    /// 对应 PHP 的 ob_end_flush() 函数
    ///
    /// # 返回
    /// 操作结果
    pub fn end_flush(&mut self) -> BufferResult {
        if self.buffers.is_empty() {
            return BufferResult::NoBuffer;
        }
        
        // 先刷新
        self.flush();
        
        // 移除缓冲区
        self.buffers.pop_front();
        
        BufferResult::Success
    }
    
    /// 清空并关闭当前缓冲区
    ///
    /// 对应 PHP 的 ob_end_clean() 函数
    ///
    /// # 返回
    /// 操作结果
    pub fn end_clean(&mut self) -> BufferResult {
        if self.buffers.is_empty() {
            return BufferResult::NoBuffer;
        }
        
        // 移除缓冲区（不刷新）
        self.buffers.pop_front();
        
        BufferResult::Success
    }
    
    /// 获取缓冲区层级
    ///
    /// 对应 PHP 的 ob_get_level() 函数
    ///
    /// # 返回
    /// 当前缓冲区层级
    pub fn get_level(&self) -> usize {
        self.buffers.len()
    }
    
    /// 获取当前缓冲区长度
    ///
    /// 对应 PHP 的 ob_get_length() 函数
    ///
    /// # 返回
    /// 当前缓冲区长度，如果没有缓冲区返回 false
    pub fn get_length(&self) -> Option<usize> {
        self.buffers.front().map(|s| s.len())
    }
    
    /// 获取所有处理器列表
    ///
    /// 对应 PHP 的 ob_list_handlers() 函数
    ///
    /// # 返回
    /// 处理器名称列表
    pub fn list_handlers(&self) -> Vec<String> {
        self.buffers
            .iter()
            .map(|s| s.handler_name.clone())
            .collect()
    }
    
    /// 获取缓冲区状态信息
    ///
    /// 对应 PHP 的 ob_get_status() 函数
    ///
    /// # 参数
    /// - `full_status`: 是否获取完整状态
    ///
    /// # 返回
    /// 缓冲区状态信息
    pub fn get_status(&self, full_status: bool) -> BufferStatus {
        if full_status {
            // 返回所有缓冲区的状态
            let levels: Vec<BufferLevelInfo> = self.buffers
                .iter()
                .enumerate()
                .map(|(i, state)| BufferLevelInfo::from_state(i, state))
                .collect();
            
            BufferStatus::Full(levels)
        } else {
            // 只返回顶层缓冲区的状态
            if let Some(state) = self.buffers.front() {
                BufferStatus::Single(BufferLevelInfo::from_state(0, state))
            } else {
                BufferStatus::Empty
            }
        }
    }
    
    /// 向缓冲区写入内容
    ///
    /// # 参数
    /// - `content`: 要写入的内容
    ///
    /// # 返回
    /// 写入的字节数
    pub fn write(&mut self, content: &str) -> usize {
        // 如果没有缓冲区，直接输出
        if self.buffers.is_empty() {
            self.output.push_str(content);
            return content.len();
        }
        
        // 写入顶层缓冲区
        if let Some(state) = self.buffers.front_mut() {
            state.append(content);
            
            // 检查是否需要自动刷新
            if state.chunk_size > 0 && state.size >= state.chunk_size {
                // 自动刷新
                self.flush();
            }
            
            content.len()
        } else {
            0
        }
    }
    
    /// 获取已输出的内容
    pub fn get_output(&self) -> &str {
        &self.output
    }
    
    /// 清空已输出的内容
    pub fn clear_output(&mut self) {
        self.output.clear();
    }
    
    /// 刷新所有缓冲区
    ///
    /// 按从内到外的顺序刷新所有缓冲区
    pub fn flush_all(&mut self) {
        while !self.buffers.is_empty() {
            self.end_flush();
        }
    }
    
    /// 清空所有缓冲区
    ///
    /// 按从内到外的顺序清空所有缓冲区
    pub fn clean_all(&mut self) {
        self.buffers.clear();
    }
    
    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// 设置是否启用
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// 检查是否有活动的缓冲区
    pub fn has_active_buffer(&self) -> bool {
        !self.buffers.is_empty()
    }
    
    /// 获取顶层缓冲区
    pub fn get_top_buffer(&self) -> Option<&BufferState> {
        self.buffers.front()
    }
    
    /// 获取顶层缓冲区（可变）
    pub fn get_top_buffer_mut(&mut self) -> Option<&mut BufferState> {
        self.buffers.front_mut()
    }
}

impl Default for OutputBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// 缓冲区状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BufferStatus {
    /// 空状态（没有缓冲区）
    Empty,
    /// 单个缓冲区状态
    Single(BufferLevelInfo),
    /// 完整状态（所有缓冲区）
    Full(Vec<BufferLevelInfo>),
}

impl BufferStatus {
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        matches!(self, BufferStatus::Empty)
    }
    
    /// 获取缓冲区数量
    pub fn count(&self) -> usize {
        match self {
            BufferStatus::Empty => 0,
            BufferStatus::Single(_) => 1,
            BufferStatus::Full(levels) => levels.len(),
        }
    }
}

/// 全局输出缓冲管理器
pub struct GlobalOutputBuffer {
    /// 内部缓冲管理器
    inner: Arc<RwLock<OutputBuffer>>,
}

impl GlobalOutputBuffer {
    /// 创建新的全局管理器
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(OutputBuffer::new())),
        }
    }
    
    /// 获取内部管理器
    pub fn inner(&self) -> Arc<RwLock<OutputBuffer>> {
        self.inner.clone()
    }
    
    /// 启动缓冲
    pub fn start(&self, callback: Option<OutputCallback>, chunk_size: usize, flags: OutputHandlerFlags) -> bool {
        self.inner.write().start(callback, chunk_size, flags)
    }
    
    /// 获取内容
    pub fn get_contents(&self) -> Option<String> {
        self.inner.read().get_contents().map(|s| s.to_string())
    }
    
    /// 获取并清空
    pub fn get_clean(&self) -> Option<String> {
        self.inner.write().get_clean()
    }
    
    /// 刷新
    pub fn flush(&self) -> BufferResult {
        self.inner.write().flush()
    }
    
    /// 清空
    pub fn clean(&self) -> BufferResult {
        self.inner.write().clean()
    }
    
    /// 结束并刷新
    pub fn end_flush(&self) -> BufferResult {
        self.inner.write().end_flush()
    }
    
    /// 结束并清空
    pub fn end_clean(&self) -> BufferResult {
        self.inner.write().end_clean()
    }
    
    /// 获取层级
    pub fn get_level(&self) -> usize {
        self.inner.read().get_level()
    }
    
    /// 获取长度
    pub fn get_length(&self) -> Option<usize> {
        self.inner.read().get_length()
    }
    
    /// 写入内容
    pub fn write(&self, content: &str) -> usize {
        self.inner.write().write(content)
    }
}

impl Default for GlobalOutputBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for GlobalOutputBuffer {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// 全局输出缓冲实例
static GLOBAL_OUTPUT_BUFFER: once_cell::sync::Lazy<GlobalOutputBuffer> = 
    once_cell::sync::Lazy::new(GlobalOutputBuffer::new);

/// 获取全局输出缓冲管理器
pub fn global_output_buffer() -> &'static GlobalOutputBuffer {
    &GLOBAL_OUTPUT_BUFFER
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_output_buffer_start() {
        let mut buffer = OutputBuffer::new();
        
        // 启动缓冲
        let started = buffer.start(None, 0, OutputHandlerFlags::default());
        assert!(started);
        assert_eq!(buffer.get_level(), 1);
    }
    
    #[test]
    fn test_output_buffer_write() {
        let mut buffer = OutputBuffer::new();
        buffer.start(None, 0, OutputHandlerFlags::default());
        
        // 写入内容
        let written = buffer.write("Hello");
        assert_eq!(written, 5);
        
        // 检查内容
        assert_eq!(buffer.get_contents(), Some("Hello"));
    }
    
    #[test]
    fn test_output_buffer_flush() {
        let mut buffer = OutputBuffer::new();
        buffer.start(None, 0, OutputHandlerFlags::default());
        buffer.write("Test content");
        
        // 刷新
        let result = buffer.flush();
        assert!(result.is_success());
        
        // 缓冲区应该为空
        assert!(buffer.get_contents().unwrap().is_empty());
        
        // 输出应该有内容
        assert_eq!(buffer.get_output(), "Test content");
    }
    
    #[test]
    fn test_output_buffer_clean() {
        let mut buffer = OutputBuffer::new();
        buffer.start(None, 0, OutputHandlerFlags::default());
        buffer.write("Test content");
        
        // 清空
        let result = buffer.clean();
        assert!(result.is_success());
        
        // 缓冲区应该为空
        assert!(buffer.get_contents().unwrap().is_empty());
        
        // 输出不应该有内容
        assert!(buffer.get_output().is_empty());
    }
    
    #[test]
    fn test_output_buffer_end_flush() {
        let mut buffer = OutputBuffer::new();
        buffer.start(None, 0, OutputHandlerFlags::default());
        buffer.write("Content to flush");
        
        // 结束并刷新
        let result = buffer.end_flush();
        assert!(result.is_success());
        
        // 缓冲区层级应该为 0
        assert_eq!(buffer.get_level(), 0);
        
        // 输出应该有内容
        assert_eq!(buffer.get_output(), "Content to flush");
    }
    
    #[test]
    fn test_output_buffer_end_clean() {
        let mut buffer = OutputBuffer::new();
        buffer.start(None, 0, OutputHandlerFlags::default());
        buffer.write("Content to discard");
        
        // 结束并清空
        let result = buffer.end_clean();
        assert!(result.is_success());
        
        // 缓冲区层级应该为 0
        assert_eq!(buffer.get_level(), 0);
        
        // 输出不应该有内容
        assert!(buffer.get_output().is_empty());
    }
    
    #[test]
    fn test_output_buffer_nested() {
        let mut buffer = OutputBuffer::new();
        
        // 启动多层缓冲
        buffer.start(None, 0, OutputHandlerFlags::default());
        buffer.write("Level 1");
        
        buffer.start(None, 0, OutputHandlerFlags::default());
        buffer.write("Level 2");
        
        assert_eq!(buffer.get_level(), 2);
        
        // 结束内层
        buffer.end_flush();
        assert_eq!(buffer.get_level(), 1);
        
        // 检查内容
        assert_eq!(buffer.get_contents(), Some("Level 1Level 2"));
    }
    
    #[test]
    fn test_output_buffer_get_status() {
        let mut buffer = OutputBuffer::new();
        
        // 空状态
        let status = buffer.get_status(false);
        assert!(status.is_empty());
        
        // 启动缓冲
        buffer.start(None, 0, OutputHandlerFlags::default());
        
        // 单个状态
        let status = buffer.get_status(false);
        assert_eq!(status.count(), 1);
    }
    
    #[test]
    fn test_output_buffer_list_handlers() {
        let mut buffer = OutputBuffer::new();
        
        buffer.start(None, 0, OutputHandlerFlags::default());
        buffer.start(Some(OutputCallback::function("my_handler")), 0, OutputHandlerFlags::default());
        
        let handlers = buffer.list_handlers();
        assert_eq!(handlers.len(), 2);
    }
}
