//! SQL 监听模块
//!
//! 提供数据库 SQL 监听功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库

use crate::interpreter::value::Value;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// 查询日志条目
#[derive(Debug, Clone)]
pub struct QueryLogEntry {
    /// SQL 语句
    pub sql: String,
    /// 绑定参数
    pub bindings: Vec<Value>,
    /// 执行时间（毫秒）
    pub duration_ms: f64,
    /// 执行时间戳
    pub timestamp: String,
    /// 连接名
    pub connection: String,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error: Option<String>,
}

impl QueryLogEntry {
    /// 创建新的查询日志条目
    pub fn new(sql: &str, bindings: Vec<Value>, duration_ms: f64, connection: &str) -> Self {
        Self {
            sql: sql.to_string(),
            bindings,
            duration_ms,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            connection: connection.to_string(),
            success: true,
            error: None,
        }
    }

    /// 设置错误
    pub fn with_error(mut self, error: &str) -> Self {
        self.success = false;
        self.error = Some(error.to_string());
        self
    }
}

/// 查询监听器类型
pub type QueryListenerCallback = Box<dyn Fn(&QueryLogEntry) + Send + Sync>;

/// 查询监听器
pub struct QueryListener {
    /// 监听器列表
    listeners: Arc<RwLock<Vec<QueryListenerCallback>>>,
    /// 查询日志
    query_log: Arc<RwLock<Vec<QueryLogEntry>>>,
    /// 是否记录日志
    enable_logging: bool,
    /// 最大日志条目数
    max_log_entries: usize,
}

impl Default for QueryListener {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryListener {
    /// 创建新的查询监听器
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(Vec::new())),
            query_log: Arc::new(RwLock::new(Vec::new())),
            enable_logging: true,
            max_log_entries: 1000,
        }
    }

    /// 设置是否记录日志
    pub fn enable_logging(mut self, enable: bool) -> Self {
        self.enable_logging = enable;
        self
    }

    /// 设置最大日志条目数
    pub fn max_log_entries(mut self, max: usize) -> Self {
        self.max_log_entries = max;
        self
    }

    /// 添加监听器
    pub fn listen<F>(&self, callback: F)
    where
        F: Fn(&QueryLogEntry) + Send + Sync + 'static,
    {
        let mut listeners = self.listeners.write().unwrap();
        listeners.push(Box::new(callback));
    }

    /// 触发查询事件
    pub fn fire(&self, entry: &QueryLogEntry) {
        // 记录日志
        if self.enable_logging {
            let mut log = self.query_log.write().unwrap();
            if log.len() >= self.max_log_entries {
                log.remove(0);
            }
            log.push(entry.clone());
        }

        // 调用监听器
        let listeners = self.listeners.read().unwrap();
        for listener in listeners.iter() {
            listener(entry);
        }
    }

    /// 开始查询计时
    pub fn start_query(&self) -> Instant {
        Instant::now()
    }

    /// 结束查询并记录
    pub fn end_query(
        &self,
        start: Instant,
        sql: &str,
        bindings: Vec<Value>,
        connection: &str,
    ) -> QueryLogEntry {
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let entry = QueryLogEntry::new(sql, bindings, duration_ms, connection);
        self.fire(&entry);
        entry
    }

    /// 结束查询并记录错误
    pub fn end_query_with_error(
        &self,
        start: Instant,
        sql: &str,
        bindings: Vec<Value>,
        connection: &str,
        error: &str,
    ) -> QueryLogEntry {
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        let entry = QueryLogEntry::new(sql, bindings, duration_ms, connection).with_error(error);
        self.fire(&entry);
        entry
    }

    /// 获取查询日志
    pub fn get_log(&self) -> Vec<QueryLogEntry> {
        let log = self.query_log.read().unwrap();
        log.clone()
    }

    /// 清空查询日志
    pub fn clear_log(&self) {
        let mut log = self.query_log.write().unwrap();
        log.clear();
    }

    /// 获取慢查询
    pub fn get_slow_queries(&self, threshold_ms: f64) -> Vec<QueryLogEntry> {
        let log = self.query_log.read().unwrap();
        log.iter()
            .filter(|entry| entry.duration_ms >= threshold_ms)
            .cloned()
            .collect()
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> QueryStats {
        let log = self.query_log.read().unwrap();

        let total_queries = log.len();
        let successful_queries = log.iter().filter(|e| e.success).count();
        let failed_queries = total_queries - successful_queries;

        let total_duration: f64 = log.iter().map(|e| e.duration_ms).sum();
        let avg_duration = if total_queries > 0 {
            total_duration / total_queries as f64
        } else {
            0.0
        };

        let max_duration = log.iter()
            .map(|e| e.duration_ms)
            .fold(0.0, f64::max);

        QueryStats {
            total_queries,
            successful_queries,
            failed_queries,
            total_duration_ms: total_duration,
            avg_duration_ms: avg_duration,
            max_duration_ms: max_duration,
        }
    }
}

/// 查询统计
#[derive(Debug, Clone)]
pub struct QueryStats {
    /// 总查询数
    pub total_queries: usize,
    /// 成功查询数
    pub successful_queries: usize,
    /// 失败查询数
    pub failed_queries: usize,
    /// 总执行时间（毫秒）
    pub total_duration_ms: f64,
    /// 平均执行时间（毫秒）
    pub avg_duration_ms: f64,
    /// 最大执行时间（毫秒）
    pub max_duration_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_log_entry() {
        let entry = QueryLogEntry::new("SELECT * FROM users", vec![], 1.5, "default");

        assert_eq!(entry.sql, "SELECT * FROM users");
        assert_eq!(entry.duration_ms, 1.5);
        assert!(entry.success);
    }

    #[test]
    fn test_query_listener() {
        let listener = QueryListener::new();
        let called = Arc::new(RwLock::new(false));

        let called_clone = called.clone();
        listener.listen(move |_entry| {
            *called_clone.write().unwrap() = true;
        });

        let entry = QueryLogEntry::new("SELECT 1", vec![], 0.1, "default");
        listener.fire(&entry);

        assert!(*called.read().unwrap());
    }

    #[test]
    fn test_get_slow_queries() {
        let listener = QueryListener::new();

        let entry1 = QueryLogEntry::new("SELECT 1", vec![], 10.0, "default");
        let entry2 = QueryLogEntry::new("SELECT 2", vec![], 100.0, "default");

        listener.fire(&entry1);
        listener.fire(&entry2);

        let slow = listener.get_slow_queries(50.0);
        assert_eq!(slow.len(), 1);
    }

    #[test]
    fn test_get_stats() {
        let listener = QueryListener::new();

        let entry1 = QueryLogEntry::new("SELECT 1", vec![], 10.0, "default");
        let entry2 = QueryLogEntry::new("SELECT 2", vec![], 20.0, "default");

        listener.fire(&entry1);
        listener.fire(&entry2);

        let stats = listener.get_stats();
        assert_eq!(stats.total_queries, 2);
        assert_eq!(stats.total_duration_ms, 30.0);
    }
}
