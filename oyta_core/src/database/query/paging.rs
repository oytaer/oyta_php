//! 分页查询模块
//!
//! 提供数据库分页查询功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//! 包括：基础分页、简单分页、游标分页等

use anyhow::Result;
use std::collections::HashMap;

use crate::interpreter::value::Value;

/// 分页结果结构体
///
/// 存储分页查询的结果和元数据
#[derive(Debug, Clone)]
pub struct PaginateResult {
    /// 数据行列表
    pub data: Vec<HashMap<String, Value>>,
    /// 当前页码
    pub current_page: u64,
    /// 每页数量
    pub per_page: u64,
    /// 总记录数
    pub total: u64,
    /// 最后一页页码
    pub last_page: u64,
    /// 总页数
    pub total_pages: u64,
    /// 是否有下一页
    pub has_more: bool,
    /// 起始记录位置
    pub from: u64,
    /// 结束记录位置
    pub to: u64,
}

impl PaginateResult {
    /// 创建新的分页结果
    ///
    /// # 参数
    /// - `data`: 数据行列表
    /// - `total`: 总记录数
    /// - `current_page`: 当前页码
    /// - `per_page`: 每页数量
    ///
    /// # 返回
    /// 新的分页结果实例
    pub fn new(
        data: Vec<HashMap<String, Value>>,
        total: u64,
        current_page: u64,
        per_page: u64,
    ) -> Self {
        // 计算总页数
        let total_pages = if per_page > 0 {
            (total + per_page - 1) / per_page
        } else {
            1
        };

        // 计算起始和结束位置
        let from = if data.is_empty() {
            0
        } else {
            (current_page - 1) * per_page + 1
        };
        let to = if data.is_empty() {
            0
        } else {
            from + data.len() as u64 - 1
        };

        Self {
            has_more: current_page < total_pages,
            last_page: total_pages,
            data,
            current_page,
            per_page,
            total,
            total_pages,
            from,
            to,
        }
    }

    /// 判断是否为空
    ///
    /// # 返回
    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 获取数据数量
    ///
    /// # 返回
    /// 当前页数据数量
    pub fn count(&self) -> usize {
        self.data.len()
    }
}

/// 简单分页结果结构体
///
/// 不包含总记录数的轻量级分页结果
#[derive(Debug, Clone)]
pub struct SimplePaginateResult {
    /// 数据行列表
    pub data: Vec<HashMap<String, Value>>,
    /// 当前页码
    pub current_page: u64,
    /// 每页数量
    pub per_page: u64,
    /// 是否有下一页
    pub has_more: bool,
}

impl SimplePaginateResult {
    /// 创建新的简单分页结果
    ///
    /// # 参数
    /// - `data`: 数据行列表
    /// - `current_page`: 当前页码
    /// - `per_page`: 每页数量
    ///
    /// # 返回
    /// 新的简单分页结果实例
    pub fn new(
        data: Vec<HashMap<String, Value>>,
        current_page: u64,
        per_page: u64,
    ) -> Self {
        // 判断是否有更多数据（查询时多取一条）
        let has_more = data.len() > per_page as usize;
        let actual_data = if has_more {
            data.into_iter().take(per_page as usize).collect()
        } else {
            data
        };

        Self {
            data: actual_data,
            current_page,
            per_page,
            has_more,
        }
    }

    /// 判断是否为空
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// 分页构建器
///
/// 提供链式调用的分页构建方法
#[derive(Debug, Clone)]
pub struct PaginateBuilder {
    /// 当前页码
    pub page: u64,
    /// 每页数量
    pub per_page: u64,
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

impl PaginateBuilder {
    /// 创建新的分页构建器
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的分页构建器实例
    pub fn new(db_type: DatabaseType) -> Self {
        Self {
            page: 1,
            per_page: 15,
            db_type,
        }
    }

    /// 设置页码
    ///
    /// # 参数
    /// - `page`: 页码（从 1 开始）
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn page(mut self, page: u64) -> Self {
        self.page = if page < 1 { 1 } else { page };
        self
    }

    /// 设置每页数量
    ///
    /// # 参数
    /// - `per_page`: 每页数量
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn per_page(mut self, per_page: u64) -> Self {
        self.per_page = if per_page < 1 { 15 } else { per_page };
        self
    }

    /// 构建 LIMIT OFFSET SQL
    ///
    /// # 返回
    /// (LIMIT OFFSET SQL, OFFSET 值)
    pub fn build_limit_offset(&self) -> (String, u64) {
        let offset = (self.page - 1) * self.per_page;
        let sql = format!("LIMIT {} OFFSET {}", self.per_page, offset);
        (sql, offset)
    }

