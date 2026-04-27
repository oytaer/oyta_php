//! 访问分析器模块
//!
//! 分析缓存访问模式，识别热点数据
//! 使用时间衰减算法和访问频率统计

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// 访问记录
/// 记录单个缓存键的访问信息
#[derive(Debug, Clone)]
pub struct AccessRecord {
    /// 缓存键
    pub key: String,
    /// 访问次数
    pub access_count: u64,
    /// 最后访问时间戳（毫秒）
    pub last_access_time: u64,
    /// 首次访问时间戳（毫秒）
    pub first_access_time: u64,
    /// 平均访问间隔（毫秒）
    pub avg_access_interval: f64,
    /// 访问间隔总和（毫秒）
    pub total_interval: u64,
    /// 上次访问时间（用于计算间隔）
    pub prev_access_time: u64,
    /// 热度分数
    pub hot_score: f64,
    /// 数据大小（字节）
    pub data_size: u64,
    /// 命中次数
    pub hit_count: u64,
    /// 未命中次数
    pub miss_count: u64,
}

impl AccessRecord {
    /// 创建新的访问记录
    pub fn new(key: String) -> Self {
        let now = Self::current_timestamp();
        
        Self {
            key,
            access_count: 1,
            last_access_time: now,
            first_access_time: now,
            avg_access_interval: 0.0,
            total_interval: 0,
            prev_access_time: now,
            hot_score: 0.0,
            data_size: 0,
            hit_count: 0,
            miss_count: 0,
        }
    }
    
    /// 获取当前时间戳（毫秒）
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
    
    /// 记录访问
    pub fn record_access(&mut self, is_hit: bool) {
        let now = Self::current_timestamp();
        
        // 计算访问间隔
        let interval = now.saturating_sub(self.prev_access_time);
        
        // 更新统计
        self.access_count += 1;
        self.total_interval += interval;
        self.avg_access_interval = self.total_interval as f64 / (self.access_count - 1).max(1) as f64;
        self.prev_access_time = now;
        self.last_access_time = now;
        
        // 更新命中/未命中计数
        if is_hit {
            self.hit_count += 1;
        } else {
            self.miss_count += 1;
        }
    }
    
    /// 计算热度分数
    /// 使用时间衰减算法：score = count * e^(-decay * age)
    pub fn calculate_hot_score(&mut self, decay_factor: f64) {
        let now = Self::current_timestamp();
        let age_seconds = (now - self.last_access_time) as f64 / 1000.0;
        
        // 时间衰减
        let decay = (-decay_factor * age_seconds).exp();
        
        // 访问频率因子
        let frequency_factor = self.access_count as f64;
        
        // 命中率因子
        let total = self.hit_count + self.miss_count;
        let hit_rate = if total > 0 {
            self.hit_count as f64 / total as f64
        } else {
            0.0
        };
        
        // 综合热度分数
        self.hot_score = frequency_factor * decay * (1.0 + hit_rate);
    }
    
    /// 获取命中率
    pub fn get_hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            return 0.0;
        }
        self.hit_count as f64 / total as f64
    }
}

/// 访问模式
/// 描述缓存的访问模式特征
#[derive(Debug, Clone)]
pub struct AccessPattern {
    /// 模式名称
    pub name: String,
    /// 模式描述
    pub description: String,
    /// 热点键列表
    pub hot_keys: Vec<String>,
    /// 访问频率阈值
    pub frequency_threshold: u64,
    /// 时间窗口（秒）
    pub time_window: u64,
    /// 模式类型
    pub pattern_type: PatternType,
}

/// 访问模式类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    /// 热点数据模式
    HotSpot,
    /// 周期性访问模式
    Periodic,
    /// 突发访问模式
    Burst,
    /// 长尾访问模式
    LongTail,
}

/// 访问分析器配置
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// 时间衰减因子
    pub decay_factor: f64,
    /// 热点阈值（访问次数）
    pub hot_threshold: u64,
    /// 分析窗口大小（秒）
    pub analysis_window: u64,
    /// 最大记录数
    pub max_records: usize,
    /// 最小访问次数（用于过滤）
    pub min_access_count: u64,
    /// 是否启用自动分析
    pub auto_analyze: bool,
    /// 自动分析间隔（秒）
    pub auto_analyze_interval: u64,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            decay_factor: 0.01,
            hot_threshold: 10,
            analysis_window: 3600,
            max_records: 10000,
            min_access_count: 1,
            auto_analyze: true,
            auto_analyze_interval: 300,
        }
    }
}

