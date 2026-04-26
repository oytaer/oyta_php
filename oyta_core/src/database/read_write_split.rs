//! 数据库读写分离模块
//!
//! 实现数据库主从架构的读写分离
//! 写操作（INSERT/UPDATE/DELETE）走主库
//! 读操作（SELECT）走从库
//! 支持多个从库的负载均衡
//! 支持强制走主库的场景

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::interpreter::value::Value;
use super::pool_trait::{DatabasePool, DatabaseTransaction, QueryResult};
use super::postgres_pool::PostgresPool;
use super::sqlite_pool::SqlitePoolWrapper;

/// 数据库连接池枚举
///
/// 统一封装不同数据库的连接池
#[derive(Clone)]
pub enum PoolType {
    /// MySQL 连接池
    MySQL(Arc<sqlx::MySqlPool>),
    /// PostgreSQL 连接池
    PostgreSQL(Arc<PostgresPool>),
    /// SQLite 连接池
    SQLite(Arc<SqlitePoolWrapper>),
}

/// 读写分离管理器
///
/// 管理主库和从库的连接池
/// 自动将读操作路由到从库，写操作路由到主库
/// 支持强制走主库的场景
pub struct ReadWriteSplitter {
    /// 主库连接池
    master: Arc<dyn DatabasePool>,
    /// 从库连接池列表
    slaves: Vec<Arc<dyn DatabasePool>>,
    /// 负载均衡策略
    balancer: LoadBalanceStrategy,
    /// 当前从库索引（用于轮询）
    current_slave_index: Arc<RwLock<usize>>,
    /// 是否强制走主库（ThreadLocal 模拟）
    force_master: Arc<RwLock<bool>>,
}

impl ReadWriteSplitter {
    /// 创建新的读写分离管理器
    ///
    /// # 参数
    /// - `master`: 主库连接池
    /// - `slaves`: 从库连接池列表
    /// - `balancer`: 负载均衡策略
    pub fn new(
        master: Arc<dyn DatabasePool>,
        slaves: Vec<Arc<dyn DatabasePool>>,
        balancer: LoadBalanceStrategy,
    ) -> Self {
        Self {
            master,
            slaves,
            balancer,
            current_slave_index: Arc::new(RwLock::new(0)),
            force_master: Arc::new(RwLock::new(false)),
        }
    }

    /// 创建只有主库的实例（无读写分离）
    pub fn single(master: Arc<dyn DatabasePool>) -> Self {
        Self {
            master,
            slaves: Vec::new(),
            balancer: LoadBalanceStrategy::RoundRobin,
            current_slave_index: Arc::new(RwLock::new(0)),
            force_master: Arc::new(RwLock::new(false)),
        }
    }

    /// 获取主库连接池
    pub fn master(&self) -> &Arc<dyn DatabasePool> {
        &self.master
    }

    /// 获取从库连接池（根据负载均衡策略）
    pub async fn slave(&self) -> Arc<dyn DatabasePool> {
        // 如果没有从库或强制走主库，返回主库
        if self.slaves.is_empty() || *self.force_master.read().await {
            return self.master.clone();
        }

        match self.balancer {
            LoadBalanceStrategy::RoundRobin => self.round_robin_slave().await,
            LoadBalanceStrategy::Random => self.random_slave().await,
            LoadBalanceStrategy::Weighted(ref weights) => self.weighted_slave(weights).await,
            LoadBalanceStrategy::LeastConnections => self.least_connections_slave().await,
        }
    }

    /// 轮询选择从库
    async fn round_robin_slave(&self) -> Arc<dyn DatabasePool> {
        let mut index = self.current_slave_index.write().await;
        *index = (*index + 1) % self.slaves.len();
        self.slaves[*index].clone()
    }

    /// 随机选择从库
    async fn random_slave(&self) -> Arc<dyn DatabasePool> {
        use rand::Rng;
        let index = rand::rng().random_range(0..self.slaves.len());
        self.slaves[index].clone()
    }

