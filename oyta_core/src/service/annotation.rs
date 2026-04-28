//! 服务注解解析器模块
//!
//! 解析 PHP 8 注解，支持服务声明和配置
//!
//! # 支持的注解
//! - #[Service] - 服务声明
//! - #[WebSocketHandler] - WebSocket 事件处理器
//! - #[QueueJob] - 队列任务定义
//! - #[TimerTask] - 定时任务定义
//!
//! # 注解格式
//! ```php
//! #[Service(name: 'websocket', enable: true, config: ['port' => 9501])]
//! class WebSocketService { }
//! ```

use std::collections::HashMap;
use std::path::Path;

use super::traits::ServiceConfig;

/// 注解类型枚举
///
/// 定义所有支持的注解类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationType {
    /// 服务注解 #[Service]
    Service,
    /// WebSocket 处理器注解 #[WebSocketHandler]
    WebSocketHandler,
    /// 队列任务注解 #[QueueJob]
    QueueJob,
    /// 定时任务注解 #[TimerTask]
    TimerTask,
    /// 服务启动注解 #[ServiceBoot]
    ServiceBoot,
    /// 服务关闭注解 #[ServiceShutdown]
    ServiceShutdown,
    /// 未知注解
    Unknown(String),
}

impl AnnotationType {
    /// 从字符串解析注解类型
    ///
    /// # 参数
    /// - `name`: 注解名称
    ///
    /// # 返回值
    /// 对应的注解类型
    pub fn from_str(name: &str) -> Self {
        match name {
            "Service" | "Oyta\\Attributes\\Service" => AnnotationType::Service,
            "WebSocketHandler" | "Oyta\\Attributes\\WebSocketHandler" => {
                AnnotationType::WebSocketHandler
            }
            "QueueJob" | "Oyta\\Attributes\\QueueJob" => AnnotationType::QueueJob,
            "TimerTask" | "Oyta\\Attributes\\TimerTask" => AnnotationType::TimerTask,
            "ServiceBoot" | "Oyta\\Attributes\\ServiceBoot" => AnnotationType::ServiceBoot,
            "ServiceShutdown" | "Oyta\\Attributes\\ServiceShutdown" => {
                AnnotationType::ServiceShutdown
            }
            _ => AnnotationType::Unknown(name.to_string()),
        }
    }
}

/// 注解参数
///
/// 存储注解的参数信息
#[derive(Debug, Clone)]
pub struct AnnotationParams {
    /// 位置参数列表
    pub positional: Vec<String>,
    /// 命名参数映射
    pub named: HashMap<String, String>,
}

impl AnnotationParams {
    /// 创建空的注解参数
    pub fn new() -> Self {
        Self {
            positional: Vec::new(),
            named: HashMap::new(),
        }
    }

    /// 获取命名参数
    ///
    /// # 参数
    /// - `name`: 参数名
    ///
    /// # 返回值
    /// 参数值
    pub fn get(&self, name: &str) -> Option<&String> {
        self.named.get(name)
    }

    /// 获取命名参数并解析为布尔值
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.named.get(name).and_then(|v| v.parse::<bool>().ok())
    }

    /// 获取命名参数并解析为整数
    pub fn get_int(&self, name: &str) -> Option<i64> {
        self.named.get(name).and_then(|v| v.parse::<i64>().ok())
    }

    /// 获取命名参数并解析为字符串
    pub fn get_string(&self, name: &str) -> Option<String> {
        self.named.get(name).map(|v| {
            // 去除引号
            if (v.starts_with('\'') && v.ends_with('\''))
                || (v.starts_with('"') && v.ends_with('"'))
            {
                v[1..v.len() - 1].to_string()
            } else {
                v.clone()
            }
        })
    }
}

impl Default for AnnotationParams {
    fn default() -> Self {
        Self::new()
    }
}

/// 注解信息
///
/// 存储完整的注解信息
#[derive(Debug, Clone)]
pub struct Annotation {
    /// 注解类型
    pub annotation_type: AnnotationType,
    /// 注解参数
    pub params: AnnotationParams,
    /// 所在文件路径
    pub file_path: String,
    /// 所在行号
    pub line_number: usize,
}

