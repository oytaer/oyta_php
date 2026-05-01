//! WHERE 条件扩展模块
//!
//! 提供丰富的 WHERE 条件构建方法
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 自动处理不同数据库的语法差异

use crate::interpreter::value::Value;
use std::collections::HashMap;

/// WHERE 条件类型枚举
///
/// 定义所有支持的 WHERE 条件类型
#[derive(Debug, Clone, PartialEq)]
pub enum WhereType {
    /// 基本 WHERE 条件
    Basic,
    /// IS NULL 条件
    Null,
    /// IS NOT NULL 条件
    NotNull,
    /// IN 条件
    In,
    /// NOT IN 条件
    NotIn,
    /// BETWEEN 条件
    Between,
    /// NOT BETWEEN 条件
    NotBetween,
    /// LIKE 条件
    Like,
    /// NOT LIKE 条件
    NotLike,
    /// EXISTS 子查询
    Exists,
    /// NOT EXISTS 子查询
    NotExists,
    /// 原始 SQL 条件
    Raw,
    /// 字段比较条件
    Column,
    /// JSON 包含条件
    JsonContains,
    /// JSON 长度条件
    JsonLength,
    /// 日期条件
    Date,
    /// 时间条件
    Time,
    /// 年份条件
    Year,
    /// 月份条件
    Month,
    /// 日期条件
    Day,
}

/// WHERE 条件子句结构体
///
/// 存储单个 WHERE 条件的完整信息
#[derive(Debug, Clone)]
pub struct WhereClause {
    /// 条件类型
    pub where_type: WhereType,
    /// 字段名
    pub field: String,
    /// 操作符（=, >, <, >=, <=, <>, LIKE 等）
    pub operator: String,
    /// 条件值
    pub value: WhereValue,
    /// 逻辑连接符（AND / OR）
    pub connector: String,
    /// 是否为否定条件
    pub is_not: bool,
}

impl WhereClause {
    /// 创建新的 WHERE 条件子句
    ///
    /// # 参数
    /// - `where_type`: 条件类型
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `value`: 条件值
    /// - `connector`: 逻辑连接符
    ///
    /// # 返回
    /// 新的 WHERE 条件子句实例
    pub fn new(
        where_type: WhereType,
        field: &str,
        operator: &str,
        value: WhereValue,
        connector: &str,
    ) -> Self {
        Self {
            where_type,
            field: field.to_string(),
            operator: operator.to_string(),
            value,
            connector: connector.to_string(),
            is_not: false,
        }
    }

    /// 设置为否定条件
    ///
    /// # 返回
    /// 修改后的 WHERE 条件子句
    pub fn not(mut self) -> Self {
        self.is_not = true;
        self
    }

    /// 设置逻辑连接符为 OR
    ///
    /// # 返回
    /// 修改后的 WHERE 条件子句
    pub fn or(mut self) -> Self {
        self.connector = "OR".to_string();
        self
    }
}

/// WHERE 条件值枚举
///
/// 支持多种类型的条件值
#[derive(Debug, Clone)]
pub enum WhereValue {
    /// 单个值
    Single(Value),
    /// 多个值（用于 IN 操作符）
    Multiple(Vec<Value>),
    /// 两个值（用于 BETWEEN 操作符）
    Between(Value, Value),
    /// 原始 SQL 表达式
    Raw(String),
    /// 子查询
    SubQuery(String),
    /// 字段名（用于字段比较）
    Column(String),
    /// JSON 值
    Json(serde_json::Value),
    /// 无值（用于 IS NULL 等）
    None,
}

/// WHERE 条件构建器
///
/// 提供链式调用的 WHERE 条件构建方法
#[derive(Debug, Clone)]
pub struct WhereBuilder {
    /// WHERE 条件列表
    pub wheres: Vec<WhereClause>,
    /// 绑定参数列表
    pub bindings: Vec<Value>,
    /// 数据库类型
    pub db_type: DatabaseType,
}

/// 数据库类型枚举
///
/// 支持的数据库类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DatabaseType {
    /// MySQL 数据库
    MySQL,
    /// PostgreSQL 数据库
    PostgreSQL,
    /// SQLite 数据库
    SQLite,
}

impl WhereBuilder {
    /// 创建新的 WHERE 条件构建器
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的 WHERE 条件构建器实例
    pub fn new(db_type: DatabaseType) -> Self {
        Self {
            wheres: Vec::new(),
            bindings: Vec::new(),
            db_type,
        }
    }