    /// 加权选择从库
    async fn weighted_slave(&self, weights: &[u32]) -> Arc<dyn DatabasePool> {
        use rand::Rng;
        let total_weight: u32 = weights.iter().sum();
        let mut random = rand::rng().random_range(0..total_weight);

        for (i, &weight) in weights.iter().enumerate() {
            if random < weight {
                return self.slaves[i].clone();
            }
            random -= weight;
        }

        self.slaves[0].clone()
    }

    /// 最少连接数选择从库
    async fn least_connections_slave(&self) -> Arc<dyn DatabasePool> {
        let mut min_connections = u32::MAX;
        let mut selected_index = 0;

        for (i, slave) in self.slaves.iter().enumerate() {
            let (active, _, _) = slave.pool_status();
            if active < min_connections {
                min_connections = active;
                selected_index = i;
            }
        }

        self.slaves[selected_index].clone()
    }

    /// 设置强制走主库
    pub async fn set_force_master(&self, force: bool) {
        *self.force_master.write().await = force;
    }

    /// 获取是否强制走主库
    pub async fn is_force_master(&self) -> bool {
        *self.force_master.read().await
    }

    /// 判断 SQL 是否为读操作
    ///
    /// # 参数
    /// - `sql`: SQL 语句
    ///
    /// # 返回
    /// 是否为读操作
    pub fn is_read_operation(sql: &str) -> bool {
        let sql_upper = sql.to_uppercase();
        let sql_trimmed = sql_upper.trim();

        // SELECT, SHOW, EXPLAIN, DESCRIBE 为读操作
        sql_trimmed.starts_with("SELECT")
            || sql_trimmed.starts_with("SHOW")
            || sql_trimmed.starts_with("EXPLAIN")
            || sql_trimmed.starts_with("DESCRIBE")
            || sql_trimmed.starts_with("DESC")
    }

    /// 判断 SQL 是否为写操作
    pub fn is_write_operation(sql: &str) -> bool {
        !Self::is_read_operation(sql)
    }

    /// 获取从库数量
    pub fn slave_count(&self) -> usize {
        self.slaves.len()
    }

    /// 检查所有从库健康状态
    pub async fn check_slaves_health(&self) -> Vec<(usize, bool)> {
        let mut results = Vec::new();
        for (i, slave) in self.slaves.iter().enumerate() {
            let healthy = slave.is_healthy().await;
            results.push((i, healthy));
        }
        results
    }

    /// 移除不健康的从库
    pub async fn remove_unhealthy_slaves(&mut self) {
        let mut healthy_slaves = Vec::new();

        for slave in self.slaves.drain(..) {
            if slave.is_healthy().await {
                healthy_slaves.push(slave);
            }
        }

        self.slaves = healthy_slaves;
    }
}

/// 负载均衡策略
#[derive(Debug, Clone)]
pub enum LoadBalanceStrategy {
    /// 轮询（Round Robin）
    /// 按顺序依次选择从库
    RoundRobin,

    /// 随机选择
    Random,

    /// 加权轮询
    /// 根据权重分配请求
    Weighted(Vec<u32>),

    /// 最少连接数
    /// 选择当前连接数最少的从库
    LeastConnections,
}

impl Default for LoadBalanceStrategy {
    fn default() -> Self {
        LoadBalanceStrategy::RoundRobin
    }
}

/// 读写分离代理
///
/// 实现 DatabasePool trait
/// 自动将读写操作路由到正确的数据库
pub struct ReadWriteProxy {
    /// 读写分离管理器
    splitter: Arc<ReadWriteSplitter>,
}

impl ReadWriteProxy {
    /// 创建新的读写分离代理
    pub fn new(splitter: Arc<ReadWriteSplitter>) -> Self {
        Self { splitter }
    }

    /// 获取读写分离管理器
    pub fn splitter(&self) -> &ReadWriteSplitter {
        &self.splitter
    }

    /// 强制走主库执行操作
    ///
    /// # 参数
    /// - `f`: 要执行的操作
    pub async fn with_master<F, T>(&self, f: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        self.splitter.set_force_master(true).await;
        let result = f.await;
        self.splitter.set_force_master(false).await;
        result
    }
}

#[async_trait]
impl DatabasePool for ReadWriteProxy {
    /// 执行查询（自动路由到从库）
    async fn query(&self, sql: &str) -> Result<QueryResult> {
        let pool = if ReadWriteSplitter::is_read_operation(sql) {
            self.splitter.slave().await
        } else {
            self.splitter.master().clone()
        };

        pool.query(sql).await
    }

