//! eval 执行器模块
//! 
//! 本模块实现 eval() 的核心执行功能，包括：
//! - 代码解析和执行
//! - 表达式求值
//! - 语句执行
//! - 错误处理

use std::collections::HashMap;
use crate::eval::context::{EvalContext, EvalVariable, EvalScope, EvalFunction, EvalClass, EvalParameter, EvalProperty, EvalVisibility};

/// eval 错误类型
#[derive(Debug, Clone)]
pub enum EvalError {
    /// 语法错误
    SyntaxError {
        /// 错误消息
        message: String,
        /// 行号
        line: usize,
    },
    /// 运行时错误
    RuntimeError {
        /// 错误消息
        message: String,
        /// 行号
        line: usize,
    },
    /// 类型错误
    TypeError {
        /// 错误消息
        message: String,
        /// 期望类型
        expected: String,
        /// 实际类型
        actual: String,
    },
    /// 未定义变量
    UndefinedVariable {
        /// 变量名
        name: String,
    },
    /// 未定义函数
    UndefinedFunction {
        /// 函数名
        name: String,
    },
    /// 未定义类
    UndefinedClass {
        /// 类名
        name: String,
    },
    /// 未定义常量
    UndefinedConstant {
        /// 常量名
        name: String,
    },
    /// 参数错误
    ArgumentError {
        /// 错误消息
        message: String,
    },
    /// 除零错误
    DivisionByZero,
    /// 越界访问
    OutOfBounds {
        /// 索引
        index: usize,
    },
    /// 访问被拒绝
    AccessDenied {
        /// 错误消息
        message: String,
    },
    /// 超时
    Timeout,
    /// 内存不足
    OutOfMemory,
    /// 异常抛出
    Exception {
        /// 异常类型
        exception_type: String,
        /// 异常消息
        message: String,
    },
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::SyntaxError { message, line } => {
                write!(f, "语法错误: {} 在第 {} 行", message, line)
            }
            EvalError::RuntimeError { message, line } => {
                write!(f, "运行时错误: {} 在第 {} 行", message, line)
            }
            EvalError::TypeError { message, expected, actual } => {
                write!(f, "类型错误: {}, 期望 {}, 实际 {}", message, expected, actual)
            }
            EvalError::UndefinedVariable { name } => {
                write!(f, "未定义变量: {}", name)
            }
            EvalError::UndefinedFunction { name } => {
                write!(f, "未定义函数: {}", name)
            }
            EvalError::UndefinedClass { name } => {
                write!(f, "未定义类: {}", name)
            }
            EvalError::UndefinedConstant { name } => {
                write!(f, "未定义常量: {}", name)
            }
            EvalError::ArgumentError { message } => {
                write!(f, "参数错误: {}", message)
            }
            EvalError::DivisionByZero => {
                write!(f, "除零错误")
            }
            EvalError::OutOfBounds { index } => {
                write!(f, "索引越界: {}", index)
            }
            EvalError::AccessDenied { message } => {
                write!(f, "访问被拒绝: {}", message)
            }
            EvalError::Timeout => {
                write!(f, "执行超时")
            }
            EvalError::OutOfMemory => {
                write!(f, "内存不足")
            }
            EvalError::Exception { exception_type, message } => {
                write!(f, "异常 {}: {}", exception_type, message)
            }
        }
    }
}

impl std::error::Error for EvalError {}

/// eval 执行结果
#[derive(Debug, Clone)]
pub struct EvalResult {
    /// 返回值
    pub value: Option<EvalVariable>,
    /// 输出内容
    pub output: String,
    /// 是否有返回
    pub has_return: bool,
    /// 是否有 break
    pub has_break: bool,
    /// 是否有 continue
    pub has_continue: bool,
    /// 是否抛出异常
    pub has_exception: bool,
    /// 异常信息
    pub exception: Option<EvalError>,
}

