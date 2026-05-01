//! 聚合查询模块
//!
//! 提供数据库聚合查询方法
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：count、sum、avg、max、min 等聚合函数

use anyhow::Result;
use std::collections::HashMap;

use crate::interpreter::value::Value;

/// 聚合函数类型枚举
///
/// 定义支持的聚合函数类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AggregateType {
    /// COUNT - 统计数量
    Count,
    /// SUM - 求和
    Sum,
    /// AVG - 平均值
    Avg,
    /// MAX - 最大值
    Max,
    /// MIN - 最小值
    Min,
}

impl AggregateType {
    /// 获取聚合函数名称
    ///
    /// # 返回
    /// 聚合函数名称字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            AggregateType::Count => "COUNT",
            AggregateType::Sum => "SUM",
            AggregateType::Avg => "AVG",
            AggregateType::Max => "MAX",
            AggregateType::Min => "MIN",
        }
    }
}

/// 聚合查询构建器
///
/// 提供链式调用的聚合查询构建方法
#[derive(Debug, Clone)]
pub struct AggregateBuilder {
    /// 聚合函数类型
    pub aggregate_type: AggregateType,
    /// 聚合字段
    pub field: String,
    /// 是否去重
    pub distinct: bool,
    /// 别名
    pub alias: Option<String>,
    /// 数据库类型
    pub db_type: DatabaseType,
}

/// 数据库类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DatabaseType {
    /// MySQL 数据库
    MySQL,
    /// PostgreSQL 数据库
    PostgreSQL,
    /// SQLite 数据库
    SQLite,
}

impl AggregateBuilder {
    /// 创建新的聚合查询构建器
    ///
    /// # 参数
    /// - `aggregate_type`: 聚合函数类型
    /// - `field`: 聚合字段
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的聚合查询构建器实例
    pub fn new(aggregate_type: AggregateType, field: &str, db_type: DatabaseType) -> Self {
        Self {
            aggregate_type,
            field: field.to_string(),
            distinct: false,
            alias: None,
            db_type,
        }
    }

    /// 设置去重
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    /// 设置别名
    ///
    /// # 参数
    /// - `alias`: 别名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn alias(mut self, alias: &str) -> Self {
        self.alias = Some(alias.to_string());
        self
    }

    /// 构建聚合 SQL
    ///
    /// # 返回
    /// 聚合 SQL 字符串
    pub fn build(&self) -> String {
        let distinct_str = if self.distinct { "DISTINCT " } else { "" };
        let field = if self.field == "*" {
            "*".to_string()
        } else {
            self.quote_identifier(&self.field)
        };

        let sql = format!("{}({}{})", self.aggregate_type.as_str(), distinct_str, field);

        if let Some(alias) = &self.alias {
            format!("{} AS {}", sql, self.quote_identifier(alias))
        } else {
            sql
        }
    }

    /// 引用标识符
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
}

/// 聚合查询结果
///
/// 存储聚合查询的结果
#[derive(Debug, Clone)]
pub struct AggregateResult {
    /// 聚合值
    pub value: Value,
}

impl AggregateResult {
    /// 创建新的聚合结果
    ///
    /// # 参数
    /// - `value`: 聚合值
    ///
    /// # 返回
    /// 新的聚合结果实例
    pub fn new(value: Value) -> Self {
        Self { value }
    }

    /// 获取整数值
    ///
    /// # 返回
    /// 整数值，如果无法转换则返回 0
    pub fn as_int(&self) -> i64 {
        match &self.value {
            Value::Int(i) => *i,
            Value::Float(f) => *f as i64,
            _ => 0,
        }
    }

    /// 获取浮点数值
    ///
    /// # 返回
    /// 浮点数值，如果无法转换则返回 0.0
    pub fn as_float(&self) -> f64 {
        match &self.value {
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            _ => 0.0,
        }
    }
}

