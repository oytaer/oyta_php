//! 指标收集器模块
//!
//! 实现指标的收集、存储和聚合功能
//! 支持多种指标类型和自定义指标

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::metrics::{
    MetricValue, MetricType, MetricCategory, MetricDefinition,
    RequestMetrics, MemoryMetrics, DatabaseMetrics,
    CacheMetrics, QueueMetrics, WebSocketMetrics,
    CustomMetric, HistogramStats,
};

/// 指标收集器
/// 负责收集和管理所有监控指标
#[derive(Debug)]
pub struct MetricCollector {
    /// 请求指标
    request_metrics: RequestMetrics,
    /// 内存指标
    memory_metrics: MemoryMetrics,
    /// 数据库指标
    database_metrics: DatabaseMetrics,
    /// 缓存指标
    cache_metrics: CacheMetrics,
    /// 队列指标
    queue_metrics: QueueMetrics,
    /// WebSocket 指标
    websocket_metrics: WebSocketMetrics,
    /// 自定义指标存储
    custom_metrics: RwLock<HashMap<String, CustomMetric>>,
    /// 直方图统计存储
    histograms: RwLock<HashMap<String, HistogramStats>>,
    /// 收集器启动时间
    start_time: Instant,
    /// 最后一次 QPS 计算时间
    last_qps_calculation: RwLock<Instant>,
    /// 最后一次 QPS 计算时的请求数
    last_request_count: AtomicU64,
    /// 配置选项
    config: CollectorConfig,
}

/// 收集器配置
#[derive(Debug, Clone)]
pub struct CollectorConfig {
    /// 是否启用监控
    pub enabled: bool,
    /// 刷新间隔（毫秒）
    pub refresh_interval_ms: u64,
    /// 最大自定义指标数量
    pub max_custom_metrics: usize,
    /// 是否收集请求体
    pub collect_request_body: bool,
    /// 是否收集响应体
    pub collect_response_body: bool,
    /// 慢查询阈值（毫秒）
    pub slow_query_threshold_ms: u64,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            refresh_interval_ms: 1000,
            max_custom_metrics: 1000,
            collect_request_body: false,
            collect_response_body: false,
            slow_query_threshold_ms: 1000,
        }
    }
}

impl MetricCollector {
    /// 创建新的指标收集器
    pub fn new() -> Self {
        Self::with_config(CollectorConfig::default())
    }
    
    /// 使用配置创建指标收集器
    pub fn with_config(config: CollectorConfig) -> Self {
        let now = Instant::now();
        
        Self {
            request_metrics: RequestMetrics::new(),
            memory_metrics: MemoryMetrics::new(),
            database_metrics: DatabaseMetrics::new(),
            cache_metrics: CacheMetrics::new(),
            queue_metrics: QueueMetrics::new(),
            websocket_metrics: WebSocketMetrics::new(),
            custom_metrics: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
            start_time: now,
            last_qps_calculation: RwLock::new(now),
            last_request_count: AtomicU64::new(0),
            config,
        }
    }
    
    /// 获取请求指标引用
    pub fn request(&self) -> &RequestMetrics {
        &self.request_metrics
    }
    
    /// 获取内存指标引用
    pub fn memory(&self) -> &MemoryMetrics {
        &self.memory_metrics
    }
    
    /// 获取数据库指标引用
    pub fn database(&self) -> &DatabaseMetrics {
        &self.database_metrics
    }
    
    /// 获取缓存指标引用
    pub fn cache(&self) -> &CacheMetrics {
        &self.cache_metrics
    }
    
    /// 获取队列指标引用
    pub fn queue(&self) -> &QueueMetrics {
        &self.queue_metrics
    }
    
    /// 获取 WebSocket 指标引用
    pub fn websocket(&self) -> &WebSocketMetrics {
        &self.websocket_metrics
    }
    
