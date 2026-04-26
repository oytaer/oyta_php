//! Session 管理器
//!
//! 管理 Session 的生命周期
//! 支持：文件、内存、Redis、Cache 驱动
//! 支持：flash 一次性消息

use anyhow::Result;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use super::driver::{FileSessionDriver, MemorySessionDriver, RedisSessionDriver, SessionDriver};

/// Session 管理器
///
/// 管理 Session 的创建、读取、写入和销毁
/// 支持 flash 数据（一次性消息）
pub struct SessionManager {
    /// Session 驱动
    driver: Arc<dyn SessionDriver>,
    /// 当前 Session ID
    session_id: Option<String>,
    /// Session 数据
    data: HashMap<String, String>,
    /// Flash 数据（一次性消息）
    /// 在下一次请求中可用，读取后自动删除
    flash: HashMap<String, String>,
    /// 待写入的 flash 数据（当前请求设置，下次请求可用）
    flash_new: HashMap<String, String>,
    /// Session 是否已启动
    started: bool,
}

impl SessionManager {
    /// 创建新的 Session 管理器
    ///
    /// # 参数
    /// - `driver_type`: 驱动类型（"memory", "file", "redis", "cache"）
    /// - `save_path`: 存储路径
    pub fn new(driver_type: &str, save_path: &std::path::Path) -> Self {
        let driver: Arc<dyn SessionDriver> = match driver_type {
            "memory" => Arc::new(MemorySessionDriver::new()),
            "redis" => {
                let url = std::env::var("SESSION_REDIS_URL")
                    .unwrap_or_else(|_| "redis://127.0.0.1:6379/1".to_string());
                let ttl = std::env::var("SESSION_TTL")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(7200);
                Arc::new(RedisSessionDriver::new(&url, "sess:", ttl))
            }
            _ => Arc::new(FileSessionDriver::new(save_path)),
        };
        Self {
            driver,
            session_id: None,
            data: HashMap::new(),
            flash: HashMap::new(),
            flash_new: HashMap::new(),
            started: false,
        }
    }

    /// 使用自定义驱动创建 Session 管理器
    pub fn with_driver(driver: Arc<dyn SessionDriver>) -> Self {
        Self {
            driver,
            session_id: None,
            data: HashMap::new(),
            flash: HashMap::new(),
            flash_new: HashMap::new(),
            started: false,
        }
    }

    /// 启动 Session
    pub fn start(&mut self, session_id: &str) -> Result<()> {
        self.session_id = Some(session_id.to_string());
        self.data = self.driver.read(session_id)?;
        self.started = true;

        // 从 Session 数据中提取 flash 数据
        // flash 数据存储在 __flash__ 键下
        if let Some(flash_json) = self.data.remove("__flash__") {
            if let Ok(flash_data) = serde_json::from_str::<HashMap<String, String>>(&flash_json) {
                self.flash = flash_data;
            }
        }

        Ok(())
    }

    /// 保存 Session
    pub fn save(&self) -> Result<()> {
        if let Some(sid) = &self.session_id {
            // 将新的 flash 数据合并到 Session 数据中
            let mut save_data = self.data.clone();
            if !self.flash_new.is_empty() {
                let flash_json = serde_json::to_string(&self.flash_new)?;
                save_data.insert("__flash__".to_string(), flash_json);
            }

            self.driver.write(sid, &save_data)?;
        }
        Ok(())
    }

    /// 销毁 Session
    pub fn destroy(&mut self) -> Result<()> {
        if let Some(sid) = &self.session_id {
            self.driver.destroy(sid)?;
        }
        self.data.clear();
        self.flash.clear();
        self.flash_new.clear();
        self.session_id = None;
        self.started = false;
        Ok(())
    }

    /// 获取 Session 值
    pub fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    /// 设置 Session 值
    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    /// 删除 Session 值
    pub fn remove(&mut self, key: &str) {
        self.data.remove(key);
    }

    /// 检查 Session 值是否存在
    pub fn has(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// 获取所有 Session 数据
    pub fn all(&self) -> &HashMap<String, String> {
        &self.data
    }

    /// 清空 Session 数据
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// 获取 Session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// 重新生成 Session ID
    pub fn regenerate(&mut self) -> Result<String> {
        let old_data = self.data.clone();
        if let Some(old_sid) = &self.session_id {
            self.driver.destroy(old_sid)?;
        }
        let new_sid = generate_session_id();
        self.session_id = Some(new_sid.clone());
        self.data = old_data;
        self.driver.write(&new_sid, &self.data)?;
        Ok(new_sid)
    }

    // ==================== Flash 功能 ====================

    /// 设置 flash 数据（一次性消息）
    /// 在当前请求中设置，在下一次请求中可读取
    /// 读取后自动删除
    ///
    /// # 参数
    /// - `key`: 键名
    /// - `value`: 值
    pub fn flash(&mut self, key: &str, value: &str) {
        self.flash_new.insert(key.to_string(), value.to_string());
    }

    /// 获取 flash 数据
    /// 读取后自动删除（一次性）
    ///
    /// # 参数
    /// - `key`: 键名
    /// - `default`: 默认值
    pub fn get_flash(&mut self, key: &str, default: Option<&str>) -> Option<String> {
        match self.flash.remove(key) {
            Some(value) => Some(value),
            None => default.map(|s| s.to_string()),
        }
    }

    /// 检查 flash 数据是否存在
    pub fn has_flash(&self, key: &str) -> bool {
        self.flash.contains_key(key)
    }

    /// 保留 flash 数据（使其在下一次请求中仍然可用）
    /// 通常在当前请求中读取了 flash 数据后又想保留时使用
    ///
    /// # 参数
    /// - `key`: 要保留的键名
    pub fn keep_flash(&mut self, key: &str) {
        if let Some(value) = self.flash.get(key).cloned() {
            self.flash_new.insert(key.to_string(), value);
        }
    }

    /// 保留所有 flash 数据
    pub fn keep_all_flash(&mut self) {
        for (key, value) in self.flash.drain() {
            self.flash_new.insert(key, value);
        }
    }

    /// 清除所有新的 flash 数据
    pub fn clear_flash(&mut self) {
        self.flash_new.clear();
    }

    /// 获取所有当前可用的 flash 数据
    pub fn all_flash(&self) -> &HashMap<String, String> {
        &self.flash
    }

    /// 重新刷新 flash 数据
    /// 将当前请求中的 flash 数据重新放入下次请求
    pub fn reflash(&mut self) {
        for (key, value) in self.flash.drain() {
            self.flash_new.insert(key, value);
        }
    }

    /// 是否已启动
    pub fn is_started(&self) -> bool {
        self.started
    }
}

/// 生成随机的 Session ID
fn generate_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let random: u64 = rand::random();
    format!("{:x}{:x}", now, random)
}

/// 全局 Session 管理器
static SESSION_MANAGER: Lazy<RwLock<Option<SessionManager>>> = Lazy::new(|| {
    RwLock::new(None)
});

/// 初始化 Session
pub fn init_session(driver_type: &str, save_path: &std::path::Path) {
    let mut manager = SESSION_MANAGER.write();
    *manager = Some(SessionManager::new(driver_type, save_path));
}

/// 获取 Session 管理器的可变引用（通过 RwLock）
pub fn with_session<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut SessionManager) -> R,
{
    let mut guard = SESSION_MANAGER.write();
    guard.as_mut().map(f)
}
