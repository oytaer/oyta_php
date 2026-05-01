//! 子查询模块
//!
//! 提供子查询构建功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：标量子查询、行子查询、表子查询、EXISTS 子查询等

use crate::interpreter::value::Value;
use std::collections::HashMap;

/// 子查询类型枚举
///
/// 定义支持的子查询类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubQueryType {
    /// 标量子查询（返回单个值）
    Scalar,
    /// 行子查询（返回单行）
    Row,
    /// 表子查询（返回多行多列）
    Table,
    /// EXISTS 子查询
    Exists,
    /// NOT EXISTS 子查询
    NotExists,
    /// IN 子查询
    In,
    /// NOT IN 子查询
    NotIn,
    /// FROM 子查询
    From,
}

/// 子查询构建器
///
/// 提供链式调用的子查询构建方法
#[derive(Debug, Clone)]
pub struct SubQueryBuilder {
    /// 子查询 SQL
    pub sql: String,
    /// 绑定参数
    pub bindings: Vec<Value>,
    /// 子查询别名
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

impl SubQueryBuilder {
    /// 创建新的子查询构建器
    ///
    /// # 参数
    /// - `sql`: 子查询 SQL
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的子查询构建器实例
    pub fn new(sql: &str, db_type: DatabaseType) -> Self {
        Self {
            sql: sql.to_string(),
            bindings: Vec::new(),
            alias: None,
            db_type,
        }
    }

    /// 创建带参数的子查询构建器
    ///
    /// # 参数
    /// - `sql`: 子查询 SQL
    /// - `bindings`: 绑定参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的子查询构建器实例
    pub fn with_bindings(sql: &str, bindings: Vec<Value>, db_type: DatabaseType) -> Self {
        Self {
            sql: sql.to_string(),
            bindings,
            alias: None,
            db_type,
        }
    }

    /// 设置子查询别名
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

    /// 添加绑定参数
    ///
    /// # 参数
    /// - `value`: 参数值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn bind(mut self, value: Value) -> Self {
        self.bindings.push(value);
        self
    }

    /// 构建标量子查询 SQL
    ///
    /// 用于 SELECT 字段中的标量子查询
    ///
    /// # 返回
    /// 标量子查询 SQL 字符串
    pub fn build_scalar(&self) -> String {
        format!("({})", self.sql)
    }

    /// 构建表子查询 SQL
    ///
    /// 用于 FROM 子句中的表子查询
    ///
    /// # 返回
    /// 表子查询 SQL 字符串
    pub fn build_table(&self) -> String {
        if let Some(alias) = &self.alias {
            format!("({}) AS {}", self.sql, self.quote_identifier(alias))
        } else {
            format!("({})", self.sql)
        }
    }

    /// 构建 EXISTS 子查询 SQL
    ///
    /// # 返回
    /// EXISTS 子查询 SQL 字符串
    pub fn build_exists(&self) -> String {
        format!("EXISTS ({})", self.sql)
    }

    /// 构建 NOT EXISTS 子查询 SQL
    ///
    /// # 返回
    /// NOT EXISTS 子查询 SQL 字符串
    pub fn build_not_exists(&self) -> String {
        format!("NOT EXISTS ({})", self.sql)
    }

    /// 构建 IN 子查询 SQL
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// IN 子查询 SQL 字符串
    pub fn build_in(&self, field: &str) -> String {
        format!("{} IN ({})", self.quote_identifier(field), self.sql)
    }

    /// 构建 NOT IN 子查询 SQL
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// NOT IN 子查询 SQL 字符串
    pub fn build_not_in(&self, field: &str) -> String {
        format!("{} NOT IN ({})", self.quote_identifier(field), self.sql)
    }

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }

    /// 获取绑定参数
    ///
    /// # 返回
    /// 绑定参数列表
    pub fn get_bindings(&self) -> Vec<Value> {
        self.bindings.clone()
    }
}

/// 子查询工厂
///
/// 用于创建各种类型的子查询
pub struct SubQueryFactory {
    /// 数据库类型
    pub db_type: DatabaseType,
}

impl SubQueryFactory {
    /// 创建新的子查询工厂
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的子查询工厂实例
    pub fn new(db_type: DatabaseType) -> Self {
        Self { db_type }
    }

