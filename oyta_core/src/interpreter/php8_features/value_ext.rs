//! Value 辅助方法模块
//!
//! 为 Value 类型提供便捷的类型转换和访问方法

use crate::interpreter::value::Value;

/// Value 类型的辅助方法
impl Value {
    /// 获取字符串值
    ///
    /// 如果 Value 是字符串类型，返回字符串切片引用
    /// 否则返回 None
    ///
    /// # 返回值
    /// - `Some(&str)`: 字符串值
    /// - `None`: 不是字符串类型
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// 获取整数值
    ///
    /// 如果 Value 是整数类型，返回 i64 值
    /// 否则返回 None
    ///
    /// # 返回值
    /// - `Some(i64)`: 整数值
    /// - `None`: 不是整数类型
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// 获取浮点数值
    ///
    /// 如果 Value 是浮点数类型，返回 f64 值
    /// 如果是整数类型，转换为 f64 返回
    /// 否则返回 None
    ///
    /// # 返回值
    /// - `Some(f64)`: 浮点数值
    /// - `None`: 不是数值类型
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// 获取布尔值
    ///
    /// 如果 Value 是布尔类型，返回布尔值
    /// 否则返回 None
    ///
    /// # 返回值
    /// - `Some(bool)`: 布尔值
    /// - `None`: 不是布尔类型
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
}
