//! 获取器扩展模块
//!
//! 实现模型的获取器扩展功能
//! 对应 ThinkPHP 的获取器（Getter/Accessor）功能
//!
//! 主要功能：
//! - 动态获取器
//! - 获取原始数据
//! - 显式调用获取器
//! - 查询结果处理

use std::collections::HashMap;

use crate::interpreter::value::Value;
use super::instance::ModelInstance;

/// 获取器回调类型
///
/// 定义获取器处理函数的签名
/// 参数：字段值、完整数据
/// 返回：处理后的值
pub type GetterCallback = Box<dyn Fn(&Value, &HashMap<String, Value>) -> Value + Send + Sync>;

/// 获取器定义
///
/// 存储单个获取器的配置
pub struct GetterDefinition {
    /// 字段名
    pub field: String,
    /// 回调函数
    pub callback: GetterCallback,
}

// 手动实现 Debug trait，因为闭包不实现 Debug
impl std::fmt::Debug for GetterDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 只显示字段名，不显示回调函数
        f.debug_struct("GetterDefinition")
            .field("field", &self.field)
            .finish()
    }
}

impl GetterDefinition {
    /// 创建新的获取器定义
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `callback`: 回调函数
    ///
    /// # 返回值
    /// 新的获取器定义
    pub fn new<F>(field: &str, callback: F) -> Self
    where
        F: Fn(&Value, &HashMap<String, Value>) -> Value + Send + Sync + 'static,
    {
        Self {
            field: field.to_string(),
            callback: Box::new(callback),
        }
    }

    /// 执行获取器
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

/// 获取器管理器
///
/// 管理模型的所有获取器
pub struct GetterManager {
    /// 获取器定义映射
    getters: HashMap<String, GetterDefinition>,
    /// 动态获取器映射（临时）
    dynamic_getters: HashMap<String, GetterCallback>,
}

// 手动实现 Debug trait，因为闭包不实现 Debug
impl std::fmt::Debug for GetterManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 只显示获取器字段名列表
        f.debug_struct("GetterManager")
            .field("getters", &self.getters.keys().collect::<Vec<_>>())
            .field("dynamic_getters", &self.dynamic_getters.keys().collect::<Vec<_>>())
            .finish()
    }
}

// 实现 Default trait
impl Default for GetterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GetterManager {
    /// 创建新的获取器管理器
    pub fn new() -> Self {
        Self {
            getters: HashMap::new(),
            dynamic_getters: HashMap::new(),
        }
    }

    /// 注册获取器
    ///
    /// # 参数
    /// - `definition`: 获取器定义
    pub fn register(&mut self, definition: GetterDefinition) {
        self.getters.insert(definition.field.clone(), definition);
    }

    /// 注册获取器（便捷方法）
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `callback`: 回调函数
    pub fn add<F>(&mut self, field: &str, callback: F)
    where
        F: Fn(&Value, &HashMap<String, Value>) -> Value + Send + Sync + 'static,
    {
        let definition = GetterDefinition::new(field, callback);
        self.register(definition);
    }

    /// 注册动态获取器
    ///
    /// 动态获取器是临时的，仅在当前查询中生效
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `callback`: 回调函数
    ///
    /// # 示例
    /// ```php
    /// User::withAttr('name', function($value, $data) {
    ///     return strtolower($value);
    /// })->select();
    /// ```
    pub fn add_dynamic<F>(&mut self, field: &str, callback: F)
    where
        F: Fn(&Value, &HashMap<String, Value>) -> Value + Send + Sync + 'static,
    {
        self.dynamic_getters.insert(field.to_string(), Box::new(callback));
    }

    /// 应用获取器
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `value`: 字段值
    /// - `data`: 完整数据
    ///
    /// # 返回值
    /// 处理后的值
    pub fn apply(&self, field: &str, value: &Value, data: &HashMap<String, Value>) -> Value {
        // 优先使用动态获取器
        if let Some(callback) = self.dynamic_getters.get(field) {
            return callback(value, data);
        }

        // 然后使用静态获取器
        if let Some(definition) = self.getters.get(field) {
            return definition.execute(value, data);
        }

        // 没有获取器，返回原值
        value.clone()
    }

    /// 检查是否存在获取器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 如果存在返回 true
    pub fn has(&self, field: &str) -> bool {
        self.getters.contains_key(field) || self.dynamic_getters.contains_key(field)
    }

    /// 移除获取器
    ///
    /// # 参数
    /// - `field`: 字段名
    pub fn remove(&mut self, field: &str) {
        self.getters.remove(field);
        self.dynamic_getters.remove(field);
    }

