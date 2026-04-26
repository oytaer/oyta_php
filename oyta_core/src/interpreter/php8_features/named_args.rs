//! 命名参数模块
//!
//! PHP 8.0 允许使用命名参数调用函数
//! 命名参数可以按任意顺序传递，并且可以跳过可选参数

use std::collections::HashMap;

use crate::interpreter::value::Value;

/// 命名参数处理器
///
/// PHP 8.0 允许使用命名参数调用函数
///
/// # 示例
/// ```php
/// // 位置参数
/// htmlspecialchars($string, ENT_QUOTES | ENT_HTML5, 'UTF-8');
///
/// // 命名参数
/// htmlspecialchars(string: $string, encoding: 'UTF-8');
/// ```
pub struct NamedArguments {
    /// 位置参数
    pub positional: Vec<Value>,
    /// 命名参数
    pub named: HashMap<String, Value>,
}

impl NamedArguments {
    /// 创建新的命名参数集合
    ///
    /// # 返回值
    /// 空的 NamedArguments 实例
    pub fn new() -> Self {
        Self {
            positional: Vec::new(),
            named: HashMap::new(),
        }
    }

    /// 添加位置参数
    ///
    /// # 参数
    /// - `value`: 参数值
    ///
    /// # 返回值
    /// 返回自身，支持链式调用
    pub fn add_positional(mut self, value: Value) -> Self {
        self.positional.push(value);
        self
    }

    /// 添加命名参数
    ///
    /// # 参数
    /// - `name`: 参数名
    /// - `value`: 参数值
    ///
    /// # 返回值
    /// 返回自身，支持链式调用
    pub fn add_named(mut self, name: &str, value: Value) -> Self {
        self.named.insert(name.to_string(), value);
        self
    }

    /// 解析函数参数
    ///
    /// 根据函数的参数定义，将命名参数和位置参数解析为按顺序排列的值列表
    ///
    /// # 参数
    /// - `param_names`: 函数参数名称列表
    /// - `param_defaults`: 参数默认值
    ///
    /// # 返回值
    /// 按参数顺序排列的值列表
    pub fn resolve(
        &self,
        param_names: &[String],
        param_defaults: &HashMap<String, Value>,
    ) -> Vec<Value> {
        let mut result = Vec::new();

        // 遍历每个参数名
        for (i, param_name) in param_names.iter().enumerate() {
            // 首先检查命名参数
            if let Some(value) = self.named.get(param_name) {
                result.push(value.clone());
            }
            // 然后检查位置参数
            else if i < self.positional.len() {
                result.push(self.positional[i].clone());
            }
            // 最后使用默认值
            else if let Some(default) = param_defaults.get(param_name) {
                result.push(default.clone());
            }
            // 没有值，使用 Null
            else {
                result.push(Value::Null);
            }
        }

        result
    }

    /// 获取参数数量
    ///
    /// # 返回值
    /// 位置参数和命名参数的总数
    pub fn len(&self) -> usize {
        self.positional.len() + self.named.len()
    }

    /// 是否为空
    ///
    /// # 返回值
    /// 如果没有任何参数返回 true
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 合并另一个参数集合
    ///
    /// # 参数
    /// - `other`: 要合并的另一个参数集合
    pub fn merge(&mut self, other: NamedArguments) {
        // 合并位置参数
        self.positional.extend(other.positional);
        // 合并命名参数
        for (name, value) in other.named {
            self.named.insert(name, value);
        }
    }

    /// 转换为位置参数列表（忽略命名参数）
    ///
    /// # 返回值
    /// 位置参数列表
    pub fn to_positional(&self) -> Vec<Value> {
        self.positional.clone()
    }
}

impl Default for NamedArguments {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试命名参数的基本功能
    #[test]
    fn test_named_arguments() {
        // 创建命名参数集合
        let args = NamedArguments::new()
            .add_positional(Value::Int(1))
            .add_named("name", Value::String("test".to_string()));

        // 验证参数数量
        assert_eq!(args.len(), 2);
    }

    /// 测试命名参数的解析
    #[test]
    fn test_named_arguments_resolve() {
        // 创建命名参数集合
        let args = NamedArguments::new()
            .add_positional(Value::Int(1))
            .add_named("name", Value::String("test".to_string()));

        // 定义函数参数
        let param_names = vec!["id".to_string(), "name".to_string(), "value".to_string()];
        let defaults = HashMap::from([("value".to_string(), Value::Int(100))]);

        // 解析参数
        let resolved = args.resolve(&param_names, &defaults);

        // 验证解析结果
        assert_eq!(resolved[0], Value::Int(1));
        assert_eq!(resolved[1], Value::String("test".to_string()));
        assert_eq!(resolved[2], Value::Int(100));
    }
}
