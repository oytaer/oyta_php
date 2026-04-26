//! 文件监听器
//!
//! 使用 notify 库监听项目目录变化
//! 当检测到 PHP 文件修改时，发送通知

use anyhow::Result;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

/// 文件变化事件
#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    /// 变化的文件路径
    pub path: PathBuf,
    /// 变化类型
    pub kind: ChangeKind,
}

/// 变化类型
#[derive(Debug, Clone, Copy)]
pub enum ChangeKind {
    /// 文件创建
    Created,
    /// 文件修改
    Modified,
    /// 文件删除
    Deleted,
    /// 其他
    Other,
}

/// 文件监听器
/// 监听项目目录下的文件变化
pub struct FileWatcher {
    /// 监听器
    watcher: RecommendedWatcher,
    /// 事件接收通道
    rx: mpsc::Receiver<FileChangeEvent>,
}

impl FileWatcher {
    /// 创建新的文件监听器
    pub fn new(project_root: &Path) -> Result<Self> {
        let (tx, rx) = mpsc::channel();

        // 创建事件发送闭包
        let event_tx = tx.clone();
        let watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                // 只关注 PHP 文件的变化
                for path in &event.paths {
                    if path.extension().and_then(|e| e.to_str()) == Some("php") {
                        let kind = match event.kind {
                            EventKind::Create(_) => ChangeKind::Created,
                            EventKind::Modify(_) => ChangeKind::Modified,
                            EventKind::Remove(_) => ChangeKind::Deleted,
                            _ => ChangeKind::Other,
                        };
                        let _ = event_tx.send(FileChangeEvent {
                            path: path.clone(),
                            kind,
                        });
                    }
                }
            }
        })?;

        let mut fw = Self { watcher, rx };

        // 监听项目目录
        fw.watcher.watch(project_root, RecursiveMode::Recursive)?;

        Ok(fw)
    }

    /// 等待文件变化事件（带超时）
    pub fn wait_for_change(&self, timeout: Duration) -> Option<FileChangeEvent> {
        self.rx.recv_timeout(timeout).ok()
    }

    /// 获取所有待处理的变化事件
    pub fn drain_events(&self) -> Vec<FileChangeEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.rx.try_recv() {
            events.push(event);
        }
        events
    }
}
