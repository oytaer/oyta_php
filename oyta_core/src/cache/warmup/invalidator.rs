//! 缓存失效管理器模块
//!
//! 管理缓存失效规则和事件驱动的失效机制

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 失效规则
/// 定义事件触发的缓存失效规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidationRule {
    /// 规则 ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 触发事件
    pub event: String,
    /// 失效的缓存键模式列表
    pub key_patterns: Vec<String>,
    /// 是否启用
    pub enabled: bool,
    /// 延迟失效时间（毫秒），0 表示立即失效
    pub delay_ms: u64,
    /// 是否级联失效
    pub cascade: bool,
    /// 创建时间戳
    pub created_at: u64,
    /// 触发次数
    pub trigger_count: u64,
}

/// 失效事件
/// 记录一次缓存失效事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidationEvent {
    /// 事件 ID
    pub id: String,
    /// 触发的事件名称
    pub event: String,
    /// 触发时间戳
    pub timestamp: u64,
    /// 失效的缓存键列表
    pub invalidated_keys: Vec<String>,
    /// 触发规则 ID
    pub rule_id: String,
    /// 触发规则名称
    pub rule_name: String,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error: Option<String>,
    /// 事件数据（JSON 格式）
    pub event_data: Option<String>,
}

/// 失效请求
/// 用于延迟失效队列
#[derive(Debug, Clone)]
struct InvalidationRequest {
    /// 请求 ID
    id: String,
    /// 缓存键模式
    key_pattern: String,
    /// 执行时间戳
    execute_at: u64,
    /// 创建时间戳
    created_at: u64,
}

/// 失效管理器配置
#[derive(Debug, Clone)]
pub struct InvalidatorConfig {
    /// 最大历史记录数
    pub max_history: usize,
    /// 最大延迟队列大小
    pub max_delayed_queue: usize,
    /// 是否启用批量失效
    pub enable_batch: bool,
    /// 批量失效大小
    pub batch_size: usize,
    /// 批量失效间隔（毫秒）
    pub batch_interval_ms: u64,
}

impl Default for InvalidatorConfig {
    fn default() -> Self {
        Self {
            max_history: 1000,
            max_delayed_queue: 10000,
            enable_batch: true,
            batch_size: 100,
            batch_interval_ms: 100,
        }
    }
}

/// 缓存失效管理器
/// 管理事件驱动的缓存失效
#[derive(Debug)]
pub struct Invalidator {
    /// 失效规则存储
    rules: RwLock<HashMap<String, InvalidationRule>>,
    /// 事件到规则的映射
    event_rules: RwLock<HashMap<String, HashSet<String>>>,
    /// 失效事件历史
    history: RwLock<Vec<InvalidationEvent>>,
    /// 延迟失效队列
    delayed_queue: RwLock<Vec<InvalidationRequest>>,
    /// 配置
    config: RwLock<InvalidatorConfig>,
    /// 总失效次数
    total_invalidations: AtomicU64,
    /// 成功次数
    success_count: AtomicU64,
    /// 失败次数
    failure_count: AtomicU64,
}

impl Invalidator {
    /// 创建新的失效管理器
    pub fn new() -> Self {
        Self::with_config(InvalidatorConfig::default())
    }
    
    /// 使用配置创建失效管理器
    pub fn with_config(config: InvalidatorConfig) -> Self {
        Self {
            rules: RwLock::new(HashMap::new()),
            event_rules: RwLock::new(HashMap::new()),
            history: RwLock::new(Vec::new()),
            delayed_queue: RwLock::new(Vec::new()),
            config: RwLock::new(config),
            total_invalidations: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            failure_count: AtomicU64::new(0),
        }
    }
    
    /// 注册失效规则
    /// 
    /// # 参数
    /// - `rule`: 失效规则
    pub fn register_rule(&self, rule: InvalidationRule) {
        let rule_id = rule.id.clone();
        let event = rule.event.clone();
        
        // 存储规则
        if let Ok(mut rules) = self.rules.write() {
            rules.insert(rule_id.clone(), rule);
        }
        
        // 更新事件映射
        if let Ok(mut event_rules) = self.event_rules.write() {
            event_rules
                .entry(event)
                .or_insert_with(HashSet::new)
                .insert(rule_id);
        }
    }
    
