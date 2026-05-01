//! 数据库日志模块
//!
//! 提供数据库日志记录功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库

use std::sync::{Arc, RwLock};

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// 调试
    Debug,
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
}

impl LogLevel {
    /// 获取日志级别名称
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
        }
    }
}

/// 日志条目
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// 日志级别
    pub level: LogLevel,
    /// 日志消息
    pub message: String,
    /// 时间戳
    pub timestamp: String,
    /// 上下文
    pub context: std::collections::HashMap<String, String>,
}

impl LogEntry {
    /// 创建新的日志条目
    pub fn new(level: LogLevel, message: &str) -> Self {
        Self {
            level,
            message: message.to_string(),
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            context: std::collections::HashMap::new(),
        }
    }

    /// 添加上下文
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }

    /// 格式化为字符串
    pub fn format(&self) -> String {
        let context_str = if self.context.is_empty() {
            String::new()
        } else {
            let pairs: Vec<String> = self.context.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!(" [{}]", pairs.join(", "))
        };

        format!(
            "[{}] [{}] {}{}",
            self.timestamp,
            self.level.as_str(),
            self.message,
            context_str
        )
    }
}

/// 日志处理器类型
pub type LogHandler = Box<dyn Fn(&LogEntry) + Send + Sync>;

/// 数据库日志器
pub struct DatabaseLogger {
    /// 日志处理器列表
    handlers: Arc<RwLock<Vec<LogHandler>>>,
    /// 最小日志级别
    min_level: LogLevel,
}

impl Default for DatabaseLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseLogger {
    /// 创建新的日志器
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
            min_level: LogLevel::Debug,
        }
    }

    /// 设置最小日志级别
    pub fn min_level(mut self, level: LogLevel) -> Self {
        self.min_level = level;
        self
    }

    /// 添加日志处理器
    pub fn add_handler<F>(&self, handler: F)
    where
        F: Fn(&LogEntry) + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.write().unwrap();
        handlers.push(Box::new(handler));
    }

    /// 记录日志
    pub fn log(&self, entry: &LogEntry) {
        if entry.level < self.min_level {
            return;
        }

        let handlers = self.handlers.read().unwrap();
        for handler in handlers.iter() {
            handler(entry);
        }
    }

    /// 记录调试日志
    pub fn debug(&self, message: &str) {
        self.log(&LogEntry::new(LogLevel::Debug, message));
    }

    /// 记录信息日志
    pub fn info(&self, message: &str) {
        self.log(&LogEntry::new(LogLevel::Info, message));
    }

    /// 记录警告日志
    pub fn warning(&self, message: &str) {
        self.log(&LogEntry::new(LogLevel::Warning, message));
    }

    /// 记录错误日志
    pub fn error(&self, message: &str) {
        self.log(&LogEntry::new(LogLevel::Error, message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry() {
        let entry = LogEntry::new(LogLevel::Info, "Test message");
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Test message");
    }

    #[test]
    fn test_log_entry_format() {
        let entry = LogEntry::new(LogLevel::Info, "Test message");
        let formatted = entry.format();

        assert!(formatted.contains("INFO"));
        assert!(formatted.contains("Test message"));
    }

    #[test]
    fn test_database_logger() {
        let logger = DatabaseLogger::new();
        let logged = Arc::new(RwLock::new(false));

        let logged_clone = logged.clone();
        logger.add_handler(move |_entry| {
            *logged_clone.write().unwrap() = true;
        });

        logger.info("Test message");

        assert!(*logged.read().unwrap());
    }

    #[test]
    fn test_min_level() {
        let logger = DatabaseLogger::new().min_level(LogLevel::Warning);
        let count = Arc::new(RwLock::new(0));

        let count_clone = count.clone();
        logger.add_handler(move |_entry| {
            *count_clone.write().unwrap() += 1;
        });

        logger.debug("Debug message");
        logger.info("Info message");
        logger.warning("Warning message");
        logger.error("Error message");

        assert_eq!(*count.read().unwrap(), 2);
    }
}
