//! 搜索器扩展模块
//!
//! 实现模型的搜索器功能
//! 对应 ThinkPHP 的搜索器（Searcher）功能
//!
//! 主要功能：
//! - 搜索器定义和管理
//! - withSearch 方法
//! - 搜索条件封装

use std::collections::HashMap;

use crate::database::query_builder::QueryBuilder;
use crate::interpreter::value::Value;
use super::instance::ModelInstance;
use super::types::ModelConfig;

/// 搜索器回调类型
///
/// 定义搜索器处理函数的签名
/// 参数：查询构建器可变引用、搜索值、完整数据
/// 注意：由于 QueryBuilder 的方法消耗 self，回调需要使用 clone_and_replace 模式
pub type SearcherCallback = Box<dyn Fn(&mut QueryBuilder, &Value, &HashMap<String, Value>) + Send + Sync>;

/// 在 QueryBuilder 上添加 WHERE 条件的辅助函数
///
/// # 参数
/// - `qb`: 查询构建器可变引用
/// - `field`: 字段名
/// - `operator`: 操作符
/// - `value`: 值
fn add_where_condition(qb: &mut QueryBuilder, field: &str, operator: &str, value: Value) {
    // 克隆当前查询构建器，添加条件后替换
    let new_qb = qb.clone().where_clause(field, operator, value);
    *qb = new_qb;
}

/// 在 QueryBuilder 上添加 WHERE IN 条件的辅助函数
///
/// # 参数
/// - `qb`: 查询构建器可变引用
/// - `field`: 字段名
/// - `values`: 值列表
fn add_where_in_condition(qb: &mut QueryBuilder, field: &str, values: Vec<Value>) {
    // 克隆当前查询构建器，添加条件后替换
    let new_qb = qb.clone().where_in(field, values);
    *qb = new_qb;
}

/// 在 QueryBuilder 上添加 WHERE BETWEEN 条件的辅助函数
///
/// # 参数
/// - `qb`: 查询构建器可变引用
/// - `field`: 字段名
/// - `start`: 开始值
/// - `end`: 结束值
fn add_where_between_condition(qb: &mut QueryBuilder, field: &str, start: Value, end: Value) {
    // 克隆当前查询构建器，添加条件后替换
    let new_qb = qb.clone().where_between(field, start, end);
    *qb = new_qb;
}

/// 在 QueryBuilder 上添加 ORDER BY 条件的辅助函数
///
/// # 参数
/// - `qb`: 查询构建器可变引用
/// - `field`: 字段名
/// - `direction`: 排序方向
fn add_order_condition(qb: &mut QueryBuilder, field: &str, direction: &str) {
    // 克隆当前查询构建器，添加条件后替换
    let new_qb = qb.clone().order(field, direction);
    *qb = new_qb;
}

/// 搜索器定义
///
/// 存储单个搜索器的配置
pub struct SearcherDefinition {
    /// 字段名
    pub field: String,
    /// 回调函数
    pub callback: SearcherCallback,
}

// 手动实现 Debug trait，因为闭包不实现 Debug
impl std::fmt::Debug for SearcherDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 只显示字段名，不显示回调函数
        f.debug_struct("SearcherDefinition")
            .field("field", &self.field)
            .finish()
    }
}

impl SearcherDefinition {
    /// 创建新的搜索器定义
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `callback`: 回调函数
    ///
    /// # 返回值
    /// 新的搜索器定义
    pub fn new<F>(field: &str, callback: F) -> Self
    where
        F: Fn(&mut QueryBuilder, &Value, &HashMap<String, Value>) + Send + Sync + 'static,
    {
        Self {
            field: field.to_string(),
            callback: Box::new(callback),
        }
    }

    /// 执行搜索器
    ///
    /// # 参数
    /// - `qb`: 查询构建器
    /// - `value`: 搜索值
    /// - `data`: 完整数据
    pub fn execute(&self, qb: &mut QueryBuilder, value: &Value, data: &HashMap<String, Value>) {
        (self.callback)(qb, value, data);
    }
}