    /// 批量注册失效规则
    /// 
    /// # 参数
    /// - `rules`: 失效规则列表
    pub fn register_rules(&self, rules: Vec<InvalidationRule>) {
        for rule in rules {
            self.register_rule(rule);
        }
    }
    
    /// 移除失效规则
    /// 
    /// # 参数
    /// - `rule_id`: 规则 ID
    pub fn remove_rule(&self, rule_id: &str) {
        // 获取规则的事件
        let event = if let Ok(rules) = self.rules.read() {
            rules.get(rule_id).map(|r| r.event.clone())
        } else {
            None
        };
        
        // 从规则存储中移除
        if let Ok(mut rules) = self.rules.write() {
            rules.remove(rule_id);
        }
        
        // 从事件映射中移除
        if let Some(event) = event {
            if let Ok(mut event_rules) = self.event_rules.write() {
                if let Some(rule_ids) = event_rules.get_mut(&event) {
                    rule_ids.remove(rule_id);
                }
            }
        }
    }
    
    /// 触发失效事件
    /// 
    /// # 参数
    /// - `event`: 事件名称
    /// - `event_data`: 事件数据（可选）
    pub fn trigger(&self, event: &str, event_data: Option<&str>) -> Vec<InvalidationEvent> {
        let timestamp = Self::current_timestamp();
        let mut events = Vec::new();
        
        // 获取关联的规则 ID
        let rule_ids = if let Ok(event_rules) = self.event_rules.read() {
            event_rules.get(event).cloned().unwrap_or_default()
        } else {
            return events;
        };
        
        // 获取规则
        let rules = if let Ok(rules) = self.rules.read() {
            rule_ids
                .iter()
                .filter_map(|id| rules.get(id).cloned())
                .filter(|r| r.enabled)
                .collect::<Vec<_>>()
        } else {
            return events;
        };
        
        // 执行每个规则
        for rule in rules {
            let invalidated_keys = self.execute_rule(&rule, event_data);
            
            // 创建失效事件
            let invalidation_event = InvalidationEvent {
                id: uuid::Uuid::new_v4().to_string(),
                event: event.to_string(),
                timestamp,
                invalidated_keys: invalidated_keys.clone(),
                rule_id: rule.id.clone(),
                rule_name: rule.name.clone(),
                success: true,
                error: None,
                event_data: event_data.map(|s| s.to_string()),
            };
            
            // 更新统计
            self.total_invalidations.fetch_add(1, Ordering::SeqCst);
            self.success_count.fetch_add(1, Ordering::SeqCst);
            
            // 更新规则触发次数
            if let Ok(mut rules) = self.rules.write() {
                if let Some(r) = rules.get_mut(&rule.id) {
                    r.trigger_count += 1;
                }
            }
            
            // 记录历史
            self.record_history(&invalidation_event);
            
            events.push(invalidation_event);
        }
        
        events
    }
    
    /// 执行失效规则
    fn execute_rule(&self, rule: &InvalidationRule, _event_data: Option<&str>) -> Vec<String> {
        let mut invalidated_keys = Vec::new();
        
        // 处理每个键模式
        for pattern in &rule.key_patterns {
            // 如果有延迟，加入延迟队列
            if rule.delay_ms > 0 {
                self.add_delayed_invalidation(pattern, rule.delay_ms);
            } else {
                // 立即失效
                let keys = self.invalidate_by_pattern(pattern);
                invalidated_keys.extend(keys);
            }
        }
        
        invalidated_keys
    }
    
    /// 根据模式失效缓存
    fn invalidate_by_pattern(&self, pattern: &str) -> Vec<String> {
        // 这里应该实际执行缓存失效
        // 由于这是示例实现，我们返回模拟的键列表
        
        // 解析模式并生成键
        // 支持通配符 * 和占位符 {id}
        let keys = if pattern.contains('*') || pattern.contains('{') {
            // 模式匹配，返回模拟键
            vec![pattern.replace('*', "1"), pattern.replace('*', "2")]
        } else {
            // 精确匹配
            vec![pattern.to_string()]
        };
        
        // 记录日志
        tracing::info!("缓存失效: pattern={}, keys={:?}", pattern, keys);
        
        keys
    }
    