    /// 清空动态获取器
    pub fn clear_dynamic(&mut self) {
        self.dynamic_getters.clear();
    }

    /// 清空所有获取器
    pub fn clear(&mut self) {
        self.getters.clear();
        self.dynamic_getters.clear();
    }

    /// 获取所有获取器字段名
    ///
    /// # 返回值
    /// 字段名列表
    pub fn get_fields(&self) -> Vec<&str> {
        let mut fields: Vec<&str> = self.getters.keys().map(|s| s.as_str()).collect();
        for field in self.dynamic_getters.keys() {
            if !fields.contains(&field.as_str()) {
                fields.push(field);
            }
        }
        fields
    }
}

/// 获取器扩展 trait
///
/// 为 ModelInstance 提供获取器扩展方法
pub trait GetterExt {
    /// 获取原始数据（不经过获取器）
    ///
    /// # 参数
    /// - `field`: 字段名（可选，不传则返回全部原始数据）
    ///
    /// # 返回值
    /// 原始数据值
    ///
    /// # 示例
    /// ```php
    /// $user = User::find(1);
    /// // 通过获取器获取字段
    /// echo $user->status;
    /// // 获取原始字段数据
    /// echo $user->getData('status');
    /// // 获取全部原始数据
    /// dump($user->getData());
    /// ```
    fn get_data(&self, field: Option<&str>) -> Value;

    /// 显式调用获取器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 经过获取器处理的值
    ///
    /// # 示例
    /// ```php
    /// $user = User::find(1);
    /// echo $user->getAttr('status');
    /// ```
    fn get_attr_explicit(&self, field: &str) -> Value;

    /// 动态添加获取器
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `callback`: 回调函数
    fn with_attr<F>(&mut self, field: &str, callback: F)
    where
        F: Fn(&Value, &HashMap<String, Value>) -> Value + Send + Sync + 'static;
}

/// 为 ModelInstance 实现获取器扩展
impl GetterExt for ModelInstance {
    /// 获取原始数据
    fn get_data(&self, field: Option<&str>) -> Value {
        match field {
            Some(f) => {
                // 返回指定字段的原始值
                self.original.get(f).cloned().unwrap_or(Value::Null)
            }
            None => {
                // 返回全部原始数据
                Value::AssociativeArray(self.original.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            }
        }
    }

    /// 显式调用获取器
    fn get_attr_explicit(&self, field: &str) -> Value {
        self.get_attr(field)
    }

    /// 动态添加获取器
    fn with_attr<F>(&mut self, field: &str, _callback: F)
    where
        F: Fn(&Value, &HashMap<String, Value>) -> Value + Send + Sync + 'static,
    {
        // 动态获取器需要在查询时通过 withAttr 方法设置
        // 这里只是标记字段需要动态处理
        self.config.append_fields.push(field.to_string());
    }
}

/// 预定义获取器
///
/// 提供常用的获取器模板
pub struct PredefinedGetters;

impl PredefinedGetters {
    /// 状态文本获取器
    ///
    /// # 参数
    /// - `status_map`: 状态映射（值 -> 文本）
    ///
    /// # 返回值
    /// 获取器定义
    ///
    /// # 示例
    /// ```php
    /// public function getStatusAttr($value) {
    ///     $status = [-1=>'删除', 0=>'禁用', 1=>'正常', 2=>'待审核'];
    ///     return $status[$value];
    /// }
    /// ```
    pub fn status_text(status_map: HashMap<i64, String>) -> GetterDefinition {
        GetterDefinition::new("status_text", move |value, data| {
            // 从完整数据中获取 status 字段
            if let Some(status_val) = data.get("status") {
                if let Value::Int(status) = status_val {
                    if let Some(text) = status_map.get(status) {
                        return Value::String(text.clone());
                    }
                }
            }
            value.clone()
        })
    }