/// 搜索器管理器
///
/// 管理模型的所有搜索器
#[derive(Debug, Default)]
pub struct SearcherManager {
    /// 搜索器定义映射
    searchers: HashMap<String, SearcherDefinition>,
    /// 字段别名映射
    aliases: HashMap<String, String>,
}

impl SearcherManager {
    /// 创建新的搜索器管理器
    pub fn new() -> Self {
        Self {
            searchers: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    /// 注册搜索器
    ///
    /// # 参数
    /// - `definition`: 搜索器定义
    pub fn register(&mut self, definition: SearcherDefinition) {
        self.searchers.insert(definition.field.clone(), definition);
    }

    /// 注册搜索器（便捷方法）
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `callback`: 回调函数
    pub fn add<F>(&mut self, field: &str, callback: F)
    where
        F: Fn(&mut QueryBuilder, &Value, &HashMap<String, Value>) + Send + Sync + 'static,
    {
        let definition = SearcherDefinition::new(field, callback);
        self.register(definition);
    }

    /// 设置字段别名
    ///
    /// # 参数
    /// - `alias`: 别名
    /// - `field`: 实际字段名
    pub fn set_alias(&mut self, alias: &str, field: &str) {
        self.aliases.insert(alias.to_string(), field.to_string());
    }

    /// 获取实际字段名
    ///
    /// # 参数
    /// - `name`: 名称（可能是别名）
    ///
    /// # 返回值
    /// 实际字段名
    pub fn get_real_field(&self, name: &str) -> String {
        // 如果存在别名映射，返回映射后的字段名，否则返回原名称
        self.aliases.get(name).cloned().unwrap_or_else(|| name.to_string())
    }

    /// 应用搜索器
    ///
    /// # 参数
    /// - `qb`: 查询构建器
    /// - `field`: 字段名
    /// - `value`: 搜索值
    /// - `data`: 完整数据
    pub fn apply(&self, qb: &mut QueryBuilder, field: &str, value: &Value, data: &HashMap<String, Value>) {
        // 获取实际字段名
        let real_field = self.get_real_field(field);

        if let Some(definition) = self.searchers.get(&real_field) {
            definition.execute(qb, value, data);
        }
    }

    /// 应用多个搜索器
    ///
    /// # 参数
    /// - `qb`: 查询构建器
    /// - `fields`: 字段名列表
    /// - `data`: 搜索数据
    ///
    /// # 示例
    /// ```php
    /// User::withSearch(['name','create_time'], [
    ///     'name' => 'think',
    ///     'create_time' => ['2018-8-1','2018-8-5'],
    ///     'status' => 1
    /// ])->select();
    /// ```
    pub fn apply_with_search(&self, qb: &mut QueryBuilder, fields: &[&str], data: &HashMap<String, Value>) {
        for field in fields {
            // 检查数据中是否存在该字段
            if let Some(value) = data.get(*field) {
                // 检查值是否为空
                if !is_empty_value(value) {
                    self.apply(qb, field, value, data);
                }
            }
        }
    }

    /// 检查是否存在搜索器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 如果存在返回 true
    pub fn has(&self, field: &str) -> bool {
        // 获取实际字段名
        let real_field = self.get_real_field(field);
        // 检查搜索器是否存在
        self.searchers.contains_key(&real_field)
    }

    /// 移除搜索器
    ///
    /// # 参数
    /// - `field`: 字段名
    pub fn remove(&mut self, field: &str) {
        self.searchers.remove(field);
        // 移除相关别名
        self.aliases.retain(|_, v| v != field);
    }

    /// 清空所有搜索器
    pub fn clear(&mut self) {
        self.searchers.clear();
        self.aliases.clear();
    }

    /// 获取所有搜索器字段名
    ///
    /// # 返回值
    /// 字段名列表
    pub fn get_fields(&self) -> Vec<&str> {
        self.searchers.keys().map(|s| s.as_str()).collect()
    }
}

/// 检查值是否为空
///
/// # 参数
/// - `value`: 要检查的值
///
/// # 返回值
/// 如果为空返回 true
fn is_empty_value(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        Value::IndexedArray(arr) => arr.is_empty(),
        Value::AssociativeArray(arr) => arr.is_empty(),
        _ => false,
    }
}

/// 搜索器构建器
///
/// 用于链式调用搜索器
pub struct SearcherBuilder {
    /// 查询构建器
    pub query_builder: QueryBuilder,
    /// 搜索器管理器
    pub searcher_manager: SearcherManager,
}

impl SearcherBuilder {
    /// 创建新的搜索器构建器
    ///
    /// # 参数
    /// - `table`: 表名
    pub fn new(table: &str) -> Self {
        Self {
            query_builder: QueryBuilder::new(table),
            searcher_manager: SearcherManager::new(),
        }
    }

