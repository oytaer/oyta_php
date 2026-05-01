//! 查询范围模块
//!
//! 实现模型的查询范围功能
//! 对应 ThinkPHP 的查询范围（Scope）功能
//!
//! 主要功能：
//! - scope 方法定义查询条件封装
//! - 全局查询范围
//! - 动态关闭查询范围

use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

use crate::database::query_builder::QueryBuilder;
use crate::interpreter::value::Value;
use super::instance::ModelInstance;
use super::types::ModelConfig;

/// 查询范围回调类型
///
/// 用于定义查询范围的处理函数
pub type ScopeCallback = Box<dyn Fn(&mut QueryBuilder) + Send + Sync>;

/// 异步查询范围回调类型
pub type AsyncScopeCallback = Box<dyn Fn(QueryBuilder) -> Pin<Box<dyn Future<Output = QueryBuilder> + Send>> + Send + Sync>;

/// 查询范围定义
///
/// 存储单个查询范围的配置
pub struct ScopeDefinition {
    /// 范围名称
    pub name: String,
    /// 同步回调函数
    pub callback: Option<ScopeCallback>,
    /// 异步回调函数
    pub async_callback: Option<AsyncScopeCallback>,
    /// 是否为全局范围
    pub is_global: bool,
}

/// 手动实现 Debug trait，因为闭包不支持 Debug
impl fmt::Debug for ScopeDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 格式化输出查询范围信息
        f.debug_struct("ScopeDefinition")
            .field("name", &self.name)
            .field("callback", &self.callback.as_ref().map(|_| "<closure>"))
            .field("async_callback", &self.async_callback.as_ref().map(|_| "<async closure>"))
            .field("is_global", &self.is_global)
            .finish()
    }
}

impl ScopeDefinition {
    /// 创建新的查询范围定义
    ///
    /// # 参数
    /// - `name`: 范围名称
    ///
    /// # 返回值
    /// 新的查询范围定义
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            callback: None,
            async_callback: None,
            is_global: false,
        }
    }

    /// 设置同步回调
    ///
    /// # 参数
    /// - `callback`: 回调函数
    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut QueryBuilder) + Send + Sync + 'static,
    {
        self.callback = Some(Box::new(callback));
        self
    }

    /// 设置异步回调
    ///
    /// # 参数
    /// - `callback`: 异步回调函数
    pub fn with_async_callback<F, Fut>(mut self, callback: F) -> Self
    where
        F: Fn(QueryBuilder) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = QueryBuilder> + Send + 'static,
    {
        self.async_callback = Some(Box::new(move |qb| Box::pin(callback(qb))));
        self
    }

    /// 设置为全局范围
    pub fn as_global(mut self) -> Self {
        self.is_global = true;
        self
    }

    /// 应用查询范围到查询构建器
    ///
    /// # 参数
    /// - `qb`: 查询构建器
    pub fn apply(&self, qb: &mut QueryBuilder) {
        if let Some(callback) = &self.callback {
            callback(qb);
        }
    }

    /// 异步应用查询范围到查询构建器
    ///
    /// # 参数
    /// - `qb`: 查询构建器
    ///
    /// # 返回值
    /// 应用后的查询构建器
    pub async fn apply_async(&self, qb: QueryBuilder) -> QueryBuilder {
        if let Some(callback) = &self.async_callback {
            callback(qb).await
        } else {
            qb
        }
    }
}

/// 查询范围管理器
///
/// 管理模型的所有查询范围
pub struct ScopeManager {
    /// 查询范围定义映射
    scopes: HashMap<String, ScopeDefinition>,
    /// 全局查询范围列表
    global_scopes: Vec<String>,
    /// 已禁用的全局范围
    disabled_scopes: Vec<String>,
}

/// 手动实现 Debug trait，因为 ScopeDefinition 包含闭包
impl fmt::Debug for ScopeManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 格式化输出查询范围管理器信息
        f.debug_struct("ScopeManager")
            .field("scopes", &format!("{} scopes", self.scopes.len()))
            .field("global_scopes", &self.global_scopes)
            .field("disabled_scopes", &self.disabled_scopes)
            .finish()
    }
}

