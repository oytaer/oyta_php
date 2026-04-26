//! 日志通道管理
//!
//! 提供多通道日志功能
//! 支持按通道配置不同的日志级别和输出目标
//! 兼容 ThinkPHP 8.0 的日志通道配置

use std::collections::HashMap;
use std::path::PathBuf;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Emergency,
    Alert,
    Critical,
    Error,
    Warning,
    Notice,
    Info,
    Debug,
}

impl LogLevel {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Emergency => "EMERGENCY",
            LogLevel::Alert => "ALERT",
            LogLevel::Critical => "CRITICAL",
            LogLevel::Error => "ERROR",
            LogLevel::Warning => "WARNING",
            LogLevel::Notice => "NOTICE",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
        }
    }

    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "EMERGENCY" => Some(LogLevel::Emergency),
            "ALERT" => Some(LogLevel::Alert),
            "CRITICAL" => Some(LogLevel::Critical),
            "ERROR" => Some(LogLevel::Error),
            "WARNING" | "WARN" => Some(LogLevel::Warning),
            "NOTICE" => Some(LogLevel::Notice),
            "INFO" => Some(LogLevel::Info),
            "DEBUG" => Some(LogLevel::Debug),
            _ => None,
        }
    }
}

/// 日志通道配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// 通道类型：file、stdout、stderr
    #[serde(rename = "type")]
    pub channel_type: String,
    /// 日志级别
    pub level: String,
    /// 日志文件路径（仅 file 类型）
    pub path: Option<String>,
    /// 日志文件前缀
    pub prefix: Option<String>,
    /// 最大保留文件数
    pub max_files: Option<usize>,
    /// 是否启用
    pub enabled: Option<bool>,
}

/// 日志通道管理器
pub struct LogManager {
    /// 通道配置
    channels: HashMap<String, ChannelConfig>,
    /// 日志根目录
    log_dir: PathBuf,
}

impl LogManager {
    /// 创建新的日志管理器
    pub fn new(log_dir: &std::path::Path) -> Self {
        Self {
            channels: HashMap::new(),
            log_dir: log_dir.to_path_buf(),
        }
    }

    /// 注册日志通道
    pub fn add_channel(&mut self, name: &str, config: ChannelConfig) {
        self.channels.insert(name.to_string(), config);
    }

    /// 获取通道配置
    pub fn get_channel(&self, name: &str) -> Option<&ChannelConfig> {
        self.channels.get(name)
    }

    /// 获取日志文件路径
    pub fn get_log_path(&self, channel_name: &str) -> PathBuf {
        if let Some(config) = self.channels.get(channel_name) {
            if let Some(path) = &config.path {
                return PathBuf::from(path);
            }
        }
        self.log_dir.join(format!("{}.log", channel_name))
    }

    /// 写入日志到指定通道
    pub fn log(&self, channel_name: &str, level: LogLevel, message: &str) {
        let config = match self.channels.get(channel_name) {
            Some(c) => c,
            None => {
                self.log_to_file(channel_name, level, message);
                return;
            }
        };

        if config.enabled == Some(false) {
            return;
        }

        if let Some(config_level) = LogLevel::from_str(&config.level) {
            if level > config_level {
                return;
            }
        }

        match config.channel_type.as_str() {
            "file" => self.log_to_file(channel_name, level, message),
            "stdout" => self.log_to_stdout(level, message),
            "stderr" => self.log_to_stderr(level, message),
            _ => self.log_to_file(channel_name, level, message),
        }
    }

    /// 写入日志到文件
    fn log_to_file(&self, channel_name: &str, level: LogLevel, message: &str) {
        let path = self.get_log_path(channel_name);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("[{}] [{}] {}", timestamp, level.as_str(), message);
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            use std::io::Write;
            let _ = writeln!(file, "{}", line);
        }
    }

    /// 写入日志到标准输出
    fn log_to_stdout(&self, level: LogLevel, message: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        println!("[{}] [{}] {}", timestamp, level.as_str(), message);
    }

    /// 写入日志到标准错误
    fn log_to_stderr(&self, level: LogLevel, message: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        eprintln!("[{}] [{}] {}", timestamp, level.as_str(), message);
    }

    /// 便捷方法：记录 INFO 日志
    pub fn info(&self, channel: &str, message: &str) {
        self.log(channel, LogLevel::Info, message);
    }

    /// 便捷方法：记录 ERROR 日志
    pub fn error(&self, channel: &str, message: &str) {
        self.log(channel, LogLevel::Error, message);
    }

    /// 便捷方法：记录 WARNING 日志
    pub fn warning(&self, channel: &str, message: &str) {
        self.log(channel, LogLevel::Warning, message);
    }

    /// 便捷方法：记录 DEBUG 日志
    pub fn debug(&self, channel: &str, message: &str) {
        self.log(channel, LogLevel::Debug, message);
    }

    /// 初始化默认通道配置
    pub fn init_default_channels(&mut self) {
        self.add_channel("default", ChannelConfig {
            channel_type: "file".to_string(),
            level: "info".to_string(),
            path: Some(self.log_dir.join("oyta.log").to_str().unwrap_or("").to_string()),
            prefix: Some("oyta".to_string()),
            max_files: Some(30),
            enabled: Some(true),
        });

        self.add_channel("error", ChannelConfig {
            channel_type: "file".to_string(),
            level: "error".to_string(),
            path: Some(self.log_dir.join("error.log").to_str().unwrap_or("").to_string()),
            prefix: Some("error".to_string()),
            max_files: Some(30),
            enabled: Some(true),
        });

        self.add_channel("sql", ChannelConfig {
            channel_type: "file".to_string(),
            level: "info".to_string(),
            path: Some(self.log_dir.join("sql.log").to_str().unwrap_or("").to_string()),
            prefix: Some("sql".to_string()),
            max_files: Some(30),
            enabled: Some(true),
        });
    }
}

/// 全局日志管理器
static LOG_MANAGER: Lazy<RwLock<Option<LogManager>>> = Lazy::new(|| {
    RwLock::new(None)
});

/// 初始化全局日志管理器
pub fn init_global(log_dir: &std::path::Path) {
    let mut manager = LogManager::new(log_dir);
    manager.init_default_channels();
    let mut guard = LOG_MANAGER.write();
    *guard = Some(manager);
}

/// 获取全局日志管理器
pub fn global() -> parking_lot::RwLockReadGuard<'static, Option<LogManager>> {
    LOG_MANAGER.read()
}

/// 全局日志写入
pub fn log(channel: &str, level: LogLevel, message: &str) {
    let guard = LOG_MANAGER.read();
    if let Some(manager) = guard.as_ref() {
        manager.log(channel, level, message);
    }
}

/// 全局 INFO 日志
pub fn info(channel: &str, message: &str) {
    log(channel, LogLevel::Info, message);
}

/// 全局 ERROR 日志
pub fn error(channel: &str, message: &str) {
    log(channel, LogLevel::Error, message);
}

/// 全局 WARNING 日志
pub fn warning(channel: &str, message: &str) {
    log(channel, LogLevel::Warning, message);
}

/// 全局 DEBUG 日志
pub fn debug(channel: &str, message: &str) {
    log(channel, LogLevel::Debug, message);
}
