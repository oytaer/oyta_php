//! 字段操作模块
//!
//! 提供数据库查询字段操作功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：字段选择、字段排除、别名、去重等

use crate::interpreter::value::Value;

/// 字段类型枚举
///
/// 定义字段的不同类型
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    /// 普通字段
    Normal,
    /// 聚合字段
    Aggregate,
    /// 表达式字段
    Expression,
    /// 原始 SQL 字段
    Raw,
    /// 子查询字段
    SubQuery,
}

/// 字段结构体
///
/// 存储单个字段的信息
#[derive(Debug, Clone)]
pub struct Field {
    /// 字段名或表达式
    pub name: String,
    /// 字段别名
    pub alias: Option<String>,
    /// 字段类型
    pub field_type: FieldType,
    /// 表名前缀
    pub table_prefix: Option<String>,
}

impl Field {
    /// 创建新的字段
    ///
    /// # 参数
    /// - `name`: 字段名
    ///
    /// # 返回
    /// 新的字段实例
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            alias: None,
            field_type: FieldType::Normal,
            table_prefix: None,
        }
    }

    /// 创建带别名的字段
    ///
    /// # 参数
    /// - `name`: 字段名
    /// - `alias`: 别名
    ///
    /// # 返回
    /// 新的字段实例
    pub fn with_alias(name: &str, alias: &str) -> Self {
        Self {
            name: name.to_string(),
            alias: Some(alias.to_string()),
            field_type: FieldType::Normal,
            table_prefix: None,
        }
    }

    /// 创建聚合字段
    ///
    /// # 参数
    /// - `expression`: 聚合表达式
    /// - `alias`: 别名
    ///
    /// # 返回
    /// 新的字段实例
    pub fn aggregate(expression: &str, alias: &str) -> Self {
        Self {
            name: expression.to_string(),
            alias: Some(alias.to_string()),
            field_type: FieldType::Aggregate,
            table_prefix: None,
        }
    }

    /// 创建表达式字段
    ///
    /// # 参数
    /// - `expression`: 表达式
    /// - `alias`: 别名
    ///
    /// # 返回
    /// 新的字段实例
    pub fn expression(expression: &str, alias: &str) -> Self {
        Self {
            name: expression.to_string(),
            alias: Some(alias.to_string()),
            field_type: FieldType::Expression,
            table_prefix: None,
        }
    }

    /// 创建原始 SQL 字段
    ///
    /// # 参数
    /// - `sql`: 原始 SQL
    ///
    /// # 返回
    /// 新的字段实例
    pub fn raw(sql: &str) -> Self {
        Self {
            name: sql.to_string(),
            alias: None,
            field_type: FieldType::Raw,
            table_prefix: None,
        }
    }

    /// 设置表名前缀
    ///
    /// # 参数
    /// - `table`: 表名
    ///
    /// # 返回
    /// 修改后的字段
    pub fn table(mut self, table: &str) -> Self {
        self.table_prefix = Some(table.to_string());
        self
    }

    /// 设置别名
    ///
    /// # 参数
    /// - `alias`: 别名
    ///
    /// # 返回
    /// 修改后的字段
    pub fn alias(mut self, alias: &str) -> Self {
        self.alias = Some(alias.to_string());
        self
    }

    /// 构建 SQL
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 字段 SQL 字符串
    pub fn build(&self, db_type: DatabaseType) -> String {
        let mut sql = String::new();

        match self.field_type {
            FieldType::Normal => {
                if let Some(table) = &self.table_prefix {
                    sql.push_str(&format!("{}.{}", 
                        quote_identifier(table, db_type),
                        quote_identifier(&self.name, db_type)
                    ));
                } else if self.name == "*" {
                    sql.push('*');
                } else {
                    sql.push_str(&quote_identifier(&self.name, db_type));
                }
            }
            FieldType::Aggregate | FieldType::Expression | FieldType::Raw => {
                sql.push_str(&self.name);
            }
            FieldType::SubQuery => {
                sql.push_str(&self.name);
            }
        }

        if let Some(alias) = &self.alias {
            sql.push_str(&format!(" AS {}", quote_identifier(alias, db_type)));
        }

        sql
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

/// 引用标识符
///
/// # 参数
/// - `identifier`: 标识符
/// - `db_type`: 数据库类型
///
/// # 返回
/// 引用后的标识符
fn quote_identifier(identifier: &str, db_type: DatabaseType) -> String {
    // 如果包含函数调用或特殊字符，不引用
    if identifier.contains('(') || identifier.contains(' ') || identifier.contains('*') {
        return identifier.to_string();
    }

    match db_type {
        DatabaseType::MySQL => format!("`{}`", identifier),
        DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
    }
}

/// 字段构建器
///
/// 提供链式调用的字段构建方法
#[derive(Debug, Clone)]
pub struct FieldBuilder {
    /// 字段列表
    pub fields: Vec<Field>,
    /// 是否去重
    pub distinct: bool,
    /// 数据库类型
    pub db_type: DatabaseType,
}

impl FieldBuilder {
    /// 创建新的字段构建器
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的字段构建器实例
    pub fn new(db_type: DatabaseType) -> Self {
        Self {
            fields: Vec::new(),
            distinct: false,
            db_type,
        }
    }

    /// 添加字段
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn field(mut self, field: &str) -> Self {
        self.fields.push(Field::new(field));
        self
    }

    /// 添加带别名的字段
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `alias`: 别名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn field_as(mut self, field: &str, alias: &str) -> Self {
        self.fields.push(Field::with_alias(field, alias));
        self
    }

    /// 添加多个字段
    ///
    /// # 参数
    /// - `fields`: 字段名列表
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn fields(mut self, fields: &[&str]) -> Self {
        for field in fields {
            self.fields.push(Field::new(field));
        }
        self
    }

    /// 添加聚合字段
    ///
    /// # 参数
    /// - `expression`: 聚合表达式
    /// - `alias`: 别名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn aggregate(mut self, expression: &str, alias: &str) -> Self {
        self.fields.push(Field::aggregate(expression, alias));
        self
    }

    /// 添加表达式字段
    ///
    /// # 参数
    /// - `expression`: 表达式
    /// - `alias`: 别名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn expression(mut self, expression: &str, alias: &str) -> Self {
        self.fields.push(Field::expression(expression, alias));
        self
    }

    /// 添加原始 SQL 字段
    ///
    /// # 参数
    /// - `sql`: 原始 SQL
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn field_raw(mut self, sql: &str) -> Self {
        self.fields.push(Field::raw(sql));
        self
    }

    /// 添加带表前缀的字段
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn table_field(mut self, table: &str, field: &str) -> Self {
        self.fields.push(Field::new(field).table(table));
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

    /// 排除指定字段
    ///
    /// # 参数
    /// - `exclude_fields`: 要排除的字段列表
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn exclude(&mut self, exclude_fields: &[&str]) {
        self.fields.retain(|f| {
            !exclude_fields.contains(&f.name.as_str())
        });
    }

    /// 只保留指定字段
    ///
    /// # 参数
    /// - `only_fields`: 要保留的字段列表
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn only(&mut self, only_fields: &[&str]) {
        self.fields.retain(|f| {
            only_fields.contains(&f.name.as_str())
        });
    }

    /// 构建 SELECT 字段 SQL
    ///
    /// # 返回
    /// 字段 SQL 字符串
    pub fn build(&self) -> String {
        if self.fields.is_empty() {
            return if self.distinct {
                "SELECT DISTINCT *".to_string()
            } else {
                "SELECT *".to_string()
            };
        }

        let distinct_str = if self.distinct { "DISTINCT " } else { "" };
        let fields: Vec<String> = self.fields.iter()
            .map(|f| f.build(self.db_type))
            .collect();

        format!("SELECT {}{}", distinct_str, fields.join(", "))
    }
}

