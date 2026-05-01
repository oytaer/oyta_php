//! 虚拟模型模块
//!
//! 实现虚拟模型功能
//! 对应 ThinkPHP 的虚拟模型（Virtual Model）功能
//!
//! 主要功能：
//! - 虚拟模型不写入数据库
//! - 数据只保存在内存中
//! - 支持获取器、模型事件、关联操作

use std::collections::HashMap;

use crate::interpreter::value::Value;
use super::instance::ModelInstance;
use super::types::ModelConfig;

/// 虚拟模型标记 trait
///
/// 用于标记模型为虚拟模型
pub trait VirtualModel: Sized {
    /// 创建虚拟模型实例
    fn create_virtual(data: HashMap<String, Value>) -> Self;

    /// 保存虚拟模型（不实际写入数据库）
    fn save_virtual(&mut self) -> bool;

    /// 删除虚拟模型（同时删除关联数据）
    fn delete_virtual(&mut self) -> bool;

    /// 检查是否为虚拟模型
    fn is_virtual(&self) -> bool;
}

/// 虚拟模型实例
///
/// 不写入数据库的模型实例
#[derive(Debug, Clone)]
pub struct VirtualModelInstance {
    /// 内部模型实例
    pub inner: ModelInstance,
    /// 是否已修改
    pub dirty: bool,
}

impl VirtualModelInstance {
    /// 创建新的虚拟模型实例
    ///
    /// # 参数
    /// - `config`: 模型配置
    ///
    /// # 返回值
    /// 新的虚拟模型实例
    pub fn new(config: ModelConfig) -> Self {
        // 标记配置为虚拟模型
        let virtual_config = ModelConfig {
            table: config.table,
            pk: config.pk,
            auto_timestamp: false, // 虚拟模型不支持自动时间戳
            ..config
        };

        Self {
            inner: ModelInstance::new(virtual_config),
            dirty: false,
        }
    }

    /// 从数据创建虚拟模型
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `data`: 初始数据
    ///
    /// # 返回值
    /// 新的虚拟模型实例
    pub fn from_data(config: ModelConfig, data: HashMap<String, Value>) -> Self {
        let mut instance = Self::new(config);
        instance.inner.attributes = data;
        instance.inner.exists = true;
        instance
    }

    /// 获取属性值
    ///
    /// # 参数
    /// - `key`: 属性名
    ///
    /// # 返回值
    /// 属性值
    pub fn get_attr(&self, key: &str) -> Value {
        self.inner.get_attr(key)
    }

    /// 设置属性值
    ///
    /// # 参数
    /// - `key`: 属性名
    /// - `value`: 属性值
    pub fn set_attr(&mut self, key: &str, value: Value) {
        self.inner.set_attr(key, value);
        self.dirty = true;
    }

    /// 保存虚拟模型
    ///
    /// 虚拟模型的保存操作不会写入数据库
    /// 只更新内存中的状态
    ///
    /// # 返回值
    /// 总是返回 true
    pub fn save(&mut self) -> bool {
        // 虚拟模型不写入数据库
        // 只同步原始数据
        self.inner.sync_original();
        self.dirty = false;
        true
    }

    /// 删除虚拟模型
    ///
    /// # 返回值
    /// 总是返回 true
    pub fn delete(&mut self) -> bool {
        // 虚拟模型的删除操作
        self.inner.exists = false;
        self.inner.attributes.clear();
        true
    }

    /// 检查是否已修改
    ///
    /// # 返回值
    /// 如果已修改返回 true
    pub fn is_dirty(&self) -> bool {
        self.dirty || self.inner.is_dirty()
    }

    /// 检查是否存在
    ///
    /// # 返回值
    /// 如果存在返回 true
    pub fn exists(&self) -> bool {
        self.inner.exists
    }

    /// 转换为数组
    ///
    /// # 返回值
    /// 数组值
    pub fn to_array(&self) -> Value {
        self.inner.to_value()
    }

    /// 转换为 JSON
    ///
    /// # 返回值
    /// JSON 字符串
    pub fn to_json(&self) -> String {
        match serde_json::to_string(&self.to_array()) {
            Ok(json) => json,
            Err(_) => "{}".to_string(),
        }
    }

    /// 获取所有属性
    ///
    /// # 返回值
    /// 属性映射
    pub fn get_attributes(&self) -> &HashMap<String, Value> {
        &self.inner.attributes
    }

    /// 设置多个属性
    ///
    /// # 参数
    /// - `data`: 属性映射
    pub fn set_attributes(&mut self, data: HashMap<String, Value>) {
        for (key, value) in data {
            self.set_attr(&key, value);
        }
    }

    /// 获取主键值
    ///
    /// # 返回值
    /// 主键值
    pub fn get_key(&self) -> Value {
        self.inner.get_key()
    }

    /// 加载关联
    ///
    /// # 参数
    /// - `relation_name`: 关联名称
    ///
    /// # 返回值
    /// 关联模型（虚拟）
    pub async fn load_relation(&self, relation_name: &str) -> anyhow::Result<Option<VirtualModelInstance>> {
        // 虚拟模型的关联也是虚拟的
        if let Some(relation) = self.inner.config.relations.get(relation_name) {
            // 创建虚拟关联模型
            let related_config = ModelConfig {
                table: class_to_table(&relation.related_model),
                ..ModelConfig::default()
            };

            Ok(Some(VirtualModelInstance::new(related_config)))
        } else {
            Ok(None)
        }
    }