/// 访问分析器
/// 分析缓存访问模式并识别热点数据
#[derive(Debug)]
pub struct AccessAnalyzer {
    /// 访问记录存储
    records: RwLock<HashMap<String, AccessRecord>>,
    /// 配置
    config: RwLock<AnalyzerConfig>,
    /// 总访问次数
    total_accesses: AtomicU64,
    /// 总命中次数
    total_hits: AtomicU64,
    /// 分析时间戳
    last_analysis_time: RwLock<u64>,
    /// 分析结果缓存
    analysis_result: RwLock<Option<AnalysisResult>>,
}

/// 分析结果
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// 分析时间戳
    pub timestamp: u64,
    /// 热点键列表
    pub hot_keys: Vec<HotKeyInfo>,
    /// 访问模式
    pub patterns: Vec<AccessPattern>,
    /// 总访问次数
    pub total_accesses: u64,
    /// 总命中次数
    pub total_hits: u64,
    /// 整体命中率
    pub overall_hit_rate: f64,
    /// 平均访问间隔
    pub avg_access_interval: f64,
}

/// 热点键信息
#[derive(Debug, Clone)]
pub struct HotKeyInfo {
    /// 缓存键
    pub key: String,
    /// 热度分数
    pub hot_score: f64,
    /// 访问次数
    pub access_count: u64,
    /// 命中率
    pub hit_rate: f64,
    /// 数据大小
    pub data_size: u64,
}

impl AccessAnalyzer {
    /// 创建新的访问分析器
    pub fn new() -> Self {
        Self::with_config(AnalyzerConfig::default())
    }
    
