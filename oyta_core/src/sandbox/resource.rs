//! 资源管理模块
//!
//! 管理沙箱的资源限制和使用情况

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 资源限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// 内存限制（字节）
    pub memory_bytes: u64,
    /// 时间限制（毫秒）
    pub time_ms: u64,
    /// CPU 限制（百分比 0-100）
    pub cpu_percent: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory_bytes: 32 * 1024 * 1024, // 32MB
            time_ms: 5000,                   // 5秒
            cpu_percent: 50,                 // 50%
        }
    }
}

/// 资源使用情况
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// 内存使用量（字节）
    pub memory_bytes: u64,
    /// 已执行时间（毫秒）
    pub time_ms: u64,
    /// CPU 使用率（百分比）
    pub cpu_percent: u32,
    /// 读取字节数
    pub bytes_read: u64,
    /// 写入字节数
    pub bytes_written: u64,
    /// 网络接收字节数
    pub network_in: u64,
    /// 网络发送字节数
    pub network_out: u64,
}

/// 资源管理器
/// 跟踪和控制沙箱的资源使用
#[derive(Debug)]
pub struct ResourceManager {
    /// 资源限制
    limits: ResourceLimits,
    /// 当前内存使用
    memory_used: AtomicU64,
    /// 开始时间
    start_time: AtomicU64,
    /// 是否正在运行
    running: AtomicBool,
    /// 读取字节数
    bytes_read: AtomicU64,
    /// 写入字节数
    bytes_written: AtomicU64,
    /// 网络接收字节数
    network_in: AtomicU64,
    /// 网络发送字节数
    network_out: AtomicU64,
}