    /// 加载一对多关联
    ///
    /// # 参数
    /// - `relation_name`: 关联名称
    ///
    /// # 返回值
    /// 关联模型列表（虚拟）
    pub async fn load_has_many(&self, relation_name: &str) -> anyhow::Result<Vec<VirtualModelInstance>> {
        // 虚拟模型的关联返回空列表
        if self.inner.config.relations.contains_key(relation_name) {
            Ok(Vec::new())
        } else {
            Ok(Vec::new())
        }
    }

    /// 关联自动删除
    ///
    /// # 参数
    /// - `relations`: 关联名称列表
    ///
    /// # 返回值
    /// 删除成功返回 true
    pub async fn together_delete(&mut self, _relations: &[&str]) -> anyhow::Result<bool> {
        // 虚拟模型的关联删除
        self.delete();
        Ok(true)
    }
}

/// 虚拟模型集合
///
/// 存储多个虚拟模型实例
#[derive(Debug, Clone, Default)]
pub struct VirtualModelCollection {
    /// 模型列表
    pub items: Vec<VirtualModelInstance>,
}

impl VirtualModelCollection {
    /// 创建新的虚拟模型集合
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// 添加模型
    pub fn push(&mut self, model: VirtualModelInstance) {
        self.items.push(model);
    }

    /// 获取模型数量
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 遍历模型
    pub fn iter(&self) -> impl Iterator<Item = &VirtualModelInstance> {
        self.items.iter()
    }

    /// 转换为数组
    pub fn to_array(&self) -> Value {
        let items: Vec<Value> = self.items.iter().map(|m| m.to_array()).collect();
        Value::IndexedArray(items)
    }

    /// 转换为 JSON
    pub fn to_json(&self) -> String {
        match serde_json::to_string(&self.to_array()) {
            Ok(json) => json,
            Err(_) => "[]".to_string(),
        }
    }
}

/// 虚拟模型工厂
///
/// 用于创建虚拟模型
pub struct VirtualModelFactory;

impl VirtualModelFactory {
    /// 创建虚拟模型
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `data`: 初始数据
    ///
    /// # 返回值
    /// 虚拟模型实例
    pub fn create(config: ModelConfig, data: HashMap<String, Value>) -> VirtualModelInstance {
        VirtualModelInstance::from_data(config, data)
    }

    /// 批量创建虚拟模型
    ///
    /// # 参数
    /// - `config`: 模型配置
    /// - `data_list`: 数据列表
    ///
    /// # 返回值
    /// 虚拟模型集合
    pub fn create_batch(config: ModelConfig, data_list: Vec<HashMap<String, Value>>) -> VirtualModelCollection {
        let mut collection = VirtualModelCollection::new();

        for data in data_list {
            let model = Self::create(config.clone(), data);
            collection.push(model);
        }

        collection
    }
}

/// 将类名转换为表名
fn class_to_table(class_name: &str) -> String {
    let short_name = if let Some(pos) = class_name.rfind('\\') {
        &class_name[pos + 1..]
    } else {
        class_name
    };

    let mut result = String::new();
    for (i, c) in short_name.chars().enumerate() {
        if c.is_uppercase() {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ModelConfig {
        ModelConfig {
            table: "users".to_string(),
            pk: "id".to_string(),
            ..ModelConfig::default()
        }
    }

    #[test]
    fn test_virtual_model_create() {
        let config = create_test_config();
        let mut data = HashMap::new();
        data.insert("id".to_string(), Value::Int(1));
        data.insert("name".to_string(), Value::String("Test".to_string()));

        let model = VirtualModelInstance::from_data(config, data);

        assert!(model.exists());
        assert_eq!(model.get_attr("name"), Value::String("Test".to_string()));
    }

    #[test]
    fn test_virtual_model_save() {
        let config = create_test_config();
        let mut model = VirtualModelInstance::new(config);

        model.set_attr("name", Value::String("Test".to_string()));

        assert!(model.is_dirty());

        // 保存不会写入数据库
        let result = model.save();

        assert!(result);
        assert!(!model.is_dirty());
    }

    #[test]
    fn test_virtual_model_delete() {
        let config = create_test_config();
        let mut model = VirtualModelInstance::new(config);

        model.set_attr("id", Value::Int(1));
        model.inner.exists = true;

        let result = model.delete();

        assert!(result);
        assert!(!model.exists());
    }

    #[test]
    fn test_virtual_model_collection() {
        let config = create_test_config();

        let mut collection = VirtualModelCollection::new();

        let mut data1 = HashMap::new();
        data1.insert("id".to_string(), Value::Int(1));

        let mut data2 = HashMap::new();
        data2.insert("id".to_string(), Value::Int(2));

        collection.push(VirtualModelInstance::from_data(config.clone(), data1));
        collection.push(VirtualModelInstance::from_data(config, data2));

        assert_eq!(collection.len(), 2);
        assert!(!collection.is_empty());
    }

    #[test]
    fn test_virtual_model_factory() {
        let config = create_test_config();

        let mut data = HashMap::new();
        data.insert("id".to_string(), Value::Int(1));
        data.insert("name".to_string(), Value::String("Test".to_string()));

        let model = VirtualModelFactory::create(config.clone(), data);

        assert!(model.exists());
    }
}
