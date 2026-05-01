//! HAVING 条件模块
//!
//! 提供数据库 HAVING 条件构建功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：HAVING 条件、HAVING RAW、HAVING BETWEEN 等

use crate::interpreter::value::Value;

/// HAVING 条件类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum HavingType {
    /// 基本 HAVING 条件
    Basic,
    /// 原始 SQL 条件
    Raw,
    /// BETWEEN 条件
    Between,
    /// NOT BETWEEN 条件
    NotBetween,
    /// IN 条件
    In,
    /// NOT IN 条件
    NotIn,
    /// NULL 条件
    Null,
    /// NOT NULL 条件
    NotNull,
}

/// HAVING 条件子句结构体
///
/// 存储单个 HAVING 条件的完整信息
#[derive(Debug, Clone)]
pub struct HavingClause {
    /// 条件类型
    pub having_type: HavingType,
    /// 字段名或表达式
    pub field: String,
    /// 操作符
    pub operator: String,
    /// 条件值
    pub value: HavingValue,
    /// 逻辑连接符
    pub connector: String,
}

impl HavingClause {
    /// 创建新的 HAVING 条件子句
    ///
    /// # 参数
    /// - `having_type`: 条件类型
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `value`: 条件值
    /// - `connector`: 逻辑连接符
    ///
    /// # 返回
    /// 新的 HAVING 条件子句实例
    pub fn new(
        having_type: HavingType,
        field: &str,
        operator: &str,
        value: HavingValue,
        connector: &str,
    ) -> Self {
        Self {
            having_type,
            field: field.to_string(),
            operator: operator.to_string(),
            value,
            connector: connector.to_string(),
        }
    }
}

/// HAVING 条件值枚举
#[derive(Debug, Clone)]
pub enum HavingValue {
    /// 单个值
    Single(Value),
    /// 多个值
    Multiple(Vec<Value>),
    /// 两个值（BETWEEN）
    Between(Value, Value),
    /// 原始 SQL
    Raw(String),
    /// 无值
    None,
}

/// HAVING 条件构建器
///
/// 提供链式调用的 HAVING 条件构建方法
#[derive(Debug, Clone)]
pub struct HavingBuilder {
    /// HAVING 条件列表
    pub havings: Vec<HavingClause>,
    /// 绑定参数列表
    pub bindings: Vec<Value>,
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

impl HavingBuilder {
    /// 创建新的 HAVING 条件构建器
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的 HAVING 条件构建器实例
    pub fn new(db_type: DatabaseType) -> Self {
        Self {
            havings: Vec::new(),
            bindings: Vec::new(),
            db_type,
        }
    }

    /// 添加基本 HAVING 条件
    ///
    /// # 参数
    /// - `field`: 字段名或聚合表达式
    /// - `operator`: 操作符
    /// - `value`: 条件值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    ///
    /// # 示例
    /// ```ignore
    /// builder.having("COUNT(*)", ">", Value::Int(10));
    /// ```
    pub fn having(mut self, field: &str, operator: &str, value: Value) -> Self {
        self.havings.push(HavingClause::new(
            HavingType::Basic,
            field,
            operator,
            HavingValue::Single(value.clone()),
            "AND",
        ));
        self.bindings.push(value);
        self
    }

    /// 添加 OR HAVING 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `operator`: 操作符
    /// - `value`: 条件值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn or_having(mut self, field: &str, operator: &str, value: Value) -> Self {
        self.havings.push(HavingClause::new(
            HavingType::Basic,
            field,
            operator,
            HavingValue::Single(value.clone()),
            "OR",
        ));
        self.bindings.push(value);
        self
    }

    /// 添加原始 HAVING 条件
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
    /// builder.having_raw("SUM(amount) > ?", vec![Value::Int(1000)]);
    /// ```
    pub fn having_raw(mut self, sql: &str, bindings: Vec<Value>) -> Self {
        self.bindings.extend(bindings);
        self.havings.push(HavingClause::new(
            HavingType::Raw,
            sql,
            "",
            HavingValue::Raw(sql.to_string()),
            "AND",
        ));
        self
    }

    /// 添加 OR 原始 HAVING 条件
    ///
    /// # 参数
    /// - `sql`: 原始 SQL 条件
    /// - `bindings`: 绑定参数
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn or_having_raw(mut self, sql: &str, bindings: Vec<Value>) -> Self {
        self.bindings.extend(bindings);
        self.havings.push(HavingClause::new(
            HavingType::Raw,
            sql,
            "",
            HavingValue::Raw(sql.to_string()),
            "OR",
        ));
        self
    }

