//! 配置文件解析模块
//!
//! 提供 PHP 配置文件的解析功能
//! 支持 env() 函数调用，从环境变量中获取值

use php_ast::ast::{Expr, ExprKind, UnaryPrefixOp};
use crate::symbol_table::types::ConfigValue;

use super::expr::expr_to_string;

/// 评估配置表达式，将 AST 表达式转换为 ConfigValue
///
/// 递归处理嵌套数组和字面量
/// 支持 env() 函数调用，从环境变量中获取值
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

        // 函数调用 - 处理 env() 函数
        ExprKind::FunctionCall(call) => {
            let func_name = expr_to_string(call.name);
            
            match func_name.as_str() {
                "env" | "\\env" => {
                    // 处理 env() 函数调用
                    if call.args.is_empty() {
                        return ConfigValue::Null;
                    }
                    
                    // 获取第一个参数 - 环境变量名
                    let env_key = match &call.args[0].value.kind {
                        ExprKind::String(s) => s.to_string(),
                        _ => {
                            tracing::warn!("env() 第一个参数必须是字符串");
                            return ConfigValue::Null;
                        }
                    };
                    
                    // 从环境变量中获取值
                    if let Ok(env_value) = std::env::var(&env_key) {
                        // 根据值的格式推断类型
                        return parse_env_value(&env_value);
                    }
                    
                    // 环境变量不存在，使用默认值（第二个参数）
                    if call.args.len() > 1 {
                        return eval_config_expr(&call.args[1].value);
                    }
                    
                    ConfigValue::Null
                }
                _ => {
                    // 其他函数调用暂不处理
                    tracing::debug!("配置中不支持函数调用: {}", func_name);
                    ConfigValue::Null
                }
            }
        }

        // 其他复杂表达式暂不处理
        _ => ConfigValue::Null,
    }
}

/// 解析环境变量值，根据格式推断类型
///
/// # 参数
/// - `value`: 环境变量字符串值
///
/// # 返回
/// 推断后的配置值
fn parse_env_value(value: &str) -> ConfigValue {
    // 布尔值
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => return ConfigValue::Bool(true),
        "false" | "0" | "no" | "off" | "" => return ConfigValue::Bool(false),
        _ => {}
    }

    // 整数
    if let Ok(i) = value.parse::<i64>() {
        return ConfigValue::Int(i);
    }

    // 浮点数
    if let Ok(f) = value.parse::<f64>() {
        return ConfigValue::Float(f);
    }

    // 默认为字符串
    ConfigValue::String(value.to_string())
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
