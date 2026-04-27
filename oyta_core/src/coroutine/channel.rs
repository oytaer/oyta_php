//! 协程通道模块
//!
//! 提供协程间的通信通道

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// 通道错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelError {
    /// 通道已关闭
    Closed,
    /// 通道已满
    Full,
    /// 通道为空
    Empty,
    /// 发送超时
    SendTimeout,
    /// 接收超时
    ReceiveTimeout,
    /// 缓冲区溢出
    BufferOverflow,
}

impl std::fmt::Display for ChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed => write!(f, "通道已关闭"),
            Self::Full => write!(f, "通道已满"),
            Self::Empty => write!(f, "通道为空"),
            Self::SendTimeout => write!(f, "发送超时"),
            Self::ReceiveTimeout => write!(f, "接收超时"),
            Self::BufferOverflow => write!(f, "缓冲区溢出"),
        }
    }
}

impl std::error::Error for ChannelError {}

/// 通道内部状态
struct ChannelInner<T> {
    /// 缓冲区
    buffer: VecDeque<T>,
    /// 是否已关闭
    closed: bool,
    /// 最大容量
    capacity: usize,
}

impl<T> std::fmt::Debug for ChannelInner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChannelInner")
            .field("buffer_len", &self.buffer.len())
            .field("closed", &self.closed)
            .field("capacity", &self.capacity)
            .finish()
    }
}

/// 协程通道
/// 用于协程间的消息传递
#[derive(Debug)]
pub struct CoroutineChannel<T> {
    /// 内部状态
    inner: Arc<(Mutex<ChannelInner<T>>, Condvar)>,
}

impl<T> CoroutineChannel<T> {
    /// 创建新的无缓冲通道
    pub fn unbounded() -> Self {
        Self {
            inner: Arc::new((
                Mutex::new(ChannelInner {
                    buffer: VecDeque::new(),
                    closed: false,
                    capacity: usize::MAX,
                }),
                Condvar::new(),
            )),
        }
    }
    
    /// 创建新的有缓冲通道
    /// 
    /// # 参数
    /// - `capacity`: 缓冲区容量
    pub fn bounded(capacity: usize) -> Self {
        Self {
            inner: Arc::new((
                Mutex::new(ChannelInner {
                    buffer: VecDeque::with_capacity(capacity),
                    closed: false,
                    capacity,
                }),
                Condvar::new(),
            )),
        }
    }
    
    /// 发送消息
    /// 
    /// # 参数
    /// - `value`: 要发送的值
    pub fn send(&self, value: T) -> Result<(), ChannelError> {
        // 获取锁
        let (lock, cvar) = &*self.inner;
        let mut inner = lock.lock().unwrap();
        
        // 检查是否已关闭
        if inner.closed {
            return Err(ChannelError::Closed);
        }
        
        // 检查缓冲区是否已满
        if inner.buffer.len() >= inner.capacity {
            return Err(ChannelError::Full);
        }
        
        // 添加到缓冲区
        inner.buffer.push_back(value);
        
        // 通知等待的接收者
        cvar.notify_one();
        
        Ok(())
    }
    
    /// 发送消息（阻塞直到有空间）
    /// 
    /// # 参数
    /// - `value`: 要发送的值
    pub fn send_blocking(&self, value: T) -> Result<(), ChannelError> {
        // 获取锁
        let (lock, cvar) = &*self.inner;
        let mut inner = lock.lock().unwrap();
        
        // 等待缓冲区有空间
        while inner.buffer.len() >= inner.capacity && !inner.closed {
            inner = cvar.wait(inner).unwrap();
        }
        
        // 检查是否已关闭
        if inner.closed {
            return Err(ChannelError::Closed);
        }
        
        // 添加到缓冲区
        inner.buffer.push_back(value);
        
        // 通知等待的接收者
        cvar.notify_one();
        
        Ok(())
    }
    
    /// 发送消息（带超时）
    /// 
    /// # 参数
    /// - `value`: 要发送的值
    /// - `timeout`: 超时时间
    pub fn send_timeout(&self, value: T, timeout: Duration) -> Result<(), ChannelError> {
        // 获取锁
        let (lock, cvar) = &*self.inner;
        let mut inner = lock.lock().unwrap();
        
        // 计算截止时间
        let deadline = std::time::Instant::now() + timeout;
        
        // 等待缓冲区有空间
        while inner.buffer.len() >= inner.capacity && !inner.closed {
            let now = std::time::Instant::now();
            if now >= deadline {
                return Err(ChannelError::SendTimeout);
            }
            
            let remaining = deadline - now;
            let result = cvar.wait_timeout(inner, remaining).unwrap();
            inner = result.0;
        }
        
        // 检查是否已关闭
        if inner.closed {
            return Err(ChannelError::Closed);
        }
        
        // 添加到缓冲区
        inner.buffer.push_back(value);
        
        // 通知等待的接收者
        cvar.notify_one();
        
        Ok(())
    }
    
