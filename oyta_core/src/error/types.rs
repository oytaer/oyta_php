//! 错误类型定义模块
//!
//! 定义错误信息、异常信息等核心类型

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use super::level::ErrorLevel;

/// 错误信息结构
///
/// 包含一个错误的完整信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// 错误级别
    pub level: ErrorLevel,
    /// 错误消息
    pub message: String,
    /// 发生错误的文件路径
    pub file: String,
    /// 发生错误的行号
    pub line: usize,
    /// 发生错误的列号（可选）
    pub column: Option<usize>,
    /// 错误发生时的上下文变量
    /// 用于调试目的
    pub context: HashMap<String, ValueSnapshot>,
    /// 错误堆栈跟踪（可选）
    pub trace: Option<Vec<StackFrame>>,
    /// 错误时间戳
    pub timestamp: i64,
    /// 错误唯一标识
    pub id: String,
}

impl ErrorInfo {
    /// 创建新的错误信息
    ///
    /// # 参数
    /// - `level`: 错误级别
    /// - `message`: 错误消息
    /// - `file`: 文件路径
    /// - `line`: 行号
    ///
    /// # 返回
    /// 新的 ErrorInfo 实例
    pub fn new(level: ErrorLevel, message: String, file: String, line: usize) -> Self {
        Self {
            level,
            message,
            file,
            line,
            column: None,
            context: HashMap::new(),
            trace: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            id: Self::generate_id(),
        }
    }
    
    /// 创建带列号的错误信息
    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }
    
    /// 添加上下文变量
    pub fn with_context(mut self, name: String, value: ValueSnapshot) -> Self {
        self.context.insert(name, value);
        self
    }
    
    /// 设置堆栈跟踪
    pub fn with_trace(mut self, trace: Vec<StackFrame>) -> Self {
        self.trace = Some(trace);
        self
    }
    
    /// 生成唯一错误 ID
    fn generate_id() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        format!("err_{}", COUNTER.fetch_add(1, Ordering::Relaxed))
    }
    
    /// 格式化为字符串
    pub fn to_string_formatted(&self) -> String {
        let level_name = self.level.name();
        let column_str = self.column.map(|c| format!(":{}", c)).unwrap_or_default();
        
        format!(
            "{}: {} in {} on line {}{}",
            level_name,
            self.message,
            self.file,
            self.line,
            column_str
        )
    }
    
    /// 格式化为 HTML
    pub fn to_html(&self) -> String {
        format!(
            r#"<div class="error error-{}">
    <strong>{}</strong>: {}<br>
    <small>in <code>{}</code> on line {}</small>
</div>"#,
            self.level.name().to_lowercase(),
            self.level.name(),
            self.escape_html(&self.message),
            self.escape_html(&self.file),
            self.line
        )
    }
    
    /// HTML 转义
    fn escape_html(&self, s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#039;")
    }
}

impl fmt::Display for ErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_formatted())
    }
}

/// 异常信息结构
///
/// 包含一个异常的完整信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExceptionInfo {
    /// 异常类名
    pub class_name: String,
    /// 异常消息
    pub message: String,
    /// 异常代码
    pub code: i32,
    /// 发生异常的文件路径
    pub file: String,
    /// 发生异常的行号
    pub line: usize,
    /// 堆栈跟踪
    pub trace: Vec<StackFrame>,
    /// 前一个异常（用于异常链）
    pub previous: Option<Box<ExceptionInfo>>,
    /// 异常时间戳
    pub timestamp: i64,
    /// 异常唯一标识
    pub id: String,
}

impl ExceptionInfo {
    /// 创建新的异常信息
    ///
    /// # 参数
    /// - `class_name`: 异常类名
    /// - `message`: 异常消息
    /// - `file`: 文件路径
    /// - `line`: 行号
    ///
    /// # 返回
    /// 新的 ExceptionInfo 实例
    pub fn new(class_name: String, message: String, file: String, line: usize) -> Self {
        Self {
            class_name,
            message,
            code: 0,
            file,
            line,
            trace: Vec::new(),
            previous: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            id: Self::generate_id(),
        }
    }
    
