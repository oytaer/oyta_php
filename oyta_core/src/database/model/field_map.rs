//! 字段映射模块
//!
//! 实现模型的字段映射功能
//! 对应 ThinkPHP 的字段映射功能
//!
//! 主要功能：
//! - 字段名映射
//! - 自动转换
//! - 双向映射

use std::collections::HashMap;

use crate::interpreter::value::Value;
use super::instance::ModelInstance;
use super::types::ModelConfig;

/// 字段映射配置
#[derive(Debug, Clone)]
pub struct FieldMapping {
    /// 映射规则：数据库字段名 -> 模型属性名
    pub map: HashMap<String, String>,
    /// 反向映射：模型属性名 -> 数据库字段名
    pub reverse_map: HashMap<String, String>,
}

impl FieldMapping {
    /// 创建新的字段映射
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            reverse_map: HashMap::new(),
        }
    }

    /// 添加映射规则
    ///
    /// # 参数
    /// - `db_field`: 数据库字段名
    /// - `model_attr`: 模型属性名
    ///
    /// # 示例
    /// ```php
    /// protected $map = [
    ///     'name' => 'real_name',
    /// ];
    /// ```
    pub fn add(&mut self, db_field: &str, model_attr: &str) {
        self.map.insert(db_field.to_string(), model_attr.to_string());
        self.reverse_map.insert(model_attr.to_string(), db_field.to_string());
    }

    /// 批量添加映射规则
    pub fn add_all(&mut self, mappings: &HashMap<&str, &str>) {
        for (db_field, model_attr) in mappings {
            self.add(db_field, model_attr);
        }
    }

    /// 将数据库字段名转换为模型属性名
    ///
    /// # 参数
    /// - `db_field`: 数据库字段名
    ///
    /// # 返回值
    /// 模型属性名
    pub fn to_attr(&self, db_field: &str) -> String {
        // 如果映射中存在，返回映射后的值；否则返回原始值
        self.map.get(db_field).cloned().unwrap_or_else(|| db_field.to_string())
    }

    /// 将模型属性名转换为数据库字段名
    ///
    /// # 参数
    /// - `model_attr`: 模型属性名
    ///
    /// # 返回值
    /// 数据库字段名
    pub fn to_db_field(&self, model_attr: &str) -> String {
        // 如果映射中存在，返回映射后的值；否则返回原始值
        self.reverse_map.get(model_attr).cloned().unwrap_or_else(|| model_attr.to_string())
    }

    /// 检查是否存在映射
    pub fn has_mapping(&self, field: &str) -> bool {
        self.map.contains_key(field) || self.reverse_map.contains_key(field)
    }

    /// 获取所有映射
    pub fn get_all_mappings(&self) -> &HashMap<String, String> {
        &self.map
    }

    /// 清空映射
    pub fn clear(&mut self) {
        self.map.clear();
        self.reverse_map.clear();
    }
}

impl Default for FieldMapping {
    fn default() -> Self {
        Self::new()
    }
}

/// 字段映射处理器
///
/// 处理模型数据的字段映射转换
pub struct FieldMappingProcessor {
    /// 字段映射配置
    pub mapping: FieldMapping,
}

impl FieldMappingProcessor {
    /// 创建新的处理器
    pub fn new() -> Self {
        Self {
            mapping: FieldMapping::new(),
        }
    }

    /// 使用指定映射创建处理器
    pub fn with_mapping(mapping: FieldMapping) -> Self {
        Self { mapping }
    }

    /// 从数据库读取时转换字段名
    ///
    /// 将数据库字段名转换为模型属性名
    ///
    /// # 参数
    /// - `data`: 数据库行数据
    ///
    /// # 返回值
    /// 转换后的数据
    pub fn process_from_db(&self, data: &HashMap<String, Value>) -> HashMap<String, Value> {
        let mut result = HashMap::new();

        for (db_field, value) in data {
            // 转换字段名
            let attr_name = self.mapping.to_attr(db_field);
            result.insert(attr_name.to_string(), value.clone());
        }

        result
    }

