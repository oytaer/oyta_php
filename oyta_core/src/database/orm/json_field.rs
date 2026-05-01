//! JSON 字段操作模块
//!
//! 提供模型 JSON 字段操作功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：JSON 字段读取、写入、查询等

use crate::interpreter::value::Value;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// JSON 字段路径
#[derive(Debug, Clone)]
pub struct JsonPath {
    /// 路径片段
    pub segments: Vec<String>,
}

impl JsonPath {
    /// 创建新的 JSON 路径
    pub fn new(path: &str) -> Self {
        let segments = path.split('.').map(|s| s.to_string()).collect();
        Self { segments }
    }

    /// 获取路径深度
    pub fn depth(&self) -> usize {
        self.segments.len()
    }

    /// 构建 MySQL JSON 路径表达式
    pub fn to_mysql_path(&self) -> String {
        let mut path = String::from("$");
        for segment in &self.segments {
            if segment.parse::<usize>().is_ok() {
                path.push_str(&format!("[{}]", segment));
            } else {
                path.push_str(&format!(".{}", segment));
            }
        }
        path
    }

    /// 构建 PostgreSQL JSON 路径表达式
    pub fn to_postgres_path(&self) -> String {
        self.segments.iter()
            .map(|s| {
                if s.parse::<usize>().is_ok() {
                    format!("->{}", s)
                } else {
                    format!("->>'{}'", s)
                }
            })
            .collect()
    }

    /// 构建 SQLite JSON 路径表达式
    pub fn to_sqlite_path(&self) -> String {
        format!("'$.{}'", self.segments.join("."))
    }
}

/// JSON 字段操作器
pub struct JsonFieldOperator {
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

impl JsonFieldOperator {
    /// 创建新的 JSON 字段操作器
    pub fn new(db_type: DatabaseType) -> Self {
        Self { db_type }
    }

    /// 构建提取 JSON 字段值的 SQL
    pub fn extract(&self, field: &str, path: &JsonPath) -> String {
        match self.db_type {
            DatabaseType::MySQL => {
                format!("JSON_UNQUOTE(JSON_EXTRACT({}, '{}'))", 
                    self.quote_identifier(field),
                    path.to_mysql_path())
            }
            DatabaseType::PostgreSQL => {
                format!("{}{}", 
                    self.quote_identifier(field),
                    path.to_postgres_path())
            }
            DatabaseType::SQLite => {
                format!("json_extract({}, '{}')", 
                    self.quote_identifier(field),
                    path.to_sqlite_path())
            }
        }
    }

    /// 构建设置 JSON 字段值的 SQL
    pub fn set(&self, field: &str, path: &JsonPath, value: &JsonValue) -> String {
        let value_str = value.to_string();
        match self.db_type {
            DatabaseType::MySQL => {
                format!("JSON_SET({}, '{}', '{}')", 
                    self.quote_identifier(field),
                    path.to_mysql_path(),
                    value_str)
            }
            DatabaseType::PostgreSQL => {
                format!("jsonb_set({}::jsonb, '{}', '{}'::jsonb)", 
                    self.quote_identifier(field),
                    path.to_postgres_path(),
                    value_str)
            }
            DatabaseType::SQLite => {
                format!("json_set({}, '{}', '{}')", 
                    self.quote_identifier(field),
                    path.to_sqlite_path(),
                    value_str)
            }
        }
    }

    /// 构建删除 JSON 字段路径的 SQL
    pub fn remove(&self, field: &str, path: &JsonPath) -> String {
        match self.db_type {
            DatabaseType::MySQL => {
                format!("JSON_REMOVE({}, '{}')", 
                    self.quote_identifier(field),
                    path.to_mysql_path())
            }
            DatabaseType::PostgreSQL => {
                format!("{} #- '{}'", 
                    self.quote_identifier(field),
                    path.to_postgres_path())
            }
            DatabaseType::SQLite => {
                format!("json_remove({}, '{}')", 
                    self.quote_identifier(field),
                    path.to_sqlite_path())
            }
        }
    }

