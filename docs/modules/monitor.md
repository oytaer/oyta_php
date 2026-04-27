# 监控模块

## 模块概述

监控模块提供实时性能监控能力，收集和展示系统运行时的各类指标，帮助开发者了解系统状态和性能瓶颈。

## 文件结构

```
monitor/
├── mod.rs          # 模块入口
├── collector.rs    # 指标收集器
├── dashboard.rs    # 监控面板
└── metrics.rs      # 指标定义
```

## 核心组件

### MetricsCollector

指标收集器，负责收集各类运行时指标：

```rust
pub struct MetricsCollector {
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
}
```

### Dashboard

监控面板，提供实时数据展示：

```rust
pub struct Dashboard {
    /// 收集器引用
    collector: Arc<MetricsCollector>,
    /// 更新间隔（毫秒）
    update_interval_ms: u64,
    /// 是否启用
    enabled: bool,
}
```

## 指标类型

### 请求指标

```rust
pub struct RequestMetrics {
    /// 总请求数
    pub total_requests: AtomicU64,
    /// 成功请求数
    pub successful_requests: AtomicU64,
    /// 失败请求数
    pub failed_requests: AtomicU64,
    /// 平均响应时间（微秒）
    pub avg_response_time_us: AtomicU64,
    /// 最大响应时间（微秒）
    pub max_response_time_us: AtomicU64,
    /// 最小响应时间（微秒）
    pub min_response_time_us: AtomicU64,
    /// 每秒请求数（QPS）
    pub requests_per_second: AtomicF64,
    /// 按状态码统计
    pub status_codes: DashMap<u16, AtomicU64>,
    /// 按路由统计
    pub route_stats: DashMap<String, RouteStats>,
}
```

### 内存指标

```rust
pub struct MemoryMetrics {
    /// 已使用内存（字节）
    pub used_memory: AtomicU64,
    /// 总内存（字节）
    pub total_memory: AtomicU64,
    /// 内存使用率
    pub memory_usage_percent: AtomicF64,
    /// GC 次数
    pub gc_count: AtomicU64,
    /// GC 总耗时（微秒）
    pub gc_total_time_us: AtomicU64,
}
```

### 数据库指标

```rust
pub struct DatabaseMetrics {
    /// 总查询数
    pub total_queries: AtomicU64,
    /// 慢查询数
    pub slow_queries: AtomicU64,
    /// 平均查询时间（微秒）
    pub avg_query_time_us: AtomicU64,
    /// 最大查询时间（微秒）
    pub max_query_time_us: AtomicU64,
    /// 活跃连接数
    pub active_connections: AtomicU64,
    /// 空闲连接数
    pub idle_connections: AtomicU64,
    /// 连接池大小
    pub pool_size: AtomicU64,
    /// 按表统计
    pub table_stats: DashMap<String, TableStats>,
}
```

### 缓存指标

```rust
pub struct CacheMetrics {
    /// 缓存命中数
    pub cache_hits: AtomicU64,
    /// 缓存未命中数
    pub cache_misses: AtomicU64,
    /// 缓存命中率
    pub hit_rate: AtomicF64,
    /// 缓存大小（字节）
    pub cache_size: AtomicU64,
    /// 缓存条目数
    pub cache_entries: AtomicU64,
    /// 驱逐次数
    pub evictions: AtomicU64,
}
```

### 队列指标

```rust
pub struct QueueMetrics {
    /// 总任务数
    pub total_jobs: AtomicU64,
    /// 已处理任务数
    pub processed_jobs: AtomicU64,
    /// 失败任务数
    pub failed_jobs: AtomicU64,
    /// 待处理任务数
    pub pending_jobs: AtomicU64,
    /// 平均处理时间（微秒）
    pub avg_processing_time_us: AtomicU64,
    /// 按队列统计
    pub queue_stats: DashMap<String, QueueStats>,
}
```

### WebSocket 指标

```rust
pub struct WebSocketMetrics {
    /// 活跃连接数
    pub active_connections: AtomicU64,
    /// 总连接数
    pub total_connections: AtomicU64,
    /// 总消息数
    pub total_messages: AtomicU64,
    /// 发送消息数
    pub sent_messages: AtomicU64,
    /// 接收消息数
    pub received_messages: AtomicU64,
    /// 错误数
    pub errors: AtomicU64,
}
```

## 告警规则

支持配置告警规则，当指标超过阈值时触发告警：

```rust
pub struct AlertRule {
    /// 规则名称
    pub name: String,
    /// 指标类型
    pub metric_type: MetricType,
    /// 阈值
    pub threshold: f64,
    /// 比较操作
    pub operator: CompareOperator,
    /// 持续时间（秒）
    pub duration_secs: u64,
    /// 告警级别
    pub level: AlertLevel,
    /// 通知方式
    pub notify: NotifyType,
}

pub enum CompareOperator {
    GreaterThan,
    LessThan,
    EqualTo,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

pub enum AlertLevel {
    Info,
    Warning,
    Critical,
    Emergency,
}
```

## 使用示例

### 在 PHP 中获取监控数据

```php
// 获取请求统计
$stats = \oyta\Monitor::getRequestStats();
echo "Total requests: " . $stats['total_requests'] . "\n";
echo "Avg response time: " . $stats['avg_response_time_us'] . "us\n";

// 获取缓存统计
$cacheStats = \oyta\Monitor::getCacheStats();
echo "Cache hit rate: " . $cacheStats['hit_rate'] . "%\n";

// 获取数据库统计
$dbStats = \oyta\Monitor::getDatabaseStats();
echo "Slow queries: " . $dbStats['slow_queries'] . "\n";

// 获取内存使用
$memory = \oyta\Monitor::getMemoryUsage();
echo "Memory usage: " . $memory['used_memory'] . " / " . $memory['total_memory'] . "\n";
```

### 配置告警规则

```php
// 配置慢查询告警
\oyta\Monitor::addAlertRule([
    'name' => 'slow_query_alert',
    'metric_type' => 'database.slow_queries',
    'threshold' => 100,
    'operator' => 'greater_than',
    'duration_secs' => 60,
    'level' => 'warning',
    'notify' => 'log',
]);

// 配置内存告警
\oyta\Monitor::addAlertRule([
    'name' => 'memory_alert',
    'metric_type' => 'memory.usage_percent',
    'threshold' => 80,
    'operator' => 'greater_than',
    'duration_secs' => 300,
    'level' => 'critical',
    'notify' => 'log',
]);
```

## 性能影响

监控模块设计为低开销：

- 使用原子操作，无锁并发
- 指标收集为异步操作
- 内存占用固定，不随请求量增长
- CPU 开销 < 1%
