//! Session 门面

use anyhow::Result;

use super::manager;

/// Session 门面
pub struct Session;

impl Session {
    /// 启动 Session
    pub fn start(session_id: &str) -> Result<()> {
        manager::with_session(|s| s.start(session_id))
            .ok_or_else(|| anyhow::anyhow!("Session 未初始化"))?
    }

    /// 获取 Session 值
    pub fn get(key: &str) -> Option<String> {
        manager::with_session(|s| s.get(key)).flatten()
    }

    /// 设置 Session 值
    pub fn set(key: &str, value: &str) -> Result<()> {
        manager::with_session(|s| {
            s.set(key, value);
            let _ = s.save();
        })
        .ok_or_else(|| anyhow::anyhow!("Session 未初始化"))
    }

    /// 删除 Session 值
    pub fn delete(key: &str) -> Result<()> {
        manager::with_session(|s| {
            s.remove(key);
            let _ = s.save();
        })
        .ok_or_else(|| anyhow::anyhow!("Session 未初始化"))
    }

    /// 检查 Session 值是否存在
    pub fn has(key: &str) -> bool {
        manager::with_session(|s| s.has(key)).unwrap_or(false)
    }

    /// 清空 Session
    pub fn clear() -> Result<()> {
        manager::with_session(|s| {
            s.clear();
            let _ = s.save();
        })
        .ok_or_else(|| anyhow::anyhow!("Session 未初始化"))
    }

    /// 销毁 Session
    pub fn destroy() -> Result<()> {
        manager::with_session(|s| s.destroy())
            .ok_or_else(|| anyhow::anyhow!("Session 未初始化"))?
    }
}
