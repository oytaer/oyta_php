//! 错误报告器模块
//!
//! 提供错误信息的格式化和输出功能
//! 支持多种输出格式：文本、HTML、JSON、日志等

use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Write;

use super::level::ErrorLevel;
use super::types::{ErrorInfo, ExceptionInfo, StackFrame, ValueSnapshot};

/// 错误报告器
///
/// 负责格式化和输出错误信息
pub struct ErrorReporter {
    /// 报告器配置
    config: ReporterConfig,
}

/// 报告器配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReporterConfig {
    /// 是否显示错误详情
    pub display_errors: bool,
    /// 是否记录错误日志
    pub log_errors: bool,
    /// 错误日志文件路径
    pub error_log: Option<String>,
    /// 是否显示源代码上下文
    pub show_source_context: bool,
    /// 源代码上下文行数
    pub source_context_lines: usize,
    /// 是否显示堆栈跟踪
    pub show_stack_trace: bool,
    /// 是否显示参数值
    pub show_args: bool,
    /// HTML 报告的 CSS 样式
    pub html_style: Option<String>,
    /// 是否使用彩色输出
    pub use_colors: bool,
}

impl Default for ReporterConfig {
    fn default() -> Self {
        Self {
            display_errors: true,
            log_errors: true,
            error_log: None,
            show_source_context: true,
            source_context_lines: 5,
            show_stack_trace: true,
            show_args: true,
            html_style: None,
            use_colors: true,
        }
    }
}

impl ErrorReporter {
    /// 创建新的错误报告器
    pub fn new() -> Self {
        Self {
            config: ReporterConfig::default(),
        }
    }
    
    /// 使用配置创建报告器
    pub fn with_config(config: ReporterConfig) -> Self {
        Self { config }
    }
    
    /// 获取配置
    pub fn config(&self) -> &ReporterConfig {
        &self.config
    }
    
    /// 获取可变配置
    pub fn config_mut(&mut self) -> &mut ReporterConfig {
        &mut self.config
    }
    
    /// 报告错误
    ///
    /// # 参数
    /// - `error`: 错误信息
    /// - `output`: 输出类型
    ///
    /// # 返回
    /// 格式化后的错误字符串
    pub fn report_error(&self, error: &ErrorInfo, output: OutputType) -> String {
        match output {
            OutputType::Text => self.format_error_text(error),
            OutputType::Html => self.format_error_html(error),
            OutputType::Json => self.format_error_json(error),
            OutputType::Log => self.format_error_log(error),
        }
    }
    
    /// 报告异常
    ///
    /// # 参数
    /// - `exception`: 异常信息
    /// - `output`: 输出类型
    ///
    /// # 返回
    /// 格式化后的异常字符串
    pub fn report_exception(&self, exception: &ExceptionInfo, output: OutputType) -> String {
        match output {
            OutputType::Text => self.format_exception_text(exception),
            OutputType::Html => self.format_exception_html(exception),
            OutputType::Json => self.format_exception_json(exception),
            OutputType::Log => self.format_exception_log(exception),
        }
    }
    
    /// 格式化错误为文本
    fn format_error_text(&self, error: &ErrorInfo) -> String {
        let mut output = String::new();
        
        // 错误级别和消息
        let level_str = if self.config.use_colors {
            self.colorize_level(error.level)
        } else {
            error.level.name().to_string()
        };
        
        output.push_str(&format!(
            "{}: {} in {} on line {}",
            level_str,
            error.message,
            error.file,
            error.line
        ));
        
        // 添加列号（如果有）
        if let Some(col) = error.column {
            output.push_str(&format!(" column {}", col));
        }
        
        output.push('\n');
        
        // 添加源代码上下文
        if self.config.show_source_context {
            if let Some(context) = self.get_source_context(&error.file, error.line) {
                output.push_str("\nSource:\n");
                output.push_str(&context);
            }
        }
        
        // 添加堆栈跟踪
        if self.config.show_stack_trace {
            if let Some(trace) = &error.trace {
                if !trace.is_empty() {
                    output.push_str("\nStack trace:\n");
                    for (i, frame) in trace.iter().enumerate() {
                        output.push_str(&format!("#{} {}\n", i, frame));
                    }
                }
            }
        }
        
        output
    }
    
    /// 格式化错误为 HTML
    fn format_error_html(&self, error: &ErrorInfo) -> String {
        let level_class = error.level.name().to_lowercase();
        
        let mut html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{level} - OYTAPHP Error</title>
    <style>
        {style}
    </style>
</head>
<body>
    <div class="error-container">
        <div class="error-header error-{level_class}">
            <h1>{level}</h1>
            <p class="error-message">{message}</p>
        </div>
        <div class="error-location">
            <strong>File:</strong> <code>{file}</code><br>
            <strong>Line:</strong> {line}
        </div>"#,
            level = error.level.name(),
            level_class = level_class,
            style = self.get_html_style(),
            message = self.escape_html(&error.message),
            file = self.escape_html(&error.file),
            line = error.line,
        );
        
