//! 修改器扩展模块
//!
//! 实现模型的修改器扩展功能
//! 对应 ThinkPHP 的修改器（Setter/Mutator）功能
//!
//! 主要功能：
//! - 修改器定义和管理
//! - 批量修改
//! - 显式调用修改器
//! - 数据设置方法

use std::collections::HashMap;
use std::fmt;

use crate::interpreter::value::Value;
use super::instance::ModelInstance;

/// 修改器回调类型
///
/// 定义修改器处理函数的签名
/// 参数：字段值、完整数据
/// 返回：处理后的值
pub type SetterCallback = Box<dyn Fn(&Value, &HashMap<String, Value>) -> Value + Send + Sync>;

/// 修改器定义
///
/// 存储单个修改器的配置
pub struct SetterDefinition {
    /// 字段名
    pub field: String,
    /// 回调函数
    pub callback: SetterCallback,
}

/// 手动实现 Debug trait，因为闭包不支持 Debug
impl fmt::Debug for SetterDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 格式化输出修改器信息
        f.debug_struct("SetterDefinition")
            .field("field", &self.field)
            .field("callback", &"<closure>")
            .finish()
    }
}

impl SetterDefinition {
    /// 创建新的修改器定义
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `callback`: 回调函数
    ///
    /// # 返回值
    /// 新的修改器定义
    pub fn new<F>(field: &str, callback: F) -> Self
    where
        F: Fn(&Value, &HashMap<String, Value>) -> Value + Send + Sync + 'static,
    {
        Self {
            field: field.to_string(),
            callback: Box::new(callback),
        }
    }

    /// 执行修改器
    ///
    /// # 参数
    /// - `value`: 字段值
    /// - `data`: 完整数据
    ///
    /// # 返回值
    /// 处理后的值
    pub fn execute(&self, value: &Value, data: &HashMap<String, Value>) -> Value {
        (self.callback)(value, data)
    }
}

/// 修改器管理器
///
/// 管理模型的所有修改器
pub struct SetterManager {
    /// 修改器定义映射
    setters: HashMap<String, SetterDefinition>,
}

/// 手动实现 Debug trait，因为 SetterDefinition 包含闭包
impl fmt::Debug for SetterManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 格式化输出修改器管理器信息
        f.debug_struct("SetterManager")
            .field("setters", &format!("{} setters", self.setters.len()))
            .finish()
    }
}

/// 实现 Default trait
impl Default for SetterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SetterManager {
    /// 创建新的修改器管理器
    pub fn new() -> Self {
        Self {
            setters: HashMap::new(),
        }
    }

    /// 注册修改器
    ///
    /// # 参数
    /// - `definition`: 修改器定义
    pub fn register(&mut self, definition: SetterDefinition) {
        self.setters.insert(definition.field.clone(), definition);
    }

    /// 注册修改器（便捷方法）
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `callback`: 回调函数
    pub fn add<F>(&mut self, field: &str, callback: F)
    where
        F: Fn(&Value, &HashMap<String, Value>) -> Value + Send + Sync + 'static,
    {
        let definition = SetterDefinition::new(field, callback);
        self.register(definition);
    }

    /// 应用修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `value`: 字段值
    /// - `data`: 完整数据
    ///
    /// # 返回值
    /// 处理后的值
    pub fn apply(&self, field: &str, value: &Value, data: &HashMap<String, Value>) -> Value {
        if let Some(definition) = self.setters.get(field) {
            definition.execute(value, data)
        } else {
            value.clone()
        }
    }

    /// 检查是否存在修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 如果存在返回 true
    pub fn has(&self, field: &str) -> bool {
        self.setters.contains_key(field)
    }

    /// 移除修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    pub fn remove(&mut self, field: &str) {
        self.setters.remove(field);
    }

    /// 清空所有修改器
    pub fn clear(&mut self) {
        self.setters.clear();
    }

    /// 获取所有修改器字段名
    ///
    /// # 返回值
    /// 字段名列表
    pub fn get_fields(&self) -> Vec<&str> {
        self.setters.keys().map(|s| s.as_str()).collect()
    }
}

/// 修改器扩展 trait
///
/// 为 ModelInstance 提供修改器扩展方法
pub trait SetterExt {
    /// 显式调用修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `value`: 字段值
    ///
    /// # 返回值
    /// 处理后的值
    ///
    /// # 示例
    /// ```php
    /// $user->setAttr('name', 'THINKPHP');
    /// ```
    fn set_attr_explicit(&mut self, field: &str, value: Value);

