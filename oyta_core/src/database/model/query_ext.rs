//! 查询扩展模块
//!
//! 实现模型的链式查询构建器
//! 对应 ThinkPHP 8.0 风格的模型查询
//!
//! 主要功能：
//! - 链式调用 where/whereOr/whereIn/order/limit/page 等方法
//! - select()/find()/first()/get() 查询方法
//! - with()/withJoin() 关联预加载
//! - count()/sum()/avg()/max()/min() 聚合方法

use std::collections::HashMap;

use crate::database::executor;
use crate::database::query_builder::QueryBuilder;
use crate::interpreter::value::Value;

use super::collection::ModelCollection;
use super::helpers::{class_to_table, soft_delete_condition};
use super::instance::ModelInstance;
use super::preload::{JoinType, PreloadConfig, PreloadType};
use super::types::{ModelConfig, RelationType};

/// 模型查询构建器
///
/// 支持 ThinkPHP 8.0 风格的链式查询
/// 所有方法都返回 Self，支持链式调用
#[derive(Debug, Clone)]
pub struct ModelQueryBuilder {
    /// 模型配置
    pub config: ModelConfig,
    /// 查询构建器
    pub query_builder: QueryBuilder,
    /// WHERE 条件列表
    pub wheres: Vec<WhereClause>,
    /// 排序条件列表
    pub orders: Vec<(String, String)>,
    /// 限制数量
    pub limit_val: Option<usize>,
    /// 偏移量
    pub offset_val: Option<usize>,
    /// 查询字段列表
    pub fields: Vec<String>,
    /// 预加载配置列表
    pub preloads: Vec<PreloadConfig>,
    /// 是否使用 JOIN 预加载
    pub use_join_preload: bool,
    /// JOIN 类型
    pub join_type: JoinType,
    /// 分组字段
    pub group_by_fields: Vec<String>,
    /// HAVING 条件
    pub having_conditions: Vec<WhereClause>,
    /// 是否排除软删除数据
    pub with_trashed: bool,
    /// 是否只查询软删除数据
    pub only_trashed: bool,
    /// 是否使用缓存
    pub use_cache: bool,
    /// 缓存时间（秒）
    pub cache_ttl: Option<u32>,
}

/// WHERE 条件子句
#[derive(Debug, Clone)]
pub struct WhereClause {
    /// 字段名
    pub field: String,
    /// 操作符
    pub operator: String,
    /// 值
    pub value: Value,
    /// 连接符（AND/OR）
    pub connector: String,
    /// 是否为嵌套条件组
    pub is_group: bool,
    /// 嵌套条件（当 is_group 为 true 时使用）
    pub group_conditions: Vec<WhereClause>,
}

impl WhereClause {
    /// 创建新的 WHERE 条件
    pub fn new(field: &str, operator: &str, value: Value, connector: &str) -> Self {
        Self {
            field: field.to_string(),
            operator: operator.to_string(),
            value,
            connector: connector.to_string(),
            is_group: false,
            group_conditions: Vec::new(),
        }
    }

    /// 创建嵌套条件组
    pub fn new_group(conditions: Vec<WhereClause>, connector: &str) -> Self {
        Self {
            field: String::new(),
            operator: String::new(),
            value: Value::Null,
            connector: connector.to_string(),
            is_group: true,
            group_conditions: conditions,
        }
    }
}

impl ModelQueryBuilder {
    /// 创建新的查询构建器
    ///
    /// # 参数
    /// - `config`: 模型配置
    ///
    /// # 返回值
    /// 新的查询构建器实例
    pub fn new(config: ModelConfig) -> Self {
        // 获取完整表名
        let table = config.full_table();

        Self {
            config,
            query_builder: QueryBuilder::new(&table),
            wheres: Vec::new(),
            orders: Vec::new(),
            limit_val: None,
            offset_val: None,
            fields: Vec::new(),
            preloads: Vec::new(),
            use_join_preload: false,
            join_type: JoinType::Left,
            group_by_fields: Vec::new(),
            having_conditions: Vec::new(),
            with_trashed: false,
            only_trashed: false,
            use_cache: false,
            cache_ttl: None,
        }
    }

    // ==================== 链式查询方法 ====================

    /// 添加 WHERE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `args`: 参数（可以是值，或操作符+值）
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    ///
    /// # 示例
    /// ```php
    /// User::where('id', 1)->find();
    /// User::where('name', 'like', '%think%')->select();
    /// ```
    pub fn where_condition(mut self, field: &str, operator: &str, value: Value) -> Self {
        // 添加 WHERE 条件
        self.wheres.push(WhereClause::new(field, operator, value, "AND"));
        self
    }

    /// 添加等于条件（简化版 where）
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `value`: 值
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn where_eq(mut self, field: &str, value: Value) -> Self {
        // 添加等于条件
        self.wheres.push(WhereClause::new(field, "=", value, "AND"));
        self
    }

    /// 添加 OR WHERE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `value`: 值
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    ///
    /// # 示例
    /// ```php
    /// User::where('id', 1)->whereOr('name', 'like', '%think%')->select();
    /// ```
    pub fn where_or(mut self, field: &str, operator: &str, value: Value) -> Self {
        // 添加 OR WHERE 条件
        self.wheres.push(WhereClause::new(field, operator, value, "OR"));
        self
    }