    /// 记录自定义指标
    /// 
    /// # 参数
    /// - `name`: 指标名称
    /// - `value`: 指标值
    pub fn record_metric(&self, name: &str, value: MetricValue) {
        // 检查是否启用
        if !self.config.enabled {
            return;
        }
        
        // 获取写锁
        if let Ok(mut metrics) = self.custom_metrics.write() {
            // 检查是否已存在
            if let Some(metric) = metrics.get_mut(name) {
                // 更新现有指标
                metric.update(value);
            } else {
                // 检查是否超过最大数量
                if metrics.len() >= self.config.max_custom_metrics {
                    // 移除最旧的指标
                    if let Some(oldest) = metrics
                        .iter()
                        .min_by_key(|(_, m)| m.updated_at)
                        .map(|(k, _)| k.clone())
                    {
                        metrics.remove(&oldest);
                    }
                }
                
                // 创建新指标
                let metric = CustomMetric::new(
                    name.to_string(),
                    MetricType::Gauge,
                    value,
                );
                metrics.insert(name.to_string(), metric);
            }
        }
    }
    
    /// 递增计数器指标
    /// 
    /// # 参数
    /// - `name`: 指标名称
    /// - `delta`: 递增量（默认为 1）
    pub fn increment_counter(&self, name: &str, delta: i64) {
        // 获取写锁
        if let Ok(mut metrics) = self.custom_metrics.write() {
            if let Some(metric) = metrics.get_mut(name) {
                // 递增现有计数器
                if let MetricValue::Int(current) = &metric.value {
                    metric.update(MetricValue::Int(current + delta));
                }
            } else {
                // 创建新计数器
                let metric = CustomMetric::new(
                    name.to_string(),
                    MetricType::Counter,
                    MetricValue::Int(delta),
                );
                metrics.insert(name.to_string(), metric);
            }
        }
    }
    
    /// 记录直方图指标
    /// 
    /// # 参数
    /// - `name`: 指标名称
    /// - `value`: 观察值
    pub fn record_histogram(&self, name: &str, value: f64) {
        // 获取写锁
        if let Ok(mut histograms) = self.histograms.write() {
            // 获取或创建直方图
            let histogram = histograms
                .entry(name.to_string())
                .or_insert_with(HistogramStats::new);
            
            // 观察值
            histogram.observe(value);
        }
    }
    
