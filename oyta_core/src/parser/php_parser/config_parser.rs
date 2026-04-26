//! 配置文件解析模块
//!
//! 提供 PHP 配置文件的解析功能

use php_ast::ast::{Expr, ExprKind, UnaryPrefixOp};
use crate::symbol_table::types::ConfigValue;

use super::expr::expr_to_string;

/// 评估配置表达式，将 AST 表达式转换为 ConfigValue
///
/// 递归处理嵌套数组和字面量
///
/// # 参数
/// - `expr`: AST 表达式引用
///
/// # 返回
/// 配置值
pub fn eval_config_expr<'arena, 'src>(expr: &Expr<'arena, 'src>) -> ConfigValue {
    match &expr.kind {
        // 整数字面量
        ExprKind::Int(i) => ConfigValue::Int(*i),
        // 浮点数字面量
        ExprKind::Float(f) => ConfigValue::Float(*f),
        // 字符串字面量
        ExprKind::String(s) => ConfigValue::String(s.to_string()),
        // 布尔字面量
        ExprKind::Bool(b) => ConfigValue::Bool(*b),
        // null 字面量
        ExprKind::Null => ConfigValue::Null,

        // 数组类型 - 需要区分索引数组和关联数组
        ExprKind::Array(elements) => {
            // 检查是否所有元素都有字符串键（关联数组）
            let has_string_key = elements.iter().any(|elem| {
                matches!(&elem.key, Some(key) if matches!(&key.kind, ExprKind::String(_)))
            });

            if has_string_key {
                // 关联数组
                let mut map = Vec::new();
                for elem in elements.iter() {
                    let key = match &elem.key {
                        Some(key_expr) => expr_to_config_key(key_expr),
                        None => continue,
                    };
                    let value = eval_config_expr(&elem.value);
                    map.push((key, value));
                }
                ConfigValue::AssociativeArray(map)
            } else {
                // 索引数组
                let mut arr = Vec::new();
                for elem in elements.iter() {
                    let value = eval_config_expr(&elem.value);
                    arr.push(value);
                }
                ConfigValue::IndexedArray(arr)
            }
        }

        // 负数
        ExprKind::UnaryPrefix(unary) => {
            match unary.op {
                UnaryPrefixOp::Negate => {
                    match eval_config_expr(unary.operand) {
                        ConfigValue::Int(i) => ConfigValue::Int(-i),
                        ConfigValue::Float(f) => ConfigValue::Float(-f),
                        other => other,
                    }
                }
                _ => ConfigValue::Null,
            }
        }

        // 其他复杂表达式暂不处理
        _ => ConfigValue::Null,
    }
}

/// 将表达式转换为配置键名
///
/// # 参数
/// - `expr`: AST 表达式引用
///
/// # 返回
/// 键名字符串
pub fn expr_to_config_key<'arena, 'src>(expr: &Expr<'arena, 'src>) -> String {
    match &expr.kind {
        // 字符串键
        ExprKind::String(s) => s.to_string(),
        // 整数键
        ExprKind::Int(i) => i.to_string(),
        // 其他类型
        _ => format!("{:?}", expr.kind),
    }
}
