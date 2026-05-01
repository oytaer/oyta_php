//! JOIN 查询扩展模块
//!
//! 提供丰富的 JOIN 查询构建方法
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：INNER JOIN、LEFT JOIN、RIGHT JOIN、CROSS JOIN、子查询 JOIN 等

use crate::interpreter::value::Value;
use std::collections::HashMap;

/// JOIN 类型枚举
///
/// 定义所有支持的 JOIN 类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JoinType {
    /// 内连接
    Inner,
    /// 左连接
    Left,
    /// 右连接
    Right,
    /// 交叉连接
    Cross,
    /// 左外连接
    LeftOuter,
    /// 右外连接
    RightOuter,
    /// 自然连接
    Natural,
}

impl JoinType {
    /// 获取 JOIN 关键字
    ///
    /// # 返回
    /// JOIN 关键字字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            JoinType::Inner => "INNER JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Cross => "CROSS JOIN",
            JoinType::LeftOuter => "LEFT OUTER JOIN",
            JoinType::RightOuter => "RIGHT OUTER JOIN",
            JoinType::Natural => "NATURAL JOIN",
        }
    }
}

/// JOIN 条件子句
///
/// 存储 JOIN 的 ON 条件
#[derive(Debug, Clone)]
pub struct JoinClause {
    /// JOIN 类型
    pub join_type: JoinType,
    /// 表名或子查询别名
    pub table: String,
    /// 是否为子查询
    pub is_subquery: bool,
    /// 子查询 SQL（如果为子查询）
    pub subquery: Option<String>,
    /// ON 条件列表
    pub conditions: Vec<JoinCondition>,
    /// USING 字段列表
    pub using: Vec<String>,
    /// 表别名
    pub alias: Option<String>,
}

impl JoinClause {
    /// 创建新的 JOIN 子句
    ///
    /// # 参数
    /// - `join_type`: JOIN 类型
    /// - `table`: 表名
    ///
    /// # 返回
    /// 新的 JOIN 子句实例
    pub fn new(join_type: JoinType, table: &str) -> Self {
        Self {
            join_type,
            table: table.to_string(),
            is_subquery: false,
            subquery: None,
            conditions: Vec::new(),
            using: Vec::new(),
            alias: None,
        }
    }

    /// 创建子查询 JOIN
    ///
    /// # 参数
    /// - `join_type`: JOIN 类型
    /// - `subquery`: 子查询 SQL
    /// - `alias`: 子查询别名
    ///
    /// # 返回
    /// 新的 JOIN 子句实例
    pub fn subquery(join_type: JoinType, subquery: &str, alias: &str) -> Self {
        Self {
            join_type,
            table: alias.to_string(),
            is_subquery: true,
            subquery: Some(subquery.to_string()),
            conditions: Vec::new(),
            using: Vec::new(),
            alias: Some(alias.to_string()),
        }
    }

    /// 设置表别名
    ///
    /// # 参数
    /// - `alias`: 别名
    ///
    /// # 返回
    /// 修改后的 JOIN 子句
    pub fn alias(mut self, alias: &str) -> Self {
        self.alias = Some(alias.to_string());
        self
    }

    /// 添加 ON 条件
    ///
    /// # 参数
    /// - `local`: 本地表字段
    /// - `operator`: 操作符
    /// - `foreign`: 外键字段
    ///
    /// # 返回
    /// 修改后的 JOIN 子句
    pub fn on(mut self, local: &str, operator: &str, foreign: &str) -> Self {
        self.conditions.push(JoinCondition {
            local: local.to_string(),
            operator: operator.to_string(),
            foreign: foreign.to_string(),
            connector: "AND",
        });
        self
    }

    /// 添加 OR ON 条件
    ///
    /// # 参数
    /// - `local`: 本地表字段
    /// - `operator`: 操作符
    /// - `foreign`: 外键字段
    ///
    /// # 返回
    /// 修改后的 JOIN 子句
    pub fn or_on(mut self, local: &str, operator: &str, foreign: &str) -> Self {
        self.conditions.push(JoinCondition {
            local: local.to_string(),
            operator: operator.to_string(),
            foreign: foreign.to_string(),
            connector: "OR",
        });
        self
    }

    /// 添加原始 ON 条件
    ///
    /// # 参数
    /// - `raw`: 原始 SQL 条件
    ///
    /// # 返回
    /// 修改后的 JOIN 子句
    pub fn on_raw(mut self, raw: &str) -> Self {
        self.conditions.push(JoinCondition {
            local: raw.to_string(),
            operator: String::new(),
            foreign: String::new(),
            connector: "AND",
        });
        self
    }

