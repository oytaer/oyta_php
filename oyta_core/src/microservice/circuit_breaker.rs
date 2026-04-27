//! 熔断器模块
//! 
//! 实现熔断器模式，防止级联故障
//! 支持三种状态：关闭、打开、半开
//! 提供自动恢复和手动控制功能

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 熔断器状态枚举
/// 定义熔断器的三种状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// 关闭状态（正常）
    /// 允许所有请求通过
    Closed,
    /// 打开状态（熔断）
    /// 拒绝所有请求
    Open,
    /// 半开状态（试探）
    /// 允许部分请求通过以测试服务是否恢复
    HalfOpen,
}

/// 为 CircuitState 实现默认值
impl Default for CircuitState {
    fn default() -> Self {
        // 默认为关闭状态
        CircuitState::Closed
    }
}

/// 熔断器配置
/// 定义熔断器的行为参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// 失败阈值（触发熔断的失败次数）
    pub failure_threshold: u32,
    /// 成功阈值（半开状态下恢复关闭的成功次数）
    pub success_threshold: u32,
    /// 超时时间（毫秒，熔断后等待多久进入半开状态）
    pub timeout_ms: u64,
    /// 时间窗口大小（毫秒，用于统计失败率）
    pub window_size_ms: u64,
    /// 失败率阈值（百分比，超过此阈值触发熔断）
    pub failure_rate_threshold: f64,
    /// 最小请求数（在时间窗口内的最小请求数，用于计算失败率）
    pub minimum_requests: u32,
    /// 半开状态允许的最大请求数
    pub half_open_max_requests: u32,
    /// 是否启用自动恢复
    pub auto_recovery: bool,
    /// 自动恢复间隔（毫秒）
    pub auto_recovery_interval_ms: u64,
}

/// 为 CircuitBreakerConfig 实现默认值
impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            // 默认失败5次触发熔断
            failure_threshold: 5,
            // 默认成功3次恢复
            success_threshold: 3,
            // 默认熔断30秒后进入半开状态
            timeout_ms: 30_000,
            // 默认时间窗口60秒
            window_size_ms: 60_000,
            // 默认失败率50%触发熔断
            failure_rate_threshold: 50.0,
            // 默认最小请求数10
            minimum_requests: 10,
            // 默认半开状态最多允许5个请求
            half_open_max_requests: 5,
            // 默认启用自动恢复
            auto_recovery: true,
            // 默认自动恢复间隔60秒
            auto_recovery_interval_ms: 60_000,
        }
    }
}

/// 熔断器统计信息
/// 记录熔断器的运行时数据
#[derive(Debug, Default)]
pub struct CircuitStats {
    /// 总请求数
    pub total_requests: AtomicU64,
    /// 成功请求数
    pub successful_requests: AtomicU64,
    /// 失败请求数
    pub failed_requests: AtomicU64,
    /// 被拒绝的请求数（熔断期间）
    pub rejected_requests: AtomicU64,
    /// 超时请求数
    pub timeout_requests: AtomicU64,
    /// 当前连续失败次数
    pub consecutive_failures: AtomicU32,
    /// 当前连续成功次数
    pub consecutive_successes: AtomicU32,
    /// 熔断次数
    pub trip_count: AtomicU64,
    /// 最后一次失败时间
    pub last_failure_time: AtomicU64,
    /// 最后一次成功时间
    pub last_success_time: AtomicU64,
    /// 最后一次状态变更时间
    pub last_state_change_time: AtomicU64,
    /// 半开状态下的请求数
    pub half_open_requests: AtomicU32,
}

/// 请求记录
/// 用于时间窗口内的统计
#[derive(Debug, Clone)]
struct RequestRecord {
    /// 请求时间
    timestamp: u64,
    /// 是否成功
    success: bool,
}

/// 熔断器
/// 实现熔断器模式的核心结构
pub struct CircuitBreaker {
    /// 熔断器名称
    name: String,
    /// 当前状态
    state: RwLock<CircuitState>,
    /// 配置
    config: CircuitBreakerConfig,
    /// 统计信息
    stats: CircuitStats,
    /// 请求记录（用于时间窗口统计）
    request_records: RwLock<Vec<RequestRecord>>,
    /// 是否启用
    enabled: AtomicBool,
    /// 最后一次检查时间
    last_check_time: AtomicU64,
}