    /// 获取所有指标的快照
    /// 
    /// # 返回
    /// 返回所有指标的 JSON 兼容数据结构
    pub fn get_snapshot(&self) -> MetricSnapshot {
        // 计算当前 QPS
        self.calculate_qps();
        
        // 获取当前时间戳
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        // 收集请求指标
        let request = RequestSnapshot {
            total_requests: self.request_metrics.total_requests.load(Ordering::SeqCst),
            successful_requests: self.request_metrics.successful_requests.load(Ordering::SeqCst),
            failed_requests: self.request_metrics.failed_requests.load(Ordering::SeqCst),
            active_requests: self.request_metrics.active_requests.load(Ordering::SeqCst),
            current_qps: self.request_metrics.current_qps.load(Ordering::SeqCst),
            avg_response_time_ms: self.request_metrics.get_avg_response_time(),
            max_response_time_ms: self.request_metrics.max_response_time_ms.load(Ordering::SeqCst),
            min_response_time_ms: self.request_metrics.min_response_time_ms.load(Ordering::SeqCst),
            error_rate: self.request_metrics.get_error_rate(),
        };
        
        // 收集内存指标
        let memory = MemorySnapshot {
            used_bytes: self.memory_metrics.used_bytes.load(Ordering::SeqCst),
            peak_bytes: self.memory_metrics.peak_bytes.load(Ordering::SeqCst),
            total_bytes: self.memory_metrics.total_bytes.load(Ordering::SeqCst),
            usage_percent: self.memory_metrics.get_usage_percent(),
            gc_count: self.memory_metrics.gc_count.load(Ordering::SeqCst),
            gc_total_time_ms: self.memory_metrics.gc_total_time_ms.load(Ordering::SeqCst),
        };
        
        // 收集数据库指标
        let database = DatabaseSnapshot {
            total_connections: self.database_metrics.total_connections.load(Ordering::SeqCst),
            active_connections: self.database_metrics.active_connections.load(Ordering::SeqCst),
            idle_connections: self.database_metrics.idle_connections.load(Ordering::SeqCst),
            total_queries: self.database_metrics.total_queries.load(Ordering::SeqCst),
            slow_queries: self.database_metrics.slow_queries.load(Ordering::SeqCst),
            avg_query_time_ms: self.database_metrics.get_avg_query_time(),
            max_query_time_ms: self.database_metrics.max_query_time_ms.load(Ordering::SeqCst),
            connection_errors: self.database_metrics.connection_errors.load(Ordering::SeqCst),
            query_errors: self.database_metrics.query_errors.load(Ordering::SeqCst),
        };
        
        // 收集缓存指标
        let cache = CacheSnapshot {
            total_items: self.cache_metrics.total_items.load(Ordering::SeqCst),
            hits: self.cache_metrics.hits.load(Ordering::SeqCst),
            misses: self.cache_metrics.misses.load(Ordering::SeqCst),
            hit_rate: self.cache_metrics.get_hit_rate(),
            memory_bytes: self.cache_metrics.memory_bytes.load(Ordering::SeqCst),
            max_memory_bytes: self.cache_metrics.max_memory_bytes.load(Ordering::SeqCst),
            memory_usage_percent: self.cache_metrics.get_memory_usage_percent(),
            evictions: self.cache_metrics.evictions.load(Ordering::SeqCst),
        };
        
        // 收集队列指标
        let queue = QueueSnapshot {
            pending_messages: self.queue_metrics.pending_messages.load(Ordering::SeqCst),
            processed_messages: self.queue_metrics.processed_messages.load(Ordering::SeqCst),
            failed_messages: self.queue_metrics.failed_messages.load(Ordering::SeqCst),
            retried_messages: self.queue_metrics.retried_messages.load(Ordering::SeqCst),
            dead_letter_messages: self.queue_metrics.dead_letter_messages.load(Ordering::SeqCst),
            active_consumers: self.queue_metrics.active_consumers.load(Ordering::SeqCst),
            avg_processing_time_ms: self.queue_metrics.get_avg_processing_time(),
        };
        
        // 收集 WebSocket 指标
        let websocket = WebSocketSnapshot {
            total_connections: self.websocket_metrics.total_connections.load(Ordering::SeqCst),
            active_connections: self.websocket_metrics.active_connections.load(Ordering::SeqCst),
            messages_received: self.websocket_metrics.messages_received.load(Ordering::SeqCst),
            messages_sent: self.websocket_metrics.messages_sent.load(Ordering::SeqCst),
            bytes_received: self.websocket_metrics.bytes_received.load(Ordering::SeqCst),
            bytes_sent: self.websocket_metrics.bytes_sent.load(Ordering::SeqCst),
            connection_errors: self.websocket_metrics.connection_errors.load(Ordering::SeqCst),
            message_errors: self.websocket_metrics.message_errors.load(Ordering::SeqCst),
        };
        
        // 收集自定义指标
        let custom = if let Ok(metrics) = self.custom_metrics.read() {
            metrics
                .iter()
                .map(|(k, v)| (k.clone(), v.value.clone()))
                .collect()
        } else {
            HashMap::new()
        };
        
        // 收集直方图摘要
        let histograms = if let Ok(hists) = self.histograms.read() {
            hists
                .iter()
                .map(|(k, v)| (k.clone(), v.get_summary()))
                .collect()
        } else {
            HashMap::new()
        };
        
        // 返回快照
        MetricSnapshot {
            timestamp,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            request,
            memory,
            database,
            cache,
            queue,
            websocket,
            custom,
            histograms,
        }
    }
    
    /// 计算 QPS
    fn calculate_qps(&self) {
        // 获取当前请求数
        let current_count = self.request_metrics.total_requests.load(Ordering::SeqCst);
        
        // 获取写锁
        if let Ok(mut last_time) = self.last_qps_calculation.write() {
            // 计算时间差
            let elapsed = last_time.elapsed();
            if elapsed.as_millis() >= 100 {
                // 计算QPS
                let last_count = self.last_request_count.load(Ordering::SeqCst);
                let delta = current_count.saturating_sub(last_count);
                let qps = delta as f64 / elapsed.as_secs_f64();
                
                // 更新 QPS
                self.request_metrics.update_qps(qps);
                
                // 更新最后计算时间和请求数
                *last_time = Instant::now();
                self.last_request_count.store(current_count, Ordering::SeqCst);
            }
        }
    }
    
