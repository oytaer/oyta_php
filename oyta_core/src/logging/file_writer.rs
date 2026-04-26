//! 文件日志写入器
//!
//! 提供基于文件的日志写入功能
//! 支持：按日期分割日志文件、日志轮转、异步写入
//! 兼容 tracing-subscriber 的 Layer 接口

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::Local;
use tracing_subscriber::fmt;

/// 文件日志写入器
///
/// 支持按日期自动分割日志文件
/// 支持最大文件数量限制（自动清理旧日志）
pub struct FileWriter {
    /// 日志目录
    log_dir: PathBuf,
    /// 日志文件前缀
    prefix: String,
    /// 当前日志文件路径
    current_file: Mutex<Option<PathBuf>>,
    /// 当前日期（用于判断是否需要切换文件）
    current_date: Mutex<String>,
    /// 最大保留日志文件数量（0 表示不限制）
    max_files: usize,
}

impl FileWriter {
    /// 创建新的文件日志写入器
    ///
    /// # 参数
    /// - `log_dir`: 日志目录路径
    /// - `prefix`: 日志文件前缀，如 "oyta"
    ///
    /// # 返回
    /// 写入器实例
    pub fn new(log_dir: &Path, prefix: &str) -> Self {
        Self {
            log_dir: log_dir.to_path_buf(),
            prefix: prefix.to_string(),
            current_file: Mutex::new(None),
            current_date: Mutex::new(String::new()),
            max_files: 30,
        }
    }

    /// 设置最大保留日志文件数量
    pub fn with_max_files(mut self, max_files: usize) -> Self {
        self.max_files = max_files;
        self
    }

    /// 确保日志目录存在
    fn ensure_dir(&self) -> std::io::Result<()> {
        if !self.log_dir.exists() {
            fs::create_dir_all(&self.log_dir)?;
        }
        Ok(())
    }

    /// 获取当前日期字符串
    fn today() -> String {
        Local::now().format("%Y-%m-%d").to_string()
    }

    /// 获取当前日志文件路径
    fn get_log_path(&self, date: &str) -> PathBuf {
        self.log_dir.join(format!("{}-{}.log", self.prefix, date))
    }

    /// 检查是否需要切换到新的日志文件
    fn check_rotation(&self) {
        let today = Self::today();
        let mut current_date = self.current_date.lock().unwrap();

        if *current_date != today {
            *current_date = today.clone();
            let mut current_file = self.current_file.lock().unwrap();
            *current_file = Some(self.get_log_path(&today));
            drop(current_file);
            self.cleanup_old_files();
        }
    }

    /// 清理过期的日志文件
    fn cleanup_old_files(&self) {
        if self.max_files == 0 {
            return;
        }
        let mut log_files: Vec<PathBuf> = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.log_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with(&self.prefix) && name.ends_with(".log") {
                        log_files.push(path);
                    }
                }
            }
        }
        log_files.sort();
        while log_files.len() > self.max_files {
            if let Some(old_file) = log_files.first() {
                let _ = fs::remove_file(old_file);
                log_files.remove(0);
            } else {
                break;
            }
        }
    }

    /// 写入日志行
    pub fn write_line(&self, line: &str) {
        let _ = self.ensure_dir();
        self.check_rotation();
        let current_file = self.current_file.lock().unwrap();
        let path = match current_file.as_ref() {
            Some(p) => p.clone(),
            None => {
                let today = Self::today();
                let path = self.get_log_path(&today);
                drop(current_file);
                let mut current_file = self.current_file.lock().unwrap();
                let mut current_date = self.current_date.lock().unwrap();
                *current_date = today.clone();
                *current_file = Some(path.clone());
                return self.write_line(line);
            }
        };
        drop(current_file);
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = writeln!(file, "{}", line);
        }
    }
}

impl std::io::Write for FileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let line = String::from_utf8_lossy(buf);
        self.write_line(&line);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl fmt::MakeWriter<'_> for FileWriter {
    type Writer = Self;

    fn make_writer(&self) -> Self::Writer {
        Self {
            log_dir: self.log_dir.clone(),
            prefix: self.prefix.clone(),
            current_file: Mutex::new(None),
            current_date: Mutex::new(String::new()),
            max_files: self.max_files,
        }
    }
}