/// 聚合查询执行器
///
/// 执行聚合查询并返回结果
pub struct AggregateExecutor {
    /// 数据库类型
    pub db_type: DatabaseType,
}

impl AggregateExecutor {
    /// 创建新的聚合查询执行器
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的聚合查询执行器实例
    pub fn new(db_type: DatabaseType) -> Self {
        Self { db_type }
    }

    /// 构建 COUNT 查询 SQL
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `field`: 字段名（默认为 *）
    /// - `distinct`: 是否去重
    /// - `where_sql`: WHERE 条件 SQL
    ///
    /// # 返回
    /// COUNT 查询 SQL
    pub fn build_count_sql(
        &self,
        table: &str,
        field: &str,
        distinct: bool,
        where_sql: &str,
    ) -> String {
        let distinct_str = if distinct { "DISTINCT " } else { "" };
        let field = if field.is_empty() || field == "*" {
            "*".to_string()
        } else {
            self.quote_identifier(field)
        };

        let mut sql = format!(
            "SELECT {}({}{}) AS aggregate_count FROM {}",
            AggregateType::Count.as_str(),
            distinct_str,
            field,
            self.quote_identifier(table)
        );

        if !where_sql.is_empty() {
            sql.push_str(&format!(" {}", where_sql));
        }

        sql
    }

    /// 构建 SUM 查询 SQL
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `field`: 字段名
    /// - `where_sql`: WHERE 条件 SQL
    ///
    /// # 返回
    /// SUM 查询 SQL
    pub fn build_sum_sql(&self, table: &str, field: &str, where_sql: &str) -> String {
        let mut sql = format!(
            "SELECT {}({}) AS aggregate_sum FROM {}",
            AggregateType::Sum.as_str(),
            self.quote_identifier(field),
            self.quote_identifier(table)
        );

        if !where_sql.is_empty() {
            sql.push_str(&format!(" {}", where_sql));
        }

        sql
    }

    /// 构建 AVG 查询 SQL
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `field`: 字段名
    /// - `where_sql`: WHERE 条件 SQL
    ///
    /// # 返回
    /// AVG 查询 SQL
    pub fn build_avg_sql(&self, table: &str, field: &str, where_sql: &str) -> String {
        let mut sql = format!(
            "SELECT {}({}) AS aggregate_avg FROM {}",
            AggregateType::Avg.as_str(),
            self.quote_identifier(field),
            self.quote_identifier(table)
        );

        if !where_sql.is_empty() {
            sql.push_str(&format!(" {}", where_sql));
        }

        sql
    }

    /// 构建 MAX 查询 SQL
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `field`: 字段名
    /// - `where_sql`: WHERE 条件 SQL
    ///
    /// # 返回
    /// MAX 查询 SQL
    pub fn build_max_sql(&self, table: &str, field: &str, where_sql: &str) -> String {
        let mut sql = format!(
            "SELECT {}({}) AS aggregate_max FROM {}",
            AggregateType::Max.as_str(),
            self.quote_identifier(field),
            self.quote_identifier(table)
        );

        if !where_sql.is_empty() {
            sql.push_str(&format!(" {}", where_sql));
        }

        sql
    }

    /// 构建 MIN 查询 SQL
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `field`: 字段名
    /// - `where_sql`: WHERE 条件 SQL
    ///
    /// # 返回
    /// MIN 查询 SQL
    pub fn build_min_sql(&self, table: &str, field: &str, where_sql: &str) -> String {
        let mut sql = format!(
            "SELECT {}({}) AS aggregate_min FROM {}",
            AggregateType::Min.as_str(),
            self.quote_identifier(field),
            self.quote_identifier(table)
        );

        if !where_sql.is_empty() {
            sql.push_str(&format!(" {}", where_sql));
        }

        sql
    }

    /// 引用标识符
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

