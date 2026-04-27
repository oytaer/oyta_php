//! 指标定义模块
//!
//! 定义监控系统中使用的所有指标类型和数据结构
//! 支持请求、内存、数据库、缓存、队列、WebSocket 等多类指标

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// 原子浮点数类型
/// 使用 AtomicU64 存储浮点数的位表示
#[derive(Debug)]
pub struct AtomicF64 {
    /// 内部存储（使用 AtomicU64）
    inner: AtomicU64,
}

impl AtomicF64 {
    /// 创建新的原子浮点数
    pub fn new(value: f64) -> Self {
        // 将浮点数转换为位表示
        Self {
            inner: AtomicU64::new(value.to_bits()),
        }
    }
    
    /// 加载浮点数值
    pub fn load(&self, order: Ordering) -> f64 {
        // 从位表示转换回浮点数
        f64::from_bits(self.inner.load(order))
    }
    
    /// 存储浮点数值
    pub fn store(&self, value: f64, order: Ordering) {
        // 将浮点数转换为位表示并存储
        self.inner.store(value.to_bits(), order);
    }
}

impl Default for AtomicF64 {
    fn default() -> Self {
        Self::new(0.0)
    }
}

/// 指标值类型
/// 支持多种数据类型的指标值
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricValue {
    /// 整数值
    Int(i64),
    /// 浮点数值
    Float(f64),
    /// 字符串值
    String(String),
    /// 布尔值
    Bool(bool),
    /// 直方图数据（用于响应时间等分布统计）
    Histogram {
        /// 最小值
        min: f64,
        /// 最大值
        max: f64,
        /// 平均值
        avg: f64,
        /// 中位数
        median: f64,
        /// 第95百分位
        p95: f64,
        /// 第99百分位
        p99: f64,
        /// 样本数量
        count: u64,
    },
    /// 时间序列数据点
    TimeSeries {
        /// 时间戳（毫秒）
        timestamp: u64,
        /// 值
        value: f64,
    },
}

/// 指标类型
/// 定义指标的计算和聚合方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    /// 计数器（只能递增）
    Counter,
    /// 计量器（可增可减）
    Gauge,
    /// 直方图（分布统计）
    Histogram,
    /// 摘要（分位数统计）
    Summary,
}

/// 指标类别
/// 按功能领域分类指标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricCategory {
    /// 请求相关指标
    Request,
    /// 内存相关指标
    Memory,
    /// 数据库相关指标
    Database,
    /// 缓存相关指标
    Cache,
    /// 队列相关指标
    Queue,
    /// WebSocket 相关指标
    WebSocket,
    /// 自定义指标
    Custom,
}

/// 单个指标定义
/// 包含指标的元数据和当前值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefinition {
    /// 指标名称
    pub name: String,
    /// 指标描述
    pub description: String,
    /// 指标类型
    pub metric_type: MetricType,
    /// 指标类别
    pub category: MetricCategory,
    /// 指标标签（用于分组和过滤）
    pub labels: HashMap<String, String>,
    /// 当前值
    pub value: MetricValue,
    /// 更新时间戳（毫秒）
    pub updated_at: u64,
}

/// 请求相关指标
/// 跟踪 HTTP 请求的性能数据
#[derive(Debug, Default)]
pub struct RequestMetrics {
    /// 总请求数
    pub total_requests: AtomicU64,
    /// 成功请求数
    pub successful_requests: AtomicU64,
    /// 失败请求数
    pub failed_requests: AtomicU64,
    /// 总响应时间（毫秒）
    pub total_response_time_ms: AtomicU64,
    /// 最大响应时间（毫秒）
    pub max_response_time_ms: AtomicU64,
    /// 最小响应时间（毫秒）
    pub min_response_time_ms: AtomicU64,
    /// 当前 QPS（每秒查询数）
    pub current_qps: AtomicF64,
    /// 当前活跃请求数
    pub active_requests: AtomicU64,
    /// 按状态码分类的请求数
    pub status_codes: HashMap<u16, AtomicU64>,
}