    /// 时间格式化获取器
    ///
    /// # 参数
    /// - `format`: 时间格式
    ///
    /// # 返回值
    /// 获取器定义
    ///
    /// # 示例
    /// ```php
    /// public function getCreateTimeAttr($value) {
    ///     return date('Y-m-d H:i:s', strtotime($value));
    /// }
    /// ```
    pub fn datetime_format(field: &str, format: &str) -> GetterDefinition {
        let format_owned = format.to_string();

        GetterDefinition::new(field, move |value, _data| {
            if let Value::String(s) = value {
                // 尝试解析时间并格式化
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                    return Value::String(dt.format(&format_owned).to_string());
                }
                // 尝试其他格式
                if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                    return Value::String(dt.format(&format_owned).to_string());
                }
            }
            value.clone()
        })
    }

    /// JSON 解析获取器
    ///
    /// # 返回值
    /// 获取器定义
    pub fn json_parse(field: &str) -> GetterDefinition {
        GetterDefinition::new(field, |value, _data| {
            if let Value::String(s) = value {
                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(s) {
                    return json_to_value(&json_val);
                }
            }
            value.clone()
        })
    }

    /// 大写转换获取器
    ///
    /// # 返回值
    /// 获取器定义
    pub fn uppercase(field: &str) -> GetterDefinition {
        GetterDefinition::new(field, |value, _data| {
            Value::String(value.to_string_value().to_uppercase())
        })
    }

    /// 小写转换获取器
    ///
    /// # 返回值
    /// 获取器定义
    pub fn lowercase(field: &str) -> GetterDefinition {
        GetterDefinition::new(field, |value, _data| {
            Value::String(value.to_string_value().to_lowercase())
        })
    }

    /// 截取字符串获取器
    ///
    /// # 参数
    /// - `length`: 截取长度
    ///
    /// # 返回值
    /// 获取器定义
    pub fn substring(field: &str, length: usize) -> GetterDefinition {
        GetterDefinition::new(field, move |value, _data| {
            let s = value.to_string_value();
            if s.len() > length {
                Value::String(format!("{}...", &s[..length]))
            } else {
                Value::String(s)
            }
        })
    }

    /// 默认值获取器
    ///
    /// # 参数
    /// - `default`: 默认值
    ///
    /// # 返回值
    /// 获取器定义
    pub fn default_value(field: &str, default: Value) -> GetterDefinition {
        GetterDefinition::new(field, move |value, _data| {
            if matches!(value, Value::Null) {
                default.clone()
            } else {
                value.clone()
            }
        })
    }

    /// 数字格式化获取器
    ///
    /// # 参数
    /// - `decimals`: 小数位数
    ///
    /// # 返回值
    /// 获取器定义
    pub fn number_format(field: &str, decimals: usize) -> GetterDefinition {
        GetterDefinition::new(field, move |value, _data| {
            let num = match value {
                Value::Int(i) => *i as f64,
                Value::Float(f) => *f,
                Value::String(s) => s.parse().unwrap_or(0.0),
                _ => 0.0,
            };
            Value::String(format!("{:.1$}", num, decimals))
        })
    }

    /// 布尔值文本获取器
    ///
    /// # 参数
    /// - `true_text`: true 时显示的文本
    /// - `false_text`: false 时显示的文本
    ///
    /// # 返回值
    /// 获取器定义
    pub fn bool_text(field: &str, true_text: &str, false_text: &str) -> GetterDefinition {
        let true_owned = true_text.to_string();
        let false_owned = false_text.to_string();

        GetterDefinition::new(field, move |value, _data| {
            let is_true = match value {
                Value::Bool(b) => *b,
                Value::Int(i) => *i != 0,
                Value::String(s) => s == "1" || s.to_lowercase() == "true",
                _ => false,
            };
            Value::String(if is_true { true_owned.clone() } else { false_owned.clone() })
        })
    }

    /// 数组/列表获取器
    ///
    /// # 参数
    /// - `separator`: 分隔符
    ///
    /// # 返回值
    /// 获取器定义
    pub fn split_to_array(field: &str, separator: &str) -> GetterDefinition {
        let sep_owned = separator.to_string();

        GetterDefinition::new(field, move |value, _data| {
            let s = value.to_string_value();
            let items: Vec<Value> = s.split(&sep_owned)
                .map(|item| Value::String(item.to_string()))
                .collect();
            Value::IndexedArray(items)
        })
    }

    /// 掩码获取器（用于敏感信息）
    ///
    /// # 参数
    /// - `start`: 开始位置
    /// - `end`: 结束位置
    /// - `mask_char`: 掩码字符
    ///
    /// # 返回值
    /// 获取器定义
    pub fn mask(field: &str, start: usize, end: usize, mask_char: char) -> GetterDefinition {
        GetterDefinition::new(field, move |value, _data| {
            let s = value.to_string_value();
            let chars: Vec<char> = s.chars().collect();
            let mut result = String::new();

            for (i, c) in chars.iter().enumerate() {
                if i >= start && i < chars.len().saturating_sub(end) {
                    result.push(mask_char);
                } else {
                    result.push(*c);
                }
            }

            Value::String(result)
        })
    }
}

