//! 实体模型模块
//!
//! 实现实体模型功能
//! 对应 ThinkPHP 的实体模型（Entity Model）功能
//!
//! 主要功能：
//! - 实体属性定义
//! - 类型安全访问
//! - 属性验证
//! - 实体转换

use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::interpreter::value::Value;
use super::instance::ModelInstance;
use super::types::ModelConfig;

/// 实体属性 trait
///
/// 定义实体属性的行为
/// 添加 Debug 约束以支持调试输出
pub trait EntityProperty: Clone + std::fmt::Debug {
    /// 属性类型名称
    fn type_name() -> &'static str;

    /// 从 Value 转换
    fn from_value(value: &Value) -> Option<Self>;

    /// 转换为 Value
    fn to_value(&self) -> Value;
}

/// 实体属性实现 - 整数
impl EntityProperty for i64 {
    fn type_name() -> &'static str { "int" }
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Int(i) => Some(*i),
            Value::Float(f) => Some(*f as i64),
            Value::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    fn to_value(&self) -> Value { Value::Int(*self) }
}

/// 实体属性实现 - 浮点数
impl EntityProperty for f64 {
    fn type_name() -> &'static str { "float" }
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Int(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
            Value::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    fn to_value(&self) -> Value { Value::Float(*self) }
}

/// 实体属性实现 - 字符串
impl EntityProperty for String {
    fn type_name() -> &'static str { "string" }
    fn from_value(value: &Value) -> Option<Self> {
        Some(value.to_string_value())
    }
    fn to_value(&self) -> Value { Value::String(self.clone()) }
}

/// 实体属性实现 - 布尔值
impl EntityProperty for bool {
    fn type_name() -> &'static str { "bool" }
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Bool(b) => Some(*b),
            Value::Int(i) => Some(*i != 0),
            Value::String(s) => Some(s == "1" || s.to_lowercase() == "true"),
            _ => None,
        }
    }
    fn to_value(&self) -> Value { Value::Bool(*self) }
}

/// 实体属性定义
///
/// 定义实体的单个属性
pub struct PropertyDefinition<T: EntityProperty> {
    /// 属性名
    pub name: String,
    /// 默认值
    pub default: Option<T>,
    /// 是否必填
    pub required: bool,
    /// 是否只读
    pub readonly: bool,
    /// 验证规则（使用 Arc 包装以支持 Clone）
    pub validators: Vec<Arc<dyn Fn(&T) -> bool + Send + Sync>>,
    /// 类型标记
    _phantom: PhantomData<T>,
}

/// 手动实现 Debug trait，因为闭包不支持 Debug
/// 添加 T: std::fmt::Debug 约束以确保可以调试输出
impl<T: EntityProperty + std::fmt::Debug> fmt::Debug for PropertyDefinition<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 格式化输出属性定义信息
        f.debug_struct("PropertyDefinition")
            .field("name", &self.name)
            .field("default", &self.default)
            .field("required", &self.required)
            .field("readonly", &self.readonly)
            .field("validators", &format!("{} validators", self.validators.len()))
            .finish()
    }
}

/// 手动实现 Clone trait，使用 Arc 实现闭包的克隆
impl<T: EntityProperty> Clone for PropertyDefinition<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            default: self.default.clone(),
            required: self.required,
            readonly: self.readonly,
            validators: self.validators.clone(), // Arc 可以克隆
            _phantom: PhantomData,
        }
    }
}

impl<T: EntityProperty> PropertyDefinition<T> {
    /// 创建新的属性定义
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            default: None,
            required: false,
            readonly: false,
            validators: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// 设置默认值
    pub fn with_default(mut self, value: T) -> Self {
        self.default = Some(value);
        self
    }

    /// 设置为必填
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// 设置为只读
    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    /// 添加验证规则
    ///
    /// # 参数
    /// - `validator`: 验证器闭包
    ///
    /// # 返回值
    /// 自身引用（支持链式调用）
    pub fn with_validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        // 使用 Arc 包装验证器以支持克隆
        self.validators.push(Arc::new(validator));
        self
    }

    /// 验证值
    pub fn validate(&self, value: &T) -> bool {
        for validator in &self.validators {
            if !validator(value) {
                return false;
            }
        }
        true
    }
}

/// 实体模型基类
///
/// 提供实体模型的基本功能
#[derive(Debug, Clone)]
pub struct EntityModel {
    /// 内部模型实例
    pub inner: ModelInstance,
    /// 属性定义
    pub properties: HashMap<String, PropertyType>,
    /// 验证错误
    pub errors: HashMap<String, Vec<String>>,
}

/// 属性类型枚举
#[derive(Debug, Clone)]
pub enum PropertyType {
    /// 整数
    Int(PropertyDefinition<i64>),
    /// 浮点数
    Float(PropertyDefinition<f64>),
    /// 字符串
    String(PropertyDefinition<String>),
    /// 布尔值
    Bool(PropertyDefinition<bool>),
}

