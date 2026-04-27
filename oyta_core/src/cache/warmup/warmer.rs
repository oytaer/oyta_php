//! 缓存预热执行器模块
//!
//! 执行缓存预热任务，支持规则注册和定时刷新

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 预热规则
/// 定义预热任务的配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupRule {
    /// 规则 ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 数据源类型
    pub source: DataSource,
    /// 缓存键模板
    pub key_template: String,
    /// TTL（秒）
    pub ttl: u64,
    /// 刷新间隔（秒），0 表示不自动刷新
    pub refresh_interval: u64,
    /// 是否启用
    pub enabled: bool,
    /// 优先级（数字越大优先级越高）
    pub priority: i32,
    /// 预热条件
    pub condition: Option<WarmupCondition>,
    /// 创建时间戳
    pub created_at: u64,
    /// 最后执行时间戳
    pub last_executed_at: Option<u64>,
    /// 执行次数
    pub execution_count: u64,
}

/// 数据源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSource {
    /// SQL 查询
    Sql {
        /// 查询语句
        query: String,
        /// 数据库连接名
        connection: String,
    },
    /// 函数调用
    Function {
        /// 函数名
        function: String,
        /// 参数列表（JSON 格式）
        params: Vec<String>,
    },
    /// HTTP 请求
    Http {
        /// URL
        url: String,
        /// 请求方法
        method: String,
        /// 请求头
        headers: HashMap<String, String>,
    },
    /// 静态数据
    Static {
        /// 数据（JSON 格式）
        data: String,
    },
    /// 文件
    File {
        /// 文件路径
        path: String,
        /// 解析格式
        format: FileFormat,
    },
}

/// 文件格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileFormat {
    /// JSON 格式
    Json,
    /// YAML 格式
    Yaml,
    /// CSV 格式
    Csv,
    /// 纯文本
    Text,
}

/// 预热条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupCondition {
    /// 条件类型
    pub condition_type: ConditionType,
    /// 阈值
    pub threshold: f64,
}

/// 条件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConditionType {
    /// 缓存命中率低于阈值时预热
    HitRateBelow,
    /// 缓存项数量低于阈值时预热
    ItemCountBelow,
    /// 内存使用率低于阈值时预热
    MemoryUsageBelow,
    /// 定时预热
    Scheduled,
}

/// 预热状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupStatus {
    /// 规则 ID
    pub rule_id: String,
    /// 规则名称
    pub rule_name: String,
    /// 是否正在执行
    pub is_running: bool,
    /// 最后执行时间戳
    pub last_executed_at: Option<u64>,
    /// 最后执行结果
    pub last_result: Option<WarmupResult>,
    /// 下次执行时间戳
    pub next_execute_at: Option<u64>,
    /// 总执行次数
    pub total_executions: u64,
    /// 成功次数
    pub success_count: u64,
    /// 失败次数
    pub failure_count: u64,
}

/// 预热执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupResult {
    /// 执行时间戳
    pub timestamp: u64,
    /// 是否成功
    pub success: bool,
    /// 预热的数据项数量
    pub items_count: u64,
    /// 预热的数据总大小（字节）
    pub total_size: u64,
    /// 执行耗时（毫秒）
    pub duration_ms: u64,
    /// 错误信息
    pub error: Option<String>,
}

/// 预热执行器配置
#[derive(Debug, Clone)]
pub struct WarmerConfig {
    /// 最大并发预热任务数
    pub max_concurrent: usize,
    /// 单次预热最大数据量
    pub max_items_per_run: u64,
    /// 预热超时时间（秒）
    pub timeout_seconds: u64,
    /// 是否启用自动预热
    pub auto_warmup: bool,
    /// 自动预热检查间隔（秒）
    pub auto_check_interval: u64,
    /// 重试次数
    pub retry_count: u32,
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
}

