//! 排序扩展模块
//!
//! 提供丰富的排序查询构建方法
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：ORDER BY、原始排序、时间排序、随机排序等

use crate::interpreter::value::Value;

/// 排序方向枚举
///
/// 定义排序方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortDirection {
    /// 升序
    Asc,
    /// 降序
    Desc,
}

impl SortDirection {
    /// 获取排序方向字符串
    ///
    /// # 返回
    /// 排序方向字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        }
    }
}

/// 排序子句结构体
///
/// 存储单个排序条件
#[derive(Debug, Clone)]
pub struct OrderClause {
    /// 排序字段
    pub field: String,
    /// 排序方向
    pub direction: SortDirection,
    /// 是否为原始 SQL
    pub is_raw: bool,
    /// NULL 值排序位置
    pub nulls: Option<NullsPosition>,
}

impl OrderClause {
    /// 创建新的排序子句
    ///
    /// # 参数
    /// - `field`: 排序字段
    /// - `direction`: 排序方向
    ///
    /// # 返回
    /// 新的排序子句实例
    pub fn new(field: &str, direction: SortDirection) -> Self {
        Self {
            field: field.to_string(),
            direction,
            is_raw: false,
            nulls: None,
        }
    }

    /// 创建原始排序子句
    ///
    /// # 参数
    /// - `raw`: 原始 SQL
    ///
    /// # 返回
    /// 新的排序子句实例
    pub fn raw(raw: &str) -> Self {
        Self {
            field: raw.to_string(),
            direction: SortDirection::Asc,
            is_raw: true,
            nulls: None,
        }
    }

    /// 设置 NULL 值排序位置
    ///
    /// # 参数
    /// - `nulls`: NULL 值位置
    ///
    /// # 返回
    /// 修改后的排序子句
    pub fn nulls(mut self, nulls: NullsPosition) -> Self {
        self.nulls = Some(nulls);
        self
    }
}

/// NULL 值排序位置枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NullsPosition {
    /// NULL 值排在最前
    First,
    /// NULL 值排在最后
    Last,
}

/// 排序构建器
///
/// 提供链式调用的排序构建方法
#[derive(Debug, Clone)]
pub struct OrderBuilder {
    /// 排序子句列表
    pub orders: Vec<OrderClause>,
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

impl OrderBuilder {
    /// 创建新的排序构建器
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的排序构建器实例
    pub fn new(db_type: DatabaseType) -> Self {
        Self {
            orders: Vec::new(),
            db_type,
        }
    }

    /// 添加升序排序
    ///
    /// # 参数
    /// - `field`: 排序字段
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn order_asc(mut self, field: &str) -> Self {
        self.orders.push(OrderClause::new(field, SortDirection::Asc));
        self
    }

    /// 添加降序排序
    ///
    /// # 参数
    /// - `field`: 排序字段
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn order_desc(mut self, field: &str) -> Self {
        self.orders.push(OrderClause::new(field, SortDirection::Desc));
        self
    }

    /// 添加排序
    ///
    /// # 参数
    /// - `field`: 排序字段
    /// - `direction`: 排序方向
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn order(mut self, field: &str, direction: SortDirection) -> Self {
        self.orders.push(OrderClause::new(field, direction));
        self
    }

    /// 添加原始排序
    ///
    /// # 参数
    /// - `raw`: 原始排序 SQL
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn order_raw(mut self, raw: &str) -> Self {
        self.orders.push(OrderClause::raw(raw));
        self
    }

    /// 按时间倒序排序（最新记录在前）
    ///
    /// # 参数
    /// - `field`: 时间字段（默认 created_at）
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn latest(mut self, field: &str) -> Self {
        self.orders.push(OrderClause::new(field, SortDirection::Desc));
        self
    }

    /// 按时间正序排序（最早记录在前）
    ///
    /// # 参数
    /// - `field`: 时间字段（默认 created_at）
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn oldest(mut self, field: &str) -> Self {
        self.orders.push(OrderClause::new(field, SortDirection::Asc));
        self
    }

    /// 随机排序
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn in_random_order(mut self) -> Self {
        let random_sql = match self.db_type {
            DatabaseType::MySQL => "RAND()",
            DatabaseType::PostgreSQL => "RANDOM()",
            DatabaseType::SQLite => "RANDOM()",
        };
        self.orders.push(OrderClause::raw(random_sql));
        self
    }