    /// 使用配置创建访问分析器
    pub fn with_config(config: AnalyzerConfig) -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
            config: RwLock::new(config),
            total_accesses: AtomicU64::new(0),
            total_hits: AtomicU64::new(0),
            last_analysis_time: RwLock::new(0),
            analysis_result: RwLock::new(None),
        }
    }
    
    /// 记录缓存访问
    /// 
    /// # 参数
    /// - `key`: 缓存键
    /// - `is_hit`: 是否命中
    /// - `data_size`: 数据大小（字节）
    pub fn record_access(&self, key: &str, is_hit: bool, data_size: u64) {
        // 更新总计数
        self.total_accesses.fetch_add(1, Ordering::SeqCst);
        if is_hit {
            self.total_hits.fetch_add(1, Ordering::SeqCst);
        }
        
        // 获取写锁
        if let Ok(mut records) = self.records.write() {
            // 获取或创建记录
            let record = records.entry(key.to_string())
                .or_insert_with(|| AccessRecord::new(key.to_string()));
            
            // 更新数据大小
            if data_size > 0 {
                record.data_size = data_size;
            }
            
            // 记录访问
            record.record_access(is_hit);
            
            // 检查是否超过最大记录数
            let max_records = self.config.read().map(|c| c.max_records).unwrap_or(10000);
            if records.len() > max_records {
                // 移除最旧的记录
                self.evict_oldest(&mut records);
            }
        }
    }
    
    /// 移除最旧的记录
    fn evict_oldest(&self, records: &mut HashMap<String, AccessRecord>) {
        // 找到最旧的记录
        if let Some(oldest_key) = records
            .iter()
            .min_by_key(|(_, r)| r.last_access_time)
            .map(|(k, _)| k.clone())
        {
            records.remove(&oldest_key);
        }
    }
    
    /// 执行分析
    /// 
    /// # 返回
    /// 返回分析结果
    pub fn analyze(&self) -> AnalysisResult {
        let now = Self::current_timestamp();
        
        // 获取配置
        let config = self.config.read().unwrap().clone();
        
        // 获取读锁
        let records = self.records.read().unwrap();
        
        // 计算每个记录的热度分数
        let mut records_clone: Vec<AccessRecord> = records.values().cloned().collect();
        for record in &mut records_clone {
            record.calculate_hot_score(config.decay_factor);
        }
        
        // 排序并获取热点键
        records_clone.sort_by(|a, b| {
            b.hot_score.partial_cmp(&a.hot_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // 筛选热点键
        let hot_keys: Vec<HotKeyInfo> = records_clone
            .iter()
            .filter(|r| r.access_count >= config.hot_threshold)
            .take(100)
            .map(|r| HotKeyInfo {
                key: r.key.clone(),
                hot_score: r.hot_score,
                access_count: r.access_count,
                hit_rate: r.get_hit_rate(),
                data_size: r.data_size,
            })
            .collect();
        
        // 识别访问模式
        let patterns = self.identify_patterns(&records_clone, &config);
        
        // 计算统计信息
        let total_accesses = self.total_accesses.load(Ordering::SeqCst);
        let total_hits = self.total_hits.load(Ordering::SeqCst);
        let overall_hit_rate = if total_accesses > 0 {
            total_hits as f64 / total_accesses as f64
        } else {
            0.0
        };
        
        // 计算平均访问间隔
        let avg_access_interval = if !records_clone.is_empty() {
            records_clone.iter().map(|r| r.avg_access_interval).sum::<f64>() / records_clone.len() as f64
        } else {
            0.0
        };
        
        // 构建结果
        let result = AnalysisResult {
            timestamp: now,
            hot_keys,
            patterns,
            total_accesses,
            total_hits,
            overall_hit_rate,
            avg_access_interval,
        };
        
        // 缓存结果
        if let Ok(mut cached) = self.analysis_result.write() {
            *cached = Some(result.clone());
        }
        
        // 更新分析时间
        if let Ok(mut last_time) = self.last_analysis_time.write() {
            *last_time = now;
        }
        
        result
    }
    
    /// 识别访问模式
    fn identify_patterns(&self, records: &[AccessRecord], config: &AnalyzerConfig) -> Vec<AccessPattern> {
        let mut patterns = Vec::new();
        
        // 识别热点数据模式
        let hot_keys: Vec<String> = records
            .iter()
            .filter(|r| r.access_count >= config.hot_threshold)
            .take(20)
            .map(|r| r.key.clone())
            .collect();
        
        if !hot_keys.is_empty() {
            patterns.push(AccessPattern {
                name: "热点数据".to_string(),
                description: format!("识别出 {} 个热点键", hot_keys.len()),
                hot_keys,
                frequency_threshold: config.hot_threshold,
                time_window: config.analysis_window,
                pattern_type: PatternType::HotSpot,
            });
        }
        
        // 识别周期性访问模式
        let periodic_keys: Vec<String> = records
            .iter()
            .filter(|r| {
                // 检查是否有规律的访问间隔
                r.access_count >= 5 && r.avg_access_interval > 0.0
            })
            .take(10)
            .map(|r| r.key.clone())
            .collect();
        
        if !periodic_keys.is_empty() {
            patterns.push(AccessPattern {
                name: "周期性访问".to_string(),
                description: format!("识别出 {} 个周期性访问键", periodic_keys.len()),
                hot_keys: periodic_keys,
                frequency_threshold: 5,
                time_window: config.analysis_window,
                pattern_type: PatternType::Periodic,
            });
        }
        
        // 识别突发访问模式
        let burst_keys: Vec<String> = records
            .iter()
            .filter(|r| {
                // 检查是否有突发访问特征
                let now = Self::current_timestamp();
                let recent_access = (now - r.last_access_time) < 60000; // 最近1分钟
                r.access_count >= 3 && recent_access
            })
            .take(10)
            .map(|r| r.key.clone())
            .collect();
        
        if !burst_keys.is_empty() {
            patterns.push(AccessPattern {
                name: "突发访问".to_string(),
                description: format!("识别出 {} 个突发访问键", burst_keys.len()),
                hot_keys: burst_keys,
                frequency_threshold: 3,
                time_window: 60,
                pattern_type: PatternType::Burst,
            });
        }
        
        patterns
    }
    
    /// 获取当前时间戳（毫秒）
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
    
    /// 获取热点键列表
    /// 
    /// # 参数
    /// - `limit`: 最大返回数量
    pub fn get_hot_keys(&self, limit: usize) -> Vec<String> {
        // 尝试使用缓存的分析结果
        if let Ok(cached) = self.analysis_result.read() {
            if let Some(result) = cached.as_ref() {
                return result.hot_keys.iter().take(limit).map(|k| k.key.clone()).collect();
            }
        }
        
        // 否则执行分析
        let result = self.analyze();
        result.hot_keys.iter().take(limit).map(|k| k.key.clone()).collect()
    }
    
    /// 获取分析结果
    pub fn get_analysis_result(&self) -> Option<AnalysisResult> {
        self.analysis_result.read().unwrap().clone()
    }
    
    /// 获取总访问次数
    pub fn get_total_accesses(&self) -> u64 {
        self.total_accesses.load(Ordering::SeqCst)
    }
    
    /// 获取总命中次数
    pub fn get_total_hits(&self) -> u64 {
        self.total_hits.load(Ordering::SeqCst)
    }
    
    /// 获取整体命中率
    pub fn get_hit_rate(&self) -> f64 {
        let total = self.total_accesses.load(Ordering::SeqCst);
        if total == 0 {
            return 0.0;
        }
        self.total_hits.load(Ordering::SeqCst) as f64 / total as f64
    }
    
    /// 清除过期记录
    /// 
    /// # 参数
    /// - `max_age_ms`: 最大存活时间（毫秒）
    pub fn clear_expired(&self, max_age_ms: u64) {
        let now = Self::current_timestamp();
        
        if let Ok(mut records) = self.records.write() {
            records.retain(|_, r| {
                now - r.last_access_time < max_age_ms
            });
        }
    }
    
    /// 重置分析器
    pub fn reset(&self) {
        // 清空记录
        if let Ok(mut records) = self.records.write() {
            records.clear();
        }
        
        // 重置计数器
        self.total_accesses.store(0, Ordering::SeqCst);
        self.total_hits.store(0, Ordering::SeqCst);
        
        // 清空分析结果
        if let Ok(mut result) = self.analysis_result.write() {
            *result = None;
        }
        
        // 重置分析时间
        if let Ok(mut last_time) = self.last_analysis_time.write() {
            *last_time = 0;
        }
    }
    
    /// 更新配置
    pub fn update_config(&self, config: AnalyzerConfig) {
        if let Ok(mut cfg) = self.config.write() {
            *cfg = config;
        }
    }
    
    /// 获取配置
    pub fn get_config(&self) -> AnalyzerConfig {
        self.config.read().unwrap().clone()
    }
    
    /// 获取记录数量
    pub fn get_record_count(&self) -> usize {
        self.records.read().unwrap().len()
    }
}

impl Default for AccessAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试访问记录创建
    #[test]
    fn test_access_record_creation() {
        let record = AccessRecord::new("test_key".to_string());
        
        assert_eq!(record.key, "test_key");
        assert_eq!(record.access_count, 1);
    }
    
    /// 测试访问记录更新
    #[test]
    fn test_access_record_update() {
        let mut record = AccessRecord::new("test_key".to_string());
        
        record.record_access(true);
        record.record_access(false);
        
        assert_eq!(record.access_count, 3);
        assert_eq!(record.hit_count, 1);
        assert_eq!(record.miss_count, 1);
    }
    
    /// 测试热度分数计算
    #[test]
    fn test_hot_score_calculation() {
        let mut record = AccessRecord::new("test_key".to_string());
        
        record.record_access(true);
        record.record_access(true);
        record.record_access(true);
        
        record.calculate_hot_score(0.01);
        
        assert!(record.hot_score > 0.0);
    }
    
    /// 测试访问分析器
    #[test]
    fn test_analyzer() {
        let analyzer = AccessAnalyzer::new();
        
        // 记录一些访问
        analyzer.record_access("key1", true, 100);
        analyzer.record_access("key1", true, 100);
        analyzer.record_access("key2", false, 200);
        analyzer.record_access("key3", true, 300);
        
        // 执行分析
        let result = analyzer.analyze();
        
        assert_eq!(result.total_accesses, 4);
        assert_eq!(result.total_hits, 3);
    }
    
    /// 测试热点键获取
    #[test]
    fn test_get_hot_keys() {
        let analyzer = AccessAnalyzer::new();
        
        // 记录多次访问
        for _ in 0..20 {
            analyzer.record_access("hot_key", true, 100);
        }
        analyzer.record_access("cold_key", true, 100);
        
        // 获取热点键
        let hot_keys = analyzer.get_hot_keys(10);
        
        assert!(!hot_keys.is_empty());
    }
}