/// 实现 Default trait
impl Default for ScopeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeManager {
    /// 创建新的查询范围管理器
    pub fn new() -> Self {
        Self {
            scopes: HashMap::new(),
            global_scopes: Vec::new(),
            disabled_scopes: Vec::new(),
        }
    }

    /// 注册查询范围
    ///
    /// # 参数
    /// - `definition`: 查询范围定义
    ///
    /// # 示例
    /// ```php
    /// class User extends Model {
    ///     public function scopeThinkphp($query) {
    ///         $query->where('name', 'thinkphp')->field('id,name');
    ///     }
    /// }
    /// ```
    pub fn register(&mut self, definition: ScopeDefinition) {
        // 如果是全局范围，添加到全局列表
        if definition.is_global {
            let name = definition.name.clone();
            if !self.global_scopes.contains(&name) {
                self.global_scopes.push(name.clone());
            }
        }
        
        // 存储范围定义
        self.scopes.insert(definition.name.clone(), definition);
    }

    /// 注册同步查询范围
    ///
    /// # 参数
    /// - `name`: 范围名称
    /// - `callback`: 回调函数
    pub fn add_scope<F>(&mut self, name: &str, callback: F)
    where
        F: Fn(&mut QueryBuilder) + Send + Sync + 'static,
    {
        let definition = ScopeDefinition::new(name).with_callback(callback);
        self.register(definition);
    }

    /// 注册异步查询范围
    ///
    /// # 参数
    /// - `name`: 范围名称
    /// - `callback`: 异步回调函数
    pub fn add_async_scope<F, Fut>(&mut self, name: &str, callback: F)
    where
        F: Fn(QueryBuilder) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = QueryBuilder> + Send + 'static,
    {
        let definition = ScopeDefinition::new(name).with_async_callback(callback);
        self.register(definition);
    }

    /// 注册全局查询范围
    ///
    /// # 参数
    /// - `name`: 范围名称
    /// - `callback`: 回调函数
    ///
    /// # 示例
    /// ```php
    /// class User extends Model {
    ///     protected $globalScope = ['status'];
    ///     
    ///     public function scopeStatus($query) {
    ///         $query->where('status', 1);
    ///     }
    /// }
    /// ```
    pub fn add_global_scope<F>(&mut self, name: &str, callback: F)
    where
        F: Fn(&mut QueryBuilder) + Send + Sync + 'static,
    {
        let definition = ScopeDefinition::new(name)
            .with_callback(callback)
            .as_global();
        self.register(definition);
    }

    /// 应用查询范围
    ///
    /// # 参数
    /// - `qb`: 查询构建器
    /// - `name`: 范围名称
    pub fn apply_scope(&self, qb: &mut QueryBuilder, name: &str) {
        if let Some(definition) = self.scopes.get(name) {
            definition.apply(qb);
        }
    }

    /// 应用多个查询范围
    ///
    /// # 参数
    /// - `qb`: 查询构建器
    /// - `names`: 范围名称列表（逗号分隔或数组）
    ///
    /// # 示例
    /// ```php
    /// User::scope('thinkphp,age')->select();
    /// ```
    pub fn apply_scopes(&self, qb: &mut QueryBuilder, names: &[&str]) {
        for name in names {
            self.apply_scope(qb, name);
        }
    }

    /// 应用全局查询范围
    ///
    /// # 参数
    /// - `qb`: 查询构建器
    pub fn apply_global_scopes(&self, qb: &mut QueryBuilder) {
        for name in &self.global_scopes {
            // 跳过已禁用的范围
            if self.disabled_scopes.contains(name) {
                continue;
            }

            if let Some(definition) = self.scopes.get(name) {
                definition.apply(qb);
            }
        }
    }

