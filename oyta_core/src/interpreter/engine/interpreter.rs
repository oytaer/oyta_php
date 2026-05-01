//! 解释器核心结构体模块
//!
//! 定义 Interpreter 结构体及其核心方法
//! 包括：构造函数、控制器方法执行、函数调用、闭包/箭头函数执行
//! 这些是解释器的主要入口点，被其他模块的 impl 块调用

use anyhow::{Context, Result};
use php_ast::ast::*;
use php_rs_parser::{ParserContext, PhpVersion};
use std::collections::HashMap;

use crate::interpreter::value::{
    CallableValue, ExecutionResult, ExecutionContext, ObjectInstance, Value,
};
use crate::symbol_table::registry::SymbolRegistry;
use crate::symbol_table::types::{ClassDef, MethodDef};

use super::builtins::create_builtins_map;
use super::helpers::parse_default_value;

/// 继承链方法查找结果
///
/// 包含方法定义和定义该方法的类信息
struct InheritanceMethodResult {
    /// 方法定义
    method_def: MethodDef,
    /// 定义该方法的类名（完整类名）
    class_name: String,
    /// 定义该方法的类文件路径
    file_path: String,
}

/// PHP 解释器引擎
///
/// 负责将 PHP AST 节点解释执行为运行时值
/// 核心功能：
/// - 解析 PHP 源文件并执行指定方法
/// - 管理内置函数表
/// - 执行语句和表达式求值（在 stmt/expr 模块中实现）
/// - 处理闭包和箭头函数
pub struct Interpreter {
    /// 符号表注册表，存储所有扫描到的类、函数、常量定义
    pub(crate) registry: std::sync::Arc<SymbolRegistry>,
    /// 内置函数映射表（函数名 → 函数指针）
    pub(crate) builtins: HashMap<String, super::builtins::BuiltinFunction>,
    /// 闭包计数器，用于生成唯一闭包 ID
    pub(crate) closure_counter: std::cell::Cell<usize>,
}

impl Interpreter {
    /// 创建新的解释器实例
    ///
    /// 初始化内置函数表，注册 90+ 个 PHP 内置函数
    ///
    /// # 参数
    /// - `registry`: 符号表注册表的 Arc 引用
    ///
    /// # 返回
    /// 初始化完成的解释器实例
    pub fn new(registry: std::sync::Arc<SymbolRegistry>) -> Self {
        Self {
            registry,
            builtins: create_builtins_map(),
            closure_counter: std::cell::Cell::new(0),
        }
    }

    /// 生成闭包唯一 ID
    ///
    /// 每次调用自增计数器，返回格式为 "closure_N" 的唯一标识
    /// 用于在 AST 中定位闭包体
    pub(crate) fn next_closure_id(&self) -> String {
        let id = self.closure_counter.get();
        self.closure_counter.set(id + 1);
        format!("closure_{}", id)
    }

    /// 在继承链中查找方法
    ///
    /// 递归查找当前类及其所有父类，直到找到目标方法
    /// 支持多层继承：class A extends B extends C
    ///
    /// # 参数
    /// - `class_name`: 起始类名（完整类名）
    /// - `method_name`: 要查找的方法名
    ///
    /// # 返回
    /// 找到返回 InheritanceMethodResult，找不到返回 None
    fn find_method_in_inheritance_chain(
        &self,
        class_name: &str,
        method_name: &str,
    ) -> Option<InheritanceMethodResult> {
        // 查找当前类定义
        let class_def = self.registry.find_class(class_name)?;
        
        // 在当前类中查找方法
        if let Some(method_def) = class_def.find_method(method_name) {
            return Some(InheritanceMethodResult {
                method_def: method_def.clone(),
                class_name: class_def.full_name.clone(),
                file_path: class_def.file_path.clone(),
            });
        }
        
        // 当前类没有该方法，查找父类
        if let Some(parent_class) = &class_def.extends {
            // 递归查找父类
            self.find_method_in_inheritance_chain(parent_class, method_name)
        } else {
            None
        }
    }

    /// 收集继承链中所有类的属性
    ///
    /// 从当前类开始，递归收集所有父类的属性
    /// 子类属性会覆盖父类同名属性
    ///
    /// # 参数
    /// - `class_name`: 起始类名
    ///
    /// # 返回
    /// 属性名到默认值的映射
    fn collect_properties_from_inheritance_chain(
        &self,
        class_name: &str,
    ) -> HashMap<String, Value> {
        let mut properties = HashMap::new();
        self.collect_properties_recursive(class_name, &mut properties);
        properties
    }