    /// 重置所有指标
    pub fn reset(&self) {
        // 重置各类指标
        self.request_metrics.reset();
        self.memory_metrics.reset();
        self.database_metrics.reset();
        self.cache_metrics.reset();
        self.queue_metrics.reset();
        self.websocket_metrics.reset();
        
        // 清空自定义指标
        if let Ok(mut metrics) = self.custom_metrics.write() {
            metrics.clear();
        }
        
        // 清空直方图
        if let Ok(mut histograms) = self.histograms.write() {
            histograms.clear();
        }
        
        // 重置 QPS 计算相关
        self.last_request_count.store(0, Ordering::SeqCst);
        if let Ok(mut last_time) = self.last_qps_calculation.write() {
            *last_time = Instant::now();
        }
    }
    
    /// 获取运行时间
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// 更新配置
    pub fn update_config(&mut self, config: CollectorConfig) {
        // 更新慢查询阈值
        self.database_metrics.set_slow_query_threshold(config.slow_query_threshold_ms);
        // 更新缓存最大内存
        self.cache_metrics.set_max_memory(config.max_custom_metrics as u64 * 1024); // 假设每个指标 1KB
        // 保存配置
        self.config = config;
    }
    
    /// 获取配置
    pub fn get_config(&self) -> &CollectorConfig {
        &self.config
    }
    
    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    
    /// 设置启用状态
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }
}

impl Default for MetricCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// 指标快照
/// 包含某一时刻所有指标的数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricSnapshot {
    /// 快照时间戳（毫秒）
    pub timestamp: u64,
    /// 运行时间（秒）
    pub uptime_seconds: u64,
    /// 请求指标
    pub request: RequestSnapshot,
    /// 内存指标
    pub memory: MemorySnapshot,
    /// 数据库指标
    pub database: DatabaseSnapshot,
    /// 缓存指标
    pub cache: CacheSnapshot,
    /// 队列指标
    pub queue: QueueSnapshot,
    /// WebSocket 指标
    pub websocket: WebSocketSnapshot,
    /// 自定义指标
    pub custom: HashMap<String, MetricValue>,
    /// 直方图摘要
    pub histograms: HashMap<String, MetricValue>,
}

/// 请求指标快照
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequestSnapshot {
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 活跃请求数
    pub active_requests: u64,
    /// 当前 QPS
    pub current_qps: f64,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
    /// 最大响应时间（毫秒）
    pub max_response_time_ms: u64,
    /// 最小响应时间（毫秒）
    pub min_response_time_ms: u64,
    /// 错误率
    pub error_rate: f64,
}

/// 内存指标快照
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemorySnapshot {
    /// 已使用内存（字节）
    pub used_bytes: u64,
    /// 内存峰值（字节）
    pub peak_bytes: u64,
    /// 总内存（字节）
    pub total_bytes: u64,
    /// 内存使用率
    pub usage_percent: f64,
    /// GC 次数
    pub gc_count: u64,
    /// GC 总耗时（毫秒）
    pub gc_total_time_ms: u64,
}

/// 数据库指标快照
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DatabaseSnapshot {
    /// 总连接数
    pub total_connections: u64,
    /// 活跃连接数
    pub active_connections: u64,
    /// 空闲连接数
    pub idle_connections: u64,
    /// 总查询数
    pub total_queries: u64,
    /// 慢查询数
    pub slow_queries: u64,
    /// 平均查询时间（毫秒）
    pub avg_query_time_ms: f64,
    /// 最大查询时间（毫秒）
    pub max_query_time_ms: u64,
    /// 连接错误数
    pub connection_errors: u64,
    /// 查询错误数
    pub query_errors: u64,
}

