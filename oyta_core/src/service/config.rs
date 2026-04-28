//! 服务配置加载器模块
//!
//! 从独立的配置文件中加载各服务的配置
//! 支持 PHP 配置文件解析和环境变量替换
//!
//! # 配置文件结构
//! config/
//! ├── cache.php      # 缓存服务配置
//! ├── queue.php      # 队列服务配置
//! ├── websocket.php  # WebSocket服务配置
//! ├── timer.php      # 定时器服务配置
//! ├── session.php    # 会话服务配置
//! └── ...

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::traits::ServiceConfig;

/// 服务配置加载器
///
/// 从配置目录加载各服务的独立配置文件
pub struct ServiceConfigLoader {
    /// 配置目录路径
    config_dir: PathBuf,
    /// 已加载的配置缓存
    cache: HashMap<String, ServiceConfig>,
}

impl ServiceConfigLoader {
    /// 创建新的配置加载器
    ///
    /// # 参数
    /// - `config_dir`: 配置目录路径
    ///
    /// # 返回值
    /// 新的配置加载器实例
    pub fn new(config_dir: &Path) -> Self {
        Self {
            config_dir: config_dir.to_path_buf(),
            cache: HashMap::new(),
        }
    }

    /// 加载指定服务的配置
    ///
    /// # 参数
    /// - `service_name`: 服务名称
    ///
    /// # 返回值
    /// 服务配置，加载失败返回空配置
    pub fn load(&mut self, service_name: &str) -> ServiceConfig {
        // 检查缓存
        if let Some(config) = self.cache.get(service_name) {
            return config.clone();
        }

        // 构建配置文件路径
        let config_file = self.config_dir.join(format!("{}.php", service_name));

        // 加载配置
        let config = if config_file.exists() {
            self.load_php_config(&config_file)
        } else {
            // 返回默认配置
            self.get_default_config(service_name)
        };

        // 缓存配置
        self.cache.insert(service_name.to_string(), config.clone());

        config
    }