    /// 添加基本 WHERE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `value`: 条件值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    ///
    /// # 示例
    /// ```ignore
    /// builder.where_clause("id", "=", Value::Int(1));
    /// ```
    pub fn where_clause(mut self, field: &str, operator: &str, value: Value) -> Self {
        self.wheres.push(WhereClause::new(
            WhereType::Basic,
            field,
            operator,
            WhereValue::Single(value.clone()),
            "AND",
        ));
        self.bindings.push(value);
        self
    }

    /// 添加 OR WHERE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `value`: 条件值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn or_where(mut self, field: &str, operator: &str, value: Value) -> Self {
        self.wheres.push(WhereClause::new(
            WhereType::Basic,
            field,
            operator,
            WhereValue::Single(value.clone()),
            "OR",
        ));
        self.bindings.push(value);
        self
    }

    /// 添加 WHERE IS NULL 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    ///
    /// # 示例
    /// ```ignore
    /// builder.where_null("deleted_at");
    /// ```
    pub fn where_null(mut self, field: &str) -> Self {
        self.wheres.push(WhereClause::new(
            WhereType::Null,
            field,
            "IS NULL",
            WhereValue::None,
            "AND",
        ));
        self
    }

    /// 添加 WHERE IS NOT NULL 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn where_not_null(mut self, field: &str) -> Self {
        self.wheres.push(WhereClause::new(
            WhereType::NotNull,
            field,
            "IS NOT NULL",
            WhereValue::None,
            "AND",
        ));
        self
    }

    /// 添加 WHERE IN 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `values`: 值列表
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    ///
    /// # 示例
    /// ```ignore
    /// builder.where_in("id", vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    /// ```
    pub fn where_in(mut self, field: &str, values: Vec<Value>) -> Self {
        self.bindings.extend(values.clone());
        self.wheres.push(WhereClause::new(
            WhereType::In,
            field,
            "IN",
            WhereValue::Multiple(values),
            "AND",
        ));
        self
    }

    /// 添加 WHERE NOT IN 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `values`: 值列表
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn where_not_in(mut self, field: &str, values: Vec<Value>) -> Self {
        self.bindings.extend(values.clone());
        self.wheres.push(WhereClause::new(
            WhereType::NotIn,
            field,
            "NOT IN",
            WhereValue::Multiple(values),
            "AND",
        ));
        self
    }

    /// 添加 WHERE BETWEEN 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `start`: 起始值
    /// - `end`: 结束值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    ///
    /// # 示例
    /// ```ignore
    /// builder.where_between("age", Value::Int(18), Value::Int(30));
    /// ```
    pub fn where_between(mut self, field: &str, start: Value, end: Value) -> Self {
        self.bindings.push(start.clone());
        self.bindings.push(end.clone());
        self.wheres.push(WhereClause::new(
            WhereType::Between,
            field,
            "BETWEEN",
            WhereValue::Between(start, end),
            "AND",
        ));
        self
    }

    /// 添加 WHERE NOT BETWEEN 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `start`: 起始值
    /// - `end`: 结束值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn where_not_between(mut self, field: &str, start: Value, end: Value) -> Self {
        self.bindings.push(start.clone());
        self.bindings.push(end.clone());
        self.wheres.push(WhereClause::new(
            WhereType::NotBetween,
            field,
            "NOT BETWEEN",
            WhereValue::Between(start, end),
            "AND",
        ));
        self
    }

    /// 添加 WHERE LIKE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `pattern`: 匹配模式
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    ///
    /// # 示例
    /// ```ignore
    /// builder.where_like("name", "%张%");
    /// ```
    pub fn where_like(mut self, field: &str, pattern: &str) -> Self {
        self.bindings.push(Value::String(pattern.to_string()));
        self.wheres.push(WhereClause::new(
            WhereType::Like,
            field,
            "LIKE",
            WhereValue::Single(Value::String(pattern.to_string())),
            "AND",
        ));
        self
    }

    /// 添加 WHERE NOT LIKE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `pattern`: 匹配模式
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn where_not_like(mut self, field: &str, pattern: &str) -> Self {
        self.bindings.push(Value::String(pattern.to_string()));
        self.wheres.push(WhereClause::new(
            WhereType::NotLike,
            field,
            "NOT LIKE",
            WhereValue::Single(Value::String(pattern.to_string())),
            "AND",
        ));
        self
    }

    /// 添加 WHERE EXISTS 子查询条件
    ///
    /// # 参数
    /// - `subquery`: 子查询 SQL
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    ///
    /// # 示例
    /// ```ignore
    /// builder.where_exists("SELECT 1 FROM orders WHERE orders.user_id = users.id");
    /// ```
    pub fn where_exists(mut self, subquery: &str) -> Self {
        self.wheres.push(WhereClause::new(
            WhereType::Exists,
            "",
            "EXISTS",
            WhereValue::SubQuery(subquery.to_string()),
            "AND",
        ));
        self
    }

    /// 添加 WHERE NOT EXISTS 子查询条件
    ///
    /// # 参数
    /// - `subquery`: 子查询 SQL
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn where_not_exists(mut self, subquery: &str) -> Self {
        self.wheres.push(WhereClause::new(
            WhereType::NotExists,
            "",
            "NOT EXISTS",
            WhereValue::SubQuery(subquery.to_string()),
            "AND",
        ));
        self
    }

    /// 添加原始 WHERE 条件
    ///
    /// # 参数
    /// - `sql`: 原始 SQL 条件
    /// - `bindings`: 绑定参数
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    ///
    /// # 示例
    /// ```ignore
    /// builder.where_raw("id = ? OR name = ?", vec![Value::Int(1), Value::String("test".to_string())]);
    /// ```
    pub fn where_raw(mut self, sql: &str, bindings: Vec<Value>) -> Self {
        self.bindings.extend(bindings);
        self.wheres.push(WhereClause::new(
            WhereType::Raw,
            "",
            "",
            WhereValue::Raw(sql.to_string()),
            "AND",
        ));
        self
    }

    /// 添加字段比较 WHERE 条件
    ///
    /// # 参数
    /// - `field1`: 第一个字段名
    /// - `operator`: 操作符
    /// - `field2`: 第二个字段名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    ///
    /// # 示例
    /// ```ignore
    /// builder.where_column("created_at", ">", "updated_at");
    /// ```
    pub fn where_column(mut self, field1: &str, operator: &str, field2: &str) -> Self {
        self.wheres.push(WhereClause::new(
            WhereType::Column,
            field1,
            operator,
            WhereValue::Column(field2.to_string()),
            "AND",
        ));
        self
    }

    /// 添加日期比较 WHERE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `date`: 日期值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    ///
    /// # 示例
    /// ```ignore
    /// builder.where_date("created_at", "=", "2024-01-01");
    /// ```
    pub fn where_date(mut self, field: &str, operator: &str, date: &str) -> Self {
        let sql = self.build_date_sql(field, operator, date);
        self.bindings.push(Value::String(date.to_string()));
        self.wheres.push(WhereClause::new(
            WhereType::Date,
            field,
            operator,
            WhereValue::Raw(sql),
            "AND",
        ));
        self
    }

    /// 构建日期比较 SQL
    ///
    /// 根据不同数据库类型生成对应的日期比较 SQL
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `date`: 日期值
    ///
    /// # 返回
    /// 日期比较 SQL 字符串
    fn build_date_sql(&self, field: &str, operator: &str, _date: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("DATE({}) {} ?", field, operator),
            DatabaseType::PostgreSQL => format!("{}::date {} ?::date", field, operator),
            DatabaseType::SQLite => format!("date({}) {} ?", field, operator),
        }
    }

    /// 添加时间比较 WHERE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `time`: 时间值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn where_time(mut self, field: &str, operator: &str, time: &str) -> Self {
        let sql = self.build_time_sql(field, operator, time);
        self.bindings.push(Value::String(time.to_string()));
        self.wheres.push(WhereClause::new(
            WhereType::Time,
            field,
            operator,
            WhereValue::Raw(sql),
            "AND",
        ));
        self
    }

    /// 构建时间比较 SQL
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `time`: 时间值
    ///
    /// # 返回
    /// 时间比较 SQL 字符串
    fn build_time_sql(&self, field: &str, operator: &str, _time: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("TIME({}) {} ?", field, operator),
            DatabaseType::PostgreSQL => format!("{}::time {} ?::time", field, operator),
            DatabaseType::SQLite => format!("time({}) {} ?", field, operator),
        }
    }

    /// 添加年份比较 WHERE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `year`: 年份值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn where_year(mut self, field: &str, operator: &str, year: i32) -> Self {
        let sql = self.build_year_sql(field, operator);
        self.bindings.push(Value::Int(year as i64));
        self.wheres.push(WhereClause::new(
            WhereType::Year,
            field,
            operator,
            WhereValue::Raw(sql),
            "AND",
        ));
        self
    }

    /// 构建年份比较 SQL
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    ///
    /// # 返回
    /// 年份比较 SQL 字符串
    fn build_year_sql(&self, field: &str, operator: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("YEAR({}) {} ?", field, operator),
            DatabaseType::PostgreSQL => format!("EXTRACT(YEAR FROM {}) {} ?", field, operator),
            DatabaseType::SQLite => format!("strftime('%Y', {}) {} ?", field, operator),
        }
    }

    /// 添加月份比较 WHERE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `month`: 月份值（1-12）
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn where_month(mut self, field: &str, operator: &str, month: i32) -> Self {
        let sql = self.build_month_sql(field, operator);
        self.bindings.push(Value::Int(month as i64));
        self.wheres.push(WhereClause::new(
            WhereType::Month,
            field,
            operator,
            WhereValue::Raw(sql),
            "AND",
        ));
        self
    }

    /// 构建月份比较 SQL
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    ///
    /// # 返回
    /// 月份比较 SQL 字符串
    fn build_month_sql(&self, field: &str, operator: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("MONTH({}) {} ?", field, operator),
            DatabaseType::PostgreSQL => format!("EXTRACT(MONTH FROM {}) {} ?", field, operator),
            DatabaseType::SQLite => format!("strftime('%m', {}) {} ?", field, operator),
        }
    }

    /// 添加日期（日）比较 WHERE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `day`: 日期值（1-31）
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn where_day(mut self, field: &str, operator: &str, day: i32) -> Self {
        let sql = self.build_day_sql(field, operator);
        self.bindings.push(Value::Int(day as i64));
        self.wheres.push(WhereClause::new(
            WhereType::Day,
            field,
            operator,
            WhereValue::Raw(sql),
            "AND",
        ));
        self
    }

    /// 构建日期（日）比较 SQL
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    ///
    /// # 返回
    /// 日期比较 SQL 字符串
    fn build_day_sql(&self, field: &str, operator: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("DAY({}) {} ?", field, operator),
            DatabaseType::PostgreSQL => format!("EXTRACT(DAY FROM {}) {} ?", field, operator),
            DatabaseType::SQLite => format!("strftime('%d', {}) {} ?", field, operator),
        }
    }

    /// 添加 JSON 包含 WHERE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `value`: JSON 值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    ///
    /// # 示例
    /// ```ignore
    /// builder.where_json_contains("tags", serde_json::json!("php"));
    /// ```
    pub fn where_json_contains(mut self, field: &str, value: serde_json::Value) -> Self {
        let sql = self.build_json_contains_sql(field, &value);
        self.bindings.push(Value::String(value.to_string()));
        self.wheres.push(WhereClause::new(
            WhereType::JsonContains,
            field,
            "JSON_CONTAINS",
            WhereValue::Json(value),
            "AND",
        ));
        // 将生成的 SQL 存储为 Raw 类型
        self.wheres.last_mut().unwrap().value = WhereValue::Raw(sql);
        self
    }

    /// 构建 JSON 包含 SQL
    ///
    /// 根据不同数据库类型生成对应的 JSON 包含 SQL
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `value`: JSON 值
    ///
    /// # 返回
    /// JSON 包含 SQL 字符串
    fn build_json_contains_sql(&self, field: &str, value: &serde_json::Value) -> String {
        match self.db_type {
            DatabaseType::MySQL => {
                // MySQL 使用 JSON_CONTAINS 函数
                format!("JSON_CONTAINS({}, ?)", self.quote_identifier(field))
            }
            DatabaseType::PostgreSQL => {
                // PostgreSQL 使用 @> 操作符
                format!("{} @> ?::jsonb", self.quote_identifier(field))
            }
            DatabaseType::SQLite => {
                // SQLite 使用 json_each 函数模拟
                format!(
                    "EXISTS (SELECT 1 FROM json_each({}) WHERE value = ?)",
                    self.quote_identifier(field)
                )
            }
        }
    }

    /// 添加 JSON 长度 WHERE 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `length`: 长度值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn where_json_length(mut self, field: &str, operator: &str, length: i64) -> Self {
        let sql = self.build_json_length_sql(field, operator);
        self.bindings.push(Value::Int(length));
        self.wheres.push(WhereClause::new(
            WhereType::JsonLength,
            field,
            operator,
            WhereValue::Raw(sql),
            "AND",
        ));
        self
    }

    /// 构建 JSON 长度 SQL
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    ///
    /// # 返回
    /// JSON 长度 SQL 字符串
    fn build_json_length_sql(&self, field: &str, operator: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("JSON_LENGTH({}) {} ?", self.quote_identifier(field), operator),
            DatabaseType::PostgreSQL => format!("jsonb_array_length({}) {} ?", self.quote_identifier(field), operator),
            DatabaseType::SQLite => format!("json_array_length({}) {} ?", self.quote_identifier(field), operator),
        }
    }

    /// 添加嵌套 WHERE 条件
    ///
    /// # 参数
    /// - `callback`: 嵌套条件构建回调函数
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    ///
    /// # 示例
    /// ```ignore
    /// builder.where_nested(|nested| {
    ///     nested.where_clause("status", "=", Value::Int(1))
    ///            .or_where("status", "=", Value::Int(2))
    /// });
    /// ```
    pub fn where_nested<F>(mut self, callback: F) -> Self
    where
        F: FnOnce(WhereBuilder) -> WhereBuilder,
    {
        // 创建嵌套构建器
        let nested_builder = callback(WhereBuilder::new(self.db_type));
        // 构建嵌套 SQL
        let (nested_sql, nested_bindings) = nested_builder.build_nested();
        // 添加嵌套条件
        self.bindings.extend(nested_bindings);
        self.wheres.push(WhereClause::new(
            WhereType::Raw,
            "",
            "",
            WhereValue::Raw(format!("({})", nested_sql)),
            "AND",
        ));
        self
    }

    /// 构建嵌套条件 SQL
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    fn build_nested(&self) -> (String, Vec<Value>) {
        let mut sql = String::new();
        let mut bindings = Vec::new();
        let mut first = true;

        for clause in &self.wheres {
            if !first {
                sql.push_str(&format!(" {} ", clause.connector));
            }
            first = false;

            let clause_sql = self.build_clause_sql(clause);
            sql.push_str(&clause_sql);
        }

        bindings.extend(self.bindings.clone());
        (sql, bindings)
    }

    /// 构建单个条件 SQL
    ///
    /// # 参数
    /// - `clause`: WHERE 条件子句
    ///
    /// # 返回
    /// 条件 SQL 字符串
    fn build_clause_sql(&self, clause: &WhereClause) -> String {
        match &clause.where_type {
            WhereType::Basic => {
                format!("{} {} ?", self.quote_identifier(&clause.field), clause.operator)
            }
            WhereType::Null => {
                format!("{} IS NULL", self.quote_identifier(&clause.field))
            }
            WhereType::NotNull => {
                format!("{} IS NOT NULL", self.quote_identifier(&clause.field))
            }
            WhereType::In | WhereType::NotIn => {
                if let WhereValue::Multiple(values) = &clause.value {
                    let placeholders: Vec<String> = values.iter().map(|_| "?".to_string()).collect();
                    format!(
                        "{} {} ({})",
                        self.quote_identifier(&clause.field),
                        clause.operator,
                        placeholders.join(", ")
                    )
                } else {
                    String::new()
                }
            }
            WhereType::Between | WhereType::NotBetween => {
                format!(
                    "{} {} ? AND ?",
                    self.quote_identifier(&clause.field),
                    clause.operator
                )
            }
            WhereType::Like | WhereType::NotLike => {
                format!(
                    "{} {} ?",
                    self.quote_identifier(&clause.field),
                    clause.operator
                )
            }
            WhereType::Exists => {
                if let WhereValue::SubQuery(subquery) = &clause.value {
                    format!("EXISTS ({})", subquery)
                } else {
                    String::new()
                }
            }
            WhereType::NotExists => {
                if let WhereValue::SubQuery(subquery) = &clause.value {
                    format!("NOT EXISTS ({})", subquery)
                } else {
                    String::new()
                }
            }
            WhereType::Raw => {
                if let WhereValue::Raw(sql) = &clause.value {
                    sql.clone()
                } else {
                    String::new()
                }
            }
            WhereType::Column => {
                if let WhereValue::Column(field2) = &clause.value {
                    format!(
                        "{} {} {}",
                        self.quote_identifier(&clause.field),
                        clause.operator,
                        self.quote_identifier(field2)
                    )
                } else {
                    String::new()
                }
            }
            WhereType::Date | WhereType::Time | WhereType::Year | WhereType::Month | WhereType::Day
            | WhereType::JsonContains | WhereType::JsonLength => {
                if let WhereValue::Raw(sql) = &clause.value {
                    sql.clone()
                } else {
                    String::new()
                }
            }
        }
    }

    /// 引用标识符
    ///
    /// 根据数据库类型使用不同的引用符号
    ///
    /// # 参数
    /// - `identifier`: 标识符名称
    ///
    /// # 返回
    /// 引用后的标识符
    fn quote_identifier(&self, identifier: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }

    /// 构建完整的 WHERE 子句
    ///
    /// # 返回
    /// (WHERE 子句 SQL, 绑定参数列表)
    pub fn build(&self) -> (String, Vec<Value>) {
        if self.wheres.is_empty() {
            return (String::new(), Vec::new());
        }

        let mut sql = String::new();
        let mut bindings = Vec::new();
        let mut first = true;

        for clause in &self.wheres {
            if !first {
                sql.push_str(&format!(" {} ", clause.connector));
            }
            first = false;

            let clause_sql = self.build_clause_sql(clause);
            sql.push_str(&clause_sql);
        }

        bindings.extend(self.bindings.clone());

        (format!("WHERE {}", sql), bindings)
    }
}