    /// 应用搜索
    ///
    /// # 参数
    /// - `fields`: 搜索字段列表
    /// - `data`: 搜索数据
    ///
    /// # 返回值
    /// 自身引用
    ///
    /// # 示例
    /// ```php
    /// User::withSearch(['name','create_time'], $data)->select();
    /// ```
    pub fn with_search(&mut self, fields: &[&str], data: &HashMap<String, Value>) -> &mut Self {
        self.searcher_manager.apply_with_search(&mut self.query_builder, fields, data);
        self
    }

    /// 添加 WHERE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `value`: 值
    ///
    /// # 返回值
    /// 自身引用
    pub fn where_clause(&mut self, field: &str, operator: &str, value: Value) -> &mut Self {
        // 使用辅助函数添加 WHERE 条件
        add_where_condition(&mut self.query_builder, field, operator, value);
        self
    }

    /// 构建 SELECT SQL
    ///
    /// # 返回值
    /// SQL 字符串
    pub fn build_select(&self) -> String {
        self.query_builder.build_select_sql()
    }
}

/// 预定义搜索器
///
/// 提供常用的搜索器模板
pub struct PredefinedSearchers;

impl PredefinedSearchers {
    /// 模糊搜索器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 搜索器定义
    ///
    /// # 示例
    /// ```php
    /// public function searchNameAttr($query, $value, $data) {
    ///     $query->where('name','like', $value . '%');
    /// }
    /// ```
    pub fn like(field: &str) -> SearcherDefinition {
        let field_owned = field.to_string();

        SearcherDefinition::new(field, move |qb, value, _data| {
            // 构建模糊搜索值
            let search_value = format!("{}%", value.to_string_value());
            // 使用辅助函数添加 WHERE 条件
            add_where_condition(qb, &field_owned, "LIKE", Value::String(search_value));
        })
    }

    /// 全模糊搜索器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 搜索器定义
    pub fn like_both(field: &str) -> SearcherDefinition {
        let field_owned = field.to_string();

        SearcherDefinition::new(field, move |qb, value, _data| {
            // 构建全模糊搜索值
            let search_value = format!("%{}%", value.to_string_value());
            // 使用辅助函数添加 WHERE 条件
            add_where_condition(qb, &field_owned, "LIKE", Value::String(search_value));
        })
    }

    /// 精确匹配搜索器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 搜索器定义
    pub fn eq(field: &str) -> SearcherDefinition {
        let field_owned = field.to_string();

        SearcherDefinition::new(field, move |qb, value, _data| {
            // 使用辅助函数添加等于条件
            add_where_condition(qb, &field_owned, "=", value.clone());
        })
    }

    /// 时间范围搜索器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 搜索器定义
    ///
    /// # 示例
    /// ```php
    /// public function searchCreateTimeAttr($query, $value, $data) {
    ///     $query->whereBetweenTime('create_time', $value[0], $value[1]);
    /// }
    /// ```
    pub fn time_between(field: &str) -> SearcherDefinition {
        let field_owned = field.to_string();

        SearcherDefinition::new(field, move |qb, value, _data| {
            // 期望值是数组 [start, end]
            if let Value::IndexedArray(arr) = value {
                if arr.len() >= 2 {
                    let start = &arr[0];
                    let end = &arr[1];
                    // 添加时间范围条件
                    add_where_condition(qb, &field_owned, ">=", start.clone());
                    add_where_condition(qb, &field_owned, "<=", end.clone());
                }
            } else if let Value::AssociativeArray(arr) = value {
                // 支持关联数组 ['start' => ..., 'end' => ...]
                let start = arr.iter().find(|(k, _)| k == "start" || k == "0");
                let end = arr.iter().find(|(k, _)| k == "end" || k == "1");

                if let (Some((_, s)), Some((_, e))) = (start, end) {
                    // 添加时间范围条件
                    add_where_condition(qb, &field_owned, ">=", s.clone());
                    add_where_condition(qb, &field_owned, "<=", e.clone());
                }
            }
        })
    }

