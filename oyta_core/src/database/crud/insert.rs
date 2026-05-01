//! 插入操作模块
//!
//! 提供数据库插入操作功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：单条插入、批量插入、插入返回ID、REPLACE INTO、UPSERT 等

use crate::interpreter::value::Value;
use std::collections::HashMap;

/// 插入构建器
///
/// 提供链式调用的插入构建方法
#[derive(Debug, Clone)]
pub struct InsertBuilder {
    /// 表名
    pub table: String,
    /// 插入数据
    pub data: Vec<HashMap<String, Value>>,
    /// 是否为 REPLACE INTO
    pub is_replace: bool,
    /// 是否忽略错误
    pub is_ignore: bool,
    /// 冲突时更新的字段（UPSERT）
    pub on_duplicate_key_update: Vec<String>,
    /// 冲突时更新的值
    pub on_duplicate_key_values: HashMap<String, Value>,
    /// 返回字段（PostgreSQL）
    pub returning: Vec<String>,
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

impl InsertBuilder {
    /// 创建新的插入构建器
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的插入构建器实例
    pub fn new(table: &str, db_type: DatabaseType) -> Self {
        Self {
            table: table.to_string(),
            data: Vec::new(),
            is_replace: false,
            is_ignore: false,
            on_duplicate_key_update: Vec::new(),
            on_duplicate_key_values: HashMap::new(),
            returning: Vec::new(),
            db_type,
        }
    }

    /// 设置单条插入数据
    ///
    /// # 参数
    /// - `data`: 数据键值对
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn values(mut self, data: HashMap<String, Value>) -> Self {
        self.data = vec![data];
        self
    }

    /// 设置批量插入数据
    ///
    /// # 参数
    /// - `data`: 数据列表
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn batch(mut self, data: Vec<HashMap<String, Value>>) -> Self {
        self.data = data;
        self
    }

    /// 设置为 REPLACE INTO
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn replace(mut self) -> Self {
        self.is_replace = true;
        self
    }

    /// 设置为 IGNORE 插入
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn ignore(mut self) -> Self {
        self.is_ignore = true;
        self
    }