        // 添加源代码上下文
        if self.config.show_source_context {
            if let Some(context) = self.get_source_context(&error.file, error.line) {
                html.push_str(&format!(
                    r#"<div class="error-source">
            <pre>{}</pre>
        </div>"#,
                    self.escape_html(&context)
                ));
            }
        }
        
        // 添加堆栈跟踪
        if self.config.show_stack_trace {
            if let Some(trace) = &error.trace {
                if !trace.is_empty() {
                    html.push_str("<div class=\"error-trace\"><h2>Stack Trace</h2><ol>");
                    for frame in trace {
                        html.push_str(&format!(
                            "<li>{}</li>",
                            self.escape_html(&frame.to_string())
                        ));
                    }
                    html.push_str("</ol></div>");
                }
            }
        }
        
        html.push_str("</div></body></html>");
        html
    }
    
    /// 格式化错误为 JSON
    fn format_error_json(&self, error: &ErrorInfo) -> String {
        serde_json::to_string_pretty(&error).unwrap_or_else(|_| "{}".to_string())
    }
    
    /// 格式化错误为日志格式
    fn format_error_log(&self, error: &ErrorInfo) -> String {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        format!(
            "[{}] {}: {} in {} on line {}",
            timestamp,
            error.level.name(),
            error.message,
            error.file,
            error.line
        )
    }
    
    /// 格式化异常为文本
    fn format_exception_text(&self, exception: &ExceptionInfo) -> String {
        let mut output = String::new();
        
        // 异常类型和消息
        output.push_str(&format!(
            "Uncaught {}: {}",
            exception.class_name,
            exception.message
        ));
        
        // 添加位置
        output.push_str(&format!(
            " in {}:{}",
            exception.file,
            exception.line
        ));
        
        output.push('\n');
        
        // 添加堆栈跟踪
        if self.config.show_stack_trace && !exception.trace.is_empty() {
            output.push_str("\nStack trace:\n");
            for (i, frame) in exception.trace.iter().enumerate() {
                output.push_str(&format!("#{} {}\n", i, frame));
            }
        }
        
        // 添加异常链
        if let Some(prev) = &exception.previous {
            output.push_str("\nCaused by:\n");
            output.push_str(&self.format_exception_text(prev));
        }
        
        output
    }
    
    /// 格式化异常为 HTML
    fn format_exception_html(&self, exception: &ExceptionInfo) -> String {
        let mut html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Uncaught {class} - OYTAPHP Exception</title>
    <style>{style}</style>
</head>
<body>
    <div class="error-container">
        <div class="error-header error-exception">
            <h1>Uncaught {class}</h1>
            <p class="error-message">{message}</p>
        </div>
        <div class="error-location">
            <strong>File:</strong> <code>{file}</code><br>
            <strong>Line:</strong> {line}
        </div>"#,
            class = exception.class_name,
            style = self.get_html_style(),
            message = self.escape_html(&exception.message),
            file = self.escape_html(&exception.file),
            line = exception.line,
        );
        
        // 添加堆栈跟踪
        if self.config.show_stack_trace && !exception.trace.is_empty() {
            html.push_str("<div class=\"error-trace\"><h2>Stack Trace</h2><ol>");
            for frame in &exception.trace {
                html.push_str(&format!(
                    "<li>{}</li>",
                    self.escape_html(&frame.to_string())
                ));
            }
            html.push_str("</ol></div>");
        }
        
        // 添加异常链
        if let Some(prev) = &exception.previous {
            html.push_str("<div class=\"error-chain\"><h2>Caused by</h2>");
            html.push_str(&self.format_exception_html_inner(prev));
            html.push_str("</div>");
        }
        
        html.push_str("</div></body></html>");
        html
    }
    
    /// 格式化异常为 HTML（内部使用，不含完整 HTML 结构）
    fn format_exception_html_inner(&self, exception: &ExceptionInfo) -> String {
        format!(
            r#"<div class="error-previous">
    <strong>{}</strong>: {}<br>
    <small>in <code>{}</code> on line {}</small>
</div>"#,
            exception.class_name,
            self.escape_html(&exception.message),
            self.escape_html(&exception.file),
            exception.line
        )
    }
    
    /// 格式化异常为 JSON
    fn format_exception_json(&self, exception: &ExceptionInfo) -> String {
        serde_json::to_string_pretty(&exception).unwrap_or_else(|_| "{}".to_string())
    }
    
    /// 格式化异常为日志格式
    fn format_exception_log(&self, exception: &ExceptionInfo) -> String {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        format!(
            "[{}] Uncaught {}: {} in {}:{}",
            timestamp,
            exception.class_name,
            exception.message,
            exception.file,
            exception.line
        )
    }
    
    /// 获取源代码上下文
    fn get_source_context(&self, file: &str, line: usize) -> Option<String> {
        // 读取文件内容
        let content = std::fs::read_to_string(file).ok()?;
        let lines: Vec<&str> = content.lines().collect();
        
        // 计算上下文范围
        let start = line.saturating_sub(self.config.source_context_lines);
        let end = (line + self.config.source_context_lines).min(lines.len());
        
        let mut context = String::new();
        for i in start..end {
            let line_num = i + 1;
            let marker = if line_num == line { ">>>" } else { "   " };
            context.push_str(&format!("{} {:4} | {}\n", marker, line_num, lines[i]));
        }
        
        Some(context)
    }
    
    /// 获取 HTML 样式
    fn get_html_style(&self) -> String {
        if let Some(style) = &self.config.html_style {
            style.clone()
        } else {
            r#"
body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: #f5f5f5;
    margin: 0;
    padding: 20px;
}
.error-container {
    max-width: 1000px;
    margin: 0 auto;
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
    overflow: hidden;
}
.error-header {
    padding: 20px;
    color: white;
}
.error-header h1 {
    margin: 0 0 10px 0;
    font-size: 24px;
}
.error-message {
    margin: 0;
    font-size: 16px;
    opacity: 0.9;
}
.error-error { background: #dc3545; }
.error-warning { background: #ffc107; color: #333; }
.error-notice { background: #17a2b8; }
.error-exception { background: #dc3545; }
.error-location {
    padding: 15px 20px;
    background: #f8f9fa;
    border-bottom: 1px solid #e9ecef;
}
.error-source {
    padding: 15px 20px;
    background: #282c34;
    color: #abb2bf;
    font-family: "Fira Code", monospace;
    font-size: 14px;
    overflow-x: auto;
}
.error-source pre {
    margin: 0;
}
.error-trace {
    padding: 20px;
}
.error-trace h2 {
    margin: 0 0 15px 0;
    font-size: 18px;
}
.error-trace ol {
    margin: 0;
    padding-left: 20px;
}
.error-trace li {
    margin-bottom: 10px;
    font-family: monospace;
    font-size: 13px;
}
.error-chain {
    padding: 20px;
    border-top: 1px solid #e9ecef;
}
.error-chain h2 {
    margin: 0 0 15px 0;
    font-size: 18px;
}
"#.to_string()
        }
    }
    
    /// HTML 转义
    fn escape_html(&self, s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#039;")
    }
    
    /// 为错误级别添加颜色
    fn colorize_level(&self, level: ErrorLevel) -> String {
        // ANSI 颜色代码
        const RED: &str = "\x1b[31m";
        const YELLOW: &str = "\x1b[33m";
        const CYAN: &str = "\x1b[36m";
        const RESET: &str = "\x1b[0m";
        
        let color = if level.is_fatal() {
            RED
        } else if level.is_warning() {
            YELLOW
        } else {
            CYAN
        };
        
        format!("{}{}{}", color, level.name(), RESET)
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// 输出类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    /// 纯文本
    Text,
    /// HTML 格式
    Html,
    /// JSON 格式
    Json,
    /// 日志格式
    Log,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn get_test_error() -> ErrorInfo {
        ErrorInfo::new(
            ErrorLevel::E_WARNING,
            "Test warning message".to_string(),
            "test.php".to_string(),
            42,
        )
    }
    
    fn get_test_exception() -> ExceptionInfo {
        ExceptionInfo::new(
            "RuntimeException".to_string(),
            "Something went wrong".to_string(),
            "app.php".to_string(),
            100,
        )
    }
    
    #[test]
    fn test_report_error_text() {
        let reporter = ErrorReporter::new();
        let error = get_test_error();
        
        let text = reporter.report_error(&error, OutputType::Text);
        assert!(text.contains("E_WARNING"));
        assert!(text.contains("Test warning message"));
        assert!(text.contains("test.php"));
        assert!(text.contains("42"));
    }
    
    #[test]
    fn test_report_error_html() {
        let reporter = ErrorReporter::new();
        let error = get_test_error();
        
        let html = reporter.report_error(&error, OutputType::Html);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("E_WARNING"));
        assert!(html.contains("Test warning message"));
    }
    
    #[test]
    fn test_report_error_json() {
        let reporter = ErrorReporter::new();
        let error = get_test_error();
        
        let json = reporter.report_error(&error, OutputType::Json);
        assert!(json.contains("\"level\""));
        assert!(json.contains("\"message\""));
    }
    
    #[test]
    fn test_report_exception_text() {
        let reporter = ErrorReporter::new();
        let exception = get_test_exception();
        
        let text = reporter.report_exception(&exception, OutputType::Text);
        assert!(text.contains("Uncaught RuntimeException"));
        assert!(text.contains("Something went wrong"));
    }
    
    #[test]
    fn test_reporter_config() {
        let config = ReporterConfig {
            display_errors: false,
            log_errors: true,
            error_log: Some("/var/log/php_errors.log".to_string()),
            show_source_context: false,
            source_context_lines: 3,
            show_stack_trace: true,
            show_args: false,
            html_style: None,
            use_colors: false,
        };
        
        let reporter = ErrorReporter::with_config(config);
        
        assert!(!reporter.config().display_errors);
        assert!(reporter.config().log_errors);
        assert_eq!(reporter.config().error_log, Some("/var/log/php_errors.log".to_string()));
    }
}
