//! 数据库查询构建器模块
//!
//! 提供 ThinkPHP 风格的链式查询构建器
//! 支持 select/where/order/limit/group/having 等

use crate::interpreter::value::Value;
use std::collections::HashMap;

/// 查询构建器
/// 对应 ThinkPHP 8.0 的 \oyta\db\Query
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    /// 表名（含前缀）
    pub table: String,
    /// 查询字段
    pub fields: Vec<String>,
    /// WHERE 条件
    pub wheres: Vec<WhereClause>,
    /// ORDER BY
    pub orders: Vec<OrderClause>,
    /// LIMIT
    pub limit_val: Option<u64>,
    /// OFFSET
    pub offset_val: Option<u64>,
    /// GROUP BY
    pub groups: Vec<String>,
    /// HAVING
    pub havings: Vec<WhereClause>,
    /// JOIN
    pub joins: Vec<JoinClause>,
    /// 参数绑定
    pub bindings: Vec<Value>,
}

/// WHERE 条件子句
#[derive(Debug, Clone)]
pub struct WhereClause {
    /// 字段名
    pub field: String,
    /// 操作符（=, >, <, >=, <=, <>, LIKE, IN, BETWEEN 等）
    pub operator: String,
    /// 值
    pub value: WhereValue,
    /// 逻辑连接符（AND / OR）
    pub connector: String,
}

/// WHERE 值类型
#[derive(Debug, Clone)]
pub enum WhereValue {
    /// 单个值
    Single(Value),
    /// 多个值（用于 IN 操作符）
    Multiple(Vec<Value>),
    /// 两个值（用于 BETWEEN 操作符）
    Between(Value, Value),
    /// 原始表达式
    Raw(String),
}

/// ORDER BY 子句
#[derive(Debug, Clone)]
pub struct OrderClause {
    /// 字段名
    pub field: String,
    /// 排序方向（ASC / DESC）
    pub direction: String,
}

/// JOIN 子句
#[derive(Debug, Clone)]
pub struct JoinClause {
    /// JOIN 类型（INNER, LEFT, RIGHT, CROSS）
    pub join_type: String,
    /// 表名
    pub table: String,
    /// ON 条件
    pub on: String,
}

