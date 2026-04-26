//! 表达式处理模块
//!
//! 提供表达式到字符串和配置值的转换

use php_ast::ast::{Expr, ExprKind, UnaryPrefixOp};
use crate::symbol_table::types::ConfigValue;

/// 将表达式转换为字符串表示（用于默认值等场景）
///
/// 这是一个简化实现，只处理常见的字面量表达式
///
/// # 参数
/// - `expr`: AST 表达式引用
///
/// # 返回
/// 表达式的字符串表示
pub fn expr_to_string<'arena, 'src>(expr: &Expr<'arena, 'src>) -> String {
    match &expr.kind {
        // 整数字面量
        ExprKind::Int(i) => i.to_string(),
        // 浮点数字面量
        ExprKind::Float(f) => f.to_string(),
        // 字符串字面量
        ExprKind::String(s) => format!("'{}'", s),
        // 布尔字面量
        ExprKind::Bool(b) => b.to_string(),
        // null 字面量
        ExprKind::Null => "null".to_string(),
        // 数组字面量
        ExprKind::Array(elements) => {
            let items: Vec<String> = elements.iter().map(|elem| {
                let value_str = expr_to_string(&elem.value);
                match &elem.key {
                    Some(key) => format!("{} => {}", expr_to_string(key), value_str),
                    None => value_str,
                }
            }).collect();
            format!("[{}]", items.join(", "))
        }
        // 变量
        ExprKind::Variable(var) => {
            format!("${}", var.as_str())
        }
        // 标识符（常量名、类名等）
        ExprKind::Identifier(id) => id.as_str().to_string(),
        // 一元前缀运算符
        ExprKind::UnaryPrefix(unary) => {
            let op = match unary.op {
                UnaryPrefixOp::Negate => "-",
                UnaryPrefixOp::Plus => "+",
                UnaryPrefixOp::BooleanNot => "!",
                UnaryPrefixOp::BitwiseNot => "~",
                UnaryPrefixOp::PreIncrement => "++",
                UnaryPrefixOp::PreDecrement => "--",
            };
            format!("{}{}", op, expr_to_string(unary.operand))
        }
        // 三元运算符
        ExprKind::Ternary(ternary) => {
            match &ternary.then_expr {
                Some(then) => format!(
                    "{} ? {} : {}",
                    expr_to_string(ternary.condition),
                    expr_to_string(then),
                    expr_to_string(ternary.else_expr)
                ),
                None => format!(
                    "{} ?: {}",
                    expr_to_string(ternary.condition),
                    expr_to_string(ternary.else_expr)
                ),
            }
        }
        // null 合并运算符
        ExprKind::NullCoalesce(nc) => {
            format!("{} ?? {}", expr_to_string(nc.left), expr_to_string(nc.right))
        }
        // 函数调用
        ExprKind::FunctionCall(call) => {
            let args: Vec<String> = call.args.iter().map(|arg| {
                expr_to_string(&arg.value)
            }).collect();
            format!("{}({})", expr_to_string(call.name), args.join(", "))
        }
        // 类常量访问
        ExprKind::ClassConstAccess(access) => {
            format!("{}::{}", expr_to_string(&access.class), expr_to_string(&access.member))
        }
        // 静态属性访问
        ExprKind::StaticPropertyAccess(access) => {
            format!("{}::${}", expr_to_string(&access.class), expr_to_string(&access.member))
        }
        // 其他复杂表达式
        _ => {
            // 对于复杂的表达式，返回占位符
            // 后续阶段在解释器中会完整处理
            "/* complex expr */".to_string()
        }
    }
}

/// 将类型提示转换为字符串
///
/// # 参数
/// - `type_hint`: 类型提示引用
///
/// # 返回
/// 类型提示的字符串表示
pub fn type_hint_to_string<'arena, 'src>(type_hint: &php_ast::ast::TypeHint<'arena, 'src>) -> String {
    match &type_hint.kind {
        // 命名类型（类名、接口名等）
        php_ast::ast::TypeHintKind::Named(name) => name.to_string_repr().to_string(),
        // 内置类型关键字
        php_ast::ast::TypeHintKind::Keyword(builtin, _) => builtin.as_str().to_string(),
        // 可空类型
        php_ast::ast::TypeHintKind::Nullable(inner) => format!("?{}", type_hint_to_string(inner)),
        // 联合类型
        php_ast::ast::TypeHintKind::Union(types) => {
            types.iter()
                .map(|t| type_hint_to_string(t))
                .collect::<Vec<_>>()
                .join("|")
        }
        // 交集类型
        php_ast::ast::TypeHintKind::Intersection(types) => {
            types.iter()
                .map(|t| type_hint_to_string(t))
                .collect::<Vec<_>>()
                .join("&")
        }
    }
}
