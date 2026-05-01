//! 删除操作模块
//!
//! 提供数据库删除操作功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：基本删除、条件删除、软删除、批量删除等

use crate::interpreter::value::Value;
use std::collections::HashMap;

/// 删除构建器
///
/// 提供链式调用的删除构建方法
#[derive(Debug, Clone)]
pub struct DeleteBuilder {
    /// 表名
    pub table: String,
    /// WHERE 条件
    pub wheres: Vec<WhereClause>,
    /// ORDER BY
    pub order_by: Option<String>,
    /// LIMIT
    pub limit: Option<u64>,
    /// 是否软删除
    pub is_soft_delete: bool,
    /// 软删除字段名
    pub soft_delete_field: Option<String>,
    /// 软删除值
    pub soft_delete_value: Option<Value>,
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

impl DeleteBuilder {
    /// 创建新的删除构建器
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的删除构建器实例
    pub fn new(table: &str, db_type: DatabaseType) -> Self {
        Self {
            table: table.to_string(),
            wheres: Vec::new(),
            order_by: None,
            limit: None,
            is_soft_delete: false,
            soft_delete_field: None,
            soft_delete_value: None,
            db_type,
        }
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

    /// 添加 WHERE IN 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `values`: 值列表
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn where_in(mut self, field: &str, values: Vec<Value>) -> Self {
        let placeholders: Vec<String> = values.iter().map(|_| "?".to_string()).collect();
        self.wheres.push(WhereClause {
            condition: format!("{} IN ({})", self.quote_identifier(field), placeholders.join(", ")),
            bindings: values,
            connector: "AND".to_string(),
        });
        self
    }

    /// 添加 OR WHERE 条件
    ///
    /// # 参数
    /// - `condition`: 条件 SQL
    /// - `bindings`: 绑定参数
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn or_where_raw(mut self, condition: &str, bindings: Vec<Value>) -> Self {
        self.wheres.push(WhereClause {
            condition: condition.to_string(),
            bindings,
            connector: "OR".to_string(),
        });
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

    /// 设置为软删除
    ///
    /// # 参数
    /// - `field`: 软删除字段名
    /// - `value`: 软删除标记值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn soft_delete(mut self, field: &str, value: Value) -> Self {
        self.is_soft_delete = true;
        self.soft_delete_field = Some(field.to_string());
        self.soft_delete_value = Some(value);
        self
    }

    /// 构建删除 SQL
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    pub fn build(&self) -> (String, Vec<Value>) {
        // 如果是软删除，构建 UPDATE SQL
        if self.is_soft_delete {
            return self.build_soft_delete();
        }

        let mut sql = String::new();
        let mut bindings = Vec::new();

        sql.push_str(&format!("DELETE FROM {}", self.quote_identifier(&self.table)));

        // 构建 WHERE 子句
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

        // 构建 ORDER BY
        if let Some(order_by) = &self.order_by {
            sql.push_str(&format!(" ORDER BY {}", order_by));
        }

        // 构建 LIMIT
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        (sql, bindings)
    }

    /// 构建软删除 SQL（实际是 UPDATE）
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    fn build_soft_delete(&self) -> (String, Vec<Value>) {
        let mut sql = String::new();
        let mut bindings = Vec::new();

        sql.push_str(&format!("UPDATE {} SET ", self.quote_identifier(&self.table)));

        if let (Some(field), Some(value)) = (&self.soft_delete_field, &self.soft_delete_value) {
            sql.push_str(&format!("{} = ?", self.quote_identifier(field)));
            bindings.push(value.clone());
        }

        // 构建 WHERE 子句
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

        (sql, bindings)
    }

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }
}

/// 删除结果
///
/// 存储删除操作的结果
#[derive(Debug, Clone)]
pub struct DeleteResult {
    /// 影响的行数
    pub affected_rows: u64,
}

impl DeleteResult {
    /// 创建新的删除结果
    ///
    /// # 参数
    /// - `affected_rows`: 影响的行数
    ///
    /// # 返回
    /// 新的删除结果实例
    pub fn new(affected_rows: u64) -> Self {
        Self { affected_rows }
    }
}

/// 销毁构建器
///
/// 用于根据 ID 或条件快速删除记录
#[derive(Debug, Clone)]
pub struct DestroyBuilder {
    /// 表名
    pub table: String,
    /// 主键字段名
    pub key_field: String,
    /// 要删除的 ID 列表
    pub ids: Vec<Value>,
    /// 数据库类型
    pub db_type: DatabaseType,
}

impl DestroyBuilder {
    /// 创建新的销毁构建器
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的销毁构建器实例
    pub fn new(table: &str, db_type: DatabaseType) -> Self {
        Self {
            table: table.to_string(),
            key_field: "id".to_string(),
            ids: Vec::new(),
            db_type,
        }
    }