    /// 写入数据库时转换字段名
    ///
    /// 将模型属性名转换为数据库字段名
    ///
    /// # 参数
    /// - `data`: 模型属性数据
    ///
    /// # 返回值
    /// 转换后的数据
    pub fn process_to_db(&self, data: &HashMap<String, Value>) -> HashMap<String, Value> {
        let mut result = HashMap::new();

        for (attr_name, value) in data {
            // 转换字段名
            let db_field = self.mapping.to_db_field(attr_name);
            result.insert(db_field.to_string(), value.clone());
        }

        result
    }
}

impl Default for FieldMappingProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// 为 ModelConfig 添加字段映射支持
impl ModelConfig {
    /// 创建带字段映射的模型配置
    pub fn with_field_mapping(mut self, mappings: &HashMap<&str, &str>) -> Self {
        // 存储映射到配置中
        // 注意：需要在 ModelConfig 中添加 field_mapping 字段
        // 这里简化处理
        self
    }
}

/// 为 ModelInstance 添加字段映射扩展方法
impl ModelInstance {
    /// 应用字段映射（从数据库读取后）
    pub fn apply_field_mapping_from_db(&mut self, mapping: &FieldMapping) {
        let mut new_attributes = HashMap::new();

        for (db_field, value) in &self.attributes {
            let attr_name = mapping.to_attr(db_field);
            new_attributes.insert(attr_name.to_string(), value.clone());
        }

        self.attributes = new_attributes;
    }

    /// 应用字段映射（写入数据库前）
    pub fn apply_field_mapping_to_db(&self, mapping: &FieldMapping) -> HashMap<String, Value> {
        let mut result = HashMap::new();

        for (attr_name, value) in &self.attributes {
            let db_field = mapping.to_db_field(attr_name);
            result.insert(db_field.to_string(), value.clone());
        }

        result
    }

    /// 获取映射后的属性名
    pub fn get_mapped_attr(&self, field: &str, mapping: &FieldMapping) -> Value {
        // 获取映射后的属性名
        let attr_name = mapping.to_attr(field);
        // 获取属性值
        self.get_attr(attr_name.as_str())
    }

    /// 设置映射后的属性值
    pub fn set_mapped_attr(&mut self, field: &str, value: Value, mapping: &FieldMapping) {
        // 获取映射后的属性名
        let attr_name = mapping.to_attr(field);
        // 设置属性值
        self.set_attr(attr_name.as_str(), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_mapping() {
        let mut mapping = FieldMapping::new();

        mapping.add("real_name", "name");
        mapping.add("real_email", "email");

        assert_eq!(mapping.to_attr("real_name"), "name");
        assert_eq!(mapping.to_attr("real_email"), "email");
        assert_eq!(mapping.to_db_field("name"), "real_name");
        assert_eq!(mapping.to_db_field("email"), "real_email");
    }

    #[test]
    fn test_field_mapping_processor() {
        let mut mapping = FieldMapping::new();
        mapping.add("real_name", "name");

        let processor = FieldMappingProcessor::with_mapping(mapping);

        // 测试从数据库读取
        let mut db_data = HashMap::new();
        db_data.insert("real_name".to_string(), Value::String("Test".to_string()));

        let result = processor.process_from_db(&db_data);
        assert!(result.contains_key("name"));
        assert!(!result.contains_key("real_name"));

        // 测试写入数据库
        let mut model_data = HashMap::new();
        model_data.insert("name".to_string(), Value::String("Test".to_string()));

        let result = processor.process_to_db(&model_data);
        assert!(result.contains_key("real_name"));
        assert!(!result.contains_key("name"));
    }

    #[test]
    fn test_model_field_mapping() {
        let mut mapping = FieldMapping::new();
        mapping.add("real_name", "name");

        let config = ModelConfig {
            table: "users".to_string(),
            ..ModelConfig::default()
        };
        let mut model = ModelInstance::new(config);

        // 设置原始字段
        model.attributes.insert("real_name".to_string(), Value::String("Test".to_string()));

        // 应用映射
        model.apply_field_mapping_from_db(&mapping);

        // 验证映射结果
        assert!(model.attributes.contains_key("name"));
    }
}