    /// 设置 USING 字段
    ///
    /// # 参数
    /// - `columns`: 字段名列表
    ///
    /// # 返回
    /// 修改后的 JOIN 子句
    pub fn using(mut self, columns: &[&str]) -> Self {
        self.using = columns.iter().map(|s| s.to_string()).collect();
        self
    }
}

/// JOIN 条件结构体
///
/// 存储单个 JOIN ON 条件
#[derive(Debug, Clone)]
pub struct JoinCondition {
    /// 本地字段
    pub local: String,
    /// 操作符
    pub operator: String,
    /// 外键字段
    pub foreign: String,
    /// 逻辑连接符
    pub connector: &'static str,
}

/// JOIN 构建器
///
/// 提供链式调用的 JOIN 构建方法
#[derive(Debug, Clone)]
pub struct JoinBuilder {
    /// JOIN 子句列表
    pub joins: Vec<JoinClause>,
    /// 绑定参数
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

impl JoinBuilder {
    /// 创建新的 JOIN 构建器
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的 JOIN 构建器实例
    pub fn new(db_type: DatabaseType) -> Self {
        Self {
            joins: Vec::new(),
            bindings: Vec::new(),
            db_type,
        }
    }

    /// 添加 INNER JOIN
    ///
    /// # 参数
    /// - `table`: 表名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn join(mut self, table: &str) -> JoinClauseBuilder {
        let clause = JoinClause::new(JoinType::Inner, table);
        JoinClauseBuilder::new(self, clause)
    }

    /// 添加 LEFT JOIN
    ///
    /// # 参数
    /// - `table`: 表名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn left_join(mut self, table: &str) -> JoinClauseBuilder {
        let clause = JoinClause::new(JoinType::Left, table);
        JoinClauseBuilder::new(self, clause)
    }

    /// 添加 RIGHT JOIN
    ///
    /// # 参数
    /// - `table`: 表名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn right_join(mut self, table: &str) -> JoinClauseBuilder {
        let clause = JoinClause::new(JoinType::Right, table);
        JoinClauseBuilder::new(self, clause)
    }

    /// 添加 CROSS JOIN
    ///
    /// # 参数
    /// - `table`: 表名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn cross_join(mut self, table: &str) -> JoinClauseBuilder {
        let clause = JoinClause::new(JoinType::Cross, table);
        JoinClauseBuilder::new(self, clause)
    }

    /// 添加子查询 JOIN
    ///
    /// # 参数
    /// - `join_type`: JOIN 类型
    /// - `subquery`: 子查询 SQL
    /// - `alias`: 子查询别名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn join_subquery(
        mut self,
        join_type: JoinType,
        subquery: &str,
        alias: &str,
    ) -> JoinClauseBuilder {
        let clause = JoinClause::subquery(join_type, subquery, alias);
        JoinClauseBuilder::new(self, clause)
    }

    /// 添加 LEFT JOIN 子查询
    ///
    /// # 参数
    /// - `subquery`: 子查询 SQL
    /// - `alias`: 子查询别名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn left_join_subquery(self, subquery: &str, alias: &str) -> JoinClauseBuilder {
        self.join_subquery(JoinType::Left, subquery, alias)
    }

    /// 添加 RIGHT JOIN 子查询
    ///
    /// # 参数
    /// - `subquery`: 子查询 SQL
    /// - `alias`: 子查询别名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn right_join_subquery(self, subquery: &str, alias: &str) -> JoinClauseBuilder {
        self.join_subquery(JoinType::Right, subquery, alias)
    }

    /// 添加原始 JOIN
    ///
    /// # 参数
    /// - `raw`: 原始 JOIN SQL
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn join_raw(mut self, raw: &str) -> Self {
        // 解析原始 JOIN SQL
        let clause = JoinClause {
            join_type: JoinType::Inner,
            table: raw.to_string(),
            is_subquery: false,
            subquery: None,
            conditions: vec![JoinCondition {
                local: raw.to_string(),
                operator: String::new(),
                foreign: String::new(),
                connector: "AND",
            }],
            using: Vec::new(),
            alias: None,
        };
        self.joins.push(clause);
        self
    }

    /// 添加 JOIN 子句
    ///
    /// # 参数
    /// - `clause`: JOIN 子句
    pub fn add_clause(&mut self, clause: JoinClause) {
        self.joins.push(clause);
    }

    /// 构建所有 JOIN SQL
    ///
    /// # 返回
    /// JOIN SQL 字符串
    pub fn build(&self) -> String {
        let mut sql = String::new();

        for join in &self.joins {
            sql.push_str(&self.build_join_sql(join));
            sql.push(' ');
        }

        sql.trim_end().to_string()
    }

    /// 构建单个 JOIN SQL
    ///
    /// # 参数
    /// - `join`: JOIN 子句
    ///
    /// # 返回
    /// 单个 JOIN SQL 字符串
    fn build_join_sql(&self, join: &JoinClause) -> String {
        let mut sql = String::new();

        // 添加 JOIN 类型
        sql.push_str(join.join_type.as_str());
        sql.push(' ');

        // 添加表名或子查询
        if join.is_subquery {
            if let Some(subquery) = &join.subquery {
                sql.push_str(&format!("({}) AS {}", subquery, self.quote_identifier(&join.table)));
            }
        } else if let Some(alias) = &join.alias {
            sql.push_str(&format!(
                "{} AS {}",
                self.quote_identifier(&join.table),
                self.quote_identifier(alias)
            ));
        } else {
            sql.push_str(&self.quote_identifier(&join.table));
        }

        // 添加 ON 条件或 USING
        if !join.conditions.is_empty() {
            sql.push_str(" ON ");
            let mut first = true;
            for condition in &join.conditions {
                if !first {
                    sql.push_str(&format!(" {} ", condition.connector));
                }
                first = false;

                if condition.operator.is_empty() {
                    // 原始条件
                    sql.push_str(&condition.local);
                } else {
                    sql.push_str(&format!(
                        "{} {} {}",
                        self.quote_identifier(&condition.local),
                        condition.operator,
                        self.quote_identifier(&condition.foreign)
                    ));
                }
            }
        } else if !join.using.is_empty() {
            let columns: Vec<String> = join.using.iter()
                .map(|c| self.quote_identifier(c))
                .collect();
            sql.push_str(&format!(" USING ({})", columns.join(", ")));
        }

        sql
    }

    /// 引用标识符
    ///
    /// # 参数
    /// - `identifier`: 标识符名称
    ///
    /// # 返回
    /// 引用后的标识符
    fn quote_identifier(&self, identifier: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }
}