    /// 从查询结果中提取聚合值
    ///
    /// # 参数
    /// - `row`: 查询结果行
    /// - `aggregate_type`: 聚合类型
    ///
    /// # 返回
    /// 聚合值
    pub fn extract_value(
        &self,
        row: &HashMap<String, Value>,
        aggregate_type: AggregateType,
    ) -> Value {
        let key = match aggregate_type {
            AggregateType::Count => "aggregate_count",
            AggregateType::Sum => "aggregate_sum",
            AggregateType::Avg => "aggregate_avg",
            AggregateType::Max => "aggregate_max",
            AggregateType::Min => "aggregate_min",
        };

        row.get(key).cloned().unwrap_or(Value::Null)
    }
}

/// 统计查询构建器
///
/// 专门用于构建统计相关的查询
#[derive(Debug, Clone)]
pub struct CountBuilder {
    /// 表名
    pub table: String,
    /// 字段名
    pub field: String,
    /// 是否去重
    pub distinct: bool,
    /// WHERE 条件
    pub conditions: Vec<String>,
    /// 绑定参数
    pub bindings: Vec<Value>,
    /// 数据库类型
    pub db_type: DatabaseType,
}

impl CountBuilder {
    /// 创建新的统计构建器
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的统计构建器实例
    pub fn new(table: &str, db_type: DatabaseType) -> Self {
        Self {
            table: table.to_string(),
            field: "*".to_string(),
            distinct: false,
            conditions: Vec::new(),
            bindings: Vec::new(),
            db_type,
        }
    }

    /// 设置统计字段
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn field(mut self, field: &str) -> Self {
        self.field = field.to_string();
        self
    }

    /// 设置去重
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    /// 添加 WHERE 条件
    ///
    /// # 参数
    /// - `condition`: 条件 SQL
    /// - `binding`: 绑定参数
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn where_raw(mut self, condition: &str, binding: Option<Value>) -> Self {
        self.conditions.push(condition.to_string());
        if let Some(v) = binding {
            self.bindings.push(v);
        }
        self
    }

    /// 构建统计 SQL
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    pub fn build(&self) -> (String, Vec<Value>) {
        let distinct_str = if self.distinct { "DISTINCT " } else { "" };
        let field = if self.field == "*" {
            "*".to_string()
        } else {
            self.quote_identifier(&self.field)
        };

        let mut sql = format!(
            "SELECT COUNT({}{}) AS aggregate_count FROM {}",
            distinct_str,
            field,
            self.quote_identifier(&self.table)
        );

        if !self.conditions.is_empty() {
            sql.push_str(&format!(" WHERE {}", self.conditions.join(" AND ")));
        }

        (sql, self.bindings.clone())
    }

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }
}

/// 分组统计结果
///
/// 用于存储 GROUP BY 后的统计结果
#[derive(Debug, Clone)]
pub struct GroupedAggregate {
    /// 分组字段值
    pub group_value: Value,
    /// 聚合值
    pub aggregate_value: Value,
}

impl GroupedAggregate {
    /// 创建新的分组统计结果
    ///
    /// # 参数
    /// - `group_value`: 分组字段值
    /// - `aggregate_value`: 聚合值
    ///
    /// # 返回
    /// 新的分组统计结果实例
    pub fn new(group_value: Value, aggregate_value: Value) -> Self {
        Self {
            group_value,
            aggregate_value,
        }
    }
}

/// 分组聚合构建器
///
/// 用于构建带 GROUP BY 的聚合查询
#[derive(Debug, Clone)]
pub struct GroupedAggregateBuilder {
    /// 表名
    pub table: String,
    /// 分组字段
    pub group_by: String,
    /// 聚合类型
    pub aggregate_type: AggregateType,
    /// 聚合字段
    pub aggregate_field: String,
    /// 是否去重
    pub distinct: bool,
    /// HAVING 条件
    pub having: Option<String>,
    /// 数据库类型
    pub db_type: DatabaseType,
}

