//! 更新操作模块
//!
//! 提供数据库更新操作功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：基本更新、字段自增/自减、表达式更新、批量更新等

use crate::interpreter::value::Value;
use std::collections::HashMap;

/// 更新构建器
///
/// 提供链式调用的更新构建方法
#[derive(Debug, Clone)]
pub struct UpdateBuilder {
    /// 表名
    pub table: String,
    /// 更新数据
    pub data: HashMap<String, UpdateValue>,
    /// WHERE 条件
    pub wheres: Vec<WhereClause>,
    /// ORDER BY
    pub order_by: Option<String>,
    /// LIMIT
    pub limit: Option<u64>,
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

/// 更新值类型
#[derive(Debug, Clone)]
pub enum UpdateValue {
    /// 普通值
    Value(Value),
    /// 自增
    Increment(i64),
    /// 自减
    Decrement(i64),
    /// 表达式
    Expression(String),
    /// 原始 SQL
    Raw(String),
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

impl UpdateBuilder {
    /// 创建新的更新构建器
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的更新构建器实例
    pub fn new(table: &str, db_type: DatabaseType) -> Self {
        Self {
            table: table.to_string(),
            data: HashMap::new(),
            wheres: Vec::new(),
            order_by: None,
            limit: None,
            db_type,
        }
    }

    /// 设置更新字段值
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `value`: 值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn set(mut self, field: &str, value: Value) -> Self {
        self.data.insert(field.to_string(), UpdateValue::Value(value));
        self
    }

    /// 批量设置更新字段
    ///
    /// # 参数
    /// - `data`: 数据键值对
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn sets(mut self, data: HashMap<String, Value>) -> Self {
        for (field, value) in data {
            self.data.insert(field, UpdateValue::Value(value));
        }
        self
    }

    /// 设置字段自增
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `amount`: 增量（默认为 1）
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn inc(mut self, field: &str, amount: i64) -> Self {
        self.data.insert(field.to_string(), UpdateValue::Increment(amount));
        self
    }

    /// 设置字段自减
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `amount`: 减量（默认为 1）
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn dec(mut self, field: &str, amount: i64) -> Self {
        self.data.insert(field.to_string(), UpdateValue::Decrement(amount));
        self
    }

    /// 设置表达式更新
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `expression`: 表达式
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn exp(mut self, field: &str, expression: &str) -> Self {
        self.data.insert(field.to_string(), UpdateValue::Expression(expression.to_string()));
        self
    }

    /// 设置原始 SQL 更新
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `sql`: 原始 SQL
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn raw(mut self, field: &str, sql: &str) -> Self {
        self.data.insert(field.to_string(), UpdateValue::Raw(sql.to_string()));
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

    /// 构建更新 SQL
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    pub fn build(&self) -> (String, Vec<Value>) {
        if self.data.is_empty() {
            return (String::new(), Vec::new());
        }

        let mut sql = String::new();
        let mut bindings = Vec::new();

        sql.push_str(&format!("UPDATE {} SET ", self.quote_identifier(&self.table)));

        // 构建 SET 子句
        let sets: Vec<String> = self.data.iter().map(|(field, value)| {
            match value {
                UpdateValue::Value(v) => {
                    bindings.push(v.clone());
                    format!("{} = ?", self.quote_identifier(field))
                }
                UpdateValue::Increment(amount) => {
                    format!("{} = {} + {}", self.quote_identifier(field), self.quote_identifier(field), amount)
                }
                UpdateValue::Decrement(amount) => {
                    format!("{} = {} - {}", self.quote_identifier(field), self.quote_identifier(field), amount)
                }
                UpdateValue::Expression(exp) => {
                    format!("{} = {}", self.quote_identifier(field), exp)
                }
                UpdateValue::Raw(raw) => {
                    format!("{} = {}", self.quote_identifier(field), raw)
                }
            }
        }).collect();

        sql.push_str(&sets.join(", "));

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

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }
}

/// 更新结果
///
/// 存储更新操作的结果
#[derive(Debug, Clone)]
pub struct UpdateResult {
    /// 影响的行数
    pub affected_rows: u64,
}

impl UpdateResult {
    /// 创建新的更新结果
    ///
    /// # 参数
    /// - `affected_rows`: 影响的行数
    ///
    /// # 返回
    /// 新的更新结果实例
    pub fn new(affected_rows: u64) -> Self {
        Self { affected_rows }
    }
}

/// 批量更新构建器
///
/// 用于构建批量更新多条记录的 SQL
#[derive(Debug, Clone)]
pub struct BatchUpdateBuilder {
    /// 表名
    pub table: String,
    /// 主键字段名
    pub key_field: String,
    /// 更新字段列表
    pub update_fields: Vec<String>,
    /// 更新数据列表
    pub data: Vec<HashMap<String, Value>>,
    /// 数据库类型
    pub db_type: DatabaseType,
}

impl BatchUpdateBuilder {
    /// 创建新的批量更新构建器
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `key_field`: 主键字段名
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的批量更新构建器实例
    pub fn new(table: &str, key_field: &str, db_type: DatabaseType) -> Self {
        Self {
            table: table.to_string(),
            key_field: key_field.to_string(),
            update_fields: Vec::new(),
            data: Vec::new(),
            db_type,
        }
    }

