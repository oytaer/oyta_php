//! 查询执行模块
//!
//! 提供数据库查询执行功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：select、find、value、column、chunk、cursor 等

use crate::interpreter::value::Value;
use std::collections::HashMap;

/// 查询构建器
///
/// 提供链式调用的查询构建方法
#[derive(Debug, Clone)]
pub struct SelectBuilder {
    /// 表名
    pub table: String,
    /// 表别名
    pub table_alias: Option<String>,
    /// 查询字段
    pub fields: Vec<String>,
    /// 是否去重
    pub distinct: bool,
    /// WHERE 条件
    pub wheres: Vec<WhereClause>,
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
    /// 锁定
    pub lock: Option<String>,
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

/// WHERE 条件子句
#[derive(Debug, Clone)]
pub struct WhereClause {
    /// 条件 SQL
    pub condition: String,
    /// 绑定参数
    pub bindings: Vec<Value>,
    /// 逻辑连接符
    pub connector: String,
}

impl SelectBuilder {
    /// 创建新的查询构建器
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的查询构建器实例
    pub fn new(table: &str, db_type: DatabaseType) -> Self {
        Self {
            table: table.to_string(),
            table_alias: None,
            fields: vec!["*".to_string()],
            distinct: false,
            wheres: Vec::new(),
            joins: Vec::new(),
            group_by: Vec::new(),
            having: None,
            order_by: Vec::new(),
            limit: None,
            offset: None,
            lock: None,
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

    /// 添加查询字段
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn add_field(mut self, field: &str) -> Self {
        self.fields.push(field.to_string());
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
    /// - `bindings`: 绑定参数
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn where_raw(mut self, condition: &str, bindings: Vec<Value>) -> Self {
        self.wheres.push(WhereClause {
            condition: condition.to_string(),
            bindings,
            connector: "AND".to_string(),
        });
        self
    }

    /// 添加 WHERE 字段等于条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `value`: 值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn where_eq(mut self, field: &str, value: Value) -> Self {
        self.wheres.push(WhereClause {
            condition: format!("{} = ?", self.quote_identifier(field)),
            bindings: vec![value],
            connector: "AND".to_string(),
        });
        self
    }

    /// 添加 JOIN 子句
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
        self.order_by.push(format!("{} {}", self.quote_identifier(field), direction));
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

    /// 设置锁定
    ///
    /// # 参数
    /// - `lock`: 锁定 SQL
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn lock(mut self, lock: &str) -> Self {
        self.lock = Some(lock.to_string());
        self
    }

    /// 构建查询 SQL
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    pub fn build(&self) -> (String, Vec<Value>) {
        let mut sql = String::new();
        let mut bindings = Vec::new();

        // SELECT 子句
        let distinct_str = if self.distinct { "DISTINCT " } else { "" };
        sql.push_str(&format!("SELECT {}{}", distinct_str, self.fields.join(", ")));

        // FROM 子句
        if let Some(alias) = &self.table_alias {
            sql.push_str(&format!(" FROM {} AS {}", self.quote_identifier(&self.table), self.quote_identifier(alias)));
        } else {
            sql.push_str(&format!(" FROM {}", self.quote_identifier(&self.table)));
        }

        // JOIN 子句
        for join in &self.joins {
            sql.push_str(&format!(" {}", join));
        }

        // WHERE 子句
        if !self.wheres.is_empty() {
            sql.push_str(" WHERE ");
            for (i, clause) in self.wheres.iter().enumerate() {
                if i > 0 {
                    sql.push_str(&format!(" {} ", clause.connector));
                }
                sql.push_str(&clause.condition);
                bindings.extend(clause.bindings.clone());
            }
        }

        // GROUP BY 子句
        if !self.group_by.is_empty() {
            let group_fields: Vec<String> = self.group_by.iter()
                .map(|f| self.quote_identifier(f))
                .collect();
            sql.push_str(&format!(" GROUP BY {}", group_fields.join(", ")));
        }

        // HAVING 子句
        if let Some(having) = &self.having {
            sql.push_str(&format!(" HAVING {}", having));
        }

        // ORDER BY 子句
        if !self.order_by.is_empty() {
            sql.push_str(&format!(" ORDER BY {}", self.order_by.join(", ")));
        }

        // LIMIT 子句
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET 子句
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        // 锁定子句
        if let Some(lock) = &self.lock {
            sql.push_str(&format!(" {}", lock));
        }

        (sql, bindings)
    }

    /// 构建查询单条的 SQL
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    pub fn build_find(&self) -> (String, Vec<Value>) {
        let mut builder = self.clone();
        builder.limit = Some(1);
        builder.build()
    }

    /// 构建查询单个值的 SQL
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    pub fn build_value(&self, field: &str) -> (String, Vec<Value>) {
        let mut builder = self.clone();
        builder.fields = vec![field.to_string()];
        builder.limit = Some(1);
        builder.build()
    }

    /// 构建查询单列的 SQL
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    pub fn build_column(&self, field: &str) -> (String, Vec<Value>) {
        let mut builder = self.clone();
        builder.fields = vec![field.to_string()];
        builder.build()
    }

    /// 构建分块查询 SQL
    ///
    /// # 参数
    /// - `chunk_size`: 分块大小
    /// - `offset`: 偏移量
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    pub fn build_chunk(&self, chunk_size: u64, offset: u64) -> (String, Vec<Value>) {
        let mut builder = self.clone();
        builder.limit = Some(chunk_size);
        builder.offset = Some(offset);
        builder.build()
    }

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }
}

/// 查询结果
///
/// 存储查询操作的结果
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// 数据行列表
    pub rows: Vec<HashMap<String, Value>>,
}

impl QueryResult {
    /// 创建新的查询结果
    ///
    /// # 参数
    /// - `rows`: 数据行列表
    ///
    /// # 返回
    /// 新的查询结果实例
    pub fn new(rows: Vec<HashMap<String, Value>>) -> Self {
        Self { rows }
    }

