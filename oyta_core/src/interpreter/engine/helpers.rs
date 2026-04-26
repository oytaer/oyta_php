//! 辅助函数模块
//!
//! 提供解释器各模块共用的工具函数
//! 包括：变量名提取、表达式名称转换、默认值解析

use crate::interpreter::value::Value;

/// 从表达式中提取变量名
///
/// 仅处理 ExprKind::Variable 类型，提取去掉 $ 前缀的变量名
/// 例如：$name → name
///
/// # 参数
/// - `expr`: PHP AST 表达式引用
///
/// # 返回
/// 变量名字符串，非变量表达式返回空字符串
pub fn expr_var_name<'arena, 'src>(expr: &php_ast::ast::Expr<'arena, 'src>) -> String {
    match &expr.kind {
        php_ast::ast::ExprKind::Variable(var) => var.as_str().trim_start_matches('$').to_string(),
        _ => String::new(),
    }
}

/// 将表达式转换为名称字符串
///
/// 处理两种情况：
/// - ExprKind::Identifier: 标识符名称（如类名、函数名）
/// - ExprKind::Variable: 变量名（含 $ 前缀）
///
/// # 参数
/// - `expr`: PHP AST 表达式引用
///
/// # 返回
/// 名称字符串，无法识别的表达式返回空字符串
pub fn expr_to_name<'arena, 'src>(expr: &php_ast::ast::Expr<'arena, 'src>) -> String {
    match &expr.kind {
        php_ast::ast::ExprKind::Identifier(id) => id.as_str().to_string(),
        php_ast::ast::ExprKind::Variable(var) => var.as_str().to_string(),
        _ => String::new(),
    }
}

/// 解析默认值字符串为 Value
///
/// 将符号表中存储的默认值字符串表示转换为运行时值
/// 支持的类型：
/// - "null" → Null
/// - "true" / "false" → Bool
/// - 整数字符串 → Int
/// - 浮点数字符串 → Float
/// - 单引号/双引号字符串 → String（去掉引号）
/// - 其他 → String（原样返回）
///
/// # 参数
/// - `s`: 默认值的字符串表示
///
/// # 返回
/// 对应的 Value 枚举值
pub fn parse_default_value(s: &str) -> Value {
    match s {
        "null" => Value::Null,
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => {
            if let Ok(i) = s.parse::<i64>() {
                return Value::Int(i);
            }
            if let Ok(f) = s.parse::<f64>() {
                return Value::Float(f);
            }
            if s.starts_with('\'') && s.ends_with('\'') {
                return Value::String(s[1..s.len() - 1].to_string());
            }
            if s.starts_with('"') && s.ends_with('"') {
                return Value::String(s[1..s.len() - 1].to_string());
            }
            Value::String(s.to_string())
        }
    }
}
