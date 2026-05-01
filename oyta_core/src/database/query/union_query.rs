//! UNION 查询模块
//!
//! 提供数据库 UNION 查询构建功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：UNION、UNION ALL、INTERSECT、EXCEPT 等

use crate::interpreter::value::Value;

/// UNION 类型枚举
///
/// 定义支持的集合操作类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnionType {
    /// UNION（去重合并）
    Union,
    /// UNION ALL（保留重复合并）
    UnionAll,
    /// INTERSECT（交集）
    Intersect,
    /// INTERSECT ALL（保留重复交集）
    IntersectAll,
    /// EXCEPT（差集）
    Except,
    /// EXCEPT ALL（保留重复差集）
    ExceptAll,
}

impl UnionType {
    /// 获取集合操作关键字
    ///
    /// # 返回
    /// 集合操作关键字字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            UnionType::Union => "UNION",
            UnionType::UnionAll => "UNION ALL",
            UnionType::Intersect => "INTERSECT",
            UnionType::IntersectAll => "INTERSECT ALL",
            UnionType::Except => "EXCEPT",
            UnionType::ExceptAll => "EXCEPT ALL",
        }
    }

    /// 检查数据库是否支持该操作
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 是否支持
    pub fn is_supported(&self, db_type: DatabaseType) -> bool {
        match self {
            UnionType::Union | UnionType::UnionAll => true,
            UnionType::Intersect | UnionType::IntersectAll => {
                // MySQL 不支持 INTERSECT
                db_type != DatabaseType::MySQL
            }
            UnionType::Except | UnionType::ExceptAll => {
                // MySQL 不支持 EXCEPT
                db_type != DatabaseType::MySQL
            }
        }
    }
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

/// UNION 子句结构体
///
/// 存储单个 UNION 查询的信息
#[derive(Debug, Clone)]
pub struct UnionClause {
    /// UNION 类型
    pub union_type: UnionType,
    /// 查询 SQL
    pub sql: String,
    /// 绑定参数
    pub bindings: Vec<Value>,
}

impl UnionClause {
    /// 创建新的 UNION 子句
    ///
    /// # 参数
    /// - `union_type`: UNION 类型
    /// - `sql`: 查询 SQL
    /// - `bindings`: 绑定参数
    ///
    /// # 返回
    /// 新的 UNION 子句实例
    pub fn new(union_type: UnionType, sql: &str, bindings: Vec<Value>) -> Self {
        Self {
            union_type,
            sql: sql.to_string(),
            bindings,
        }
    }
}

/// UNION 查询构建器
///
/// 提供链式调用的 UNION 查询构建方法
#[derive(Debug, Clone)]
pub struct UnionBuilder {
    /// 第一个查询 SQL
    pub first_query: String,
    /// 第一个查询绑定参数
    pub first_bindings: Vec<Value>,
    /// UNION 子句列表
    pub unions: Vec<UnionClause>,
    /// 数据库类型
    pub db_type: DatabaseType,
    /// ORDER BY 子句
    pub order_by: Option<String>,
    /// LIMIT
    pub limit: Option<u64>,
    /// OFFSET
    pub offset: Option<u64>,
}

impl UnionBuilder {
    /// 创建新的 UNION 查询构建器
    ///
    /// # 参数
    /// - `first_query`: 第一个查询 SQL
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的 UNION 查询构建器实例
    pub fn new(first_query: &str, db_type: DatabaseType) -> Self {
        Self {
            first_query: first_query.to_string(),
            first_bindings: Vec::new(),
            unions: Vec::new(),
            db_type,
            order_by: None,
            limit: None,
            offset: None,
        }
    }

    /// 创建带参数的 UNION 查询构建器
    ///
    /// # 参数
    /// - `first_query`: 第一个查询 SQL
    /// - `bindings`: 绑定参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的 UNION 查询构建器实例
    pub fn with_bindings(first_query: &str, bindings: Vec<Value>, db_type: DatabaseType) -> Self {
        Self {
            first_query: first_query.to_string(),
            first_bindings: bindings,
            unions: Vec::new(),
            db_type,
            order_by: None,
            limit: None,
            offset: None,
        }
    }

    /// 添加 UNION 查询
    ///
    /// # 参数
    /// - `sql`: 查询 SQL
    /// - `bindings`: 绑定参数
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn union(mut self, sql: &str, bindings: Vec<Value>) -> Self {
        self.unions.push(UnionClause::new(UnionType::Union, sql, bindings));
        self
    }