/// 为 CircuitBreaker 实现相关方法
impl CircuitBreaker {
    /// 创建新的熔断器
    /// 
    /// # 参数
    /// - `name`: 熔断器名称
    /// - `config`: 熔断器配置
    /// 
    /// # 返回
    /// 新的熔断器实例
    pub fn new(name: &str, config: CircuitBreakerConfig) -> Self {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        Self {
            // 存储名称
            name: name.to_string(),
            // 初始状态为关闭
            state: RwLock::new(CircuitState::Closed),
            // 存储配置
            config,
            // 初始化统计
            stats: CircuitStats::default(),
            // 初始化请求记录
            request_records: RwLock::new(Vec::new()),
            // 默认启用
            enabled: AtomicBool::new(true),
            // 初始化检查时间
            last_check_time: AtomicU64::new(now),
        }
    }
    
    /// 使用默认配置创建熔断器
    /// 
    /// # 参数
    /// - `name`: 熔断器名称
    /// 
    /// # 返回
    /// 使用默认配置的熔断器实例
    pub fn with_defaults(name: &str) -> Self {
        Self::new(name, CircuitBreakerConfig::default())
    }
    
    /// 检查是否允许请求通过
    /// 
    /// # 返回
    /// 允许返回true，拒绝返回false
    pub fn allow_request(&self) -> bool {
        // 如果未启用，总是允许
        if !self.enabled.load(Ordering::Relaxed) {
            return true;
        }
        
        // 获取当前状态
        let state = self.get_state();
        
        match state {
            // 关闭状态：允许所有请求
            CircuitState::Closed => true,
            // 打开状态：拒绝所有请求
            CircuitState::Open => {
                // 检查是否应该进入半开状态
                if self.should_attempt_reset() {
                    self.transition_to_half_open();
                    return true;
                }
                // 更新拒绝统计
                self.stats.rejected_requests.fetch_add(1, Ordering::Relaxed);
                false
            }
            // 半开状态：允许有限数量的请求
            CircuitState::HalfOpen => {
                // 检查半开状态下的请求数限制
                let current = self.stats.half_open_requests.load(Ordering::Relaxed);
                if current < self.config.half_open_max_requests {
                    self.stats.half_open_requests.fetch_add(1, Ordering::Relaxed);
                    true
                } else {
                    self.stats.rejected_requests.fetch_add(1, Ordering::Relaxed);
                    false
                }
            }
        }
    }
    
    /// 记录成功请求
    pub fn record_success(&self) {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 更新统计
        self.stats.total_requests.fetch_add(1, Ordering::Relaxed);
        self.stats.successful_requests.fetch_add(1, Ordering::Relaxed);
        self.stats.consecutive_failures.store(0, Ordering::Relaxed);
        self.stats.consecutive_successes.fetch_add(1, Ordering::Relaxed);
        self.stats.last_success_time.store(now, Ordering::Relaxed);
        
        // 记录请求
        self.record_request(true);
        
        // 根据状态处理
        let state = self.get_state();
        match state {
            // 半开状态下，检查是否应该恢复
            CircuitState::HalfOpen => {
                let successes = self.stats.consecutive_successes.load(Ordering::Relaxed);
                if successes >= self.config.success_threshold {
                    self.transition_to_closed();
                }
            }
            _ => {}
        }
    }
    
    /// 记录失败请求
    pub fn record_failure(&self) {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 更新统计
        self.stats.total_requests.fetch_add(1, Ordering::Relaxed);
        self.stats.failed_requests.fetch_add(1, Ordering::Relaxed);
        self.stats.consecutive_successes.store(0, Ordering::Relaxed);
        self.stats.consecutive_failures.fetch_add(1, Ordering::Relaxed);
        self.stats.last_failure_time.store(now, Ordering::Relaxed);
        
        // 记录请求
        self.record_request(false);
        
        // 根据状态处理
        let state = self.get_state();
        match state {
            // 关闭状态下，检查是否应该熔断
            CircuitState::Closed => {
                if self.should_trip() {
                    self.transition_to_open();
                }
            }
            // 半开状态下，失败立即熔断
            CircuitState::HalfOpen => {
                self.transition_to_open();
            }
            _ => {}
        }
    }
    
