//! 调试追踪模块
//!
//! 实现 ThinkPHP 8.0 风格的调试追踪面板
//! 在调试模式下，页面底部显示请求信息、SQL 查询、内存使用等
//! 对应 ThinkPHP 8.0 的 Trace 调试功能

use std::collections::HashMap;
use std::time::Instant;

use parking_lot::RwLock;

/// SQL 查询日志条目
#[derive(Debug, Clone)]
pub struct SqlLogEntry {
    /// SQL 语句
    pub sql: String,
    /// 绑定参数
    pub params: Vec<String>,
    /// 执行耗时（毫秒）
    pub elapsed_ms: u64,
    /// 查询时间戳
    pub timestamp: u64,
}

/// 调试追踪数据
#[derive(Debug, Clone)]
pub struct TraceData {
    /// 请求开始时间
    pub start_time: Instant,
    /// SQL 查询日志
    pub sql_logs: Vec<SqlLogEntry>,
    /// 文件加载列表
    pub loaded_files: Vec<String>,
    /// 自定义日志消息
    pub messages: Vec<(String, String)>,
    /// 内存使用峰值（字节）
    pub peak_memory: u64,
    /// 请求信息
    pub request_info: HashMap<String, String>,
}

impl TraceData {
    /// 创建新的追踪数据
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            sql_logs: Vec::new(),
            loaded_files: Vec::new(),
            messages: Vec::new(),
            peak_memory: 0,
            request_info: HashMap::new(),
        }
    }

    /// 记录 SQL 查询
    pub fn log_sql(&mut self, sql: &str, params: &[String], elapsed_ms: u64) {
        self.sql_logs.push(SqlLogEntry {
            sql: sql.to_string(),
            params: params.to_vec(),
            elapsed_ms,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        });
    }

    /// 记录文件加载
    pub fn log_file(&mut self, file_path: &str) {
        if !self.loaded_files.contains(&file_path.to_string()) {
            self.loaded_files.push(file_path.to_string());
        }
    }

    /// 记录自定义消息
    pub fn log_message(&mut self, level: &str, message: &str) {
        self.messages.push((level.to_string(), message.to_string()));
    }

    /// 获取总耗时（毫秒）
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    /// 获取 SQL 查询总数
    pub fn sql_count(&self) -> usize {
        self.sql_logs.len()
    }

    /// 获取 SQL 总耗时
    pub fn sql_total_ms(&self) -> u64 {
        self.sql_logs.iter().map(|e| e.elapsed_ms).sum()
    }

    /// 获取文件加载数量
    pub fn file_count(&self) -> usize {
        self.loaded_files.len()
    }

    /// 生成 HTML 调试面板
    pub fn render_html(&self) -> String {
        let elapsed = self.elapsed_ms();
        let sql_count = self.sql_count();
        let sql_time = self.sql_total_ms();
        let file_count = self.file_count();
        let memory_mb = self.peak_memory as f64 / (1024.0 * 1024.0);

        let sql_rows: String = self.sql_logs.iter().enumerate().map(|(i, entry)| {
            format!(
                r#"<tr><td>{}</td><td><code>{}</code></td><td>{}ms</td></tr>"#,
                i + 1,
                html_escape(&entry.sql),
                entry.elapsed_ms
            )
        }).collect();

        let msg_rows: String = self.messages.iter().map(|(level, msg)| {
            let color = match level.as_str() {
                "error" => "#e74c3c",
                "warning" => "#f39c12",
                "info" => "#3498db",
                _ => "#95a5a6",
            };
            format!(
                r#"<div style="padding:2px 8px;border-left:3px solid {};">[{}] {}</div>"#,
                color, level, html_escape(msg)
            )
        }).collect();

        format!(r#"
<div id="oyta-trace" style="position:fixed;bottom:0;left:0;right:0;background:#1e1e2e;color:#cdd6f4;font-family:monospace;font-size:12px;z-index:99999;max-height:300px;overflow-y:auto;border-top:2px solid #89b4fa;">
  <div style="display:flex;padding:6px 12px;background:#313244;cursor:pointer;" onclick="document.getElementById('oyta-trace-detail').style.display=document.getElementById('oyta-trace-detail').style.display==='none'?'block':'none'">
    <span style="color:#89b4fa;font-weight:bold;">OYTAPHP Trace</span>
    <span style="margin-left:16px;">耗时: <b style="color:#a6e3a1;">{}ms</b></span>
    <span style="margin-left:16px;">SQL: <b style="color:#f9e2af;">{}</b> ({}ms)</span>
    <span style="margin-left:16px;">文件: <b style="color:#cba6f7;">{}</b></span>
    <span style="margin-left:16px;">内存: <b style="color:#fab387;">{:.2}MB</b></span>
  </div>
  <div id="oyta-trace-detail" style="display:none;padding:8px 12px;">
    <div style="margin-bottom:8px;">
      <b style="color:#89b4fa;">SQL 查询 ({}):</b>
      <table style="width:100%;border-collapse:collapse;margin-top:4px;">
        <tr style="background:#45475a;"><th style="padding:4px 8px;text-align:left;">#</th><th style="padding:4px 8px;text-align:left;">SQL</th><th style="padding:4px 8px;text-align:left;">耗时</th></tr>
        {}
      </table>
    </div>
    <div style="margin-bottom:8px;">
      <b style="color:#89b4fa;">日志消息:</b>
      <div style="margin-top:4px;">{}</div>
    </div>
  </div>
</div>"#,
            elapsed,
            sql_count, sql_time,
            file_count,
            memory_mb,
            sql_count,
            sql_rows,
            msg_rows,
        )
    }
}

/// HTML 转义
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 全局追踪数据
static TRACE_DATA: once_cell::sync::Lazy<RwLock<Option<TraceData>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(None));

/// 初始化追踪
pub fn init_trace() {
    let mut guard = TRACE_DATA.write();
    *guard = Some(TraceData::new());
}

/// 记录 SQL 查询
pub fn trace_sql(sql: &str, params: &[String], elapsed_ms: u64) {
    let mut guard = TRACE_DATA.write();
    if let Some(data) = guard.as_mut() {
        data.log_sql(sql, params, elapsed_ms);
    }
}

/// 记录文件加载
pub fn trace_file(file_path: &str) {
    let mut guard = TRACE_DATA.write();
    if let Some(data) = guard.as_mut() {
        data.log_file(file_path);
    }
}

/// 记录自定义消息
pub fn trace_message(level: &str, message: &str) {
    let mut guard = TRACE_DATA.write();
    if let Some(data) = guard.as_mut() {
        data.log_message(level, message);
    }
}

/// 获取追踪数据的 HTML 渲染
pub fn get_trace_html() -> Option<String> {
    let guard = TRACE_DATA.read();
    guard.as_ref().map(|data| data.render_html())
}

/// 清除追踪数据
pub fn clear_trace() {
    let mut guard = TRACE_DATA.write();
    *guard = None;
}
