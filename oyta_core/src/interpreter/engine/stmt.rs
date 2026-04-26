//! 语句执行模块
//!
//! 实现 PHP 的所有语句类型执行逻辑
//! 包括：表达式语句、return、echo、if/elseif/else、while、do-while、for、foreach、
//! switch/case、try/catch/finally、throw、break、continue、block、nop、
//! global、static、unset、namespace、use、const、inline_html

use anyhow::Result;
use php_ast::ast::*;

use crate::interpreter::value::{
    ExecutionResult, ExecutionContext, Value,
};

use super::Interpreter;
use super::binary_op::values_equal;
use super::helpers::expr_var_name;

impl Interpreter {
    /// 执行单个语句
    ///
    /// 根据 StmtKind 分发到对应的处理逻辑
    /// 每种语句类型返回 ExecutionResult 表示执行结果
    ///
    /// # 参数
    /// - `stmt`: PHP AST 语句引用
    /// - `ctx`: 执行上下文（可变借用）
    ///
    /// # 返回
    /// 执行结果（Value/Return/Break/Continue/Throw/FunctionCall）
    pub(crate) fn run_stmt<'arena, 'src>(
        &mut self,
        stmt: &Stmt<'arena, 'src>,
        ctx: &mut ExecutionContext,
    ) -> Result<ExecutionResult> {
        match &stmt.kind {
            /// 表达式语句：$a = 1; foo();
            StmtKind::Expression(expr) => {
                let value = self.eval_expr(expr, ctx)?;
                Ok(ExecutionResult::Value(value))
            }

            /// return 语句：return $value;
            StmtKind::Return(expr) => {
                let value = match expr {
                    Some(e) => self.eval_expr(e, ctx)?,
                    None => Value::Null,
                };
                Ok(ExecutionResult::Return(value))
            }

            /// echo 语句：echo "hello", "world";
            StmtKind::Echo(exprs) => {
                let mut output = String::new();
                for expr in exprs.iter() {
                    let value = self.eval_expr(expr, ctx)?;
                    output.push_str(&value.to_string_value());
                }
                Ok(ExecutionResult::Value(Value::String(output)))
            }

            /// if/elseif/else 语句
            StmtKind::If(if_stmt) => {
                let condition = self.eval_expr(&if_stmt.condition, ctx)?;
                if condition.is_truthy() {
                    self.run_stmt(if_stmt.then_branch, ctx)
                } else {
                    for elseif in if_stmt.elseif_branches.iter() {
                        let else_cond = self.eval_expr(&elseif.condition, ctx)?;
                        if else_cond.is_truthy() {
                            return self.run_stmt(&elseif.body, ctx);
                        }
                    }
                    if let Some(else_branch) = if_stmt.else_branch {
                        self.run_stmt(else_branch, ctx)
                    } else {
                        Ok(ExecutionResult::Value(Value::Null))
                    }
                }
            }

            /// while 循环：while ($cond) { ... }
            StmtKind::While(while_stmt) => {
                let mut result = Value::Null;
                loop {
                    let cond = self.eval_expr(&while_stmt.condition, ctx)?;
                    if !cond.is_truthy() {
                        break;
                    }
                    match self.run_stmt(while_stmt.body, ctx)? {
                        ExecutionResult::Break(_) => break,
                        ExecutionResult::Continue(_) => continue,
                        ExecutionResult::Return(v) => return Ok(ExecutionResult::Return(v)),
                        ExecutionResult::Throw(v) => return Ok(ExecutionResult::Throw(v)),
                        ExecutionResult::Value(v) => result = v,
                        ExecutionResult::FunctionCall { name, args } => {
                            result = self.call_function(&name, args)?;
                        }
                    }
                }
                Ok(ExecutionResult::Value(result))
            }

            /// do-while 循环：do { ... } while ($cond);
            StmtKind::DoWhile(do_while) => {
                let mut result = Value::Null;
                loop {
                    match self.run_stmt(do_while.body, ctx)? {
                        ExecutionResult::Break(_) => break,
                        ExecutionResult::Continue(_) => {}
                        ExecutionResult::Return(v) => return Ok(ExecutionResult::Return(v)),
                        ExecutionResult::Throw(v) => return Ok(ExecutionResult::Throw(v)),
                        ExecutionResult::Value(v) => result = v,
                        ExecutionResult::FunctionCall { name, args } => {
                            result = self.call_function(&name, args)?;
                        }
                    }
                    let cond = self.eval_expr(&do_while.condition, ctx)?;
                    if !cond.is_truthy() {
                        break;
                    }
                }
                Ok(ExecutionResult::Value(result))
            }

            /// for 循环：for ($i=0; $i<10; $i++) { ... }
            StmtKind::For(for_stmt) => {
                for init in for_stmt.init.iter() {
                    self.eval_expr(init, ctx)?;
                }
                let mut result = Value::Null;
                loop {
                    if !for_stmt.condition.is_empty() {
                        let cond = self.eval_expr(&for_stmt.condition[0], ctx)?;
                        if !cond.is_truthy() {
                            break;
                        }
                    }
                    match self.run_stmt(for_stmt.body, ctx)? {
                        ExecutionResult::Break(_) => break,
                        ExecutionResult::Continue(_) => {}
                        ExecutionResult::Return(v) => return Ok(ExecutionResult::Return(v)),
                        ExecutionResult::Throw(v) => return Ok(ExecutionResult::Throw(v)),
                        ExecutionResult::Value(v) => result = v,
                        ExecutionResult::FunctionCall { name, args } => {
                            result = self.call_function(&name, args)?;
                        }
                    }
                    for update in for_stmt.update.iter() {
                        self.eval_expr(update, ctx)?;
                    }
                }
                Ok(ExecutionResult::Value(result))
            }

            /// foreach 循环：foreach ($arr as $key => $value) { ... }
            StmtKind::Foreach(foreach_stmt) => {
                let iterable = self.eval_expr(&foreach_stmt.expr, ctx)?;
                let mut result = Value::Null;

                let items: Vec<(Option<Value>, Value)> = match &iterable {
                    Value::IndexedArray(arr) => arr
                        .iter()
                        .enumerate()
                        .map(|(i, v)| (Some(Value::Int(i as i64)), v.clone()))
                        .collect(),
                    Value::AssociativeArray(map) => map
                        .iter()
                        .map(|(k, v)| (Some(Value::String(k.clone())), v.clone()))
                        .collect(),
                    _ => return Ok(ExecutionResult::Value(Value::Null)),
                };

                for (key, value) in items {
                    if let Some(key_expr) = &foreach_stmt.key {
                        let var_name = expr_var_name(key_expr);
                        ctx.set_var(&var_name, key.unwrap_or(Value::Null));
                    }
                    let val_name = expr_var_name(&foreach_stmt.value);
                    ctx.set_var(&val_name, value);

                    match self.run_stmt(foreach_stmt.body, ctx)? {
                        ExecutionResult::Break(_) => break,
                        ExecutionResult::Continue(_) => continue,
                        ExecutionResult::Return(v) => return Ok(ExecutionResult::Return(v)),
                        ExecutionResult::Throw(v) => return Ok(ExecutionResult::Throw(v)),
                        ExecutionResult::Value(v) => result = v,
                        ExecutionResult::FunctionCall { name, args } => {
                            result = self.call_function(&name, args)?;
                        }
                    }
                }
                Ok(ExecutionResult::Value(result))
            }

            /// switch/case 语句
            StmtKind::Switch(switch_stmt) => {
                let subject = self.eval_expr(&switch_stmt.expr, ctx)?;
                let mut matched = false;
                let mut result = Value::Null;

                for case in switch_stmt.cases.iter() {
                    if !matched {
                        match &case.value {
                            Some(case_expr) => {
                                let case_val = self.eval_expr(case_expr, ctx)?;
                                if values_equal(&subject, &case_val) {
                                    matched = true;
                                }
                            }
                            None => {
                                matched = true;
                            }
                        }
                    }

                    if matched {
                        for stmt in case.body.iter() {
                            match self.run_stmt(stmt, ctx)? {
                                ExecutionResult::Break(_) => {
                                    return Ok(ExecutionResult::Value(result))
                                }
                                ExecutionResult::Return(v) => {
                                    return Ok(ExecutionResult::Return(v))
                                }
                                ExecutionResult::Throw(v) => {
                                    return Ok(ExecutionResult::Throw(v))
                                }
                                ExecutionResult::Continue(_) => {}
                                ExecutionResult::Value(v) => result = v,
                                ExecutionResult::FunctionCall { name, args } => {
                                    result = self.call_function(&name, args)?;
                                }
                            }
                        }
                    }
                }

                Ok(ExecutionResult::Value(result))
            }

            /// try/catch/finally 语句
            StmtKind::TryCatch(try_stmt) => {
                let mut result = Value::Null;
                let mut caught = false;

                match self.run_stmts(&try_stmt.body, ctx) {
                    Ok(ExecutionResult::Throw(throw_val)) => {
                        for catch_clause in try_stmt.catches.iter() {
                            let type_names: Vec<String> = catch_clause
                                .types
                                .iter()
                                .map(|t| t.to_string_repr().to_string())
                                .collect();
                            let matches = type_names.iter().any(|t| {
                                t == "Exception"
                                    || t == "Throwable"
                                    || t == "\\Exception"
                                    || t == "\\Throwable"
                            }) || type_names.is_empty();
                            if matches {
                                if let Some(var_name) = catch_clause.var {
                                    let var = var_name.trim_start_matches('$').to_string();
                                    ctx.set_var(&var, throw_val.clone());
                                }
                                match self.run_stmts(&catch_clause.body, ctx) {
                                    Ok(r) => {
                                        result = r.into_value();
                                        caught = true;
                                        break;
                                    }
                                    Err(e) => return Err(e),
                                }
                            }
                        }
                        if !caught {
                            if let Some(finally_body) = &try_stmt.finally {
                                self.run_stmts(finally_body, ctx)?;
                            }
                            return Ok(ExecutionResult::Throw(throw_val));
                        }
                    }
                    Ok(r) => {
                        result = r.into_value();
                    }
                    Err(e) => return Err(e),
                }

                if let Some(finally_body) = &try_stmt.finally {
                    self.run_stmts(finally_body, ctx)?;
                }

                Ok(ExecutionResult::Value(result))
            }

            /// throw 语句：throw new Exception("error");
            StmtKind::Throw(expr) => {
                let value = self.eval_expr(expr, ctx)?;
                Ok(ExecutionResult::Throw(value))
            }

            /// break 语句：break; 或 break 2;
            StmtKind::Break(level) => {
                let lvl = level
                    .as_ref()
                    .and_then(|e| match e.kind {
                        ExprKind::Int(i) => Some(i as i32),
                        _ => None,
                    })
                    .unwrap_or(1);
                Ok(ExecutionResult::Break(Some(lvl)))
            }

            /// continue 语句：continue; 或 continue 2;
            StmtKind::Continue(level) => {
                let lvl = level
                    .as_ref()
                    .and_then(|e| match e.kind {
                        ExprKind::Int(i) => Some(i as i32),
                        _ => None,
                    })
                    .unwrap_or(1);
                Ok(ExecutionResult::Continue(Some(lvl)))
            }

            /// 语句块：{ stmt1; stmt2; }
            StmtKind::Block(stmts) => self.run_stmts(stmts, ctx),

            /// 空语句：;
            StmtKind::Nop => Ok(ExecutionResult::Value(Value::Null)),

            /// global 语句：global $var1, $var2;
            StmtKind::Global(vars) => {
                for var in vars.iter() {
                    if let ExprKind::Variable(name) = &var.kind {
                        let var_name = name.as_str().trim_start_matches('$').to_string();
                        if !ctx.has_var(&var_name) {
                            ctx.set_var(&var_name, Value::Null);
                        }
                    }
                }
                Ok(ExecutionResult::Value(Value::Null))
            }

            /// static 语句：static $count = 0;
            StmtKind::StaticVar(vars) => {
                for sv in vars.iter() {
                    let var_name = sv.name.trim_start_matches('$').to_string();
                    let value = match &sv.default {
                        Some(e) => self.eval_expr(e, ctx)?,
                        None => Value::Null,
                    };
                    ctx.set_var(&var_name, value);
                }
                Ok(ExecutionResult::Value(Value::Null))
            }

            /// unset 语句：unset($var1, $var2);
            StmtKind::Unset(vars) => {
                for var in vars.iter() {
                    if let ExprKind::Variable(name) = &var.kind {
                        let var_name = name.as_str().trim_start_matches('$').to_string();
                        ctx.remove_var(&var_name);
                    }
                }
                Ok(ExecutionResult::Value(Value::Null))
            }

            /// namespace 声明：namespace App\Controller;
            StmtKind::Namespace(ns_decl) => {
                if let Some(name) = &ns_decl.name {
                    ctx.namespace = Some(name.to_string_repr().to_string());
                }
                Ok(ExecutionResult::Value(Value::Null))
            }

            /// use 声明：use App\Controller\Index;
            StmtKind::Use(use_decl) => {
                for item in use_decl.uses.iter() {
                    let full_name = item.name.to_string_repr().to_string();
                    let alias = item
                        .alias
                        .map(|a| a.to_string())
                        .unwrap_or_else(|| full_name.split('\\').last().unwrap_or(&full_name).to_string());
                    ctx.use_map.insert(alias, full_name);
                }
                Ok(ExecutionResult::Value(Value::Null))
            }

            /// const 声明：const MAX_SIZE = 100;
            StmtKind::Const(items) => {
                for item in items.iter() {
                    let const_name = item.name.to_string();
                    let value = self.eval_expr(&item.value, ctx)?;
                    ctx.set_var(&const_name, value);
                }
                Ok(ExecutionResult::Value(Value::Null))
            }

            /// 内联 HTML：<?php ... ?> <div>html</div> <?php ... ?>
            StmtKind::InlineHtml(html) => Ok(ExecutionResult::Value(Value::String(html.to_string()))),

            /// 其他未处理的语句类型
            _ => Ok(ExecutionResult::Value(Value::Null)),
        }
    }
}