    /// 设置更新字段
    ///
    /// # 参数
    /// - `fields`: 字段名列表
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn fields(mut self, fields: &[&str]) -> Self {
        self.update_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 添加更新数据
    ///
    /// # 参数
    /// - `row`: 数据行
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn add_row(mut self, row: HashMap<String, Value>) -> Self {
        self.data.push(row);
        self
    }

    /// 批量设置更新数据
    ///
    /// # 参数
    /// - `data`: 数据列表
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn data(mut self, data: Vec<HashMap<String, Value>>) -> Self {
        self.data = data;
        self
    }

    /// 构建批量更新 SQL
    ///
    /// 使用 CASE WHEN 语法实现批量更新
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    pub fn build(&self) -> (String, Vec<Value>) {
        if self.data.is_empty() || self.update_fields.is_empty() {
            return (String::new(), Vec::new());
        }

        let mut sql = String::new();
        let mut bindings = Vec::new();

        sql.push_str(&format!("UPDATE {} SET ", self.quote_identifier(&self.table)));

        // 构建 SET 子句（使用 CASE WHEN）
        let sets: Vec<String> = self.update_fields.iter().map(|field| {
            let mut cases = Vec::new();
            for row in &self.data {
                if let (Some(key_value), Some(field_value)) = (row.get(&self.key_field), row.get(field)) {
                    cases.push(format!("WHEN {} = ? THEN ?", self.quote_identifier(&self.key_field)));
                    bindings.push(key_value.clone());
                    bindings.push(field_value.clone());
                }
            }
            format!("{} = CASE {} ELSE {} END",
                self.quote_identifier(field),
                cases.join(" "),
                self.quote_identifier(field)
            )
        }).collect();

        sql.push_str(&sets.join(", "));

        // 构建 WHERE IN 子句
        let key_values: Vec<String> = self.data.iter()
            .filter_map(|row| row.get(&self.key_field).map(|_| "?".to_string()))
            .collect();
        
        for row in &self.data {
            if let Some(key_value) = row.get(&self.key_field) {
                bindings.push(key_value.clone());
            }
        }

        sql.push_str(&format!(" WHERE {} IN ({})",
            self.quote_identifier(&self.key_field),
            key_values.join(", ")
        ));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_builder() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), Value::String("updated".to_string()));

        let builder = UpdateBuilder::new("users", DatabaseType::MySQL)
            .sets(data)
            .where_eq("id", Value::Int(1));

        let (sql, bindings) = builder.build();
        assert!(sql.contains("UPDATE"));
        assert!(sql.contains("SET"));
        assert!(sql.contains("WHERE"));
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_update_increment() {
        let builder = UpdateBuilder::new("users", DatabaseType::MySQL)
            .inc("views", 1)
            .where_eq("id", Value::Int(1));

        let (sql, _) = builder.build();
        // MySQL 使用反引号引用标识符
        assert!(sql.contains("`views` = `views` + 1"));
    }

    #[test]
    fn test_update_decrement() {
        let builder = UpdateBuilder::new("users", DatabaseType::MySQL)
            .dec("stock", 5)
            .where_eq("id", Value::Int(1));

        let (sql, _) = builder.build();
        // MySQL 使用反引号引用标识符
        assert!(sql.contains("`stock` = `stock` - 5"));
    }

    #[test]
    fn test_update_expression() {
        let builder = UpdateBuilder::new("users", DatabaseType::MySQL)
            .exp("updated_at", "NOW()")
            .where_eq("id", Value::Int(1));

        let (sql, _) = builder.build();
        // MySQL 使用反引号引用标识符
        assert!(sql.contains("`updated_at` = NOW()"));
    }

    #[test]
    fn test_update_with_limit() {
        let builder = UpdateBuilder::new("users", DatabaseType::MySQL)
            .set("status", Value::Int(0))
            .limit(10);

        let (sql, _) = builder.build();
        assert!(sql.contains("LIMIT 10"));
    }
}