    /// 创建带错误码的异常
    pub fn with_code(mut self, code: i32) -> Self {
        self.code = code;
        self
    }
    
    /// 设置堆栈跟踪
    pub fn with_trace(mut self, trace: Vec<StackFrame>) -> Self {
        self.trace = trace;
        self
    }
    
    /// 设置前一个异常
    pub fn with_previous(mut self, previous: ExceptionInfo) -> Self {
        self.previous = Some(Box::new(previous));
        self
    }
    
    /// 生成唯一异常 ID
    fn generate_id() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        format!("exc_{}", COUNTER.fetch_add(1, Ordering::Relaxed))
    }
    
    /// 格式化为字符串
    pub fn to_string_formatted(&self) -> String {
        let mut result = format!(
            "Uncaught {}: {} in {}:{}",
            self.class_name,
            self.message,
            self.file,
            self.line
        );
        
        // 添加堆栈跟踪
        if !self.trace.is_empty() {
            result.push_str("\nStack trace:\n");
            for (i, frame) in self.trace.iter().enumerate() {
                result.push_str(&format!("#{} {}\n", i, frame));
            }
        }
        
        // 添加前一个异常
        if let Some(prev) = &self.previous {
            result.push_str(&format!("\n  caused by: {}", prev.to_string_formatted()));
        }
        
        result
    }
    
    /// 获取完整的异常链
    pub fn get_chain(&self) -> Vec<&ExceptionInfo> {
        let mut chain = vec![self];
        let mut current = self.previous.as_ref();
        
        while let Some(exc) = current {
            chain.push(exc);
            current = exc.previous.as_ref();
        }
        
        chain
    }
}

impl fmt::Display for ExceptionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_formatted())
    }
}

/// 堆栈帧
///
/// 表示调用堆栈中的一帧
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StackFrame {
    /// 函数名或方法名
    pub function: String,
    /// 类名（如果是方法调用）
    pub class: Option<String>,
    /// 调用类型（-> 或 ::）
    pub call_type: Option<String>,
    /// 参数列表
    pub args: Vec<ValueSnapshot>,
    /// 文件路径
    pub file: Option<String>,
    /// 行号
    pub line: Option<usize>,
}

impl StackFrame {
    /// 创建新的堆栈帧
    pub fn new(function: String) -> Self {
        Self {
            function,
            class: None,
            call_type: None,
            args: Vec::new(),
            file: None,
            line: None,
        }
    }
    
    /// 设置类名和调用类型
    pub fn with_class(mut self, class: String, is_static: bool) -> Self {
        self.class = Some(class);
        self.call_type = Some(if is_static { "::".to_string() } else { "->".to_string() });
        self
    }
    
    /// 设置参数
    pub fn with_args(mut self, args: Vec<ValueSnapshot>) -> Self {
        self.args = args;
        self
    }
    
    /// 设置文件和行号
    pub fn with_location(mut self, file: String, line: usize) -> Self {
        self.file = Some(file);
        self.line = Some(line);
        self
    }
}

impl fmt::Display for StackFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 格式：file(line): class->function(args)
        
        // 文件和行号
        if let (Some(file), Some(line)) = (&self.file, self.line) {
            write!(f, "{}({}): ", file, line)?;
        }
        
        // 类和方法
        if let (Some(class), Some(call_type)) = (&self.class, &self.call_type) {
            write!(f, "{}{}", class, call_type)?;
        }
        
        // 函数名
        write!(f, "{}(", self.function)?;
        
        // 参数
        let args_str: Vec<String> = self.args.iter().map(|a| a.to_string()).collect();
        write!(f, "{})", args_str.join(", "))
    }
}

/// 值快照
///
/// 用于在错误发生时捕获变量的值
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueSnapshot {
    /// 值的类型
    pub type_name: String,
    /// 值的字符串表示
    pub value_string: String,
    /// 值的长度（对于字符串和数组）
    pub length: Option<usize>,
}

