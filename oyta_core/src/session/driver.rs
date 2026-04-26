//! Session 驱动模块
//!
//! 提供 Session 存储驱动
//! 支持：内存、文件、Redis 三种驱动

use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use std::collections::HashMap;

/// Session 驱动 trait
///
/// 定义 Session 存储的统一接口
/// 所有驱动必须实现此 trait
pub trait SessionDriver: Send + Sync {
    /// 读取 Session 数据
    fn read(&self, session_id: &str) -> Result<HashMap<String, String>>;
    /// 写入 Session 数据
    fn write(&self, session_id: &str, data: &HashMap<String, String>) -> Result<()>;
    /// 销毁 Session
    fn destroy(&self, session_id: &str) -> Result<()>;
    /// 垃圾回收
    fn gc(&self, max_lifetime: u64) -> Result<()>;
}

/// 内存 Session 驱动
///
/// 使用 DashMap 在内存中存储 Session 数据
/// 适用于单进程开发和测试环境
/// 不支持多进程间共享
pub struct MemorySessionDriver {
    sessions: dashmap::DashMap<String, HashMap<String, String>>,
}

impl MemorySessionDriver {
    pub fn new() -> Self {
        Self {
            sessions: dashmap::DashMap::new(),
        }
    }
}

impl Default for MemorySessionDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionDriver for MemorySessionDriver {
    fn read(&self, session_id: &str) -> Result<HashMap<String, String>> {
        Ok(self.sessions.get(session_id).map(|d| d.clone()).unwrap_or_default())
    }

    fn write(&self, session_id: &str, data: &HashMap<String, String>) -> Result<()> {
        self.sessions.insert(session_id.to_string(), data.clone());
        Ok(())
    }

    fn destroy(&self, session_id: &str) -> Result<()> {
        self.sessions.remove(session_id);
        Ok(())
    }

    fn gc(&self, _max_lifetime: u64) -> Result<()> {
        Ok(())
    }
}

/// 文件 Session 驱动
///
/// 使用文件系统存储 Session 数据
/// 每个会话一个 JSON 文件
/// 支持垃圾回收（按修改时间清理过期文件）
pub struct FileSessionDriver {
    save_path: std::path::PathBuf,
}

impl FileSessionDriver {
    pub fn new(save_path: &std::path::Path) -> Self {
        if !save_path.exists() {
            let _ = std::fs::create_dir_all(save_path);
        }
        Self {
            save_path: save_path.to_path_buf(),
        }
    }

    fn session_file(&self, session_id: &str) -> std::path::PathBuf {
        self.save_path.join(format!("sess_{}", session_id))
    }
}

impl SessionDriver for FileSessionDriver {
    fn read(&self, session_id: &str) -> Result<HashMap<String, String>> {
        let path = self.session_file(session_id);
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str(&content).or_else(|_| Ok(HashMap::new()))
    }

    fn write(&self, session_id: &str, data: &HashMap<String, String>) -> Result<()> {
        let path = self.session_file(session_id);
        let content = serde_json::to_string(data)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    fn destroy(&self, session_id: &str) -> Result<()> {
        let path = self.session_file(session_id);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    fn gc(&self, max_lifetime: u64) -> Result<()> {
        if self.save_path.exists() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            for entry in std::fs::read_dir(&self.save_path)? {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            let modified_secs = modified
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            if now - modified_secs > max_lifetime {
                                let _ = std::fs::remove_file(entry.path());
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// Redis Session 驱动
///
/// 使用 Redis 存储 Session 数据
/// 支持多进程间共享
/// 支持自动过期（通过 Redis TTL）
/// 对应 ThinkPHP 8.0 的 Redis Session 驱动
pub struct RedisSessionDriver {
    /// Redis 连接 URL
    url: String,
    /// 连接池（惰性创建）
    pool: OnceCell<deadpool_redis::Pool>,
    /// 键前缀
    prefix: String,
    /// Session 过期时间（秒）
    ttl: u64,
}

impl RedisSessionDriver {
    /// 创建新的 Redis Session 驱动
    ///
    /// # 参数
    /// - `url`: Redis 连接 URL
    /// - `prefix`: 键前缀
    /// - `ttl`: Session 过期时间（秒），0 表示永不过期
    pub fn new(url: &str, prefix: &str, ttl: u64) -> Self {
        Self {
            url: url.to_string(),
            pool: OnceCell::new(),
            prefix: prefix.to_string(),
            ttl,
        }
    }

    /// 获取带前缀的完整键名
    fn full_key(&self, session_id: &str) -> String {
        format!("{}{}", self.prefix, session_id)
    }

    /// 获取 Redis 连接池
    fn get_pool(&self) -> Result<&deadpool_redis::Pool> {
        self.pool.get_or_try_init(|| {
            let cfg = deadpool_redis::Config::from_url(&self.url);
            cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
                .with_context(|| format!("无法创建 Redis 连接池: {}", self.url))
        })
    }

    /// 获取 Redis 连接
    fn get_connection(&self) -> Result<deadpool_redis::Connection> {
        let pool = self.get_pool()?;
        let rt = tokio::runtime::Runtime::new()
            .with_context(|| "无法创建 tokio runtime")?;
        rt.block_on(pool.get())
            .map_err(|e| anyhow::anyhow!("获取 Redis 连接失败: {}", e))
    }
}

impl SessionDriver for RedisSessionDriver {
    fn read(&self, session_id: &str) -> Result<HashMap<String, String>> {
        let full_key = self.full_key(session_id);
        let rt = tokio::runtime::Runtime::new()
            .with_context(|| "无法创建 tokio runtime")?;
        rt.block_on(async {
            let mut conn = self.get_connection()?;
            let data: Option<String> = redis::cmd("GET")
                .arg(&full_key)
                .query_async(&mut *conn)
                .await
                .ok()
                .flatten();
            match data {
                Some(json) => serde_json::from_str(&json).or_else(|_| Ok(HashMap::new())),
                None => Ok(HashMap::new()),
            }
        })
    }

    fn write(&self, session_id: &str, data: &HashMap<String, String>) -> Result<()> {
        let full_key = self.full_key(session_id);
        let json = serde_json::to_string(data)?;
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let mut conn = self.get_connection()?;
            if self.ttl > 0 {
                redis::cmd("SETEX")
                    .arg(&full_key)
                    .arg(self.ttl)
                    .arg(&json)
                    .query_async::<()>(&mut *conn)
                    .await
                    .with_context(|| format!("Redis SETEX 失败: {}", full_key))?;
            } else {
                redis::cmd("SET")
                    .arg(&full_key)
                    .arg(&json)
                    .query_async::<()>(&mut *conn)
                    .await
                    .with_context(|| format!("Redis SET 失败: {}", full_key))?;
            }
            Ok(())
        })
    }

    fn destroy(&self, session_id: &str) -> Result<()> {
        let full_key = self.full_key(session_id);
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let mut conn = self.get_connection()?;
            redis::cmd("DEL")
                .arg(&full_key)
                .query_async::<()>(&mut *conn)
                .await
                .with_context(|| format!("Redis DEL 失败: {}", full_key))?;
            Ok(())
        })
    }

    fn gc(&self, _max_lifetime: u64) -> Result<()> {
        Ok(())
    }
}
