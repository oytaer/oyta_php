//! 监控面板服务模块
//!
//! 实现监控面板的 HTTP 服务和可视化界面
//! 提供实时数据推送和静态页面服务

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::collector::{MetricCollector, MetricSnapshot, CollectorConfig};
use super::metrics::MetricValue;

/// 监控面板配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// 是否启用面板
    pub enabled: bool,
    /// 面板路由路径
    pub route: String,
    /// 访问权限列表
    pub auth: Vec<String>,
    /// 刷新间隔（毫秒）
    pub refresh_ms: u64,
    /// 是否启用 WebSocket 实时推送
    pub enable_websocket: bool,
    /// 是否记录请求体
    pub log_request_body: bool,
    /// 是否记录响应体
    pub log_response_body: bool,
    /// 最大历史记录数
    pub max_history: usize,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            route: "/oyta/monitor".to_string(),
            auth: vec!["admin".to_string()],
            refresh_ms: 1000,
            enable_websocket: true,
            log_request_body: false,
            log_response_body: false,
            max_history: 1000,
        }
    }
}

/// 监控面板服务
/// 提供 HTTP 接口和可视化界面
#[derive(Debug)]
pub struct MonitorDashboard {
    /// 指标收集器
    collector: Arc<MetricCollector>,
    /// 面板配置
    config: RwLock<DashboardConfig>,
    /// 历史数据存储
    history: RwLock<Vec<MetricSnapshot>>,
    /// 告警规则
    alert_rules: RwLock<Vec<AlertRule>>,
    /// 告警历史
    alert_history: RwLock<Vec<AlertEvent>>,
}

/// 告警规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// 规则 ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 指标名称
    pub metric: String,
    /// 条件类型
    pub condition: AlertCondition,
    /// 阈值
    pub threshold: f64,
    /// 持续时间（秒）
    pub duration_seconds: u64,
    /// 严重级别
    pub severity: AlertSeverity,
    /// 是否启用
    pub enabled: bool,
    /// 通知渠道
    pub notify_channels: Vec<String>,
}

/// 告警条件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertCondition {
    /// 大于阈值
    GreaterThan,
    /// 小于阈值
    LessThan,
    /// 等于阈值
    Equals,
    /// 不等于阈值
    NotEquals,
    /// 大于等于阈值
    GreaterThanOrEqual,
    /// 小于等于阈值
    LessThanOrEqual,
}

/// 告警严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 严重
    Critical,
}

/// 告警事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    /// 事件 ID
    pub id: String,
    /// 规则 ID
    pub rule_id: String,
    /// 规则名称
    pub rule_name: String,
    /// 触发时间戳
    pub timestamp: u64,
    /// 指标名称
    pub metric: String,
    /// 当前值
    pub value: f64,
    /// 阈值
    pub threshold: f64,
    /// 严重级别
    pub severity: AlertSeverity,
    /// 是否已恢复
    pub resolved: bool,
    /// 恢复时间戳
    pub resolved_at: Option<u64>,
}

impl MonitorDashboard {
    /// 创建新的监控面板
    pub fn new(collector: Arc<MetricCollector>) -> Self {
        Self {
            collector,
            config: RwLock::new(DashboardConfig::default()),
            history: RwLock::new(Vec::new()),
            alert_rules: RwLock::new(Vec::new()),
            alert_history: RwLock::new(Vec::new()),
        }
    }
    
    /// 使用配置创建监控面板
    pub fn with_config(collector: Arc<MetricCollector>, config: DashboardConfig) -> Self {
        Self {
            collector,
            config: RwLock::new(config),
            history: RwLock::new(Vec::new()),
            alert_rules: RwLock::new(Vec::new()),
            alert_history: RwLock::new(Vec::new()),
        }
    }
    
    /// 获取当前配置
    pub fn get_config(&self) -> DashboardConfig {
        self.config.read().unwrap().clone()
    }
    
    /// 更新配置
    pub fn update_config(&self, config: DashboardConfig) {
        if let Ok(mut cfg) = self.config.write() {
            *cfg = config;
        }
    }
    