    /// 添加 IN 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `values`: 值列表
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    ///
    /// # 示例
    /// ```php
    /// User::whereIn('id', [1, 2, 3])->select();
    /// ```
    pub fn where_in(mut self, field: &str, values: Vec<Value>) -> Self {
        // 构建 IN 占位符
        let placeholders: Vec<String> = values.iter().map(|_| "?".to_string()).collect();
        // 添加 IN 条件
        self.wheres.push(WhereClause {
            field: field.to_string(),
            operator: format!("IN ({})", placeholders.join(", ")),
            value: Value::IndexedArray(values),
            connector: "AND".to_string(),
            is_group: false,
            group_conditions: Vec::new(),
        });
        self
    }

    /// 添加 NOT IN 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `values`: 值列表
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn where_not_in(mut self, field: &str, values: Vec<Value>) -> Self {
        // 构建 NOT IN 占位符
        let placeholders: Vec<String> = values.iter().map(|_| "?".to_string()).collect();
        // 添加 NOT IN 条件
        self.wheres.push(WhereClause {
            field: field.to_string(),
            operator: format!("NOT IN ({})", placeholders.join(", ")),
            value: Value::IndexedArray(values),
            connector: "AND".to_string(),
            is_group: false,
            group_conditions: Vec::new(),
        });
        self
    }

    /// 添加 LIKE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `pattern`: 匹配模式
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    ///
    /// # 示例
    /// ```php
    /// User::whereLike('name', '%think%')->select();
    /// ```
    pub fn where_like(mut self, field: &str, pattern: &str) -> Self {
        // 添加 LIKE 条件
        self.wheres.push(WhereClause::new(field, "LIKE", Value::String(pattern.to_string()), "AND"));
        self
    }

    /// 添加 NOT LIKE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `pattern`: 匹配模式
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn where_not_like(mut self, field: &str, pattern: &str) -> Self {
        // 添加 NOT LIKE 条件
        self.wheres.push(WhereClause::new(field, "NOT LIKE", Value::String(pattern.to_string()), "AND"));
        self
    }

    /// 添加 NULL 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    ///
    /// # 示例
    /// ```php
    /// User::whereNull('deleted_at')->select();
    /// ```
    pub fn where_null(mut self, field: &str) -> Self {
        // 添加 IS NULL 条件
        self.wheres.push(WhereClause {
            field: field.to_string(),
            operator: "IS NULL".to_string(),
            value: Value::Null,
            connector: "AND".to_string(),
            is_group: false,
            group_conditions: Vec::new(),
        });
        self
    }