    /// 构建 JSON 包含查询 SQL
    pub fn contains(&self, field: &str, value: &JsonValue) -> String {
        let value_str = value.to_string();
        match self.db_type {
            DatabaseType::MySQL => {
                format!("JSON_CONTAINS({}, '{}')", 
                    self.quote_identifier(field),
                    value_str)
            }
            DatabaseType::PostgreSQL => {
                format!("{} @> '{}'::jsonb", 
                    self.quote_identifier(field),
                    value_str)
            }
            DatabaseType::SQLite => {
                format!("EXISTS (SELECT 1 FROM json_each({}) WHERE value = '{}')", 
                    self.quote_identifier(field),
                    value_str)
            }
        }
    }

    /// 构建 JSON 数组长度查询 SQL
    pub fn array_length(&self, field: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => {
                format!("JSON_LENGTH({})", self.quote_identifier(field))
            }
            DatabaseType::PostgreSQL => {
                format!("jsonb_array_length({})", self.quote_identifier(field))
            }
            DatabaseType::SQLite => {
                format!("json_array_length({})", self.quote_identifier(field))
            }
        }
    }

    /// 构建 JSON 类型查询 SQL
    pub fn json_type(&self, field: &str, path: &JsonPath) -> String {
        match self.db_type {
            DatabaseType::MySQL => {
                format!("JSON_TYPE({}, '{}')", 
                    self.quote_identifier(field),
                    path.to_mysql_path())
            }
            DatabaseType::PostgreSQL => {
                format!("jsonb_typeof({}{})", 
                    self.quote_identifier(field),
                    path.to_postgres_path())
            }
            DatabaseType::SQLite => {
                format!("json_type({}, '{}')", 
                    self.quote_identifier(field),
                    path.to_sqlite_path())
            }
        }
    }

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }
}

/// JSON 字段访问器
pub struct JsonFieldAccessor {
    /// 字段名
    pub field: String,
    /// JSON 值
    pub value: Option<JsonValue>,
}

impl JsonFieldAccessor {
    /// 创建新的 JSON 字段访问器
    pub fn new(field: &str) -> Self {
        Self {
            field: field.to_string(),
            value: None,
        }
    }

    /// 从 JSON 值创建
    pub fn from_json(field: &str, json: JsonValue) -> Self {
        Self {
            field: field.to_string(),
            value: Some(json),
        }
    }

    /// 获取路径值
    pub fn get(&self, path: &str) -> Option<&JsonValue> {
        let path = JsonPath::new(path);
        let mut current = self.value.as_ref()?;

        for segment in &path.segments {
            match current {
                JsonValue::Object(map) => {
                    current = map.get(segment)?;
                }
                JsonValue::Array(arr) => {
                    let index: usize = segment.parse().ok()?;
                    current = arr.get(index)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }

    /// 设置路径值
    pub fn set(&mut self, path: &str, value: JsonValue) {
        if self.value.is_none() {
            self.value = Some(JsonValue::Object(HashMap::new().into_iter().collect()));
        }

        // 简化实现：只支持一级路径
        if let Some(JsonValue::Object(ref mut map)) = self.value {
            map.insert(path.to_string(), value);
        }
    }

    /// 转换为 Value
    pub fn to_value(&self) -> Value {
        match &self.value {
            Some(json) => Value::String(json.to_string()),
            None => Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_path() {
        let path = JsonPath::new("user.profile.name");
        assert_eq!(path.depth(), 3);
    }

    #[test]
    fn test_json_field_operator_mysql() {
        let operator = JsonFieldOperator::new(DatabaseType::MySQL);
        let path = JsonPath::new("name");

        let sql = operator.extract("data", &path);
        assert!(sql.contains("JSON_EXTRACT"));
    }

    #[test]
    fn test_json_field_operator_postgres() {
        let operator = JsonFieldOperator::new(DatabaseType::PostgreSQL);
        let path = JsonPath::new("name");

        let sql = operator.extract("data", &path);
        assert!(sql.contains("->"));
    }

    #[test]
    fn test_json_contains() {
        let operator = JsonFieldOperator::new(DatabaseType::MySQL);
        let value = serde_json::json!({"tag": "php"});

        let sql = operator.contains("data", &value);
        assert!(sql.contains("JSON_CONTAINS"));
    }

    #[test]
    fn test_json_field_accessor() {
        let json = serde_json::json!({
            "name": "test",
            "age": 25
        });

        let accessor = JsonFieldAccessor::from_json("data", json);

        assert_eq!(accessor.get("name"), Some(&JsonValue::String("test".to_string())));
        assert_eq!(accessor.get("age"), Some(&JsonValue::Number(25.into())));
    }
}
