//! 模型辅助函数模块
//!
//! 包含模型操作相关的辅助函数

use crate::interpreter::value::Value;

use super::types::{FieldType, SoftDeleteDefault};

/// 生成软删除条件 SQL
///
/// 根据软删除默认值类型生成对应的 SQL 条件语句
///
/// # 参数
/// - `default`: 软删除默认值类型
/// - `field`: 软删除字段名
///
/// # 返回值
/// SQL 条件字符串
pub fn soft_delete_condition(default: &SoftDeleteDefault, field: &str) -> String {
    match default {
        // NULL 表示未删除
        SoftDeleteDefault::Null => format!("{} IS NULL", field),
        // 0 表示未删除
        SoftDeleteDefault::Zero => format!("{} = 0", field),
        // 空字符串表示未删除
        SoftDeleteDefault::Empty => format!("{} = ''", field),
    }
}

/// 获取当前日期时间字符串
///
/// 根据指定的格式返回当前时间
///
/// # 参数
/// - `format`: 时间格式（"datetime" 或 "timestamp"）
///
/// # 返回值
/// 格式化后的时间字符串
pub fn current_datetime(format: &str) -> String {
    let now = chrono::Local::now();
    match format {
        // Unix 时间戳格式
        "timestamp" => now.timestamp().to_string(),
        // 日期时间格式（默认）
        _ => now.format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

/// 将类名转换为表名
///
/// 将驼峰命名的类名转换为下划线命名的表名
/// 例如："User" → "user", "UserInfo" → "user_info"
///
/// # 参数
/// - `class_name`: 类名（可能包含命名空间）
///
/// # 返回值
/// 表名字符串
pub fn class_to_table(class_name: &str) -> String {
    // 提取短类名（去除命名空间）
    let short_name = if let Some(pos) = class_name.rfind('\\') {
        &class_name[pos + 1..]
    } else {
        class_name
    };

    // 转换为下划线命名
    let mut result = String::new();
    for (i, c) in short_name.chars().enumerate() {
        if c.is_uppercase() {
            // 大写字母前添加下划线（非首字母）
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// 应用字段类型转换
///
/// 根据字段类型定义将值转换为对应的类型
///
/// # 参数
/// - `value`: 原始值
/// - `field_type`: 字段类型定义
///
/// # 返回值
/// 转换后的值
pub fn apply_field_type(value: &Value, field_type: &FieldType) -> Value {
    match field_type {
        // 整数类型：转换为 i64
        FieldType::Int => Value::Int(value.to_int()),
        // 浮点类型：保持或转换为 f64
        FieldType::Float => match value {
            Value::Float(f) => Value::Float(*f),
            _ => Value::Float(value.to_float()),
        },
        // 字符串类型：转换为字符串
        FieldType::String => Value::String(value.to_string_value()),
        // 布尔类型：转换为布尔值
        FieldType::Bool => Value::Bool(value.is_truthy()),
        // JSON 类型：解析 JSON 字符串
        FieldType::Json => {
            if let Value::String(s) = value {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(s) {
                    return json_to_value(&val);
                }
            }
            value.clone()
        }
        // 日期时间类型：保持原值
        FieldType::Datetime => value.clone(),
        // 数组类型：保持原值
        FieldType::Array => value.clone(),
        // 枚举类型：验证值是否在枚举列表中
        FieldType::Enum(variants) => {
            if let Value::String(s) = value {
                if variants.contains(&s.to_string()) {
                    Value::String(s.clone())
                } else {
                    Value::Null
                }
            } else {
                value.clone()
            }
        }
    }
}

/// 将 JSON 值转换为 Value
///
/// 将 serde_json::Value 转换为解释器的 Value 类型
///
/// # 参数
/// - `json`: JSON 值引用
///
/// # 返回值
/// 转换后的 Value
pub fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        // JSON null 转换为 Value::Null
        serde_json::Value::Null => Value::Null,
        // JSON 布尔值转换为 Value::Bool
        serde_json::Value::Bool(b) => Value::Bool(*b),
        // JSON 数字转换为整数或浮点数
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::String(n.to_string())
            }
        }
        // JSON 字符串转换为 Value::String
        serde_json::Value::String(s) => Value::String(s.clone()),
        // JSON 数组转换为 Value::IndexedArray
        serde_json::Value::Array(arr) => {
            let values: Vec<Value> = arr.iter().map(json_to_value).collect();
            Value::IndexedArray(values)
        }
        // JSON 对象转换为 Value::AssociativeArray
        serde_json::Value::Object(obj) => {
            let pairs: Vec<(String, Value)> = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_value(v)))
                .collect();
            Value::AssociativeArray(pairs)
        }
    }
}

/// 将 Value 转换为 JSON 字符串
///
/// 将解释器的 Value 类型序列化为 JSON 字符串
///
/// # 参数
/// - `value`: Value 引用
///
/// # 返回值
/// JSON 字符串包装在 Value::String 中
pub fn value_to_json_string(value: &Value) -> Value {
    // 先转换为 serde_json::Value
    let json_val = value.to_json_value();
    // 序列化为字符串
    match serde_json::to_string(&json_val) {
        Ok(s) => Value::String(s),
        Err(_) => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试类名转表名
    #[test]
    fn test_class_to_table() {
        assert_eq!(class_to_table("User"), "user");
        assert_eq!(class_to_table("UserInfo"), "user_info");
        assert_eq!(class_to_table("app\\model\\User"), "user");
        assert_eq!(class_to_table("app\\model\\UserInfo"), "user_info");
    }

    /// 测试软删除条件生成
    #[test]
    fn test_soft_delete_condition() {
        assert_eq!(
            soft_delete_condition(&SoftDeleteDefault::Null, "delete_time"),
            "delete_time IS NULL"
        );
        assert_eq!(
            soft_delete_condition(&SoftDeleteDefault::Zero, "delete_time"),
            "delete_time = 0"
        );
        assert_eq!(
            soft_delete_condition(&SoftDeleteDefault::Empty, "delete_time"),
            "delete_time = ''"
        );
    }

    /// 测试当前时间获取
    #[test]
    fn test_current_datetime() {
        let datetime = current_datetime("datetime");
        assert!(datetime.contains('-'));
        assert!(datetime.contains(':'));

        let timestamp = current_datetime("timestamp");
        assert!(timestamp.parse::<i64>().is_ok());
    }
}
