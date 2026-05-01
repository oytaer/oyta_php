//! 数据集类模块
//!
//! 实现模型查询返回的数据集对象
//! 对应 ThinkPHP 的 think\model\Collection 类
//!
//! 主要功能：
//! - 数据集遍历和访问
//! - 数据集过滤和排序
//! - 数据集聚合操作
//! - 批量更新和删除

use std::collections::HashMap;
use std::sync::Arc;

use crate::interpreter::value::Value;
use super::instance::ModelInstance;
use super::types::ModelConfig;

/// 数据集类
///
/// 模型 select 查询返回的数据集对象
/// 继承自基础 Collection，提供额外的模型操作方法
#[derive(Debug, Clone)]
pub struct ModelCollection {
    /// 数据项列表
    items: Vec<ModelInstance>,
    /// 是否为空标记
    empty: bool,
}

impl ModelCollection {
    /// 创建新的数据集
    ///
    /// # 参数
    /// - `items`: 模型实例列表
    ///
    /// # 返回值
    /// 新的数据集实例
    pub fn new(items: Vec<ModelInstance>) -> Self {
        // 判断是否为空
        let empty = items.is_empty();
        Self { items, empty }
    }

    /// 创建空数据集
    ///
    /// # 返回值
    /// 空的数据集实例
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            empty: true,
        }
    }

    /// 判断数据集是否为空
    ///
    /// # 返回值
    /// 如果数据集为空返回 true
    ///
    /// # 示例
    /// ```php
    /// $users = User::select();
    /// if($users->isEmpty()){
    ///     echo '数据集为空';
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.empty
    }

    /// 获取数据集数量
    ///
    /// # 返回值
    /// 数据集中的元素数量
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// 获取数据集长度（count 的别名）
    ///
    /// # 返回值
    /// 数据集中的元素数量
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 获取指定索引的元素
    ///
    /// # 参数
    /// - `index`: 索引值
    ///
    /// # 返回值
    /// 如果存在返回模型实例的引用，否则返回 None
    pub fn get(&self, index: usize) -> Option<&ModelInstance> {
        self.items.get(index)
    }

    /// 获取指定索引的元素（可变引用）
    ///
    /// # 参数
    /// - `index`: 索引值
    ///
    /// # 返回值
    /// 如果存在返回模型实例的可变引用，否则返回 None
    pub fn get_mut(&mut self, index: usize) -> Option<&mut ModelInstance> {
        self.items.get_mut(index)
    }

    /// 获取第一个元素
    ///
    /// # 返回值
    /// 第一个模型实例的引用，如果为空返回 None
    pub fn first(&self) -> Option<&ModelInstance> {
        self.items.first()
    }

    /// 获取最后一个元素
    ///
    /// # 返回值
    /// 最后一个模型实例的引用，如果为空返回 None
    pub fn last(&self) -> Option<&ModelInstance> {
        self.items.last()
    }

    /// 获取所有数据项
    ///
    /// # 返回值
    /// 模型实例列表的引用
    pub fn all(&self) -> &[ModelInstance] {
        &self.items
    }

    /// 将数据集转换为向量
    ///
    /// # 返回值
    /// 模型实例向量
    pub fn to_vec(&self) -> Vec<ModelInstance> {
        self.items.clone()
    }

    /// 遍历数据集
    ///
    /// # 返回值
    /// 迭代器
    pub fn iter(&self) -> impl Iterator<Item = &ModelInstance> {
        self.items.iter()
    }

    /// 可变遍历数据集
    ///
    /// # 返回值
    /// 可变迭代器
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ModelInstance> {
        self.items.iter_mut()
    }

    // ==================== 数据集过滤方法 ====================

    /// 过滤数据集（类似 SQL WHERE）
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `value`: 字段值
    ///
    /// # 返回值
    /// 过滤后的新数据集
    ///
    /// # 示例
    /// ```php
    /// $list = User::select()->where('name', 'think');
    /// ```
    pub fn where_eq(&self, field: &str, value: &Value) -> Self {
        // 过滤符合条件的元素
        let filtered: Vec<ModelInstance> = self.items
            .iter()
            .filter(|item| {
                // 获取字段值
                let field_value = item.get_attr(field);
                // 比较值
                &field_value == value
            })
            .cloned()
            .collect();

        Self::new(filtered)
    }

    /// 过滤数据集（自定义条件）
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符（=, !=, >, <, >=, <=）
    /// - `value`: 比较值
    ///
    /// # 返回值
    /// 过滤后的新数据集
    pub fn where_clause(&self, field: &str, operator: &str, value: &Value) -> Self {
        // 过滤符合条件的元素
        let filtered: Vec<ModelInstance> = self.items
            .iter()
            .filter(|item| {
                // 获取字段值
                let field_value = item.get_attr(field);
                // 根据操作符比较
                compare_values(&field_value, operator, value)
            })
            .cloned()
            .collect();

        Self::new(filtered)
    }

    /// 模糊查询过滤
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `pattern`: 匹配模式（支持 % 通配符）
    ///
    /// # 返回值
    /// 过滤后的新数据集
    ///
    /// # 示例
    /// ```php
    /// $list = User::select()->whereLike('name', 'think%');
    /// ```
    pub fn where_like(&self, field: &str, pattern: &str) -> Self {
        // 将 SQL LIKE 模式转换为正则表达式
        let regex_pattern = pattern
            .replace('%', ".*")
            .replace('_', ".");

        // 构建正则表达式
        let regex = regex::Regex::new(&format!("^{}$", regex_pattern)).ok();

        // 过滤符合条件的元素
        let filtered: Vec<ModelInstance> = self.items
            .iter()
            .filter(|item| {
                // 获取字段值
                let field_value = item.get_attr(field);
                // 转换为字符串
                let str_value = field_value.to_string_value();
                // 匹配正则
                if let Some(re) = &regex {
                    re.is_match(&str_value)
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        Self::new(filtered)
    }

    /// IN 查询过滤
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `values`: 值列表
    ///
    /// # 返回值
    /// 过滤后的新数据集
    ///
    /// # 示例
    /// ```php
    /// $list = User::select()->whereIn('id', [1, 2, 3]);
    /// ```
    pub fn where_in(&self, field: &str, values: &[Value]) -> Self {
        // 过滤符合条件的元素
        let filtered: Vec<ModelInstance> = self.items
            .iter()
            .filter(|item| {
                // 获取字段值
                let field_value = item.get_attr(field);
                // 检查是否在值列表中
                values.contains(&field_value)
            })
            .cloned()
            .collect();

        Self::new(filtered)
    }

    /// NOT IN 查询过滤
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `values`: 值列表
    ///
    /// # 返回值
    /// 过滤后的新数据集
    pub fn where_not_in(&self, field: &str, values: &[Value]) -> Self {
        // 过滤不符合条件的元素
        let filtered: Vec<ModelInstance> = self.items
            .iter()
            .filter(|item| {
                // 获取字段值
                let field_value = item.get_attr(field);
                // 检查是否不在值列表中
                !values.contains(&field_value)
            })
            .cloned()
            .collect();

        Self::new(filtered)
    }

    /// BETWEEN 查询过滤
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `min`: 最小值
    /// - `max`: 最大值
    ///
    /// # 返回值
    /// 过滤后的新数据集
    ///
    /// # 示例
    /// ```php
    /// $list = User::select()->whereBetween('score', [80, 100]);
    /// ```
    pub fn where_between(&self, field: &str, min: &Value, max: &Value) -> Self {
        // 过滤符合条件的元素
        let filtered: Vec<ModelInstance> = self.items
            .iter()
            .filter(|item| {
                // 获取字段值
                let field_value = item.get_attr(field);
                // 检查是否在范围内
                compare_values(&field_value, ">=", min) && compare_values(&field_value, "<=", max)
            })
            .cloned()
            .collect();

        Self::new(filtered)
    }

    /// NULL 值过滤
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 过滤后的新数据集
    pub fn where_null(&self, field: &str) -> Self {
        // 过滤字段值为 NULL 的元素
        let filtered: Vec<ModelInstance> = self.items
            .iter()
            .filter(|item| {
                let field_value = item.get_attr(field);
                matches!(field_value, Value::Null)
            })
            .cloned()
            .collect();

        Self::new(filtered)
    }

    /// 非 NULL 值过滤
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 过滤后的新数据集
    pub fn where_not_null(&self, field: &str) -> Self {
        // 过滤字段值不为 NULL 的元素
        let filtered: Vec<ModelInstance> = self.items
            .iter()
            .filter(|item| {
                let field_value = item.get_attr(field);
                !matches!(field_value, Value::Null)
            })
            .cloned()
            .collect();

        Self::new(filtered)
    }

    // ==================== 数据集排序方法 ====================

    /// 排序数据集
    ///
    /// # 参数
    /// - `field`: 排序字段
    /// - `order`: 排序方向（asc/desc）
    ///
    /// # 返回值
    /// 排序后的新数据集
    ///
    /// # 示例
    /// ```php
    /// $list = User::select()->order('create_time', 'desc');
    /// ```
    pub fn order(&self, field: &str, order: &str) -> Self {
        // 复制数据
        let mut sorted = self.items.clone();

        // 根据排序方向排序
        match order.to_lowercase().as_str() {
            "desc" | "descend" => {
                // 降序排序
                sorted.sort_by(|a, b| {
                    let a_val = a.get_attr(field);
                    let b_val = b.get_attr(field);
                    compare_values_for_sort(&b_val, &a_val)
                });
            }
            _ => {
                // 升序排序（默认）
                sorted.sort_by(|a, b| {
                    let a_val = a.get_attr(field);
                    let b_val = b.get_attr(field);
                    compare_values_for_sort(&a_val, &b_val)
                });
            }
        }

        Self::new(sorted)
    }

    /// 升序排序
    ///
    /// # 参数
    /// - `field`: 排序字段
    ///
    /// # 返回值
    /// 排序后的新数据集
    pub fn order_asc(&self, field: &str) -> Self {
        self.order(field, "asc")
    }

    /// 降序排序
    ///
    /// # 参数
    /// - `field`: 排序字段
    ///
    /// # 返回值
    /// 排序后的新数据集
    pub fn order_desc(&self, field: &str) -> Self {
        self.order(field, "desc")
    }

    /// 多字段排序
    ///
    /// # 参数
    /// - `orders`: 排序规则列表（字段名, 方向）
    ///
    /// # 返回值
    /// 排序后的新数据集
    pub fn order_by(&self, orders: &[(&str, &str)]) -> Self {
        // 复制数据
        let mut sorted = self.items.clone();

        // 多字段排序（从后往前排序，确保第一个排序规则优先级最高）
        for (field, order) in orders.iter().rev() {
            match order.to_lowercase().as_str() {
                "desc" | "descend" => {
                    sorted.sort_by(|a, b| {
                        let a_val = a.get_attr(field);
                        let b_val = b.get_attr(field);
                        compare_values_for_sort(&b_val, &a_val)
                    });
                }
                _ => {
                    sorted.sort_by(|a, b| {
                        let a_val = a.get_attr(field);
                        let b_val = b.get_attr(field);
                        compare_values_for_sort(&a_val, &b_val)
                    });
                }
            }
        }

        Self::new(sorted)
    }

    // ==================== 数据集聚合方法 ====================

    /// 计算差集
    ///
    /// # 参数
    /// - `other`: 另一个数据集
    ///
    /// # 返回值
    /// 差集数据集（存在于当前数据集但不存在于另一个数据集的元素）
    ///
    /// # 示例
    /// ```php
    /// $diff = $list1->diff($list2);
    /// ```
    pub fn diff(&self, other: &ModelCollection) -> Self {
        // 获取另一个数据集的主键集合
        let other_keys: Vec<String> = other.items
            .iter()
            .map(|m| m.get_key().to_string_value())
            .collect();

        // 过滤出不在另一个数据集中的元素
        let diff_items: Vec<ModelInstance> = self.items
            .iter()
            .filter(|item| {
                let key = item.get_key().to_string_value();
                !other_keys.contains(&key)
            })
            .cloned()
            .collect();

        Self::new(diff_items)
    }

    /// 计算交集
    ///
    /// # 参数
    /// - `other`: 另一个数据集
    ///
    /// # 返回值
    /// 交集数据集（同时存在于两个数据集的元素）
    ///
    /// # 示例
    /// ```php
    /// $intersect = $list1->intersect($list2);
    /// ```
    pub fn intersect(&self, other: &ModelCollection) -> Self {
        // 获取另一个数据集的主键集合
        let other_keys: Vec<String> = other.items
            .iter()
            .map(|m| m.get_key().to_string_value())
            .collect();

        // 过滤出同时在另一个数据集中的元素
        let intersect_items: Vec<ModelInstance> = self.items
            .iter()
            .filter(|item| {
                let key = item.get_key().to_string_value();
                other_keys.contains(&key)
            })
            .cloned()
            .collect();

        Self::new(intersect_items)
    }

    /// 合并数据集
    ///
    /// # 参数
    /// - `other`: 另一个数据集
    ///
    /// # 返回值
    /// 合并后的新数据集
    pub fn merge(&self, other: &ModelCollection) -> Self {
        // 合并两个数据集
        let mut merged = self.items.clone();
        merged.extend(other.items.clone());
        Self::new(merged)
    }

    /// 去重
    ///
    /// # 参数
    /// - `field`: 用于去重的字段（可选，默认使用主键）
    ///
    /// # 返回值
    /// 去重后的新数据集
    pub fn unique(&self, field: Option<&str>) -> Self {
        let mut seen_keys: Vec<String> = Vec::new();
        let mut unique_items: Vec<ModelInstance> = Vec::new();

        for item in &self.items {
            // 获取用于去重的键值
            let key = if let Some(f) = field {
                item.get_attr(f).to_string_value()
            } else {
                item.get_key().to_string_value()
            };

            // 检查是否已存在
            if !seen_keys.contains(&key) {
                seen_keys.push(key);
                unique_items.push(item.clone());
            }
        }

        Self::new(unique_items)
    }

    // ==================== 数据集切片方法 ====================

    /// 获取指定数量的元素
    ///
    /// # 参数
    /// - `n`: 数量
    ///
    /// # 返回值
    /// 包含前 n 个元素的新数据集
    pub fn take(&self, n: usize) -> Self {
        let taken: Vec<ModelInstance> = self.items.iter().take(n).cloned().collect();
        Self::new(taken)
    }

    /// 跳过指定数量的元素
    ///
    /// # 参数
    /// - `n`: 数量
    ///
    /// # 返回值
    /// 跳过前 n 个元素后的新数据集
    pub fn skip(&self, n: usize) -> Self {
        let skipped: Vec<ModelInstance> = self.items.iter().skip(n).cloned().collect();
        Self::new(skipped)
    }

    /// 分页获取数据
    ///
    /// # 参数
    /// - `page`: 页码（从 1 开始）
    /// - `per_page`: 每页数量
    ///
    /// # 返回值
    /// 指定页的数据集
    pub fn for_page(&self, page: usize, per_page: usize) -> Self {
        let offset = (page.saturating_sub(1)) * per_page;
        let items: Vec<ModelInstance> = self.items
            .iter()
            .skip(offset)
            .take(per_page)
            .cloned()
            .collect();
        Self::new(items)
    }

    // ==================== 数据集转换方法 ====================

    /// 转换为数组
    ///
    /// # 返回值
    /// 包含所有模型数据的数组
    ///
    /// # 示例
    /// ```php
    /// $array = $list->toArray();
    /// ```
    pub fn to_array(&self) -> Value {
        // 将每个模型转换为关联数组
        let items: Vec<Value> = self.items
            .iter()
            .map(|item| item.to_value())
            .collect();

        Value::IndexedArray(items)
    }

    /// 转换为 JSON 字符串
    ///
    /// # 参数
    /// - `options`: JSON 编码选项
    ///
    /// # 返回值
    /// JSON 字符串
    ///
    /// # 示例
    /// ```php
    /// $json = $list->toJson();
    /// ```
    pub fn to_json(&self, options: Option<u32>) -> String {
        // 获取数组值
        let array = self.to_array();

        // 转换为 JSON
        match serde_json::to_string(&array) {
            Ok(json) => json,
            Err(_) => "[]".to_string(),
        }
    }

    /// 获取某列的值
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 该字段值的数组
    ///
    /// # 示例
    /// ```php
    /// $names = $list->column('name');
    /// ```
    pub fn column(&self, field: &str) -> Vec<Value> {
        self.items
            .iter()
            .map(|item| item.get_attr(field))
            .collect()
    }

    /// 获取键值对（以某字段为键，某字段为值）
    ///
    /// # 参数
    /// - `key_field`: 作为键的字段
    /// - `value_field`: 作为值的字段
    ///
    /// # 返回值
    /// 键值对 HashMap
    pub fn key_value_pair(&self, key_field: &str, value_field: &str) -> HashMap<String, Value> {
        let mut result = HashMap::new();

        for item in &self.items {
            let key = item.get_attr(key_field).to_string_value();
            let value = item.get_attr(value_field);
            result.insert(key, value);
        }

        result
    }

    /// 以某字段为键重新索引数据集
    ///
    /// # 参数
    /// - `field`: 作为键的字段
    ///
    /// # 返回值
    /// 以指定字段为键的 HashMap
    pub fn key_by(&self, field: &str) -> HashMap<String, ModelInstance> {
        let mut result = HashMap::new();

        for item in &self.items {
            let key = item.get_attr(field).to_string_value();
            result.insert(key, item.clone());
        }

        result
    }

    /// 以某字段分组
    ///
    /// # 参数
    /// - `field`: 分组字段
    ///
    /// # 返回值
    /// 分组后的 HashMap
    pub fn group_by(&self, field: &str) -> HashMap<String, Vec<ModelInstance>> {
        let mut result: HashMap<String, Vec<ModelInstance>> = HashMap::new();

        for item in &self.items {
            let key = item.get_attr(field).to_string_value();
            result.entry(key).or_insert_with(Vec::new).push(item.clone());
        }

        result
    }

    // ==================== 数据集输出处理方法 ====================

    /// 设置隐藏字段
    ///
    /// # 参数
    /// - `fields`: 要隐藏的字段列表
    ///
    /// # 返回值
    /// 处理后的新数据集
    ///
    /// # 示例
    /// ```php
    /// $list = $list->hidden(['password', 'salt']);
    /// ```
    pub fn hidden(&self, fields: &[&str]) -> Self {
        // 为每个模型设置隐藏字段
        let items: Vec<ModelInstance> = self.items
            .iter()
            .map(|item| {
                let mut new_item = item.clone();
                for field in fields {
                    new_item.config.hidden_fields.push(field.to_string());
                }
                new_item
            })
            .collect();

        Self::new(items)
    }

    /// 设置可见字段
    ///
    /// # 参数
    /// - `fields`: 要显示的字段列表
    ///
    /// # 返回值
    /// 处理后的新数据集
    ///
    /// # 示例
    /// ```php
    /// $list = $list->visible(['id', 'name', 'email']);
    /// ```
    pub fn visible(&self, fields: &[&str]) -> Self {
        // 为每个模型设置可见字段
        let items: Vec<ModelInstance> = self.items
            .iter()
            .map(|item| {
                let mut new_item = item.clone();
                // 清空隐藏字段，只保留可见字段
                new_item.config.hidden_fields.clear();
                // 将不在可见列表中的字段添加到隐藏列表
                for (key, _) in &item.attributes {
                    if !fields.contains(&key.as_str()) {
                        new_item.config.hidden_fields.push(key.clone());
                    }
                }
                new_item
            })
            .collect();

        Self::new(items)
    }

    /// 追加字段
    ///
    /// # 参数
    /// - `fields`: 要追加的字段列表（通常是获取器定义的虚拟字段）
    ///
    /// # 返回值
    /// 处理后的新数据集
    ///
    /// # 示例
    /// ```php
    /// $list = $list->append(['status_text']);
    /// ```
    pub fn append(&self, fields: &[&str]) -> Self {
        // 为每个模型追加字段
        let items: Vec<ModelInstance> = self.items
            .iter()
            .map(|item| {
                let mut new_item = item.clone();
                for field in fields {
                    new_item.config.append_fields.push(field.to_string());
                }
                new_item
            })
            .collect();

        Self::new(items)
    }

    /// 动态获取器
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `callback`: 处理回调
    ///
    /// # 返回值
    /// 处理后的新数据集
    ///
    /// # 示例
    /// ```php
    /// $list = $list->withAttr('name', function($value) {
    ///     return strtolower($value);
    /// });
    /// ```
    pub fn with_attr<F>(&self, field: &str, callback: F) -> Self
    where
        F: Fn(&Value) -> Value + Clone,
    {
        // 为每个模型应用动态获取器
        let items: Vec<ModelInstance> = self.items
            .iter()
            .map(|item| {
                let mut new_item = item.clone();
                // 获取原始值
                if let Some(value) = new_item.attributes.get(field) {
                    // 应用回调
                    let new_value = callback(value);
                    new_item.attributes.insert(field.to_string(), new_value);
                }
                new_item
            })
            .collect();

        Self::new(items)
    }

    // ==================== 批量操作方法 ====================

    /// 批量更新数据集
    ///
    /// # 参数
    /// - `data`: 要更新的数据
    ///
    /// # 返回值
    /// 更新成功返回 true
    ///
    /// # 示例
    /// ```php
    /// $list = User::where('status', 1)->select();
    /// $list->update(['name' => 'php']);
    /// ```
    pub async fn update(&mut self, data: &HashMap<String, Value>) -> anyhow::Result<bool> {
        // 遍历所有模型实例并更新
        for item in &mut self.items {
            // 设置更新数据
            for (key, value) in data {
                item.set_attr(key, value.clone());
            }
            // 保存到数据库
            item.save().await?;
        }

        Ok(true)
    }

    /// 批量删除数据集
    ///
    /// # 返回值
    /// 删除成功返回 true
    ///
    /// # 示例
    /// ```php
    /// $list = User::where('status', 1)->select();
    /// $list->delete();
    /// ```
    pub async fn delete(&mut self) -> anyhow::Result<bool> {
        // 遍历所有模型实例并删除
        for item in &mut self.items {
            item.delete().await?;
        }

        // 清空数据集
        self.items.clear();
        self.empty = true;

        Ok(true)
    }

    // ==================== 延迟预载入方法 ====================

    /// 延迟预载入关联
    ///
    /// # 参数
    /// - `relations`: 要预载入的关联列表
    ///
    /// # 返回值
    /// 预载入成功返回 true
    ///
    /// # 示例
    /// ```php
    /// $list = User::select([1, 2, 3]);
    /// $list->load(['cards']);
    /// ```
    pub async fn load(&mut self, relations: &[&str]) -> anyhow::Result<()> {
        // 遍历所有关联
        for relation in relations {
            // 为每个模型加载关联
            for item in &mut self.items {
                // 加载关联数据
                let _ = item.load_relation(relation).await?;
            }
        }

        Ok(())
    }

    /// 延迟预载入关联（带缓存）
    ///
    /// # 参数
    /// - `relations`: 要预载入的关联列表
    /// - `cache_ttl`: 缓存时间（秒）
    ///
    /// # 返回值
    /// 预载入成功返回 true
    pub async fn load_with_cache(&mut self, relations: &[&str], _cache_ttl: u32) -> anyhow::Result<()> {
        // TODO: 实现带缓存的延迟预载入
        self.load(relations).await
    }

    // ==================== 聚合统计方法 ====================

    /// 求和
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 求和结果
    pub fn sum(&self, field: &str) -> f64 {
        self.items
            .iter()
            .map(|item| {
                let value = item.get_attr(field);
                value_to_f64(&value)
            })
            .sum()
    }

    /// 求平均值
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 平均值
    pub fn avg(&self, field: &str) -> f64 {
        if self.items.is_empty() {
            return 0.0;
        }

        let sum = self.sum(field);
        sum / self.items.len() as f64
    }

    /// 求最大值
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 最大值
    pub fn max(&self, field: &str) -> f64 {
        self.items
            .iter()
            .map(|item| {
                let value = item.get_attr(field);
                value_to_f64(&value)
            })
            .fold(f64::NEG_INFINITY, |a, b| a.max(b))
    }

    /// 求最小值
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 最小值
    pub fn min(&self, field: &str) -> f64 {
        self.items
            .iter()
            .map(|item| {
                let value = item.get_attr(field);
                value_to_f64(&value)
            })
            .fold(f64::INFINITY, |a, b| a.min(b))
    }

    /// 按条件过滤（使用闭包）
    ///
    /// # 参数
    /// - `predicate`: 过滤条件闭包
    ///
    /// # 返回值
    /// 过滤后的新数据集
    pub fn filter<F>(&self, predicate: F) -> Self
    where
        F: Fn(&ModelInstance) -> bool,
    {
        let filtered: Vec<ModelInstance> = self.items
            .iter()
            .filter(|item| predicate(item))
            .cloned()
            .collect();

        Self::new(filtered)
    }

    /// 映射转换（使用闭包）
    ///
    /// # 参数
    /// - `mapper`: 转换闭包
    ///
    /// # 返回值
    /// 转换后的新数据集
    pub fn map<F>(&self, mut mapper: F) -> Self
    where
        F: FnMut(&ModelInstance) -> ModelInstance,
    {
        let mapped: Vec<ModelInstance> = self.items
            .iter()
            .map(|item| mapper(item))
            .collect();

        Self::new(mapped)
    }

    /// 遍历执行（使用闭包）
    ///
    /// # 参数
    /// - `action`: 执行闭包
    pub fn each<F>(&self, mut action: F)
    where
        F: FnMut(&ModelInstance),
    {
        for item in &self.items {
            action(item);
        }
    }

    /// 检查是否存在符合条件的元素
    ///
    /// # 参数
    /// - `predicate`: 条件闭包
    ///
    /// # 返回值
    /// 如果存在返回 true
    pub fn some<F>(&self, predicate: F) -> bool
    where
        F: Fn(&ModelInstance) -> bool,
    {
        self.items.iter().any(|item| predicate(item))
    }

    /// 检查是否所有元素都符合条件
    ///
    /// # 参数
    /// - `predicate`: 条件闭包
    ///
    /// # 返回值
    /// 如果全部符合返回 true
    pub fn every<F>(&self, predicate: F) -> bool
    where
        F: Fn(&ModelInstance) -> bool,
    {
        self.items.iter().all(|item| predicate(item))
    }

    /// 查找第一个符合条件的元素
    ///
    /// # 参数
    /// - `predicate`: 条件闭包
    ///
    /// # 返回值
    /// 找到返回模型实例，否则返回 None
    pub fn find<F>(&self, predicate: F) -> Option<&ModelInstance>
    where
        F: Fn(&ModelInstance) -> bool,
    {
        self.items.iter().find(|item| predicate(item))
    }

    /// 查找第一个符合条件的元素索引
    ///
    /// # 参数
    /// - `predicate`: 条件闭包
    ///
    /// # 返回值
    /// 找到返回索引，否则返回 None
    pub fn find_index<F>(&self, predicate: F) -> Option<usize>
    where
        F: Fn(&ModelInstance) -> bool,
    {
        self.items.iter().position(|item| predicate(item))
    }

    /// 反转数据集
    ///
    /// # 返回值
    /// 反转后的新数据集
    pub fn reverse(&self) -> Self {
        let mut reversed = self.items.clone();
        reversed.reverse();
        Self::new(reversed)
    }

    /// 打乱数据集顺序
    ///
    /// # 返回值
    /// 随机排序后的新数据集
    pub fn shuffle(&self) -> Self {
        let mut shuffled = self.items.clone();
        // 使用简单的随机打乱算法
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hash};

        // 随机排序
        let random_state = RandomState::new();
        shuffled.sort_by(|a, b| {
            let hash_a = random_state.hash_one(a.get_key().to_string_value());
            let hash_b = random_state.hash_one(b.get_key().to_string_value());
            hash_a.cmp(&hash_b)
        });

        Self::new(shuffled)
    }

    /// 随机获取一个元素
    ///
    /// # 返回值
    /// 随机选择的模型实例
    pub fn random(&self) -> Option<&ModelInstance> {
        if self.items.is_empty() {
            return None;
        }

        // 使用简单的随机选择
        let index = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as usize % self.items.len())
            .unwrap_or(0);

        self.items.get(index)
    }

    /// 分块数据集
    ///
    /// # 参数
    /// - `size`: 每块大小
    ///
    /// # 返回值
    /// 分块后的数据集列表
    pub fn chunk(&self, size: usize) -> Vec<Self> {
        if size == 0 {
            return vec![];
        }

        self.items
            .chunks(size)
            .map(|chunk| Self::new(chunk.to_vec()))
            .collect()
    }

    /// 拼接数据集元素
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `separator`: 分隔符
    ///
    /// # 返回值
    /// 拼接后的字符串
    pub fn implode(&self, field: &str, separator: &str) -> String {
        self.items
            .iter()
            .map(|item| item.get_attr(field).to_string_value())
            .collect::<Vec<_>>()
            .join(separator)
    }

    /// 弹出最后一个元素
    ///
    /// # 返回值
    /// 弹出的模型实例
    pub fn pop(&mut self) -> Option<ModelInstance> {
        let item = self.items.pop();
        self.empty = self.items.is_empty();
        item
    }

    /// 弹出第一个元素
    ///
    /// # 返回值
    /// 弹出的模型实例
    pub fn shift(&mut self) -> Option<ModelInstance> {
        if self.items.is_empty() {
            return None;
        }
        let item = self.items.remove(0);
        self.empty = self.items.is_empty();
        Some(item)
    }

    /// 在末尾添加元素
    ///
    /// # 参数
    /// - `item`: 要添加的模型实例
    pub fn push(&mut self, item: ModelInstance) {
        self.items.push(item);
        self.empty = false;
    }

    /// 在开头添加元素
    ///
    /// # 参数
    /// - `item`: 要添加的模型实例
    pub fn unshift(&mut self, item: ModelInstance) {
        self.items.insert(0, item);
        self.empty = false;
    }

    /// 移除指定索引的元素
    ///
    /// # 参数
    /// - `index`: 索引值
    ///
    /// # 返回值
    /// 被移除的模型实例
    pub fn remove(&mut self, index: usize) -> Option<ModelInstance> {
        if index >= self.items.len() {
            return None;
        }
        let item = self.items.remove(index);
        self.empty = self.items.is_empty();
        Some(item)
    }

    /// 清空数据集
    pub fn clear(&mut self) {
        self.items.clear();
        self.empty = true;
    }
}