    /// 禁用全局查询范围
    ///
    /// # 参数
    /// - `names`: 要禁用的范围名称列表
    ///
    /// # 示例
    /// ```php
    /// User::withoutGlobalScope(['status'])->select();
    /// ```
    pub fn without_global_scope(&mut self, names: &[&str]) {
        for name in names {
            if !self.disabled_scopes.contains(&name.to_string()) {
                self.disabled_scopes.push(name.to_string());
            }
        }
    }

    /// 禁用所有全局查询范围
    ///
    /// # 示例
    /// ```php
    /// User::withoutGlobalScope()->select();
    /// ```
    pub fn without_all_global_scopes(&mut self) {
        self.disabled_scopes = self.global_scopes.clone();
    }

    /// 恢复全局查询范围
    ///
    /// # 参数
    /// - `names`: 要恢复的范围名称列表
    pub fn restore_global_scope(&mut self, names: &[&str]) {
        self.disabled_scopes.retain(|n| !names.contains(&n.as_str()));
    }

    /// 恢复所有全局查询范围
    pub fn restore_all_global_scopes(&mut self) {
        self.disabled_scopes.clear();
    }

    /// 检查查询范围是否存在
    ///
    /// # 参数
    /// - `name`: 范围名称
    ///
    /// # 返回值
    /// 如果存在返回 true
    pub fn has_scope(&self, name: &str) -> bool {
        self.scopes.contains_key(name)
    }

    /// 获取所有查询范围名称
    ///
    /// # 返回值
    /// 范围名称列表
    pub fn get_scope_names(&self) -> Vec<&str> {
        self.scopes.keys().map(|s| s.as_str()).collect()
    }

    /// 获取全局查询范围名称
    ///
    /// # 返回值
    /// 全局范围名称列表
    pub fn get_global_scope_names(&self) -> Vec<&str> {
        self.global_scopes.iter().map(|s| s.as_str()).collect()
    }

    /// 移除查询范围
    ///
    /// # 参数
    /// - `name`: 范围名称
    pub fn remove_scope(&mut self, name: &str) {
        self.scopes.remove(name);
        self.global_scopes.retain(|n| n != name);
        self.disabled_scopes.retain(|n| n != name);
    }

    /// 清空所有查询范围
    pub fn clear(&mut self) {
        self.scopes.clear();
        self.global_scopes.clear();
        self.disabled_scopes.clear();
    }
}

/// 查询范围构建器
///
/// 用于链式调用查询范围
pub struct ScopeBuilder {
    /// 查询构建器
    pub query_builder: QueryBuilder,
    /// 范围管理器
    pub scope_manager: ScopeManager,
}

impl ScopeBuilder {
    /// 创建新的范围构建器
    ///
    /// # 参数
    /// - `table`: 表名
    pub fn new(table: &str) -> Self {
        Self {
            query_builder: QueryBuilder::new(table),
            scope_manager: ScopeManager::new(),
        }
    }

    /// 应用查询范围
    ///
    /// # 参数
    /// - `name`: 范围名称
    ///
    /// # 返回值
    /// 自身引用（支持链式调用）
    ///
    /// # 示例
    /// ```php
    /// User::scope('thinkphp')->find();
    /// ```
    pub fn scope(&mut self, name: &str) -> &mut Self {
        self.scope_manager.apply_scope(&mut self.query_builder, name);
        self
    }

    /// 应用多个查询范围
    ///
    /// # 参数
    /// - `names`: 范围名称列表（逗号分隔）
    ///
    /// # 返回值
    /// 自身引用
    ///
    /// # 示例
    /// ```php
    /// User::scope('thinkphp,age')->select();
    /// ```
    pub fn scopes(&mut self, names: &str) -> &mut Self {
        let scope_names: Vec<&str> = names.split(',').map(|s| s.trim()).collect();
        self.scope_manager.apply_scopes(&mut self.query_builder, &scope_names);
        self
    }

    /// 禁用全局查询范围
    ///
    /// # 参数
    /// - `names`: 范围名称列表
    ///
    /// # 返回值
    /// 自身引用
    pub fn without_global_scope(&mut self, names: &[&str]) -> &mut Self {
        self.scope_manager.without_global_scope(names);
        self
    }

