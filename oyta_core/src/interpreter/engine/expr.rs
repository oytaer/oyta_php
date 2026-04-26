//! 表达式求值模块
//!
//! 实现 PHP 的所有表达式类型求值逻辑
//! 包括：字面量、变量、赋值、二元/一元运算、三元/Null合并、类型转换、
//! 函数调用、方法调用、属性访问、静态调用、数组操作、对象创建、
//! 闭包/箭头函数、match、isset/empty、魔术常量等

use anyhow::Result;
use php_ast::ast::*;
use std::collections::HashMap;

use crate::interpreter::value::{
    CallableValue, CastKind, ExecutionContext, ObjectInstance, Value,
};

use super::Interpreter;
use super::binary_op::{eval_binary_op, values_identical};
use super::helpers::{expr_to_name, parse_default_value};

impl Interpreter {
    /// 评估表达式
    ///
    /// 根据 ExprKind 分发到对应的求值逻辑
    /// 每种表达式类型返回对应的 Value
    ///
    /// # 参数
    /// - `expr`: PHP AST 表达式引用
    /// - `ctx`: 执行上下文（可变借用）
    ///
    /// # 返回
    /// 表达式求值结果
    pub(crate) fn eval_expr<'arena, 'src>(
        &mut self,
        expr: &Expr<'arena, 'src>,
        ctx: &mut ExecutionContext,
    ) -> Result<Value> {
        match &expr.kind {
            /// 整数字面量：42
            ExprKind::Int(i) => Ok(Value::Int(*i)),
            /// 浮点数字面量：3.14
            ExprKind::Float(f) => Ok(Value::Float(*f)),
            /// 字符串字面量："hello"
            ExprKind::String(s) => Ok(Value::String(s.to_string())),
            /// 布尔字面量：true / false
            ExprKind::Bool(b) => Ok(Value::Bool(*b)),
            /// null 字面量
            ExprKind::Null => Ok(Value::Null),

            /// 字符串插值："Hello, $name!"
            /// 支持变量插值和表达式插值
            ExprKind::InterpolatedString(parts) => {
                let mut result = String::new();
                for part in parts.iter() {
                    match part {
                        StringPart::Literal(s) => result.push_str(s),
                        StringPart::Expr(e) => {
                            let val = self.eval_expr(e, ctx)?;
                            result.push_str(&val.to_string_value());
                        }
                    }
                }
                Ok(Value::String(result))
            }

            /// 变量引用：$name
            ExprKind::Variable(var) => {
                let name = var.as_str().trim_start_matches('$');
                Ok(ctx.get_var(name).cloned().unwrap_or(Value::Null))
            }

            /// 可变变量：$$name
            ExprKind::VariableVariable(inner) => {
                let name_val = self.eval_expr(inner, ctx)?;
                let name = name_val
                    .to_string_value()
                    .trim_start_matches('$')
                    .to_string();
                Ok(ctx.get_var(&name).cloned().unwrap_or(Value::Null))
            }

            /// 赋值表达式：$a = 1, $a += 2
            ExprKind::Assign(assign) => {
                let value = self.eval_expr(assign.value, ctx)?;
                self.assign_to(&assign.target, value.clone(), ctx)?;
                Ok(value)
            }

            /// 二元运算：$a + $b, $a == $b 等
            ExprKind::Binary(binary) => {
                let left = self.eval_expr(binary.left, ctx)?;
                let right = self.eval_expr(binary.right, ctx)?;
                Ok(eval_binary_op(&binary.op, &left, &right))
            }

            /// 前缀一元运算：-$a, !$b, ++$i, --$i, ~$bits
            ExprKind::UnaryPrefix(unary) => {
                let operand = self.eval_expr(unary.operand, ctx)?;
                match unary.op {
                    UnaryPrefixOp::Negate => Ok(Value::Int(-operand.to_int())),
                    UnaryPrefixOp::BooleanNot => Ok(Value::Bool(!operand.is_truthy())),
                    UnaryPrefixOp::BitwiseNot => Ok(Value::Int(!operand.to_int())),
                    UnaryPrefixOp::PreIncrement => {
                        let new_val = Value::Int(operand.to_int() + 1);
                        self.assign_to(unary.operand, new_val.clone(), ctx)?;
                        Ok(new_val)
                    }
                    UnaryPrefixOp::PreDecrement => {
                        let new_val = Value::Int(operand.to_int() - 1);
                        self.assign_to(unary.operand, new_val.clone(), ctx)?;
                        Ok(new_val)
                    }
                    _ => Ok(operand),
                }
            }

            /// 后缀一元运算：$i++, $i--
            ExprKind::UnaryPostfix(unary) => {
                let operand = self.eval_expr(unary.operand, ctx)?;
                let old_val = operand.clone();
                match unary.op {
                    UnaryPostfixOp::PostIncrement => {
                        let new_val = Value::Int(operand.to_int() + 1);
                        self.assign_to(unary.operand, new_val, ctx)?;
                        Ok(old_val)
                    }
                    UnaryPostfixOp::PostDecrement => {
                        let new_val = Value::Int(operand.to_int() - 1);
                        self.assign_to(unary.operand, new_val, ctx)?;
                        Ok(old_val)
                    }
                }
            }

            /// 三元运算：$cond ? $then : $else
            /// 简写形式：$cond ?: $else（then_expr 为 None 时返回条件值本身）
            ExprKind::Ternary(ternary) => {
                let cond = self.eval_expr(ternary.condition, ctx)?;
                if cond.is_truthy() {
                    match &ternary.then_expr {
                        Some(then) => self.eval_expr(then, ctx),
                        None => Ok(cond),
                    }
                } else {
                    self.eval_expr(ternary.else_expr, ctx)
                }
            }

            /// Null 合并运算符：$a ?? $b
            /// 左侧非 null 则返回左侧，否则返回右侧
            ExprKind::NullCoalesce(nc) => {
                let left = self.eval_expr(nc.left, ctx)?;
                if left.is_null() {
                    self.eval_expr(nc.right, ctx)
                } else {
                    Ok(left)
                }
            }

            /// 类型转换：(int)$a, (string)$b 等
            ExprKind::Cast(kind, operand) => {
                let value = self.eval_expr(operand, ctx)?;
                let cast_kind = match kind {
                    php_ast::ast::CastKind::Int => CastKind::Int,
                    php_ast::ast::CastKind::Float => CastKind::Float,
                    php_ast::ast::CastKind::String => CastKind::String,
                    php_ast::ast::CastKind::Bool => CastKind::Bool,
                    php_ast::ast::CastKind::Array => CastKind::Array,
                    php_ast::ast::CastKind::Object => CastKind::Object,
                    php_ast::ast::CastKind::Unset => CastKind::Unset,
                    php_ast::ast::CastKind::Void => CastKind::Unset,
                };
                Ok(value.cast(&cast_kind))
            }

            /// 函数调用：foo($arg1, $arg2)
            ExprKind::FunctionCall(call) => {
                let func_name = expr_to_name(call.name);
                let args: Vec<Value> = call
                    .args
                    .iter()
                    .map(|arg| self.eval_expr(&arg.value, ctx))
                    .collect::<Result<Vec<_>>>()?;
                self.call_function(&func_name, args)
            }

            /// 方法调用：$obj->method($arg)
            ExprKind::MethodCall(method_call) => {
                let obj_val = self.eval_expr(method_call.object, ctx)?;
                let method_name = expr_to_name(method_call.method);
                let args: Vec<Value> = method_call
                    .args
                    .iter()
                    .map(|arg| self.eval_expr(&arg.value, ctx))
                    .collect::<Result<Vec<_>>>()?;

                if let ExprKind::Variable(var) = &method_call.object.kind {
                    if var.as_str() == "this" {
                        return self.call_function(&method_name, args);
                    }
                }

                match &obj_val {
                    Value::Object(obj) => {
                        self.call_function(&format!("{}::{}", obj.class_name, method_name), args)
                    }
                    Value::String(s) => self.call_string_method(s, &method_name, args),
                    Value::IndexedArray(_) | Value::AssociativeArray(_) => {
                        self.call_array_method(&obj_val, &method_name, args)
                    }
                    _ => Ok(Value::Null),
                }
            }

            /// Null 安全方法调用：$obj?->method($arg)
            ExprKind::NullsafeMethodCall(method_call) => {
                let obj_val = self.eval_expr(method_call.object, ctx)?;
                if obj_val.is_null() {
                    return Ok(Value::Null);
                }
                let method_name = expr_to_name(method_call.method);
                let args: Vec<Value> = method_call
                    .args
                    .iter()
                    .map(|arg| self.eval_expr(&arg.value, ctx))
                    .collect::<Result<Vec<_>>>()?;
                if let ExprKind::Variable(var) = &method_call.object.kind {
                    if var.as_str() == "this" {
                        return self.call_function(&method_name, args);
                    }
                }
                match &obj_val {
                    Value::Object(obj) => {
                        self.call_function(&format!("{}::{}", obj.class_name, method_name), args)
                    }
                    _ => Ok(Value::Null),
                }
            }

            /// 属性访问：$obj->prop
            ExprKind::PropertyAccess(prop) => {
                if let ExprKind::Variable(var) = &prop.object.kind {
                    if var.as_str() == "this" {
                        if let Some(ref instance) = ctx.this {
                            let prop_name = expr_to_name(prop.property)
                                .trim_start_matches('$')
                                .to_string();
                            return Ok(instance
                                .properties
                                .get(&prop_name)
                                .cloned()
                                .unwrap_or(Value::Null));
                        }
                    }
                    let var_name = var.as_str().trim_start_matches('$');
                    if let Some(val) = ctx.get_var(var_name) {
                        if let Value::Object(obj) = val {
                            let prop_name = expr_to_name(prop.property)
                                .trim_start_matches('$')
                                .to_string();
                            return Ok(obj.properties.get(&prop_name).cloned().unwrap_or(Value::Null));
                        }
                    }
                }
                Ok(Value::Null)
            }

            /// Null 安全属性访问：$obj?->prop
            ExprKind::NullsafePropertyAccess(prop) => {
                let obj_val = self.eval_expr(prop.object, ctx)?;
                if obj_val.is_null() {
                    return Ok(Value::Null);
                }
                if let Value::Object(obj) = &obj_val {
                    let prop_name = expr_to_name(prop.property)
                        .trim_start_matches('$')
                        .to_string();
                    return Ok(obj.properties.get(&prop_name).cloned().unwrap_or(Value::Null));
                }
                Ok(Value::Null)
            }

            /// 静态方法调用：ClassName::method($arg)
            ExprKind::StaticMethodCall(call) => {
                let class_name = expr_to_name(call.class);
                let method_name = expr_to_name(call.method);
                let resolved = ctx.resolve_class_name(&class_name);
                let args: Vec<Value> = call
                    .args
                    .iter()
                    .map(|arg| self.eval_expr(&arg.value, ctx))
                    .collect::<Result<Vec<_>>>()?;
                self.call_function(&format!("{}::{}", resolved, method_name), args)
            }

            /// 静态属性访问：ClassName::$prop
            ExprKind::StaticPropertyAccess(access) => {
                let class_name = expr_to_name(access.class);
                let prop_name = expr_to_name(access.member)
                    .trim_start_matches('$')
                    .to_string();
                let resolved = ctx.resolve_class_name(&class_name);
                if let Some(class_def) = self.registry.find_class(&resolved) {
                    for prop in &class_def.properties {
                        if prop.name == prop_name && prop.is_static {
                            return Ok(prop
                                .default_value
                                .as_ref()
                                .map(|v| parse_default_value(v))
                                .unwrap_or(Value::Null));
                        }
                    }
                }
                Ok(Value::Null)
            }

            /// 类常量访问：ClassName::CONST
            ExprKind::ClassConstAccess(access) => {
                let class_name = expr_to_name(access.class);
                let const_name = expr_to_name(access.member);
                let resolved = ctx.resolve_class_name(&class_name);
                if let Some(class_def) = self.registry.find_class(&resolved) {
                    for c in &class_def.constants {
                        if c.name == const_name {
                            return Ok(c
                                .value
                                .as_ref()
                                .map(|v| parse_default_value(v))
                                .unwrap_or(Value::Null));
                        }
                    }
                }
                Ok(Value::Null)
            }

            /// 数组字面量：[1, 2, 3] 或 ['key' => 'value']
            ExprKind::Array(elements) => {
                let has_string_key = elements.iter().any(|elem| {
                    matches!(
                        &elem.key,
                        Some(key) if matches!(&key.kind, ExprKind::String(_))
                    )
                });
                if has_string_key {
                    let mut map = Vec::new();
                    for elem in elements.iter() {
                        let key = match &elem.key {
                            Some(key_expr) => self.eval_expr(key_expr, ctx)?.to_string_value(),
                            None => continue,
                        };
                        let value = self.eval_expr(&elem.value, ctx)?;
                        map.push((key, value));
                    }
                    Ok(Value::AssociativeArray(map))
                } else {
                    let mut arr = Vec::new();
                    for elem in elements.iter() {
                        if !elem.unpack {
                            arr.push(self.eval_expr(&elem.value, ctx)?);
                        }
                    }
                    Ok(Value::IndexedArray(arr))
                }
            }

            /// 数组访问：$arr[0], $arr['key']
            ExprKind::ArrayAccess(access) => {
                let array = self.eval_expr(access.array, ctx)?;
                match &access.index {
                    Some(index_expr) => {
                        let index = self.eval_expr(index_expr, ctx)?;
                        match &array {
                            Value::IndexedArray(arr) => Ok(arr
                                .get(index.to_int() as usize)
                                .cloned()
                                .unwrap_or(Value::Null)),
                            Value::AssociativeArray(map) => {
                                let key = index.to_string_value();
                                Ok(map
                                    .iter()
                                    .find(|(k, _)| k == &key)
                                    .map(|(_, v)| v.clone())
                                    .unwrap_or(Value::Null))
                            }
                            Value::String(s) => {
                                let idx = index.to_int() as usize;
                                Ok(s.chars()
                                    .nth(idx)
                                    .map(|c| Value::String(c.to_string()))
                                    .unwrap_or(Value::Null))
                            }
                            _ => Ok(Value::Null),
                        }
                    }
                    None => Ok(Value::Null),
                }
            }

            /// new 表达式：new ClassName($arg)
            ExprKind::New(new_expr) => {
                let class_name = expr_to_name(new_expr.class);
                let resolved = ctx.resolve_class_name(&class_name);
                let mut instance = ObjectInstance {
                    class_name: resolved.clone(),
                    properties: HashMap::new(),
                };
                if let Some(class_def) = self.registry.find_class(&resolved) {
                    for prop in &class_def.properties {
                        let default_value = match &prop.default_value {
                            Some(v) => parse_default_value(v),
                            None => Value::Null,
                        };
                        instance.properties.insert(prop.name.clone(), default_value);
                    }
                }
                Ok(Value::Object(instance))
            }

            /// 匿名类：new class { ... }
            ExprKind::AnonymousClass(class_decl) => {
                let class_name = format!("anonymous_class_{}", self.next_closure_id());
                let mut instance = ObjectInstance {
                    class_name: class_name.clone(),
                    properties: HashMap::new(),
                };
                for member in class_decl.members.iter() {
                    if let ClassMemberKind::Property(prop) = &member.kind {
                        let prop_name = prop.name.to_string();
                        let default_value = prop
                            .default
                            .as_ref()
                            .and_then(|e| Some(self.eval_expr(e, ctx).unwrap_or(Value::Null)))
                            .unwrap_or(Value::Null);
                        instance.properties.insert(prop_name, default_value);
                    }
                }
                Ok(Value::Object(instance))
            }

            /// 闭包：function($x) use ($y) { return $x + $y; }
            ExprKind::Closure(closure) => {
                let id = self.next_closure_id();
                let params: Vec<String> = closure
                    .params
                    .iter()
                    .map(|p| p.name.trim_start_matches('$').to_string())
                    .collect();
                let captured_vars: Vec<String> = closure
                    .use_vars
                    .iter()
                    .map(|v| v.name.trim_start_matches('$').to_string())
                    .collect();
                let captured = ctx.capture_vars(&captured_vars);
                let file_path = ctx
                    .this
                    .as_ref()
                    .map(|i| {
                        self.registry
                            .find_class(&i.class_name)
                            .map(|c| c.file_path.clone())
                            .unwrap_or_default()
                    })
                    .unwrap_or_default();
                Ok(Value::Callable(CallableValue::Closure {
                    id,
                    params,
                    captured,
                    file_path,
                    by_ref: closure.by_ref,
                }))
            }

            /// 箭头函数：fn($x) => $x * 2
            /// 自动捕获外层所有变量
            ExprKind::ArrowFunction(arrow) => {
                let id = self.next_closure_id();
                let params: Vec<String> = arrow
                    .params
                    .iter()
                    .map(|p| p.name.trim_start_matches('$').to_string())
                    .collect();
                let captured = ctx.capture_all_vars();
                let file_path = ctx
                    .this
                    .as_ref()
                    .map(|i| {
                        self.registry
                            .find_class(&i.class_name)
                            .map(|c| c.file_path.clone())
                            .unwrap_or_default()
                    })
                    .unwrap_or_default();
                Ok(Value::Callable(CallableValue::Arrow {
                    id,
                    params,
                    captured,
                    file_path,
                }))
            }

            /// match 表达式：match($val) { 1 => 'one', 2 => 'two', default => 'other' }
            ExprKind::Match(match_expr) => {
                let subject = self.eval_expr(match_expr.subject, ctx)?;
                for arm in match_expr.arms.iter() {
                    match &arm.conditions {
                        Some(conditions) => {
                            for cond in conditions.iter() {
                                let cond_val = self.eval_expr(cond, ctx)?;
                                if values_identical(&subject, &cond_val) {
                                    return self.eval_expr(&arm.body, ctx);
                                }
                            }
                        }
                        None => {
                            return self.eval_expr(&arm.body, ctx);
                        }
                    }
                }
                Ok(Value::Null)
            }

            /// throw 表达式（PHP 8.0+）：$error ?? throw new Exception()
            ExprKind::ThrowExpr(expr) => {
                let _value = self.eval_expr(expr, ctx)?;
                Ok(Value::Null)
            }

            /// 标识符：类名、常量名等
            ExprKind::Identifier(id) => {
                let name = id.as_str();
                if let Some(val) = ctx.get_var(name) {
                    return Ok(val.clone());
                }
                if let Some(const_def) = self.registry.find_constant(name) {
                    if let Some(v) = &const_def.value {
                        return Ok(parse_default_value(v));
                    }
                }
                Ok(Value::String(name.to_string()))
            }

            /// print 表达式：print "hello"
            ExprKind::Print(_expr) => Ok(Value::Int(1)),

            /// 括号表达式：($a + $b)
            ExprKind::Parenthesized(expr) => self.eval_expr(expr, ctx),

            /// isset 表达式：isset($var), isset($arr['key'])
            ExprKind::Isset(vars) => {
                for var in vars.iter() {
                    if let ExprKind::Variable(name) = &var.kind {
                        let var_name = name.as_str().trim_start_matches('$');
                        if ctx.get_var(var_name).is_none() {
                            return Ok(Value::Bool(false));
                        }
                        if let Some(Value::Null) = ctx.get_var(var_name) {
                            return Ok(Value::Bool(false));
                        }
                    }
                }
                Ok(Value::Bool(true))
            }

            /// empty 表达式：empty($var)
            ExprKind::Empty(expr) => {
                let val = self.eval_expr(expr, ctx)?;
                Ok(Value::Bool(!val.is_truthy()))
            }

            /// 错误抑制运算符：@expr（忽略表达式产生的错误）
            ExprKind::ErrorSuppress(expr) => self.eval_expr(expr, ctx),

            /// exit/die 表达式：exit(1), die("error")
            ExprKind::Exit(code) => match code {
                Some(e) => {
                    let val = self.eval_expr(e, ctx)?;
                    Ok(Value::String(val.to_string_value()))
                }
                None => Ok(Value::String(String::new())),
            },

            /// clone 表达式：clone $obj
            ExprKind::Clone(expr) => {
                let val = self.eval_expr(expr, ctx)?;
                Ok(val)
            }

            /// 魔术常量：__LINE__, __FILE__, __DIR__, __NAMESPACE__ 等
            ExprKind::MagicConst(kind) => match kind {
                MagicConstKind::Line => Ok(Value::Int(0)),
                MagicConstKind::File => Ok(Value::String(String::new())),
                MagicConstKind::Dir => Ok(Value::String(String::new())),
                MagicConstKind::Namespace => {
                    Ok(Value::String(ctx.namespace.clone().unwrap_or_default()))
                }
                MagicConstKind::Class => Ok(Value::String(
                    ctx.this
                        .as_ref()
                        .map(|i| i.class_name.clone())
                        .unwrap_or_default(),
                )),
                MagicConstKind::Function => Ok(Value::String(String::new())),
                MagicConstKind::Method => Ok(Value::String(String::new())),
                MagicConstKind::Trait => Ok(Value::String(String::new())),
                MagicConstKind::Property => Ok(Value::String(String::new())),
            },

            /// 第一类可调用创建：strlen(...)（PHP 8.1+）
            ExprKind::CallableCreate(_) => Ok(Value::Callable(CallableValue::Function {
                name: "callable".to_string(),
            })),

            /// 省略表达式（参数占位符）
            ExprKind::Omit => Ok(Value::Null),

            /// 其他未处理的表达式类型
            _ => Ok(Value::Null),
        }
    }
}