/// 比较两个值
///
/// # 参数
/// - `left`: 左值
/// - `operator`: 操作符
/// - `right`: 右值
///
/// # 返回值
/// 比较结果
fn compare_values(left: &Value, operator: &str, right: &Value) -> bool {
    match operator {
        "=" | "==" => left == right,
        "!=" | "<>" => left != right,
        ">" => compare_values_for_sort(left, right) == std::cmp::Ordering::Greater,
        "<" => compare_values_for_sort(left, right) == std::cmp::Ordering::Less,
        ">=" => {
            let cmp = compare_values_for_sort(left, right);
            cmp == std::cmp::Ordering::Greater || cmp == std::cmp::Ordering::Equal
        }
        "<=" => {
            let cmp = compare_values_for_sort(left, right);
            cmp == std::cmp::Ordering::Less || cmp == std::cmp::Ordering::Equal
        }
        _ => false,
    }
}

/// 比较两个值用于排序
///
/// # 参数
/// - `left`: 左值
/// - `right`: 右值
///
/// # 返回值
/// 排序比较结果
fn compare_values_for_sort(left: &Value, right: &Value) -> std::cmp::Ordering {
    match (left, right) {
        // 整数比较
        (Value::Int(a), Value::Int(b)) => a.cmp(b),
        // 浮点数比较
        (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
        // 整数和浮点数比较
        (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)).unwrap_or(std::cmp::Ordering::Equal),
        // 字符串比较
        (Value::String(a), Value::String(b)) => a.cmp(b),
        // 布尔值比较
        (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
        // NULL 值处理
        (Value::Null, Value::Null) => std::cmp::Ordering::Equal,
        (Value::Null, _) => std::cmp::Ordering::Less,
        (_, Value::Null) => std::cmp::Ordering::Greater,
        // 其他情况转为字符串比较
        _ => left.to_string_value().cmp(&right.to_string_value()),
    }
}

/// 将值转换为 f64
///
/// # 参数
/// - `value`: 要转换的值
///
/// # 返回值
/// 转换后的 f64 值
fn value_to_f64(value: &Value) -> f64 {
    match value {
        Value::Int(i) => *i as f64,
        Value::Float(f) => *f,
        Value::String(s) => s.parse().unwrap_or(0.0),
        Value::Bool(b) => if *b { 1.0 } else { 0.0 },
        _ => 0.0,
    }
}

/// 实现默认值
impl Default for ModelCollection {
    fn default() -> Self {
        Self::empty()
    }
}

/// 实现 IntoIterator trait
impl IntoIterator for ModelCollection {
    type Item = ModelInstance;
    type IntoIter = std::vec::IntoIter<ModelInstance>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

/// 实现索引访问
impl std::ops::Index<usize> for ModelCollection {
    type Output = ModelInstance;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

/// 实现显示 trait
impl std::fmt::Display for ModelCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ModelCollection({} items)", self.items.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ModelConfig;

    fn create_test_model(id: i64, name: &str) -> ModelInstance {
        let config = ModelConfig {
            table: "users".to_string(),
            pk: "id".to_string(),
            ..ModelConfig::default()
        };
        let mut model = ModelInstance::new(config);
        model.attributes.insert("id".to_string(), Value::Int(id));
        model.attributes.insert("name".to_string(), Value::String(name.to_string()));
        model.exists = true;
        model
    }

    #[test]
    fn test_collection_basic() {
        let items = vec![
            create_test_model(1, "user1"),
            create_test_model(2, "user2"),
            create_test_model(3, "user3"),
        ];

        let collection = ModelCollection::new(items);

        assert_eq!(collection.count(), 3);
        assert!(!collection.is_empty());
        assert_eq!(collection.first().unwrap().get_attr("name").to_string_value(), "user1");
        assert_eq!(collection.last().unwrap().get_attr("name").to_string_value(), "user3");
    }

    #[test]
    fn test_collection_filter() {
        let items = vec![
            create_test_model(1, "apple"),
            create_test_model(2, "banana"),
            create_test_model(3, "apricot"),
        ];

        let collection = ModelCollection::new(items);
        let filtered = collection.where_like("name", "ap%");

        assert_eq!(filtered.count(), 2);
    }

    #[test]
    fn test_collection_order() {
        let items = vec![
            create_test_model(3, "charlie"),
            create_test_model(1, "alice"),
            create_test_model(2, "bob"),
        ];

        let collection = ModelCollection::new(items);
        let sorted = collection.order_asc("id");

        assert_eq!(sorted[0].get_attr("id"), Value::Int(1));
        assert_eq!(sorted[1].get_attr("id"), Value::Int(2));
        assert_eq!(sorted[2].get_attr("id"), Value::Int(3));
    }

    #[test]
    fn test_collection_aggregation() {
        let mut items = vec![
            create_test_model(1, "user1"),
            create_test_model(2, "user2"),
            create_test_model(3, "user3"),
        ];

        // 添加 score 字段
        items[0].attributes.insert("score".to_string(), Value::Float(80.0));
        items[1].attributes.insert("score".to_string(), Value::Float(90.0));
        items[2].attributes.insert("score".to_string(), Value::Float(85.0));

        let collection = ModelCollection::new(items);

        assert_eq!(collection.sum("score"), 255.0);
        assert_eq!(collection.avg("score"), 85.0);
        assert_eq!(collection.max("score"), 90.0);
        assert_eq!(collection.min("score"), 80.0);
    }

    #[test]
    fn test_collection_column() {
        let items = vec![
            create_test_model(1, "alice"),
            create_test_model(2, "bob"),
            create_test_model(3, "charlie"),
        ];

        let collection = ModelCollection::new(items);
        let names = collection.column("name");

        assert_eq!(names.len(), 3);
        assert_eq!(names[0], Value::String("alice".to_string()));
    }

    #[test]
    fn test_collection_chunk() {
        let items: Vec<ModelInstance> = (1..=10)
            .map(|i| create_test_model(i, &format!("user{}", i)))
            .collect();

        let collection = ModelCollection::new(items);
        let chunks = collection.chunk(3);

        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0].count(), 3);
        assert_eq!(chunks[3].count(), 1);
    }
}