    /// 日期范围搜索器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 搜索器定义
    pub fn date_between(field: &str) -> SearcherDefinition {
        let field_owned = field.to_string();

        SearcherDefinition::new(field, move |qb, value, _data| {
            if let Value::IndexedArray(arr) = value {
                if arr.len() >= 2 {
                    let start = arr[0].to_string_value();
                    let end = arr[1].to_string_value();
                    // 添加日期范围条件（包含时间部分）
                    add_where_condition(qb, &field_owned, ">=", Value::String(format!("{} 00:00:00", start)));
                    add_where_condition(qb, &field_owned, "<=", Value::String(format!("{} 23:59:59", end)));
                }
            }
        })
    }

    /// IN 搜索器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 搜索器定义
    pub fn in_values(field: &str) -> SearcherDefinition {
        let field_owned = field.to_string();

        SearcherDefinition::new(field, move |qb, value, _data| {
            if let Value::IndexedArray(arr) = value {
                // 使用辅助函数添加 WHERE IN 条件
                add_where_in_condition(qb, &field_owned, arr.clone());
            } else {
                // 单个值使用等于条件
                add_where_condition(qb, &field_owned, "=", value.clone());
            }
        })
    }

    /// 比较搜索器
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    ///
    /// # 返回值
    /// 搜索器定义
    pub fn compare(field: &str, operator: &str) -> SearcherDefinition {
        let field_owned = field.to_string();
        let operator_owned = operator.to_string();

        SearcherDefinition::new(field, move |qb, value, _data| {
            // 使用辅助函数添加比较条件
            add_where_condition(qb, &field_owned, &operator_owned, value.clone());
        })
    }

    /// 大于搜索器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 搜索器定义
    pub fn gt(field: &str) -> SearcherDefinition {
        Self::compare(field, ">")
    }

    /// 大于等于搜索器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 搜索器定义
    pub fn gte(field: &str) -> SearcherDefinition {
        Self::compare(field, ">=")
    }

    /// 小于搜索器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 搜索器定义
    pub fn lt(field: &str) -> SearcherDefinition {
        Self::compare(field, "<")
    }

    /// 小于等于搜索器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 搜索器定义
    pub fn lte(field: &str) -> SearcherDefinition {
        Self::compare(field, "<=")
    }

    /// 范围搜索器
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 搜索器定义
    pub fn between(field: &str) -> SearcherDefinition {
        let field_owned = field.to_string();

        SearcherDefinition::new(field, move |qb, value, _data| {
            if let Value::IndexedArray(arr) = value {
                if arr.len() >= 2 {
                    // 使用辅助函数添加 BETWEEN 条件
                    add_where_between_condition(qb, &field_owned, arr[0].clone(), arr[1].clone());
                }
            }
        })
    }