    /// 记录超时请求
    pub fn record_timeout(&self) {
        // 更新统计
        self.stats.timeout_requests.fetch_add(1, Ordering::Relaxed);
        // 超时也算作失败
        self.record_failure();
    }
    
    /// 记录请求到时间窗口
    fn record_request(&self, success: bool) {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 获取写锁
        let mut records = self.request_records.write().unwrap();
        
        // 添加新记录
        records.push(RequestRecord {
            timestamp: now,
            success,
        });
        
        // 清理过期记录
        let window_start = now.saturating_sub(self.config.window_size_ms);
        records.retain(|r| r.timestamp >= window_start);
    }
    
    /// 检查是否应该触发熔断
    fn should_trip(&self) -> bool {
        // 检查连续失败次数
        let consecutive_failures = self.stats.consecutive_failures.load(Ordering::Relaxed);
        if consecutive_failures >= self.config.failure_threshold {
            return true;
        }
        
        // 检查失败率
        let records = self.request_records.read().unwrap();
        if records.len() < self.config.minimum_requests as usize {
            return false;
        }
        
        // 计算失败率
        let failures = records.iter().filter(|r| !r.success).count();
        let failure_rate = (failures as f64 / records.len() as f64) * 100.0;
        
        failure_rate >= self.config.failure_rate_threshold
    }
    
    /// 检查是否应该尝试重置（进入半开状态）
    fn should_attempt_reset(&self) -> bool {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 检查是否超过超时时间
        let last_state_change = self.stats.last_state_change_time.load(Ordering::Relaxed);
        now.saturating_sub(last_state_change) >= self.config.timeout_ms
    }
    
    /// 转换到打开状态
    fn transition_to_open(&self) {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 更新状态
        {
            let mut state = self.state.write().unwrap();
            *state = CircuitState::Open;
        }
        
        // 更新统计
        self.stats.trip_count.fetch_add(1, Ordering::Relaxed);
        self.stats.last_state_change_time.store(now, Ordering::Relaxed);
        self.stats.half_open_requests.store(0, Ordering::Relaxed);
    }
    
    /// 转换到关闭状态
    fn transition_to_closed(&self) {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 更新状态
        {
            let mut state = self.state.write().unwrap();
            *state = CircuitState::Closed;
        }
        
        // 重置统计
        self.stats.consecutive_failures.store(0, Ordering::Relaxed);
        self.stats.consecutive_successes.store(0, Ordering::Relaxed);
        self.stats.last_state_change_time.store(now, Ordering::Relaxed);
        self.stats.half_open_requests.store(0, Ordering::Relaxed);
        
        // 清空请求记录
        {
            let mut records = self.request_records.write().unwrap();
            records.clear();
        }
    }
    
    /// 转换到半开状态
    fn transition_to_half_open(&self) {
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64;
        
        // 更新状态
        {
            let mut state = self.state.write().unwrap();
            *state = CircuitState::HalfOpen;
        }
        
        // 重置半开计数
        self.stats.consecutive_successes.store(0, Ordering::Relaxed);
        self.stats.last_state_change_time.store(now, Ordering::Relaxed);
        self.stats.half_open_requests.store(0, Ordering::Relaxed);
    }
    
    /// 获取当前状态
    /// 
    /// # 返回
    /// 当前熔断器状态
    pub fn get_state(&self) -> CircuitState {
        *self.state.read().unwrap()
    }
    
    /// 手动打开熔断器
    pub fn trip(&self) {
        self.transition_to_open();
    }
    
    /// 手动关闭熔断器
    pub fn reset(&self) {
        self.transition_to_closed();
    }
    
    /// 获取名称
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// 获取统计信息
    pub fn stats(&self) -> &CircuitStats {
        &self.stats
    }
    
    /// 启用熔断器
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }
    
    /// 禁用熔断器
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }
    
    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
    
    /// 获取失败率
    /// 
    /// # 返回
    /// 失败率（百分比）
    pub fn failure_rate(&self) -> f64 {
        let records = self.request_records.read().unwrap();
        if records.is_empty() {
            return 0.0;
        }
        
        let failures = records.iter().filter(|r| !r.success).count();
        (failures as f64 / records.len() as f64) * 100.0
    }
    
    /// 获取时间窗口内的请求数
    pub fn window_request_count(&self) -> usize {
        self.request_records.read().unwrap().len()
    }
}

