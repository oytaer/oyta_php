//! Nullsafe 运算符模块
//!
//! PHP 8.0 引入 ?-> 运算符，用于安全地访问可能为 null 的对象

use std::collections::HashMap;

use crate::interpreter::value::Value;

/// Nullsafe 运算符处理器
///
/// PHP 8.0 引入 ?-> 运算符，用于安全地访问可能为 null 的对象
///
/// # 示例
/// ```php
/// // 传统写法
/// $country = $user !== null ? $user->address->country : null;
///
/// // Nullsafe 运算符
/// $country = $user?->address?->country;
/// ```
pub struct NullsafeOperator;

impl NullsafeOperator {
    /// 安全访问对象属性
    ///
    /// 如果对象为 null，直接返回 null
    /// 否则尝试获取属性值
    ///
    /// # 参数
    /// - `object`: 对象值
    /// - `property`: 属性名
    ///
    /// # 返回值
    /// 属性值，如果对象为 null 则返回 null
    pub fn get_property(object: &Value, property: &str) -> Value {
        match object {
            // 如果是 null，返回 null
            Value::Null => Value::Null,
            // 如果是对象，尝试获取属性
            Value::Object(obj) => obj.properties.get(property).cloned().unwrap_or(Value::Null),
            // 其他类型返回 null
            _ => Value::Null,
        }
    }

    /// 安全调用对象方法
    ///
    /// 如果对象为 null，直接返回 null
    /// 否则尝试调用方法
    ///
    /// # 参数
    /// - `object`: 对象值
    /// - `method`: 方法名
    /// - `args`: 参数列表
    /// - `context`: 执行上下文（用于方法调用）
    ///
    /// # 返回值
    /// 方法返回值，如果对象为 null 则返回 null
    pub fn call_method(
        object: &Value,
        method: &str,
        args: &[Value],
        context: Option<&MethodCallContext>,
    ) -> Value {
        match object {
            // 如果是 null，返回 null
            Value::Null => Value::Null,
            // 如果是对象，尝试调用方法
            Value::Object(obj) => {
                // 在实际实现中，这里需要通过解释器调用对象方法
                // 由于 ObjectInstance 没有 methods 字段，方法调用需要通过解释器进行
                tracing::debug!(
                    "调用对象方法: {}::{}({:?})",
                    obj.class_name,
                    method,
                    args
                );

                // 如果有上下文，使用上下文执行方法
                if let Some(ctx) = context {
                    tracing::debug!("方法调用上下文: {:?}", ctx.class_name);
                }

                // 返回占位值，实际实现需要解释器支持
                Value::Null
            }
            // 其他类型返回 null
            _ => Value::Null,
        }
    }

    /// 链式安全访问
    ///
    /// 按顺序访问属性链，如果链中任何值为 null 则返回 null
    ///
    /// # 参数
    /// - `object`: 起始对象
    /// - `chain`: 访问链（如 ["user", "address", "country"]）
    ///
    /// # 返回值
    /// 最终值，如果链中任何值为 null 则返回 null
    pub fn chain_access(object: &Value, chain: &[&str]) -> Value {
        let mut current = object.clone();

        // 遍历访问链
        for property in chain {
            current = Self::get_property(&current, property);
            // 如果当前值为 null，提前返回
            if matches!(current, Value::Null) {
                return Value::Null;
            }
        }

        current
    }

    /// 链式安全方法调用
    ///
    /// 按顺序执行调用链，如果链中任何值为 null 则返回 null
    ///
    /// # 参数
    /// - `object`: 起始对象
    /// - `calls`: 调用链（属性名或方法调用）
    ///
    /// # 返回值
    /// 最终值
    pub fn chain_call(object: &Value, calls: &[ChainCall]) -> Value {
        let mut current = object.clone();

        // 遍历调用链
        for call in calls {
            // 如果当前值为 null，提前返回
            if matches!(current, Value::Null) {
                return Value::Null;
            }

            match call {
                // 属性访问
                ChainCall::Property(name) => {
                    current = Self::get_property(&current, name);
                }
                // 方法调用
                ChainCall::Method { name, args } => {
                    current = Self::call_method(&current, name, args, None);
                }
                // 数组索引
                ChainCall::ArrayIndex(index) => {
                    current = Self::get_array_item(&current, index);
                }
            }
        }

        current
    }

    /// 获取数组元素
    ///
    /// 支持索引数组和关联数组
    ///
    /// # 参数
    /// - `array`: 数组值
    /// - `index`: 索引值
    ///
    /// # 返回值
    /// 数组元素值，如果不存在返回 null
    fn get_array_item(array: &Value, index: &Value) -> Value {
        match (array, index) {
            // 索引数组访问
            (Value::IndexedArray(arr), Value::Int(i)) => {
                if *i >= 0 && (*i as usize) < arr.len() {
                    arr[*i as usize].clone()
                } else {
                    Value::Null
                }
            }
            // 关联数组访问
            (Value::AssociativeArray(map), Value::String(key)) => {
                // 关联数组是 Vec<(String, Value)>，需要查找
                for (k, v) in map {
                    if k == key {
                        return v.clone();
                    }
                }
                Value::Null
            }
            // 其他情况返回 null
            _ => Value::Null,
        }
    }
}

/// 链式调用类型
#[derive(Debug, Clone)]
pub enum ChainCall {
    /// 属性访问
    Property(String),
    /// 方法调用
    Method {
        /// 方法名
        name: String,
        /// 参数
        args: Vec<Value>,
    },
    /// 数组索引
    ArrayIndex(Value),
}

/// 方法调用上下文
#[derive(Debug, Clone)]
pub struct MethodCallContext {
    /// 当前类名
    pub class_name: String,
    /// 当前对象
    pub this: Value,
    /// 作用域变量
    pub scope: HashMap<String, Value>,
}

impl MethodCallContext {
    /// 创建新的方法调用上下文
    ///
    /// # 参数
    /// - `class_name`: 类名
    /// - `this`: 当前对象引用
    ///
    /// # 返回值
    /// 新的 MethodCallContext 实例
    pub fn new(class_name: &str, this: Value) -> Self {
        Self {
            class_name: class_name.to_string(),
            this,
            scope: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::value::ObjectInstance;

    /// 测试 nullsafe 属性访问
    #[test]
    fn test_nullsafe_operator() {
        // 访问 null 对象的属性
        let result = NullsafeOperator::get_property(&Value::Null, "name");
        assert_eq!(result, Value::Null);
    }

    /// 测试 nullsafe 链式访问
    #[test]
    fn test_nullsafe_chain() {
        // 创建一个空对象
        let obj = Value::Object(ObjectInstance {
            class_name: "User".to_string(),
            properties: HashMap::new(),
        });

        // 链式访问不存在的属性
        let result = NullsafeOperator::chain_access(&obj, &["name", "first"]);
        assert_eq!(result, Value::Null);
    }
}