impl ResourceManager {
    /// 创建新的资源管理器
    /// 
    /// # 参数
    /// - `limits`: 资源限制配置
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            limits,
            memory_used: AtomicU64::new(0),
            start_time: AtomicU64::new(0),
            running: AtomicBool::new(false),
            bytes_read: AtomicU64::new(0),
            bytes_written: AtomicU64::new(0),
            network_in: AtomicU64::new(0),
            network_out: AtomicU64::new(0),
        }
    }
    
    /// 开始资源监控
    pub fn start(&self) {
        // 记录开始时间
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        self.start_time.store(now, Ordering::SeqCst);
        self.running.store(true, Ordering::SeqCst);
    }
    
    /// 停止资源监控
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
    
    /// 获取当前资源使用情况
    pub fn current_usage(&self) -> ResourceUsage {
        // 计算已执行时间
        let time_ms = if self.running.load(Ordering::SeqCst) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            now.saturating_sub(self.start_time.load(Ordering::SeqCst))
        } else {
            0
        };
        
        ResourceUsage {
            memory_bytes: self.memory_used.load(Ordering::SeqCst),
            time_ms,
            cpu_percent: 0, // CPU 使用率需要更复杂的计算
            bytes_read: self.bytes_read.load(Ordering::SeqCst),
            bytes_written: self.bytes_written.load(Ordering::SeqCst),
            network_in: self.network_in.load(Ordering::SeqCst),
            network_out: self.network_out.load(Ordering::SeqCst),
        }
    }
    
    /// 记录内存分配
    /// 
    /// # 参数
    /// - `bytes`: 分配的字节数
    pub fn allocate_memory(&self, bytes: u64) -> bool {
        // 原子地增加内存使用
        let new_total = self.memory_used.fetch_add(bytes, Ordering::SeqCst) + bytes;
        
        // 检查是否超过限制
        if new_total > self.limits.memory_bytes {
            // 回滚
            self.memory_used.fetch_sub(bytes, Ordering::SeqCst);
            false
        } else {
            true
        }
    }
    
    /// 记录内存释放
    /// 
    /// # 参数
    /// - `bytes`: 释放的字节数
    pub fn free_memory(&self, bytes: u64) {
        // 原子地减少内存使用
        let prev = self.memory_used.fetch_sub(bytes, Ordering::SeqCst);
        // 防止下溢
        if prev < bytes {
            self.memory_used.store(0, Ordering::SeqCst);
        }
    }
    
    /// 记录文件读取
    /// 
    /// # 参数
    /// - `bytes`: 读取的字节数
    pub fn record_read(&self, bytes: u64) {
        self.bytes_read.fetch_add(bytes, Ordering::SeqCst);
    }
    
    /// 记录文件写入
    /// 
    /// # 参数
    /// - `bytes`: 写入的字节数
    pub fn record_write(&self, bytes: u64) {
        self.bytes_written.fetch_add(bytes, Ordering::SeqCst);
    }
    
    /// 记录网络接收
    /// 
    /// # 参数
    /// - `bytes`: 接收的字节数
    pub fn record_network_in(&self, bytes: u64) {
        self.network_in.fetch_add(bytes, Ordering::SeqCst);
    }
    
    /// 记录网络发送
    /// 
    /// # 参数
    /// - `bytes`: 发送的字节数
    pub fn record_network_out(&self, bytes: u64) {
        self.network_out.fetch_add(bytes, Ordering::SeqCst);
    }
    
    /// 检查是否超过内存限制
    pub fn is_memory_exceeded(&self) -> bool {
        self.memory_used.load(Ordering::SeqCst) > self.limits.memory_bytes
    }
    
    /// 检查是否超过时间限制
    pub fn is_time_exceeded(&self) -> bool {
        if !self.running.load(Ordering::SeqCst) {
            return false;
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        let elapsed = now.saturating_sub(self.start_time.load(Ordering::SeqCst));
        elapsed > self.limits.time_ms
    }
    
    /// 获取剩余时间（毫秒）
    pub fn remaining_time(&self) -> u64 {
        if !self.running.load(Ordering::SeqCst) {
            return self.limits.time_ms;
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        let elapsed = now.saturating_sub(self.start_time.load(Ordering::SeqCst));
        self.limits.time_ms.saturating_sub(elapsed)
    }
    
    /// 获取剩余内存（字节）
    pub fn remaining_memory(&self) -> u64 {
        let used = self.memory_used.load(Ordering::SeqCst);
        self.limits.memory_bytes.saturating_sub(used)
    }
    
    /// 获取资源限制
    pub fn limits(&self) -> &ResourceLimits {
        &self.limits
    }
    
    /// 更新资源限制
    pub fn update_limits(&mut self, limits: ResourceLimits) {
        self.limits = limits;
    }
    
    /// 重置资源使用统计
    pub fn reset(&self) {
        self.memory_used.store(0, Ordering::SeqCst);
        self.start_time.store(0, Ordering::SeqCst);
        self.running.store(false, Ordering::SeqCst);
        self.bytes_read.store(0, Ordering::SeqCst);
        self.bytes_written.store(0, Ordering::SeqCst);
        self.network_in.store(0, Ordering::SeqCst);
        self.network_out.store(0, Ordering::SeqCst);
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new(ResourceLimits::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试资源限制
    #[test]
    fn test_limits() {
        let limits = ResourceLimits {
            memory_bytes: 1024,
            time_ms: 100,
            cpu_percent: 50,
        };
        
        let manager = ResourceManager::new(limits);
        
        assert!(manager.allocate_memory(512));
        assert!(!manager.allocate_memory(1024)); // 超过限制
    }
    
    /// 测试时间限制
    #[test]
    fn test_time_limit() {
        let limits = ResourceLimits {
            memory_bytes: 1024 * 1024,
            time_ms: 100,
            cpu_percent: 50,
        };
        
        let manager = ResourceManager::new(limits);
        
        manager.start();
        assert!(!manager.is_time_exceeded());
        
        // 等待一段时间
        std::thread::sleep(Duration::from_millis(150));
        
        assert!(manager.is_time_exceeded());
    }
    
    /// 测试内存释放
    #[test]
    fn test_memory_free() {
        let limits = ResourceLimits {
            memory_bytes: 1024,
            time_ms: 1000,
            cpu_percent: 50,
        };
        
        let manager = ResourceManager::new(limits);
        
        manager.allocate_memory(512);
        assert_eq!(manager.current_usage().memory_bytes, 512);
        
        manager.free_memory(256);
        assert_eq!(manager.current_usage().memory_bytes, 256);
    }
    
    /// 测试重置
    #[test]
    fn test_reset() {
        let limits = ResourceLimits::default();
        let manager = ResourceManager::new(limits);
        
        manager.start();
        manager.allocate_memory(1024);
        manager.record_read(512);
        
        manager.reset();
        
        let usage = manager.current_usage();
        assert_eq!(usage.memory_bytes, 0);
        assert_eq!(usage.bytes_read, 0);
    }
}