    /// 构建 MySQL 风格的分页 SQL
    ///
    /// # 返回
    /// (分页 SQL, OFFSET 值)
    pub fn build_mysql_limit(&self) -> (String, u64) {
        let offset = (self.page - 1) * self.per_page;
        let sql = format!("LIMIT {}, {}", offset, self.per_page);
        (sql, offset)
    }

    /// 构建计数查询 SQL
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `where_sql`: WHERE 条件 SQL
    ///
    /// # 返回
    /// 计数查询 SQL
    pub fn build_count_sql(&self, table: &str, where_sql: &str) -> String {
        let mut sql = format!("SELECT COUNT(*) AS total FROM {}", self.quote_identifier(table));
        if !where_sql.is_empty() {
            sql.push_str(&format!(" {}", where_sql));
        }
        sql
    }

    /// 计算偏移量
    ///
    /// # 返回
    /// OFFSET 值
    pub fn offset(&self) -> u64 {
        (self.page - 1) * self.per_page
    }

    /// 引用标识符
    fn quote_identifier(&self, identifier: &str) -> String {
        match self.db_type {
            DatabaseType::MySQL => format!("`{}`", identifier),
            DatabaseType::PostgreSQL | DatabaseType::SQLite => format!("\"{}\"", identifier),
        }
    }
}

/// 游标分页结果结构体
///
/// 基于游标的分页结果，适用于大数据量场景
#[derive(Debug, Clone)]
pub struct CursorPaginateResult {
    /// 数据行列表
    pub data: Vec<HashMap<String, Value>>,
    /// 下一页游标
    pub next_cursor: Option<String>,
    /// 上一页游标
    pub prev_cursor: Option<String>,
    /// 是否有下一页
    pub has_more: bool,
}

impl CursorPaginateResult {
    /// 创建新的游标分页结果
    ///
    /// # 参数
    /// - `data`: 数据行列表
    /// - `cursor_field`: 游标字段名
    /// - `has_more`: 是否有更多数据
    ///
    /// # 返回
    /// 新的游标分页结果实例
    pub fn new(
        data: Vec<HashMap<String, Value>>,
        cursor_field: &str,
        has_more: bool,
    ) -> Self {
        let next_cursor = data.last().and_then(|row| {
            row.get(cursor_field).and_then(|v| match v {
                Value::Int(i) => Some(i.to_string()),
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
        });

        let prev_cursor = data.first().and_then(|row| {
            row.get(cursor_field).and_then(|v| match v {
                Value::Int(i) => Some(i.to_string()),
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
        });

        Self {
            data,
            next_cursor,
            prev_cursor,
            has_more,
        }
    }
}

/// 游标分页构建器
///
/// 用于构建基于游标的分页查询
#[derive(Debug, Clone)]
pub struct CursorPaginateBuilder {
    /// 游标字段名
    pub cursor_field: String,
    /// 每页数量
    pub per_page: u64,
    /// 游标值
    pub cursor: Option<String>,
    /// 排序方向
    pub direction: CursorDirection,
    /// 数据库类型
    pub db_type: DatabaseType,
}

/// 游标排序方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorDirection {
    /// 正向（下一页）
    Forward,
    /// 反向（上一页）
    Backward,
}

impl CursorPaginateBuilder {
    /// 创建新的游标分页构建器
    ///
    /// # 参数
    /// - `cursor_field`: 游标字段名
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// 新的游标分页构建器实例
    pub fn new(cursor_field: &str, db_type: DatabaseType) -> Self {
        Self {
            cursor_field: cursor_field.to_string(),
            per_page: 15,
            cursor: None,
            direction: CursorDirection::Forward,
            db_type,
        }
    }

    /// 设置每页数量
    ///
    /// # 参数
    /// - `per_page`: 每页数量
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn per_page(mut self, per_page: u64) -> Self {
        self.per_page = if per_page < 1 { 15 } else { per_page };
        self
    }

    /// 设置游标
    ///
    /// # 参数
    /// - `cursor`: 游标值
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn cursor(mut self, cursor: &str) -> Self {
        self.cursor = Some(cursor.to_string());
        self
    }

    /// 设置为正向分页
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn forward(mut self) -> Self {
        self.direction = CursorDirection::Forward;
        self
    }

    /// 设置为反向分页
    ///
    /// # 返回
    /// 构建器自身，支持链式调用
    pub fn backward(mut self) -> Self {
        self.direction = CursorDirection::Backward;
        self
    }

    /// 构建游标分页 SQL
    ///
    /// # 参数
    /// - `table`: 表名
    /// - `where_sql`: WHERE 条件 SQL
    ///
    /// # 返回
    /// (分页 SQL, 绑定参数)
    pub fn build(&self, table: &str, where_sql: &str) -> (String, Vec<Value>) {
        let mut sql = format!("SELECT * FROM {}", self.quote_identifier(table));
        let mut bindings = Vec::new();
        let mut conditions = Vec::new();

        // 添加原有 WHERE 条件
        if !where_sql.is_empty() {
            conditions.push(where_sql.to_string());
        }

        // 添加游标条件
        if let Some(cursor) = &self.cursor {
            let operator = match self.direction {
                CursorDirection::Forward => ">",
                CursorDirection::Backward => "<",
            };
            conditions.push(format!("{} {} ?", self.quote_identifier(&self.cursor_field), operator));
            bindings.push(Value::String(cursor.clone()));
        }

        // 添加 WHERE 子句
        if !conditions.is_empty() {
            sql.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
        }

        // 添加排序
        let order_direction = match self.direction {
            CursorDirection::Forward => "ASC",
            CursorDirection::Backward => "DESC",
        };
        sql.push_str(&format!(" ORDER BY {} {}", self.quote_identifier(&self.cursor_field), order_direction));

        // 添加 LIMIT（多取一条用于判断是否有下一页）
        sql.push_str(&format!(" LIMIT {}", self.per_page + 1));

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

/// 分页配置
///
/// 定义分页的默认配置
#[derive(Debug, Clone)]
pub struct PaginateConfig {
    /// 默认每页数量
    pub default_per_page: u64,
    /// 最大每页数量
    pub max_per_page: u64,
}

impl Default for PaginateConfig {
    fn default() -> Self {
        Self {
            default_per_page: 15,
            max_per_page: 100,
        }
    }
}

impl PaginateConfig {
    /// 创建新的分页配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置默认每页数量
    pub fn default_per_page(mut self, n: u64) -> Self {
        self.default_per_page = n;
        self
    }

    /// 设置最大每页数量
    pub fn max_per_page(mut self, n: u64) -> Self {
        self.max_per_page = n;
        self
    }

    /// 规范化每页数量
    ///
    /// # 参数
    /// - `per_page`: 请求的每页数量
    ///
    /// # 返回
    /// 规范化后的每页数量
    pub fn normalize_per_page(&self, per_page: Option<u64>) -> u64 {
        match per_page {
            Some(n) if n > 0 && n <= self.max_per_page => n,
            Some(n) if n > self.max_per_page => self.max_per_page,
            _ => self.default_per_page,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginate_result() {
        let data = vec![
            HashMap::from([("id".to_string(), Value::Int(1))]),
            HashMap::from([("id".to_string(), Value::Int(2))]),
        ];

        let result = PaginateResult::new(data, 100, 1, 15);

        assert_eq!(result.current_page, 1);
        assert_eq!(result.per_page, 15);
        assert_eq!(result.total, 100);
        assert_eq!(result.total_pages, 7);
        assert!(result.has_more);
    }

    #[test]
    fn test_simple_paginate_result() {
        let data = vec![
            HashMap::from([("id".to_string(), Value::Int(1))]),
            HashMap::from([("id".to_string(), Value::Int(2))]),
        ];

        let result = SimplePaginateResult::new(data, 1, 1);

        assert_eq!(result.data.len(), 1);
        assert!(result.has_more);
    }

    #[test]
    fn test_paginate_builder() {
        let builder = PaginateBuilder::new(DatabaseType::MySQL)
            .page(2)
            .per_page(10);

        let (sql, offset) = builder.build_limit_offset();
        assert!(sql.contains("LIMIT 10 OFFSET 10"));
        assert_eq!(offset, 10);
    }

    #[test]
    fn test_paginate_builder_offset() {
        let builder = PaginateBuilder::new(DatabaseType::MySQL)
            .page(3)
            .per_page(20);

        assert_eq!(builder.offset(), 40);
    }

    #[test]
    fn test_cursor_paginate_builder() {
        let builder = CursorPaginateBuilder::new("id", DatabaseType::MySQL)
            .per_page(10)
            .cursor("100")
            .forward();

        let (sql, bindings) = builder.build("users", "");
        // MySQL 使用反引号引用标识符
        assert!(sql.contains("`id` > ?"));
        assert!(sql.contains("ORDER BY `id` ASC"));
        assert_eq!(bindings.len(), 1);
    }

    #[test]
    fn test_paginate_config() {
        let config = PaginateConfig::new()
            .default_per_page(20)
            .max_per_page(50);

        assert_eq!(config.normalize_per_page(None), 20);
        assert_eq!(config.normalize_per_page(Some(30)), 30);
        assert_eq!(config.normalize_per_page(Some(100)), 50);
    }
}
