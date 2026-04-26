//! 赋值操作模块
//!
//! 实现 PHP 的各种赋值目标处理逻辑
//! 包括：变量赋值、可变变量赋值、属性赋值、数组元素赋值、静态属性赋值

use anyhow::Result;
use php_ast::ast::*;

use crate::interpreter::value::{ExecutionContext, Value};

use super::Interpreter;
use super::helpers::expr_to_name;

impl Interpreter {
    /// 赋值到目标表达式
    ///
    /// 根据 target 的表达式类型将 value 赋值到对应位置
    /// 支持的赋值目标：
    /// - ExprKind::Variable: $a = value
    /// - ExprKind::VariableVariable: $$name = value
    /// - ExprKind::PropertyAccess: $this->prop = value
    /// - ExprKind::ArrayAccess: $arr[0] = value, $arr['key'] = value
    /// - ExprKind::StaticPropertyAccess: ClassName::$prop = value
    pub(crate) fn assign_to<'arena, 'src>(
        &mut self,
        target: &Expr<'arena, 'src>,
        value: Value,
        ctx: &mut ExecutionContext,
    ) -> Result<()> {
        match &target.kind {
            // 普通变量赋值：$a = value
            ExprKind::Variable(var) => {
                let name = var.as_str().trim_start_matches('$');
                ctx.set_var(name, value);
                Ok(())
            }

            // 可变变量赋值：$$name = value
            // 先求值内部表达式得到变量名，再赋值
            ExprKind::VariableVariable(inner) => {
                let name_val = self.eval_expr(inner, ctx)?;
                let name = name_val
                    .to_string_value()
                    .trim_start_matches('$')
                    .to_string();
                ctx.set_var(&name, value);
                Ok(())
            }

            // 属性赋值：$this->prop = value
            // 仅处理 $this 的情况，将值设置到当前对象实例的属性中
            ExprKind::PropertyAccess(prop) => {
                if let ExprKind::Variable(var) = &prop.object.kind {
                    if var.as_str() == "this" {
                        if let Some(ref mut instance) = ctx.this {
                            let prop_name = expr_to_name(prop.property)
                                .trim_start_matches('$')
                                .to_string();
                            instance.properties.insert(prop_name, value);
                        }
                    }
                }
                Ok(())
            }

            // 数组元素赋值：$arr[0] = value, $arr['key'] = value
            // 支持索引数组和关联数组
            ExprKind::ArrayAccess(access) => {
                if let Some(index_expr) = &access.index {
                    let index = self.eval_expr(index_expr, ctx)?;
                    if let ExprKind::Variable(var) = &access.array.kind {
                        let var_name = var.as_str().trim_start_matches('$').to_string();
                        let mut current = ctx
                            .get_var(&var_name)
                            .cloned()
                            .unwrap_or(Value::IndexedArray(Vec::new()));
                        match &mut current {
                            Value::IndexedArray(arr) => {
                                let idx = index.to_int() as usize;
                                if idx < arr.len() {
                                    arr[idx] = value;
                                } else {
                                    arr.push(value);
                                }
                            }
                            Value::AssociativeArray(map) => {
                                let key = index.to_string_value();
                                if let Some(entry) = map.iter_mut().find(|(k, _)| k == &key) {
                                    entry.1 = value;
                                } else {
                                    map.push((key, value));
                                }
                            }
                            _ => {}
                        }
                        ctx.set_var(&var_name, current);
                    }
                }
                Ok(())
            }

            // 静态属性赋值：ClassName::$prop = value
            // 当前简化实现，不做实际存储
            ExprKind::StaticPropertyAccess(_access) => Ok(()),

            // 其他不支持的赋值目标
            _ => Ok(()),
        }
    }
}
