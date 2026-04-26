//! 联合类型模块
//!
//! PHP 8.0 允许类型包含多种类型

use crate::interpreter::value::Value;

/// 联合类型
///
/// PHP 8.0 允许类型包含多种类型
///
/// # 示例
/// ```php
/// public function process(int|float $number): int|float {
///     return $number * 2;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct UnionType {
    /// 类型列表
    pub types: Vec<String>,
}

impl UnionType {
    /// 创建新的联合类型
    ///
    /// # 参数
    /// - `types`: 类型名称列表
    ///
    /// # 返回值
    /// 新的 UnionType 实例
    pub fn new(types: Vec<&str>) -> Self {
        Self {
            types: types.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// 从字符串解析联合类型
    ///
    /// # 参数
    /// - `type_str`: 类型字符串（如 "int|float|null"）
    ///
    /// # 返回值
    /// 解析后的 UnionType 实例
    pub fn parse(type_str: &str) -> Self {
        Self {
            types: type_str.split('|').map(|s| s.trim().to_string()).collect(),
        }
    }

    /// 检查值是否匹配类型
    ///
    /// # 参数
    /// - `value`: 要检查的值
    ///
    /// # 返回值
    /// 如果值匹配任一类型返回 true
    pub fn matches(&self, value: &Value) -> bool {
        for type_name in &self.types {
            if Self::value_matches_type(value, type_name) {
                return true;
            }
        }
        false
    }

    /// 检查单个类型匹配
    ///
    /// # 参数
    /// - `value`: 要检查的值
    /// - `type_name`: 类型名称
    ///
    /// # 返回值
    /// 如果值匹配类型返回 true
    fn value_matches_type(value: &Value, type_name: &str) -> bool {
        match type_name.to_lowercase().as_str() {
            // 整数类型
            "int" | "integer" => matches!(value, Value::Int(_)),
            // 浮点数类型
            "float" | "double" => matches!(value, Value::Float(_)),
            // 字符串类型
            "string" => matches!(value, Value::String(_)),
            // 布尔类型
            "bool" | "boolean" => matches!(value, Value::Bool(_)),
            // 数组类型
            "array" => matches!(value, Value::IndexedArray(_) | Value::AssociativeArray(_)),
            // 对象类型
            "object" => matches!(value, Value::Object(_)),
            // null 类型
            "null" => matches!(value, Value::Null),
            // 混合类型（匹配任何值）
            "mixed" => true,
            // 可调用类型
            "callable" => matches!(value, Value::Callable(_)),
            // 可迭代类型
            "iterable" => {
                matches!(value, Value::IndexedArray(_) | Value::AssociativeArray(_))
            }
            // 数值类型
            "numeric" => matches!(value, Value::Int(_) | Value::Float(_)),
            // 标量类型
            "scalar" => matches!(
                value,
                Value::Int(_) | Value::Float(_) | Value::String(_) | Value::Bool(_)
            ),
            // 其他类型（类名等）暂不验证
            _ => true,
        }
    }

    /// 获取类型字符串
    ///
    /// # 返回值
    /// 用 | 分隔的类型字符串
    pub fn to_type_string(&self) -> String {
        self.types.join("|")
    }

    /// 检查是否包含 null 类型
    ///
    /// # 返回值
    /// 如果包含 null 类型返回 true
    pub fn is_nullable(&self) -> bool {
        self.types.iter().any(|t| t.to_lowercase() == "null")
    }

    /// 获取非 null 类型
    ///
    /// # 返回值
    /// 不包含 null 的类型列表
    pub fn get_non_null_types(&self) -> Vec<&String> {
        self.types
            .iter()
            .filter(|t| t.to_lowercase() != "null")
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试联合类型匹配
    #[test]
    fn test_union_type() {
        let union = UnionType::new(vec!["int", "float"]);

        assert!(union.matches(&Value::Int(42)));
        assert!(union.matches(&Value::Float(3.14)));
        assert!(!union.matches(&Value::String("hello".to_string())));
    }

    /// 测试可空联合类型
    #[test]
    fn test_union_type_nullable() {
        let union = UnionType::parse("int|null");

        assert!(union.is_nullable());
        assert!(union.matches(&Value::Int(42)));
        assert!(union.matches(&Value::Null));
    }
}