/// 缓存指标快照
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheSnapshot {
    /// 缓存项总数
    pub total_items: u64,
    /// 命中数
    pub hits: u64,
    /// 未命中数
    pub misses: u64,
    /// 命中率
    pub hit_rate: f64,
    /// 内存使用（字节）
    pub memory_bytes: u64,
    /// 最大内存（字节）
    pub max_memory_bytes: u64,
    /// 内存使用率
    pub memory_usage_percent: f64,
    /// 淘汰数
    pub evictions: u64,
}

/// 队列指标快照
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueueSnapshot {
    /// 待处理消息数
    pub pending_messages: u64,
    /// 已处理消息数
    pub processed_messages: u64,
    /// 失败消息数
    pub failed_messages: u64,
    /// 重试消息数
    pub retried_messages: u64,
    /// 死信消息数
    pub dead_letter_messages: u64,
    /// 活跃消费者数
    pub active_consumers: u64,
    /// 平均处理时间（毫秒）
    pub avg_processing_time_ms: f64,
}

/// WebSocket 指标快照
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebSocketSnapshot {
    /// 总连接数
    pub total_connections: u64,
    /// 活跃连接数
    pub active_connections: u64,
    /// 接收消息数
    pub messages_received: u64,
    /// 发送消息数
    pub messages_sent: u64,
    /// 接收字节数
    pub bytes_received: u64,
    /// 发送字节数
    pub bytes_sent: u64,
    /// 连接错误数
    pub connection_errors: u64,
    /// 消息错误数
    pub message_errors: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试指标收集器创建
    #[test]
    fn test_collector_creation() {
        // 创建收集器
        let collector = MetricCollector::new();
        
        // 验证初始状态
        assert!(collector.is_enabled());
        assert_eq!(collector.request().total_requests.load(Ordering::SeqCst), 0);
    }
    
    /// 测试自定义指标记录
    #[test]
    fn test_custom_metric() {
        // 创建收集器
        let collector = MetricCollector::new();
        
        // 记录指标
        collector.record_metric("test_metric", MetricValue::Int(42));
        
        // 验证
        let snapshot = collector.get_snapshot();
        assert!(snapshot.custom.contains_key("test_metric"));
    }
    
    /// 测试直方图记录
    #[test]
    fn test_histogram() {
        // 创建收集器
        let collector = MetricCollector::new();
        
        // 记录直方图值
        collector.record_histogram("response_time", 0.1);
        collector.record_histogram("response_time", 0.2);
        collector.record_histogram("response_time", 0.3);
        
        // 验证
        let snapshot = collector.get_snapshot();
        assert!(snapshot.histograms.contains_key("response_time"));
    }
    
    /// 测试请求指标记录
    #[test]
    fn test_request_metrics() {
        // 创建收集器
        let collector = MetricCollector::new();
        
        // 记录请求
        collector.request().record_request_start();
        collector.request().record_request_end(200, 50);
        
        // 验证
        let snapshot = collector.get_snapshot();
        assert_eq!(snapshot.request.total_requests, 1);
        assert_eq!(snapshot.request.successful_requests, 1);
    }
    
    /// 测试缓存命中率
    #[test]
    fn test_cache_hit_rate() {
        // 创建收集器
        let collector = MetricCollector::new();
        
        // 记录缓存操作
        collector.cache().record_hit();
        collector.cache().record_hit();
        collector.cache().record_miss();
        
        // 验证
        let snapshot = collector.get_snapshot();
        assert!((snapshot.cache.hit_rate - 0.6666666666666666).abs() < 0.0001);
    }
    
    /// 测试重置功能
    #[test]
    fn test_reset() {
        // 创建收集器
        let collector = MetricCollector::new();
        
        // 记录一些数据
        collector.request().record_request_start();
        collector.request().record_request_end(200, 50);
        collector.record_metric("test", MetricValue::Int(1));
        
        // 重置
        collector.reset();
        
        // 验证
        let snapshot = collector.get_snapshot();
        assert_eq!(snapshot.request.total_requests, 0);
        assert!(snapshot.custom.is_empty());
    }
}
