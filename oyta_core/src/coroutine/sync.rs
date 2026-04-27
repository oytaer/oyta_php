//! 协程同步原语模块
//!
//! 提供协程间的同步机制，包括互斥锁、读写锁、信号量

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// 协程互斥锁
/// 支持异步等待的互斥锁
#[derive(Debug)]
pub struct CoroutineMutex {
    /// 内部状态
    inner: Arc<(Mutex<MutexState>, Condvar)>,
}

/// 互斥锁状态
#[derive(Debug)]
struct MutexState {
    /// 是否已锁定
    locked: bool,
    /// 等待队列
    waiters: VecDeque<u64>,
}

impl CoroutineMutex {
    /// 创建新的互斥锁
    pub fn new() -> Self {
        Self {
            inner: Arc::new((
                Mutex::new(MutexState {
                    locked: false,
                    waiters: VecDeque::new(),
                }),
                Condvar::new(),
            )),
        }
    }
    
    /// 尝试获取锁
    /// 
    /// # 返回
    /// 成功返回 true，失败返回 false
    pub fn try_lock(&self) -> bool {
        let (lock, _) = &*self.inner;
        let mut state = lock.lock().unwrap();
        
        if state.locked {
            return false;
        }
        
        state.locked = true;
        true
    }
    
    /// 获取锁（阻塞）
    pub fn lock(&self) {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        
        // 等待锁释放
        while state.locked {
            state = cvar.wait(state).unwrap();
        }
        
        state.locked = true;
    }
    
    /// 获取锁（带超时）
    /// 
    /// # 参数
    /// - `timeout`: 超时时间
    pub fn lock_timeout(&self, timeout: Duration) -> bool {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        
        // 计算截止时间
        let deadline = std::time::Instant::now() + timeout;
        
        // 等待锁释放
        while state.locked {
            let now = std::time::Instant::now();
            if now >= deadline {
                return false;
            }
            
            let remaining = deadline - now;
            let result = cvar.wait_timeout(state, remaining).unwrap();
            state = result.0;
        }
        
        state.locked = true;
        true
    }
    
    /// 释放锁
    pub fn unlock(&self) {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        
        state.locked = false;
        
        // 通知等待者
        cvar.notify_one();
    }
    
    /// 检查是否已锁定
    pub fn is_locked(&self) -> bool {
        let (lock, _) = &*self.inner;
        let state = lock.lock().unwrap();
        state.locked
    }
}

impl Default for CoroutineMutex {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CoroutineMutex {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// 协程读写锁
/// 支持并发读取和独占写入
#[derive(Debug)]
pub struct CoroutineRwLock {
    /// 内部状态
    inner: Arc<(Mutex<RwLockState>, Condvar)>,
}

/// 读写锁状态
#[derive(Debug)]
struct RwLockState {
    /// 读取者数量
    readers: u32,
    /// 是否有写入者
    writer: bool,
    /// 等待写入的数量
    waiting_writers: u32,
}

impl CoroutineRwLock {
    /// 创建新的读写锁
    pub fn new() -> Self {
        Self {
            inner: Arc::new((
                Mutex::new(RwLockState {
                    readers: 0,
                    writer: false,
                    waiting_writers: 0,
                }),
                Condvar::new(),
            )),
        }
    }
    
    /// 尝试获取读锁
    pub fn try_read(&self) -> bool {
        let (lock, _) = &*self.inner;
        let mut state = lock.lock().unwrap();
        
        // 如果有写入者或等待的写入者，不能获取读锁
        if state.writer || state.waiting_writers > 0 {
            return false;
        }
        
        state.readers += 1;
        true
    }
    
    /// 获取读锁（阻塞）
    pub fn read(&self) {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        
        // 等待写入者释放
        while state.writer || state.waiting_writers > 0 {
            state = cvar.wait(state).unwrap();
        }
        
        state.readers += 1;
    }
    
    /// 释放读锁
    pub fn read_unlock(&self) {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        
        state.readers = state.readers.saturating_sub(1);
        
        // 如果没有读取者了，通知等待的写入者
        if state.readers == 0 {
            cvar.notify_all();
        }
    }
    
    /// 尝试获取写锁
    pub fn try_write(&self) -> bool {
        let (lock, _) = &*self.inner;
        let mut state = lock.lock().unwrap();
        
        // 如果有读取者或写入者，不能获取写锁
        if state.readers > 0 || state.writer {
            return false;
        }
        
        state.writer = true;
        true
    }
    
    /// 获取写锁（阻塞）
    pub fn write(&self) {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        
        // 标记等待写入
        state.waiting_writers += 1;
        
        // 等待读取者和写入者释放
        while state.readers > 0 || state.writer {
            state = cvar.wait(state).unwrap();
        }
        
        state.waiting_writers -= 1;
        state.writer = true;
    }
    
    /// 释放写锁
    pub fn write_unlock(&self) {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        
        state.writer = false;
        
        // 通知所有等待者
        cvar.notify_all();
    }
    
    /// 检查是否有读取者
    pub fn has_readers(&self) -> bool {
        let (lock, _) = &*self.inner;
        let state = lock.lock().unwrap();
        state.readers > 0
    }
    