    /// 批量设置属性（触发修改器）
    ///
    /// # 参数
    /// - `data`: 数据映射
    ///
    /// # 示例
    /// ```php
    /// $user->setAttrs(['name' => 'THINKPHP', 'email' => 'test@test.com']);
    /// ```
    fn set_attrs(&mut self, data: &HashMap<String, Value>);

    /// 批量设置数据
    ///
    /// # 参数
    /// - `data`: 数据映射
    /// - `trigger_setter`: 是否触发修改器
    ///
    /// # 示例
    /// ```php
    /// $user->data($data, true);
    /// ```
    fn data(&mut self, data: &HashMap<String, Value>, trigger_setter: bool);

    /// 追加数据
    ///
    /// # 参数
    /// - `data`: 数据映射
    /// - `trigger_setter`: 是否触发修改器
    ///
    /// # 示例
    /// ```php
    /// $user->appendData($data, true);
    /// ```
    fn append_data(&mut self, data: &HashMap<String, Value>, trigger_setter: bool);

    /// 设置其他字段值（在修改器中使用）
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `value`: 字段值
    ///
    /// # 示例
    /// ```php
    /// public function setTestFieldAttr($value, $data) {
    ///     $this->set('other_field', $data['some_field']);
    /// }
    /// ```
    fn set_field(&mut self, field: &str, value: Value);
}

/// 为 ModelInstance 实现修改器扩展
impl SetterExt for ModelInstance {
    /// 显式调用修改器
    fn set_attr_explicit(&mut self, field: &str, value: Value) {
        self.set_attr(field, value);
    }

    /// 批量设置属性
    fn set_attrs(&mut self, data: &HashMap<String, Value>) {
        for (key, value) in data {
            self.set_attr(key, value.clone());
        }
    }

    /// 批量设置数据
    fn data(&mut self, data: &HashMap<String, Value>, trigger_setter: bool) {
        for (key, value) in data {
            if trigger_setter {
                self.set_attr(key, value.clone());
            } else {
                self.attributes.insert(key.clone(), value.clone());
                if !self.dirty.contains(key) {
                    self.dirty.push(key.clone());
                }
            }
        }
    }

    /// 追加数据
    fn append_data(&mut self, data: &HashMap<String, Value>, trigger_setter: bool) {
        self.data(data, trigger_setter);
    }

    /// 设置其他字段值
    fn set_field(&mut self, field: &str, value: Value) {
        self.attributes.insert(field.to_string(), value);
        if !self.dirty.contains(&field.to_string()) {
            self.dirty.push(field.to_string());
        }
    }
}

/// 预定义修改器
///
/// 提供常用的修改器模板
pub struct PredefinedSetters;

impl PredefinedSetters {
    /// 小写转换修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 修改器定义
    ///
    /// # 示例
    /// ```php
    /// public function setNameAttr($value) {
    ///     return strtolower($value);
    /// }
    /// ```
    pub fn lowercase(field: &str) -> SetterDefinition {
        SetterDefinition::new(field, |value, _data| {
            Value::String(value.to_string_value().to_lowercase())
        })
    }

    /// 大写转换修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 修改器定义
    pub fn uppercase(field: &str) -> SetterDefinition {
        SetterDefinition::new(field, |value, _data| {
            Value::String(value.to_string_value().to_uppercase())
        })
    }

    /// 去除空格修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 修改器定义
    pub fn trim(field: &str) -> SetterDefinition {
        SetterDefinition::new(field, |value, _data| {
            Value::String(value.to_string_value().trim().to_string())
        })
    }

    /// 密码哈希修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 修改器定义
    ///
    /// # 示例
    /// ```php
    /// public function setPasswordAttr($value) {
    ///     return password_hash($value, PASSWORD_DEFAULT);
    /// }
    /// ```
    pub fn password_hash(field: &str) -> SetterDefinition {
        SetterDefinition::new(field, |value, _data| {
            // 简化实现：实际应使用真正的密码哈希
            let password = value.to_string_value();
            // 使用简单的哈希模拟
            let hash = format!("hashed_{}", password);
            Value::String(hash)
        })
    }

    /// JSON 编码修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 修改器定义
    pub fn json_encode(field: &str) -> SetterDefinition {
        SetterDefinition::new(field, |value, _data| {
            match serde_json::to_string(value) {
                Ok(json) => Value::String(json),
                Err(_) => value.clone(),
            }
        })
    }