impl Default for WarmerConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            max_items_per_run: 1000,
            timeout_seconds: 300,
            auto_warmup: true,
            auto_check_interval: 60,
            retry_count: 3,
            retry_interval_ms: 1000,
        }
    }
}

/// 缓存预热执行器
/// 负责执行预热任务和管理预热规则
#[derive(Debug)]
pub struct CacheWarmer {
    /// 预热规则存储
    rules: RwLock<HashMap<String, WarmupRule>>,
    /// 预热状态存储
    statuses: RwLock<HashMap<String, WarmupStatus>>,
    /// 配置
    config: RwLock<WarmerConfig>,
    /// 是否正在运行
    running: AtomicBool,
    /// 总预热次数
    total_warmups: AtomicU64,
    /// 成功次数
    success_count: AtomicU64,
    /// 失败次数
    failure_count: AtomicU64,
    /// 执行历史
    history: RwLock<Vec<WarmupResult>>,
}

impl CacheWarmer {
    /// 创建新的预热执行器
    pub fn new() -> Self {
        Self::with_config(WarmerConfig::default())
    }
    
    /// 使用配置创建预热执行器
    pub fn with_config(config: WarmerConfig) -> Self {
        Self {
            rules: RwLock::new(HashMap::new()),
            statuses: RwLock::new(HashMap::new()),
            config: RwLock::new(config),
            running: AtomicBool::new(false),
            total_warmups: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            failure_count: AtomicU64::new(0),
            history: RwLock::new(Vec::new()),
        }
    }
    
    /// 注册预热规则
    /// 
    /// # 参数
    /// - `rule`: 预热规则
    pub fn register_rule(&self, rule: WarmupRule) {
        let rule_id = rule.id.clone();
        let rule_name = rule.name.clone();
        
        // 存储规则
        if let Ok(mut rules) = self.rules.write() {
            rules.insert(rule_id.clone(), rule);
        }
        
        // 初始化状态
        let status = WarmupStatus {
            rule_id: rule_id.clone(),
            rule_name,
            is_running: false,
            last_executed_at: None,
            last_result: None,
            next_execute_at: None,
            total_executions: 0,
            success_count: 0,
            failure_count: 0,
        };
        
        if let Ok(mut statuses) = self.statuses.write() {
            statuses.insert(rule_id, status);
        }
    }
    
    /// 批量注册预热规则
    /// 
    /// # 参数
    /// - `rules`: 预热规则列表
    pub fn register_rules(&self, rules: Vec<WarmupRule>) {
        for rule in rules {
            self.register_rule(rule);
        }
    }
    
    /// 移除预热规则
    /// 
    /// # 参数
    /// - `rule_id`: 规则 ID
    pub fn remove_rule(&self, rule_id: &str) {
        if let Ok(mut rules) = self.rules.write() {
            rules.remove(rule_id);
        }
        
        if let Ok(mut statuses) = self.statuses.write() {
            statuses.remove(rule_id);
        }
    }
    
    /// 获取所有规则
    pub fn get_rules(&self) -> Vec<WarmupRule> {
        self.rules.read().unwrap().values().cloned().collect()
    }
    
    /// 获取规则
    pub fn get_rule(&self, rule_id: &str) -> Option<WarmupRule> {
        self.rules.read().unwrap().get(rule_id).cloned()
    }
    