impl EvalResult {
    /// 创建空结果
    pub fn empty() -> Self {
        EvalResult {
            value: None,
            output: String::new(),
            has_return: false,
            has_break: false,
            has_continue: false,
            has_exception: false,
            exception: None,
        }
    }

    /// 创建带值的结果
    pub fn with_value(value: EvalVariable) -> Self {
        EvalResult {
            value: Some(value),
            output: String::new(),
            has_return: false,
            has_break: false,
            has_continue: false,
            has_exception: false,
            exception: None,
        }
    }

    /// 创建返回结果
    pub fn return_value(value: EvalVariable) -> Self {
        EvalResult {
            value: Some(value),
            output: String::new(),
            has_return: true,
            has_break: false,
            has_continue: false,
            has_exception: false,
            exception: None,
        }
    }

    /// 创建错误结果
    pub fn error(err: EvalError) -> Self {
        EvalResult {
            value: None,
            output: String::new(),
            has_return: false,
            has_break: false,
            has_continue: false,
            has_exception: true,
            exception: Some(err),
        }
    }

    /// 追加输出
    pub fn append_output(&mut self, output: &str) {
        self.output.push_str(output);
    }

    /// 合并结果
    pub fn merge(&mut self, other: EvalResult) {
        if let Some(value) = other.value {
            self.value = Some(value);
        }
        self.output.push_str(&other.output);
        self.has_return = other.has_return;
        self.has_break = other.has_break;
        self.has_continue = other.has_continue;
        self.has_exception = other.has_exception;
        self.exception = other.exception;
    }
}

impl Default for EvalResult {
    fn default() -> Self {
        Self::empty()
    }
}

/// eval 执行器
pub struct Eval {
    /// 执行上下文
    context: EvalContext,
    /// 最大递归深度
    max_recursion_depth: usize,
    /// 当前递归深度
    current_depth: usize,
    /// 是否允许危险函数
    allow_dangerous_functions: bool,
}

impl Eval {
    /// 创建新的执行器
    pub fn new() -> Self {
        Eval {
            context: EvalContext::new(),
            max_recursion_depth: 256,
            current_depth: 0,
            allow_dangerous_functions: false,
        }
    }

    /// 从上下文创建执行器
    pub fn with_context(context: EvalContext) -> Self {
        Eval {
            context,
            max_recursion_depth: 256,
            current_depth: 0,
            allow_dangerous_functions: false,
        }
    }

    /// 获取上下文引用
    pub fn context(&self) -> &EvalContext {
        &self.context
    }

    /// 获取上下文可变引用
    pub fn context_mut(&mut self) -> &mut EvalContext {
        &mut self.context
    }

    /// 设置最大递归深度
    pub fn set_max_recursion_depth(&mut self, depth: usize) {
        self.max_recursion_depth = depth;
    }

    /// 允许危险函数
    pub fn allow_dangerous_functions(&mut self, allow: bool) {
        self.allow_dangerous_functions = allow;
    }

    /// 执行代码字符串
    /// 
    /// # 参数
    /// - `code`: PHP 代码字符串
    /// 
    /// # 返回
    /// 返回执行结果
    pub fn eval(&mut self, code: &str) -> EvalResult {
        // 检查递归深度
        if self.current_depth >= self.max_recursion_depth {
            return EvalResult::error(EvalError::RuntimeError {
                message: "达到最大递归深度".to_string(),
                line: 0,
            });
        }
        self.current_depth += 1;
        // 解析代码
        let result = self.parse_and_execute(code);
        self.current_depth -= 1;
        result
    }