impl ValueSnapshot {
    /// 从值创建快照
    pub fn from_type_and_value(type_name: &str, value_string: &str) -> Self {
        Self {
            type_name: type_name.to_string(),
            value_string: value_string.to_string(),
            length: None,
        }
    }
    
    /// 创建带长度的快照
    pub fn with_length(mut self, length: usize) -> Self {
        self.length = Some(length);
        self
    }
    
    /// 创建 null 快照
    pub fn null() -> Self {
        Self::from_type_and_value("null", "NULL")
    }
    
    /// 创建布尔快照
    pub fn bool(value: bool) -> Self {
        Self::from_type_and_value("bool", if value { "true" } else { "false" })
    }
    
    /// 创建整数快照
    pub fn int(value: i64) -> Self {
        Self::from_type_and_value("int", &value.to_string())
    }
    
    /// 创建浮点数快照
    pub fn float(value: f64) -> Self {
        Self::from_type_and_value("float", &value.to_string())
    }
    
    /// 创建字符串快照
    pub fn string(value: &str) -> Self {
        let snapshot = Self::from_type_and_value("string", value);
        snapshot.with_length(value.len())
    }
    
    /// 创建数组快照
    pub fn array(length: usize) -> Self {
        Self::from_type_and_value("array", "Array").with_length(length)
    }
    
    /// 创建对象快照
    pub fn object(class_name: &str) -> Self {
        Self::from_type_and_value("object", &format!("{} Object", class_name))
    }
    
    /// 创建资源快照
    pub fn resource(resource_type: &str) -> Self {
        Self::from_type_and_value("resource", &format!("Resource({})", resource_type))
    }
}

impl fmt::Display for ValueSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.length {
            Some(len) => write!(f, "{}({}) {}", self.type_name, len, self.value_string),
            None => write!(f, "{} {}", self.type_name, self.value_string),
        }
    }
}

/// 错误处理结果
///
/// 表示错误处理后的状态
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorHandlingResult {
    /// 继续执行
    Continue,
    /// 终止执行
    Halt,
    /// 抛出异常
    ThrowException(ExceptionInfo),
}

/// 异常处理结果
#[derive(Debug, Clone, PartialEq)]
pub enum ExceptionHandlingResult {
    /// 已处理，继续执行
    Handled,
    /// 未处理，需要继续传播
    Unhandled,
    /// 替换为新的异常
    Replace(ExceptionInfo),
}

// ============================================================================
// 数据库异常类型
// ============================================================================

/// 模型未找到异常
///
/// 当使用 findOrFail、firstOrFail 等方法查询数据时
/// 如果未找到匹配的记录，则抛出此异常
///
/// 对应 ThinkPHP 8.0 的 \think\exception\ModelNotFoundException
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelNotFoundException {
    /// 模型类名
    pub model: String,
    /// 查询条件
    pub query: Option<String>,
    /// 异常消息
    pub message: String,
    /// 错误代码
    pub code: i32,
    /// 发生异常的文件路径
    pub file: String,
    /// 发生异常的行号
    pub line: usize,
}

impl ModelNotFoundException {
    /// 创建新的模型未找到异常
    ///
    /// # 参数
    /// - `model`: 模型类名
    /// - `query`: 查询条件描述
    ///
    /// # 返回
    /// 新的 ModelNotFoundException 实例
    pub fn new(model: &str, query: Option<&str>) -> Self {
        let message = match query {
            Some(q) => format!("模型 [{}] 未找到，查询条件: {}", model, q),
            None => format!("模型 [{}] 未找到", model),
        };

        Self {
            model: model.to_string(),
            query: query.map(|s| s.to_string()),
            message,
            code: 0,
            file: String::new(),
            line: 0,
        }
    }

    /// 设置文件和行号
    pub fn with_location(mut self, file: &str, line: usize) -> Self {
        self.file = file.to_string();
        self.line = line;
        self
    }

    /// 设置错误代码
    pub fn with_code(mut self, code: i32) -> Self {
        self.code = code;
        self
    }