impl Annotation {
    /// 创建新的注解
    ///
    /// # 参数
    /// - `annotation_type`: 注解类型
    /// - `params`: 注解参数
    ///
    /// # 返回值
    /// 新的注解实例
    pub fn new(annotation_type: AnnotationType, params: AnnotationParams) -> Self {
        Self {
            annotation_type,
            params,
            file_path: String::new(),
            line_number: 0,
        }
    }

    /// 设置文件路径
    pub fn with_file(mut self, file_path: &str) -> Self {
        self.file_path = file_path.to_string();
        self
    }

    /// 设置行号
    pub fn with_line(mut self, line_number: usize) -> Self {
        self.line_number = line_number;
        self
    }
}

/// 服务注解信息
///
/// 从 #[Service] 注解解析出的服务定义
#[derive(Debug, Clone)]
pub struct ServiceAnnotation {
    /// 服务名称
    pub name: String,
    /// 是否启用
    pub enable: bool,
    /// 服务配置
    pub config: ServiceConfig,
    /// 是否延迟加载
    pub deferred: bool,
    /// 类名
    pub class_name: String,
    /// 文件路径
    pub file_path: String,
}

/// WebSocket 处理器注解信息
#[derive(Debug, Clone)]
pub struct WebSocketHandlerAnnotation {
    /// 事件名称
    pub event: String,
    /// 方法名
    pub method_name: String,
    /// 类名
    pub class_name: String,
}

/// 队列任务注解信息
#[derive(Debug, Clone)]
pub struct QueueJobAnnotation {
    /// 队列名称
    pub queue: String,
    /// 重试次数
    pub tries: u32,
    /// 超时时间
    pub timeout: u64,
    /// 延迟时间
    pub delay: u64,
    /// 方法名
    pub method_name: String,
    /// 类名
    pub class_name: String,
}

/// 定时任务注解信息
#[derive(Debug, Clone)]
pub struct TimerTaskAnnotation {
    /// Cron 表达式
    pub cron: Option<String>,
    /// 执行间隔（秒）
    pub interval: Option<u64>,
    /// 延迟时间（秒）
    pub delay: Option<u64>,
    /// 方法名
    pub method_name: String,
    /// 类名
    pub class_name: String,
}

/// 注解解析器
///
/// 解析 PHP 文件中的注解
pub struct AnnotationParser {
    /// 已解析的注解缓存
    cache: HashMap<String, Vec<Annotation>>,
}

impl AnnotationParser {
    /// 创建新的注解解析器
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// 解析文件中的注解
    ///
    /// # 参数
    /// - `file_path`: 文件路径
    ///
    /// # 返回值
    /// 文件中所有注解的列表
    pub fn parse_file(&mut self, file_path: &Path) -> anyhow::Result<Vec<Annotation>> {
        let file_path_str = file_path.to_string_lossy().to_string();

        // 检查缓存
        if let Some(annotations) = self.cache.get(&file_path_str) {
            return Ok(annotations.clone());
        }

        // 读取文件内容
        let content = std::fs::read_to_string(file_path)?;

        // 解析注解
        let annotations = self.parse_content(&content, &file_path_str);

        // 缓存结果
        self.cache
            .insert(file_path_str.clone(), annotations.clone());

        Ok(annotations)
    }

    /// 解析内容中的注解
    ///
    /// # 参数
    /// - `content`: 文件内容
    /// - `file_path`: 文件路径
    ///
    /// # 返回值
    /// 所有注解的列表
    fn parse_content(&self, content: &str, file_path: &str) -> Vec<Annotation> {
        let mut annotations = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            // 查找注解行（以 #[ 开头）
            if let Some(stripped) = line.trim().strip_prefix("#[") {
                if let Some(end) = stripped.rfind(']') {
                    let annotation_content = &stripped[..end];
                    if let Some(annotation) = self.parse_annotation(annotation_content, file_path, line_num + 1) {
                        annotations.push(annotation);
                    }
                }
            }
        }