    /// 创建 SELECT 子查询
    ///
    /// # 参数
    /// - `fields`: 查询字段
    /// - `table`: 表名
    /// - `where_clause`: WHERE 条件
    ///
    /// # 返回
    /// 子查询构建器
    pub fn select(&self, fields: &str, table: &str, where_clause: Option<&str>) -> SubQueryBuilder {
        let mut sql = format!("SELECT {} FROM {}", fields, self.quote_identifier(table));

        if let Some(where_sql) = where_clause {
            sql.push_str(&format!(" WHERE {}", where_sql));
        }

        SubQueryBuilder::new(&sql, self.db_type)
    }

    /// 创建 COUNT 子查询
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `where_clause`: WHERE 条件
    ///
    /// # 返回
    /// 子查询构建器
    pub fn count(&self, table: &str, where_clause: Option<&str>) -> SubQueryBuilder {
        let mut sql = format!("SELECT COUNT(*) FROM {}", self.quote_identifier(table));

        if let Some(where_sql) = where_clause {
            sql.push_str(&format!(" WHERE {}", where_sql));
        }

        SubQueryBuilder::new(&sql, self.db_type)
    }

    /// 创建 EXISTS 子查询
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `where_clause`: WHERE 条件
    ///
    /// # 返回
    /// 子查询构建器
    pub fn exists(&self, table: &str, where_clause: &str) -> SubQueryBuilder {
        let sql = format!(
            "SELECT 1 FROM {} WHERE {} LIMIT 1",
            self.quote_identifier(table),
            where_clause
        );
        SubQueryBuilder::new(&sql, self.db_type)
    }

    /// 创建 FROM 子查询
    ///
    /// # 参数
    /// - `subquery`: 子查询 SQL
    /// - `alias`: 别名
    ///
    /// # 返回
    /// 子查询构建器
    pub fn from_subquery(&self, subquery: &str, alias: &str) -> SubQueryBuilder {
        SubQueryBuilder::new(subquery, self.db_type).alias(alias)
    }

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }
}

/// 嵌套查询构建器
///
/// 用于构建复杂的嵌套查询
#[derive(Debug, Clone)]
pub struct NestedQueryBuilder {
    /// 外层查询字段
    pub fields: Vec<String>,
    /// 主表名
    pub table: String,
    /// 表别名
    pub table_alias: Option<String>,
    /// WHERE 条件
    pub wheres: Vec<String>,
    /// JOIN 子句
    pub joins: Vec<String>,
    /// GROUP BY 字段
    pub group_by: Vec<String>,
    /// HAVING 条件
    pub having: Option<String>,
    /// ORDER BY 字段
    pub order_by: Vec<String>,
    /// LIMIT
    pub limit: Option<u64>,
    /// OFFSET
    pub offset: Option<u64>,
    /// 绑定参数
    pub bindings: Vec<Value>,
    /// 数据库类型
    pub db_type: DatabaseType,
}

impl NestedQueryBuilder {
    /// 创建新的嵌套查询构建器
    ///
    /// # 参数
    /// - `table`: 主表名
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的嵌套查询构建器实例
    pub fn new(table: &str, db_type: DatabaseType) -> Self {
        Self {
            fields: vec!["*".to_string()],
            table: table.to_string(),
            table_alias: None,
            wheres: Vec::new(),
            joins: Vec::new(),
            group_by: Vec::new(),
            having: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
            bindings: Vec::new(),
            db_type,
        }
    }

    /// 设置查询字段
    ///
    /// # 参数
    /// - `fields`: 字段列表
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn select(mut self, fields: &[&str]) -> Self {
        self.fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 设置表别名
    ///
    /// # 参数
    /// - `alias`: 别名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn alias(mut self, alias: &str) -> Self {
        self.table_alias = Some(alias.to_string());
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
        self.wheres.push(condition.to_string());
        if let Some(v) = binding {
            self.bindings.push(v);
        }
        self
    }

    /// 添加 JOIN
    ///
    /// # 参数
    /// - `join`: JOIN SQL
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn join(mut self, join: &str) -> Self {
        self.joins.push(join.to_string());
        self
    }