impl RequestMetrics {
    /// 创建新的请求指标实例
    pub fn new() -> Self {
        // 初始化所有计数器为 0
        let mut status_codes = HashMap::new();
        // 预初始化常见状态码
        for code in [200, 201, 204, 301, 302, 400, 401, 403, 404, 500, 502, 503] {
            status_codes.insert(code, AtomicU64::new(0));
        }
        
        Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_response_time_ms: AtomicU64::new(0),
            max_response_time_ms: AtomicU64::new(0),
            min_response_time_ms: AtomicU64::new(u64::MAX),
            current_qps: AtomicF64::new(0.0),
            active_requests: AtomicU64::new(0),
            status_codes,
        }
    }
    
    /// 记录请求开始
    pub fn record_request_start(&self) {
        // 增加总请求数
        self.total_requests.fetch_add(1, Ordering::SeqCst);
        // 增加活跃请求数
        self.active_requests.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录请求结束
    /// 
    /// # 参数
    /// - `status_code`: HTTP 状态码
    /// - `response_time_ms`: 响应时间（毫秒）
    pub fn record_request_end(&self, status_code: u16, response_time_ms: u64) {
        // 减少活跃请求数
        self.active_requests.fetch_sub(1, Ordering::SeqCst);
        
        // 根据状态码分类
        if status_code >= 200 && status_code < 400 {
            // 2xx 和 3xx 视为成功
            self.successful_requests.fetch_add(1, Ordering::SeqCst);
        } else {
            // 4xx 和 5xx 视为失败
            self.failed_requests.fetch_add(1, Ordering::SeqCst);
        }
        
        // 更新响应时间统计
        self.total_response_time_ms.fetch_add(response_time_ms, Ordering::SeqCst);
        
        // 更新最大响应时间
        let mut current_max = self.max_response_time_ms.load(Ordering::SeqCst);
        while response_time_ms > current_max {
            match self.max_response_time_ms.compare_exchange(
                current_max,
                response_time_ms,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }
        
        // 更新最小响应时间
        let mut current_min = self.min_response_time_ms.load(Ordering::SeqCst);
        while response_time_ms < current_min {
            match self.min_response_time_ms.compare_exchange(
                current_min,
                response_time_ms,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(actual) => current_min = actual,
            }
        }
        
        // 更新状态码计数
        if let Some(counter) = self.status_codes.get(&status_code) {
            counter.fetch_add(1, Ordering::SeqCst);
        }
    }
    
    /// 更新 QPS 值
    pub fn update_qps(&self, qps: f64) {
        self.current_qps.store(qps, Ordering::SeqCst);
    }
    
    /// 获取平均响应时间
    pub fn get_avg_response_time(&self) -> f64 {
        let total = self.total_requests.load(Ordering::SeqCst);
        if total == 0 {
            return 0.0;
        }
        let total_time = self.total_response_time_ms.load(Ordering::SeqCst);
        total_time as f64 / total as f64
    }
    
    /// 获取错误率
    pub fn get_error_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::SeqCst);
        if total == 0 {
            return 0.0;
        }
        let failed = self.failed_requests.load(Ordering::SeqCst);
        failed as f64 / total as f64
    }
    
    /// 重置所有指标
    pub fn reset(&self) {
        // 重置所有计数器
        self.total_requests.store(0, Ordering::SeqCst);
        self.successful_requests.store(0, Ordering::SeqCst);
        self.failed_requests.store(0, Ordering::SeqCst);
        self.total_response_time_ms.store(0, Ordering::SeqCst);
        self.max_response_time_ms.store(0, Ordering::SeqCst);
        self.min_response_time_ms.store(u64::MAX, Ordering::SeqCst);
        self.current_qps.store(0.0, Ordering::SeqCst);
        self.active_requests.store(0, Ordering::SeqCst);
        
        // 重置状态码计数
        for counter in self.status_codes.values() {
            counter.store(0, Ordering::SeqCst);
        }
    }
}

/// 内存相关指标
/// 跟踪内存使用情况
#[derive(Debug, Default)]
pub struct MemoryMetrics {
    /// 当前内存使用量（字节）
    pub used_bytes: AtomicU64,
    /// 内存使用峰值（字节）
    pub peak_bytes: AtomicU64,
    /// 总可用内存（字节）
    pub total_bytes: AtomicU64,
    /// GC 次数
    pub gc_count: AtomicU64,
    /// GC 总耗时（毫秒）
    pub gc_total_time_ms: AtomicU64,
    /// 内存分配次数
    pub allocation_count: AtomicU64,
    /// 内存释放次数
    pub deallocation_count: AtomicU64,
}