    /// 重置排序
    ///
    /// 清除所有已设置的排序条件
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn reorder(mut self) -> Self {
        self.orders.clear();
        self
    }

    /// 设置 NULL 值排在最前
    ///
    /// # 参数
    /// - `field`: 排序字段
    /// - `direction`: 排序方向
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn order_nulls_first(mut self, field: &str, direction: SortDirection) -> Self {
        let mut clause = OrderClause::new(field, direction);
        clause.nulls = Some(NullsPosition::First);
        self.orders.push(clause);
        self
    }

    /// 设置 NULL 值排在最后
    ///
    /// # 参数
    /// - `field`: 排序字段
    /// - `direction`: 排序方向
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn order_nulls_last(mut self, field: &str, direction: SortDirection) -> Self {
        let mut clause = OrderClause::new(field, direction);
        clause.nulls = Some(NullsPosition::Last);
        self.orders.push(clause);
        self
    }

    /// 构建排序 SQL
    ///
    /// # 返回
    /// 排序 SQL 字符串
    pub fn build(&self) -> String {
        if self.orders.is_empty() {
            return String::new();
        }

        let order_parts: Vec<String> = self.orders.iter()
            .map(|o| self.build_order_clause(o))
            .collect();

        format!("ORDER BY {}", order_parts.join(", "))
    }

    /// 构建单个排序子句 SQL
    ///
    /// # 参数
    /// - `clause`: 排序子句
    ///
    /// # 返回
    /// 排序子句 SQL 字符串
    fn build_order_clause(&self, clause: &OrderClause) -> String {
        if clause.is_raw {
            return clause.field.clone();
        }

        let field = self.quote_identifier(&clause.field);
        let direction = clause.direction.as_str();

        // 处理 NULL 值排序位置
        let nulls_sql = match (&clause.nulls, self.db_type) {
            (Some(NullsPosition::First), DatabaseType::PostgreSQL) => " NULLS FIRST",
            (Some(NullsPosition::Last), DatabaseType::PostgreSQL) => " NULLS LAST",
            (Some(NullsPosition::First), DatabaseType::MySQL) => {
                // MySQL 使用 IS NULL 来模拟 NULLS FIRST
                return format!("{} IS NULL DESC, {} {}", field, field, direction);
            }
            (Some(NullsPosition::Last), DatabaseType::MySQL) => {
                // MySQL 使用 IS NULL 来模拟 NULLS LAST
                return format!("{} IS NULL ASC, {} {}", field, field, direction);
            }
            (Some(NullsPosition::First), DatabaseType::SQLite) => " NULLS FIRST",
            (Some(NullsPosition::Last), DatabaseType::SQLite) => " NULLS LAST",
            _ => "",
        };

        format!("{} {}{}", field, direction, nulls_sql)
    }

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }
}

/// 字段排序构建器
///
/// 用于构建复杂的多字段排序
#[derive(Debug, Clone)]
pub struct FieldOrderBuilder {
    /// 字段名
    pub field: String,
    /// 值排序映射
    pub values: Vec<(String, i32)>,
    /// 数据库类型
    pub db_type: DatabaseType,
}

impl FieldOrderBuilder {
    /// 创建新的字段排序构建器
    ///
    /// # 参数
    /// - `field`: 字段名
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的字段排序构建器实例
    pub fn new(field: &str, db_type: DatabaseType) -> Self {
        Self {
            field: field.to_string(),
            values: Vec::new(),
            db_type,
        }
    }

    /// 添加排序值
    ///
    /// # 参数
    /// - `value`: 字段值
    /// - `priority`: 排序优先级（数字越小越靠前）
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn add_value(mut self, value: &str, priority: i32) -> Self {
        self.values.push((value.to_string(), priority));
        self
    }