    /// 递归收集属性（内部辅助函数）
    fn collect_properties_recursive(
        &self,
        class_name: &str,
        properties: &mut HashMap<String, Value>,
    ) {
        if let Some(class_def) = self.registry.find_class(class_name) {
            // 先收集父类属性（子类会覆盖）
            if let Some(parent_class) = &class_def.extends {
                self.collect_properties_recursive(parent_class, properties);
            }
            
            // 收集当前类属性
            for prop in &class_def.properties {
                let default_value = match &prop.default_value {
                    Some(v) => parse_default_value(v),
                    None => Value::Null,
                };
                properties.insert(prop.name.clone(), default_value);
            }
        }
    }

    /// 执行控制器方法
    ///
    /// 这是 HTTP 请求处理的核心入口
    /// 流程：
    /// 1. 从符号表查找类定义
    /// 2. 创建对象实例并初始化属性（包括继承的属性）
    /// 3. 设置执行上下文（$this、命名空间、请求对象）
    /// 4. 绑定方法参数
    /// 5. 从源文件解析 AST 并执行方法体（支持继承链查找）
    ///
    /// # 参数
    /// - `class_name`: 类名（含命名空间）
    /// - `method_name`: 方法名
    /// - `args`: 方法参数列表
    /// - `request`: 可选的请求对象值
    ///
    /// # 返回
    /// 方法执行的返回值
    pub fn execute_controller_method(
        &mut self,
        class_name: &str,
        method_name: &str,
        args: Vec<Value>,
        request: Option<Value>,
    ) -> Result<Value> {
        // 验证类存在
        let class_def = self
            .registry
            .find_class(class_name)
            .with_context(|| format!("类不存在: {}", class_name))?;

        // 在继承链中查找方法（支持父类方法）
        let method_result = self
            .find_method_in_inheritance_chain(class_name, method_name)
            .with_context(|| format!("方法不存在: {}::{}", class_name, method_name))?;

        // 收集继承链中所有属性（父类 + 子类）
        let properties = self.collect_properties_from_inheritance_chain(class_name);

        let instance = ObjectInstance {
            class_name: class_name.to_string(),
            properties,
        };

        let mut ctx = ExecutionContext::new();
        ctx.this = Some(instance);
        if let Some(ns) = &class_def.namespace {
            ctx.namespace = Some(ns.clone());
        }

        if let Some(req_value) = request {
            ctx.set_var("request", req_value);
        }

        // 绑定方法参数
        for (i, param) in method_result.method_def.params.iter().enumerate() {
            let param_name = param.name.trim_start_matches('$').to_string();
            let value = if i < args.len() {
                args[i].clone()
            } else if let Some(default) = &param.default_value {
                parse_default_value(default)
            } else {
                Value::Null
            };
            ctx.set_var(&param_name, value);
        }

        // 执行方法（从定义该方法的类文件中执行）
        self.execute_from_source(&method_result.file_path, &method_result.class_name, method_name, &mut ctx)
    }

