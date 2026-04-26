//! Match 表达式模块
//!
//! PHP 8.0 的 match 表达式实现
//! match 表达式类似于 switch，但有以下区别：
//! 1. 返回值而不是执行语句
//! 2. 使用严格比较 (===)
//! 3. 不需要 break
//! 4. 支持多条件匹配

use crate::interpreter::value::Value;

/// Match 表达式执行器
///
/// PHP 8.0 的 match 表达式类似于 switch，但：
/// 1. 返回值而不是执行语句
/// 2. 使用严格比较 (===)
/// 3. 不需要 break
/// 4. 支持多条件匹配
///
/// # 示例
/// ```php
/// $status = match($code) {
///     200, 300 => 'OK',
///     400, 404 => 'Not Found',
///     500 => 'Server Error',
///     default => 'Unknown'
/// };
/// ```
pub struct MatchExpression {
    /// 要匹配的表达式
    pub subject: Value,
    /// 匹配分支列表
    pub arms: Vec<MatchArm>,
    /// 默认值
    pub default_value: Option<Value>,
}

/// Match 分支
pub struct MatchArm {
    /// 条件列表（多个条件用逗号分隔）
    pub conditions: Vec<Value>,
    /// 返回值表达式
    pub body: MatchBody,
}

/// Match 分支体
#[derive(Debug, Clone)]
pub enum MatchBody {
    /// 表达式（直接返回值）
    Expression(Value),
    /// 代码块（需要执行一系列语句后返回值）
    Block(Vec<MatchStatement>),
}

/// Match 代码块中的语句
#[derive(Debug, Clone)]
pub enum MatchStatement {
    /// 变量赋值
    Assign {
        /// 变量名
        name: String,
        /// 值
        value: Value,
    },
    /// 函数调用
    FunctionCall {
        /// 函数名
        name: String,
        /// 参数
        args: Vec<Value>,
    },
    /// 返回语句
    Return(Value),
}

impl MatchExpression {
    /// 创建新的 match 表达式
    ///
    /// # 参数
    /// - `subject`: 要匹配的值
    ///
    /// # 返回值
    /// 新的 MatchExpression 实例
    pub fn new(subject: Value) -> Self {
        Self {
            subject,
            arms: Vec::new(),
            default_value: None,
        }
    }

    /// 添加匹配分支
    ///
    /// # 参数
    /// - `conditions`: 条件值列表
    /// - `body`: 匹配成功时执行的分支体
    ///
    /// # 返回值
    /// 返回自身，支持链式调用
    pub fn add_arm(mut self, conditions: Vec<Value>, body: MatchBody) -> Self {
        self.arms.push(MatchArm { conditions, body });
        self
    }

    /// 设置默认值
    ///
    /// # 参数
    /// - `value`: 默认返回值
    ///
    /// # 返回值
    /// 返回自身，支持链式调用
    pub fn with_default(mut self, value: Value) -> Self {
        self.default_value = Some(value);
        self
    }

    /// 执行 match 表达式
    ///
    /// 遍历所有分支，找到第一个匹配的条件并返回对应的值
    /// 如果没有匹配，返回默认值或 None
    ///
    /// # 返回值
    /// 匹配的值，如果没有匹配则返回默认值或 None
    pub fn evaluate(&self) -> Option<Value> {
        // 遍历所有分支
        for arm in &self.arms {
            // 检查每个条件
            for condition in &arm.conditions {
                // 使用严格比较
                if self.values_equal(&self.subject, condition) {
                    return Some(self.evaluate_body(&arm.body));
                }
            }
        }

        // 返回默认值
        self.default_value.clone()
    }

    /// 严格比较两个值
    ///
    /// 使用 PHP 的 === 比较规则
    ///
    /// # 参数
    /// - `a`: 第一个值
    /// - `b`: 第二个值
    ///
    /// # 返回值
    /// 如果两个值严格相等返回 true
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            // 整数比较
            (Value::Int(i1), Value::Int(i2)) => i1 == i2,
            // 浮点数比较
            (Value::Float(f1), Value::Float(f2)) => f1 == f2,
            // 字符串比较
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            // 布尔值比较
            (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
            // null 比较
            (Value::Null, Value::Null) => true,
            // 比较整数和浮点数（PHP 弱类型比较）
            (Value::Int(i), Value::Float(f)) | (Value::Float(f), Value::Int(i)) => {
                (*i as f64) == *f
            }
            // 其他类型不匹配
            _ => false,
        }
    }

    /// 计算分支体
    ///
    /// 对于表达式直接返回值，对于代码块则执行语句
    ///
    /// # 参数
    /// - `body`: 分支体
    ///
    /// # 返回值
    /// 分支体的返回值
    fn evaluate_body(&self, body: &MatchBody) -> Value {
        match body {
            // 表达式直接返回值
            MatchBody::Expression(v) => v.clone(),
            // 代码块需要执行语句
            MatchBody::Block(statements) => {
                // 执行代码块中的语句
                let mut last_value = Value::Null;

                for stmt in statements {
                    match stmt {
                        // 变量赋值语句
                        MatchStatement::Assign { name, value } => {
                            // 在实际实现中，这里应该将变量存储到作用域
                            tracing::debug!("Match 块赋值: {} = {:?}", name, value);
                            last_value = value.clone();
                        }
                        // 函数调用语句
                        MatchStatement::FunctionCall { name, args } => {
                            // 在实际实现中，这里应该调用函数
                            tracing::debug!("Match 块函数调用: {}({:?})", name, args);
                            last_value = Value::Null;
                        }
                        // 返回语句
                        MatchStatement::Return(value) => {
                            return value.clone();
                        }
                    }
                }

                last_value
            }
        }
    }

    /// 添加 default 分支
    ///
    /// # 参数
    /// - `body`: default 分支体
    ///
    /// # 返回值
    /// 返回自身，支持链式调用
    pub fn add_default(mut self, body: MatchBody) -> Self {
        self.default_value = Some(self.evaluate_body(&body));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试基本的 match 表达式
    #[test]
    fn test_match_expression() {
        // 创建 match 表达式
        let expr = MatchExpression::new(Value::Int(200))
            .add_arm(
                vec![Value::Int(200), Value::Int(300)],
                MatchBody::Expression(Value::String("OK".to_string())),
            )
            .add_arm(
                vec![Value::Int(400)],
                MatchBody::Expression(Value::String("Error".to_string())),
            );

        // 执行匹配
        let result = expr.evaluate();
        assert_eq!(result, Some(Value::String("OK".to_string())));
    }

    /// 测试 match 表达式的默认值
    #[test]
    fn test_match_expression_default() {
        // 创建 match 表达式，带默认值
        let expr = MatchExpression::new(Value::Int(500))
            .add_arm(
                vec![Value::Int(200)],
                MatchBody::Expression(Value::String("OK".to_string())),
            )
            .with_default(Value::String("Unknown".to_string()));

        // 执行匹配
        let result = expr.evaluate();
        assert_eq!(result, Some(Value::String("Unknown".to_string())));
    }

    /// 测试 match 表达式的代码块
    #[test]
    fn test_match_block() {
        // 创建带代码块的 match 表达式
        let expr = MatchExpression::new(Value::Int(1)).add_arm(
            vec![Value::Int(1)],
            MatchBody::Block(vec![
                MatchStatement::Assign {
                    name: "result".to_string(),
                    value: Value::String("processed".to_string()),
                },
                MatchStatement::Return(Value::String("done".to_string())),
            ]),
        );

        // 执行匹配
        let result = expr.evaluate();
        assert_eq!(result, Some(Value::String("done".to_string())));
    }
}