    /// 解析并执行代码
    fn parse_and_execute(&mut self, code: &str) -> EvalResult {
        let mut result = EvalResult::empty();
        // 简化实现：直接处理基本语句
        let lines: Vec<&str> = code.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            self.context.set_line(line_num + 1);
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("#") {
                continue;
            }
            // 处理语句
            match self.execute_statement(trimmed) {
                Ok(stmt_result) => {
                    result.merge(stmt_result);
                    if result.has_return || result.has_exception {
                        break;
                    }
                }
                Err(err) => {
                    result.merge(EvalResult::error(err));
                    break;
                }
            }
        }
        result
    }

    /// 执行单个语句
    fn execute_statement(&mut self, stmt: &str) -> Result<EvalResult, EvalError> {
        // 变量赋值: $var = value;
        if stmt.starts_with('$') && stmt.contains('=') && stmt.ends_with(';') {
            return self.execute_assignment(stmt);
        }
        // echo 语句
        if stmt.starts_with("echo ") || stmt.starts_with("echo(") {
            return self.execute_echo(stmt);
        }
        // print 语句
        if stmt.starts_with("print ") || stmt.starts_with("print(") {
            return self.execute_print(stmt);
        }
        // return 语句
        if stmt.starts_with("return ") || stmt == "return;" {
            return self.execute_return(stmt);
        }
        // 函数调用
        if stmt.contains('(') && stmt.ends_with(';') {
            return self.execute_function_call(stmt);
        }
        // if 语句
        if stmt.starts_with("if ") || stmt.starts_with("if(") {
            return self.execute_if(stmt);
        }
        // 空语句
        if stmt == ";" {
            return Ok(EvalResult::empty());
        }
        // 未知语句
        Ok(EvalResult::empty())
    }

    /// 执行变量赋值
    fn execute_assignment(&mut self, stmt: &str) -> Result<EvalResult, EvalError> {
        let stmt = stmt.trim_end_matches(';').trim();
        let parts: Vec<&str> = stmt.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(EvalError::SyntaxError {
                message: "无效的赋值语句".to_string(),
                line: self.context.current_line,
            });
        }
        let var_name = parts[0].trim().trim_start_matches('$').to_string();
        let value_expr = parts[1].trim();
        // 解析值
        let value = self.parse_value(value_expr)?;
        self.context.set_variable(&var_name, value);
        Ok(EvalResult::empty())
    }

    /// 解析值表达式
    fn parse_value(&self, expr: &str) -> Result<EvalVariable, EvalError> {
        let expr = expr.trim();
        // 字符串
        if (expr.starts_with('"') && expr.ends_with('"')) || (expr.starts_with('\'') && expr.ends_with('\'')) {
            let s = &expr[1..expr.len()-1];
            return Ok(EvalVariable::string(s));
        }
        // 布尔值
        if expr == "true" {
            return Ok(EvalVariable::boolean(true));
        }
        if expr == "false" {
            return Ok(EvalVariable::boolean(false));
        }
        // null
        if expr == "null" || expr == "NULL" {
            return Ok(EvalVariable::null());
        }
        // 整数
        if let Ok(i) = expr.parse::<i64>() {
            return Ok(EvalVariable::integer(i));
        }
        // 浮点数
        if let Ok(f) = expr.parse::<f64>() {
            return Ok(EvalVariable::float(f));
        }
        // 变量引用
        if expr.starts_with('$') {
            let var_name = &expr[1..];
            if let Some(val) = self.context.get_variable(var_name) {
                return Ok(val);
            }
            return Ok(EvalVariable::null());
        }
        // 数组
        if expr.starts_with('[') && expr.ends_with(']') {
            return self.parse_array(expr);
        }
        // 常量
        if let Some(val) = self.context.get_constant(expr) {
            return Ok(val.clone());
        }
        // 默认返回字符串
        Ok(EvalVariable::string(expr))
    }

    /// 解析数组
    fn parse_array(&self, expr: &str) -> Result<EvalVariable, EvalError> {
        let inner = &expr[1..expr.len()-1].trim();
        if inner.is_empty() {
            return Ok(EvalVariable::array(Vec::new()));
        }
        let mut arr = Vec::new();
        // 简化的数组解析
        for item in inner.split(',') {
            let item = item.trim();
            if !item.is_empty() {
                arr.push(self.parse_value(item)?);
            }
        }
        Ok(EvalVariable::array(arr))
    }

    /// 执行 echo 语句
    fn execute_echo(&mut self, stmt: &str) -> Result<EvalResult, EvalError> {
        let stmt = stmt.trim_start_matches("echo").trim().trim_end_matches(';');
        let value = self.parse_value(stmt)?;
        let mut result = EvalResult::empty();
        result.append_output(&value.to_string());
        Ok(result)
    }

    /// 执行 print 语句
    fn execute_print(&mut self, stmt: &str) -> Result<EvalResult, EvalError> {
        let stmt = stmt.trim_start_matches("print").trim().trim_end_matches(';');
        let value = self.parse_value(stmt)?;
        let mut result = EvalResult::empty();
        result.append_output(&value.to_string());
        result.value = Some(EvalVariable::integer(1));
        Ok(result)
    }

    /// 执行 return 语句
    fn execute_return(&mut self, stmt: &str) -> Result<EvalResult, EvalError> {
        let stmt = stmt.trim_start_matches("return").trim().trim_end_matches(';');
        if stmt.is_empty() {
            return Ok(EvalResult::return_value(EvalVariable::null()));
        }
        let value = self.parse_value(stmt)?;
        Ok(EvalResult::return_value(value))
    }

    /// 执行函数调用
    fn execute_function_call(&mut self, stmt: &str) -> Result<EvalResult, EvalError> {
        let stmt = stmt.trim_end_matches(';').trim();
        // 解析函数名和参数
        let open_paren = stmt.find('(').ok_or_else(|| EvalError::SyntaxError {
            message: "无效的函数调用".to_string(),
            line: self.context.current_line,
        })?;
        let func_name = &stmt[..open_paren];
        let args_str = &stmt[open_paren + 1..stmt.len() - 1];
        // 解析参数
        let args = self.parse_arguments(args_str)?;
        // 调用内置函数
        self.call_function(func_name, &args)
    }

    /// 解析参数列表
    fn parse_arguments(&self, args_str: &str) -> Result<Vec<EvalVariable>, EvalError> {
        let args_str = args_str.trim();
        if args_str.is_empty() {
            return Ok(Vec::new());
        }
        let mut args = Vec::new();
        for arg in args_str.split(',') {
            let arg = arg.trim();
            if !arg.is_empty() {
                args.push(self.parse_value(arg)?);
            }
        }
        Ok(args)
    }

    /// 调用函数
    fn call_function(&mut self, name: &str, args: &[EvalVariable]) -> Result<EvalResult, EvalError> {
        // 内置函数
        match name {
            "strlen" => {
                if args.is_empty() {
                    return Err(EvalError::ArgumentError {
                        message: "strlen() 需要一个参数".to_string(),
                    });
                }
                let len = args[0].to_string().len() as i64;
                Ok(EvalResult::with_value(EvalVariable::integer(len)))
            }
            "substr" => {
                if args.len() < 2 {
                    return Err(EvalError::ArgumentError {
                        message: "substr() 需要至少两个参数".to_string(),
                    });
                }
                let s = args[0].to_string();
                let start = args[1].to_integer() as usize;
                let result = if args.len() > 2 {
                    let len = args[2].to_integer() as usize;
                    s.chars().skip(start).take(len).collect::<String>()
                } else {
                    s.chars().skip(start).collect::<String>()
                };
                Ok(EvalResult::with_value(EvalVariable::string(&result)))
            }
            "str_replace" => {
                if args.len() < 3 {
                    return Err(EvalError::ArgumentError {
                        message: "str_replace() 需要三个参数".to_string(),
                    });
                }
                let search = args[0].to_string();
                let replace = args[1].to_string();
                let subject = args[2].to_string();
                let result = subject.replace(&search, &replace);
                Ok(EvalResult::with_value(EvalVariable::string(&result)))
            }
            "strtolower" => {
                if args.is_empty() {
                    return Err(EvalError::ArgumentError {
                        message: "strtolower() 需要一个参数".to_string(),
                    });
                }
                Ok(EvalResult::with_value(EvalVariable::string(&args[0].to_string().to_lowercase())))
            }
            "strtoupper" => {
                if args.is_empty() {
                    return Err(EvalError::ArgumentError {
                        message: "strtoupper() 需要一个参数".to_string(),
                    });
                }
                Ok(EvalResult::with_value(EvalVariable::string(&args[0].to_string().to_uppercase())))
            }
            "trim" => {
                if args.is_empty() {
                    return Err(EvalError::ArgumentError {
                        message: "trim() 需要一个参数".to_string(),
                    });
                }
                Ok(EvalResult::with_value(EvalVariable::string(args[0].to_string().trim())))
            }
            "count" => {
                if args.is_empty() {
                    return Err(EvalError::ArgumentError {
                        message: "count() 需要一个参数".to_string(),
                    });
                }
                let count = match &args[0] {
                    EvalVariable::Array(a) => a.len() as i64,
                    EvalVariable::AssociativeArray(a) => a.len() as i64,
                    _ => 1,
                };
                Ok(EvalResult::with_value(EvalVariable::integer(count)))
            }
            "is_array" => {
                if args.is_empty() {
                    return Err(EvalError::ArgumentError {
                        message: "is_array() 需要一个参数".to_string(),
                    });
                }
                let is_arr = matches!(&args[0], EvalVariable::Array(_) | EvalVariable::AssociativeArray(_));
                Ok(EvalResult::with_value(EvalVariable::boolean(is_arr)))
            }
            "is_string" => {
                if args.is_empty() {
                    return Err(EvalError::ArgumentError {
                        message: "is_string() 需要一个参数".to_string(),
                    });
                }
                let is_str = matches!(&args[0], EvalVariable::String(_));
                Ok(EvalResult::with_value(EvalVariable::boolean(is_str)))
            }
            "is_int" | "is_integer" => {
                if args.is_empty() {
                    return Err(EvalError::ArgumentError {
                        message: format!("{}() 需要一个参数", name),
                    });
                }
                let is_int = matches!(&args[0], EvalVariable::Integer(_));
                Ok(EvalResult::with_value(EvalVariable::boolean(is_int)))
            }
            "is_numeric" => {
                if args.is_empty() {
                    return Err(EvalError::ArgumentError {
                        message: "is_numeric() 需要一个参数".to_string(),
                    });
                }
                let is_num = matches!(&args[0], EvalVariable::Integer(_) | EvalVariable::Float(_));
                Ok(EvalResult::with_value(EvalVariable::boolean(is_num)))
            }
            "empty" => {
                if args.is_empty() {
                    return Err(EvalError::ArgumentError {
                        message: "empty() 需要一个参数".to_string(),
                    });
                }
                Ok(EvalResult::with_value(EvalVariable::boolean(args[0].is_empty())))
            }
            "isset" => {
                if args.is_empty() {
                    return Err(EvalError::ArgumentError {
                        message: "isset() 需要一个参数".to_string(),
                    });
                }
                Ok(EvalResult::with_value(EvalVariable::boolean(!args[0].is_null())))
            }
            "var_dump" => {
                let mut output = String::new();
                for arg in args {
                    output.push_str(&format!("{:?}\n", arg));
                }
                let mut result = EvalResult::empty();
                result.append_output(&output);
                Ok(result)
            }
            "print_r" => {
                let output = if args.is_empty() {
                    String::new()
                } else {
                    format!("{:?}\n", args[0])
                };
                let mut result = EvalResult::empty();
                result.append_output(&output);
                Ok(result)
            }
            "die" | "exit" => {
                let mut result = EvalResult::empty();
                if !args.is_empty() {
                    result.append_output(&args[0].to_string());
                }
                result.has_return = true;
                Ok(result)
            }
            _ => {
                // 检查用户定义的函数
                let func_opt = self.context.get_function(name).cloned();
                if let Some(func) = func_opt {
                    // 调用用户函数
                    self.call_user_function(&func, args)
                } else {
                    Err(EvalError::UndefinedFunction {
                        name: name.to_string(),
                    })
                }
            }
        }
    }

    /// 调用用户定义的函数
    fn call_user_function(&mut self, func: &EvalFunction, args: &[EvalVariable]) -> Result<EvalResult, EvalError> {
        // 检查参数数量
        let required_params = func.parameters.iter().filter(|p| !p.optional).count();
        if args.len() < required_params {
            return Err(EvalError::ArgumentError {
                message: format!("{}() 需要 {} 个参数，提供了 {}", func.name, required_params, args.len()),
            });
        }
        // 创建函数作用域
        self.context.push_scope(&func.name);
        // 绑定参数
        for (i, param) in func.parameters.iter().enumerate() {
            let value = if i < args.len() {
                args[i].clone()
            } else if let Some(ref default) = param.default_value {
                default.clone()
            } else {
                EvalVariable::null()
            };
            self.context.set_variable(&param.name, value);
        }
        // 执行函数体
        let result = self.eval(&func.body);
        // 退出作用域
        self.context.pop_scope();
        Ok(result)
    }

    /// 执行 if 语句
    fn execute_if(&mut self, stmt: &str) -> Result<EvalResult, EvalError> {
        // 简化的 if 语句处理
        // 实际实现需要完整的语法解析
        Ok(EvalResult::empty())
    }

    /// 执行文件
    /// 
    /// # 参数
    /// - `path`: 文件路径
    /// 
    /// # 返回
    /// 返回执行结果
    pub fn eval_file(&mut self, path: &str) -> Result<EvalResult, EvalError> {
        let content = std::fs::read_to_string(path).map_err(|e| EvalError::RuntimeError {
            message: format!("无法读取文件: {}", e),
            line: 0,
        })?;
        self.context.set_file(path);
        Ok(self.eval(&content))
    }

    /// 注册内置函数
    pub fn register_builtin_functions(&mut self) {
        // 注册常用的内置函数
        // 这里只是声明，实际实现在 call_function 中
    }

    /// 定义函数
    pub fn define_function(&mut self, name: &str, params: Vec<EvalParameter>, body: &str) {
        let func = EvalFunction {
            name: name.to_string(),
            parameters: params,
            body: body.to_string(),
            returns_reference: false,
            is_builtin: false,
            static_vars: HashMap::new(),
        };
        self.context.define_function(func);
    }

    /// 定义类
    pub fn define_class(&mut self, class: EvalClass) {
        self.context.define_class(class);
    }

    /// 定义常量
    pub fn define_constant(&mut self, name: &str, value: EvalVariable) {
        self.context.define_constant(name, value);
    }
}

impl Default for Eval {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_assignment() {
        let mut eval = Eval::new();
        let result = eval.eval("$x = 42;");
        assert!(!result.has_exception);
        assert!(eval.context().has_variable("x"));
    }

    #[test]
    fn test_eval_echo() {
        let mut eval = Eval::new();
        let result = eval.eval("echo 'hello';");
        assert!(!result.has_exception);
        assert_eq!(result.output, "hello");
    }

    #[test]
    fn test_eval_return() {
        let mut eval = Eval::new();
        let result = eval.eval("return 42;");
        assert!(result.has_return);
        assert_eq!(result.value.unwrap().to_integer(), 42);
    }

    #[test]
    fn test_eval_strlen() {
        let mut eval = Eval::new();
        let result = eval.eval("$len = strlen('hello');");
        assert!(!result.has_exception);
        let len = eval.context().get_variable("len").unwrap();
        assert_eq!(len.to_integer(), 5);
    }
}