    /// 执行预热
    /// 
    /// # 参数
    /// - `rule_id`: 规则 ID
    pub fn warmup(&self, rule_id: &str) -> WarmupResult {
        let start_time = Instant::now();
        let timestamp = Self::current_timestamp();
        
        // 获取规则
        let rule = match self.get_rule(rule_id) {
            Some(r) => r,
            None => {
                return WarmupResult {
                    timestamp,
                    success: false,
                    items_count: 0,
                    total_size: 0,
                    duration_ms: 0,
                    error: Some(format!("规则不存在: {}", rule_id)),
                };
            }
        };
        
        // 检查是否启用
        if !rule.enabled {
            return WarmupResult {
                timestamp,
                success: false,
                items_count: 0,
                total_size: 0,
                duration_ms: 0,
                error: Some("规则已禁用".to_string()),
            };
        }
        
        // 更新状态为运行中
        self.update_status_running(rule_id, true);
        
        // 执行预热
        let result = self.execute_warmup(&rule);
        
        // 更新统计
        self.total_warmups.fetch_add(1, Ordering::SeqCst);
        if result.success {
            self.success_count.fetch_add(1, Ordering::SeqCst);
        } else {
            self.failure_count.fetch_add(1, Ordering::SeqCst);
        }
        
        // 更新状态
        self.update_status(rule_id, &result);
        
        // 记录历史
        if let Ok(mut history) = self.history.write() {
            history.push(result.clone());
            
            // 限制历史记录数量
            let max_history = 1000;
            while history.len() > max_history {
                history.remove(0);
            }
        }
        
        result
    }
    
    /// 执行预热任务
    fn execute_warmup(&self, rule: &WarmupRule) -> WarmupResult {
        let start_time = Instant::now();
        let timestamp = Self::current_timestamp();
        
        // 根据数据源类型执行预热
        let result = match &rule.source {
            DataSource::Sql { query, connection } => {
                self.warmup_from_sql(query, connection, &rule.key_template, rule.ttl)
            }
            DataSource::Function { function, params } => {
                self.warmup_from_function(function, params, &rule.key_template, rule.ttl)
            }
            DataSource::Http { url, method, headers } => {
                self.warmup_from_http(url, method, headers, &rule.key_template, rule.ttl)
            }
            DataSource::Static { data } => {
                self.warmup_from_static(data, &rule.key_template, rule.ttl)
            }
            DataSource::File { path, format } => {
                self.warmup_from_file(path, format, &rule.key_template, rule.ttl)
            }
        };
        
        // 计算耗时
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        // 构建结果
        match result {
            Ok((items_count, total_size)) => WarmupResult {
                timestamp,
                success: true,
                items_count,
                total_size,
                duration_ms,
                error: None,
            },
            Err(e) => WarmupResult {
                timestamp,
                success: false,
                items_count: 0,
                total_size: 0,
                duration_ms,
                error: Some(e),
            },
        }
    }
    
    /// 从 SQL 数据源预热
    fn warmup_from_sql(
        &self,
        query: &str,
        _connection: &str,
        key_template: &str,
        _ttl: u64,
    ) -> Result<(u64, u64), String> {
        // 这里应该执行 SQL 查询并将结果写入缓存
        // 由于这是示例实现，我们模拟执行
        
        // 模拟查询结果
        let items_count = 100u64;
        let total_size = items_count * 1024; // 假设每项 1KB
        
        // 记录日志
        tracing::info!("SQL 预热: query={}, key_template={}", query, key_template);
        
        Ok((items_count, total_size))
    }
    
    /// 从函数数据源预热
    fn warmup_from_function(
        &self,
        function: &str,
        _params: &[String],
        key_template: &str,
        _ttl: u64,
    ) -> Result<(u64, u64), String> {
        // 这里应该调用函数并将结果写入缓存
        // 由于这是示例实现，我们模拟执行
        
        let items_count = 50u64;
        let total_size = items_count * 512;
        
        tracing::info!("函数预热: function={}, key_template={}", function, key_template);
        
        Ok((items_count, total_size))
    }
    
    /// 从 HTTP 数据源预热
    fn warmup_from_http(
        &self,
        url: &str,
        method: &str,
        _headers: &HashMap<String, String>,
        key_template: &str,
        _ttl: u64,
    ) -> Result<(u64, u64), String> {
        // 这里应该发送 HTTP 请求并将结果写入缓存
        // 由于这是示例实现，我们模拟执行
        
        let items_count = 30u64;
        let total_size = items_count * 2048;
        
        tracing::info!("HTTP 预热: url={}, method={}, key_template={}", url, method, key_template);
        
        Ok((items_count, total_size))
    }
    