    /// 执行参数化查询（自动路由到从库）
    async fn query_with_params(&self, sql: &str, params: &[Value]) -> Result<QueryResult> {
        let pool = if ReadWriteSplitter::is_read_operation(sql) {
            self.splitter.slave().await
        } else {
            self.splitter.master().clone()
        };

        pool.query_with_params(sql, params).await
    }

    /// 执行更新（路由到主库）
    async fn execute(&self, sql: &str) -> Result<u64> {
        self.splitter.master().execute(sql).await
    }

    /// 执行参数化更新（路由到主库）
    async fn execute_with_params(&self, sql: &str, params: &[Value]) -> Result<u64> {
        self.splitter.master().execute_with_params(sql, params).await
    }

    /// 执行 INSERT 并返回 ID（路由到主库）
    async fn insert_get_id(&self, sql: &str) -> Result<i64> {
        self.splitter.master().insert_get_id(sql).await
    }

    /// 开启事务（在主库上）
    async fn begin_transaction(&self) -> Result<Box<dyn DatabaseTransaction + Send>> {
        self.splitter.master().begin_transaction().await
    }

    /// 获取连接池状态
    fn pool_status(&self) -> (u32, u32, u32) {
        self.splitter.master().pool_status()
    }

    /// 检查连接是否可用
    async fn is_healthy(&self) -> bool {
        self.splitter.master().is_healthy().await
    }

    /// 获取数据库类型
    fn database_type(&self) -> &str {
        self.splitter.master().database_type()
    }
}

/// 主从配置
#[derive(Debug, Clone)]
pub struct MasterSlaveConfig {
    /// 主库连接配置
    pub master: DatabaseConnectionConfig,
    /// 从库连接配置列表
    pub slaves: Vec<DatabaseConnectionConfig>,
    /// 负载均衡策略
    pub balance_strategy: LoadBalanceStrategy,
}

/// 数据库连接配置
#[derive(Debug, Clone)]
pub struct DatabaseConnectionConfig {
    /// 连接名称
    pub name: String,
    /// 数据库类型
    pub db_type: String,
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 数据库名
    pub database: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 字符集
    pub charset: String,
    /// 连接池大小
    pub pool_size: u32,
}

impl DatabaseConnectionConfig {
    /// 构建 DSN 连接字符串
    pub fn dsn(&self) -> String {
        match self.db_type.as_str() {
            "mysql" => format!(
                "mysql://{}:{}@{}:{}/{}?charset={}",
                self.username, self.password, self.host, self.port, self.database, self.charset
            ),
            "pgsql" | "postgres" | "postgresql" => format!(
                "postgres://{}:{}@{}:{}/{}",
                self.username, self.password, self.host, self.port, self.database
            ),
            "sqlite" => format!("sqlite:{}", self.database),
            _ => format!(
                "mysql://{}:{}@{}:{}/{}",
                self.username, self.password, self.host, self.port, self.database
            ),
        }
    }
}

impl Default for DatabaseConnectionConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            db_type: "mysql".to_string(),
            host: "127.0.0.1".to_string(),
            port: 3306,
            database: "test".to_string(),
            username: "root".to_string(),
            password: String::new(),
            charset: "utf8mb4".to_string(),
            pool_size: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_read_operation() {
        assert!(ReadWriteSplitter::is_read_operation("SELECT * FROM users"));
        assert!(ReadWriteSplitter::is_read_operation("  SELECT id FROM users"));
        assert!(ReadWriteSplitter::is_read_operation("SHOW TABLES"));
        assert!(ReadWriteSplitter::is_read_operation("EXPLAIN SELECT * FROM users"));
        assert!(ReadWriteSplitter::is_read_operation("DESCRIBE users"));

        assert!(!ReadWriteSplitter::is_read_operation("INSERT INTO users VALUES (1)"));
        assert!(!ReadWriteSplitter::is_read_operation("UPDATE users SET name = 'test'"));
        assert!(!ReadWriteSplitter::is_read_operation("DELETE FROM users"));
    }
}