    /// 添加 UNION ALL 查询
    ///
    /// # 参数
    /// - `sql`: 查询 SQL
    /// - `bindings`: 绑定参数
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn union_all(mut self, sql: &str, bindings: Vec<Value>) -> Self {
        self.unions.push(UnionClause::new(UnionType::UnionAll, sql, bindings));
        self
    }

    /// 添加 INTERSECT 查询
    ///
    /// # 参数
    /// - `sql`: 查询 SQL
    /// - `bindings`: 绑定参数
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn intersect(mut self, sql: &str, bindings: Vec<Value>) -> Self {
        if UnionType::Intersect.is_supported(self.db_type) {
            self.unions.push(UnionClause::new(UnionType::Intersect, sql, bindings));
        }
        self
    }

    /// 添加 INTERSECT ALL 查询
    ///
    /// # 参数
    /// - `sql`: 查询 SQL
    /// - `bindings`: 绑定参数
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn intersect_all(mut self, sql: &str, bindings: Vec<Value>) -> Self {
        if UnionType::IntersectAll.is_supported(self.db_type) {
            self.unions.push(UnionClause::new(UnionType::IntersectAll, sql, bindings));
        }
        self
    }

    /// 添加 EXCEPT 查询
    ///
    /// # 参数
    /// - `sql`: 查询 SQL
    /// - `bindings`: 绑定参数
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn except(mut self, sql: &str, bindings: Vec<Value>) -> Self {
        if UnionType::Except.is_supported(self.db_type) {
            self.unions.push(UnionClause::new(UnionType::Except, sql, bindings));
        }
        self
    }

    /// 添加 EXCEPT ALL 查询
    ///
    /// # 参数
    /// - `sql`: 查询 SQL
    /// - `bindings`: 绑定参数
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn except_all(mut self, sql: &str, bindings: Vec<Value>) -> Self {
        if UnionType::ExceptAll.is_supported(self.db_type) {
            self.unions.push(UnionClause::new(UnionType::ExceptAll, sql, bindings));
        }
        self
    }

    /// 设置 ORDER BY
    ///
    /// # 参数
    /// - `order_by`: 排序字段
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn order_by(mut self, order_by: &str) -> Self {
        self.order_by = Some(order_by.to_string());
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

    /// 构建完整的 UNION 查询 SQL
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    pub fn build(&self) -> (String, Vec<Value>) {
        let mut sql = String::new();
        let mut bindings = Vec::new();

        // 添加第一个查询
        sql.push_str(&self.first_query);
        bindings.extend(self.first_bindings.clone());

        // 添加 UNION 子句
        for union_clause in &self.unions {
            sql.push_str(&format!(" {} ({})", union_clause.union_type.as_str(), union_clause.sql));
            bindings.extend(union_clause.bindings.clone());
        }

        // 添加 ORDER BY
        if let Some(order_by) = &self.order_by {
            sql.push_str(&format!(" ORDER BY {}", order_by));
        }

        // 添加 LIMIT
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // 添加 OFFSET
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        (sql, bindings)
    }
}

/// UNION 查询工厂
///
/// 用于快速创建 UNION 查询
pub struct UnionFactory {
    /// 数据库类型
    pub db_type: DatabaseType,
}

impl UnionFactory {
    /// 创建新的 UNION 查询工厂
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的 UNION 查询工厂实例
    pub fn new(db_type: DatabaseType) -> Self {
        Self { db_type }
    }

    /// 创建两个查询的 UNION
    ///
    /// # 参数
    /// - `query1`: 第一个查询
    /// - `query2`: 第二个查询
    ///
    /// # 返回
    /// UNION 查询构建器
    pub fn union(&self, query1: &str, query2: &str) -> UnionBuilder {
        UnionBuilder::new(query1, self.db_type).union(query2, Vec::new())
    }

    /// 创建两个查询的 UNION ALL
    ///
    /// # 参数
    /// - `query1`: 第一个查询
    /// - `query2`: 第二个查询
    ///
    /// # 返回
    /// UNION 查询构建器
    pub fn union_all(&self, query1: &str, query2: &str) -> UnionBuilder {
        UnionBuilder::new(query1, self.db_type).union_all(query2, Vec::new())
    }

