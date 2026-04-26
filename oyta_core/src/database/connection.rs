//! 数据库连接管理模块
//!
//! 管理数据库连接池，支持 MySQL、PostgreSQL、SQLite
//! 使用 sqlx 实现异步数据库连接
//! 连接池在首次使用时惰性创建，避免启动时连接失败导致程序退出

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use std::collections::HashMap;
use std::sync::Arc;

/// 数据库连接配置
///
/// 对应 ThinkPHP 8.0 的 database.php 配置文件
/// 支持从配置文件自动加载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 连接类型（mysql、postgres、sqlite）
    pub r#type: String,
    /// 主机地址
    pub hostname: String,
    /// 数据库名
    pub database: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 端口号
    pub port: u16,
    /// 字符集
    pub charset: String,
    /// 表前缀
    pub prefix: String,
    /// 连接池大小
    pub pool_size: u32,
    /// 是否开启调试模式
    pub debug: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            r#type: "mysql".to_string(),
            hostname: "127.0.0.1".to_string(),
            database: "test".to_string(),
            username: "root".to_string(),
            password: String::new(),
            port: 3306,
            charset: "utf8mb4".to_string(),
            prefix: String::new(),
            pool_size: 10,
            debug: false,
        }
    }
}

impl DatabaseConfig {
    /// 从配置值构建数据库配置
    ///
    /// 从 ThinkPHP 风格的配置数组中提取数据库连接参数
    /// 支持的字段：type, hostname, database, username, password, hostport/port, charset, prefix
    ///
    /// # 参数
    /// - `config`: 配置值（关联数组形式）
    ///
    /// # 返回
    /// 构建好的 DatabaseConfig 实例
    pub fn from_config_value(config: &crate::symbol_table::types::ConfigValue) -> Self {
        let mut db_config = DatabaseConfig::default();

        if let Some(t) = config.get("type").and_then(|v| v.as_str()) {
            db_config.r#type = t.to_string();
        }
        if let Some(h) = config.get("hostname").and_then(|v| v.as_str()) {
            db_config.hostname = h.to_string();
        }
        if let Some(d) = config.get("database").and_then(|v| v.as_str()) {
            db_config.database = d.to_string();
        }
        if let Some(u) = config.get("username").and_then(|v| v.as_str()) {
            db_config.username = u.to_string();
        }
        if let Some(p) = config.get("password").and_then(|v| v.as_str()) {
            db_config.password = p.to_string();
        }
        if let Some(port) = config.get("hostport").and_then(|v| v.as_i64()) {
            db_config.port = port as u16;
        } else if let Some(port) = config.get("port").and_then(|v| v.as_i64()) {
            db_config.port = port as u16;
        }
        if let Some(c) = config.get("charset").and_then(|v| v.as_str()) {
            db_config.charset = c.to_string();
        }
        if let Some(p) = config.get("prefix").and_then(|v| v.as_str()) {
            db_config.prefix = p.to_string();
        }

        db_config
    }

    /// 构建 DSN（数据源名称）
    ///
    /// 根据数据库类型生成对应的连接字符串
    /// - MySQL: mysql://user:pass@host:port/db?charset=utf8mb4
    /// - PostgreSQL: postgres://user:pass@host:port/db
    /// - SQLite: sqlite:path
    ///
    /// # 返回
    /// DSN 连接字符串
    pub fn dsn(&self) -> String {
        match self.r#type.as_str() {
            "mysql" => format!(
                "mysql://{}:{}@{}:{}/{}?charset={}",
                self.username, self.password, self.hostname, self.port, self.database, self.charset
            ),
            "pgsql" | "postgres" => format!(
                "postgres://{}:{}@{}:{}/{}",
                self.username, self.password, self.hostname, self.port, self.database
            ),
            "sqlite" => format!("sqlite:{}", self.database),
            _ => format!(
                "mysql://{}:{}@{}:{}/{}",
                self.username, self.password, self.hostname, self.port, self.database
            ),
        }
    }
}

/// 数据库连接池管理器
///
/// 管理多个命名的数据库连接池
/// 支持从配置文件初始化，惰性创建连接池
/// 连接池创建后缓存在内存中，后续调用直接复用
pub struct ConnectionManager {
    /// 连接配置映射（连接名 → 配置）
    configs: HashMap<String, DatabaseConfig>,
    /// 已创建的连接池（连接名 → MySQL 连接池）
    pools: HashMap<String, Arc<MySqlPool>>,
    /// 默认连接名
    default_connection: String,
}