/// JSON 值转换为 Value
///
/// # 参数
/// - `json`: JSON 值
///
/// # 返回值
/// Value 类型
fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::String(n.to_string())
            }
        }
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            let items: Vec<Value> = arr.iter().map(json_to_value).collect();
            Value::IndexedArray(items)
        }
        serde_json::Value::Object(obj) => {
            let items: Vec<(String, Value)> = obj.iter()
                .map(|(k, v)| (k.clone(), json_to_value(v)))
                .collect();
            Value::AssociativeArray(items)
        }
    }
}

/// 查询结果处理器
///
/// 用于对查询结果进行统一处理
pub struct ResultProcessor {
    /// 处理回调列表
    processors: Vec<Box<dyn Fn(&mut ModelInstance) + Send + Sync>>,
}

impl ResultProcessor {
    /// 创建新的结果处理器
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    /// 添加处理器
    ///
    /// # 参数
    /// - `callback`: 处理回调
    ///
    /// # 示例
    /// ```php
    /// User::filter(function($user) {
    ///     $user->name = 'new value';
    ///     $user->test = 'test';
    /// })->select();
    /// ```
    pub fn add<F>(&mut self, callback: F)
    where
        F: Fn(&mut ModelInstance) + Send + Sync + 'static,
    {
        self.processors.push(Box::new(callback));
    }

    /// 处理模型实例
    ///
    /// # 参数
    /// - `model`: 模型实例
    pub fn process(&self, model: &mut ModelInstance) {
        for processor in &self.processors {
            processor(model);
        }
    }

    /// 处理模型列表
    ///
    /// # 参数
    /// - `models`: 模型实例列表
    pub fn process_all(&self, models: &mut [ModelInstance]) {
        for model in models {
            self.process(model);
        }
    }

    /// 清空处理器
    pub fn clear(&mut self) {
        self.processors.clear();
    }
}

impl Default for ResultProcessor {
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
        model.attributes.insert("name".to_string(), Value::String("Test".to_string()));
        model.attributes.insert("status".to_string(), Value::Int(1));
        model.original = model.attributes.clone();
        model.exists = true;
        model
    }

    #[test]
    fn test_getter_manager() {
        let mut manager = GetterManager::new();

        // 添加获取器
        manager.add("status", |value, _data| {
            let status_map = vec!["禁用", "正常", "待审核"];
            if let Value::Int(i) = value {
                if *i >= 0 && (*i as usize) < status_map.len() {
                    return Value::String(status_map[*i as usize].to_string());
                }
            }
            value.clone()
        });

        // 测试获取器
        let mut data = HashMap::new();
        data.insert("status".to_string(), Value::Int(1));

        let result = manager.apply("status", &Value::Int(1), &data);
        assert_eq!(result, Value::String("正常".to_string()));
    }

    #[test]
    fn test_dynamic_getter() {
        let mut manager = GetterManager::new();

        // 添加动态获取器
        manager.add_dynamic("name", |value, _data| {
            Value::String(value.to_string_value().to_lowercase())
        });

        // 测试动态获取器
        let data = HashMap::new();
        let result = manager.apply("name", &Value::String("TEST".to_string()), &data);
        assert_eq!(result, Value::String("test".to_string()));
    }

    #[test]
    fn test_get_data() {
        let model = create_test_model();

        // 获取单个原始字段
        let status = model.get_data(Some("status"));
        assert_eq!(status, Value::Int(1));

        // 获取全部原始数据
        let all_data = model.get_data(None);
        assert!(matches!(all_data, Value::AssociativeArray(_)));
    }

    #[test]
    fn test_predefined_getters() {
        // 测试大写获取器
        let uppercase_getter = PredefinedGetters::uppercase("name");
        let data = HashMap::new();
        let result = uppercase_getter.execute(&Value::String("test".to_string()), &data);
        assert_eq!(result, Value::String("TEST".to_string()));

        // 测试小写获取器
        let lowercase_getter = PredefinedGetters::lowercase("name");
        let result = lowercase_getter.execute(&Value::String("TEST".to_string()), &data);
        assert_eq!(result, Value::String("test".to_string()));
    }

    #[test]
    fn test_result_processor() {
        let mut processor = ResultProcessor::new();

        // 添加处理器
        processor.add(|model| {
            model.attributes.insert("processed".to_string(), Value::Bool(true));
        });

        // 处理模型
        let mut model = create_test_model();
        processor.process(&mut model);

        // 验证处理结果
        assert!(model.attributes.contains_key("processed"));
    }
}