impl GroupedAggregateBuilder {
    /// 创建新的分组聚合构建器
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `group_by`: 分组字段
    /// - `aggregate_type`: 聚合类型
    /// - `aggregate_field`: 聚合字段
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的分组聚合构建器实例
    pub fn new(
        table: &str,
        group_by: &str,
        aggregate_type: AggregateType,
        aggregate_field: &str,
        db_type: DatabaseType,
    ) -> Self {
        Self {
            table: table.to_string(),
            group_by: group_by.to_string(),
            aggregate_type,
            aggregate_field: aggregate_field.to_string(),
            distinct: false,
            having: None,
            db_type,
        }
    }

    /// 设置去重
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    /// 设置 HAVING 条件
    ///
    /// # 参数
    /// - `having`: HAVING 条件 SQL
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn having(mut self, having: &str) -> Self {
        self.having = Some(having.to_string());
        self
    }

    /// 构建分组聚合 SQL
    ///
    /// # 返回
    /// 分组聚合 SQL 字符串
    pub fn build(&self) -> String {
        let distinct_str = if self.distinct { "DISTINCT " } else { "" };
        let agg_field = self.quote_identifier(&self.aggregate_field);
        let group_field = self.quote_identifier(&self.group_by);

        let mut sql = format!(
            "SELECT {}, {}({}{}) AS aggregate_value FROM {} GROUP BY {}",
            group_field,
            self.aggregate_type.as_str(),
            distinct_str,
            agg_field,
            self.quote_identifier(&self.table),
            group_field
        );

        if let Some(having) = &self.having {
            sql.push_str(&format!(" HAVING {}", having));
        }

        sql
    }

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_builder() {
        let builder = AggregateBuilder::new(AggregateType::Count, "*", DatabaseType::MySQL);
        let sql = builder.build();
        assert_eq!(sql, "COUNT(*)");
    }

    #[test]
    fn test_aggregate_builder_with_alias() {
        let builder = AggregateBuilder::new(AggregateType::Sum, "price", DatabaseType::MySQL)
            .alias("total_price");
        let sql = builder.build();
        assert_eq!(sql, "SUM(`price`) AS `total_price`");
    }

    #[test]
    fn test_aggregate_builder_distinct() {
        let builder = AggregateBuilder::new(AggregateType::Count, "user_id", DatabaseType::MySQL)
            .distinct();
        let sql = builder.build();
        assert_eq!(sql, "COUNT(DISTINCT `user_id`)");
    }

    #[test]
    fn test_count_builder() {
        let builder = CountBuilder::new("users", DatabaseType::MySQL);
        let (sql, _) = builder.build();
        assert_eq!(sql, "SELECT COUNT(*) AS aggregate_count FROM `users`");
    }

    #[test]
    fn test_count_builder_with_where() {
        let builder = CountBuilder::new("users", DatabaseType::MySQL)
            .where_raw("status = ?", Some(Value::Int(1)));
        let (sql, bindings) = builder.build();
        assert!(sql.contains("WHERE"));
        assert_eq!(bindings.len(), 1);
    }

    #[test]
    fn test_grouped_aggregate_builder() {
        let builder = GroupedAggregateBuilder::new(
            "orders",
            "user_id",
            AggregateType::Sum,
            "amount",
            DatabaseType::MySQL,
        );
        let sql = builder.build();
        assert!(sql.contains("GROUP BY"));
        assert!(sql.contains("SUM(`amount`)"));
    }

    #[test]
    fn test_database_type_quote() {
        let mysql_builder = AggregateBuilder::new(AggregateType::Count, "id", DatabaseType::MySQL);
        assert_eq!(mysql_builder.quote_identifier("field"), "`field`");

        let pg_builder = AggregateBuilder::new(AggregateType::Count, "id", DatabaseType::PostgreSQL);
        assert_eq!(pg_builder.quote_identifier("field"), "\"field\"");
    }
}