/// WHERE 条件组
///
/// 用于构建复杂的 WHERE 条件组合
#[derive(Debug, Clone)]
pub struct WhereGroup {
    /// 条件组内的条件列表
    pub clauses: Vec<WhereClause>,
    /// 条件组的逻辑连接符
    pub connector: String,
}

impl WhereGroup {
    /// 创建新的条件组
    ///
    /// # 参数
    /// - `connector`: 逻辑连接符（AND / OR）
    ///
    /// # 返回
    /// 新的条件组实例
    pub fn new(connector: &str) -> Self {
        Self {
            clauses: Vec::new(),
            connector: connector.to_string(),
        }
    }

    /// 添加条件到组
    ///
    /// # 参数
    /// - `clause`: WHERE 条件子句
    pub fn add(&mut self, clause: WhereClause) {
        self.clauses.push(clause);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_where_basic() {
        let builder = WhereBuilder::new(DatabaseType::MySQL)
            .where_clause("id", "=", Value::Int(1));

        let (sql, bindings) = builder.build();
        // MySQL 使用反引号引用标识符
        assert!(sql.contains("`id` = ?"));
        assert_eq!(bindings.len(), 1);
    }

    #[test]
    fn test_where_null() {
        let builder = WhereBuilder::new(DatabaseType::MySQL)
            .where_null("deleted_at");

        let (sql, _) = builder.build();
        // MySQL 使用反引号引用标识符
        assert!(sql.contains("`deleted_at` IS NULL"));
    }

    #[test]
    fn test_where_in() {
        let builder = WhereBuilder::new(DatabaseType::MySQL)
            .where_in("id", vec![Value::Int(1), Value::Int(2), Value::Int(3)]);

        let (sql, bindings) = builder.build();
        // MySQL 使用反引号引用标识符
        assert!(sql.contains("`id` IN (?, ?, ?)"));
        assert_eq!(bindings.len(), 3);
    }

    #[test]
    fn test_where_between() {
        let builder = WhereBuilder::new(DatabaseType::MySQL)
            .where_between("age", Value::Int(18), Value::Int(30));

        let (sql, bindings) = builder.build();
        // MySQL 使用反引号引用标识符
        assert!(sql.contains("`age` BETWEEN ? AND ?"));
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_or_where() {
        let builder = WhereBuilder::new(DatabaseType::MySQL)
            .where_clause("status", "=", Value::Int(1))
            .or_where("status", "=", Value::Int(2));

        let (sql, _) = builder.build();
        assert!(sql.contains("OR"));
    }

    #[test]
    fn test_database_type_quote() {
        let mysql_builder = WhereBuilder::new(DatabaseType::MySQL);
        assert_eq!(mysql_builder.quote_identifier("table"), "`table`");

        let pg_builder = WhereBuilder::new(DatabaseType::PostgreSQL);
        assert_eq!(pg_builder.quote_identifier("table"), "\"table\"");

        let sqlite_builder = WhereBuilder::new(DatabaseType::SQLite);
        assert_eq!(sqlite_builder.quote_identifier("table"), "\"table\"");
    }
}