/// 熔断器管理器
/// 管理多个熔断器实例
pub struct CircuitBreakerManager {
    /// 熔断器映射
    breakers: RwLock<HashMap<String, Arc<CircuitBreaker>>>,
    /// 默认配置
    default_config: CircuitBreakerConfig,
}

/// 为 CircuitBreakerManager 实现相关方法
impl CircuitBreakerManager {
    /// 创建新的熔断器管理器
    pub fn new() -> Self {
        Self {
            breakers: RwLock::new(HashMap::new()),
            default_config: CircuitBreakerConfig::default(),
        }
    }
    
    /// 设置默认配置
    pub fn set_default_config(&mut self, config: CircuitBreakerConfig) {
        self.default_config = config;
    }
    
    /// 获取或创建熔断器
    /// 
    /// # 参数
    /// - `name`: 熔断器名称
    /// 
    /// # 返回
    /// 熔断器实例
    pub fn get_breaker(&self, name: &str) -> Arc<CircuitBreaker> {
        // 先尝试获取现有的
        {
            let breakers = self.breakers.read().unwrap();
            if let Some(breaker) = breakers.get(name) {
                return breaker.clone();
            }
        }
        
        // 创建新的熔断器
        let breaker = Arc::new(CircuitBreaker::new(name, self.default_config.clone()));
        
        // 存储并返回
        {
            let mut breakers = self.breakers.write().unwrap();
            breakers.entry(name.to_string())
                .or_insert_with(|| breaker.clone());
        }
        
        breaker
    }
    
    /// 移除熔断器
    /// 
    /// # 参数
    /// - `name`: 熔断器名称
    /// 
    /// # 返回
    /// 是否成功移除
    pub fn remove_breaker(&self, name: &str) -> bool {
        let mut breakers = self.breakers.write().unwrap();
        breakers.remove(name).is_some()
    }
    
    /// 列出所有熔断器名称
    pub fn list_breakers(&self) -> Vec<String> {
        self.breakers.read().unwrap().keys().cloned().collect()
    }
    
    /// 重置所有熔断器
    pub fn reset_all(&self) {
        let breakers = self.breakers.read().unwrap();
        for breaker in breakers.values() {
            breaker.reset();
        }
    }
}

/// 为 CircuitBreakerManager 实现 Default trait
impl Default for CircuitBreakerManager {
    fn default() -> Self {
        Self::new()
    }
}

use std::collections::HashMap;

/// 单元测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试熔断器创建
    #[test]
    fn test_create() {
        let breaker = CircuitBreaker::with_defaults("test");
        
        assert_eq!(breaker.name(), "test");
        assert_eq!(breaker.get_state(), CircuitState::Closed);
        assert!(breaker.is_enabled());
    }
    
    /// 测试允许请求
    #[test]
    fn test_allow_request() {
        let breaker = CircuitBreaker::with_defaults("test");
        
        // 关闭状态应该允许
        assert!(breaker.allow_request());
    }
    
    /// 测试熔断触发
    #[test]
    fn test_trip() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let breaker = CircuitBreaker::new("test", config);
        
        // 记录失败
        for _ in 0..3 {
            breaker.record_failure();
        }
        
        // 应该进入打开状态
        assert_eq!(breaker.get_state(), CircuitState::Open);
        
        // 应该拒绝请求
        assert!(!breaker.allow_request());
    }
    
    /// 测试手动重置
    #[test]
    fn test_manual_reset() {
        let breaker = CircuitBreaker::with_defaults("test");
        
        // 手动打开
        breaker.trip();
        assert_eq!(breaker.get_state(), CircuitState::Open);
        
        // 手动重置
        breaker.reset();
        assert_eq!(breaker.get_state(), CircuitState::Closed);
    }
    
    /// 测试熔断器管理器
    #[test]
    fn test_manager() {
        let manager = CircuitBreakerManager::new();
        
        let breaker1 = manager.get_breaker("service1");
        let breaker2 = manager.get_breaker("service2");
        
        assert_eq!(breaker1.name(), "service1");
        assert_eq!(breaker2.name(), "service2");
        
        let names = manager.list_breakers();
        assert_eq!(names.len(), 2);
    }
}