    /// 添加 HAVING BETWEEN 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `start`: 起始值
    /// - `end`: 结束值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn having_between(mut self, field: &str, start: Value, end: Value) -> Self {
        self.bindings.push(start.clone());
        self.bindings.push(end.clone());
        self.havings.push(HavingClause::new(
            HavingType::Between,
            field,
            "BETWEEN",
            HavingValue::Between(start, end),
            "AND",
        ));
        self
    }

    /// 添加 HAVING NOT BETWEEN 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `start`: 起始值
    /// - `end`: 结束值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn having_not_between(mut self, field: &str, start: Value, end: Value) -> Self {
        self.bindings.push(start.clone());
        self.bindings.push(end.clone());
        self.havings.push(HavingClause::new(
            HavingType::NotBetween,
            field,
            "NOT BETWEEN",
            HavingValue::Between(start, end),
            "AND",
        ));
        self
    }

    /// 添加 HAVING IN 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `values`: 值列表
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn having_in(mut self, field: &str, values: Vec<Value>) -> Self {
        self.bindings.extend(values.clone());
        self.havings.push(HavingClause::new(
            HavingType::In,
            field,
            "IN",
            HavingValue::Multiple(values),
            "AND",
        ));
        self
    }

    /// 添加 HAVING NOT IN 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `values`: 值列表
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn having_not_in(mut self, field: &str, values: Vec<Value>) -> Self {
        self.bindings.extend(values.clone());
        self.havings.push(HavingClause::new(
            HavingType::NotIn,
            field,
            "NOT IN",
            HavingValue::Multiple(values),
            "AND",
        ));
        self
    }

    /// 添加 HAVING IS NULL 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn having_null(mut self, field: &str) -> Self {
        self.havings.push(HavingClause::new(
            HavingType::Null,
            field,
            "IS NULL",
            HavingValue::None,
            "AND",
        ));
        self
    }

    /// 添加 HAVING IS NOT NULL 条件
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn having_not_null(mut self, field: &str) -> Self {
        self.havings.push(HavingClause::new(
            HavingType::NotNull,
            field,
            "IS NOT NULL",
            HavingValue::None,
            "AND",
        ));
        self
    }

    /// 构建完整的 HAVING 子句
    ///
    /// # 返回
    /// (HAVING 子句 SQL, 绑定参数列表)
    pub fn build(&self) -> (String, Vec<Value>) {
        if self.havings.is_empty() {
            return (String::new(), Vec::new());
        }

        let mut sql = String::new();
        let mut first = true;

        for clause in &self.havings {
            if !first {
                sql.push_str(&format!(" {} ", clause.connector));
            }
            first = false;

            let clause_sql = self.build_clause_sql(clause);
            sql.push_str(&clause_sql);
        }

        (format!("HAVING {}", sql), self.bindings.clone())
    }

    /// 构建单个条件 SQL
    ///
    /// # 参数
    /// - `clause`: HAVING 条件子句
    ///
    /// # 返回
    /// 条件 SQL 字符串
    fn build_clause_sql(&self, clause: &HavingClause) -> String {
        match &clause.having_type {
            HavingType::Basic => {
                format!("{} {} ?", clause.field, clause.operator)
            }
            HavingType::Raw => {
                if let HavingValue::Raw(sql) = &clause.value {
                    sql.clone()
                } else {
                    String::new()
                }
            }
            HavingType::Between | HavingType::NotBetween => {
                format!("{} {} ? AND ?", clause.field, clause.operator)
            }
            HavingType::In | HavingType::NotIn => {
                if let HavingValue::Multiple(values) = &clause.value {
                    let placeholders: Vec<String> = values.iter().map(|_| "?".to_string()).collect();
                    format!("{} {} ({})", clause.field, clause.operator, placeholders.join(", "))
                } else {
                    String::new()
                }
            }
            HavingType::Null => {
                format!("{} IS NULL", clause.field)
            }
            HavingType::NotNull => {
                format!("{} IS NOT NULL", clause.field)
            }
        }
    }
}

/// HAVING 条件组
///
/// 用于构建复杂的 HAVING 条件组合
#[derive(Debug, Clone)]
pub struct HavingGroup {
    /// 条件组内的条件列表
    pub clauses: Vec<HavingClause>,
    /// 条件组的逻辑连接符
    pub connector: String,
}

impl HavingGroup {
    /// 创建新的条件组
    ///
    /// # 参数
    /// - `connector`: 逻辑连接符
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
    /// - `clause`: HAVING 条件子句
    pub fn add(&mut self, clause: HavingClause) {
        self.clauses.push(clause);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_having_basic() {
        let builder = HavingBuilder::new(DatabaseType::MySQL)
            .having("COUNT(*)", ">", Value::Int(10));

        let (sql, bindings) = builder.build();
        assert!(sql.contains("HAVING"));
        assert!(sql.contains("COUNT(*) > ?"));
        assert_eq!(bindings.len(), 1);
    }

    #[test]
    fn test_having_raw() {
        let builder = HavingBuilder::new(DatabaseType::MySQL)
            .having_raw("SUM(amount) > ?", vec![Value::Int(1000)]);

        let (sql, bindings) = builder.build();
        assert!(sql.contains("SUM(amount) > ?"));
        assert_eq!(bindings.len(), 1);
    }

    #[test]
    fn test_having_between() {
        let builder = HavingBuilder::new(DatabaseType::MySQL)
            .having_between("COUNT(*)", Value::Int(5), Value::Int(10));

        let (sql, bindings) = builder.build();
        assert!(sql.contains("BETWEEN ? AND ?"));
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_having_in() {
        let builder = HavingBuilder::new(DatabaseType::MySQL)
            .having_in("status", vec![Value::Int(1), Value::Int(2), Value::Int(3)]);

        let (sql, bindings) = builder.build();
        assert!(sql.contains("IN (?, ?, ?)"));
        assert_eq!(bindings.len(), 3);
    }

    #[test]
    fn test_or_having() {
        let builder = HavingBuilder::new(DatabaseType::MySQL)
            .having("COUNT(*)", ">", Value::Int(10))
            .or_having("SUM(amount)", ">", Value::Int(1000));

        let (sql, _) = builder.build();
        assert!(sql.contains("OR"));
    }

    #[test]
    fn test_having_null() {
        let builder = HavingBuilder::new(DatabaseType::MySQL)
            .having_null("deleted_at");

        let (sql, _) = builder.build();
        assert!(sql.contains("IS NULL"));
    }
}
