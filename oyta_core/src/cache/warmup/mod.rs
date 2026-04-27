//! 智能缓存预热模块
//!
//! 提供自动分析访问模式、预热热点数据、智能失效功能
//! 基于访问频率和时间衰减算法识别热点数据

// 引入子模块
pub mod analyzer;
pub mod warmer;
pub mod invalidator;

// 重导出主要类型
pub use analyzer::{AccessAnalyzer, AccessPattern, AccessRecord};
pub use warmer::{CacheWarmer, WarmupRule, WarmupStatus};
pub use invalidator::{Invalidator, InvalidationRule, InvalidationEvent};