impl EntityModel {
    /// 创建新的实体模型
    pub fn new(config: ModelConfig) -> Self {
        Self {
            inner: ModelInstance::new(config),
            properties: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    /// 从模型实例创建实体
    pub fn from_instance(instance: ModelInstance) -> Self {
        Self {
            inner: instance,
            properties: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    /// 定义整数属性
    pub fn define_int(&mut self, def: PropertyDefinition<i64>) {
        self.properties.insert(def.name.clone(), PropertyType::Int(def));
    }

    /// 定义浮点数属性
    pub fn define_float(&mut self, def: PropertyDefinition<f64>) {
        self.properties.insert(def.name.clone(), PropertyType::Float(def));
    }

    /// 定义字符串属性
    pub fn define_string(&mut self, def: PropertyDefinition<String>) {
        self.properties.insert(def.name.clone(), PropertyType::String(def));
    }

    /// 定义布尔值属性
    pub fn define_bool(&mut self, def: PropertyDefinition<bool>) {
        self.properties.insert(def.name.clone(), PropertyType::Bool(def));
    }

    /// 获取整数属性
    pub fn get_int(&self, name: &str) -> Option<i64> {
        let value = self.inner.attributes.get(name)?;
        i64::from_value(value)
    }

    /// 获取浮点数属性
    pub fn get_float(&self, name: &str) -> Option<f64> {
        let value = self.inner.attributes.get(name)?;
        f64::from_value(value)
    }

    /// 获取字符串属性
    pub fn get_string(&self, name: &str) -> Option<String> {
        let value = self.inner.attributes.get(name)?;
        String::from_value(value)
    }

    /// 获取布尔值属性
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        let value = self.inner.attributes.get(name)?;
        bool::from_value(value)
    }

    /// 设置整数属性
    pub fn set_int(&mut self, name: &str, value: i64) -> bool {
        // 检查属性定义
        if let Some(PropertyType::Int(def)) = self.properties.get(name) {
            // 检查只读
            if def.readonly && self.inner.exists {
                return false;
            }

            // 验证
            if !def.validate(&value) {
                self.add_error(name, "Validation failed");
                return false;
            }
        }

        self.inner.attributes.insert(name.to_string(), Value::Int(value));
        true
    }

    /// 设置浮点数属性
    pub fn set_float(&mut self, name: &str, value: f64) -> bool {
        if let Some(PropertyType::Float(def)) = self.properties.get(name) {
            if def.readonly && self.inner.exists {
                return false;
            }

            if !def.validate(&value) {
                self.add_error(name, "Validation failed");
                return false;
            }
        }

        self.inner.attributes.insert(name.to_string(), Value::Float(value));
        true
    }

    /// 设置字符串属性
    pub fn set_string(&mut self, name: &str, value: String) -> bool {
        if let Some(PropertyType::String(def)) = self.properties.get(name) {
            if def.readonly && self.inner.exists {
                return false;
            }

            if !def.validate(&value) {
                self.add_error(name, "Validation failed");
                return false;
            }
        }

        self.inner.attributes.insert(name.to_string(), Value::String(value));
        true
    }

    /// 设置布尔值属性
    pub fn set_bool(&mut self, name: &str, value: bool) -> bool {
        if let Some(PropertyType::Bool(def)) = self.properties.get(name) {
            if def.readonly && self.inner.exists {
                return false;
            }

            if !def.validate(&value) {
                self.add_error(name, "Validation failed");
                return false;
            }
        }

        self.inner.attributes.insert(name.to_string(), Value::Bool(value));
        true
    }

    /// 验证所有属性
    pub fn validate(&mut self) -> bool {
        // 清空之前的错误
        self.errors.clear();
        // 标记验证是否通过
        let mut valid = true;
        // 收集验证错误（避免在迭代时修改 self）
        let mut validation_errors: Vec<(String, String)> = Vec::new();

        // 遍历所有属性定义
        for (name, prop_type) in &self.properties {
            // 获取属性值
            let value = self.inner.attributes.get(name);

            // 根据属性类型进行验证
            match prop_type {
                PropertyType::Int(def) => {
                    // 检查必填
                    if def.required && value.is_none() {
                        validation_errors.push((name.clone(), "Field is required".to_string()));
                        valid = false;
                        continue;
                    }

                    // 验证值
                    if let Some(v) = value {
                        if let Some(i) = i64::from_value(v) {
                            if !def.validate(&i) {
                                validation_errors.push((name.clone(), "Validation failed".to_string()));
                                valid = false;
                            }
                        }
                    }
                }
                PropertyType::Float(def) => {
                    // 检查必填
                    if def.required && value.is_none() {
                        validation_errors.push((name.clone(), "Field is required".to_string()));
                        valid = false;
                        continue;
                    }

                    // 验证值
                    if let Some(v) = value {
                        if let Some(f) = f64::from_value(v) {
                            if !def.validate(&f) {
                                validation_errors.push((name.clone(), "Validation failed".to_string()));
                                valid = false;
                            }
                        }
                    }
                }
                PropertyType::String(def) => {
                    // 检查必填
                    if def.required && value.is_none() {
                        validation_errors.push((name.clone(), "Field is required".to_string()));
                        valid = false;
                        continue;
                    }

                    // 验证值
                    if let Some(v) = value {
                        let s = String::from_value(v).unwrap_or_default();
                        if !def.validate(&s) {
                            validation_errors.push((name.clone(), "Validation failed".to_string()));
                            valid = false;
                        }
                    }
                }
                PropertyType::Bool(def) => {
                    // 检查必填
                    if def.required && value.is_none() {
                        validation_errors.push((name.clone(), "Field is required".to_string()));
                        valid = false;
                        continue;
                    }

                    // 验证值
                    if let Some(v) = value {
                        if let Some(b) = bool::from_value(v) {
                            if !def.validate(&b) {
                                validation_errors.push((name.clone(), "Validation failed".to_string()));
                                valid = false;
                            }
                        }
                    }
                }
            }
        }

        // 添加所有收集到的错误
        for (field, message) in validation_errors {
            self.add_error(&field, &message);
        }

        valid
    }

    /// 添加验证错误
    pub fn add_error(&mut self, field: &str, message: &str) {
        self.errors
            .entry(field.to_string())
            .or_insert_with(Vec::new)
            .push(message.to_string());
    }

    /// 获取所有错误
    pub fn get_errors(&self) -> &HashMap<String, Vec<String>> {
        &self.errors
    }

    /// 检查是否有错误
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// 转换为数组
    pub fn to_array(&self) -> Value {
        self.inner.to_value()
    }

    /// 转换为 JSON
    pub fn to_json(&self) -> String {
        match serde_json::to_string(&self.to_array()) {
            Ok(json) => json,
            Err(_) => "{}".to_string(),
        }
    }
}

/// 实体构建器
///
/// 用于构建实体模型
pub struct EntityBuilder {
    /// 模型配置
    config: ModelConfig,
    /// 属性定义
    properties: HashMap<String, PropertyType>,
}

impl EntityBuilder {
    /// 创建新的实体构建器
    pub fn new(table: &str) -> Self {
        Self {
            config: ModelConfig {
                table: table.to_string(),
                ..ModelConfig::default()
            },
            properties: HashMap::new(),
        }
    }

    /// 添加整数属性
    pub fn int(mut self, def: PropertyDefinition<i64>) -> Self {
        self.properties.insert(def.name.clone(), PropertyType::Int(def));
        self
    }

    /// 添加浮点数属性
    pub fn float(mut self, def: PropertyDefinition<f64>) -> Self {
        self.properties.insert(def.name.clone(), PropertyType::Float(def));
        self
    }

    /// 添加字符串属性
    pub fn string(mut self, def: PropertyDefinition<String>) -> Self {
        self.properties.insert(def.name.clone(), PropertyType::String(def));
        self
    }

    /// 添加布尔值属性
    pub fn bool(mut self, def: PropertyDefinition<bool>) -> Self {
        self.properties.insert(def.name.clone(), PropertyType::Bool(def));
        self
    }

    /// 构建实体模型
    pub fn build(self) -> EntityModel {
        EntityModel {
            inner: ModelInstance::new(self.config),
            properties: self.properties,
            errors: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_definition() {
        let def = PropertyDefinition::<i64>::new("age")
            .with_default(0)
            .required()
            .with_validator(|v| *v >= 0 && *v <= 150);

        assert_eq!(def.name, "age");
        assert!(def.required);
        assert!(def.validate(&25));
        assert!(!def.validate(&200));
    }

    #[test]
    fn test_entity_model() {
        let mut entity = EntityModel::new(ModelConfig {
            table: "users".to_string(),
            ..ModelConfig::default()
        });

        // 定义属性
        entity.define_int(PropertyDefinition::new("age").with_validator(|v| *v >= 0));
        entity.define_string(PropertyDefinition::new("name").required());

        // 设置值
        assert!(entity.set_int("age", 25));
        assert!(entity.set_string("name", "Test".to_string()));

        // 获取值
        assert_eq!(entity.get_int("age"), Some(25));
        assert_eq!(entity.get_string("name"), Some("Test".to_string()));
    }

    #[test]
    fn test_entity_validation() {
        let mut entity = EntityModel::new(ModelConfig {
            table: "users".to_string(),
            ..ModelConfig::default()
        });

        entity.define_string(PropertyDefinition::new("name").required());

        // 不设置 name，验证应该失败
        let valid = entity.validate();
        assert!(!valid);
        assert!(entity.has_errors());
    }

    #[test]
    fn test_entity_builder() {
        let entity = EntityBuilder::new("users")
            .int(PropertyDefinition::new("id").required())
            .string(PropertyDefinition::new("name").required())
            .build();

        assert!(entity.properties.contains_key("id"));
        assert!(entity.properties.contains_key("name"));
    }
}