        annotations
    }

    /// 解析单个注解
    ///
    /// # 参数
    /// - `content`: 注解内容（不含 #[ 和 ]）
    /// - `file_path`: 文件路径
    /// - `line_number`: 行号
    ///
    /// # 返回值
    /// 解析后的注解
    fn parse_annotation(
        &self,
        content: &str,
        file_path: &str,
        line_number: usize,
    ) -> Option<Annotation> {
        // 提取注解名称
        let name_end = content.find('(').unwrap_or(content.len());
        let name = content[..name_end].trim();

        // 解析注解类型
        let annotation_type = AnnotationType::from_str(name);
        if matches!(annotation_type, AnnotationType::Unknown(_)) {
            return None;
        }

        // 解析参数
        let params = if name_end < content.len() {
            let params_str = &content[name_end..];
            self.parse_params(params_str)
        } else {
            AnnotationParams::new()
        };

        Some(
            Annotation::new(annotation_type, params)
                .with_file(file_path)
                .with_line(line_number),
        )
    }

    /// 解析注解参数
    ///
    /// # 参数
    /// - `content`: 参数内容（包含括号）
    ///
    /// # 返回值
    /// 解析后的参数
    fn parse_params(&self, content: &str) -> AnnotationParams {
        let mut params = AnnotationParams::new();

        // 去除括号
        let content = content.trim();
        if !content.starts_with('(') || !content.ends_with(')') {
            return params;
        }

        let content = &content[1..content.len() - 1];

        // 解析参数列表
        // 支持格式: name: 'value', name: 123, 'positional_value'
        let mut current = String::new();
        let mut in_string = false;
        let mut string_char = ' ';
        let mut depth = 0;

        for ch in content.chars() {
            match ch {
                '\'' | '"' if depth == 0 => {
                    if in_string && ch == string_char {
                        in_string = false;
                    } else if !in_string {
                        in_string = true;
                        string_char = ch;
                    }
                    current.push(ch);
                }
                '[' | '(' | '{' => {
                    depth += 1;
                    current.push(ch);
                }
                ']' | ')' | '}' => {
                    depth -= 1;
                    current.push(ch);
                }
                ',' if !in_string && depth == 0 => {
                    // 参数分隔符
                    self.parse_single_param(&current.trim(), &mut params);
                    current.clear();
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        // 处理最后一个参数
        if !current.trim().is_empty() {
            self.parse_single_param(&current.trim(), &mut params);
        }

        params
    }

    /// 解析单个参数
    ///
    /// # 参数
    /// - `param`: 参数字符串
    /// - `params`: 参数集合
    fn parse_single_param(&self, param: &str, params: &mut AnnotationParams) {
        // 检查是否是命名参数 (name: value)
        if let Some(colon_pos) = param.find(':') {
            let name = param[..colon_pos].trim();
            let value = param[colon_pos + 1..].trim();

            // 去除尾部逗号
            let value = value.trim_end_matches(',');

            params.named.insert(name.to_string(), value.to_string());
        } else {
            // 位置参数
            let value = param.trim_end_matches(',');
            params.positional.push(value.to_string());
        }
    }

    /// 扫描目录中的所有服务注解
    ///
    /// # 参数
    /// - `dir`: 目录路径
    ///
    /// # 返回值
    /// 所有服务注解的列表
    pub fn scan_directory(&mut self, dir: &Path) -> anyhow::Result<Vec<ServiceAnnotation>> {
        let mut services = Vec::new();

        if !dir.exists() {
            return Ok(services);
        }

        // 递归扫描目录
        self.scan_directory_recursive(dir, &mut services)?;

        Ok(services)
    }

    /// 递归扫描目录
    fn scan_directory_recursive(
        &mut self,
        dir: &Path,
        services: &mut Vec<ServiceAnnotation>,
    ) -> anyhow::Result<()> {
        let entries = std::fs::read_dir(dir)?;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // 递归扫描子目录
                self.scan_directory_recursive(&path, services)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("php") {
                // 解析 PHP 文件
                let annotations = self.parse_file(&path)?;

                // 提取服务注解
                for annotation in annotations {
                    if annotation.annotation_type == AnnotationType::Service {
                        if let Some(service) = self.extract_service_annotation(&annotation, &path)
                        {
                            services.push(service);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 从注解提取服务信息
    fn extract_service_annotation(
        &self,
        annotation: &Annotation,
        file_path: &Path,
    ) -> Option<ServiceAnnotation> {
        // 获取服务名称
        let name = annotation.params.get_string("name")?;

        // 获取是否启用
        let enable = annotation.params.get_bool("enable").unwrap_or(true);

        // 获取是否延迟加载
        let deferred = annotation.params.get_bool("deferred").unwrap_or(true);

        // 获取配置
        let config = self.parse_config_param(annotation.params.get("config"));

        // 提取类名（从文件路径）
        let class_name = file_path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Some(ServiceAnnotation {
            name,
            enable,
            config,
            deferred,
            class_name,
            file_path: file_path.to_string_lossy().to_string(),
        })
    }

    /// 解析配置参数
    fn parse_config_param(&self, config_str: Option<&String>) -> ServiceConfig {
        let mut config = HashMap::new();

        if let Some(s) = config_str {
            // 解析数组格式的配置 ['key' => 'value', ...]
            let s = s.trim();

            if s.starts_with('[') && s.ends_with(']') {
                let content = &s[1..s.len() - 1];

                // 简单解析键值对
                for pair in content.split(',') {
                    let pair = pair.trim();
                    if let Some(arrow_pos) = pair.find("=>") {
                        let key = pair[..arrow_pos].trim();
                        let value = pair[arrow_pos + 2..].trim();

                        // 去除引号
                        let key = if (key.starts_with('\'') && key.ends_with('\''))
                            || (key.starts_with('"') && key.ends_with('"'))
                        {
                            &key[1..key.len() - 1]
                        } else {
                            key
                        };

                        // 解析值
                        let json_value = if (value.starts_with('\'') && value.ends_with('\''))
                            || (value.starts_with('"') && value.ends_with('"'))
                        {
                            serde_json::Value::String(value[1..value.len() - 1].to_string())
                        } else if let Ok(i) = value.parse::<i64>() {
                            serde_json::Value::Number(i.into())
                        } else if let Ok(b) = value.parse::<bool>() {
                            serde_json::Value::Bool(b)
                        } else {
                            serde_json::Value::String(value.to_string())
                        };

                        config.insert(key.to_string(), json_value);
                    }
                }
            }
        }

        config
    }

    /// 清除缓存
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for AnnotationParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_annotation_type() {
        assert_eq!(
            AnnotationType::from_str("Service"),
            AnnotationType::Service
        );
        assert_eq!(
            AnnotationType::from_str("Oyta\\Attributes\\Service"),
            AnnotationType::Service
        );
        assert_eq!(
            AnnotationType::from_str("QueueJob"),
            AnnotationType::QueueJob
        );
    }

    #[test]
    fn test_parse_params() {
        let parser = AnnotationParser::new();
        let params = parser.parse_params("(name: 'cache', enable: true, port: 9501)");

        assert_eq!(params.get_string("name"), Some("cache".to_string()));
        assert_eq!(params.get_bool("enable"), Some(true));
        assert_eq!(params.get_int("port"), Some(9501));
    }

    #[test]
    fn test_parse_content() {
        let parser = AnnotationParser::new();
        let content = r#"<?php
#[Service(name: 'websocket', enable: true)]
class WebSocketService {
    #[WebSocketHandler(event: 'onMessage')]
    public function handleMessage($data) {}
}
"#;

        let annotations = parser.parse_content(content, "test.php");

        assert_eq!(annotations.len(), 2);
        assert_eq!(annotations[0].annotation_type, AnnotationType::Service);
        assert_eq!(
            annotations[1].annotation_type,
            AnnotationType::WebSocketHandler
        );
    }
}