/// JOIN 子句构建器
///
/// 用于构建单个 JOIN 子句的 ON 条件
#[derive(Debug)]
pub struct JoinClauseBuilder {
    /// 父构建器
    pub parent: JoinBuilder,
    /// 当前 JOIN 子句
    pub clause: JoinClause,
}

impl JoinClauseBuilder {
    /// 创建新的 JOIN 子句构建器
    ///
    /// # 参数
    /// - `parent`: 父构建器
    /// - `clause`: JOIN 子句
    ///
    /// # 返回
    /// 新的 JOIN 子句构建器实例
    pub fn new(parent: JoinBuilder, clause: JoinClause) -> Self {
        Self { parent, clause }
    }

    /// 添加 ON 条件
    ///
    /// # 参数
    /// - `local`: 本地字段
    /// - `operator`: 操作符
    /// - `foreign`: 外键字段
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn on(mut self, local: &str, operator: &str, foreign: &str) -> Self {
        self.clause.conditions.push(JoinCondition {
            local: local.to_string(),
            operator: operator.to_string(),
            foreign: foreign.to_string(),
            connector: "AND",
        });
        self
    }

    /// 添加 OR ON 条件
    ///
    /// # 参数
    /// - `local`: 本地字段
    /// - `operator`: 操作符
    /// - `foreign`: 外键字段
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn or_on(mut self, local: &str, operator: &str, foreign: &str) -> Self {
        self.clause.conditions.push(JoinCondition {
            local: local.to_string(),
            operator: operator.to_string(),
            foreign: foreign.to_string(),
            connector: "OR",
        });
        self
    }

    /// 添加原始 ON 条件
    ///
    /// # 参数
    /// - `raw`: 原始 SQL 条件
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn on_raw(mut self, raw: &str) -> Self {
        self.clause.conditions.push(JoinCondition {
            local: raw.to_string(),
            operator: String::new(),
            foreign: String::new(),
            connector: "AND",
        });
        self
    }

    /// 设置 USING 字段
    ///
    /// # 参数
    /// - `columns`: 字段名列表
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn using(mut self, columns: &[&str]) -> Self {
        self.clause.using = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 完成当前 JOIN 子句，返回父构建器
    ///
    /// # 返回
    /// 父构建器
    pub fn end(mut self) -> JoinBuilder {
        self.parent.joins.push(self.clause);
        self.parent
    }
}

/// 关联查询构建器
///
/// 用于构建基于模型关联的 JOIN 查询
#[derive(Debug, Clone)]
pub struct RelationJoinBuilder {
    /// 主表名
    pub main_table: String,
    /// 主表别名
    pub main_alias: Option<String>,
    /// JOIN 构建器
    pub join_builder: JoinBuilder,
}

impl RelationJoinBuilder {
    /// 创建新的关联查询构建器
    ///
    /// # 参数
    /// - `main_table`: 主表名
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的关联查询构建器实例
    pub fn new(main_table: &str, db_type: DatabaseType) -> Self {
        Self {
            main_table: main_table.to_string(),
            main_alias: None,
            join_builder: JoinBuilder::new(db_type),
        }
    }