    /// 序列化修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 修改器定义
    pub fn serialize(field: &str) -> SetterDefinition {
        SetterDefinition::new(field, |value, _data| {
            // 简化实现：使用 JSON 序列化
            match serde_json::to_string(value) {
                Ok(json) => Value::String(format!("serialized:{}", json)),
                Err(_) => value.clone(),
            }
        })
    }

    /// 整数转换修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 修改器定义
    pub fn to_int(field: &str) -> SetterDefinition {
        SetterDefinition::new(field, |value, _data| {
            match value {
                Value::Int(i) => Value::Int(*i),
                Value::Float(f) => Value::Int(*f as i64),
                Value::String(s) => Value::Int(s.parse().unwrap_or(0)),
                Value::Bool(b) => Value::Int(if *b { 1 } else { 0 }),
                _ => Value::Int(0),
            }
        })
    }

    /// 浮点数转换修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 修改器定义
    pub fn to_float(field: &str) -> SetterDefinition {
        SetterDefinition::new(field, |value, _data| {
            match value {
                Value::Int(i) => Value::Float(*i as f64),
                Value::Float(f) => Value::Float(*f),
                Value::String(s) => Value::Float(s.parse().unwrap_or(0.0)),
                _ => Value::Float(0.0),
            }
        })
    }

    /// 布尔值转换修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 修改器定义
    pub fn to_bool(field: &str) -> SetterDefinition {
        SetterDefinition::new(field, |value, _data| {
            match value {
                Value::Bool(b) => Value::Bool(*b),
                Value::Int(i) => Value::Bool(*i != 0),
                Value::String(s) => Value::Bool(s == "1" || s.to_lowercase() == "true"),
                _ => Value::Bool(false),
            }
        })
    }

    /// 时间戳转换修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 修改器定义
    pub fn to_timestamp(field: &str) -> SetterDefinition {
        SetterDefinition::new(field, |value, _data| {
            match value {
                Value::String(s) => {
                    // 尝试解析时间字符串
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                        return Value::Int(dt.timestamp());
                    }
                    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                        return Value::Int(dt.and_utc().timestamp());
                    }
                    // 尝试直接解析为时间戳
                    if let Ok(ts) = s.parse::<i64>() {
                        return Value::Int(ts);
                    }
                    Value::Int(0)
                }
                Value::Int(i) => Value::Int(*i),
                _ => Value::Int(0),
            }
        })
    }

    /// 默认值修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `default`: 默认值
    ///
    /// # 返回值
    /// 修改器定义
    pub fn default_value(field: &str, default: Value) -> SetterDefinition {
        SetterDefinition::new(field, move |value, _data| {
            if matches!(value, Value::Null) || value.to_string_value().is_empty() {
                default.clone()
            } else {
                value.clone()
            }
        })
    }

    /// 限制长度修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `max_length`: 最大长度
    ///
    /// # 返回值
    /// 修改器定义
    pub fn limit_length(field: &str, max_length: usize) -> SetterDefinition {
        SetterDefinition::new(field, move |value, _data| {
            let s = value.to_string_value();
            if s.len() > max_length {
                Value::String(s[..max_length].to_string())
            } else {
                Value::String(s)
            }
        })
    }

    /// 正则替换修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `pattern`: 正则模式
    /// - `replacement`: 替换字符串
    ///
    /// # 返回值
    /// 修改器定义
    pub fn regex_replace(field: &str, pattern: &str, replacement: &str) -> SetterDefinition {
        let pattern_owned = pattern.to_string();
        let replacement_owned = replacement.to_string();

        SetterDefinition::new(field, move |value, _data| {
            let s = value.to_string_value();
            if let Ok(re) = regex::Regex::new(&pattern_owned) {
                Value::String(re.replace_all(&s, replacement_owned.as_str()).to_string())
            } else {
                Value::String(s)
            }
        })
    }

    /// 枚举值验证修改器
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `allowed_values`: 允许的值列表
    ///
    /// # 返回值
    /// 修改器定义
    pub fn enum_validate(field: &str, allowed_values: Vec<Value>) -> SetterDefinition {
        SetterDefinition::new(field, move |value, _data| {
            if allowed_values.contains(value) {
                value.clone()
            } else {
                // 返回第一个允许的值作为默认值
                allowed_values.first().cloned().unwrap_or(Value::Null)
            }
        })
    }
}