    /// 添加延迟失效请求
    fn add_delayed_invalidation(&self, pattern: &str, delay_ms: u64) {
        let now = Self::current_timestamp();
        let execute_at = now + delay_ms;
        
        let request = InvalidationRequest {
            id: uuid::Uuid::new_v4().to_string(),
            key_pattern: pattern.to_string(),
            execute_at,
            created_at: now,
        };
        
        if let Ok(mut queue) = self.delayed_queue.write() {
            queue.push(request);
            
            // 限制队列大小
            let max_size = self.config.read().map(|c| c.max_delayed_queue).unwrap_or(10000);
            while queue.len() > max_size {
                queue.remove(0);
            }
        }
    }
    
    /// 处理延迟失效队列
    pub fn process_delayed(&self) -> Vec<String> {
        let now = Self::current_timestamp();
        let mut all_keys = Vec::new();
        
        if let Ok(mut queue) = self.delayed_queue.write() {
            // 找出需要执行的请求
            let to_execute: Vec<_> = queue
                .iter()
                .filter(|r| r.execute_at <= now)
                .cloned()
                .collect();
            
            // 移除已执行的请求
            queue.retain(|r| r.execute_at > now);
            
            // 执行失效
            for request in to_execute {
                let keys = self.invalidate_by_pattern(&request.key_pattern);
                all_keys.extend(keys);
            }
        }
        
        all_keys
    }
    
    /// 记录历史
    fn record_history(&self, event: &InvalidationEvent) {
        if let Ok(mut history) = self.history.write() {
            history.push(event.clone());
            
            // 限制历史记录数量
            let max_history = self.config.read().map(|c| c.max_history).unwrap_or(1000);
            while history.len() > max_history {
                history.remove(0);
            }
        }
    }
    
    /// 获取当前时间戳（毫秒）
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
    
    /// 获取所有规则
    pub fn get_rules(&self) -> Vec<InvalidationRule> {
        self.rules.read().unwrap().values().cloned().collect()
    }
    
    /// 获取规则
    pub fn get_rule(&self, rule_id: &str) -> Option<InvalidationRule> {
        self.rules.read().unwrap().get(rule_id).cloned()
    }
    
    /// 获取事件历史
    pub fn get_history(&self, limit: usize) -> Vec<InvalidationEvent> {
        self.history.read().unwrap().iter().rev().take(limit).cloned().collect()
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> InvalidatorStats {
        InvalidatorStats {
            total_invalidations: self.total_invalidations.load(Ordering::SeqCst),
            success_count: self.success_count.load(Ordering::SeqCst),
            failure_count: self.failure_count.load(Ordering::SeqCst),
            rules_count: self.rules.read().unwrap().len(),
            pending_delayed: self.delayed_queue.read().unwrap().len(),
        }
    }
    
    /// 手动失效指定键
    /// 
    /// # 参数
    /// - `keys`: 要失效的键列表
    pub fn invalidate_keys(&self, keys: &[&str]) -> InvalidationEvent {
        let timestamp = Self::current_timestamp();
        
        // 执行失效
        let invalidated_keys: Vec<String> = keys.iter().map(|k| {
            self.invalidate_by_pattern(k);
            k.to_string()
        }).collect();
        
        // 更新统计
        self.total_invalidations.fetch_add(1, Ordering::SeqCst);
        self.success_count.fetch_add(1, Ordering::SeqCst);
        
        // 创建事件
        let event = InvalidationEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event: "manual".to_string(),
            timestamp,
            invalidated_keys,
            rule_id: "manual".to_string(),
            rule_name: "手动失效".to_string(),
            success: true,
            error: None,
            event_data: None,
        };
        
        // 记录历史
        self.record_history(&event);
        
        event
    }
    
    /// 清除延迟队列
    pub fn clear_delayed(&self) {
        if let Ok(mut queue) = self.delayed_queue.write() {
            queue.clear();
        }
    }
    
