//! 二元运算符求值模块
//!
//! 实现 PHP 的所有二元运算符求值逻辑
//! 包括：算术运算、比较运算、逻辑运算、位运算、字符串拼接等

use crate::interpreter::value::Value;

/// 求值二元运算
///
/// 根据 PHP 语义对两个值执行二元运算
/// 支持的运算符：
/// - 算术：+ - * / % **（加、减、乘、除、取模、幂）
/// - 字符串：.（拼接）
/// - 比较：== != === !== < <= > >= <=>（松散/严格/飞船比较）
/// - 逻辑：&& || and or（逻辑与/或）
/// - 位运算：& | ^ << >>（按位与/或/异或/左移/右移）
pub fn eval_binary_op(op: &php_ast::ast::BinaryOp, left: &Value, right: &Value) -> Value {
    match op {
        // 加法运算：整数+整数=整数，浮点数参与则结果为浮点数，数组+数组=数组合并
        php_ast::ast::BinaryOp::Add => match (left, right) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
            (Value::Int(a), Value::Float(b)) => Value::Float(*a as f64 + b),
            (Value::Float(a), Value::Int(b)) => Value::Float(a + *b as f64),
            (Value::String(a), Value::String(b)) => Value::String(format!("{}{}", a, b)),
            (Value::IndexedArray(a), Value::IndexedArray(b)) => {
                let mut result = a.clone();
                result.extend(b.iter().cloned());
                Value::IndexedArray(result)
            }
            _ => Value::Int(0),
        },
        // 减法运算
        php_ast::ast::BinaryOp::Sub => match (left, right) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
            _ => Value::Int(0),
        },
        // 乘法运算
        php_ast::ast::BinaryOp::Mul => match (left, right) {
            (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
            (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
            _ => Value::Int(0),
        },
        // 除法运算（除零返回 Null）
        php_ast::ast::BinaryOp::Div => match (left, right) {
            (Value::Int(a), Value::Int(b)) if *b != 0 => Value::Int(a / b),
            (Value::Float(a), Value::Float(b)) if *b != 0.0 => Value::Float(a / b),
            _ => Value::Null,
        },
        // 取模运算（除零返回 Null）
        php_ast::ast::BinaryOp::Mod => match (left, right) {
            (Value::Int(a), Value::Int(b)) if *b != 0 => Value::Int(a % b),
            _ => Value::Null,
        },
        // 幂运算（结果始终为浮点数）
        php_ast::ast::BinaryOp::Pow => match (left, right) {
            (Value::Int(a), Value::Int(b)) => Value::Float((*a as f64).powf(*b as f64)),
            (Value::Float(a), Value::Float(b)) => Value::Float(a.powf(*b)),
            _ => Value::Null,
        },
        // 字符串拼接运算符（PHP 的 . 运算符）
        php_ast::ast::BinaryOp::Concat => {
            Value::String(format!(
                "{}{}",
                left.to_string_value(),
                right.to_string_value()
            ))
        }
        // 松散相等比较（PHP 的 == 运算符）
        php_ast::ast::BinaryOp::Equal => Value::Bool(values_equal(left, right)),
        // 松散不等比较（PHP 的 != 运算符）
        php_ast::ast::BinaryOp::NotEqual => Value::Bool(!values_equal(left, right)),
        // 严格相等比较（PHP 的 === 运算符，类型和值都必须相同）
        php_ast::ast::BinaryOp::Identical => Value::Bool(values_identical(left, right)),
        // 严格不等比较（PHP 的 !== 运算符）
        php_ast::ast::BinaryOp::NotIdentical => Value::Bool(!values_identical(left, right)),
        // 小于比较
        php_ast::ast::BinaryOp::Less => Value::Bool(left.to_float() < right.to_float()),
        // 小于等于比较
        php_ast::ast::BinaryOp::LessOrEqual => Value::Bool(left.to_float() <= right.to_float()),
        // 大于比较
        php_ast::ast::BinaryOp::Greater => Value::Bool(left.to_float() > right.to_float()),
        // 大于等于比较
        php_ast::ast::BinaryOp::GreaterOrEqual => {
            Value::Bool(left.to_float() >= right.to_float())
        }
        // 飞船运算符（PHP 的 <=> 运算符），返回 -1/0/1
        php_ast::ast::BinaryOp::Spaceship => {
            let cmp = left.to_float().partial_cmp(&right.to_float());
            Value::Int(match cmp {
                Some(std::cmp::Ordering::Less) => -1,
                Some(std::cmp::Ordering::Equal) => 0,
                Some(std::cmp::Ordering::Greater) => 1,
                None => 0,
            })
        }
        // 逻辑与（PHP 的 && 和 and 运算符）
        php_ast::ast::BinaryOp::BooleanAnd | php_ast::ast::BinaryOp::LogicalAnd => {
            Value::Bool(left.is_truthy() && right.is_truthy())
        }
        // 逻辑或（PHP 的 || 和 or 运算符）
        php_ast::ast::BinaryOp::BooleanOr | php_ast::ast::BinaryOp::LogicalOr => {
            Value::Bool(left.is_truthy() || right.is_truthy())
        }
        // 按位与
        php_ast::ast::BinaryOp::BitwiseAnd => Value::Int(left.to_int() & right.to_int()),
        // 按位或
        php_ast::ast::BinaryOp::BitwiseOr => Value::Int(left.to_int() | right.to_int()),
        // 按位异或
        php_ast::ast::BinaryOp::BitwiseXor => Value::Int(left.to_int() ^ right.to_int()),
        // 左移
        php_ast::ast::BinaryOp::ShiftLeft => Value::Int(left.to_int() << right.to_int()),
        // 右移
        php_ast::ast::BinaryOp::ShiftRight => Value::Int(left.to_int() >> right.to_int()),
        _ => Value::Null,
    }
}

/// PHP 松散相等比较（== 运算符）
///
/// 比较规则：
/// - 同类型直接比较值
/// - 不同类型只比较 Null==Null, Bool==Bool, Int==Int, Float==Int, String==String
/// - 其他情况返回 false（简化实现，未完全遵循 PHP 的类型转换规则）
pub fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        _ => false,
    }
}

/// PHP 严格相等比较（=== 运算符）
///
/// 要求类型和值都完全相同
/// 先检查类型判别式是否相同，再检查值是否相等
pub fn values_identical(a: &Value, b: &Value) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b) && values_equal(a, b)
}