    /// 转换为 ExceptionInfo
    pub fn to_exception_info(&self) -> ExceptionInfo {
        ExceptionInfo::new(
            "ModelNotFoundException".to_string(),
            self.message.clone(),
            self.file.clone(),
            self.line,
        )
        .with_code(self.code)
    }
}

impl fmt::Display for ModelNotFoundException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ModelNotFoundException: {}", self.message)
    }
}

impl std::error::Error for ModelNotFoundException {}

/// 数据库查询异常
///
/// 当执行 SQL 查询时发生错误，则抛出此异常
/// 包含详细的错误信息和 SQL 语句
///
/// 对应 ThinkPHP 8.0 的 \think\exception\DbException
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryException {
    /// SQL 语句
    pub sql: String,
    /// 错误消息
    pub message: String,
    /// 错误代码
    pub code: i32,
    /// 数据库错误码
    pub sql_state: Option<String>,
    /// 发生异常的文件路径
    pub file: String,
    /// 发生异常的行号
    pub line: usize,
    /// 绑定参数
    pub bindings: Vec<String>,
}

impl QueryException {
    /// 创建新的查询异常
    ///
    /// # 参数
    /// - `sql`: 执行的 SQL 语句
    /// - `message`: 错误消息
    ///
    /// # 返回
    /// 新的 QueryException 实例
    pub fn new(sql: &str, message: &str) -> Self {
        Self {
            sql: sql.to_string(),
            message: message.to_string(),
            code: 0,
            sql_state: None,
            file: String::new(),
            line: 0,
            bindings: Vec::new(),
        }
    }

    /// 设置 SQL 状态码
    pub fn with_sql_state(mut self, sql_state: &str) -> Self {
        self.sql_state = Some(sql_state.to_string());
        self
    }

    /// 设置错误代码
    pub fn with_code(mut self, code: i32) -> Self {
        self.code = code;
        self
    }

    /// 设置文件和行号
    pub fn with_location(mut self, file: &str, line: usize) -> Self {
        self.file = file.to_string();
        self.line = line;
        self
    }

    /// 设置绑定参数
    pub fn with_bindings(mut self, bindings: Vec<String>) -> Self {
        self.bindings = bindings;
        self
    }

    /// 转换为 ExceptionInfo
    pub fn to_exception_info(&self) -> ExceptionInfo {
        let mut full_message = format!("SQL 执行错误: {}\nSQL: {}", self.message, self.sql);

        if !self.bindings.is_empty() {
            full_message.push_str(&format!("\n绑定参数: [{}]", self.bindings.join(", ")));
        }

        if let Some(ref sql_state) = self.sql_state {
            full_message.push_str(&format!("\nSQLSTATE: {}", sql_state));
        }

        ExceptionInfo::new(
            "QueryException".to_string(),
            full_message,
            self.file.clone(),
            self.line,
        )
        .with_code(self.code)
    }
}

impl fmt::Display for QueryException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QueryException: {} (SQL: {})", self.message, self.sql)
    }
}

impl std::error::Error for QueryException {}

/// 事务异常
///
/// 当事务操作失败时抛出此异常
/// 包含事务操作的详细信息和失败原因
///
/// 对应 ThinkPHP 8.0 的事务相关异常
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionException {
    /// 操作类型（begin, commit, rollback）
    pub operation: String,
    /// 错误消息
    pub message: String,
    /// 错误代码
    pub code: i32,
    /// 发生异常的文件路径
    pub file: String,
    /// 发生异常的行号
    pub line: usize,
    /// 连接名称
    pub connection: Option<String>,
}

impl TransactionException {
    /// 创建新的事务异常
    ///
    /// # 参数
    /// - `operation`: 操作类型（begin, commit, rollback）
    /// - `message`: 错误消息
    ///
    /// # 返回
    /// 新的 TransactionException 实例
    pub fn new(operation: &str, message: &str) -> Self {
        Self {
            operation: operation.to_string(),
            message: message.to_string(),
            code: 0,
            file: String::new(),
            line: 0,
            connection: None,
        }
    }