    /// 设置 GROUP BY
    ///
    /// # 参数
    /// - `fields`: 分组字段
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn group_by(mut self, fields: &[&str]) -> Self {
        self.group_by = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 设置 HAVING
    ///
    /// # 参数
    /// - `having`: HAVING 条件
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn having(mut self, having: &str) -> Self {
        self.having = Some(having.to_string());
        self
    }

    /// 设置 ORDER BY
    ///
    /// # 参数
    /// - `field`: 排序字段
    /// - `direction`: 排序方向
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn order_by(mut self, field: &str, direction: &str) -> Self {
        self.order_by.push(format!("{} {}", field, direction));
        self
    }

    /// 设置 LIMIT
    ///
    /// # 参数
    /// - `limit`: 限制数量
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// 设置 OFFSET
    ///
    /// # 参数
    /// - `offset`: 偏移量
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// 构建 SQL
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    pub fn build(&self) -> (String, Vec<Value>) {
        let mut sql = String::new();

        // SELECT 字段
        sql.push_str(&format!("SELECT {}", self.fields.join(", ")));

        // FROM 表
        if let Some(alias) = &self.table_alias {
            sql.push_str(&format!(" FROM {} AS {}", self.quote_identifier(&self.table), self.quote_identifier(alias)));
        } else {
            sql.push_str(&format!(" FROM {}", self.quote_identifier(&self.table)));
        }

        // JOIN
        for join in &self.joins {
            sql.push_str(&format!(" {}", join));
        }

        // WHERE
        if !self.wheres.is_empty() {
            sql.push_str(&format!(" WHERE {}", self.wheres.join(" AND ")));
        }

        // GROUP BY
        if !self.group_by.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }

        // HAVING
        if let Some(having) = &self.having {
            sql.push_str(&format!(" HAVING {}", having));
        }

        // ORDER BY
        if !self.order_by.is_empty() {
            sql.push_str(&format!(" ORDER BY {}", self.order_by.join(", ")));
        }

        // LIMIT
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        (sql, self.bindings.clone())
    }

    /// 构建为子查询
    ///
    /// # 参数
    /// - `alias`: 子查询别名
    ///
    /// # 返回
    /// (子查询 SQL, 绑定参数列表)
    pub fn build_as_subquery(&self, alias: &str) -> (String, Vec<Value>) {
        let (sql, bindings) = self.build();
        (format!("({}) AS {}", sql, self.quote_identifier(alias)), bindings)
    }

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }
}

/// 行子查询比较操作符
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RowComparisonOperator {
    /// 等于
    Eq,
    /// 不等于
    Ne,
    /// 小于
    Lt,
    /// 小于等于
    Le,
    /// 大于
    Gt,
    /// 大于等于
    Ge,
}

impl RowComparisonOperator {
    /// 获取操作符字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            RowComparisonOperator::Eq => "=",
            RowComparisonOperator::Ne => "<>",
            RowComparisonOperator::Lt => "<",
            RowComparisonOperator::Le => "<=",
            RowComparisonOperator::Gt => ">",
            RowComparisonOperator::Ge => ">=",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subquery_builder() {
        let builder = SubQueryBuilder::new("SELECT name FROM users WHERE id = ?", DatabaseType::MySQL)
            .bind(Value::Int(1));

        let sql = builder.build_scalar();
        assert!(sql.starts_with("("));
        assert!(sql.ends_with(")"));
    }

    #[test]
    fn test_subquery_factory() {
        let factory = SubQueryFactory::new(DatabaseType::MySQL);
        let builder = factory.select("name", "users", Some("id = ?"));

        let sql = builder.build_scalar();
        assert!(sql.contains("SELECT name"));
    }

    #[test]
    fn test_exists_subquery() {
        let factory = SubQueryFactory::new(DatabaseType::MySQL);
        let builder = factory.exists("orders", "orders.user_id = users.id");

        let sql = builder.build_exists();
        assert!(sql.contains("EXISTS"));
    }

    #[test]
    fn test_nested_query_builder() {
        let builder = NestedQueryBuilder::new("users", DatabaseType::MySQL)
            .select(&["id", "name"])
            .where_raw("status = ?", Some(Value::Int(1)))
            .order_by("id", "DESC")
            .limit(10);

        let (sql, bindings) = builder.build();
        assert!(sql.contains("SELECT id, name"));
        assert!(sql.contains("WHERE status = ?"));
        assert!(sql.contains("ORDER BY id DESC"));
        assert!(sql.contains("LIMIT 10"));
        assert_eq!(bindings.len(), 1);
    }

    #[test]
    fn test_build_as_subquery() {
        let builder = NestedQueryBuilder::new("users", DatabaseType::MySQL)
            .select(&["id", "name"]);

        let (sql, _) = builder.build_as_subquery("u");
        assert!(sql.starts_with("("));
        assert!(sql.contains("AS `u`"));
    }
}