    /// 带排序的搜索器
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `sort_field`: 排序字段
    ///
    /// # 返回值
    /// 搜索器定义
    pub fn with_order(field: &str, sort_field: &str) -> SearcherDefinition {
        let field_owned = field.to_string();
        let sort_field_owned = sort_field.to_string();

        SearcherDefinition::new(field, move |qb, value, data| {
            // 应用搜索条件
            add_where_condition(qb, &field_owned, "LIKE", Value::String(format!("{}%", value.to_string_value())));

            // 检查是否有排序参数
            if let Some(Value::AssociativeArray(sort_data)) = data.get("sort") {
                if let Some(sort_value) = sort_data.iter().find(|(k, _)| k == &sort_field_owned) {
                    // 获取排序方向
                    let direction = match &sort_value.1 {
                        Value::String(d) => d.clone(),
                        _ => "ASC".to_string(),
                    };
                    // 使用辅助函数添加排序条件
                    add_order_condition(qb, &sort_field_owned, &direction);
                }
            }
        })
    }
}

/// 为 ModelConfig 添加搜索器支持
impl ModelConfig {
    /// 创建带搜索的查询构建器
    ///
    /// # 参数
    /// - `fields`: 搜索字段列表
    /// - `data`: 搜索数据
    /// - `searcher_manager`: 搜索器管理器
    ///
    /// # 返回值
    /// 查询构建器
    pub fn create_query_with_search(
        &self,
        fields: &[&str],
        data: &HashMap<String, Value>,
        searcher_manager: &SearcherManager,
    ) -> QueryBuilder {
        // 创建查询构建器
        let mut qb = QueryBuilder::new(&self.full_table());

        // 应用搜索器
        searcher_manager.apply_with_search(&mut qb, fields, data);

        qb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_searcher_manager() {
        let mut manager = SearcherManager::new();

        // 添加搜索器
        manager.add("name", |qb, value, _data| {
            // 使用辅助函数添加 WHERE 条件
            add_where_condition(qb, "name", "LIKE", Value::String(format!("{}%", value.to_string_value())));
        });

        // 测试搜索器
        let mut qb = QueryBuilder::new("users");
        let data = HashMap::new();

        manager.apply(&mut qb, "name", &Value::String("test".to_string()), &data);

        let sql = qb.build_select_sql();
        assert!(sql.contains("LIKE"));
    }

    #[test]
    fn test_with_search() {
        let mut manager = SearcherManager::new();

        // 添加搜索器
        manager.add("name", |qb, value, _data| {
            // 使用辅助函数添加 WHERE 条件
            add_where_condition(qb, "name", "LIKE", Value::String(format!("{}%", value.to_string_value())));
        });

        manager.add("status", |qb, value, _data| {
            // 使用辅助函数添加等于条件
            add_where_condition(qb, "status", "=", value.clone());
        });

        // 应用搜索
        let mut qb = QueryBuilder::new("users");
        let mut data = HashMap::new();
        data.insert("name".to_string(), Value::String("test".to_string()));
        data.insert("status".to_string(), Value::Int(1));
        data.insert("other".to_string(), Value::String("ignored".to_string()));

        manager.apply_with_search(&mut qb, &["name", "status", "other"], &data);

        let sql = qb.build_select_sql();
        assert!(sql.contains("LIKE"));
        // 字段名被反引号包裹（安全转义）
        assert!(sql.contains("`status` =") || sql.contains("status ="));
    }

    #[test]
    fn test_searcher_alias() {
        let mut manager = SearcherManager::new();

        // 添加搜索器
        manager.add("username", |qb, value, _data| {
            // 使用辅助函数添加等于条件
            add_where_condition(qb, "username", "=", value.clone());
        });

        // 设置别名
        manager.set_alias("user", "username");

        // 使用别名
        let mut qb = QueryBuilder::new("users");
        let data = HashMap::new();

        manager.apply(&mut qb, "user", &Value::String("test".to_string()), &data);

        let sql = qb.build_select_sql();
        // 字段名被反引号包裹（安全转义）
        assert!(sql.contains("`username` =") || sql.contains("username ="));
    }

    #[test]
    fn test_predefined_searchers() {
        // 测试模糊搜索器
        let like_searcher = PredefinedSearchers::like("name");
        let mut qb = QueryBuilder::new("users");
        let data = HashMap::new();

        like_searcher.execute(&mut qb, &Value::String("test".to_string()), &data);
        let sql = qb.build_select_sql();
        assert!(sql.contains("test%"));

        // 测试时间范围搜索器
        let time_searcher = PredefinedSearchers::time_between("created_at");
        let mut qb2 = QueryBuilder::new("users");
        let time_value = Value::IndexedArray(vec![
            Value::String("2024-01-01".to_string()),
            Value::String("2024-12-31".to_string()),
        ]);

        time_searcher.execute(&mut qb2, &time_value, &data);
        let sql2 = qb2.build_select_sql();
        assert!(sql2.contains(">="));
        assert!(sql2.contains("<="));
    }
}
