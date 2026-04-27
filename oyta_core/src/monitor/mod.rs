//! 实时性能监控面板模块
//!
//! 提供实时性能监控功能，包括指标收集、面板服务和数据可视化
//! 基于现有 Trace 调试模块增强，提供可视化界面

// 引入子模块
pub mod collector;
pub mod dashboard;
pub mod metrics;

// 重导出主要类型
pub use collector::MetricCollector;
pub use dashboard::MonitorDashboard;
pub use metrics::{
    MetricValue, MetricType, MetricCategory, 
    RequestMetrics, MemoryMetrics, DatabaseMetrics,
    CacheMetrics, QueueMetrics, WebSocketMetrics,
};
