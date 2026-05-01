//! 模型输出模块
//!
//! 实现模型的输出转换功能
//! 对应 ThinkPHP 的模型输出功能
//!
//! 主要功能：
//! - toArray 转换为数组
//! - toJson 转换为 JSON
//! - visible 设置可见字段
//! - hidden 设置隐藏字段
//! - append 追加字段
//! - 关联属性输出

use std::collections::HashMap;

use crate::interpreter::value::Value;
use super::instance::ModelInstance;

/// 模型输出配置
///
/// 控制模型输出时的字段处理
#[derive(Debug, Clone, Default)]
pub struct OutputConfig {
    /// 可见字段列表（白名单）
    pub visible: Vec<String>,
    /// 隐藏字段列表（黑名单）
    pub hidden: Vec<String>,
    /// 追加字段列表
    pub append: Vec<String>,
    /// 是否包含关联
    pub with_relations: bool,
    /// 关联输出配置
    pub relation_config: HashMap<String, RelationOutputConfig>,
}

/// 关联输出配置
///
/// 控制关联模型的输出
#[derive(Debug, Clone, Default)]
pub struct RelationOutputConfig {
    /// 可见字段
    pub visible: Vec<String>,
    /// 隐藏字段
    pub hidden: Vec<String>,
    /// 追加字段
    pub append: Vec<String>,
}

impl OutputConfig {
    /// 创建新的输出配置
    pub fn new() -> Self {
        Self {
            visible: Vec::new(),
            hidden: Vec::new(),
            append: Vec::new(),
            with_relations: false,
            relation_config: HashMap::new(),
        }
    }