    /// 获取当前指标快照
    pub fn get_metrics(&self) -> MetricSnapshot {
        self.collector.get_snapshot()
    }
    
    /// 收集并存储快照
    pub fn collect_snapshot(&self) {
        // 获取当前快照
        let snapshot = self.collector.get_snapshot();
        
        // 存储到历史记录
        if let Ok(mut history) = self.history.write() {
            // 添加新快照
            history.push(snapshot.clone());
            
            // 获取最大历史记录数
            let max_history = self.config.read().map(|c| c.max_history).unwrap_or(1000);
            
            // 超过限制时移除旧记录
            while history.len() > max_history {
                history.remove(0);
            }
        }
        
        // 检查告警规则
        self.check_alerts(&snapshot);
    }
    
    /// 获取历史数据
    pub fn get_history(&self, limit: usize) -> Vec<MetricSnapshot> {
        if let Ok(history) = self.history.read() {
            history.iter().rev().take(limit).cloned().collect()
        } else {
            Vec::new()
        }
    }
    
    /// 添加告警规则
    pub fn add_alert_rule(&self, rule: AlertRule) {
        if let Ok(mut rules) = self.alert_rules.write() {
            rules.push(rule);
        }
    }
    
    /// 移除告警规则
    pub fn remove_alert_rule(&self, rule_id: &str) {
        if let Ok(mut rules) = self.alert_rules.write() {
            rules.retain(|r| r.id != rule_id);
        }
    }
    
    /// 获取所有告警规则
    pub fn get_alert_rules(&self) -> Vec<AlertRule> {
        self.alert_rules.read().unwrap().clone()
    }
    
    /// 获取告警历史
    pub fn get_alert_history(&self, limit: usize) -> Vec<AlertEvent> {
        self.alert_history.read().unwrap().iter().rev().take(limit).cloned().collect()
    }
    