impl QueryBuilder {
    /// 创建新的查询构建器
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            fields: vec!["*".to_string()],
            wheres: Vec::new(),
            orders: Vec::new(),
            limit_val: None,
            offset_val: None,
            groups: Vec::new(),
            havings: Vec::new(),
            joins: Vec::new(),
            bindings: Vec::new(),
        }
    }

    /// 设置查询字段
    /// 如: .field("id, name, email")
    /// 或: .field(&["id", "name", "email"])
    pub fn field(mut self, fields: &str) -> Self {
        self.fields = fields.split(',')
            .map(|f| f.trim().to_string())
            .filter(|f| !f.is_empty())
            .collect();
        self
    }

    /// 添加 WHERE 条件（AND 连接）
    pub fn where_clause(mut self, field: &str, op: &str, value: Value) -> Self {
        self.wheres.push(WhereClause {
            field: field.to_string(),
            operator: op.to_string(),
            value: WhereValue::Single(value),
            connector: "AND".to_string(),
        });
        self
    }

    /// 添加 WHERE 条件（OR 连接）
    pub fn where_or(mut self, field: &str, op: &str, value: Value) -> Self {
        self.wheres.push(WhereClause {
            field: field.to_string(),
            operator: op.to_string(),
            value: WhereValue::Single(value),
            connector: "OR".to_string(),
        });
        self
    }

    /// WHERE IN 条件
    pub fn where_in(mut self, field: &str, values: Vec<Value>) -> Self {
        self.wheres.push(WhereClause {
            field: field.to_string(),
            operator: "IN".to_string(),
            value: WhereValue::Multiple(values),
            connector: "AND".to_string(),
        });
        self
    }

    /// WHERE BETWEEN 条件
    pub fn where_between(mut self, field: &str, start: Value, end: Value) -> Self {
        self.wheres.push(WhereClause {
            field: field.to_string(),
            operator: "BETWEEN".to_string(),
            value: WhereValue::Between(start, end),
            connector: "AND".to_string(),
        });
        self
    }

    /// WHERE LIKE 条件
    pub fn where_like(mut self, field: &str, pattern: &str) -> Self {
        self.wheres.push(WhereClause {
            field: field.to_string(),
            operator: "LIKE".to_string(),
            value: WhereValue::Single(Value::String(format!("%{}%", pattern))),
            connector: "AND".to_string(),
        });
        self
    }

    /// 添加 ORDER BY
    pub fn order(mut self, field: &str, direction: &str) -> Self {
        self.orders.push(OrderClause {
            field: field.to_string(),
            direction: direction.to_uppercase(),
        });
        self
    }

    /// 设置 LIMIT
    pub fn limit(mut self, count: u64) -> Self {
        self.limit_val = Some(count);
        self
    }

    /// 设置 OFFSET
    pub fn offset(mut self, count: u64) -> Self {
        self.offset_val = Some(count);
        self
    }

    /// 分页查询
    pub fn page(mut self, page: u64, page_size: u64) -> Self {
        self.limit_val = Some(page_size);
        self.offset_val = Some((page - 1) * page_size);
        self
    }

    /// 添加 GROUP BY
    pub fn group(mut self, field: &str) -> Self {
        self.groups.push(field.to_string());
        self
    }

    /// 添加 INNER JOIN
    pub fn join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(JoinClause {
            join_type: "INNER JOIN".to_string(),
            table: table.to_string(),
            on: on.to_string(),
        });
        self
    }

    /// 添加 LEFT JOIN
    pub fn left_join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(JoinClause {
            join_type: "LEFT JOIN".to_string(),
            table: table.to_string(),
            on: on.to_string(),
        });
        self
    }

    /// 构建 SELECT SQL
    pub fn build_select_sql(&self) -> String {
        let mut sql = String::new();

        // SELECT 字段
        sql.push_str("SELECT ");
        sql.push_str(&self.fields.join(", "));

        // FROM 表
        sql.push_str(" FROM ");
        sql.push_str(&self.table);

        // JOIN
        for join in &self.joins {
            sql.push_str(&format!(" {} {} ON {}", join.join_type, join.table, join.on));
        }

        // WHERE
        if !self.wheres.is_empty() {
            sql.push_str(" WHERE ");
            let mut first = true;
            for w in &self.wheres {
                if !first {
                    sql.push_str(&format!(" {} ", w.connector));
                }
                first = false;
                match &w.value {
                    WhereValue::Single(v) => {
                        sql.push_str(&format!("{} {} {}", w.field, w.operator, value_to_sql(v)));
                    }
                    WhereValue::Multiple(vals) => {
                        let items: Vec<String> = vals.iter().map(value_to_sql).collect();
                        sql.push_str(&format!("{} IN ({})", w.field, items.join(", ")));
                    }
                    WhereValue::Between(a, b) => {
                        sql.push_str(&format!("{} BETWEEN {} AND {}", w.field, value_to_sql(a), value_to_sql(b)));
                    }
                    WhereValue::Raw(expr) => {
                        sql.push_str(&format!("{} {} {}", w.field, w.operator, expr));
                    }
                }
            }
        }

        // GROUP BY
        if !self.groups.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.groups.join(", "));
        }

        // ORDER BY
        if !self.orders.is_empty() {
            sql.push_str(" ORDER BY ");
            let order_parts: Vec<String> = self.orders.iter()
                .map(|o| format!("{} {}", o.field, o.direction))
                .collect();
            sql.push_str(&order_parts.join(", "));
        }

        // LIMIT
        if let Some(limit) = self.limit_val {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET
        if let Some(offset) = self.offset_val {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        sql
    }

    /// 构建 INSERT SQL
    pub fn build_insert_sql(&self, data: &HashMap<String, Value>) -> String {
        let fields: Vec<&str> = data.keys().map(|s| s.as_str()).collect();
        let values: Vec<String> = data.values().map(value_to_sql).collect();

        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table,
            fields.join(", "),
            values.join(", ")
        )
    }

    /// 构建 UPDATE SQL
    pub fn build_update_sql(&self, data: &HashMap<String, Value>) -> String {
        let sets: Vec<String> = data.iter()
            .map(|(k, v)| format!("{} = {}", k, value_to_sql(v)))
            .collect();

        let mut sql = format!("UPDATE {} SET {}", self.table, sets.join(", "));

        if !self.wheres.is_empty() {
            sql.push_str(" WHERE ");
            let mut first = true;
            for w in &self.wheres {
                if !first { sql.push_str(&format!(" {} ", w.connector)); }
                first = false;
                if let WhereValue::Single(v) = &w.value {
                    sql.push_str(&format!("{} {} {}", w.field, w.operator, value_to_sql(v)));
                }
            }
        }

        sql
    }

    /// 构建 DELETE SQL
    pub fn build_delete_sql(&self) -> String {
        let mut sql = format!("DELETE FROM {}", self.table);

        if !self.wheres.is_empty() {
            sql.push_str(" WHERE ");
            let mut first = true;
            for w in &self.wheres {
                if !first { sql.push_str(&format!(" {} ", w.connector)); }
                first = false;
                if let WhereValue::Single(v) = &w.value {
                    sql.push_str(&format!("{} {} {}", w.field, w.operator, value_to_sql(v)));
                }
            }
        }

        sql
    }
}

/// 将 Value 转换为 SQL 字面量
fn value_to_sql(v: &Value) -> String {
    match v {
        Value::Null => "NULL".to_string(),
        Value::Bool(b) => if *b { "1" } else { "0" }.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => format!("'{}'", s.replace('\'', "''")),
        _ => "NULL".to_string(),
    }
}