    /// 检查是否有写入者
    pub fn has_writer(&self) -> bool {
        let (lock, _) = &*self.inner;
        let state = lock.lock().unwrap();
        state.writer
    }
}

impl Default for CoroutineRwLock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CoroutineRwLock {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// 协程信号量
/// 用于控制并发访问数量
#[derive(Debug)]
pub struct CoroutineSemaphore {
    /// 内部状态
    inner: Arc<(Mutex<SemaphoreState>, Condvar)>,
}

/// 信号量状态
#[derive(Debug)]
struct SemaphoreState {
    /// 当前可用许可数
    permits: u32,
    /// 最大许可数
    max_permits: u32,
}

impl CoroutineSemaphore {
    /// 创建新的信号量
    /// 
    /// # 参数
    /// - `permits`: 初始许可数
    pub fn new(permits: u32) -> Self {
        Self {
            inner: Arc::new((
                Mutex::new(SemaphoreState {
                    permits,
                    max_permits: permits,
                }),
                Condvar::new(),
            )),
        }
    }
    
    /// 尝试获取许可
    /// 
    /// # 参数
    /// - `n`: 要获取的许可数
    pub fn try_acquire(&self, n: u32) -> bool {
        let (lock, _) = &*self.inner;
        let mut state = lock.lock().unwrap();
        
        if state.permits >= n {
            state.permits -= n;
            return true;
        }
        
        false
    }
    
    /// 获取许可（阻塞）
    /// 
    /// # 参数
    /// - `n`: 要获取的许可数
    pub fn acquire(&self, n: u32) {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        
        // 等待足够的许可
        while state.permits < n {
            state = cvar.wait(state).unwrap();
        }
        
        state.permits -= n;
    }
    
    /// 获取许可（带超时）
    /// 
    /// # 参数
    /// - `n`: 要获取的许可数
    /// - `timeout`: 超时时间
    pub fn acquire_timeout(&self, n: u32, timeout: Duration) -> bool {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        
        // 计算截止时间
        let deadline = std::time::Instant::now() + timeout;
        
        // 等待足够的许可
        while state.permits < n {
            let now = std::time::Instant::now();
            if now >= deadline {
                return false;
            }
            
            let remaining = deadline - now;
            let result = cvar.wait_timeout(state, remaining).unwrap();
            state = result.0;
        }
        
        state.permits -= n;
        true
    }
    
    /// 释放许可
    /// 
    /// # 参数
    /// - `n`: 要释放的许可数
    pub fn release(&self, n: u32) {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        
        // 增加许可数（不超过最大值）
        state.permits = (state.permits + n).min(state.max_permits);
        
        // 通知等待者
        cvar.notify_all();
    }
    
    /// 获取当前可用许可数
    pub fn available_permits(&self) -> u32 {
        let (lock, _) = &*self.inner;
        let state = lock.lock().unwrap();
        state.permits
    }
    
    /// 获取最大许可数
    pub fn max_permits(&self) -> u32 {
        let (lock, _) = &*self.inner;
        let state = lock.lock().unwrap();
        state.max_permits
    }
}

impl Clone for CoroutineSemaphore {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试互斥锁
    #[test]
    fn test_mutex() {
        let mutex = CoroutineMutex::new();
        
        // 初始未锁定
        assert!(!mutex.is_locked());
        
        // 尝试获取锁
        assert!(mutex.try_lock());
        assert!(mutex.is_locked());
        
        // 再次尝试获取失败
        assert!(!mutex.try_lock());
        
        // 释放锁
        mutex.unlock();
        assert!(!mutex.is_locked());
    }
    
    /// 测试读写锁
    #[test]
    fn test_rwlock() {
        let rwlock = CoroutineRwLock::new();
        
        // 获取读锁
        assert!(rwlock.try_read());
        assert!(rwlock.has_readers());
        
        // 可以再次获取读锁
        assert!(rwlock.try_read());
        
        // 不能获取写锁
        assert!(!rwlock.try_write());
        
        // 释放读锁
        rwlock.read_unlock();
        rwlock.read_unlock();
        
        // 现在可以获取写锁
        assert!(rwlock.try_write());
        assert!(rwlock.has_writer());
    }
    
    /// 测试信号量
    #[test]
    fn test_semaphore() {
        let sem = CoroutineSemaphore::new(3);
        
        // 初始有 3 个许可
        assert_eq!(sem.available_permits(), 3);
        
        // 获取 2 个许可
        assert!(sem.try_acquire(2));
        assert_eq!(sem.available_permits(), 1);
        
        // 尝试获取 2 个失败
        assert!(!sem.try_acquire(2));
        
        // 释放 1 个许可
        sem.release(1);
        assert_eq!(sem.available_permits(), 2);
    }
    
    /// 测试互斥锁超时
    #[test]
    fn test_mutex_timeout() {
        let mutex = CoroutineMutex::new();
        
        // 获取锁
        mutex.lock();
        
        // 尝试带超时获取（应该超时）
        let result = mutex.lock_timeout(Duration::from_millis(50));
        assert!(!result);
        
        // 释放锁
        mutex.unlock();
        
        // 现在应该可以获取
        let result = mutex.lock_timeout(Duration::from_millis(50));
        assert!(result);
    }
}
