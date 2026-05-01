//! 表结构定义模块
//!
//! 提供数据库表结构定义功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库

use anyhow::Result;

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

/// 列定义
#[derive(Debug, Clone)]
pub struct ColumnDefinition {
    /// 列名
    pub name: String,
    /// 数据类型
    pub data_type: String,
    /// 是否允许 NULL
    pub nullable: bool,
    /// 默认值
    pub default: Option<String>,
    /// 是否自增
    pub auto_increment: bool,
    /// 是否无符号（MySQL）
    pub unsigned: bool,
    /// 注释
    pub comment: Option<String>,
    /// 排序规则
    pub collation: Option<String>,
}

impl ColumnDefinition {
    /// 创建新的列定义
    pub fn new(name: &str, data_type: &str) -> Self {
        Self {
            name: name.to_string(),
            data_type: data_type.to_string(),
            nullable: true,
            default: None,
            auto_increment: false,
            unsigned: false,
            comment: None,
            collation: None,
        }
    }

    /// 设置不允许 NULL
    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    /// 设置默认值
    pub fn default(mut self, value: &str) -> Self {
        self.default = Some(value.to_string());
        self
    }

    /// 设置自增
    pub fn auto_increment(mut self) -> Self {
        self.auto_increment = true;
        self
    }

    /// 设置无符号
    pub fn unsigned(mut self) -> Self {
        self.unsigned = true;
        self
    }

    /// 设置注释
    pub fn comment(mut self, comment: &str) -> Self {
        self.comment = Some(comment.to_string());
        self
    }

    /// 构建列定义 SQL
    pub fn build(&self, db_type: DatabaseType) -> String {
        let mut sql = format!("{} {}", self.quote_identifier(&self.name, db_type), self.data_type);

        if self.unsigned && db_type == DatabaseType::MySQL {
            sql.push_str(" UNSIGNED");
        }

        if !self.nullable {
            sql.push_str(" NOT NULL");
        }

        if self.auto_increment {
            match db_type {
                DatabaseType::MySQL => sql.push_str(" AUTO_INCREMENT"),
                DatabaseType::PostgreSQL => sql = format!("{} SERIAL", self.quote_identifier(&self.name, db_type)),
                DatabaseType::SQLite => sql.push_str(" AUTOINCREMENT"),
            }
        }

        if let Some(default) = &self.default {
            sql.push_str(&format!(" DEFAULT {}", default));
        }

        if let Some(comment) = &self.comment {
            if db_type == DatabaseType::MySQL {
                sql.push_str(&format!(" COMMENT '{}'", comment));
            }
        }

        sql
    }

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str, db_type: DatabaseType) -> String {
        match db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }
}

/// 索引定义
#[derive(Debug, Clone)]
pub struct IndexDefinition {
    /// 索引名
    pub name: String,
    /// 索引类型
    pub index_type: IndexType,
    /// 列名列表
    pub columns: Vec<String>,
}

/// 索引类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndexType {
    /// 普通索引
    Index,
    /// 唯一索引
    Unique,
    /// 主键
    Primary,
    /// 全文索引
    FullText,
}

impl IndexDefinition {
    /// 创建新的索引定义
    pub fn new(name: &str, index_type: IndexType, columns: Vec<&str>) -> Self {
        Self {
            name: name.to_string(),
            index_type,
            columns: columns.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// 构建索引定义 SQL
    pub fn build(&self, db_type: DatabaseType) -> String {
        let columns: Vec<String> = self.columns.iter()
            .map(|c| self.quote_identifier(c, db_type))
            .collect();

        match self.index_type {
            IndexType::Primary => {
                format!("PRIMARY KEY ({})", columns.join(", "))
            }
            IndexType::Unique => {
                format!("UNIQUE KEY {} ({})", self.quote_identifier(&self.name, db_type), columns.join(", "))
            }
            IndexType::Index => {
                format!("INDEX {} ({})", self.quote_identifier(&self.name, db_type), columns.join(", "))
            }
            IndexType::FullText => {
                match db_type {
                    DatabaseType::MySQL => format!("FULLTEXT INDEX {} ({})", self.quote_identifier(&self.name, db_type), columns.join(", ")),
                    _ => format!("INDEX {} ({})", self.quote_identifier(&self.name, db_type), columns.join(", ")),
                }
            }
        }
    }

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str, db_type: DatabaseType) -> String {
        match db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }
}

/// 外键定义
#[derive(Debug, Clone)]
pub struct ForeignKeyDefinition {
    /// 外键名
    pub name: String,
    /// 本地列
    pub local_columns: Vec<String>,
    /// 外表名
    pub foreign_table: String,
    /// 外表列
    pub foreign_columns: Vec<String>,
    /// 删除行为
    pub on_delete: Option<ReferenceOption>,
    /// 更新行为
    pub on_update: Option<ReferenceOption>,
}

/// 引用行为枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReferenceOption {
    /// 级联
    Cascade,
    /// 设为 NULL
    SetNull,
    /// 限制
    Restrict,
    /// 无操作
    NoAction,
}

