//! 断线重连模块
//!
//! 提供数据库断线重连功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库

use anyhow::Result;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// 重连配置
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// 是否启用断线重连
    pub enabled: bool,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
    /// 重试间隔倍数
    pub retry_multiplier: f64,
    /// 最大重试间隔（毫秒）
    pub max_retry_interval_ms: u64,
    /// 连接超时（秒）
    pub connection_timeout_secs: u64,
    /// 断线标识字符串
    pub break_match_strings: Vec<String>,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_retries: 3,
            retry_interval_ms: 100,
            retry_multiplier: 2.0,
            max_retry_interval_ms: 5000,
            connection_timeout_secs: 30,
            break_match_strings: vec![
                "server has gone away".to_string(),
                "no connection to the server".to_string(),
                "Lost connection".to_string(),
                "is dead or not enabled".to_string(),
                "error with".to_string(),
                "Operation timed out".to_string(),
                "Timeout expired".to_string(),
            ],
        }
    }
}

impl ReconnectConfig {
    /// 创建新的重连配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置是否启用
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// 设置最大重试次数
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// 设置重试间隔
    pub fn retry_interval(mut self, interval_ms: u64) -> Self {
        self.retry_interval_ms = interval_ms;
        self
    }

    /// 设置重试间隔倍数
    pub fn retry_multiplier(mut self, multiplier: f64) -> Self {
        self.retry_multiplier = multiplier;
        self
    }

    /// 设置最大重试间隔
    pub fn max_retry_interval(mut self, max_ms: u64) -> Self {
        self.max_retry_interval_ms = max_ms;
        self
    }

    /// 添加断线标识字符串
    pub fn add_break_match_string(mut self, s: &str) -> Self {
        self.break_match_strings.push(s.to_string());
        self
    }

    /// 检查错误是否为断线错误
    pub fn is_break_error(&self, error: &str) -> bool {
        self.break_match_strings.iter()
            .any(|s| error.to_lowercase().contains(&s.to_lowercase()))
    }
}

/// 重连状态
#[derive(Debug, Clone)]
pub struct ReconnectState {
    /// 当前重试次数
    pub retry_count: u32,
    /// 上次重试时间
    pub last_retry_at: Option<Instant>,
    /// 是否正在重连
    pub is_reconnecting: bool,
    /// 最后的错误信息
    pub last_error: Option<String>,
}

impl Default for ReconnectState {
    fn default() -> Self {
        Self::new()
    }
}

impl ReconnectState {
    /// 创建新的重连状态
    pub fn new() -> Self {
        Self {
            retry_count: 0,
            last_retry_at: None,
            is_reconnecting: false,
            last_error: None,
        }
    }

    /// 重置状态
    pub fn reset(&mut self) {
        self.retry_count = 0;
        self.last_retry_at = None;
        self.is_reconnecting = false;
        self.last_error = None;
    }

    /// 记录重试
    pub fn record_retry(&mut self, error: &str) {
        self.retry_count += 1;
        self.last_retry_at = Some(Instant::now());
        self.last_error = Some(error.to_string());
    }
}

/// 重连管理器
pub struct ReconnectManager {
    /// 配置
    pub config: ReconnectConfig,
    /// 状态
    pub state: Arc<RwLock<ReconnectState>>,
}

impl ReconnectManager {
    /// 创建新的重连管理器
    pub fn new(config: ReconnectConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(ReconnectState::new())),
        }
    }

    /// 检查是否应该重试
    pub fn should_retry(&self) -> bool {
        if !self.config.enabled {
            return false;
        }

        let state = self.state.read().unwrap();
        state.retry_count < self.config.max_retries
    }

    /// 检查错误是否为断线错误
    pub fn is_break_error(&self, error: &str) -> bool {
        self.config.is_break_error(error)
    }

    /// 计算下次重试间隔
    pub fn get_next_retry_interval(&self) -> Duration {
        let state = self.state.read().unwrap();
        let interval = self.config.retry_interval_ms as f64
            * self.config.retry_multiplier.powi(state.retry_count as i32);

        let interval = interval.min(self.config.max_retry_interval_ms as f64);
        Duration::from_millis(interval as u64)
    }

    /// 开始重连
    pub fn start_reconnect(&self, error: &str) {
        let mut state = self.state.write().unwrap();
        state.is_reconnecting = true;
        state.record_retry(error);
    }

    /// 结束重连
    pub fn end_reconnect(&self, success: bool) {
        let mut state = self.state.write().unwrap();
        state.is_reconnecting = false;
        if success {
            state.reset();
        }
    }

    /// 执行带重连的操作
    pub fn execute_with_reconnect<F, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Result<T>,
    {
        loop {
            match operation() {
                Ok(result) => {
                    self.end_reconnect(true);
                    return Ok(result);
                }
                Err(e) => {
                    let error_str = e.to_string();

                    if !self.is_break_error(&error_str) {
                        return Err(e);
                    }

                    if !self.should_retry() {
                        return Err(e);
                    }

                    self.start_reconnect(&error_str);

                    let interval = self.get_next_retry_interval();
                    std::thread::sleep(interval);
                }
            }
        }
    }

    /// 获取当前状态
    pub fn get_state(&self) -> ReconnectState {
        self.state.read().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconnect_config() {
        let config = ReconnectConfig::new()
            .max_retries(5)
            .retry_interval(200);

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.retry_interval_ms, 200);
    }

    #[test]
    fn test_is_break_error() {
        let config = ReconnectConfig::new();

        assert!(config.is_break_error("server has gone away"));
        assert!(config.is_break_error("Lost connection to MySQL server"));
        assert!(!config.is_break_error("Syntax error"));
    }

    #[test]
    fn test_reconnect_state() {
        let mut state = ReconnectState::new();

        state.record_retry("Connection lost");

        assert_eq!(state.retry_count, 1);
        assert!(state.last_retry_at.is_some());
        assert_eq!(state.last_error, Some("Connection lost".to_string()));
    }

    #[test]
    fn test_reconnect_manager() {
        let config = ReconnectConfig::new().max_retries(3);
        let manager = ReconnectManager::new(config);

        assert!(manager.should_retry());

        manager.start_reconnect("Connection lost");
        assert!(manager.get_state().is_reconnecting);

        manager.end_reconnect(true);
        assert!(!manager.get_state().is_reconnecting);
    }

    #[test]
    fn test_get_next_retry_interval() {
        let config = ReconnectConfig::new()
            .retry_interval(100)
            .retry_multiplier(2.0);

        let manager = ReconnectManager::new(config);

        let interval1 = manager.get_next_retry_interval();
        manager.start_reconnect("error");

        let interval2 = manager.get_next_retry_interval();
        assert!(interval2 >= interval1);
    }
}