    /// 禁用所有全局查询范围
    ///
    /// # 返回值
    /// 自身引用
    pub fn without_global_scopes(&mut self) -> &mut Self {
        self.scope_manager.without_all_global_scopes();
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
        // 使用可变引用版本的方法添加 WHERE 条件
        self.query_builder.where_clause_mut(field, operator, value);
        self
    }

    /// 添加等于条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `value`: 值
    ///
    /// # 返回值
    /// 自身引用
    pub fn where_eq(&mut self, field: &str, value: Value) -> &mut Self {
        // 调用 where_clause 方法添加等于条件
        self.where_clause(field, "=", value)
    }

    /// 设置字段
    ///
    /// # 参数
    /// - `fields`: 字段列表
    ///
    /// # 返回值
    /// 自身引用
    pub fn field(&mut self, fields: &[&str]) -> &mut Self {
        // 使用可变引用版本的方法设置字段
        self.query_builder.field_mut(fields.join(", ").as_str());
        self
    }

    /// 设置限制
    ///
    /// # 参数
    /// - `limit`: 限制数量
    ///
    /// # 返回值
    /// 自身引用
    pub fn limit(&mut self, limit: usize) -> &mut Self {
        // 使用可变引用版本的方法设置限制（将 usize 转换为 u64）
        self.query_builder.limit_mut(limit as u64);
        self
    }

    /// 设置排序
    ///
    /// # 参数
    /// - `field`: 排序字段
    /// - `order`: 排序方向
    ///
    /// # 返回值
    /// 自身引用
    pub fn order(&mut self, field: &str, order: &str) -> &mut Self {
        // 使用可变引用版本的方法设置排序
        self.query_builder.order_mut(field, order);
        self
    }

    /// 构建 SELECT SQL
    ///
    /// # 返回值
    /// SQL 字符串
    pub fn build_select(&self) -> String {
        self.query_builder.build_select_sql()
    }

    /// 获取查询构建器
    ///
    /// # 返回值
    /// 查询构建器引用
    pub fn get_query_builder(&self) -> &QueryBuilder {
        &self.query_builder
    }

    /// 获取可变查询构建器
    ///
    /// # 返回值
    /// 可变查询构建器引用
    pub fn get_query_builder_mut(&mut self) -> &mut QueryBuilder {
        &mut self.query_builder
    }
}

/// 预定义查询范围
///
/// 提供常用的查询范围模板
pub struct PredefinedScopes;

impl PredefinedScopes {
    /// 创建状态范围
    ///
    /// # 参数
    /// - `status_value`: 状态值
    ///
    /// # 返回值
    /// 查询范围定义
    pub fn status(status_value: i64) -> ScopeDefinition {
        ScopeDefinition::new("status")
            .with_callback(move |qb| {
                // 使用可变引用版本的方法添加 WHERE 条件
                qb.where_clause_mut("status", "=", Value::Int(status_value));
            })
    }

    /// 创建时间范围
    ///
    /// # 参数
    /// - `field`: 时间字段名
    /// - `start`: 开始时间
    /// - `end`: 结束时间
    ///
    /// # 返回值
    /// 查询范围定义
    pub fn time_range(field: &str, start: &str, end: &str) -> ScopeDefinition {
        // 复制字段名和时间值
        let field_owned = field.to_string();
        let start_owned = start.to_string();
        let end_owned = end.to_string();

        ScopeDefinition::new(&format!("{}_range", field))
            .with_callback(move |qb| {
                // 使用可变引用版本的方法添加 WHERE 条件
                qb.where_clause_mut(&field_owned, ">=", Value::String(start_owned.clone()));
                qb.where_clause_mut(&field_owned, "<=", Value::String(end_owned.clone()));
            })
    }