    /// 设置主表别名
    ///
    /// # 参数
    /// - `alias`: 别名
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn alias(mut self, alias: &str) -> Self {
        self.main_alias = Some(alias.to_string());
        self
    }

    /// 添加 hasOne 关联 JOIN
    ///
    /// # 参数
    /// - `related_table`: 关联表名
    /// - `foreign_key`: 外键字段
    /// - `local_key`: 本地主键字段
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn has_one(self, related_table: &str, foreign_key: &str, local_key: &str) -> Self {
        let main_alias = self.main_alias.as_ref().unwrap_or(&self.main_table);
        let on_condition = format!(
            "{}.{} = {}.{}",
            self.quote_identifier(related_table),
            self.quote_identifier(foreign_key),
            self.quote_identifier(main_alias),
            self.quote_identifier(local_key)
        );

        let mut join_builder = self.join_builder;
        join_builder = join_builder.left_join(related_table).on_raw(&on_condition).end();
        Self {
            join_builder,
            ..self
        }
    }

    /// 添加 hasMany 关联 JOIN
    ///
    /// # 参数
    /// - `related_table`: 关联表名
    /// - `foreign_key`: 外键字段
    /// - `local_key`: 本地主键字段
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn has_many(self, related_table: &str, foreign_key: &str, local_key: &str) -> Self {
        self.has_one(related_table, foreign_key, local_key)
    }

    /// 添加 belongsTo 关联 JOIN
    ///
    /// # 参数
    /// - `related_table`: 关联表名
    /// - `foreign_key`: 本地外键字段
    /// - `owner_key`: 关联表主键字段
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn belongs_to(self, related_table: &str, foreign_key: &str, owner_key: &str) -> Self {
        let main_alias = self.main_alias.as_ref().unwrap_or(&self.main_table);
        let on_condition = format!(
            "{}.{} = {}.{}",
            self.quote_identifier(main_alias),
            self.quote_identifier(foreign_key),
            self.quote_identifier(related_table),
            self.quote_identifier(owner_key)
        );

        let mut join_builder = self.join_builder;
        join_builder = join_builder.left_join(related_table).on_raw(&on_condition).end();
        Self {
            join_builder,
            ..self
        }
    }

    /// 构建 JOIN SQL
    ///
    /// # 返回
    /// JOIN SQL 字符串
    pub fn build(&self) -> String {
        self.join_builder.build()
    }

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str) -> String {
        self.join_builder.quote_identifier(identifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_builder() {
        let builder = JoinBuilder::new(DatabaseType::MySQL)
            .join("posts")
            .on("users.id", "=", "posts.user_id")
            .end();

        let sql = builder.build();
        assert!(sql.contains("INNER JOIN"));
        assert!(sql.contains("ON"));
    }

    #[test]
    fn test_left_join() {
        let builder = JoinBuilder::new(DatabaseType::MySQL)
            .left_join("profiles")
            .on("users.id", "=", "profiles.user_id")
            .end();

        let sql = builder.build();
        assert!(sql.contains("LEFT JOIN"));
    }

    #[test]
    fn test_multiple_joins() {
        let builder = JoinBuilder::new(DatabaseType::MySQL)
            .join("posts")
            .on("users.id", "=", "posts.user_id")
            .end()
            .left_join("comments")
            .on("posts.id", "=", "comments.post_id")
            .end();

        let sql = builder.build();
        assert!(sql.contains("INNER JOIN"));
        assert!(sql.contains("LEFT JOIN"));
    }

    #[test]
    fn test_join_subquery() {
        let builder = JoinBuilder::new(DatabaseType::MySQL)
            .left_join_subquery(
                "SELECT user_id, COUNT(*) as post_count FROM posts GROUP BY user_id",
                "post_stats"
            )
            .on("users.id", "=", "post_stats.user_id")
            .end();

        let sql = builder.build();
        assert!(sql.contains("LEFT JOIN"));
        assert!(sql.contains("SELECT user_id, COUNT(*)"));
    }

    #[test]
    fn test_join_using() {
        let builder = JoinBuilder::new(DatabaseType::MySQL)
            .join("user_roles")
            .using(&["user_id"])
            .end();

        let sql = builder.build();
        assert!(sql.contains("USING"));
    }

    #[test]
    fn test_database_type_quote() {
        let mysql_builder = JoinBuilder::new(DatabaseType::MySQL);
        assert_eq!(mysql_builder.quote_identifier("table"), "`table`");

        let pg_builder = JoinBuilder::new(DatabaseType::PostgreSQL);
        assert_eq!(pg_builder.quote_identifier("table"), "\"table\"");
    }
}