    /// 设置连接名称
    pub fn with_connection(mut self, connection: &str) -> Self {
        self.connection = Some(connection.to_string());
        self
    }

    /// 设置错误代码
    pub fn with_code(mut self, code: i32) -> Self {
        self.code = code;
        self
    }

    /// 设置文件和行号
    pub fn with_location(mut self, file: &str, line: usize) -> Self {
        self.file = file.to_string();
        self.line = line;
        self
    }

    /// 转换为 ExceptionInfo
    pub fn to_exception_info(&self) -> ExceptionInfo {
        let mut full_message = format!("事务操作失败 [{}]: {}", self.operation, self.message);

        if let Some(ref connection) = self.connection {
            full_message.push_str(&format!("\n连接: {}", connection));
        }

        ExceptionInfo::new(
            "TransactionException".to_string(),
            full_message,
            self.file.clone(),
            self.line,
        )
        .with_code(self.code)
    }
}

impl fmt::Display for TransactionException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TransactionException: {} (操作: {})",
            self.message, self.operation
        )
    }
}

impl std::error::Error for TransactionException {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_info_creation() {
        let error = ErrorInfo::new(
            ErrorLevel::E_WARNING,
            "Test warning message".to_string(),
            "/path/to/file.php".to_string(),
            42,
        );
        
        assert_eq!(error.level, ErrorLevel::E_WARNING);
        assert_eq!(error.message, "Test warning message");
        assert_eq!(error.file, "/path/to/file.php");
        assert_eq!(error.line, 42);
    }
    
    #[test]
    fn test_error_info_format() {
        let error = ErrorInfo::new(
            ErrorLevel::E_ERROR,
            "Fatal error".to_string(),
            "test.php".to_string(),
            10,
        );
        
        let formatted = error.to_string_formatted();
        assert!(formatted.contains("E_ERROR"));
        assert!(formatted.contains("Fatal error"));
        assert!(formatted.contains("test.php"));
        assert!(formatted.contains("10"));
    }
    
    #[test]
    fn test_exception_info_creation() {
        let exc = ExceptionInfo::new(
            "RuntimeException".to_string(),
            "Something went wrong".to_string(),
            "app.php".to_string(),
            100,
        );
        
        assert_eq!(exc.class_name, "RuntimeException");
        assert_eq!(exc.message, "Something went wrong");
        assert_eq!(exc.code, 0);
    }
    
    #[test]
    fn test_exception_chain() {
        let inner = ExceptionInfo::new(
            "InvalidArgumentException".to_string(),
            "Invalid argument".to_string(),
            "lib.php".to_string(),
            50,
        );
        
        let outer = ExceptionInfo::new(
            "RuntimeException".to_string(),
            "Wrapper exception".to_string(),
            "app.php".to_string(),
            100,
        ).with_previous(inner);
        
        let chain = outer.get_chain();
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].class_name, "RuntimeException");
        assert_eq!(chain[1].class_name, "InvalidArgumentException");
    }
    
    #[test]
    fn test_stack_frame() {
        let frame = StackFrame::new("myFunction".to_string())
            .with_class("MyClass".to_string(), false)
            .with_location("src/MyClass.php".to_string(), 42);
        
        let formatted = format!("{}", frame);
        assert!(formatted.contains("MyClass"));
        assert!(formatted.contains("myFunction"));
        assert!(formatted.contains("src/MyClass.php"));
        assert!(formatted.contains("42"));
    }
    
    #[test]
    fn test_value_snapshot() {
        // 测试各种类型的快照
        let null = ValueSnapshot::null();
        assert_eq!(null.type_name, "null");
        
        let bool_val = ValueSnapshot::bool(true);
        assert_eq!(bool_val.type_name, "bool");
        assert_eq!(bool_val.value_string, "true");
        
        let int_val = ValueSnapshot::int(42);
        assert_eq!(int_val.type_name, "int");
        assert_eq!(int_val.value_string, "42");
        
        let str_val = ValueSnapshot::string("hello");
        assert_eq!(str_val.type_name, "string");
        assert_eq!(str_val.length, Some(5));
    }
}