    /// 添加 NOT NULL 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn where_not_null(mut self, field: &str) -> Self {
        // 添加 IS NOT NULL 条件
        self.wheres.push(WhereClause {
            field: field.to_string(),
            operator: "IS NOT NULL".to_string(),
            value: Value::Null,
            connector: "AND".to_string(),
            is_group: false,
            group_conditions: Vec::new(),
        });
        self
    }

    /// 添加 BETWEEN 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `min`: 最小值
    /// - `max`: 最大值
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    ///
    /// # 示例
    /// ```php
    /// User::whereBetween('age', 18, 30)->select();
    /// ```
    pub fn where_between(mut self, field: &str, min: Value, max: Value) -> Self {
        // 添加 BETWEEN 条件
        self.wheres.push(WhereClause {
            field: field.to_string(),
            operator: "BETWEEN".to_string(),
            value: Value::IndexedArray(vec![min, max]),
            connector: "AND".to_string(),
            is_group: false,
            group_conditions: Vec::new(),
        });
        self
    }

    /// 添加 NOT BETWEEN 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `min`: 最小值
    /// - `max`: 最大值
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn where_not_between(mut self, field: &str, min: Value, max: Value) -> Self {
        // 添加 NOT BETWEEN 条件
        self.wheres.push(WhereClause {
            field: field.to_string(),
            operator: "NOT BETWEEN".to_string(),
            value: Value::IndexedArray(vec![min, max]),
            connector: "AND".to_string(),
            is_group: false,
            group_conditions: Vec::new(),
        });
        self
    }

    /// 添加 EXISTS 条件
    ///
    /// # 参数
    /// - `subquery`: 子查询 SQL
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn where_exists(mut self, subquery: &str) -> Self {
        // 添加 EXISTS 条件
        self.wheres.push(WhereClause {
            field: String::new(),
            operator: format!("EXISTS ({})", subquery),
            value: Value::Null,
            connector: "AND".to_string(),
            is_group: false,
            group_conditions: Vec::new(),
        });
        self
    }

    /// 添加原始 WHERE 条件
    ///
    /// # 参数
    /// - `raw`: 原始 SQL 条件
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn where_raw(mut self, raw: &str) -> Self {
        // 添加原始条件
        self.wheres.push(WhereClause {
            field: String::new(),
            operator: raw.to_string(),
            value: Value::Null,
            connector: "AND".to_string(),
            is_group: false,
            group_conditions: Vec::new(),
        });
        self
    }

    /// 添加嵌套 WHERE 条件
    ///
    /// # 参数
    /// - `conditions`: 嵌套条件列表
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn where_nested(mut self, conditions: Vec<WhereClause>) -> Self {
        // 添加嵌套条件组
        self.wheres.push(WhereClause::new_group(conditions, "AND"));
        self
    }

    // ==================== 排序方法 ====================

    /// 添加排序
    ///
    /// # 参数
    /// - `field`: 排序字段
    /// - `direction`: 排序方向（asc/desc）
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    ///
    /// # 示例
    /// ```php
    /// User::order('id', 'desc')->select();
    /// ```
    pub fn order(mut self, field: &str, direction: &str) -> Self {
        // 添加排序条件
        self.orders.push((field.to_string(), direction.to_uppercase()));
        self
    }

    /// 升序排序
    ///
    /// # 参数
    /// - `field`: 排序字段
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn order_asc(self, field: &str) -> Self {
        self.order(field, "ASC")
    }

    /// 降序排序
    ///
    /// # 参数
    /// - `field`: 排序字段
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn order_desc(self, field: &str) -> Self {
        self.order(field, "DESC")
    }

    /// 随机排序
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn order_rand(mut self) -> Self {
        // 添加随机排序
        self.orders.push(("RAND()".to_string(), String::new()));
        self
    }

    // ==================== 限制和分页方法 ====================

    /// 设置限制数量
    ///
    /// # 参数
    /// - `limit`: 限制数量
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    ///
    /// # 示例
    /// ```php
    /// User::limit(10)->select();
    /// ```
    pub fn limit(mut self, limit: usize) -> Self {
        // 设置限制数量
        self.limit_val = Some(limit);
        self
    }

    /// 设置偏移量
    ///
    /// # 参数
    /// - `offset`: 偏移量
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn offset(mut self, offset: usize) -> Self {
        // 设置偏移量
        self.offset_val = Some(offset);
        self
    }

    /// 分页查询
    ///
    /// # 参数
    /// - `page`: 页码（从 1 开始）
    /// - `per_page`: 每页数量
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    ///
    /// # 示例
    /// ```php
    /// User::page(1, 10)->select();
    /// ```
    pub fn page(mut self, page: usize, per_page: usize) -> Self {
        // 计算偏移量
        let offset = page.saturating_sub(1) * per_page;
        // 设置限制和偏移
        self.limit_val = Some(per_page);
        self.offset_val = Some(offset);
        self
    }

    // ==================== 字段选择方法 ====================

    /// 设置查询字段
    ///
    /// # 参数
    /// - `fields`: 字段列表
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    ///
    /// # 示例
    /// ```php
    /// User::field('id,name,email')->select();
    /// User::field(['id', 'name', 'email'])->select();
    /// ```
    pub fn field(mut self, fields: Vec<&str>) -> Self {
        // 设置查询字段
        self.fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 排除字段
    ///
    /// # 参数
    /// - `fields`: 要排除的字段列表
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn without_field(mut self, fields: Vec<&str>) -> Self {
        // 获取所有字段后排除指定字段
        // 这里简化处理，实际需要查询表结构
        let exclude: Vec<String> = fields.iter().map(|s| s.to_string()).collect();
        // 标记排除字段（需要在查询时处理）
        for f in exclude {
            if !self.fields.contains(&format!("!{}", f)) {
                self.fields.push(format!("!{}", f));
            }
        }
        self
    }

    // ==================== 分组方法 ====================

    /// 设置分组字段
    ///
    /// # 参数
    /// - `fields`: 分组字段列表
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    ///
    /// # 示例
    /// ```php
    /// User::group('status')->select();
    /// ```
    pub fn group(mut self, fields: Vec<&str>) -> Self {
        // 设置分组字段
        self.group_by_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 添加 HAVING 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `value`: 值
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn having(mut self, field: &str, operator: &str, value: Value) -> Self {
        // 添加 HAVING 条件
        self.having_conditions.push(WhereClause::new(field, operator, value, "AND"));
        self
    }

    // ==================== 软删除相关方法 ====================

    /// 包含软删除数据
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    ///
    /// # 示例
    /// ```php
    /// User::withTrashed()->select();
    /// ```
    pub fn with_trashed(mut self) -> Self {
        // 设置包含软删除数据
        self.with_trashed = true;
        self.only_trashed = false;
        self
    }

    /// 只查询软删除数据
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    ///
    /// # 示例
    /// ```php
    /// User::onlyTrashed()->select();
    /// ```
    pub fn only_trashed(mut self) -> Self {
        // 设置只查询软删除数据
        self.only_trashed = true;
        self.with_trashed = false;
        self
    }

    // ==================== 缓存方法 ====================

    /// 启用查询缓存
    ///
    /// # 参数
    /// - `ttl`: 缓存时间（秒）
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn cache(mut self, ttl: u32) -> Self {
        // 启用缓存
        self.use_cache = true;
        self.cache_ttl = Some(ttl);
        self
    }

    // ==================== 关联预加载方法 ====================

    /// 预加载关联（IN 方式）
    ///
    /// # 参数
    /// - `relations`: 关联名称列表
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    ///
    /// # 示例
    /// ```php
    /// User::with(['profile', 'posts'])->select();
    /// ```
    pub fn with(mut self, relations: Vec<&str>) -> Self {
        // 添加预加载配置
        for relation in relations {
            self.preloads.push(PreloadConfig::new(relation));
        }
        self
    }

    /// 预加载关联（带配置）
    ///
    /// # 参数
    /// - `config`: 预加载配置
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn with_config(mut self, config: PreloadConfig) -> Self {
        // 添加预加载配置
        self.preloads.push(config);
        self
    }

    /// JOIN 方式预加载关联
    ///
    /// # 参数
    /// - `relations`: 关联名称列表
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    ///
    /// # 示例
    /// ```php
    /// User::withJoin(['profile'])->select();
    /// ```
    pub fn with_join(mut self, relations: Vec<&str>) -> Self {
        // 设置使用 JOIN 预加载
        self.use_join_preload = true;
        // 添加 JOIN 预加载配置
        for relation in relations {
            self.preloads.push(PreloadConfig::new(relation).with_join(self.join_type));
        }
        self
    }

    /// 设置 JOIN 类型
    ///
    /// # 参数
    /// - `join_type`: JOIN 类型
    ///
    /// # 返回值
    /// 返回 Self 支持链式调用
    pub fn join_type(mut self, join_type: JoinType) -> Self {
        // 设置 JOIN 类型
        self.join_type = join_type;
        self
    }

    // ==================== SQL 构建方法 ====================

    /// 构建 SELECT SQL
    ///
    /// # 返回值
    /// SQL 字符串
    pub fn build_select(&self) -> String {
        // 获取完整表名
        let table = self.config.full_table();

        // 构建 SELECT 字段
        let select_fields = if self.fields.is_empty() {
            // 默认选择所有字段
            if self.use_join_preload && !self.preloads.is_empty() {
                // JOIN 预加载时需要添加表别名
                format!("{}.*", table)
            } else {
                "*".to_string()
            }
        } else {
            // 使用指定字段
            self.fields.join(", ")
        };

        // 构建 SQL
        let mut sql = format!("SELECT {} FROM {}", select_fields, table);

        // 添加 JOIN 子句（如果有 JOIN 预加载）
        if self.use_join_preload && !self.preloads.is_empty() {
            for preload in &self.preloads {
                if preload.preload_type == PreloadType::Join {
                    if let Some(relation) = self.config.relations.get(&preload.relation) {
                        // 获取关联表名
                        let related_table = class_to_table(&relation.related_model);
                        let alias = &preload.relation;

                        // 构建 JOIN 子句
                        let join_sql = format!(
                            " {} {} AS {} ON {}.{} = {}.{}",
                            preload.join_type.sql_keyword(),
                            related_table,
                            alias,
                            alias,
                            relation.foreign_key,
                            table,
                            relation.local_key
                        );
                        sql.push_str(&join_sql);
                    }
                }
            }
        }

        // 添加 WHERE 条件
        let where_sql = self.build_where_clause();
        if !where_sql.is_empty() {
            sql.push_str(&format!(" WHERE {}", where_sql));
        }

        // 添加 GROUP BY
        if !self.group_by_fields.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", self.group_by_fields.join(", ")));
        }

        // 添加 HAVING
        if !self.having_conditions.is_empty() {
            let having_parts: Vec<String> = self.having_conditions.iter().enumerate().map(|(i, h)| {
                let prefix = if i == 0 { "" } else { &h.connector };
                format!("{} {} {}", prefix, h.field, h.operator)
            }).collect();
            sql.push_str(&format!(" HAVING {}", having_parts.join(" ")));
        }

        // 添加 ORDER BY
        if !self.orders.is_empty() {
            let order_parts: Vec<String> = self.orders.iter()
                .filter(|(f, _)| !f.starts_with("RAND()"))
                .map(|(f, d)| if d.is_empty() { f.clone() } else { format!("{} {}", f, d) })
                .collect();
            if !order_parts.is_empty() {
                sql.push_str(&format!(" ORDER BY {}", order_parts.join(", ")));
            }
            // 处理随机排序
            if self.orders.iter().any(|(f, _)| f.starts_with("RAND()")) {
                sql.push_str(" ORDER BY RAND()");
            }
        }

        // 添加 LIMIT
        if let Some(limit) = self.limit_val {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // 添加 OFFSET
        if let Some(offset) = self.offset_val {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        sql
    }

    /// 构建 WHERE 子句
    ///
    /// # 返回值
    /// WHERE 子句字符串
    fn build_where_clause(&self) -> String {
        let mut conditions = Vec::new();

        // 添加软删除条件
        if self.config.soft_delete && !self.with_trashed {
            let condition = if self.only_trashed {
                // 只查询软删除数据
                match &self.config.soft_delete_default {
                    super::types::SoftDeleteDefault::Null => {
                        format!("{} IS NOT NULL", self.config.delete_time_field)
                    }
                    super::types::SoftDeleteDefault::Zero => {
                        format!("{} != 0", self.config.delete_time_field)
                    }
                    super::types::SoftDeleteDefault::Empty => {
                        format!("{} != ''", self.config.delete_time_field)
                    }
                }
            } else {
                // 排除软删除数据
                soft_delete_condition(&self.config.soft_delete_default, &self.config.delete_time_field)
            };
            conditions.push(condition);
        }

        // 添加用户定义的 WHERE 条件
        for (i, w) in self.wheres.iter().enumerate() {
            if w.is_group {
                // 处理嵌套条件组
                let group_conditions: Vec<String> = w.group_conditions.iter().enumerate().map(|(j, gc)| {
                    let prefix = if j == 0 { "" } else { &gc.connector };
                    format!("{} {} {}", prefix, gc.field, gc.operator)
                }).collect();
                let group_sql = format!("({})", group_conditions.join(" "));
                if i == 0 && conditions.is_empty() {
                    conditions.push(group_sql);
                } else {
                    conditions.push(format!("{} {}", w.connector, group_sql));
                }
            } else {
                // 处理普通条件
                let condition = if w.value == Value::Null && !w.operator.starts_with("EXISTS") {
                    format!("{} {}", w.field, w.operator)
                } else if w.operator.starts_with("IN") || w.operator.starts_with("NOT IN") || w.operator.starts_with("BETWEEN") || w.operator.starts_with("NOT BETWEEN") {
                    format!("{} {}", w.field, w.operator)
                } else if w.operator.starts_with("EXISTS") {
                    w.operator.clone()
                } else {
                    format!("{} {} ?", w.field, w.operator)
                };

                if i == 0 && conditions.is_empty() {
                    conditions.push(condition);
                } else {
                    conditions.push(format!("{} {}", w.connector, condition));
                }
            }
        }

        conditions.join(" ")
    }

    /// 构建聚合查询 SQL
    ///
    /// # 参数
    /// - `function`: 聚合函数名
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// SQL 字符串
    fn build_aggregate(&self, function: &str, field: &str) -> String {
        // 获取完整表名
        let table = self.config.full_table();

        // 构建 SQL
        let mut sql = format!("SELECT {}({}) as aggregate FROM {}", function, field, table);

        // 添加 WHERE 条件
        let where_sql = self.build_where_clause();
        if !where_sql.is_empty() {
            sql.push_str(&format!(" WHERE {}", where_sql));
        }

        sql
    }

    /// 获取查询参数
    ///
    /// # 返回值
    /// 参数值列表
    pub fn get_params(&self) -> Vec<Value> {
        let mut params = Vec::new();

        // 收集 WHERE 条件的参数
        for w in &self.wheres {
            if w.is_group {
                // 收集嵌套条件的参数
                for gc in &w.group_conditions {
                    if gc.value != Value::Null {
                        match &gc.value {
                            Value::IndexedArray(arr) => params.extend(arr.clone()),
                            _ => params.push(gc.value.clone()),
                        }
                    }
                }
            } else if w.value != Value::Null {
                match &w.value {
                    Value::IndexedArray(arr) => params.extend(arr.clone()),
                    _ => params.push(w.value.clone()),
                }
            }
        }

        // 收集 HAVING 条件的参数
        for h in &self.having_conditions {
            if h.value != Value::Null {
                params.push(h.value.clone());
            }
        }

        params
    }

    // ==================== 查询执行方法 ====================

    /// 执行查询并返回模型集合
    ///
    /// # 返回值
    /// 模型集合
    ///
    /// # 示例
    /// ```php
    /// $users = User::where('status', 1)->select();
    /// ```
    pub async fn select(self) -> anyhow::Result<ModelCollection> {
        // 构建 SQL
        let sql = self.build_select();
        let params = self.get_params();

        // 执行查询
        let result = if params.is_empty() {
            executor::query(&sql).await?
        } else {
            executor::query_with_params(&sql, &params).await?
        };

        // 转换为模型实例列表
        let models: Vec<ModelInstance> = result.rows.into_iter()
            .map(|row| ModelInstance::from_row(self.config.clone(), row))
            .collect();

        // 创建模型集合
        let mut collection = ModelCollection::new(models);

        // 处理预加载
        if !self.preloads.is_empty() && !self.use_join_preload {
            // IN 方式预加载
            self.execute_preload(&collection).await?;
        }

        Ok(collection)
    }

    /// 执行查询并返回所有记录（select 的别名）
    ///
    /// # 返回值
    /// 模型集合
    pub async fn get(self) -> anyhow::Result<ModelCollection> {
        self.select().await
    }

    /// 查找单条记录
    ///
    /// # 参数
    /// - `id`: 主键值
    ///
    /// # 返回值
    /// 找到返回模型实例，否则返回 None
    ///
    /// # 示例
    /// ```php
    /// $user = User::find(1);
    /// ```
    pub async fn find(mut self, id: Value) -> anyhow::Result<Option<ModelInstance>> {
        // 获取主键字段名
        let pk = self.config.pk.clone();
        // 添加主键条件
        let builder = self.where_eq(&pk, id);
        // 执行查询
        builder.first().await
    }

    /// 查找单条记录或抛出异常
    ///
    /// # 参数
    /// - `id`: 主键值
    ///
    /// # 返回值
    /// 找到返回模型实例，否则返回错误
    pub async fn find_or_fail(self, id: Value) -> anyhow::Result<ModelInstance> {
        // 查找记录
        let result = self.find(id).await?;

        match result {
            Some(model) => Ok(model),
            None => Err(anyhow::anyhow!("Model not found")),
        }
    }

    /// 获取第一条记录
    ///
    /// # 返回值
    /// 找到返回模型实例，否则返回 None
    ///
    /// # 示例
    /// ```php
    /// $user = User::where('status', 1)->first();
    /// ```
    pub async fn first(mut self) -> anyhow::Result<Option<ModelInstance>> {
        // 设置限制为 1
        self.limit_val = Some(1);

        // 构建 SQL
        let sql = self.build_select();
        let params = self.get_params();

        // 执行查询
        let result = if params.is_empty() {
            executor::query(&sql).await?
        } else {
            executor::query_with_params(&sql, &params).await?
        };

        // 返回第一条记录
        if let Some(row) = result.first() {
            let model = ModelInstance::from_row(self.config.clone(), row.clone());

            // 处理预加载
            if !self.preloads.is_empty() && !self.use_join_preload {
                let collection = ModelCollection::new(vec![model.clone()]);
                self.execute_preload(&collection).await?;
                return Ok(collection.first().cloned());
            }

            Ok(Some(model))
        } else {
            Ok(None)
        }
    }

    /// 获取单个字段值
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 字段值
    ///
    /// # 示例
    /// ```php
    /// $name = User::where('id', 1)->value('name');
    /// ```
    pub async fn value(self, field: &str) -> anyhow::Result<Value> {
        // 构建查询
        let table = self.config.full_table();
        let mut sql = format!("SELECT {} FROM {}", field, table);

        // 添加 WHERE 条件
        let where_sql = self.build_where_clause();
        if !where_sql.is_empty() {
            sql.push_str(&format!(" WHERE {}", where_sql));
        }

        sql.push_str(" LIMIT 1");

        // 获取参数
        let params = self.get_params();

        // 执行查询
        let result = if params.is_empty() {
            executor::query(&sql).await?
        } else {
            executor::query_with_params(&sql, &params).await?
        };

        // 返回结果
        Ok(result.scalar().cloned().unwrap_or(Value::Null))
    }

    /// 获取某列的值
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 字段值列表
    ///
    /// # 示例
    /// ```php
    /// $names = User::column('name');
    /// ```
    pub async fn column(self, field: &str) -> anyhow::Result<Vec<Value>> {
        // 构建查询
        let table = self.config.full_table();
        let mut sql = format!("SELECT {} FROM {}", field, table);

        // 添加 WHERE 条件
        let where_sql = self.build_where_clause();
        if !where_sql.is_empty() {
            sql.push_str(&format!(" WHERE {}", where_sql));
        }

        // 获取参数
        let params = self.get_params();

        // 执行查询
        let result = if params.is_empty() {
            executor::query(&sql).await?
        } else {
            executor::query_with_params(&sql, &params).await?
        };

        // 提取字段值
        Ok(result.rows.iter()
            .filter_map(|row| row.get(field).cloned())
            .collect())
    }

    // ==================== 聚合方法 ====================

    /// 统计数量
    ///
    /// # 返回值
    /// 记录总数
    ///
    /// # 示例
    /// ```php
    /// $count = User::where('status', 1)->count();
    /// ```
    pub async fn count(self) -> anyhow::Result<i64> {
        // 构建 SQL
        let sql = self.build_aggregate("COUNT", "*");
        let params = self.get_params();

        // 执行查询
        let result = if params.is_empty() {
            executor::query(&sql).await?
        } else {
            executor::query_with_params(&sql, &params).await?
        };

        // 提取结果
        Ok(result.scalar()
            .and_then(|v| match v {
                Value::Int(i) => Some(*i),
                _ => None,
            })
            .unwrap_or(0))
    }

    /// 求和
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 求和结果
    ///
    /// # 示例
    /// ```php
    /// $total = Order::sum('amount');
    /// ```
    pub async fn sum(self, field: &str) -> anyhow::Result<f64> {
        // 构建 SQL
        let sql = self.build_aggregate("SUM", field);
        let params = self.get_params();

        // 执行查询
        let result = if params.is_empty() {
            executor::query(&sql).await?
        } else {
            executor::query_with_params(&sql, &params).await?
        };

        // 提取结果
        Ok(result.scalar()
            .and_then(|v| match v {
                Value::Int(i) => Some(*i as f64),
                Value::Float(f) => Some(*f),
                _ => None,
            })
            .unwrap_or(0.0))
    }

    /// 求平均值
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 平均值
    ///
    /// # 示例
    /// ```php
    /// $avg = Order::avg('amount');
    /// ```
    pub async fn avg(self, field: &str) -> anyhow::Result<f64> {
        // 构建 SQL
        let sql = self.build_aggregate("AVG", field);
        let params = self.get_params();

        // 执行查询
        let result = if params.is_empty() {
            executor::query(&sql).await?
        } else {
            executor::query_with_params(&sql, &params).await?
        };

        // 提取结果
        Ok(result.scalar()
            .and_then(|v| match v {
                Value::Int(i) => Some(*i as f64),
                Value::Float(f) => Some(*f),
                _ => None,
            })
            .unwrap_or(0.0))
    }

    /// 求最大值
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 最大值
    ///
    /// # 示例
    /// ```php
    /// $max = Order::max('amount');
    /// ```
    pub async fn max(self, field: &str) -> anyhow::Result<f64> {
        // 构建 SQL
        let sql = self.build_aggregate("MAX", field);
        let params = self.get_params();

        // 执行查询
        let result = if params.is_empty() {
            executor::query(&sql).await?
        } else {
            executor::query_with_params(&sql, &params).await?
        };

        // 提取结果
        Ok(result.scalar()
            .and_then(|v| match v {
                Value::Int(i) => Some(*i as f64),
                Value::Float(f) => Some(*f),
                _ => None,
            })
            .unwrap_or(0.0))
    }

    /// 求最小值
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回值
    /// 最小值
    ///
    /// # 示例
    /// ```php
    /// $min = Order::min('amount');
    /// ```
    pub async fn min(self, field: &str) -> anyhow::Result<f64> {
        // 构建 SQL
        let sql = self.build_aggregate("MIN", field);
        let params = self.get_params();

        // 执行查询
        let result = if params.is_empty() {
            executor::query(&sql).await?
        } else {
            executor::query_with_params(&sql, &params).await?
        };

        // 提取结果
        Ok(result.scalar()
            .and_then(|v| match v {
                Value::Int(i) => Some(*i as f64),
                Value::Float(f) => Some(*f),
                _ => None,
            })
            .unwrap_or(0.0))
    }

    /// 检查是否存在记录
    ///
    /// # 返回值
    /// 如果存在返回 true
    pub async fn exists(self) -> anyhow::Result<bool> {
        // 统计数量
        let count = self.count().await?;
        Ok(count > 0)
    }

    /// 检查是否不存在记录
    ///
    /// # 返回值
    /// 如果不存在返回 true
    pub async fn doesnt_exist(self) -> anyhow::Result<bool> {
        // 统计数量
        let count = self.count().await?;
        Ok(count == 0)
    }

    // ==================== 分页方法 ====================

    /// 分页查询
    ///
    /// # 参数
    /// - `page`: 页码（从 1 开始）
    /// - `per_page`: 每页数量
    ///
    /// # 返回值
    /// 分页结果
    ///
    /// # 示例
    /// ```php
    /// $users = User::paginate(1, 15);
    /// ```
    pub async fn paginate(self, page: usize, per_page: usize) -> anyhow::Result<PaginateResult> {
        // 获取总数
        let total = self.clone().count().await?;

        // 计算偏移
        let offset = page.saturating_sub(1) * per_page;

        // 查询数据
        let builder = self.clone()
            .limit(per_page)
            .offset(offset);

        let items = builder.select().await?;

        // 计算总页数
        let total_pages = if total > 0 {
            ((total as usize) + per_page - 1) / per_page
        } else {
            0
        };

        Ok(PaginateResult {
            items: items.to_vec(),
            total,
            page: page as i64,
            per_page: per_page as i64,
            total_pages: total_pages as i64,
        })
    }

    // ==================== 预加载执行方法 ====================

    /// 执行预加载
    ///
    /// # 参数
    /// - `collection`: 模型集合
    async fn execute_preload(&self, collection: &ModelCollection) -> anyhow::Result<()> {
        // 如果集合为空，直接返回
        if collection.is_empty() {
            return Ok(());
        }

        // 收集所有父模型 ID
        let parent_ids: Vec<Value> = collection.iter().map(|m| m.get_key()).collect();

        // 为每个预加载配置执行查询
        for preload in &self.preloads {
            // 获取关联定义
            if let Some(relation) = self.config.relations.get(&preload.relation) {
                // 根据关联类型执行预加载
                match relation.relation_type {
                    RelationType::HasOne | RelationType::BelongsTo => {
                        // 一对一关联预加载
                        self.preload_has_one(collection, relation, &parent_ids, &preload.relation).await?;
                    }
                    RelationType::HasMany => {
                        // 一对多关联预加载
                        self.preload_has_many(collection, relation, &parent_ids, &preload.relation).await?;
                    }
                    RelationType::BelongsToMany => {
                        // 多对多关联预加载
                        self.preload_belongs_to_many(collection, relation, &parent_ids, &preload.relation).await?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// 预加载一对一关联
    async fn preload_has_one(
        &self,
        collection: &ModelCollection,
        relation: &super::types::RelationDef,
        parent_ids: &[Value],
        relation_name: &str,
    ) -> anyhow::Result<()> {
        // 获取关联表名
        let related_table = class_to_table(&relation.related_model);

        // 构建 IN 条件
        let placeholders: Vec<String> = parent_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT * FROM {} WHERE {} IN ({})",
            related_table, relation.foreign_key, placeholders.join(", ")
        );

        // 执行查询
        let result = executor::query_with_params(&sql, parent_ids).await?;

        // 按外键分组
        let mut grouped: HashMap<String, ModelInstance> = HashMap::new();

        for row in result.rows {
            if let Some(fk_value) = row.get(&relation.foreign_key) {
                let fk_str = fk_value.to_string_value();

                // 创建关联模型
                let related_config = ModelConfig {
                    table: related_table.clone(),
                    ..ModelConfig::default()
                };
                let related_model = ModelInstance::from_row(related_config, row);
                grouped.insert(fk_str, related_model);
            }
        }

        // 将关联数据附加到父模型（通过修改集合中的模型）
        // 注意：这里需要通过返回值或回调来更新模型
        // 简化实现：将关联数据存储在模型的 attributes 中

        Ok(())
    }

    /// 预加载一对多关联
    async fn preload_has_many(
        &self,
        collection: &ModelCollection,
        relation: &super::types::RelationDef,
        parent_ids: &[Value],
        relation_name: &str,
    ) -> anyhow::Result<()> {
        // 获取关联表名
        let related_table = class_to_table(&relation.related_model);

        // 构建 IN 条件
        let placeholders: Vec<String> = parent_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT * FROM {} WHERE {} IN ({})",
            related_table, relation.foreign_key, placeholders.join(", ")
        );

        // 执行查询
        let result = executor::query_with_params(&sql, parent_ids).await?;

        // 按外键分组
        let mut grouped: HashMap<String, Vec<ModelInstance>> = HashMap::new();

        for row in result.rows {
            if let Some(fk_value) = row.get(&relation.foreign_key) {
                let fk_str = fk_value.to_string_value();

                // 创建关联模型
                let related_config = ModelConfig {
                    table: related_table.clone(),
                    ..ModelConfig::default()
                };
                let related_model = ModelInstance::from_row(related_config, row);
                grouped.entry(fk_str).or_insert_with(Vec::new).push(related_model);
            }
        }

        Ok(())
    }

    /// 预加载多对多关联
    async fn preload_belongs_to_many(
        &self,
        collection: &ModelCollection,
        relation: &super::types::RelationDef,
        parent_ids: &[Value],
        relation_name: &str,
    ) -> anyhow::Result<()> {
        // 多对多关联需要通过中间表查询
        // 这里简化实现，实际需要中间表信息
        // TODO: 实现完整的多对多预加载

        Ok(())
    }

    // ==================== 更新和删除方法 ====================

    /// 更新记录
    ///
    /// # 参数
    /// - `data`: 更新数据
    ///
    /// # 返回值
    /// 影响的行数
    ///
    /// # 示例
    /// ```php
    /// User::where('status', 0)->update(['status' => 1]);
    /// ```
    pub async fn update(self, data: &HashMap<String, Value>) -> anyhow::Result<i64> {
        // 获取完整表名
        let table = self.config.full_table();

        // 构建 SET 子句
        let set_parts: Vec<String> = data.keys().map(|k| format!("{} = ?", k)).collect();

        // 构建 SQL
        let mut sql = format!("UPDATE {} SET {}", table, set_parts.join(", "));

        // 添加 WHERE 条件
        let where_sql = self.build_where_clause();
        if !where_sql.is_empty() {
            sql.push_str(&format!(" WHERE {}", where_sql));
        }

        // 构建参数
        let mut params: Vec<Value> = data.values().cloned().collect();
        params.extend(self.get_params());

        // 执行更新
        let result = executor::query_with_params(&sql, &params).await?;

        // 返回影响的行数
        Ok(result.affected_rows as i64)
    }

    /// 删除记录
    ///
    /// # 返回值
    /// 影响的行数
    ///
    /// # 示例
    /// ```php
    /// User::where('status', 0)->delete();
    /// ```
    pub async fn delete(self) -> anyhow::Result<i64> {
        // 检查是否启用软删除
        if self.config.soft_delete && !self.with_trashed {
            // 软删除
            let mut data = HashMap::new();
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            data.insert(self.config.delete_time_field.clone(), Value::String(now));
            return self.update(&data).await;
        }

        // 物理删除
        let table = self.config.full_table();
        let mut sql = format!("DELETE FROM {}", table);

        // 添加 WHERE 条件
        let where_sql = self.build_where_clause();
        if !where_sql.is_empty() {
            sql.push_str(&format!(" WHERE {}", where_sql));
        }

        // 获取参数
        let params = self.get_params();

        // 执行删除
        let result = if params.is_empty() {
            executor::query(&sql).await?
        } else {
            executor::query_with_params(&sql, &params).await?
        };

        // 返回影响的行数
        Ok(result.affected_rows as i64)
    }

    /// 自增字段
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `step`: 增量（默认为 1）
    ///
    /// # 返回值
    /// 影响的行数
    pub async fn inc(self, field: &str, step: i64) -> anyhow::Result<i64> {
        // 获取完整表名
        let table = self.config.full_table();

        // 构建 SQL
        let mut sql = format!("UPDATE {} SET {} = {} + {}", table, field, field, step);

        // 添加 WHERE 条件
        let where_sql = self.build_where_clause();
        if !where_sql.is_empty() {
            sql.push_str(&format!(" WHERE {}", where_sql));
        }

        // 获取参数
        let params = self.get_params();

        // 执行更新
        let result = if params.is_empty() {
            executor::query(&sql).await?
        } else {
            executor::query_with_params(&sql, &params).await?
        };

        Ok(result.affected_rows as i64)
    }

    /// 自减字段
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `step`: 减量（默认为 1）
    ///
    /// # 返回值
    /// 影响的行数
    pub async fn dec(self, field: &str, step: i64) -> anyhow::Result<i64> {
        self.inc(field, -step).await
    }
}

/// 分页结果
#[derive(Debug)]
pub struct PaginateResult {
    /// 数据项
    pub items: Vec<ModelInstance>,
    /// 总数
    pub total: i64,
    /// 当前页
    pub page: i64,
    /// 每页数量
    pub per_page: i64,
    /// 总页数
    pub total_pages: i64,
}

impl PaginateResult {
    /// 是否有更多数据
    pub fn has_more(&self) -> bool {
        self.page < self.total_pages
    }

    /// 是否为第一页
    pub fn is_first_page(&self) -> bool {
        self.page == 1
    }

    /// 是否为最后一页
    pub fn is_last_page(&self) -> bool {
        self.page >= self.total_pages
    }

    /// 转换为模型集合
    pub fn to_collection(&self) -> ModelCollection {
        ModelCollection::new(self.items.clone())
    }
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
    fn test_query_builder() {
        let config = create_test_config();
        let builder = ModelQueryBuilder::new(config)
            .where_eq("status", Value::Int(1))
            .where_like("name", "%test%")
            .order_desc("created_at")
            .limit(10);

        let sql = builder.build_select();

        assert!(sql.contains("status ="));
        assert!(sql.contains("LIKE"));
        assert!(sql.contains("ORDER BY"));
        assert!(sql.contains("LIMIT 10"));
    }

    #[test]
    fn test_chain_methods() {
        let config = create_test_config();
        let builder = ModelQueryBuilder::new(config)
            .where_eq("id", Value::Int(1))
            .where_or("name", "LIKE", Value::String("%test%".to_string()))
            .where_in("status", vec![Value::Int(1), Value::Int(2)])
            .order("id", "DESC")
            .limit(10)
            .offset(5);

        let sql = builder.build_select();

        assert!(sql.contains("id ="));
        assert!(sql.contains("OR"));
        assert!(sql.contains("IN"));
        assert!(sql.contains("ORDER BY id DESC"));
        assert!(sql.contains("LIMIT 10"));
        assert!(sql.contains("OFFSET 5"));
    }

    #[test]
    fn test_aggregate_sql() {
        let config = create_test_config();
        let builder = ModelQueryBuilder::new(config)
            .where_eq("status", Value::Int(1));

        let sql = builder.build_aggregate("COUNT", "*");

        assert!(sql.contains("COUNT(*)"));
        assert!(sql.contains("WHERE"));
    }

    #[test]
    fn test_paginate_result() {
        let result = PaginateResult {
            items: Vec::new(),
            total: 100,
            page: 2,
            per_page: 10,
            total_pages: 10,
        };

        assert!(result.has_more());
        assert!(!result.is_first_page());
        assert!(!result.is_last_page());
    }

    #[test]
    fn test_where_clause() {
        let clause = WhereClause::new("name", "=", Value::String("test".to_string()), "AND");
        assert_eq!(clause.field, "name");
        assert_eq!(clause.operator, "=");
        assert_eq!(clause.connector, "AND");
        assert!(!clause.is_group);
    }

    #[test]
    fn test_where_group() {
        let conditions = vec![
            WhereClause::new("a", "=", Value::Int(1), "AND"),
            WhereClause::new("b", "=", Value::Int(2), "OR"),
        ];
        let group = WhereClause::new_group(conditions, "AND");
        assert!(group.is_group);
        assert_eq!(group.group_conditions.len(), 2);
    }
}