    /// 加载所有服务配置
    ///
    /// 扫描配置目录，加载所有 .php 配置文件
    ///
    /// # 返回值
    /// 服务名到配置的映射
    pub fn load_all(&mut self) -> HashMap<String, ServiceConfig> {
        let mut configs = HashMap::new();

        // 扫描配置目录
        if let Ok(entries) = std::fs::read_dir(&self.config_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("php") {
                    if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                        let config = self.load(name);
                        configs.insert(name.to_string(), config);
                    }
                }
            }
        }

        configs
    }

    /// 加载 PHP 配置文件
    ///
    /// 解析 PHP 配置文件并转换为 Rust HashMap
    ///
    /// # 参数
    /// - `path`: 配置文件路径
    ///
    /// # 返回值
    /// 解析后的配置
    fn load_php_config(&self, path: &Path) -> ServiceConfig {
        // 读取文件内容
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("无法读取配置文件 {}: {}", path.display(), e);
                return HashMap::new();
            }
        };

        // 解析 PHP 配置
        self.parse_php_config(&content)
    }

    /// 解析 PHP 配置内容
    ///
    /// 将 PHP 数组语法转换为 Rust HashMap
    ///
    /// # 参数
    /// - `content`: PHP 配置文件内容
    ///
    /// # 返回值
    /// 解析后的配置
    fn parse_php_config(&self, content: &str) -> ServiceConfig {
        let mut config = HashMap::new();

        // 简单的 PHP 配置解析
        // 支持 return ['key' => 'value'] 格式
        // 注意：这是一个简化实现，实际项目中应使用完整的 PHP 解析器

        // 提取键值对
        for line in content.lines() {
            let line = line.trim();

            // 跳过注释和空行
            if line.starts_with("//") || line.starts_with("#") || line.starts_with("/*") {
                continue;
            }
            if line.is_empty() || line.starts_with("return") || line.starts_with("<?php") {
                continue;
            }

            // 解析键值对: 'key' => 'value' 或 "key" => "value"
            if let Some((key, value)) = self.parse_key_value(line) {
                config.insert(key, value);
            }
        }

        config
    }

    /// 解析键值对
    ///
    /// 解析 PHP 数组中的键值对
    ///
    /// # 参数
    /// - `line`: 配置行
    ///
    /// # 返回值
    /// 解析成功返回 (key, value) 元组
    fn parse_key_value(&self, line: &str) -> Option<(String, serde_json::Value)> {
        // 查找 => 分隔符
        let separator = line.find("=>")?;
        let key_part = line[..separator].trim();
        let value_part = line[separator + 2..].trim();

        // 提取键名（去除引号）
        let key = self.extract_string(key_part)?;

        // 解析值
        let value = self.parse_value(value_part)?;

        Some((key, value))
    }

    /// 提取字符串
    ///
    /// 从引号中提取字符串内容
    ///
    /// # 参数
    /// - `s`: 包含引号的字符串
    ///
    /// # 返回值
    /// 去除引号后的字符串
    fn extract_string(&self, s: &str) -> Option<String> {
        let s = s.trim();
        if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
            Some(s[1..s.len() - 1].to_string())
        } else {
            Some(s.to_string())
        }
    }

    /// 解析配置值
    ///
    /// 将 PHP 值转换为 JSON Value
    ///
    /// # 参数
    /// - `s`: 值字符串
    ///
    /// # 返回值
    /// 转换后的 JSON Value
    fn parse_value(&self, s: &str) -> Option<serde_json::Value> {
        let s = s.trim_end_matches(',').trim();

        // 布尔值
        if s == "true" {
            return Some(serde_json::Value::Bool(true));
        }
        if s == "false" {
            return Some(serde_json::Value::Bool(false));
        }

        // null
        if s == "null" {
            return Some(serde_json::Value::Null);
        }

        // 数字
        if let Ok(i) = s.parse::<i64>() {
            return Some(serde_json::Value::Number(i.into()));
        }
        if let Ok(f) = s.parse::<f64>() {
            if let Some(n) = serde_json::Number::from_f64(f) {
                return Some(serde_json::Value::Number(n));
            }
        }

        // 字符串
        if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
            return Some(serde_json::Value::String(s[1..s.len() - 1].to_string()));
        }

        // env() 函数调用
        if s.starts_with("env(") {
            return self.parse_env_function(s);
        }

        // 默认作为字符串
        Some(serde_json::Value::String(s.to_string()))
    }

    /// 解析 env() 函数调用
    ///
    /// 处理环境变量引用
    ///
    /// # 参数
    /// - `s`: env() 调用字符串
    ///
    /// # 返回值
    /// 环境变量值
    fn parse_env_function(&self, s: &str) -> Option<serde_json::Value> {
        // 提取 env() 参数
        let start = s.find('(')?;
        let end = s.rfind(')')?;
        let args = &s[start + 1..end];

        // 分割参数
        let parts: Vec<&str> = args.split(',').collect();
        if parts.is_empty() {
            return None;
        }

        // 获取环境变量名
        let env_key = self.extract_string(parts[0].trim())?;

        // 获取默认值
        let default = if parts.len() > 1 {
            self.parse_value(parts[1].trim())
        } else {
            None
        };

        // 读取环境变量
        let value = std::env::var(&env_key).ok();

        match value {
            Some(v) => {
                // 尝试解析为数字或布尔值
                if let Ok(i) = v.parse::<i64>() {
                    Some(serde_json::Value::Number(i.into()))
                } else if let Ok(f) = v.parse::<f64>() {
                    if let Some(n) = serde_json::Number::from_f64(f) {
                        Some(serde_json::Value::Number(n))
                    } else {
                        Some(serde_json::Value::String(v))
                    }
                } else if v == "true" {
                    Some(serde_json::Value::Bool(true))
                } else if v == "false" {
                    Some(serde_json::Value::Bool(false))
                } else {
                    Some(serde_json::Value::String(v))
                }
            }
            None => default,
        }
    }

    /// 获取服务的默认配置
    ///
    /// 当配置文件不存在时，返回服务的默认配置
    ///
    /// # 参数
    /// - `service_name`: 服务名称
    ///
    /// # 返回值
    /// 默认配置
    fn get_default_config(&self, service_name: &str) -> ServiceConfig {
        let mut config = HashMap::new();

        match service_name {
            "cache" => {
                config.insert("default".to_string(), serde_json::json!("file"));
                config.insert("enable".to_string(), serde_json::json!(true));
                config.insert("deferred".to_string(), serde_json::json!(true));
            }
            "queue" => {
                config.insert("default".to_string(), serde_json::json!("sync"));
                config.insert("enable".to_string(), serde_json::json!(true));
                config.insert("deferred".to_string(), serde_json::json!(true));
            }
            "websocket" => {
                config.insert("enable".to_string(), serde_json::json!(false));
                config.insert("port".to_string(), serde_json::json!(9501));
                config.insert("max_connections".to_string(), serde_json::json!(1000));
                config.insert("deferred".to_string(), serde_json::json!(true));
            }
            "timer" => {
                config.insert("enable".to_string(), serde_json::json!(true));
                config.insert("precision".to_string(), serde_json::json!(100));
                config.insert("deferred".to_string(), serde_json::json!(true));
            }
            "session" => {
                config.insert("driver".to_string(), serde_json::json!("file"));
                config.insert("enable".to_string(), serde_json::json!(true));
                config.insert("lifetime".to_string(), serde_json::json!(1440));
                config.insert("deferred".to_string(), serde_json::json!(true));
            }
            "log" => {
                config.insert("default".to_string(), serde_json::json!("file"));
                config.insert("enable".to_string(), serde_json::json!(true));
                config.insert("deferred".to_string(), serde_json::json!(true));
            }
            "database" => {
                config.insert("default".to_string(), serde_json::json!("mysql"));
                config.insert("enable".to_string(), serde_json::json!(true));
                config.insert("deferred".to_string(), serde_json::json!(true));
            }
            _ => {
                // 通用默认配置
                config.insert("enable".to_string(), serde_json::json!(true));
                config.insert("deferred".to_string(), serde_json::json!(true));
            }
        }

        config
    }

    /// 合并配置
    ///
    /// 将多个配置源合并，后面的配置覆盖前面的
    ///
    /// # 参数
    /// - `base`: 基础配置
    /// - `override`: 覆盖配置
    ///
    /// # 返回值
    /// 合并后的配置
    pub fn merge_configs(base: &ServiceConfig, override_config: &ServiceConfig) -> ServiceConfig {
        let mut merged = base.clone();
        for (key, value) in override_config {
            merged.insert(key.clone(), value.clone());
        }
        merged
    }

    /// 清除配置缓存
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