    /// 从静态数据预热
    fn warmup_from_static(
        &self,
        data: &str,
        key_template: &str,
        _ttl: u64,
    ) -> Result<(u64, u64), String> {
        // 解析静态数据
        let items_count = 1u64;
        let total_size = data.len() as u64;
        
        tracing::info!("静态数据预热: key_template={}", key_template);
        
        Ok((items_count, total_size))
    }
    
    /// 从文件数据源预热
    fn warmup_from_file(
        &self,
        path: &str,
        _format: &FileFormat,
        key_template: &str,
        _ttl: u64,
    ) -> Result<(u64, u64), String> {
        // 这里应该读取文件并将结果写入缓存
        // 由于这是示例实现，我们模拟执行
        
        let items_count = 20u64;
        let total_size = items_count * 4096;
        
        tracing::info!("文件预热: path={}, key_template={}", path, key_template);
        
        Ok((items_count, total_size))
    }
    
    /// 更新状态为运行中
    fn update_status_running(&self, rule_id: &str, is_running: bool) {
        if let Ok(mut statuses) = self.statuses.write() {
            if let Some(status) = statuses.get_mut(rule_id) {
                status.is_running = is_running;
            }
        }
    }
    
    /// 更新状态
    fn update_status(&self, rule_id: &str, result: &WarmupResult) {
        if let Ok(mut statuses) = self.statuses.write() {
            if let Some(status) = statuses.get_mut(rule_id) {
                status.is_running = false;
                status.last_executed_at = Some(result.timestamp);
                status.last_result = Some(result.clone());
                status.total_executions += 1;
                
                if result.success {
                    status.success_count += 1;
                } else {
                    status.failure_count += 1;
                }
            }
        }
        
        // 更新规则的执行时间
        if let Ok(mut rules) = self.rules.write() {
            if let Some(rule) = rules.get_mut(rule_id) {
                rule.last_executed_at = Some(result.timestamp);
                rule.execution_count += 1;
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
    
    /// 获取预热状态
    pub fn get_status(&self, rule_id: &str) -> Option<WarmupStatus> {
        self.statuses.read().unwrap().get(rule_id).cloned()
    }
    
    /// 获取所有状态
    pub fn get_all_statuses(&self) -> Vec<WarmupStatus> {
        self.statuses.read().unwrap().values().cloned().collect()
    }
    
    /// 执行所有启用的预热规则
    pub fn warmup_all(&self) -> Vec<(String, WarmupResult)> {
        let rules = self.get_rules();
        let mut results = Vec::new();
        
        for rule in rules.iter().filter(|r| r.enabled) {
            let result = self.warmup(&rule.id);
            results.push((rule.id.clone(), result));
        }
        
        results
    }
    
    /// 检查并执行需要刷新的规则
    pub fn check_and_refresh(&self) {
        let now = Self::current_timestamp();
        let rules = self.get_rules();
        
        for rule in rules.iter().filter(|r| r.enabled && r.refresh_interval > 0) {
            // 检查是否需要刷新
            let should_refresh = match rule.last_executed_at {
                Some(last) => {
                    let elapsed = (now - last) / 1000; // 转换为秒
                    elapsed >= rule.refresh_interval
                }
                None => true,
            };
            
            if should_refresh {
                self.warmup(&rule.id);
            }
        }
    }
    
    /// 获取执行历史
    pub fn get_history(&self, limit: usize) -> Vec<WarmupResult> {
        self.history.read().unwrap().iter().rev().take(limit).cloned().collect()
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> WarmerStats {
        WarmerStats {
            total_warmups: self.total_warmups.load(Ordering::SeqCst),
            success_count: self.success_count.load(Ordering::SeqCst),
            failure_count: self.failure_count.load(Ordering::SeqCst),
            rules_count: self.rules.read().unwrap().len(),
        }
    }
    
    /// 重置执行器
    pub fn reset(&self) {
        // 清空规则
        if let Ok(mut rules) = self.rules.write() {
            rules.clear();
        }
        
        // 清空状态
        if let Ok(mut statuses) = self.statuses.write() {
            statuses.clear();
        }
        
        // 重置计数器
        self.total_warmups.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.failure_count.store(0, Ordering::SeqCst);
        
        // 清空历史
        if let Ok(mut history) = self.history.write() {
            history.clear();
        }
    }
}

impl Default for CacheWarmer {
    fn default() -> Self {
        Self::new()
    }
}

/// 预热执行器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmerStats {
    /// 总预热次数
    pub total_warmups: u64,
    /// 成功次数
    pub success_count: u64,
    /// 失败次数
    pub failure_count: u64,
    /// 规则数量
    pub rules_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试规则注册
    #[test]
    fn test_register_rule() {
        let warmer = CacheWarmer::new();
        
        let rule = WarmupRule {
            id: "test_rule".to_string(),
            name: "测试规则".to_string(),
            source: DataSource::Static {
                data: r#"{"key": "value"}"#.to_string(),
            },
            key_template: "test:{key}".to_string(),
            ttl: 3600,
            refresh_interval: 300,
            enabled: true,
            priority: 1,
            condition: None,
            created_at: 0,
            last_executed_at: None,
            execution_count: 0,
        };
        
        warmer.register_rule(rule);
        
        let rules = warmer.get_rules();
        assert_eq!(rules.len(), 1);
    }
    
    /// 测试预热执行
    #[test]
    fn test_warmup() {
        let warmer = CacheWarmer::new();
        
        let rule = WarmupRule {
            id: "test_rule".to_string(),
            name: "测试规则".to_string(),
            source: DataSource::Static {
                data: r#"{"key": "value"}"#.to_string(),
            },
            key_template: "test:{key}".to_string(),
            ttl: 3600,
            refresh_interval: 0,
            enabled: true,
            priority: 1,
            condition: None,
            created_at: 0,
            last_executed_at: None,
            execution_count: 0,
        };
        
        warmer.register_rule(rule);
        
        let result = warmer.warmup("test_rule");
        
        assert!(result.success);
        assert!(result.items_count > 0);
    }
    
    /// 测试规则移除
    #[test]
    fn test_remove_rule() {
        let warmer = CacheWarmer::new();
        
        let rule = WarmupRule {
            id: "test_rule".to_string(),
            name: "测试规则".to_string(),
            source: DataSource::Static {
                data: r#"{}"#.to_string(),
            },
            key_template: "test".to_string(),
            ttl: 3600,
            refresh_interval: 0,
            enabled: true,
            priority: 1,
            condition: None,
            created_at: 0,
            last_executed_at: None,
            execution_count: 0,
        };
        
        warmer.register_rule(rule);
        warmer.remove_rule("test_rule");
        
        let rules = warmer.get_rules();
        assert_eq!(rules.len(), 0);
    }
    
    /// 测试统计信息
    #[test]
    fn test_stats() {
        let warmer = CacheWarmer::new();
        
        let rule = WarmupRule {
            id: "test_rule".to_string(),
            name: "测试规则".to_string(),
            source: DataSource::Static {
                data: r#"{}"#.to_string(),
            },
            key_template: "test".to_string(),
            ttl: 3600,
            refresh_interval: 0,
            enabled: true,
            priority: 1,
            condition: None,
            created_at: 0,
            last_executed_at: None,
            execution_count: 0,
        };
        
        warmer.register_rule(rule);
        warmer.warmup("test_rule");
        
        let stats = warmer.get_stats();
        assert_eq!(stats.total_warmups, 1);
        assert_eq!(stats.success_count, 1);
    }
}