impl ConnectionManager {
    /// 创建新的连接管理器
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            pools: HashMap::new(),
            default_connection: "mysql".to_string(),
        }
    }

    /// 从配置存储初始化
    ///
    /// 读取 config 中的 database.default 和 database.connections 配置
    /// 不会立即创建连接池，连接池在首次使用时惰性创建
    ///
    /// # 参数
    /// - `config`: 配置存储引用
    pub fn init_from_config(&mut self, config: &crate::config::store::ConfigStore) {
        self.default_connection = config
            .get_string("database.default")
            .unwrap_or_else(|| "mysql".to_string());

        if let Some(connections) = config.get("database.connections") {
            if let crate::symbol_table::types::ConfigValue::AssociativeArray(map) = connections {
                for (name, conn_config) in map {
                    let db_config = DatabaseConfig::from_config_value(&conn_config);
                    self.configs.insert(name.clone(), db_config);
                }
            }
        }
    }

    /// 获取或创建指定名称的 MySQL 连接池
    ///
    /// 如果连接池已存在，直接返回缓存的连接池
    /// 如果不存在，根据配置创建新的连接池
    ///
    /// # 参数
    /// - `name`: 连接名称
    ///
    /// # 返回
    /// MySQL 连接池的 Arc 引用
    pub async fn get_pool(&mut self, name: &str) -> Result<Arc<MySqlPool>> {
        if let Some(pool) = self.pools.get(name) {
            return Ok(pool.clone());
        }

        let config = self
            .configs
            .get(name)
            .with_context(|| format!("数据库连接配置不存在: {}", name))?;

        let pool = self.create_pool(config).await?;
        let pool_arc = Arc::new(pool);
        self.pools.insert(name.to_string(), pool_arc.clone());

        Ok(pool_arc)
    }

    /// 获取或创建默认连接池
    ///
    /// # 返回
    /// 默认连接的 MySQL 连接池
    pub async fn get_default_pool(&mut self) -> Result<Arc<MySqlPool>> {
        self.get_pool(&self.default_connection.clone()).await
    }

    /// 创建 MySQL 连接池
    ///
    /// 根据 DatabaseConfig 创建 sqlx::MySqlPool
    /// 连接池大小由 config.pool_size 决定
    ///
    /// # 参数
    /// - `config`: 数据库配置
    ///
    /// # 返回
    /// 创建好的 MySQL 连接池
    async fn create_pool(&self, config: &DatabaseConfig) -> Result<MySqlPool> {
        let dsn = config.dsn();
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(config.pool_size)
            .connect(&dsn)
            .await
            .with_context(|| format!("无法连接数据库: {}", dsn))?;
        Ok(pool)
    }

    /// 获取默认连接配置
    pub fn default_config(&self) -> Option<&DatabaseConfig> {
        self.configs.get(&self.default_connection)
    }

    /// 获取指定连接配置
    pub fn get_config(&self, name: &str) -> Option<&DatabaseConfig> {
        self.configs.get(name)
    }

    /// 获取默认连接名
    pub fn default_connection_name(&self) -> &str {
        &self.default_connection
    }

    /// 获取所有连接名
    pub fn connection_names(&self) -> Vec<String> {
        self.configs.keys().cloned().collect()
    }

    /// 关闭所有连接池
    ///
    /// 依次关闭所有已创建的连接池，释放数据库连接资源
    pub async fn close_all(&mut self) {
        for (_, pool) in self.pools.drain() {
            let pool = Arc::try_unwrap(pool).unwrap_or_else(|arc| (*arc).clone());
            pool.close().await;
        }
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局数据库连接管理器
///
/// 使用 Lazy + RwLock 实现线程安全的全局单例
/// 所有数据库操作通过此管理器获取连接池
static DB_MANAGER: Lazy<RwLock<ConnectionManager>> = Lazy::new(|| RwLock::new(ConnectionManager::new()));

/// 获取全局连接管理器的读锁
pub fn get_manager() -> parking_lot::RwLockReadGuard<'static, ConnectionManager> {
    DB_MANAGER.read()
}

/// 获取全局连接管理器的写锁
pub fn get_manager_mut() -> parking_lot::RwLockWriteGuard<'static, ConnectionManager> {
    DB_MANAGER.write()
}

/// 初始化全局数据库连接管理器
///
/// 从配置存储中加载数据库配置
/// 应在应用启动时调用
///
/// # 参数
/// - `config`: 配置存储引用
pub fn init_db(config: &crate::config::store::ConfigStore) {
    let mut manager = DB_MANAGER.write();
    manager.init_from_config(config);
}