    /// 检查告警规则
    fn check_alerts(&self, snapshot: &MetricSnapshot) {
        // 获取告警规则
        let rules = self.alert_rules.read().unwrap().clone();
        
        // 遍历每个规则
        for rule in rules.iter().filter(|r| r.enabled) {
            // 获取指标值
            let value = self.get_metric_value(snapshot, &rule.metric);
            
            // 检查条件
            let triggered = match rule.condition {
                AlertCondition::GreaterThan => value > rule.threshold,
                AlertCondition::LessThan => value < rule.threshold,
                AlertCondition::Equals => (value - rule.threshold).abs() < f64::EPSILON,
                AlertCondition::NotEquals => (value - rule.threshold).abs() >= f64::EPSILON,
                AlertCondition::GreaterThanOrEqual => value >= rule.threshold,
                AlertCondition::LessThanOrEqual => value <= rule.threshold,
            };
            
            // 如果触发告警
            if triggered {
                // 创建告警事件
                let event = AlertEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    rule_id: rule.id.clone(),
                    rule_name: rule.name.clone(),
                    timestamp: snapshot.timestamp,
                    metric: rule.metric.clone(),
                    value,
                    threshold: rule.threshold,
                    severity: rule.severity,
                    resolved: false,
                    resolved_at: None,
                };
                
                // 存储告警事件
                if let Ok(mut history) = self.alert_history.write() {
                    history.push(event);
                }
                
                // 发送通知（这里只是记录，实际应该发送到通知渠道）
                tracing::warn!(
                    "告警触发: {} - {} = {} (阈值: {})",
                    rule.name,
                    rule.metric,
                    value,
                    rule.threshold
                );
            }
        }
    }
    
    /// 获取指标值
    fn get_metric_value(&self, snapshot: &MetricSnapshot, metric: &str) -> f64 {
        // 根据指标名称获取值
        match metric {
            // 请求指标
            "request.total" => snapshot.request.total_requests as f64,
            "request.success" => snapshot.request.successful_requests as f64,
            "request.failed" => snapshot.request.failed_requests as f64,
            "request.active" => snapshot.request.active_requests as f64,
            "request.qps" => snapshot.request.current_qps,
            "request.avg_response_time" => snapshot.request.avg_response_time_ms,
            "request.max_response_time" => snapshot.request.max_response_time_ms as f64,
            "request.error_rate" => snapshot.request.error_rate,
            
            // 内存指标
            "memory.used" => snapshot.memory.used_bytes as f64,
            "memory.peak" => snapshot.memory.peak_bytes as f64,
            "memory.total" => snapshot.memory.total_bytes as f64,
            "memory.usage_percent" => snapshot.memory.usage_percent,
            "memory.gc_count" => snapshot.memory.gc_count as f64,
            
            // 数据库指标
            "db.connections" => snapshot.database.total_connections as f64,
            "db.active_connections" => snapshot.database.active_connections as f64,
            "db.queries" => snapshot.database.total_queries as f64,
            "db.slow_queries" => snapshot.database.slow_queries as f64,
            "db.avg_query_time" => snapshot.database.avg_query_time_ms,
            
            // 缓存指标
            "cache.items" => snapshot.cache.total_items as f64,
            "cache.hits" => snapshot.cache.hits as f64,
            "cache.misses" => snapshot.cache.misses as f64,
            "cache.hit_rate" => snapshot.cache.hit_rate,
            "cache.memory" => snapshot.cache.memory_bytes as f64,
            "cache.evictions" => snapshot.cache.evictions as f64,
            
            // 队列指标
            "queue.pending" => snapshot.queue.pending_messages as f64,
            "queue.processed" => snapshot.queue.processed_messages as f64,
            "queue.failed" => snapshot.queue.failed_messages as f64,
            "queue.consumers" => snapshot.queue.active_consumers as f64,
            
            // WebSocket 指标
            "ws.connections" => snapshot.websocket.active_connections as f64,
            "ws.messages_received" => snapshot.websocket.messages_received as f64,
            "ws.messages_sent" => snapshot.websocket.messages_sent as f64,
            
            // 自定义指标
            _ => {
                if let Some(value) = snapshot.custom.get(metric) {
                    match value {
                        MetricValue::Int(v) => *v as f64,
                        MetricValue::Float(v) => *v,
                        _ => 0.0,
                    }
                } else {
                    0.0
                }
            }
        }
    }
    
    /// 生成 HTML 面板页面
    pub fn generate_html(&self) -> String {
        let config = self.get_config();
        
        format!(r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OYTAPHP 性能监控面板</title>
    <style>
        /* 全局样式 */
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: #e4e4e4;
            min-height: 100vh;
        }}
        
        /* 头部样式 */
        .header {{
            background: rgba(0, 0, 0, 0.3);
            padding: 20px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        }}
        
        .header h1 {{
            font-size: 24px;
            color: #00d4ff;
        }}
        
        .header .uptime {{
            font-size: 14px;
            color: #888;
            margin-top: 5px;
        }}
        
        /* 主容器 */
        .container {{
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
        }}
        
        /* 网格布局 */
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 20px;
        }}
        
        /* 卡片样式 */
        .card {{
            background: rgba(255, 255, 255, 0.05);
            border-radius: 12px;
            padding: 20px;
            border: 1px solid rgba(255, 255, 255, 0.1);
        }}
        
        .card h2 {{
            font-size: 16px;
            color: #00d4ff;
            margin-bottom: 15px;
            display: flex;
            align-items: center;
            gap: 10px;
        }}
        
        .card h2 .icon {{
            font-size: 20px;
        }}
        
        /* 指标项 */
        .metric {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 10px 0;
            border-bottom: 1px solid rgba(255, 255, 255, 0.05);
        }}
        
        .metric:last-child {{
            border-bottom: none;
        }}
        
        .metric .label {{
            color: #888;
            font-size: 14px;
        }}
        
        .metric .value {{
            font-size: 18px;
            font-weight: bold;
            color: #fff;
        }}
        
        .metric .value.success {{
            color: #00ff88;
        }}
        
        .metric .value.warning {{
            color: #ffaa00;
        }}
        
        .metric .value.error {{
            color: #ff4444;
        }}
        
        /* 进度条 */
        .progress-bar {{
            width: 100%;
            height: 8px;
            background: rgba(255, 255, 255, 0.1);
            border-radius: 4px;
            overflow: hidden;
            margin-top: 10px;
        }}
        
        .progress-bar .fill {{
            height: 100%;
            background: linear-gradient(90deg, #00d4ff, #00ff88);
            transition: width 0.3s ease;
        }}
        
        .progress-bar .fill.warning {{
            background: linear-gradient(90deg, #ffaa00, #ff6600);
        }}
        
        .progress-bar .fill.error {{
            background: linear-gradient(90deg, #ff4444, #cc0000);
        }}
        
        /* 状态指示器 */
        .status {{
            display: inline-flex;
            align-items: center;
            gap: 5px;
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 12px;
            font-weight: 500;
        }}
        
        .status.healthy {{
            background: rgba(0, 255, 136, 0.2);
            color: #00ff88;
        }}
        
        .status.warning {{
            background: rgba(255, 170, 0, 0.2);
            color: #ffaa00;
        }}
        
        .status.error {{
            background: rgba(255, 68, 68, 0.2);
            color: #ff4444;
        }}
        
        /* 告警列表 */
        .alert-list {{
            max-height: 300px;
            overflow-y: auto;
        }}
        
        .alert-item {{
            padding: 10px;
            margin-bottom: 10px;
            border-radius: 8px;
            background: rgba(255, 255, 255, 0.03);
            border-left: 3px solid;
        }}
        
        .alert-item.info {{
            border-color: #00d4ff;
        }}
        
        .alert-item.warning {{
            border-color: #ffaa00;
        }}
        
        .alert-item.error {{
            border-color: #ff4444;
        }}
        
        .alert-item.critical {{
            border-color: #ff0000;
            background: rgba(255, 0, 0, 0.1);
        }}
        
        .alert-item .time {{
            font-size: 12px;
            color: #888;
        }}
        
        /* 刷新控制 */
        .controls {{
            display: flex;
            align-items: center;
            gap: 15px;
            margin-bottom: 20px;
        }}
        
        .controls button {{
            padding: 10px 20px;
            border: none;
            border-radius: 8px;
            background: #00d4ff;
            color: #000;
            font-weight: bold;
            cursor: pointer;
            transition: all 0.3s ease;
        }}
        
        .controls button:hover {{
            background: #00a8cc;
            transform: translateY(-2px);
        }}
        
        .controls .refresh-interval {{
            display: flex;
            align-items: center;
            gap: 10px;
            color: #888;
        }}
        
        .controls select {{
            padding: 8px 12px;
            border: 1px solid rgba(255, 255, 255, 0.2);
            border-radius: 6px;
            background: rgba(0, 0, 0, 0.3);
            color: #fff;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🚀 OYTAPHP 性能监控面板</h1>
        <div class="uptime">运行时间: <span id="uptime">--</span></div>
    </div>
    
    <div class="container">
        <div class="controls">
            <button onclick="refreshMetrics()">刷新数据</button>
            <div class="refresh-interval">
                <label>自动刷新:</label>
                <select id="refreshInterval" onchange="updateRefreshInterval()">
                    <option value="1000">1秒</option>
                    <option value="5000">5秒</option>
                    <option value="10000">10秒</option>
                    <option value="30000">30秒</option>
                    <option value="0">关闭</option>
                </select>
            </div>
        </div>
        
        <div class="grid">
            <!-- 请求指标卡片 -->
            <div class="card">
                <h2><span class="icon">📊</span> 请求指标</h2>
                <div class="metric">
                    <span class="label">当前 QPS</span>
                    <span class="value" id="qps">--</span>
                </div>
                <div class="metric">
                    <span class="label">总请求数</span>
                    <span class="value" id="totalRequests">--</span>
                </div>
                <div class="metric">
                    <span class="label">活跃请求</span>
                    <span class="value" id="activeRequests">--</span>
                </div>
                <div class="metric">
                    <span class="label">平均响应时间</span>
                    <span class="value" id="avgResponseTime">--</span>
                </div>
                <div class="metric">
                    <span class="label">错误率</span>
                    <span class="value" id="errorRate">--</span>
                </div>
                <div class="progress-bar">
                    <div class="fill" id="errorRateBar" style="width: 0%"></div>
                </div>
            </div>
            
            <!-- 内存指标卡片 -->
            <div class="card">
                <h2><span class="icon">💾</span> 内存指标</h2>
                <div class="metric">
                    <span class="label">内存使用</span>
                    <span class="value" id="memoryUsed">--</span>
                </div>
                <div class="metric">
                    <span class="label">内存峰值</span>
                    <span class="value" id="memoryPeak">--</span>
                </div>
                <div class="metric">
                    <span class="label">使用率</span>
                    <span class="value" id="memoryUsage">--</span>
                </div>
                <div class="progress-bar">
                    <div class="fill" id="memoryBar" style="width: 0%"></div>
                </div>
                <div class="metric">
                    <span class="label">GC 次数</span>
                    <span class="value" id="gcCount">--</span>
                </div>
            </div>
            
            <!-- 数据库指标卡片 -->
            <div class="card">
                <h2><span class="icon">🗄️</span> 数据库指标</h2>
                <div class="metric">
                    <span class="label">活跃连接</span>
                    <span class="value" id="dbConnections">--</span>
                </div>
                <div class="metric">
                    <span class="label">总查询数</span>
                    <span class="value" id="dbQueries">--</span>
                </div>
                <div class="metric">
                    <span class="label">慢查询</span>
                    <span class="value" id="dbSlowQueries">--</span>
                </div>
                <div class="metric">
                    <span class="label">平均查询时间</span>
                    <span class="value" id="dbAvgQueryTime">--</span>
                </div>
            </div>
            
            <!-- 缓存指标卡片 -->
            <div class="card">
                <h2><span class="icon">⚡</span> 缓存指标</h2>
                <div class="metric">
                    <span class="label">缓存项数</span>
                    <span class="value" id="cacheItems">--</span>
                </div>
                <div class="metric">
                    <span class="label">命中率</span>
                    <span class="value success" id="cacheHitRate">--</span>
                </div>
                <div class="progress-bar">
                    <div class="fill" id="cacheHitBar" style="width: 0%"></div>
                </div>
                <div class="metric">
                    <span class="label">内存使用</span>
                    <span class="value" id="cacheMemory">--</span>
                </div>
                <div class="metric">
                    <span class="label">淘汰数</span>
                    <span class="value" id="cacheEvictions">--</span>
                </div>
            </div>
            
            <!-- 队列指标卡片 -->
            <div class="card">
                <h2><span class="icon">📬</span> 队列指标</h2>
                <div class="metric">
                    <span class="label">待处理消息</span>
                    <span class="value" id="queuePending">--</span>
                </div>
                <div class="metric">
                    <span class="label">已处理消息</span>
                    <span class="value" id="queueProcessed">--</span>
                </div>
                <div class="metric">
                    <span class="label">失败消息</span>
                    <span class="value" id="queueFailed">--</span>
                </div>
                <div class="metric">
                    <span class="label">活跃消费者</span>
                    <span class="value" id="queueConsumers">--</span>
                </div>
            </div>
            
            <!-- WebSocket 指标卡片 -->
            <div class="card">
                <h2><span class="icon">🔌</span> WebSocket 指标</h2>
                <div class="metric">
                    <span class="label">活跃连接</span>
                    <span class="value" id="wsConnections">--</span>
                </div>
                <div class="metric">
                    <span class="label">接收消息</span>
                    <span class="value" id="wsMessagesReceived">--</span>
                </div>
                <div class="metric">
                    <span class="label">发送消息</span>
                    <span class="value" id="wsMessagesSent">--</span>
                </div>
                <div class="metric">
                    <span class="label">流量</span>
                    <span class="value" id="wsTraffic">--</span>
                </div>
            </div>
        </div>
        
        <!-- 告警卡片 -->
        <div class="card">
            <h2><span class="icon">⚠️</span> 告警历史</h2>
            <div class="alert-list" id="alertList">
                <p style="color: #888; text-align: center; padding: 20px;">暂无告警</p>
            </div>
        </div>
    </div>
    
    <script>
        // 刷新间隔定时器
        let refreshTimer = null;
        
        // 格式化字节数
        function formatBytes(bytes) {{
            if (bytes === 0) return '0 B';
            const k = 1024;
            const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
            const i = Math.floor(Math.log(bytes) / Math.log(k));
            return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
        }}
        
        // 格式化时间
        function formatUptime(seconds) {{
            const days = Math.floor(seconds / 86400);
            const hours = Math.floor((seconds % 86400) / 3600);
            const minutes = Math.floor((seconds % 3600) / 60);
            const secs = seconds % 60;
            
            let result = '';
            if (days > 0) result += days + '天 ';
            if (hours > 0) result += hours + '小时 ';
            if (minutes > 0) result += minutes + '分钟 ';
            result += secs + '秒';
            
            return result;
        }}
        
        // 刷新指标数据
        async function refreshMetrics() {{
            try {{
                const response = await fetch('{}api');
                const data = await response.json();
                
                // 更新运行时间
                document.getElementById('uptime').textContent = formatUptime(data.uptime_seconds);
                
                // 更新请求指标
                document.getElementById('qps').textContent = data.request.current_qps.toFixed(2);
                document.getElementById('totalRequests').textContent = data.request.total_requests.toLocaleString();
                document.getElementById('activeRequests').textContent = data.request.active_requests;
                document.getElementById('avgResponseTime').textContent = data.request.avg_response_time_ms.toFixed(2) + ' ms';
                
                const errorRate = data.request.error_rate * 100;
                const errorRateEl = document.getElementById('errorRate');
                errorRateEl.textContent = errorRate.toFixed(2) + '%';
                errorRateEl.className = 'value ' + (errorRate > 5 ? 'error' : errorRate > 1 ? 'warning' : 'success');
                
                const errorBar = document.getElementById('errorRateBar');
                errorBar.style.width = Math.min(errorRate, 100) + '%';
                errorBar.className = 'fill ' + (errorRate > 5 ? 'error' : errorRate > 1 ? 'warning' : '');
                
                // 更新内存指标
                document.getElementById('memoryUsed').textContent = formatBytes(data.memory.used_bytes);
                document.getElementById('memoryPeak').textContent = formatBytes(data.memory.peak_bytes);
                document.getElementById('memoryUsage').textContent = data.memory.usage_percent.toFixed(1) + '%';
                document.getElementById('gcCount').textContent = data.memory.gc_count;
                
                const memBar = document.getElementById('memoryBar');
                memBar.style.width = data.memory.usage_percent + '%';
                memBar.className = 'fill ' + (data.memory.usage_percent > 80 ? 'error' : data.memory.usage_percent > 60 ? 'warning' : '');
                
                // 更新数据库指标
                document.getElementById('dbConnections').textContent = data.database.active_connections;
                document.getElementById('dbQueries').textContent = data.database.total_queries.toLocaleString();
                document.getElementById('dbSlowQueries').textContent = data.database.slow_queries;
                document.getElementById('dbAvgQueryTime').textContent = data.database.avg_query_time_ms.toFixed(2) + ' ms';
                
                // 更新缓存指标
                document.getElementById('cacheItems').textContent = data.cache.total_items;
                document.getElementById('cacheHitRate').textContent = (data.cache.hit_rate * 100).toFixed(1) + '%';
                document.getElementById('cacheMemory').textContent = formatBytes(data.cache.memory_bytes);
                document.getElementById('cacheEvictions').textContent = data.cache.evictions;
                
                const hitBar = document.getElementById('cacheHitBar');
                hitBar.style.width = (data.cache.hit_rate * 100) + '%';
                
                // 更新队列指标
                document.getElementById('queuePending').textContent = data.queue.pending_messages;
                document.getElementById('queueProcessed').textContent = data.queue.processed_messages.toLocaleString();
                document.getElementById('queueFailed').textContent = data.queue.failed_messages;
                document.getElementById('queueConsumers').textContent = data.queue.active_consumers;
                
                // 更新 WebSocket 指标
                document.getElementById('wsConnections').textContent = data.websocket.active_connections;
                document.getElementById('wsMessagesReceived').textContent = data.websocket.messages_received.toLocaleString();
                document.getElementById('wsMessagesSent').textContent = data.websocket.messages_sent.toLocaleString();
                document.getElementById('wsTraffic').textContent = formatBytes(data.websocket.bytes_received + data.websocket.bytes_sent);
                
            }} catch (error) {{
                console.error('获取指标数据失败:', error);
            }}
        }}
        
        // 更新刷新间隔
        function updateRefreshInterval() {{
            const interval = parseInt(document.getElementById('refreshInterval').value);
            
            // 清除现有定时器
            if (refreshTimer) {{
                clearInterval(refreshTimer);
                refreshTimer = null;
            }}
            
            // 设置新定时器
            if (interval > 0) {{
                refreshTimer = setInterval(refreshMetrics, interval);
            }}
        }}
        
        // 页面加载时初始化
        document.addEventListener('DOMContentLoaded', function() {{
            refreshMetrics();
            updateRefreshInterval();
        }});
    </script>
</body>
</html>"#, config.route)
    }
    
    /// 重置所有数据
    pub fn reset(&self) {
        // 重置收集器
        self.collector.reset();
        
        // 清空历史记录
        if let Ok(mut history) = self.history.write() {
            history.clear();
        }
        
        // 清空告警历史
        if let Ok(mut alerts) = self.alert_history.write() {
            alerts.clear();
        }
    }
}

impl Default for MonitorDashboard {
    fn default() -> Self {
        Self::new(Arc::new(MetricCollector::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试面板创建
    #[test]
    fn test_dashboard_creation() {
        // 创建面板
        let collector = Arc::new(MetricCollector::new());
        let dashboard = MonitorDashboard::new(collector);
        
        // 验证配置
        let config = dashboard.get_config();
        assert!(config.enabled);
        assert_eq!(config.route, "/oyta/monitor");
    }
    
    /// 测试指标快照
    #[test]
    fn test_snapshot() {
        // 创建面板
        let collector = Arc::new(MetricCollector::new());
        let dashboard = MonitorDashboard::new(collector);
        
        // 获取快照
        let snapshot = dashboard.get_metrics();
        
        // 验证
        assert_eq!(snapshot.request.total_requests, 0);
    }
    
    /// 测试告警规则
    #[test]
    fn test_alert_rules() {
        // 创建面板
        let collector = Arc::new(MetricCollector::new());
        let dashboard = MonitorDashboard::new(collector);
        
        // 添加告警规则
        let rule = AlertRule {
            id: "test-rule".to_string(),
            name: "测试规则".to_string(),
            metric: "request.error_rate".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 0.05,
            duration_seconds: 60,
            severity: AlertSeverity::Warning,
            enabled: true,
            notify_channels: vec!["email".to_string()],
        };
        
        dashboard.add_alert_rule(rule);
        
        // 验证
        let rules = dashboard.get_alert_rules();
        assert_eq!(rules.len(), 1);
        
        // 移除规则
        dashboard.remove_alert_rule("test-rule");
        let rules = dashboard.get_alert_rules();
        assert_eq!(rules.len(), 0);
    }
    
    /// 测试 HTML 生成
    #[test]
    fn test_html_generation() {
        // 创建面板
        let collector = Arc::new(MetricCollector::new());
        let dashboard = MonitorDashboard::new(collector);
        
        // 生成 HTML
        let html = dashboard.generate_html();
        
        // 验证
        assert!(html.contains("OYTAPHP 性能监控面板"));
        assert!(html.contains("请求指标"));
    }
}