    /// 创建今日范围
    ///
    /// # 参数
    /// - `field`: 时间字段名
    ///
    /// # 返回值
    /// 查询范围定义
    pub fn today(field: &str) -> ScopeDefinition {
        // 复制字段名
        let field_owned = field.to_string();

        ScopeDefinition::new(&format!("{}_today", field))
            .with_callback(move |qb| {
                // 获取今天的日期
                let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                // 使用可变引用版本的方法添加 WHERE 条件
                qb.where_clause_mut(&field_owned, ">=", Value::String(format!("{} 00:00:00", today)));
                qb.where_clause_mut(&field_owned, "<=", Value::String(format!("{} 23:59:59", today)));
            })
    }

    /// 创建最近 N 天范围
    ///
    /// # 参数
    /// - `field`: 时间字段名
    /// - `days`: 天数
    ///
    /// # 返回值
    /// 查询范围定义
    pub fn recent_days(field: &str, days: i64) -> ScopeDefinition {
        // 复制字段名
        let field_owned = field.to_string();

        ScopeDefinition::new(&format!("{}_recent_{}", field, days))
            .with_callback(move |qb| {
                // 计算时间范围
                let end = chrono::Local::now();
                let start = end - chrono::Duration::days(days);
                // 使用可变引用版本的方法添加 WHERE 条件
                qb.where_clause_mut(&field_owned, ">=", Value::String(start.format("%Y-%m-%d %H:%M:%S").to_string()));
                qb.where_clause_mut(&field_owned, "<=", Value::String(end.format("%Y-%m-%d %H:%M:%S").to_string()));
            })
    }

    /// 创建排序范围
    ///
    /// # 参数
    /// - `field`: 排序字段
    /// - `order`: 排序方向
    ///
    /// # 返回值
    /// 查询范围定义
    pub fn order_by(field: &str, order: &str) -> ScopeDefinition {
        // 复制字段名和排序方向
        let field_owned = field.to_string();
        let order_owned = order.to_string();

        ScopeDefinition::new(&format!("order_{}_{}", field, order))
            .with_callback(move |qb| {
                // 使用可变引用版本的方法添加排序
                qb.order_mut(&field_owned, &order_owned);
            })
    }

    /// 创建限制范围
    ///
    /// # 参数
    /// - `limit`: 限制数量
    ///
    /// # 返回值
    /// 查询范围定义
    pub fn limit_n(limit: usize) -> ScopeDefinition {
        ScopeDefinition::new(&format!("limit_{}", limit))
            .with_callback(move |qb| {
                // 使用可变引用版本的方法设置限制（将 usize 转换为 u64）
                qb.limit_mut(limit as u64);
            })
    }

    /// 创建搜索范围
    ///
    /// # 参数
    /// - `field`: 搜索字段
    /// - `keyword`: 关键词
    ///
    /// # 返回值
    /// 查询范围定义
    pub fn search(field: &str, keyword: &str) -> ScopeDefinition {
        // 复制字段名和关键词
        let field_owned = field.to_string();
        let keyword_owned = keyword.to_string();

        ScopeDefinition::new(&format!("search_{}", field))
            .with_callback(move |qb| {
                // 使用可变引用版本的方法添加 LIKE 条件
                qb.where_clause_mut(&field_owned, "LIKE", Value::String(format!("%{}%", keyword_owned)));
            })
    }

    /// 创建 IN 范围
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `values`: 值列表
    ///
    /// # 返回值
    /// 查询范围定义
    pub fn in_values(field: &str, values: Vec<Value>) -> ScopeDefinition {
        // 复制字段名
        let field_owned = field.to_string();

        ScopeDefinition::new(&format!("{}_in", field))
            .with_callback(move |qb| {
                // 使用可变引用版本的方法添加 IN 条件
                qb.where_in_mut(&field_owned, values.clone());
            })
    }

    /// 创建 NOT IN 范围
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `values`: 值列表
    ///
    /// # 返回值
    /// 查询范围定义
    pub fn not_in_values(field: &str, values: Vec<Value>) -> ScopeDefinition {
        // 复制字段名
        let field_owned = field.to_string();

        ScopeDefinition::new(&format!("{}_not_in", field))
            .with_callback(move |qb| {
                // 使用可变引用版本的方法添加 NOT IN 条件
                qb.where_not_in_mut(&field_owned, values.clone());
            })
    }