    /// 设置 ON DUPLICATE KEY UPDATE（MySQL UPSERT）
    ///
    /// # 参数
    /// - `fields`: 更新字段列表
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn on_duplicate_key_update(mut self, fields: &[&str]) -> Self {
        self.on_duplicate_key_update = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 设置 ON CONFLICT DO UPDATE（PostgreSQL UPSERT）
    ///
    /// # 参数
    /// - `conflict_columns`: 冲突列
    /// - `update_columns`: 更新列
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn on_conflict_update(mut self, conflict_columns: &[&str], update_columns: &[&str]) -> Self {
        self.on_duplicate_key_update = update_columns.iter().map(|s| s.to_string()).collect();
        self.returning = conflict_columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 设置返回字段（PostgreSQL）
    ///
    /// # 参数
    /// - `fields`: 返回字段列表
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn returning(mut self, fields: &[&str]) -> Self {
        self.returning = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 构建插入 SQL
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    pub fn build(&self) -> (String, Vec<Value>) {
        if self.data.is_empty() {
            return (String::new(), Vec::new());
        }

        match self.db_type {
            DatabaseType::MySQL => self.build_mysql(),
            DatabaseType::PostgreSQL => self.build_postgres(),
            DatabaseType::SQLite => self.build_sqlite(),
        }
    }

    /// 构建 MySQL 插入 SQL
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    fn build_mysql(&self) -> (String, Vec<Value>) {
        let mut sql = String::new();
        let mut bindings = Vec::new();

        // 构建 INSERT/REPLACE 关键字
        if self.is_replace {
            sql.push_str("REPLACE INTO ");
        } else if self.is_ignore {
            sql.push_str("INSERT IGNORE INTO ");
        } else {
            sql.push_str("INSERT INTO ");
        }

        sql.push_str(&self.quote_identifier(&self.table));

        // 获取所有字段名
        let fields: Vec<&String> = self.data[0].keys().collect();
        let field_names: Vec<String> = fields.iter()
            .map(|f| self.quote_identifier(f))
            .collect();
        sql.push_str(&format!(" ({}) VALUES ", field_names.join(", ")));

        // 构建值占位符
        let value_count = self.data.len();
        let field_count = fields.len();

        for (i, row) in self.data.iter().enumerate() {
            sql.push('(');
            for (j, field) in fields.iter().enumerate() {
                sql.push('?');
                if let Some(value) = row.get(*field) {
                    bindings.push(value.clone());
                } else {
                    bindings.push(Value::Null);
                }
                if j < field_count - 1 {
                    sql.push_str(", ");
                }
            }
            sql.push(')');
            if i < value_count - 1 {
                sql.push_str(", ");
            }
        }

        // 添加 ON DUPLICATE KEY UPDATE
        if !self.on_duplicate_key_update.is_empty() {
            sql.push_str(" ON DUPLICATE KEY UPDATE ");
            let updates: Vec<String> = self.on_duplicate_key_update.iter()
                .map(|f| format!("{} = VALUES({})", self.quote_identifier(f), self.quote_identifier(f)))
                .collect();
            sql.push_str(&updates.join(", "));
        }

        (sql, bindings)
    }

    /// 构建 PostgreSQL 插入 SQL
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    fn build_postgres(&self) -> (String, Vec<Value>) {
        let mut sql = String::new();
        let mut bindings = Vec::new();

        sql.push_str("INSERT INTO ");
        sql.push_str(&self.quote_identifier(&self.table));

        // 获取所有字段名
        let fields: Vec<&String> = self.data[0].keys().collect();
        let field_names: Vec<String> = fields.iter()
            .map(|f| self.quote_identifier(f))
            .collect();
        sql.push_str(&format!(" ({}) VALUES ", field_names.join(", ")));

        // 构建值占位符（PostgreSQL 使用 $1, $2, ...）
        let mut param_index = 1;
        for (i, row) in self.data.iter().enumerate() {
            sql.push('(');
            for (j, field) in fields.iter().enumerate() {
                sql.push_str(&format!("${}", param_index));
                param_index += 1;
                if let Some(value) = row.get(*field) {
                    bindings.push(value.clone());
                } else {
                    bindings.push(Value::Null);
                }
                if j < fields.len() - 1 {
                    sql.push_str(", ");
                }
            }
            sql.push(')');
            if i < self.data.len() - 1 {
                sql.push_str(", ");
            }
        }

        // 添加 RETURNING
        if !self.returning.is_empty() {
            let returning: Vec<String> = self.returning.iter()
                .map(|f| self.quote_identifier(f))
                .collect();
            sql.push_str(&format!(" RETURNING {}", returning.join(", ")));
        }

        // 添加 ON CONFLICT DO UPDATE
        if !self.on_duplicate_key_update.is_empty() && !self.returning.is_empty() {
            let conflict_cols: Vec<String> = self.returning.iter()
                .map(|f| self.quote_identifier(f))
                .collect();
            let updates: Vec<String> = self.on_duplicate_key_update.iter()
                .map(|f| format!("{} = EXCLUDED.{}", self.quote_identifier(f), self.quote_identifier(f)))
                .collect();
            sql.insert_str(sql.find(" RETURNING").unwrap_or(sql.len()),
                &format!(" ON CONFLICT ({}) DO UPDATE SET {}", conflict_cols.join(", "), updates.join(", ")));
        }

        (sql, bindings)
    }

    /// 构建 SQLite 插入 SQL
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    fn build_sqlite(&self) -> (String, Vec<Value>) {
        let mut sql = String::new();
        let mut bindings = Vec::new();

        if self.is_replace {
            sql.push_str("INSERT OR REPLACE INTO ");
        } else if self.is_ignore {
            sql.push_str("INSERT OR IGNORE INTO ");
        } else {
            sql.push_str("INSERT INTO ");
        }

        sql.push_str(&self.quote_identifier(&self.table));

        // 获取所有字段名
        let fields: Vec<&String> = self.data[0].keys().collect();
        let field_names: Vec<String> = fields.iter()
            .map(|f| self.quote_identifier(f))
            .collect();
        sql.push_str(&format!(" ({}) VALUES ", field_names.join(", ")));

        // 构建值占位符
        for (i, row) in self.data.iter().enumerate() {
            sql.push('(');
            for (j, field) in fields.iter().enumerate() {
                sql.push('?');
                if let Some(value) = row.get(*field) {
                    bindings.push(value.clone());
                } else {
                    bindings.push(Value::Null);
                }
                if j < fields.len() - 1 {
                    sql.push_str(", ");
                }
            }
            sql.push(')');
            if i < self.data.len() - 1 {
                sql.push_str(", ");
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

/// 插入结果
///
/// 存储插入操作的结果
#[derive(Debug, Clone)]
pub struct InsertResult {
    /// 影响的行数
    pub affected_rows: u64,
    /// 最后插入的 ID
    pub last_insert_id: Option<i64>,
    /// 所有插入的 ID（批量插入）
    pub all_insert_ids: Vec<i64>,
}

impl InsertResult {
    /// 创建新的插入结果
    ///
    /// # 参数
    /// - `affected_rows`: 影响的行数
    ///
    /// # 返回
    /// 新的插入结果实例
    pub fn new(affected_rows: u64) -> Self {
        Self {
            affected_rows,
            last_insert_id: None,
            all_insert_ids: Vec::new(),
        }
    }

    /// 设置最后插入的 ID
    ///
    /// # 参数
    /// - `id`: 最后插入的 ID
    ///
    /// # 返回
    /// 修改后的插入结果
    pub fn with_last_id(mut self, id: i64) -> Self {
        self.last_insert_id = Some(id);
        self
    }

    /// 设置所有插入的 ID
    ///
    /// # 参数
    /// - `ids`: 所有插入的 ID
    ///
    /// # 返回
    /// 修改后的插入结果
    pub fn with_all_ids(mut self, ids: Vec<i64>) -> Self {
        self.all_insert_ids = ids;
        self
    }
}

/// UPSERT 构建器
///
/// 专门用于构建 UPSERT（插入或更新）操作
#[derive(Debug, Clone)]
pub struct UpsertBuilder {
    /// 插入构建器
    pub insert_builder: InsertBuilder,
    /// 唯一键列
    pub unique_columns: Vec<String>,
}

impl UpsertBuilder {
    /// 创建新的 UPSERT 构建器
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的 UPSERT 构建器实例
    pub fn new(table: &str, db_type: DatabaseType) -> Self {
        Self {
            insert_builder: InsertBuilder::new(table, db_type),
            unique_columns: Vec::new(),
        }
    }

    /// 设置插入数据
    ///
    /// # 参数
    /// - `data`: 数据键值对
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn values(mut self, data: HashMap<String, Value>) -> Self {
        self.insert_builder = self.insert_builder.values(data);
        self
    }

    /// 设置唯一键列
    ///
    /// # 参数
    /// - `columns`: 唯一键列名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn unique_columns(mut self, columns: &[&str]) -> Self {
        self.unique_columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 设置更新列
    ///
    /// # 参数
    /// - `columns`: 更新列名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn update_columns(mut self, columns: &[&str]) -> Self {
        self.insert_builder = self.insert_builder.on_duplicate_key_update(columns);
        self
    }

    /// 构建 UPSERT SQL
    ///
    /// # 返回
    /// (SQL 字符串, 绑定参数列表)
    pub fn build(&self) -> (String, Vec<Value>) {
        self.insert_builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_builder() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), Value::String("test".to_string()));
        data.insert("age".to_string(), Value::Int(25));

        let builder = InsertBuilder::new("users", DatabaseType::MySQL)
            .values(data);

        let (sql, bindings) = builder.build();
        assert!(sql.contains("INSERT INTO"));
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_insert_batch() {
        let mut data1 = HashMap::new();
        data1.insert("name".to_string(), Value::String("user1".to_string()));

        let mut data2 = HashMap::new();
        data2.insert("name".to_string(), Value::String("user2".to_string()));

        let builder = InsertBuilder::new("users", DatabaseType::MySQL)
            .batch(vec![data1, data2]);

        let (sql, _) = builder.build();
        assert!(sql.contains("INSERT INTO"));
    }

    #[test]
    fn test_insert_replace() {
        let mut data = HashMap::new();
        data.insert("id".to_string(), Value::Int(1));
        data.insert("name".to_string(), Value::String("test".to_string()));

        let builder = InsertBuilder::new("users", DatabaseType::MySQL)
            .values(data)
            .replace();

        let (sql, _) = builder.build();
        assert!(sql.contains("REPLACE INTO"));
    }

    #[test]
    fn test_insert_ignore() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), Value::String("test".to_string()));

        let builder = InsertBuilder::new("users", DatabaseType::MySQL)
            .values(data)
            .ignore();

        let (sql, _) = builder.build();
        assert!(sql.contains("INSERT IGNORE INTO"));
    }

    #[test]
    fn test_upsert_mysql() {
        let mut data = HashMap::new();
        data.insert("id".to_string(), Value::Int(1));
        data.insert("name".to_string(), Value::String("test".to_string()));

        let builder = InsertBuilder::new("users", DatabaseType::MySQL)
            .values(data)
            .on_duplicate_key_update(&["name"]);

        let (sql, _) = builder.build();
        assert!(sql.contains("ON DUPLICATE KEY UPDATE"));
    }

    #[test]
    fn test_postgres_returning() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), Value::String("test".to_string()));

        let builder = InsertBuilder::new("users", DatabaseType::PostgreSQL)
            .values(data)
            .returning(&["id", "name"]);

        let (sql, _) = builder.build();
        assert!(sql.contains("RETURNING"));
    }
}