/// 表别名构建器
///
/// 用于构建带别名的表引用
#[derive(Debug, Clone)]
pub struct AliasBuilder {
    /// 表名
    pub table: String,
    /// 别名
    pub alias: Option<String>,
    /// 数据库类型
    pub db_type: DatabaseType,
}

impl AliasBuilder {
    /// 创建新的别名构建器
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的别名构建器实例
    pub fn new(table: &str, db_type: DatabaseType) -> Self {
        Self {
            table: table.to_string(),
            alias: None,
            db_type,
        }
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

    /// 构建表引用 SQL
    ///
    /// # 返回
    /// 表引用 SQL 字符串
    pub fn build(&self) -> String {
        let quoted_table = quote_identifier(&self.table, self.db_type);
        if let Some(alias) = &self.alias {
            format!("{} AS {}", quoted_table, quote_identifier(alias, self.db_type))
        } else {
            quoted_table
        }
    }
}

/// 字段转换器
///
/// 用于字段值的类型转换
#[derive(Debug, Clone)]
pub struct FieldCaster {
    /// 数据库类型
    pub db_type: DatabaseType,
}

impl FieldCaster {
    /// 创建新的字段转换器
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的字段转换器实例
    pub fn new(db_type: DatabaseType) -> Self {
        Self { db_type }
    }