    /// 创建多个查询的 UNION
    ///
    /// # 参数
    /// - `queries`: 查询 SQL 列表
    ///
    /// # 返回
    /// UNION 查询构建器
    pub fn union_multiple(&self, queries: &[&str]) -> UnionBuilder {
        if queries.is_empty() {
            return UnionBuilder::new("", self.db_type);
        }

        let mut builder = UnionBuilder::new(queries[0], self.db_type);
        for query in queries.iter().skip(1) {
            builder = builder.union(query, Vec::new());
        }
        builder
    }
}

/// 子查询 UNION 构建器
///
/// 用于构建包含子查询的复杂 UNION 查询
#[derive(Debug, Clone)]
pub struct SubQueryUnionBuilder {
    /// UNION 构建器
    pub union_builder: UnionBuilder,
}

impl SubQueryUnionBuilder {
    /// 创建新的子查询 UNION 构建器
    ///
    /// # 参数
    /// - `first_query`: 第一个查询
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的子查询 UNION 构建器实例
    pub fn new(first_query: &str, db_type: DatabaseType) -> Self {
        Self {
            union_builder: UnionBuilder::new(first_query, db_type),
        }
    }

    /// 添加子查询 UNION
    ///
    /// # 参数
    /// - `subquery`: 子查询
    /// - `bindings`: 绑定参数
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn union_subquery(mut self, subquery: &str, bindings: Vec<Value>) -> Self {
        self.union_builder = self.union_builder.union(subquery, bindings);
        self
    }

    /// 构建 SQL
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    pub fn build(&self) -> (String, Vec<Value>) {
        self.union_builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_union_builder() {
        let builder = UnionBuilder::new("SELECT id FROM users", DatabaseType::MySQL)
            .union("SELECT id FROM admins", Vec::new());

        let (sql, _) = builder.build();
        assert!(sql.contains("UNION"));
        assert!(sql.contains("SELECT id FROM users"));
        assert!(sql.contains("SELECT id FROM admins"));
    }

    #[test]
    fn test_union_all() {
        let builder = UnionBuilder::new("SELECT id FROM users", DatabaseType::MySQL)
            .union_all("SELECT id FROM admins", Vec::new());

        let (sql, _) = builder.build();
        assert!(sql.contains("UNION ALL"));
    }

    #[test]
    fn test_union_with_order_limit() {
        let builder = UnionBuilder::new("SELECT id FROM users", DatabaseType::MySQL)
            .union("SELECT id FROM admins", Vec::new())
            .order_by("id DESC")
            .limit(10);

        let (sql, _) = builder.build();
        assert!(sql.contains("ORDER BY id DESC"));
        assert!(sql.contains("LIMIT 10"));
    }

    #[test]
    fn test_intersect_postgres() {
        let builder = UnionBuilder::new("SELECT id FROM users", DatabaseType::PostgreSQL)
            .intersect("SELECT id FROM admins", Vec::new());

        let (sql, _) = builder.build();
        assert!(sql.contains("INTERSECT"));
    }

    #[test]
    fn test_intersect_mysql_not_supported() {
        let builder = UnionBuilder::new("SELECT id FROM users", DatabaseType::MySQL)
            .intersect("SELECT id FROM admins", Vec::new());

        let (sql, _) = builder.build();
        // MySQL 不支持 INTERSECT，所以不应该包含
        assert!(!sql.contains("INTERSECT"));
    }

    #[test]
    fn test_except_postgres() {
        let builder = UnionBuilder::new("SELECT id FROM users", DatabaseType::PostgreSQL)
            .except("SELECT id FROM banned_users", Vec::new());

        let (sql, _) = builder.build();
        assert!(sql.contains("EXCEPT"));
    }

    #[test]
    fn test_union_factory() {
        let factory = UnionFactory::new(DatabaseType::MySQL);
        let builder = factory.union("SELECT id FROM users", "SELECT id FROM admins");

        let (sql, _) = builder.build();
        assert!(sql.contains("UNION"));
    }

    #[test]
    fn test_union_multiple() {
        let factory = UnionFactory::new(DatabaseType::MySQL);
        let builder = factory.union_multiple(&[
            "SELECT id FROM users",
            "SELECT id FROM admins",
            "SELECT id FROM moderators",
        ]);

        let (sql, _) = builder.build();
        assert!(sql.contains("SELECT id FROM users"));
        assert!(sql.contains("SELECT id FROM admins"));
        assert!(sql.contains("SELECT id FROM moderators"));
    }
}