impl MemoryMetrics {
    /// 创建新的内存指标实例
    pub fn new() -> Self {
        Self {
            used_bytes: AtomicU64::new(0),
            peak_bytes: AtomicU64::new(0),
            total_bytes: AtomicU64::new(0),
            gc_count: AtomicU64::new(0),
            gc_total_time_ms: AtomicU64::new(0),
            allocation_count: AtomicU64::new(0),
            deallocation_count: AtomicU64::new(0),
        }
    }
    
    /// 更新内存使用量
    pub fn update_memory_usage(&self, used: u64, total: u64) {
        // 更新当前使用量
        self.used_bytes.store(used, Ordering::SeqCst);
        // 更新总内存
        self.total_bytes.store(total, Ordering::SeqCst);
        
        // 更新峰值
        let mut current_peak = self.peak_bytes.load(Ordering::SeqCst);
        while used > current_peak {
            match self.peak_bytes.compare_exchange(
                current_peak,
                used,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(actual) => current_peak = actual,
            }
        }
    }
    
    /// 记录内存分配
    pub fn record_allocation(&self) {
        self.allocation_count.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录内存释放
    pub fn record_deallocation(&self) {
        self.deallocation_count.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录 GC 执行
    pub fn record_gc(&self, duration_ms: u64) {
        self.gc_count.fetch_add(1, Ordering::SeqCst);
        self.gc_total_time_ms.fetch_add(duration_ms, Ordering::SeqCst);
    }
    
    /// 获取内存使用率
    pub fn get_usage_percent(&self) -> f64 {
        let total = self.total_bytes.load(Ordering::SeqCst);
        if total == 0 {
            return 0.0;
        }
        let used = self.used_bytes.load(Ordering::SeqCst);
        (used as f64 / total as f64) * 100.0
    }
    
    /// 重置所有指标
    pub fn reset(&self) {
        self.used_bytes.store(0, Ordering::SeqCst);
        self.peak_bytes.store(0, Ordering::SeqCst);
        self.total_bytes.store(0, Ordering::SeqCst);
        self.gc_count.store(0, Ordering::SeqCst);
        self.gc_total_time_ms.store(0, Ordering::SeqCst);
        self.allocation_count.store(0, Ordering::SeqCst);
        self.deallocation_count.store(0, Ordering::SeqCst);
    }
}

/// 数据库相关指标
/// 跟踪数据库连接和查询性能
#[derive(Debug, Default)]
pub struct DatabaseMetrics {
    /// 总连接数
    pub total_connections: AtomicU64,
    /// 活跃连接数
    pub active_connections: AtomicU64,
    /// 空闲连接数
    pub idle_connections: AtomicU64,
    /// 总查询数
    pub total_queries: AtomicU64,
    /// 慢查询数
    pub slow_queries: AtomicU64,
    /// 查询总耗时（毫秒）
    pub total_query_time_ms: AtomicU64,
    /// 最大查询时间（毫秒）
    pub max_query_time_ms: AtomicU64,
    /// 连接错误数
    pub connection_errors: AtomicU64,
    /// 查询错误数
    pub query_errors: AtomicU64,
    /// 慢查询阈值（毫秒）
    pub slow_query_threshold_ms: AtomicU64,
}

impl DatabaseMetrics {
    /// 创建新的数据库指标实例
    pub fn new() -> Self {
        Self {
            total_connections: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            idle_connections: AtomicU64::new(0),
            total_queries: AtomicU64::new(0),
            slow_queries: AtomicU64::new(0),
            total_query_time_ms: AtomicU64::new(0),
            max_query_time_ms: AtomicU64::new(0),
            connection_errors: AtomicU64::new(0),
            query_errors: AtomicU64::new(0),
            slow_query_threshold_ms: AtomicU64::new(1000), // 默认 1 秒
        }
    }
    
    /// 记录连接创建
    pub fn record_connection_created(&self) {
        self.total_connections.fetch_add(1, Ordering::SeqCst);
        self.active_connections.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录连接释放
    pub fn record_connection_released(&self) {
        self.active_connections.fetch_sub(1, Ordering::SeqCst);
        self.idle_connections.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录连接获取
    pub fn record_connection_acquired(&self) {
        self.idle_connections.fetch_sub(1, Ordering::SeqCst);
        self.active_connections.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录查询执行
    pub fn record_query(&self, duration_ms: u64, is_error: bool) {
        // 增加总查询数
        self.total_queries.fetch_add(1, Ordering::SeqCst);
        
        // 如果是错误，增加错误计数
        if is_error {
            self.query_errors.fetch_add(1, Ordering::SeqCst);
            return;
        }
        
        // 更新查询时间统计
        self.total_query_time_ms.fetch_add(duration_ms, Ordering::SeqCst);
        
        // 更新最大查询时间
        let mut current_max = self.max_query_time_ms.load(Ordering::SeqCst);
        while duration_ms > current_max {
            match self.max_query_time_ms.compare_exchange(
                current_max,
                duration_ms,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }
        
        // 检查是否为慢查询
        let threshold = self.slow_query_threshold_ms.load(Ordering::SeqCst);
        if duration_ms > threshold {
            self.slow_queries.fetch_add(1, Ordering::SeqCst);
        }
    }
    
    /// 记录连接错误
    pub fn record_connection_error(&self) {
        self.connection_errors.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 设置慢查询阈值
    pub fn set_slow_query_threshold(&self, threshold_ms: u64) {
        self.slow_query_threshold_ms.store(threshold_ms, Ordering::SeqCst);
    }
    
    /// 获取平均查询时间
    pub fn get_avg_query_time(&self) -> f64 {
        let total = self.total_queries.load(Ordering::SeqCst);
        if total == 0 {
            return 0.0;
        }
        let total_time = self.total_query_time_ms.load(Ordering::SeqCst);
        total_time as f64 / total as f64
    }
    
    /// 重置所有指标
    pub fn reset(&self) {
        self.total_connections.store(0, Ordering::SeqCst);
        self.active_connections.store(0, Ordering::SeqCst);
        self.idle_connections.store(0, Ordering::SeqCst);
        self.total_queries.store(0, Ordering::SeqCst);
        self.slow_queries.store(0, Ordering::SeqCst);
        self.total_query_time_ms.store(0, Ordering::SeqCst);
        self.max_query_time_ms.store(0, Ordering::SeqCst);
        self.connection_errors.store(0, Ordering::SeqCst);
        self.query_errors.store(0, Ordering::SeqCst);
    }
}

/// 缓存相关指标
/// 跟踪缓存命中率和内存使用
#[derive(Debug, Default)]
pub struct CacheMetrics {
    /// 总缓存项数
    pub total_items: AtomicU64,
    /// 缓存命中数
    pub hits: AtomicU64,
    /// 缓存未命中数
    pub misses: AtomicU64,
    /// 缓存内存使用量（字节）
    pub memory_bytes: AtomicU64,
    /// 最大内存限制（字节）
    pub max_memory_bytes: AtomicU64,
    /// 缓存淘汰数
    pub evictions: AtomicU64,
    /// 缓存写入数
    pub writes: AtomicU64,
    /// 缓存删除数
    pub deletes: AtomicU64,
}

impl CacheMetrics {
    /// 创建新的缓存指标实例
    pub fn new() -> Self {
        Self {
            total_items: AtomicU64::new(0),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            memory_bytes: AtomicU64::new(0),
            max_memory_bytes: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            writes: AtomicU64::new(0),
            deletes: AtomicU64::new(0),
        }
    }
    
    /// 记录缓存命中
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录缓存未命中
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录缓存写入
    pub fn record_write(&self, size_bytes: u64) {
        self.writes.fetch_add(1, Ordering::SeqCst);
        self.total_items.fetch_add(1, Ordering::SeqCst);
        self.memory_bytes.fetch_add(size_bytes, Ordering::SeqCst);
    }
    
    /// 记录缓存删除
    pub fn record_delete(&self, size_bytes: u64) {
        self.deletes.fetch_add(1, Ordering::SeqCst);
        self.total_items.fetch_sub(1, Ordering::SeqCst);
        self.memory_bytes.fetch_sub(size_bytes, Ordering::SeqCst);
    }
    
    /// 记录缓存淘汰
    pub fn record_eviction(&self, size_bytes: u64) {
        self.evictions.fetch_add(1, Ordering::SeqCst);
        self.total_items.fetch_sub(1, Ordering::SeqCst);
        self.memory_bytes.fetch_sub(size_bytes, Ordering::SeqCst);
    }
    
    /// 设置最大内存限制
    pub fn set_max_memory(&self, max_bytes: u64) {
        self.max_memory_bytes.store(max_bytes, Ordering::SeqCst);
    }
    
    /// 获取缓存命中率
    pub fn get_hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::SeqCst);
        let misses = self.misses.load(Ordering::SeqCst);
        let total = hits + misses;
        if total == 0 {
            return 0.0;
        }
        hits as f64 / total as f64
    }
    
    /// 获取内存使用率
    pub fn get_memory_usage_percent(&self) -> f64 {
        let max = self.max_memory_bytes.load(Ordering::SeqCst);
        if max == 0 {
            return 0.0;
        }
        let used = self.memory_bytes.load(Ordering::SeqCst);
        (used as f64 / max as f64) * 100.0
    }
    
    /// 重置所有指标
    pub fn reset(&self) {
        self.total_items.store(0, Ordering::SeqCst);
        self.hits.store(0, Ordering::SeqCst);
        self.misses.store(0, Ordering::SeqCst);
        self.memory_bytes.store(0, Ordering::SeqCst);
        self.max_memory_bytes.store(0, Ordering::SeqCst);
        self.evictions.store(0, Ordering::SeqCst);
        self.writes.store(0, Ordering::SeqCst);
        self.deletes.store(0, Ordering::SeqCst);
    }
}

/// 队列相关指标
/// 跟踪消息队列的状态
#[derive(Debug, Default)]
pub struct QueueMetrics {
    /// 队列中待处理消息数
    pub pending_messages: AtomicU64,
    /// 已处理消息数
    pub processed_messages: AtomicU64,
    /// 失败消息数
    pub failed_messages: AtomicU64,
    /// 重试消息数
    pub retried_messages: AtomicU64,
    /// 死信消息数
    pub dead_letter_messages: AtomicU64,
    /// 当前活跃消费者数
    pub active_consumers: AtomicU64,
    /// 消息处理总耗时（毫秒）
    pub total_processing_time_ms: AtomicU64,
    /// 最大处理时间（毫秒）
    pub max_processing_time_ms: AtomicU64,
}

impl QueueMetrics {
    /// 创建新的队列指标实例
    pub fn new() -> Self {
        Self {
            pending_messages: AtomicU64::new(0),
            processed_messages: AtomicU64::new(0),
            failed_messages: AtomicU64::new(0),
            retried_messages: AtomicU64::new(0),
            dead_letter_messages: AtomicU64::new(0),
            active_consumers: AtomicU64::new(0),
            total_processing_time_ms: AtomicU64::new(0),
            max_processing_time_ms: AtomicU64::new(0),
        }
    }
    
    /// 记录消息入队
    pub fn record_message_enqueued(&self) {
        self.pending_messages.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录消息开始处理
    pub fn record_message_processing_start(&self) {
        self.pending_messages.fetch_sub(1, Ordering::SeqCst);
    }
    
    /// 记录消息处理完成
    pub fn record_message_processed(&self, duration_ms: u64) {
        self.processed_messages.fetch_add(1, Ordering::SeqCst);
        self.total_processing_time_ms.fetch_add(duration_ms, Ordering::SeqCst);
        
        // 更新最大处理时间
        let mut current_max = self.max_processing_time_ms.load(Ordering::SeqCst);
        while duration_ms > current_max {
            match self.max_processing_time_ms.compare_exchange(
                current_max,
                duration_ms,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }
    }
    
    /// 记录消息处理失败
    pub fn record_message_failed(&self) {
        self.failed_messages.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录消息重试
    pub fn record_message_retried(&self) {
        self.retried_messages.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录死信消息
    pub fn record_dead_letter(&self) {
        self.dead_letter_messages.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录消费者上线
    pub fn record_consumer_online(&self) {
        self.active_consumers.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录消费者下线
    pub fn record_consumer_offline(&self) {
        self.active_consumers.fetch_sub(1, Ordering::SeqCst);
    }
    
    /// 获取平均处理时间
    pub fn get_avg_processing_time(&self) -> f64 {
        let total = self.processed_messages.load(Ordering::SeqCst);
        if total == 0 {
            return 0.0;
        }
        let total_time = self.total_processing_time_ms.load(Ordering::SeqCst);
        total_time as f64 / total as f64
    }
    
    /// 重置所有指标
    pub fn reset(&self) {
        self.pending_messages.store(0, Ordering::SeqCst);
        self.processed_messages.store(0, Ordering::SeqCst);
        self.failed_messages.store(0, Ordering::SeqCst);
        self.retried_messages.store(0, Ordering::SeqCst);
        self.dead_letter_messages.store(0, Ordering::SeqCst);
        self.active_consumers.store(0, Ordering::SeqCst);
        self.total_processing_time_ms.store(0, Ordering::SeqCst);
        self.max_processing_time_ms.store(0, Ordering::SeqCst);
    }
}

/// WebSocket 相关指标
/// 跟踪 WebSocket 连接和消息
#[derive(Debug, Default)]
pub struct WebSocketMetrics {
    /// 总连接数
    pub total_connections: AtomicU64,
    /// 当前活跃连接数
    pub active_connections: AtomicU64,
    /// 接收的消息数
    pub messages_received: AtomicU64,
    /// 发送的消息数
    pub messages_sent: AtomicU64,
    /// 接收的字节数
    pub bytes_received: AtomicU64,
    /// 发送的字节数
    pub bytes_sent: AtomicU64,
    /// 连接错误数
    pub connection_errors: AtomicU64,
    /// 消息错误数
    pub message_errors: AtomicU64,
}

impl WebSocketMetrics {
    /// 创建新的 WebSocket 指标实例
    pub fn new() -> Self {
        Self {
            total_connections: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            messages_received: AtomicU64::new(0),
            messages_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            connection_errors: AtomicU64::new(0),
            message_errors: AtomicU64::new(0),
        }
    }
    
    /// 记录新连接
    pub fn record_connection(&self) {
        self.total_connections.fetch_add(1, Ordering::SeqCst);
        self.active_connections.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录连接断开
    pub fn record_disconnection(&self) {
        self.active_connections.fetch_sub(1, Ordering::SeqCst);
    }
    
    /// 记录消息接收
    pub fn record_message_received(&self, size_bytes: u64) {
        self.messages_received.fetch_add(1, Ordering::SeqCst);
        self.bytes_received.fetch_add(size_bytes, Ordering::SeqCst);
    }
    
    /// 记录消息发送
    pub fn record_message_sent(&self, size_bytes: u64) {
        self.messages_sent.fetch_add(1, Ordering::SeqCst);
        self.bytes_sent.fetch_add(size_bytes, Ordering::SeqCst);
    }
    
    /// 记录连接错误
    pub fn record_connection_error(&self) {
        self.connection_errors.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 记录消息错误
    pub fn record_message_error(&self) {
        self.message_errors.fetch_add(1, Ordering::SeqCst);
    }
    
    /// 重置所有指标
    pub fn reset(&self) {
        self.total_connections.store(0, Ordering::SeqCst);
        self.active_connections.store(0, Ordering::SeqCst);
        self.messages_received.store(0, Ordering::SeqCst);
        self.messages_sent.store(0, Ordering::SeqCst);
        self.bytes_received.store(0, Ordering::SeqCst);
        self.bytes_sent.store(0, Ordering::SeqCst);
        self.connection_errors.store(0, Ordering::SeqCst);
        self.message_errors.store(0, Ordering::SeqCst);
    }
}

/// 自定义指标
/// 用于存储用户自定义的监控指标
#[derive(Debug, Clone)]
pub struct CustomMetric {
    /// 指标名称
    pub name: String,
    /// 指标类型
    pub metric_type: MetricType,
    /// 指标值
    pub value: MetricValue,
    /// 标签
    pub labels: HashMap<String, String>,
    /// 创建时间
    pub created_at: Instant,
    /// 最后更新时间
    pub updated_at: Instant,
}

impl CustomMetric {
    /// 创建新的自定义指标
    pub fn new(name: String, metric_type: MetricType, value: MetricValue) -> Self {
        let now = Instant::now();
        Self {
            name,
            metric_type,
            value,
            labels: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// 更新指标值
    pub fn update(&mut self, value: MetricValue) {
        self.value = value;
        self.updated_at = Instant::now();
    }
    
    /// 添加标签
    pub fn add_label(&mut self, key: String, value: String) {
        self.labels.insert(key, value);
    }
}

/// 直方图桶
/// 用于计算分位数统计
#[derive(Debug, Clone)]
pub struct HistogramBucket {
    /// 桶的上界
    pub upper_bound: f64,
    /// 桶中的计数
    pub count: u64,
}

impl HistogramBucket {
    /// 创建新的直方图桶
    pub fn new(upper_bound: f64) -> Self {
        Self {
            upper_bound,
            count: 0,
        }
    }
    
    /// 增加计数
    pub fn increment(&mut self) {
        self.count += 1;
    }
}

/// 直方图统计器
/// 用于计算响应时间等分布统计
#[derive(Debug, Clone)]
pub struct HistogramStats {
    /// 桶列表
    pub buckets: Vec<HistogramBucket>,
    /// 总和
    pub sum: f64,
    /// 计数
    pub count: u64,
}

impl HistogramStats {
    /// 创建新的直方图统计器
    /// 使用默认的桶边界
    pub fn new() -> Self {
        // 默认桶边界：0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10
        let buckets = vec![
            HistogramBucket::new(0.005),
            HistogramBucket::new(0.01),
            HistogramBucket::new(0.025),
            HistogramBucket::new(0.05),
            HistogramBucket::new(0.1),
            HistogramBucket::new(0.25),
            HistogramBucket::new(0.5),
            HistogramBucket::new(1.0),
            HistogramBucket::new(2.5),
            HistogramBucket::new(5.0),
            HistogramBucket::new(10.0),
            HistogramBucket::new(f64::INFINITY),
        ];
        
        Self {
            buckets,
            sum: 0.0,
            count: 0,
        }
    }
    
    /// 使用自定义桶边界创建直方图
    pub fn with_buckets(bounds: Vec<f64>) -> Self {
        let buckets: Vec<HistogramBucket> = bounds
            .into_iter()
            .map(HistogramBucket::new)
            .chain(std::iter::once(HistogramBucket::new(f64::INFINITY)))
            .collect();
        
        Self {
            buckets,
            sum: 0.0,
            count: 0,
        }
    }
    
    /// 观察一个值
    pub fn observe(&mut self, value: f64) {
        // 更新总和和计数
        self.sum += value;
        self.count += 1;
        
        // 更新桶计数
        for bucket in &mut self.buckets {
            if value <= bucket.upper_bound {
                bucket.increment();
            }
        }
    }
    
    /// 获取平均值
    pub fn get_avg(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        self.sum / self.count as f64
    }
    
    /// 计算分位数
    pub fn get_quantile(&self, q: f64) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        
        let target = (self.count as f64 * q) as u64;
        let mut cumulative = 0u64;
        
        for bucket in &self.buckets {
            cumulative += bucket.count;
            if cumulative >= target {
                return bucket.upper_bound;
            }
        }
        
        self.buckets.last().map(|b| b.upper_bound).unwrap_or(0.0)
    }
    
    /// 获取统计摘要
    pub fn get_summary(&self) -> MetricValue {
        MetricValue::Histogram {
            min: self.buckets.first().map(|b| b.upper_bound).unwrap_or(0.0),
            max: self.buckets.last().map(|b| b.upper_bound).unwrap_or(0.0),
            avg: self.get_avg(),
            median: self.get_quantile(0.5),
            p95: self.get_quantile(0.95),
            p99: self.get_quantile(0.99),
            count: self.count,
        }
    }
    
    /// 重置统计
    pub fn reset(&mut self) {
        for bucket in &mut self.buckets {
            bucket.count = 0;
        }
        self.sum = 0.0;
        self.count = 0;
    }
}

impl Default for HistogramStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试请求指标
    #[test]
    fn test_request_metrics() {
        // 创建请求指标
        let metrics = RequestMetrics::new();
        
        // 记录请求
        metrics.record_request_start();
        metrics.record_request_end(200, 50);
        
        // 验证
        assert_eq!(metrics.total_requests.load(Ordering::SeqCst), 1);
        assert_eq!(metrics.successful_requests.load(Ordering::SeqCst), 1);
    }
    
    /// 测试缓存命中率
    #[test]
    fn test_cache_hit_rate() {
        // 创建缓存指标
        let metrics = CacheMetrics::new();
        
        // 记录命中和未命中
        metrics.record_hit();
        metrics.record_hit();
        metrics.record_miss();
        
        // 验证命中率
        let rate = metrics.get_hit_rate();
        assert!((rate - 0.6666666666666666).abs() < 0.0001);
    }
    
    /// 测试直方图统计
    #[test]
    fn test_histogram_stats() {
        // 创建直方图
        let mut histogram = HistogramStats::new();
        
        // 观察一些值
        histogram.observe(0.1);
        histogram.observe(0.2);
        histogram.observe(0.3);
        histogram.observe(0.4);
        histogram.observe(0.5);
        
        // 验证平均值
        let avg = histogram.get_avg();
        assert!((avg - 0.3).abs() < 0.0001);
        
        // 验证计数
        assert_eq!(histogram.count, 5);
    }
}