    /// 构建字段排序 SQL
    ///
    /// 使用 CASE WHEN 或 FIELD 函数实现自定义排序
    ///
    /// # 返回
    /// 字段排序 SQL 字符串
    pub fn build(&self) -> String {
        if self.values.is_empty() {
            return self.quote_identifier(&self.field);
        }

        match self.db_type {
            DatabaseType::MySQL => {
                // MySQL 使用 FIELD 函数
                let mut sorted_values: Vec<_> = self.values.iter().collect();
                sorted_values.sort_by_key(|(_, p)| *p);
                let values: Vec<String> = sorted_values.iter()
                    .map(|(v, _)| format!("'{}'", v))
                    .collect();
                format!("FIELD({}, {})", self.quote_identifier(&self.field), values.join(", "))
            }
            DatabaseType::PostgreSQL | DatabaseType::SQLite => {
                // PostgreSQL 和 SQLite 使用 CASE WHEN
                let mut sorted_values: Vec<_> = self.values.iter().enumerate().collect();
                sorted_values.sort_by_key(|(_, (_, p))| *p);
                let cases: Vec<String> = sorted_values.iter()
                    .map(|(i, (v, _))| format!("WHEN '{}' THEN {}", v, i))
                    .collect();
                format!(
                    "CASE {} {} ELSE {} END",
                    self.quote_identifier(&self.field),
                    cases.join(" "),
                    self.values.len()
                )
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

/// 排序规则
///
/// 定义排序规则配置
#[derive(Debug, Clone)]
pub struct SortRule {
    /// 排序字段
    pub field: String,
    /// 排序方向
    pub direction: SortDirection,
    /// 排序规则（COLLATE）
    pub collation: Option<String>,
    /// NULL 值位置
    pub nulls: Option<NullsPosition>,
}

impl SortRule {
    /// 创建新的排序规则
    ///
    /// # 参数
    /// - `field`: 排序字段
    /// - `direction`: 排序方向
    ///
    /// # 返回
    /// 新的排序规则实例
    pub fn new(field: &str, direction: SortDirection) -> Self {
        Self {
            field: field.to_string(),
            direction,
            collation: None,
            nulls: None,
        }
    }

    /// 设置排序规则
    ///
    /// # 参数
    /// - `collation`: 排序规则名称
    ///
    /// # 返回
    /// 修改后的排序规则
    pub fn collation(mut self, collation: &str) -> Self {
        self.collation = Some(collation.to_string());
        self
    }

    /// 设置 NULL 值位置
    ///
    /// # 参数
    /// - `nulls`: NULL 值位置
    ///
    /// # 返回
    /// 修改后的排序规则
    pub fn nulls(mut self, nulls: NullsPosition) -> Self {
        self.nulls = Some(nulls);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_builder() {
        let builder = OrderBuilder::new(DatabaseType::MySQL)
            .order_asc("name")
            .order_desc("created_at");

        let sql = builder.build();
        assert!(sql.contains("ORDER BY"));
        assert!(sql.contains("ASC"));
        assert!(sql.contains("DESC"));
    }

    #[test]
    fn test_latest() {
        let builder = OrderBuilder::new(DatabaseType::MySQL)
            .latest("created_at");

        let sql = builder.build();
        assert!(sql.contains("DESC"));
    }

    #[test]
    fn test_oldest() {
        let builder = OrderBuilder::new(DatabaseType::MySQL)
            .oldest("created_at");

        let sql = builder.build();
        assert!(sql.contains("ASC"));
    }

    #[test]
    fn test_random_order() {
        let mysql_builder = OrderBuilder::new(DatabaseType::MySQL).in_random_order();
        assert!(mysql_builder.build().contains("RAND()"));

        let pg_builder = OrderBuilder::new(DatabaseType::PostgreSQL).in_random_order();
        assert!(pg_builder.build().contains("RANDOM()"));
    }

    #[test]
    fn test_reorder() {
        let builder = OrderBuilder::new(DatabaseType::MySQL)
            .order_asc("name")
            .reorder()
            .order_desc("id");

        let sql = builder.build();
        // MySQL 使用反引号引用标识符
        assert!(sql.contains("`id` DESC"));
        assert!(!sql.contains("name"));
    }

    #[test]
    fn test_nulls_position() {
        let builder = OrderBuilder::new(DatabaseType::PostgreSQL)
            .order_nulls_first("deleted_at", SortDirection::Desc);

        let sql = builder.build();
        assert!(sql.contains("NULLS FIRST"));
    }
}