    /// 设置可见字段
    pub fn set_visible(&mut self, fields: &[&str]) -> &mut Self {
        self.visible = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 设置隐藏字段
    pub fn set_hidden(&mut self, fields: &[&str]) -> &mut Self {
        self.hidden = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 设置追加字段
    pub fn set_append(&mut self, fields: &[&str]) -> &mut Self {
        self.append = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 设置关联输出配置
    pub fn set_relation(&mut self, relation: &str, config: RelationOutputConfig) -> &mut Self {
        self.relation_config.insert(relation.to_string(), config);
        self.with_relations = true;
        self
    }
}

/// 模型输出器
///
/// 提供模型输出转换方法
pub struct ModelOutput {
    /// 输出配置
    pub config: OutputConfig,
}

impl ModelOutput {
    /// 创建新的模型输出器
    pub fn new() -> Self {
        Self {
            config: OutputConfig::new(),
        }
    }

    /// 使用指定配置创建输出器
    pub fn with_config(config: OutputConfig) -> Self {
        Self { config }
    }

    /// 设置可见字段
    ///
    /// # 参数
    /// - `fields`: 可见字段列表
    ///
    /// # 返回值
    /// 自身引用
    ///
    /// # 示例
    /// ```php
    /// $user->visible(['id','name','email'])->toArray();
    /// ```
    pub fn visible(&mut self, fields: &[&str]) -> &mut Self {
        self.config.set_visible(fields);
        self
    }

    /// 设置隐藏字段
    ///
    /// # 参数
    /// - `fields`: 隐藏字段列表
    ///
    /// # 返回值
    /// 自身引用
    ///
    /// # 示例
    /// ```php
    /// $user->hidden(['password','salt'])->toArray();
    /// ```
    pub fn hidden(&mut self, fields: &[&str]) -> &mut Self {
        self.config.set_hidden(fields);
        self
    }

    /// 设置追加字段
    ///
    /// # 参数
    /// - `fields`: 追加字段列表
    ///
    /// # 返回值
    /// 自身引用
    ///
    /// # 示例
    /// ```php
    /// $user->append(['status_text'])->toArray();
    /// ```
    pub fn append(&mut self, fields: &[&str]) -> &mut Self {
        self.config.set_append(fields);
        self
    }

    /// 追加关联属性
    ///
    /// # 参数
    /// - `relation`: 关联名
    /// - `fields`: 字段列表
    ///
    /// # 返回值
    /// 自身引用
    ///
    /// # 示例
    /// ```php
    /// $user->append(['profile' => ['email', 'nickname']])->toArray();
    /// ```
    pub fn append_relation(&mut self, relation: &str, fields: &[&str]) -> &mut Self {
        let mut relation_config = RelationOutputConfig::default();
        relation_config.append = fields.iter().map(|s| s.to_string()).collect();
        self.config.set_relation(relation, relation_config);
        self
    }

    /// 将模型转换为数组
    ///
    /// # 参数
    /// - `model`: 模型实例
    ///
    /// # 返回值
    /// 数组值
    ///
    /// # 示例
    /// ```php
    /// $array = $user->toArray();
    /// ```
    pub fn to_array(&self, model: &ModelInstance) -> Value {
        let mut result: Vec<(String, Value)> = Vec::new();

        // 确定要输出的字段
        let fields_to_output = self.determine_fields(model);

        // 输出字段
        for field in &fields_to_output {
            // 跳过隐藏字段
            if self.config.hidden.contains(field) {
                continue;
            }

            // 获取字段值（经过获取器处理）
            let value = model.get_attr(field);
            result.push((field.clone(), value));
        }

        // 追加字段
        for field in &self.config.append {
            let value = model.get_attr(field);
            result.push((field.clone(), value));
        }

        Value::AssociativeArray(result)
    }

    /// 将模型转换为 JSON
    ///
    /// # 参数
    /// - `model`: 模型实例
    ///
    /// # 返回值
    /// JSON 字符串
    ///
    /// # 示例
    /// ```php
    /// $json = $user->toJson();
    /// ```
    pub fn to_json(&self, model: &ModelInstance) -> String {
        let array = self.to_array(model);

        // 转换为 JSON
        match serde_json::to_string(&array) {
            Ok(json) => json,
            Err(_) => "{}".to_string(),
        }
    }

    /// 确定要输出的字段
    ///
    /// # 参数
    /// - `model`: 模型实例
    ///
    /// # 返回值
    /// 字段名列表
    fn determine_fields(&self, model: &ModelInstance) -> Vec<String> {
        // 如果设置了可见字段，使用可见字段列表
        if !self.config.visible.is_empty() {
            return self.config.visible.clone();
        }

        // 否则使用模型的所有属性字段
        model.attributes.keys().cloned().collect()
    }
}

impl Default for ModelOutput {
    fn default() -> Self {
        Self::new()
    }
}

/// 模型输出扩展 trait
///
/// 为 ModelInstance 提供输出扩展方法
pub trait OutputExt {
    /// 转换为数组
    ///
    /// # 返回值
    /// 数组值
    fn to_array(&self) -> Value;

    /// 转换为 JSON
    ///
    /// # 返回值
    /// JSON 字符串
    fn to_json(&self) -> String;

    /// 设置可见字段
    ///
    /// # 参数
    /// - `fields`: 字段列表
    ///
    /// # 返回值
    /// 新的模型实例
    fn visible(&self, fields: &[&str]) -> ModelInstance;

    /// 设置隐藏字段
    ///
    /// # 参数
    /// - `fields`: 字段列表
    ///
    /// # 返回值
    /// 新的模型实例
    fn hidden(&self, fields: &[&str]) -> ModelInstance;

    /// 追加字段
    ///
    /// # 参数
    /// - `fields`: 字段列表
    ///
    /// # 返回值
    /// 新的模型实例
    fn append(&self, fields: &[&str]) -> ModelInstance;

    /// 绑定关联属性
    ///
    /// # 参数
    /// - `relation`: 关联名
    /// - `fields`: 字段列表
    ///
    /// # 返回值
    /// 新的模型实例
    fn bind_attr(&self, relation: &str, fields: &[&str]) -> ModelInstance;
}

/// 为 ModelInstance 实现输出扩展
impl OutputExt for ModelInstance {
    /// 转换为数组
    fn to_array(&self) -> Value {
        let output = ModelOutput::new();
        output.to_array(self)
    }

    /// 转换为 JSON
    fn to_json(&self) -> String {
        let output = ModelOutput::new();
        output.to_json(self)
    }

    /// 设置可见字段
    fn visible(&self, fields: &[&str]) -> ModelInstance {
        let mut new_model = self.clone();
        new_model.config.hidden_fields.clear();
        new_model.config.hidden_fields.extend(
            self.attributes.keys()
                .filter(|k| !fields.contains(&k.as_str()))
                .map(|k| k.clone())
        );
        new_model
    }

    /// 设置隐藏字段
    fn hidden(&self, fields: &[&str]) -> ModelInstance {
        let mut new_model = self.clone();
        for field in fields {
            if !new_model.config.hidden_fields.contains(&field.to_string()) {
                new_model.config.hidden_fields.push(field.to_string());
            }
        }
        new_model
    }

    /// 追加字段
    fn append(&self, fields: &[&str]) -> ModelInstance {
        let mut new_model = self.clone();
        for field in fields {
            if !new_model.config.append_fields.contains(&field.to_string()) {
                new_model.config.append_fields.push(field.to_string());
            }
        }
        new_model
    }

    /// 绑定关联属性
    fn bind_attr(&self, relation: &str, fields: &[&str]) -> ModelInstance {
        let mut new_model = self.clone();
        // 将关联字段添加到追加列表
        for field in fields {
            let append_field = format!("{}.{}", relation, field);
            if !new_model.config.append_fields.contains(&append_field) {
                new_model.config.append_fields.push(append_field);
            }
        }
        new_model
    }
}

/// 数据集输出器
///
/// 为数据集提供输出方法
pub struct CollectionOutput;

impl CollectionOutput {
    /// 将数据集转换为数组
    ///
    /// # 参数
    /// - `models`: 模型实例列表
    ///
    /// # 返回值
    /// 数组值
    pub fn to_array(models: &[ModelInstance]) -> Value {
        let items: Vec<Value> = models.iter().map(|m| m.to_array()).collect();
        Value::IndexedArray(items)
    }

    /// 将数据集转换为 JSON
    ///
    /// # 参数
    /// - `models`: 模型实例列表
    ///
    /// # 返回值
    /// JSON 字符串
    pub fn to_json(models: &[ModelInstance]) -> String {
        let array = Self::to_array(models);
        match serde_json::to_string(&array) {
            Ok(json) => json,
            Err(_) => "[]".to_string(),
        }
    }

    /// 设置数据集可见字段
    ///
    /// # 参数
    /// - `models`: 模型实例列表
    /// - `fields`: 字段列表
    ///
    /// # 返回值
    /// 新的模型实例列表
    pub fn visible(models: &[ModelInstance], fields: &[&str]) -> Vec<ModelInstance> {
        models.iter().map(|m| m.visible(fields)).collect()
    }

    /// 设置数据集隐藏字段
    ///
    /// # 参数
    /// - `models`: 模型实例列表
    /// - `fields`: 字段列表
    ///
    /// # 返回值
    /// 新的模型实例列表
    pub fn hidden(models: &[ModelInstance], fields: &[&str]) -> Vec<ModelInstance> {
        models.iter().map(|m| m.hidden(fields)).collect()
    }

    /// 设置数据集追加字段
    ///
    /// # 参数
    /// - `models`: 模型实例列表
    /// - `fields`: 字段列表
    ///
    /// # 返回值
    /// 新的模型实例列表
    pub fn append(models: &[ModelInstance], fields: &[&str]) -> Vec<ModelInstance> {
        models.iter().map(|m| m.append(fields)).collect()
    }
}

/// JSON 序列化支持
///
/// 为模型实现 JSON 序列化
impl serde::Serialize for ModelInstance {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // 使用 to_array 方法获取数据
        let array = self.to_array();

        // 序列化数组
        array.serialize(serializer)
    }
}

/// 关联输出处理器
///
/// 处理关联模型的输出
pub struct RelationOutputHandler;

impl RelationOutputHandler {
    /// 处理关联属性追加
    ///
    /// # 参数
    /// - `model`: 模型实例
    /// - `relation`: 关联名
    /// - `fields`: 字段列表
    ///
    /// # 返回值
    /// 处理后的属性映射
    pub fn process_relation_append(
        model: &ModelInstance,
        relation: &str,
        fields: &[&str],
    ) -> HashMap<String, Value> {
        let mut result = HashMap::new();

        // 检查关联是否存在
        if !model.config.relations.contains_key(relation) {
            return result;
        }

        // 为每个字段创建追加属性
        for field in fields {
            let attr_name = format!("{}_{}", relation, field);
            // 标记需要从关联中获取
            result.insert(attr_name, Value::String(format!("__relation:{}:{}", relation, field)));
        }

        result
    }

    /// 解析关联属性标记
    ///
    /// # 参数
    /// - `value`: 可能包含标记的值
    ///
    /// # 返回值
    /// 如果是标记值，返回 (关联名, 字段名)
    pub fn parse_relation_marker(value: &Value) -> Option<(String, String)> {
        if let Value::String(s) = value {
            if s.starts_with("__relation:") {
                let parts: Vec<&str> = s.trim_start_matches("__relation:").split(':').collect();
                if parts.len() == 2 {
                    return Some((parts[0].to_string(), parts[1].to_string()));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ModelConfig;

    fn create_test_model() -> ModelInstance {
        let config = ModelConfig {
            table: "users".to_string(),
            hidden_fields: vec!["password".to_string()],
            ..ModelConfig::default()
        };
        let mut model = ModelInstance::new(config);
        model.attributes.insert("id".to_string(), Value::Int(1));
        model.attributes.insert("name".to_string(), Value::String("Test".to_string()));
        model.attributes.insert("email".to_string(), Value::String("test@example.com".to_string()));
        model.attributes.insert("password".to_string(), Value::String("secret".to_string()));
        model.exists = true;
        model
    }

    #[test]
    fn test_to_array() {
        let model = create_test_model();
        let output = ModelOutput::new();
        let array = output.to_array(&model);

        // 验证输出
        if let Value::AssociativeArray(data) = array {
            let keys: Vec<&String> = data.iter().map(|(k, _)| k).collect();
            assert!(keys.contains(&&"id".to_string()));
            assert!(keys.contains(&&"name".to_string()));
        } else {
            panic!("Expected AssociativeArray");
        }
    }

    #[test]
    fn test_visible() {
        let model = create_test_model();
        let mut output = ModelOutput::new();
        output.visible(&["id", "name"]);

        let array = output.to_array(&model);

        if let Value::AssociativeArray(data) = array {
            assert_eq!(data.len(), 2);
        } else {
            panic!("Expected AssociativeArray");
        }
    }

    #[test]
    fn test_hidden() {
        let model = create_test_model();
        let mut output = ModelOutput::new();
        output.hidden(&["password", "email"]);

        let array = output.to_array(&model);

        if let Value::AssociativeArray(data) = array {
            // password 应该被隐藏
            assert!(!data.iter().any(|(k, _)| k == "password"));
        } else {
            panic!("Expected AssociativeArray");
        }
    }

    #[test]
    fn test_append() {
        let model = create_test_model();
        let mut output = ModelOutput::new();
        output.append(&["status_text"]);

        let array = output.to_array(&model);

        if let Value::AssociativeArray(data) = array {
            // status_text 应该被追加
            assert!(data.iter().any(|(k, _)| k == "status_text"));
        } else {
            panic!("Expected AssociativeArray");
        }
    }

    #[test]
    fn test_to_json() {
        let model = create_test_model();
        let output = ModelOutput::new();
        let json = output.to_json(&model);

        // 验证 JSON 是有效的
        assert!(json.starts_with("{"));
        assert!(json.ends_with("}"));
    }

    #[test]
    fn test_output_ext() {
        let model = create_test_model();

        // 测试 visible 方法 - 返回新模型，隐藏不在列表中的字段
        let visible_model = model.visible(&["id", "name"]);
        // 验证其他字段被添加到隐藏列表
        assert!(visible_model.config.hidden_fields.contains(&"email".to_string()));
        assert!(visible_model.config.hidden_fields.contains(&"password".to_string()));

        // 测试 hidden 方法
        let hidden_model = model.hidden(&["password"]);
        assert!(hidden_model.config.hidden_fields.contains(&"password".to_string()));

        // 测试 append 方法
        let append_model = model.append(&["status_text"]);
        assert!(append_model.config.append_fields.contains(&"status_text".to_string()));
    }
}
