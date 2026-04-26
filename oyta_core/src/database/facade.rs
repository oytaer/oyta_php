//! 数据库 Db 门面模块
//!
//! 对应 ThinkPHP 8.0 的 Db 门面
//! 提供静态方法入口，简化数据库操作
//! 支持：query/execute/table/transaction 等操作

use anyhow::Result;
use std::collections::HashMap;

use crate::interpreter::value::Value;
use super::connection::{self, DatabaseConfig};
use super::executor::{self, QueryResult};
use super::query_builder::QueryBuilder;

/// Db 门面
///
/// 对应 ThinkPHP 8.0 的 \oyta\Db 类
/// 提供静态方法入口，所有方法委托给底层模块
///
/// 使用示例（PHP 代码风格）：
/// ```php
/// Db::query("SELECT * FROM users WHERE id = ?", [1]);
/// Db::table("users")->where("id", ">", 0)->select();
/// Db::execute("INSERT INTO users (name) VALUES (?)", ["test"]);
/// ```
pub struct Db;

impl Db {
    /// 初始化数据库连接管理器
    ///
    /// 从配置存储中加载数据库配置
    /// 应在应用启动时调用
    ///
    /// # 参数
    /// - `config`: 配置存储引用
    pub fn init(config: &crate::config::store::ConfigStore) {
        connection::init_db(config);
    }

    /// 获取默认连接配置
    pub fn get_default_config() -> Option<DatabaseConfig> {
        connection::get_manager().default_config().cloned()
    }

    /// 获取指定连接配置
    pub fn get_config(name: &str) -> Option<DatabaseConfig> {
        connection::get_manager().get_config(name).cloned()
    }

    /// 获取默认连接的 DSN
    pub fn dsn() -> Option<String> {
        connection::get_manager().default_config().map(|c| c.dsn())
    }

    /// 获取表前缀
    pub fn get_prefix() -> String {
        connection::get_manager()
            .default_config()
            .map(|c| c.prefix.clone())
            .unwrap_or_default()
    }

    /// 执行 SQL 查询（SELECT）
    ///
    /// # 参数
    /// - `sql`: SQL 查询语句
    ///
    /// # 返回
    /// 查询结果
    pub async fn query(sql: &str) -> Result<QueryResult> {
        executor::query(sql).await
    }

    /// 执行参数化 SQL 查询
    ///
    /// # 参数
    /// - `sql`: 带占位符的 SQL 查询语句
    /// - `params`: 参数值列表
    ///
    /// # 返回
    /// 查询结果
    pub async fn query_with_params(sql: &str, params: &[Value]) -> Result<QueryResult> {
        executor::query_with_params(sql, params).await
    }

    /// 执行 SQL 更新（INSERT/UPDATE/DELETE）
    ///
    /// # 参数
    /// - `sql`: SQL 更新语句
    ///
    /// # 返回
    /// 受影响的行数
    pub async fn execute(sql: &str) -> Result<u64> {
        executor::execute(sql).await
    }

    /// 执行参数化 SQL 更新
    ///
    /// # 参数
    /// - `sql`: 带占位符的 SQL 更新语句
    /// - `params`: 参数值列表
    ///
    /// # 返回
    /// 受影响的行数
    pub async fn execute_with_params(sql: &str, params: &[Value]) -> Result<u64> {
        executor::execute_with_params(sql, params).await
    }

    /// 执行 INSERT 并返回自增 ID
    ///
    /// # 参数
    /// - `sql`: INSERT SQL 语句
    ///
    /// # 返回
    /// 最后插入的自增 ID
    pub async fn insert_get_id(sql: &str) -> Result<i64> {
        executor::insert_get_id(sql).await
    }

    /// 查询单行数据
    ///
    /// # 参数
    /// - `sql`: SQL 查询语句
    ///
    /// # 返回
    /// 单行数据
    pub async fn find(sql: &str) -> Result<Option<HashMap<String, Value>>> {
        executor::query_one(sql).await
    }

    /// 查询单个标量值
    ///
    /// # 参数
    /// - `sql`: SQL 查询语句
    ///
    /// # 返回
    /// 标量值
    pub async fn value(sql: &str) -> Result<Value> {
        executor::query_scalar(sql).await
    }

    /// 创建查询构建器
    ///
    /// 对应 ThinkPHP 的 Db::table('users')
    /// 自动添加表前缀
    ///
    /// # 参数
    /// - `table`: 表名（不含前缀）
    ///
    /// # 返回
    /// 查询构建器实例
    pub fn table(table: &str) -> QueryBuilder {
        let prefix = Self::get_prefix();
        let full_table = format!("{}{}", prefix, table);
        QueryBuilder::new(&full_table)
    }

    /// 创建查询构建器（指定完整表名）
    ///
    /// 不添加表前缀，直接使用传入的表名
    ///
    /// # 参数
    /// - `table`: 完整表名
    ///
    /// # 返回
    /// 查询构建器实例
    pub fn name(table: &str) -> QueryBuilder {
        QueryBuilder::new(table)
    }

    /// 关闭所有数据库连接
    ///
    /// 应在应用退出时调用，释放所有连接资源
    pub async fn close() {
        let mut manager = connection::get_manager_mut();
        manager.close_all().await;
    }
}