    /// 设置主键字段名
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn key_field(mut self, field: &str) -> Self {
        self.key_field = field.to_string();
        self
    }

    /// 添加要删除的 ID
    ///
    /// # 参数
    /// - `id`: ID 值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn id(mut self, id: Value) -> Self {
        self.ids.push(id);
        self
    }

    /// 批量添加要删除的 ID
    ///
    /// # 参数
    /// - `ids`: ID 值列表
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn ids(mut self, ids: Vec<Value>) -> Self {
        self.ids.extend(ids);
        self
    }

    /// 构建删除 SQL
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    pub fn build(&self) -> (String, Vec<Value>) {
        if self.ids.is_empty() {
            return (String::new(), Vec::new());
        }

        let placeholders: Vec<String> = self.ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "DELETE FROM {} WHERE {} IN ({})",
            self.quote_identifier(&self.table),
            self.quote_identifier(&self.key_field),
            placeholders.join(", ")
        );

        (sql, self.ids.clone())
    }

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }
}

/// 软删除配置
///
/// 配置软删除行为
#[derive(Debug, Clone)]
pub struct SoftDeleteConfig {
    /// 软删除字段名
    pub field: String,
    /// 软删除标记值
    pub deleted_value: Value,
    /// 未删除标记值
    pub not_deleted_value: Value,
}

impl Default for SoftDeleteConfig {
    fn default() -> Self {
        Self {
            field: "deleted_at".to_string(),
            deleted_value: Value::Null,
            not_deleted_value: Value::Null,
        }
    }
}

impl SoftDeleteConfig {
    /// 创建新的软删除配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置软删除字段名
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 修改后的配置
    pub fn field(mut self, field: &str) -> Self {
        self.field = field.to_string();
        self
    }

    /// 设置软删除标记值
    ///
    /// # 参数
    /// - `value`: 标记值
    ///
    /// # 返回
    /// 修改后的配置
    pub fn deleted_value(mut self, value: Value) -> Self {
        self.deleted_value = value;
        self
    }

    /// 构建查询排除软删除记录的 WHERE 条件
    ///
    /// # 返回
    /// WHERE 条件 SQL
    pub fn build_not_deleted_condition(&self) -> String {
        match &self.not_deleted_value {
            Value::Null => format!("{} IS NULL", self.field),
            _ => format!("{} = ?", self.field),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_builder() {
        let builder = DeleteBuilder::new("users", DatabaseType::MySQL)
            .where_eq("id", Value::Int(1));

        let (sql, bindings) = builder.build();
        assert!(sql.contains("DELETE FROM"));
        assert!(sql.contains("WHERE"));
        assert_eq!(bindings.len(), 1);
    }

    #[test]
    fn test_delete_with_where_in() {
        let builder = DeleteBuilder::new("users", DatabaseType::MySQL)
            .where_in("id", vec![Value::Int(1), Value::Int(2), Value::Int(3)]);

        let (sql, bindings) = builder.build();
        assert!(sql.contains("IN"));
        assert_eq!(bindings.len(), 3);
    }

    #[test]
    fn test_delete_with_limit() {
        let builder = DeleteBuilder::new("logs", DatabaseType::MySQL)
            .where_raw("created_at < ?", vec![Value::String("2024-01-01".to_string())])
            .order_by("id ASC")
            .limit(1000);

        let (sql, _) = builder.build();
        assert!(sql.contains("ORDER BY"));
        assert!(sql.contains("LIMIT 1000"));
    }

    #[test]
    fn test_soft_delete() {
        let builder = DeleteBuilder::new("users", DatabaseType::MySQL)
            .where_eq("id", Value::Int(1))
            .soft_delete("deleted_at", Value::String("2024-01-01 00:00:00".to_string()));

        let (sql, _) = builder.build();
        assert!(sql.contains("UPDATE"));
        assert!(sql.contains("deleted_at"));
    }

    #[test]
    fn test_destroy_builder() {
        let builder = DestroyBuilder::new("users", DatabaseType::MySQL)
            .ids(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);

        let (sql, bindings) = builder.build();
        assert!(sql.contains("DELETE FROM"));
        assert!(sql.contains("IN"));
        assert_eq!(bindings.len(), 3);
    }
}