    /// 从源文件解析并执行指定方法
    ///
    /// 读取 PHP 源文件，解析为 AST，在类声明中查找目标方法并执行
    /// 支持继承链：如果在当前类找不到方法，会查找父类
    ///
    /// # 参数
    /// - `file_path`: PHP 源文件路径
    /// - `class_name`: 类名
    /// - `method_name`: 方法名
    /// - `ctx`: 执行上下文
    fn execute_from_source(
        &mut self,
        file_path: &str,
        class_name: &str,
        method_name: &str,
        ctx: &mut ExecutionContext,
    ) -> Result<Value> {
        let source = std::fs::read_to_string(file_path)
            .with_context(|| format!("无法读取源文件: {}", file_path))?;

        let mut parser_context = ParserContext::new();
        let result = parser_context.reparse_versioned(&source, PhpVersion::Php85);

        let short_name = class_name.split('\\').last().unwrap_or(class_name);

        for stmt in result.program.stmts.iter() {
            if let StmtKind::Class(class_stmt) = &stmt.kind {
                if let Some(name) = class_stmt.name {
                    if name == short_name || name == class_name {
                        for member in class_stmt.members.iter() {
                            if let ClassMemberKind::Method(method) = &member.kind {
                                if method.name == method_name {
                                    if let Some(body) = &method.body {
                                        let result_val = self.run_stmts(body, ctx)?;
                                        return Ok(result_val.into_value());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 当前类文件中找不到方法，尝试在父类中查找
        if let Some(class_def) = self.registry.find_class(class_name) {
            if let Some(parent_class) = &class_def.extends {
                // 递归查找父类
                if let Some(parent_method) = self.find_method_in_inheritance_chain(parent_class, method_name) {
                    return self.execute_from_source(
                        &parent_method.file_path,
                        &parent_method.class_name,
                        method_name,
                        ctx
                    );
                }
            }
        }

        anyhow::bail!("未找到方法体: {}::{}", class_name, method_name)
    }

    /// 执行函数调用
    ///
    /// 按优先级查找函数：
    /// 1. 内置函数表
    /// 2. 符号表中注册的用户函数
    /// 3. Callable 值调用
    ///
    /// # 参数
    /// - `name`: 函数名
    /// - `args`: 参数列表
    ///
    /// # 返回
    /// 函数执行的返回值
    pub fn call_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value> {
        if let Some(builtin) = self.builtins.get(name) {
            return builtin(&args);
        }

        if let Some(func_def) = self.registry.find_function(name) {
            let mut ctx = ExecutionContext::new();
            for (i, param) in func_def.params.iter().enumerate() {
                let param_name = param.name.trim_start_matches('$').to_string();
                let value = if i < args.len() {
                    args[i].clone()
                } else {
                    Value::Null
                };
                ctx.set_var(&param_name, value);
            }

            let source = std::fs::read_to_string(&func_def.file_path)
                .with_context(|| format!("无法读取源文件: {}", func_def.file_path))?;

            let mut parser_context = ParserContext::new();
            let result = parser_context.reparse_versioned(&source, PhpVersion::Php85);

            let short_name = name.split('\\').last().unwrap_or(name);
            for stmt in result.program.stmts.iter() {
                if let StmtKind::Function(func_stmt) = &stmt.kind {
                    if func_stmt.name == short_name || func_stmt.name == name {
                        let result_val = self.run_stmts(&func_stmt.body, &mut ctx)?;
                        return Ok(result_val.into_value());
                    }
                }
            }

            return Ok(Value::Null);
        }

        // 检查是否是方法调用格式 (ClassName::methodName)
        if name.contains("::") {
            let parts: Vec<&str> = name.splitn(2, "::").collect();
            if parts.len() == 2 {
                let class_name = parts[0];
                let method_name = parts[1];
                
                // 首先检查用户定义的类（支持继承链）
                if let Some(method_result) = self.find_method_in_inheritance_chain(class_name, method_name) {
                    let mut ctx = ExecutionContext::new();
                    for (i, param) in method_result.method_def.params.iter().enumerate() {
                        let param_name = param.name.trim_start_matches('$').to_string();
                        let value = if i < args.len() {
                            args[i].clone()
                        } else {
                            Value::Null
                        };
                        ctx.set_var(&param_name, value);
                    }
                    return self.execute_from_source(
                        &method_result.file_path,
                        &method_result.class_name,
                        method_name,
                        &mut ctx
                    );
                }
                
                // 检查内置类注册表
                let builtin_registry = super::builtin_classes::get_builtin_class_registry();
                
                // 尝试调用静态方法
                if let Some(result) = builtin_registry.call_static_method(class_name, method_name, &args) {
                    return result;
                }
                
                // 检查门面类注册表
                let facade_registry = super::builtin_facades::get_facade_class_registry();
                if let Some(result) = facade_registry.call_static_method(class_name, method_name, &args) {
                    return result;
                }
            }
        }

        if args.len() >= 1 {
            if let Value::Callable(callable) = &args[0] {
                return self.call_callable(callable, &args[1..]);
            }
        }

        anyhow::bail!("函数不存在: {}", name)
    }

    /// 调用可调用值
    ///
    /// 支持四种可调用类型：
    /// - Function: 命名函数调用
    /// - Method: 类方法调用
    /// - Closure: 闭包调用（捕获变量 + 重新解析源文件定位闭包体）
    /// - Arrow: 箭头函数调用（自动捕获外层变量 + 定位箭头函数体）
    ///
    /// # 参数
    /// - `callable`: 可调用值引用
    /// - `args`: 参数列表
    fn call_callable(&mut self, callable: &CallableValue, args: &[Value]) -> Result<Value> {
        match callable {
            CallableValue::Function { name } => self.call_function(name, args.to_vec()),
            CallableValue::Method {
                class_name,
                method_name,
            } => self.call_function(&format!("{}::{}", class_name, method_name), args.to_vec()),
            CallableValue::Closure {
                id,
                params,
                captured,
                file_path,
                ..
            } => {
                let mut ctx = ExecutionContext::new();
                for (k, v) in captured {
                    ctx.set_var(k, v.clone());
                }
                for (i, param_name) in params.iter().enumerate() {
                    let value = if i < args.len() {
                        args[i].clone()
                    } else {
                        Value::Null
                    };
                    ctx.set_var(param_name, value);
                }
                let source = std::fs::read_to_string(file_path)
                    .with_context(|| format!("无法读取源文件: {}", file_path))?;
                let mut parser_context = ParserContext::new();
                let result = parser_context.reparse_versioned(&source, PhpVersion::Php85);
                if let Some(body) = Self::find_closure_body(&result.program, id) {
                    let result_val = self.run_stmts(body, &mut ctx)?;
                    return Ok(result_val.into_value());
                }
                Ok(Value::Null)
            }
            CallableValue::Arrow {
                id,
                params,
                captured,
                file_path,
            } => {
                let mut ctx = ExecutionContext::new();
                for (k, v) in captured {
                    ctx.set_var(k, v.clone());
                }
                for (i, param_name) in params.iter().enumerate() {
                    let value = if i < args.len() {
                        args[i].clone()
                    } else {
                        Value::Null
                    };
                    ctx.set_var(param_name, value);
                }
                let source = std::fs::read_to_string(file_path)
                    .with_context(|| format!("无法读取源文件: {}", file_path))?;
                let mut parser_context = ParserContext::new();
                let result = parser_context.reparse_versioned(&source, PhpVersion::Php85);
                if let Some(expr) = Self::find_arrow_body(&result.program, id) {
                    return self.eval_expr(expr, &mut ctx);
                }
                Ok(Value::Null)
            }
        }
    }

    /// 在 AST 中查找闭包体
    ///
    /// 遍历 AST 中的类方法体，查找闭包表达式
    /// 返回闭包体的语句列表引用
    ///
    /// # 参数
    /// - `program`: PHP 程序 AST
    /// - `_id`: 闭包 ID（当前未使用，返回第一个找到的闭包）
    fn find_closure_body<'arena, 'src>(
        program: &Program<'arena, 'src>,
        _id: &str,
    ) -> Option<&'arena ArenaVec<'arena, Stmt<'arena, 'src>>> {
        for stmt in program.stmts.iter() {
            if let StmtKind::Class(class_stmt) = &stmt.kind {
                for member in class_stmt.members.iter() {
                    if let ClassMemberKind::Method(method) = &member.kind {
                        if let Some(body) = &method.body {
                            for s in body.iter() {
                                if let StmtKind::Expression(expr) = &s.kind {
                                    if let ExprKind::Assign(assign) = &expr.kind {
                                        if let ExprKind::Closure(closure) = &assign.target.kind {
                                            return Some(&closure.body);
                                        }
                                        if let ExprKind::Closure(closure) = &assign.value.kind {
                                            return Some(&closure.body);
                                        }
                                    }
                                    if let ExprKind::Closure(closure) = &expr.kind {
                                        return Some(&closure.body);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// 在 AST 中查找箭头函数体
    ///
    /// 遍历 AST 中的类方法体，查找箭头函数表达式
    /// 返回箭头函数体的表达式引用
    ///
    /// # 参数
    /// - `program`: PHP 程序 AST
    /// - `_id`: 箭头函数 ID（当前未使用，返回第一个找到的箭头函数）
    fn find_arrow_body<'arena, 'src>(
        program: &Program<'arena, 'src>,
        _id: &str,
    ) -> Option<&'arena Expr<'arena, 'src>> {
        for stmt in program.stmts.iter() {
            if let StmtKind::Class(class_stmt) = &stmt.kind {
                for member in class_stmt.members.iter() {
                    if let ClassMemberKind::Method(method) = &member.kind {
                        if let Some(body) = &method.body {
                            for s in body.iter() {
                                if let StmtKind::Expression(expr) = &s.kind {
                                    if let ExprKind::Assign(assign) = &expr.kind {
                                        if let ExprKind::ArrowFunction(arrow) =
                                            &assign.target.kind
                                        {
                                            return Some(arrow.body);
                                        }
                                        if let ExprKind::ArrowFunction(arrow) =
                                            &assign.value.kind
                                        {
                                            return Some(arrow.body);
                                        }
                                    }
                                    if let ExprKind::ArrowFunction(arrow) = &expr.kind {
                                        return Some(arrow.body);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// 执行语句列表
    ///
    /// 顺序执行 ArenaVec 中的每个语句
    /// 遇到 Return/Break/Continue/Throw 控制流时立即返回
    /// 遇到 FunctionCall 结果时调用函数并继续
    ///
    /// # 参数
    /// - `stmts`: 语句列表
    /// - `ctx`: 执行上下文
    pub(crate) fn run_stmts<'arena, 'src>(
        &mut self,
        stmts: &ArenaVec<'arena, Stmt<'arena, 'src>>,
        ctx: &mut ExecutionContext,
    ) -> Result<ExecutionResult> {
        let mut last_value = Value::Null;

        for stmt in stmts.iter() {
            let result = self.run_stmt(stmt, ctx)?;
            match result {
                ExecutionResult::Value(v) => last_value = v,
                ExecutionResult::Return(_)
                | ExecutionResult::Break(_)
                | ExecutionResult::Continue(_)
                | ExecutionResult::Throw(_) => return Ok(result),
                ExecutionResult::FunctionCall { name, args } => {
                    last_value = self.call_function(&name, args)?;
                }
            }
        }

        Ok(ExecutionResult::Value(last_value))
    }
}