impl ForeignKeyDefinition {
    /// 创建新的外键定义
    pub fn new(name: &str, local_columns: Vec<&str>, foreign_table: &str, foreign_columns: Vec<&str>) -> Self {
        Self {
            name: name.to_string(),
            local_columns: local_columns.iter().map(|s| s.to_string()).collect(),
            foreign_table: foreign_table.to_string(),
            foreign_columns: foreign_columns.iter().map(|s| s.to_string()).collect(),
            on_delete: None,
            on_update: None,
        }
    }

    /// 设置删除行为
    pub fn on_delete(mut self, option: ReferenceOption) -> Self {
        self.on_delete = Some(option);
        self
    }

    /// 设置更新行为
    pub fn on_update(mut self, option: ReferenceOption) -> Self {
        self.on_update = Some(option);
        self
    }

    /// 构建外键定义 SQL
    pub fn build(&self, db_type: DatabaseType) -> String {
        let local: Vec<String> = self.local_columns.iter()
            .map(|c| self.quote_identifier(c, db_type))
            .collect();
        let foreign: Vec<String> = self.foreign_columns.iter()
            .map(|c| self.quote_identifier(c, db_type))
            .collect();

        let mut sql = format!(
            "CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({})",
            self.quote_identifier(&self.name, db_type),
            local.join(", "),
            self.quote_identifier(&self.foreign_table, db_type),
            foreign.join(", ")
        );

        if let Some(on_delete) = &self.on_delete {
            sql.push_str(&format!(" ON DELETE {}", self.option_to_str(on_delete)));
        }

        if let Some(on_update) = &self.on_update {
            sql.push_str(&format!(" ON UPDATE {}", self.option_to_str(on_update)));
        }

        sql
    }

    /// 引用行为转字符串
    fn option_to_str(&self, option: &ReferenceOption) -> &'static str {
        match option {
            ReferenceOption::Cascade => "CASCADE",
            ReferenceOption::SetNull => "SET NULL",
            ReferenceOption::Restrict => "RESTRICT",
            ReferenceOption::NoAction => "NO ACTION",
        }
    }

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str, db_type: DatabaseType) -> String {
        match db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }
}

/// 表定义
#[derive(Debug, Clone)]
pub struct TableDefinition {
    /// 表名
    pub name: String,
    /// 列定义列表
    pub columns: Vec<ColumnDefinition>,
    /// 索引定义列表
    pub indexes: Vec<IndexDefinition>,
    /// 外键定义列表
    pub foreign_keys: Vec<ForeignKeyDefinition>,
    /// 表注释
    pub comment: Option<String>,
    /// 表引擎（MySQL）
    pub engine: Option<String>,
    /// 字符集
    pub charset: Option<String>,
}

impl TableDefinition {
    /// 创建新的表定义
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            columns: Vec::new(),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            comment: None,
            engine: None,
            charset: None,
        }
    }

    /// 添加列
    pub fn add_column(mut self, column: ColumnDefinition) -> Self {
        self.columns.push(column);
        self
    }

    /// 添加索引
    pub fn add_index(mut self, index: IndexDefinition) -> Self {
        self.indexes.push(index);
        self
    }

    /// 添加外键
    pub fn add_foreign_key(mut self, foreign_key: ForeignKeyDefinition) -> Self {
        self.foreign_keys.push(foreign_key);
        self
    }

    /// 设置表注释
    pub fn comment(mut self, comment: &str) -> Self {
        self.comment = Some(comment.to_string());
        self
    }

    /// 设置表引擎
    pub fn engine(mut self, engine: &str) -> Self {
        self.engine = Some(engine.to_string());
        self
    }

    /// 设置字符集
    pub fn charset(mut self, charset: &str) -> Self {
        self.charset = Some(charset.to_string());
        self
    }

    /// 构建建表 SQL
    pub fn build(&self, db_type: DatabaseType) -> String {
        let mut sql = format!("CREATE TABLE {} (\n", self.quote_identifier(&self.name, db_type));

        // 添加列定义
        let column_defs: Vec<String> = self.columns.iter()
            .map(|c| format!("  {}", c.build(db_type)))
            .collect();
        sql.push_str(&column_defs.join(",\n"));

        // 添加索引
        if !self.indexes.is_empty() {
            sql.push_str(",\n");
            let index_defs: Vec<String> = self.indexes.iter()
                .map(|i| format!("  {}", i.build(db_type)))
                .collect();
            sql.push_str(&index_defs.join(",\n"));
        }

        // 添加外键
        if !self.foreign_keys.is_empty() {
            sql.push_str(",\n");
            let fk_defs: Vec<String> = self.foreign_keys.iter()
                .map(|f| format!("  {}", f.build(db_type)))
                .collect();
            sql.push_str(&fk_defs.join(",\n"));
        }

        sql.push_str("\n)");

        // 添加表选项
        if db_type == DatabaseType::MySQL {
            if let Some(engine) = &self.engine {
                sql.push_str(&format!(" ENGINE={}", engine));
            }
            if let Some(charset) = &self.charset {
                sql.push_str(&format!(" DEFAULT CHARSET={}", charset));
            }
            if let Some(comment) = &self.comment {
                sql.push_str(&format!(" COMMENT='{}'", comment));
            }
        }

        sql
    }

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str, db_type: DatabaseType) -> String {
        match db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }
}