    /// 接收消息
    pub fn recv(&self) -> Result<T, ChannelError> {
        // 获取锁
        let (lock, cvar) = &*self.inner;
        let mut inner = lock.lock().unwrap();
        
        // 检查缓冲区是否为空
        if inner.buffer.is_empty() {
            if inner.closed {
                return Err(ChannelError::Closed);
            }
            return Err(ChannelError::Empty);
        }
        
        // 从缓冲区取出
        let value = inner.buffer.pop_front().unwrap();
        
        // 通知等待的发送者
        cvar.notify_one();
        
        Ok(value)
    }
    
    /// 接收消息（阻塞直到有消息）
    pub fn recv_blocking(&self) -> Result<T, ChannelError> {
        // 获取锁
        let (lock, cvar) = &*self.inner;
        let mut inner = lock.lock().unwrap();
        
        // 等待缓冲区有消息
        while inner.buffer.is_empty() && !inner.closed {
            inner = cvar.wait(inner).unwrap();
        }
        
        // 检查是否已关闭且为空
        if inner.buffer.is_empty() && inner.closed {
            return Err(ChannelError::Closed);
        }
        
        // 从缓冲区取出
        let value = inner.buffer.pop_front().unwrap();
        
        // 通知等待的发送者
        cvar.notify_one();
        
        Ok(value)
    }
    
    /// 接收消息（带超时）
    /// 
    /// # 参数
    /// - `timeout`: 超时时间
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, ChannelError> {
        // 获取锁
        let (lock, cvar) = &*self.inner;
        let mut inner = lock.lock().unwrap();
        
        // 计算截止时间
        let deadline = std::time::Instant::now() + timeout;
        
        // 等待缓冲区有消息
        while inner.buffer.is_empty() && !inner.closed {
            let now = std::time::Instant::now();
            if now >= deadline {
                return Err(ChannelError::ReceiveTimeout);
            }
            
            let remaining = deadline - now;
            let result = cvar.wait_timeout(inner, remaining).unwrap();
            inner = result.0;
        }
        
        // 检查是否已关闭且为空
        if inner.buffer.is_empty() && inner.closed {
            return Err(ChannelError::Closed);
        }
        
        // 从缓冲区取出
        let value = inner.buffer.pop_front().unwrap();
        
        // 通知等待的发送者
        cvar.notify_one();
        
        Ok(value)
    }
    
    /// 尝试接收消息
    pub fn try_recv(&self) -> Result<T, ChannelError> {
        self.recv()
    }
    
    /// 关闭通道
    pub fn close(&self) {
        // 获取锁
        let (lock, cvar) = &*self.inner;
        let mut inner = lock.lock().unwrap();
        
        // 设置关闭标志
        inner.closed = true;
        
        // 通知所有等待者
        cvar.notify_all();
    }
    
    /// 检查是否已关闭
    pub fn is_closed(&self) -> bool {
        let (lock, _) = &*self.inner;
        let inner = lock.lock().unwrap();
        inner.closed
    }
    
    /// 获取缓冲区大小
    pub fn len(&self) -> usize {
        let (lock, _) = &*self.inner;
        let inner = lock.lock().unwrap();
        inner.buffer.len()
    }
    
    /// 检查缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// 获取缓冲区容量
    pub fn capacity(&self) -> usize {
        let (lock, _) = &*self.inner;
        let inner = lock.lock().unwrap();
        inner.capacity
    }
}

impl<T> Clone for CoroutineChannel<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试无缓冲通道
    #[test]
    fn test_unbounded_channel() {
        let channel = CoroutineChannel::<i32>::unbounded();
        
        // 发送消息
        assert!(channel.send(1).is_ok());
        assert!(channel.send(2).is_ok());
        assert!(channel.send(3).is_ok());
        
        // 接收消息
        assert_eq!(channel.recv().unwrap(), 1);
        assert_eq!(channel.recv().unwrap(), 2);
        assert_eq!(channel.recv().unwrap(), 3);
    }
    
    /// 测试有缓冲通道
    #[test]
    fn test_bounded_channel() {
        let channel = CoroutineChannel::<i32>::bounded(2);
        
        // 发送消息
        assert!(channel.send(1).is_ok());
        assert!(channel.send(2).is_ok());
        
        // 缓冲区已满
        assert!(matches!(channel.send(3), Err(ChannelError::Full)));
        
        // 接收消息
        assert_eq!(channel.recv().unwrap(), 1);
        
        // 现在可以发送了
        assert!(channel.send(3).is_ok());
    }
    
    /// 测试通道关闭
    #[test]
    fn test_close() {
        let channel = CoroutineChannel::<i32>::bounded(1);
        
        channel.close();
        
        // 发送应该失败
        assert!(matches!(channel.send(1), Err(ChannelError::Closed)));
        
        // 检查关闭状态
        assert!(channel.is_closed());
    }
    
    /// 测试通道长度
    #[test]
    fn test_len() {
        let channel = CoroutineChannel::<i32>::bounded(10);
        
        assert!(channel.is_empty());
        
        channel.send(1).unwrap();
        channel.send(2).unwrap();
        
        assert_eq!(channel.len(), 2);
    }
}