/// 数据设置器
///
/// 用于批量设置模型数据
pub struct DataSetter {
    /// 数据映射
    data: HashMap<String, Value>,
    /// 是否触发修改器
    trigger_setters: bool,
}

impl DataSetter {
    /// 创建新的数据设置器
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            trigger_setters: true,
        }
    }

    /// 设置是否触发修改器
    ///
    /// # 参数
    /// - `trigger`: 是否触发
    pub fn trigger_setters(mut self, trigger: bool) -> Self {
        self.trigger_setters = trigger;
        self
    }

    /// 添加数据
    ///
    /// # 参数
    /// - `key`: 字段名
    /// - `value`: 字段值
    pub fn set(mut self, key: &str, value: Value) -> Self {
        self.data.insert(key.to_string(), value);
        self
    }

    /// 批量添加数据
    ///
    /// # 参数
    /// - `data`: 数据映射
    pub fn set_all(mut self, data: HashMap<String, Value>) -> Self {
        self.data.extend(data);
        self
    }

    /// 应用到模型实例
    ///
    /// # 参数
    /// - `model`: 模型实例
    pub fn apply(&self, model: &mut ModelInstance) {
        model.data(&self.data, self.trigger_setters);
    }

    /// 获取数据
    ///
    /// # 返回值
    /// 数据映射引用
    pub fn get_data(&self) -> &HashMap<String, Value> {
        &self.data
    }

    /// 构建数据映射
    ///
    /// # 返回值
    /// 数据映射
    pub fn build(self) -> HashMap<String, Value> {
        self.data
    }
}

impl Default for DataSetter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ModelConfig;

    fn create_test_model() -> ModelInstance {
        let config = ModelConfig {
            table: "users".to_string(),
            ..ModelConfig::default()
        };
        let mut model = ModelInstance::new(config);
        model.attributes.insert("id".to_string(), Value::Int(1));
        model.original = model.attributes.clone();
        model.exists = true;
        model
    }

    #[test]
    fn test_setter_manager() {
        let mut manager = SetterManager::new();

        // 添加修改器
        manager.add("name", |value, _data| {
            Value::String(value.to_string_value().to_lowercase())
        });

        // 测试修改器
        let data = HashMap::new();
        let result = manager.apply("name", &Value::String("TEST".to_string()), &data);
        assert_eq!(result, Value::String("test".to_string()));
    }

    #[test]
    fn test_set_attrs() {
        let mut model = create_test_model();

        // 批量设置属性
        let mut data = HashMap::new();
        data.insert("name".to_string(), Value::String("Test User".to_string()));
        data.insert("email".to_string(), Value::String("test@example.com".to_string()));

        model.set_attrs(&data);

        // 验证设置结果
        assert_eq!(model.get_attr("name"), Value::String("Test User".to_string()));
        assert_eq!(model.get_attr("email"), Value::String("test@example.com".to_string()));
    }

    #[test]
    fn test_data_method() {
        let mut model = create_test_model();

        // 批量设置数据（不触发修改器）
        let mut data = HashMap::new();
        data.insert("field1".to_string(), Value::String("value1".to_string()));
        data.insert("field2".to_string(), Value::Int(100));

        model.data(&data, false);

        // 验证设置结果
        assert!(model.attributes.contains_key("field1"));
        assert!(model.attributes.contains_key("field2"));
    }

    #[test]
    fn test_predefined_setters() {
        // 测试小写修改器
        let lowercase_setter = PredefinedSetters::lowercase("name");
        let data = HashMap::new();
        let result = lowercase_setter.execute(&Value::String("TEST".to_string()), &data);
        assert_eq!(result, Value::String("test".to_string()));

        // 测试整数转换修改器
        let int_setter = PredefinedSetters::to_int("count");
        let result = int_setter.execute(&Value::String("123".to_string()), &data);
        assert_eq!(result, Value::Int(123));
    }

    #[test]
    fn test_data_setter() {
        let setter = DataSetter::new()
            .trigger_setters(false)
            .set("name", Value::String("test".to_string()))
            .set("status", Value::Int(1));

        let data = setter.build();

        assert_eq!(data.len(), 2);
        assert_eq!(data.get("name"), Some(&Value::String("test".to_string())));
        assert_eq!(data.get("status"), Some(&Value::Int(1)));
    }
}