/// Schema 构建器
pub struct SchemaBuilder {
    /// 数据库类型
    pub db_type: DatabaseType,
}

impl SchemaBuilder {
    /// 创建新的 Schema 构建器
    pub fn new(db_type: DatabaseType) -> Self {
        Self { db_type }
    }

    /// 创建表
    pub fn create_table(&self, name: &str) -> TableDefinition {
        TableDefinition::new(name)
    }

    /// 构建删除表 SQL
    pub fn drop_table(&self, name: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("DROP TABLE IF EXISTS `{}`", name),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("DROP TABLE IF EXISTS \"{}\"", name),
        }
    }

    /// 构建重命名表 SQL
    pub fn rename_table(&self, old_name: &str, new_name: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("RENAME TABLE `{}` TO `{}`", old_name, new_name),
            DatabaseType::PostgreSQL => format!("ALTER TABLE \"{}\" RENAME TO \"{}\"", old_name, new_name),
            DatabaseType::SQLite => format!("ALTER TABLE \"{}\" RENAME TO \"{}\"", old_name, new_name),
        }
    }

    /// 构建添加列 SQL
    pub fn add_column(&self, table: &str, column: &ColumnDefinition) -> String {
        format!(
            "ALTER TABLE {} ADD {}",
            self.quote_identifier(table),
            column.build(self.db_type)
        )
    }

    /// 构建删除列 SQL
    pub fn drop_column(&self, table: &str, column: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("ALTER TABLE `{}` DROP COLUMN `{}`", table, column),
            DatabaseType::PostgreSQL => format!("ALTER TABLE \"{}\" DROP COLUMN \"{}\"", table, column),
            DatabaseType::SQLite => format!("ALTER TABLE \"{}\" DROP COLUMN \"{}\"", table, column),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_definition() {
        let column = ColumnDefinition::new("id", "BIGINT")
            .not_null()
            .auto_increment();

        let sql = column.build(DatabaseType::MySQL);
        assert!(sql.contains("NOT NULL"));
        assert!(sql.contains("AUTO_INCREMENT"));
    }

    #[test]
    fn test_index_definition() {
        let index = IndexDefinition::new("idx_name", IndexType::Index, vec!["name"]);

        let sql = index.build(DatabaseType::MySQL);
        assert!(sql.contains("INDEX"));
        assert!(sql.contains("name"));
    }

    #[test]
    fn test_foreign_key_definition() {
        let fk = ForeignKeyDefinition::new(
            "fk_user_id",
            vec!["user_id"],
            "users",
            vec!["id"]
        ).on_delete(ReferenceOption::Cascade);

        let sql = fk.build(DatabaseType::MySQL);
        assert!(sql.contains("FOREIGN KEY"));
        assert!(sql.contains("ON DELETE CASCADE"));
    }

    #[test]
    fn test_table_definition() {
        let table = TableDefinition::new("users")
            .add_column(ColumnDefinition::new("id", "BIGINT").not_null().auto_increment())
            .add_column(ColumnDefinition::new("name", "VARCHAR(255)").not_null())
            .add_index(IndexDefinition::new("PRIMARY", IndexType::Primary, vec!["id"]))
            .engine("InnoDB")
            .charset("utf8mb4");

        let sql = table.build(DatabaseType::MySQL);
        assert!(sql.contains("CREATE TABLE"));
        assert!(sql.contains("ENGINE=InnoDB"));
    }

    #[test]
    fn test_schema_builder() {
        let builder = SchemaBuilder::new(DatabaseType::MySQL);

        let sql = builder.drop_table("users");
        assert!(sql.contains("DROP TABLE"));
    }
}