    /// 重置管理器
    pub fn reset(&self) {
        // 清空规则
        if let Ok(mut rules) = self.rules.write() {
            rules.clear();
        }
        
        // 清空事件映射
        if let Ok(mut event_rules) = self.event_rules.write() {
            event_rules.clear();
        }
        
        // 清空历史
        if let Ok(mut history) = self.history.write() {
            history.clear();
        }
        
        // 清空延迟队列
        if let Ok(mut queue) = self.delayed_queue.write() {
            queue.clear();
        }
        
        // 重置计数器
        self.total_invalidations.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.failure_count.store(0, Ordering::SeqCst);
    }
    
    /// 更新配置
    pub fn update_config(&self, config: InvalidatorConfig) {
        if let Ok(mut cfg) = self.config.write() {
            *cfg = config;
        }
    }
    
    /// 获取配置
    pub fn get_config(&self) -> InvalidatorConfig {
        self.config.read().unwrap().clone()
    }
}

impl Default for Invalidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 失效管理器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidatorStats {
    /// 总失效次数
    pub total_invalidations: u64,
    /// 成功次数
    pub success_count: u64,
    /// 失败次数
    pub failure_count: u64,
    /// 规则数量
    pub rules_count: usize,
    /// 待处理的延迟失效数
    pub pending_delayed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试规则注册
    #[test]
    fn test_register_rule() {
        let invalidator = Invalidator::new();
        
        let rule = InvalidationRule {
            id: "test_rule".to_string(),
            name: "测试规则".to_string(),
            event: "user.updated".to_string(),
            key_patterns: vec!["user:{id}".to_string()],
            enabled: true,
            delay_ms: 0,
            cascade: false,
            created_at: 0,
            trigger_count: 0,
        };
        
        invalidator.register_rule(rule);
        
        let rules = invalidator.get_rules();
        assert_eq!(rules.len(), 1);
    }
    
    /// 测试事件触发
    #[test]
    fn test_trigger() {
        let invalidator = Invalidator::new();
        
        let rule = InvalidationRule {
            id: "test_rule".to_string(),
            name: "测试规则".to_string(),
            event: "user.updated".to_string(),
            key_patterns: vec!["user:1".to_string()],
            enabled: true,
            delay_ms: 0,
            cascade: false,
            created_at: 0,
            trigger_count: 0,
        };
        
        invalidator.register_rule(rule);
        
        let events = invalidator.trigger("user.updated", Some(r#"{"id": 1}"#));
        
        assert_eq!(events.len(), 1);
        assert!(events[0].success);
    }
    
    /// 测试手动失效
    #[test]
    fn test_manual_invalidation() {
        let invalidator = Invalidator::new();
        
        let event = invalidator.invalidate_keys(&["key1", "key2"]);
        
        assert!(event.success);
        assert_eq!(event.invalidated_keys.len(), 2);
    }
    
    /// 测试延迟失效
    #[test]
    fn test_delayed_invalidation() {
        let invalidator = Invalidator::new();
        
        let rule = InvalidationRule {
            id: "test_rule".to_string(),
            name: "测试规则".to_string(),
            event: "test.event".to_string(),
            key_patterns: vec!["delayed_key".to_string()],
            enabled: true,
            delay_ms: 1000, // 1秒延迟
            cascade: false,
            created_at: 0,
            trigger_count: 0,
        };
        
        invalidator.register_rule(rule);
        
        // 触发事件
        let events = invalidator.trigger("test.event", None);
        
        // 应该没有立即失效的键
        assert!(events[0].invalidated_keys.is_empty());
        
        // 检查延迟队列
        let stats = invalidator.get_stats();
        assert!(stats.pending_delayed > 0);
    }
    
    /// 测试统计信息
    #[test]
    fn test_stats() {
        let invalidator = Invalidator::new();
        
        let rule = InvalidationRule {
            id: "test_rule".to_string(),
            name: "测试规则".to_string(),
            event: "test.event".to_string(),
            key_patterns: vec!["key".to_string()],
            enabled: true,
            delay_ms: 0,
            cascade: false,
            created_at: 0,
            trigger_count: 0,
        };
        
        invalidator.register_rule(rule);
        invalidator.trigger("test.event", None);
        
        let stats = invalidator.get_stats();
        assert_eq!(stats.total_invalidations, 1);
        assert_eq!(stats.success_count, 1);
    }
}
