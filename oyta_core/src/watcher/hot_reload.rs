//! 热重载管理器
//!
//! 当文件变化时，自动重新解析并更新符号表和配置

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use super::watcher::{ChangeKind, FileChangeEvent, FileWatcher};
use crate::parser::php_parser::PhpParser;
use crate::symbol_table::registry::SymbolRegistry;

/// 热重载管理器
pub struct HotReloader {
    /// 文件监听器
    watcher: FileWatcher,
    /// PHP 解析器
    parser: PhpParser,
    /// 符号表注册表
    registry: Arc<SymbolRegistry>,
    /// 项目根目录
    project_root: std::path::PathBuf,
}

impl HotReloader {
    /// 创建新的热重载管理器
    pub fn new(
        project_root: &Path,
        registry: Arc<SymbolRegistry>,
    ) -> Result<Self> {
        let watcher = FileWatcher::new(project_root)?;
        Ok(Self {
            watcher,
            parser: PhpParser::new(),
            registry,
            project_root: project_root.to_path_buf(),
        })
    }

    /// 启动热重载循环
    /// 在后台线程中监听文件变化
    pub async fn run(&mut self) {
        tracing::info!("热重载监听已启动: {}", self.project_root.display());

        loop {
            // 等待文件变化（1秒超时）
            if let Some(event) = self.watcher.wait_for_change(Duration::from_secs(1)) {
                self.handle_change(event).await;
            }

            // 检查是否还有待处理的事件
            let events = self.watcher.drain_events();
            for event in events {
                self.handle_change(event).await;
            }
        }
    }

    /// 处理文件变化事件
    async fn handle_change(&mut self, event: FileChangeEvent) {
        let path_str = event.path.display().to_string();

        match event.kind {
            ChangeKind::Created | ChangeKind::Modified => {
                tracing::info!("文件变化: {} ({:?})", path_str, event.kind);

                // 重新解析文件
                match self.parser.parse_file(&event.path) {
                    Ok(result) => {
                        // 更新符号表
                        self.registry.unregister_file(&result.file_path);
                        self.registry.register_file(&result);

                        tracing::info!(
                            "热重载完成: {} ({} 个类, {} 个函数)",
                            path_str,
                            result.classes.len(),
                            result.functions.len()
                        );
                    }
                    Err(e) => {
                        tracing::warn!("热重载解析失败: {} - {}", path_str, e);
                    }
                }
            }
            ChangeKind::Deleted => {
                tracing::info!("文件删除: {}", path_str);
                // 从符号表中移除
                self.registry.unregister_file(&path_str);
            }
            ChangeKind::Other => {}
        }
    }
}