/// 缓存服务配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 默认驱动
    pub default_driver: String,
    /// 是否启用
    pub enable: bool,
    /// 键前缀
    pub prefix: String,
    /// Redis 配置
    pub redis: Option<RedisConfig>,
    /// 文件配置
    pub file: Option<FileCacheConfig>,
}

/// Redis 配置
#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 密码
    pub password: String,
    /// 数据库编号
    pub database: u8,
    /// 连接超时
    pub timeout: u64,
    /// 键前缀
    pub prefix: String,
}

/// 文件缓存配置
#[derive(Debug, Clone)]
pub struct FileCacheConfig {
    /// 缓存目录
    pub path: String,
    /// 过期时间
    pub expire: u64,
}

/// 队列服务配置
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// 默认驱动
    pub default_driver: String,
    /// 是否启用
    pub enable: bool,
    /// Worker 数量
    pub workers: usize,
    /// 重试次数
    pub tries: u32,
    /// 超时时间
    pub timeout: u64,
    /// Redis 配置
    pub redis: Option<RedisConfig>,
}

/// WebSocket 服务配置
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// 是否启用
    pub enable: bool,
    /// 监听地址
    pub host: String,
    /// 监听端口
    pub port: u16,
    /// 最大连接数
    pub max_connections: usize,
    /// 心跳间隔（秒）
    pub heartbeat_interval: u64,
    /// 心跳超时（秒）
    pub heartbeat_timeout: u64,
    /// 最大消息大小
    pub max_message_size: usize,
}

/// 定时器服务配置
#[derive(Debug, Clone)]
pub struct TimerConfig {
    /// 是否启用
    pub enable: bool,
    /// 时间轮精度（毫秒）
    pub tick_ms: u64,
    /// 每层槽数
    pub slots_per_wheel: usize,
    /// 层数
    pub wheels: usize,
    /// 最大任务数
    pub max_tasks: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_driver: "file".to_string(),
            enable: true,
            prefix: "oyta_".to_string(),
            redis: None,
            file: Some(FileCacheConfig {
                path: "runtime/cache".to_string(),
                expire: 3600,
            }),
        }
    }
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            default_driver: "sync".to_string(),
            enable: true,
            workers: 4,
            tries: 3,
            timeout: 60,
            redis: None,
        }
    }
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            enable: false,
            host: "0.0.0.0".to_string(),
            port: 9501,
            max_connections: 1000,
            heartbeat_interval: 30,
            heartbeat_timeout: 60,
            max_message_size: 1024 * 1024, // 1MB
        }
    }
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            enable: true,
            tick_ms: 100,
            slots_per_wheel: 60,
            wheels: 4,
            max_tasks: 10000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load_config() {
        // 创建临时目录
        let dir = tempdir().unwrap();
        let config_dir = dir.path();

        // 创建测试配置文件
        let config_content = r#"<?php
return [
    'default' => 'redis',
    'enable' => true,
    'prefix' => 'test_',
];
"#;
        let mut file = std::fs::File::create(config_dir.join("cache.php")).unwrap();
        file.write_all(config_content.as_bytes()).unwrap();

        // 加载配置
        let mut loader = ServiceConfigLoader::new(config_dir);
        let config = loader.load("cache");

        // 验证
        assert_eq!(
            config.get("default").and_then(|v| v.as_str()),
            Some("redis")
        );
        assert_eq!(config.get("enable").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn test_default_config() {
        let dir = tempdir().unwrap();
        let mut loader = ServiceConfigLoader::new(dir.path());

        // 加载不存在的配置
        let config = loader.load("cache");

        // 应该返回默认配置
        assert_eq!(
            config.get("default").and_then(|v| v.as_str()),
            Some("file")
        );
    }

    #[test]
    fn test_env_function() {
        // 设置环境变量（unsafe 操作）
        unsafe {
            std::env::set_var("TEST_VAR", "test_value");
        }

        let loader = ServiceConfigLoader::new(Path::new("."));
        let value = loader.parse_env_function("env('TEST_VAR', 'default')");

        assert_eq!(value, Some(serde_json::Value::String("test_value".to_string())));

        // 清理（unsafe 操作）
        unsafe {
            std::env::remove_var("TEST_VAR");
        }
    }
}