    /// 创建 NULL 范围
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 查询范围定义
    pub fn is_null(field: &str) -> ScopeDefinition {
        // 复制字段名
        let field_owned = field.to_string();

        ScopeDefinition::new(&format!("{}_null", field))
            .with_callback(move |qb| {
                // 使用可变引用版本的方法添加 NULL 条件
                qb.where_null_mut(&field_owned);
            })
    }

    /// 创建 NOT NULL 范围
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 查询范围定义
    pub fn is_not_null(field: &str) -> ScopeDefinition {
        // 复制字段名
        let field_owned = field.to_string();

        ScopeDefinition::new(&format!("{}_not_null", field))
            .with_callback(move |qb| {
                // 使用可变引用版本的方法添加 NOT NULL 条件
                qb.where_not_null_mut(&field_owned);
            })
    }
}

/// 为 ModelConfig 添加查询范围支持
impl ModelConfig {
    /// 创建带全局范围的查询构建器
    ///
    /// # 返回值
    /// 查询构建器
    pub fn create_query_with_global_scopes(&self, scope_manager: &ScopeManager) -> QueryBuilder {
        // 创建查询构建器
        let mut qb = QueryBuilder::new(&self.full_table());

        // 应用全局查询范围
        scope_manager.apply_global_scopes(&mut qb);

        qb
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_manager() {
        // 创建范围管理器
        let mut manager = ScopeManager::new();

        // 注册查询范围（使用可变引用版本的方法）
        manager.add_scope("thinkphp", |qb| {
            qb.where_clause_mut("name", "=", Value::String("thinkphp".to_string()));
            qb.field_mut("id, name");
        });

        // 检查范围是否存在
        assert!(manager.has_scope("thinkphp"));
        assert!(!manager.has_scope("not_exists"));
    }

    #[test]
    fn test_global_scope() {
        // 创建范围管理器
        let mut manager = ScopeManager::new();

        // 注册全局范围（使用可变引用版本的方法）
        manager.add_global_scope("status", |qb| {
            qb.where_clause_mut("status", "=", Value::Int(1));
        });

        // 检查全局范围
        let global_names = manager.get_global_scope_names();
        assert!(global_names.contains(&"status"));
    }

    #[test]
    fn test_disable_global_scope() {
        // 创建范围管理器
        let mut manager = ScopeManager::new();

        // 注册全局范围（使用可变引用版本的方法）
        manager.add_global_scope("status", |qb| {
            qb.where_clause_mut("status", "=", Value::Int(1));
        });

        // 禁用全局范围
        manager.without_global_scope(&["status"]);

        // 检查禁用状态
        assert!(manager.disabled_scopes.contains(&"status".to_string()));

        // 恢复全局范围
        manager.restore_global_scope(&["status"]);
        assert!(!manager.disabled_scopes.contains(&"status".to_string()));
    }

    #[test]
    fn test_scope_builder() {
        // 创建范围构建器
        let mut builder = ScopeBuilder::new("users");

        // 注册范围（使用可变引用版本的方法）
        builder.scope_manager.add_scope("active", |qb| {
            qb.where_clause_mut("status", "=", Value::Int(1));
        });

        // 应用范围
        builder.scope("active");

        // 构建 SQL
        let sql = builder.build_select();
        // 字段名被反引号包裹（安全转义）
        assert!(sql.contains("`status` =") || sql.contains("status ="));
    }

    #[test]
    fn test_predefined_scopes() {
        // 测试状态范围
        let status_scope = PredefinedScopes::status(1);
        assert_eq!(status_scope.name, "status");

        // 测试排序范围
        let order_scope = PredefinedScopes::order_by("created_at", "desc");
        assert!(order_scope.name.contains("created_at"));

        // 测试限制范围
        let limit_scope = PredefinedScopes::limit_n(10);
        assert!(limit_scope.name.contains("10"));
    }
}
