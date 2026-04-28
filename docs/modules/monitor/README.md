# 监控模块 (Monitor)

## 模块概述

监控模块提供实时性能监控功能，包括指标收集、面板服务和数据可视化。

## 模块结构

```
monitor/
├── mod.rs          # 模块入口
├── collector.rs    # 指标收集器
├── dashboard.rs    # 监控面板
└── metrics.rs      # 指标定义
```

## 主要类型

| 类型 | 说明 |
|------|------|
| MetricCollector | 指标收集器 |
| MonitorDashboard | 监控面板 |
| RequestMetrics | 请求指标 |
| MemoryMetrics | 内存指标 |
| DatabaseMetrics | 数据库指标 |
| CacheMetrics | 缓存指标 |

## 监控指标

| 指标类型 | 说明 |
|----------|------|
| RequestMetrics | 请求数量、响应时间、错误率 |
| MemoryMetrics | 内存使用、GC 次数 |
| DatabaseMetrics | 查询次数、连接数、慢查询 |
| CacheMetrics | 命中率、缓存大小 |
| QueueMetrics | 队列长度、处理速度 |
| WebSocketMetrics | 连接数、消息数 |

## 使用示例

### 获取监控数据

```php
<?php
// 访问监控面板
// http://localhost:8000/_monitor

// 获取指标数据
$metrics = Monitor::getMetrics();

echo "请求数: " . $metrics['requests']['total'] . "\n";
echo "平均响应时间: " . $metrics['requests']['avg_time'] . "ms\n";
echo "内存使用: " . $metrics['memory']['used'] . "MB\n";
```

### 配置示例

```php
<?php
// config/monitor.php
return [
    'enabled' => true,
    'path' => '/_monitor',
    'collect_interval' => 1000, // 收集间隔（毫秒）
    'retention' => 3600,        // 数据保留时间（秒）
];
```

## 技术实现

- 实时指标收集
- 可视化监控面板
- 多维度指标统计
- 低性能开销