    /// 转换为整数
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 转换表达式
    pub fn as_integer(&self, field: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("CAST({} AS SIGNED)", field),
            DatabaseType::PostgreSQL => format!("{}::integer", field),
            DatabaseType::SQLite => format!("CAST({} AS INTEGER)", field),
        }
    }

    /// 转换为浮点数
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 转换表达式
    pub fn as_float(&self, field: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("CAST({} AS DECIMAL)", field),
            DatabaseType::PostgreSQL => format!("{}::float", field),
            DatabaseType::SQLite => format!("CAST({} AS REAL)", field),
        }
    }

    /// 转换为字符串
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 转换表达式
    pub fn as_string(&self, field: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("CAST({} AS CHAR)", field),
            DatabaseType::PostgreSQL => format!("{}::text", field),
            DatabaseType::SQLite => format!("CAST({} AS TEXT)", field),
        }
    }

    /// 转换为日期
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 转换表达式
    pub fn as_date(&self, field: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("CAST({} AS DATE)", field),
            DatabaseType::PostgreSQL => format!("{}::date", field),
            DatabaseType::SQLite => format!("date({})", field),
        }
    }

    /// 转换为时间
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 转换表达式
    pub fn as_datetime(&self, field: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("CAST({} AS DATETIME)", field),
            DatabaseType::PostgreSQL => format!("{}::timestamp", field),
            DatabaseType::SQLite => format!("datetime({})", field),
        }
    }

    /// 转换为布尔值
    ///
    /// # 参数
    /// - `field`: 字段名
    ///
    /// # 返回
    /// 转换表达式
    pub fn as_boolean(&self, field: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("CAST({} AS SIGNED)", field),
            DatabaseType::PostgreSQL => format!("{}::boolean", field),
            DatabaseType::SQLite => format!("CAST({} AS INTEGER)", field),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_builder() {
        let builder = FieldBuilder::new(DatabaseType::MySQL)
            .field("id")
            .field_as("name", "user_name")
            .aggregate("COUNT(*)", "total");

        let sql = builder.build();
        assert!(sql.contains("id"));
        assert!(sql.contains("user_name"));
        assert!(sql.contains("COUNT(*)"));
    }

    #[test]
    fn test_field_distinct() {
        let builder = FieldBuilder::new(DatabaseType::MySQL)
            .field("name")
            .distinct();

        let sql = builder.build();
        assert!(sql.contains("DISTINCT"));
    }

    #[test]
    fn test_field_table_prefix() {
        let builder = FieldBuilder::new(DatabaseType::MySQL)
            .table_field("users", "id");

        let sql = builder.build();
        // MySQL 使用反引号引用标识符
        assert!(sql.contains("`users`.`id`"));
    }

    #[test]
    fn test_alias_builder() {
        let builder = AliasBuilder::new("users", DatabaseType::MySQL)
            .alias("u");

        let sql = builder.build();
        assert_eq!(sql, "`users` AS `u`");
    }

    #[test]
    fn test_field_caster_integer() {
        let caster = FieldCaster::new(DatabaseType::MySQL);
        let sql = caster.as_integer("price");
        assert!(sql.contains("CAST"));
        assert!(sql.contains("SIGNED"));
    }

    #[test]
    fn test_field_caster_postgres() {
        let caster = FieldCaster::new(DatabaseType::PostgreSQL);
        let sql = caster.as_integer("price");
        assert!(sql.contains("::integer"));
    }

    #[test]
    fn test_field_caster_sqlite() {
        let caster = FieldCaster::new(DatabaseType::SQLite);
        let sql = caster.as_integer("price");
        assert!(sql.contains("CAST"));
        assert!(sql.contains("INTEGER"));
    }
}