    /// 判断是否为空
    ///
    /// # 返回
    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// 获取第一行
    ///
    /// # 返回
    /// 第一行数据
    pub fn first(&self) -> Option<&HashMap<String, Value>> {
        self.rows.first()
    }

    /// 获取行数
    ///
    /// # 返回
    /// 行数
    pub fn count(&self) -> usize {
        self.rows.len()
    }
}

/// 分块迭代器
///
/// 用于分块处理大量数据
pub struct ChunkIterator {
    /// 查询构建器
    pub builder: SelectBuilder,
    /// 分块大小
    pub chunk_size: u64,
    /// 当前偏移量
    pub offset: u64,
    /// 是否完成
    pub finished: bool,
}

impl ChunkIterator {
    /// 创建新的分块迭代器
    ///
    /// # 参数
    /// - `builder`: 查询构建器
    /// - `chunk_size`: 分块大小
    ///
    /// # 返回
    /// 新的分块迭代器实例
    pub fn new(builder: SelectBuilder, chunk_size: u64) -> Self {
        Self {
            builder,
            chunk_size,
            offset: 0,
            finished: false,
        }
    }

    /// 获取下一块数据的 SQL
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表, 是否还有更多数据)
    pub fn next(&mut self) -> (String, Vec<Value>, bool) {
        let (sql, bindings) = self.builder.build_chunk(self.chunk_size, self.offset);
        self.offset += self.chunk_size;
        (sql, bindings, !self.finished)
    }

    /// 标记完成
    pub fn mark_finished(&mut self) {
        self.finished = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_builder() {
        let builder = SelectBuilder::new("users", DatabaseType::MySQL)
            .select(&["id", "name"])
            .where_eq("status", Value::Int(1));

        let (sql, bindings) = builder.build();
        assert!(sql.contains("SELECT id, name"));
        assert!(sql.contains("FROM `users`"));
        assert!(sql.contains("WHERE"));
        assert_eq!(bindings.len(), 1);
    }

    #[test]
    fn test_select_distinct() {
        let builder = SelectBuilder::new("users", DatabaseType::MySQL)
            .select(&["name"])
            .distinct();

        let (sql, _) = builder.build();
        assert!(sql.contains("SELECT DISTINCT"));
    }

    #[test]
    fn test_select_with_join() {
        let builder = SelectBuilder::new("users", DatabaseType::MySQL)
            .join("INNER JOIN posts ON users.id = posts.user_id");

        let (sql, _) = builder.build();
        assert!(sql.contains("INNER JOIN"));
    }

    #[test]
    fn test_select_with_order_limit() {
        let builder = SelectBuilder::new("users", DatabaseType::MySQL)
            .order_by("id", "DESC")
            .limit(10)
            .offset(5);

        let (sql, _) = builder.build();
        assert!(sql.contains("ORDER BY"));
        assert!(sql.contains("LIMIT 10"));
        assert!(sql.contains("OFFSET 5"));
    }

    #[test]
    fn test_build_find() {
        let builder = SelectBuilder::new("users", DatabaseType::MySQL)
            .where_eq("id", Value::Int(1));

        let (sql, _) = builder.build_find();
        assert!(sql.contains("LIMIT 1"));
    }

    #[test]
    fn test_build_value() {
        let builder = SelectBuilder::new("users", DatabaseType::MySQL)
            .where_eq("id", Value::Int(1));

        let (sql, _) = builder.build_value("name");
        assert!(sql.contains("SELECT name"));
    }

    #[test]
    fn test_chunk_iterator() {
        let builder = SelectBuilder::new("users", DatabaseType::MySQL);
        let mut iter = ChunkIterator::new(builder, 100);

        let (sql, _, _) = iter.next();
        assert!(sql.contains("LIMIT 100"));
        assert!(sql.contains("OFFSET 0"));
    }
}
