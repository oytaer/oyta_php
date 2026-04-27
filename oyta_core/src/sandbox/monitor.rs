//! 沙箱监控模块
//!
//! 提供沙箱的资源监控和违规事件记录

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

/// 违规类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    /// 内存超限
    MemoryLimit,
    /// 时间超限
    TimeLimit,
    /// CPU 超限
    CpuLimit,
    /// 非法函数调用
    FunctionCall,
    /// 非法文件访问
    FileAccess,
    /// 非法网络访问
    NetworkAccess,
    /// 输出大小超限
    OutputLimit,
    /// 其他违规
    Other,
}

/// 违规事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationEvent {
    /// 违规类型
    pub violation_type: ViolationType,
    /// 违规描述
    pub description: String,
    /// 时间戳（毫秒）
    pub timestamp: u64,
}

impl ViolationEvent {
    /// 创建新的违规事件
    pub fn new(violation_type: ViolationType, description: String) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Self {
            violation_type,
            description,
            timestamp,
        }
    }
}

/// 监控统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MonitorStats {
    /// 总执行次数
    pub total_executions: u64,
    /// 成功次数
    pub success_count: u64,
    /// 失败次数
    pub failure_count: u64,
    /// 违规次数
    pub violation_count: u64,
    /// 内存超限次数
    pub memory_violations: u64,
    /// 时间超限次数
    pub time_violations: u64,
    /// 函数调用违规次数
    pub function_violations: u64,
    /// 文件访问违规次数
    pub file_violations: u64,
    /// 网络访问违规次数
    pub network_violations: u64,
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: u64,
    /// 最大执行时间（毫秒）
    pub max_execution_time_ms: u64,
    /// 总内存使用（字节）
    pub total_memory_used: u64,
    /// 最大内存使用（字节）
    pub max_memory_used: u64,
}

impl MonitorStats {
    /// 获取成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            return 0.0;
        }
        self.success_count as f64 / self.total_executions as f64
    }
    
    /// 获取平均执行时间
    pub fn avg_execution_time(&self) -> f64 {
        if self.total_executions == 0 {
            return 0.0;
        }
        self.total_execution_time_ms as f64 / self.total_executions as f64
    }
    
    /// 获取平均内存使用
    pub fn avg_memory_used(&self) -> f64 {
        if self.total_executions == 0 {
            return 0.0;
        }
        self.total_memory_used as f64 / self.total_executions as f64
    }
}

/// 沙箱监控器
/// 记录和统计沙箱的运行情况
#[derive(Debug)]
pub struct SandboxMonitor {
    /// 统计信息
    stats: MonitorStats,
    /// 违规事件历史
    violations: RwLock<VecDeque<ViolationEvent>>,
    /// 最大历史记录数
    max_history: usize,
}

impl SandboxMonitor {
    /// 创建新的监控器
    pub fn new() -> Self {
        Self {
            stats: MonitorStats::default(),
            violations: RwLock::new(VecDeque::new()),
            max_history: 1000,
        }
    }
    
    /// 设置最大历史记录数
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }
    
    /// 记录执行开始
    pub fn record_execution_start(&mut self) {
        self.stats.total_executions += 1;
    }
    
    /// 记录执行成功
    pub fn record_success(&mut self, execution_time_ms: u64, memory_used: u64) {
        // 更新成功计数
        self.stats.success_count += 1;
        
        // 更新时间统计
        self.stats.total_execution_time_ms += execution_time_ms;
        if execution_time_ms > self.stats.max_execution_time_ms {
            self.stats.max_execution_time_ms = execution_time_ms;
        }
        
        // 更新内存统计
        self.stats.total_memory_used += memory_used;
        if memory_used > self.stats.max_memory_used {
            self.stats.max_memory_used = memory_used;
        }
    }
    
    /// 记录执行失败
    pub fn record_failure(&mut self) {
        self.stats.failure_count += 1;
    }
    
    /// 记录违规事件
    pub fn record_violation(&self, event: ViolationEvent) {
        // 更新统计
        // 注意：这里使用 unsafe 因为需要在不可变引用中修改统计
        // 实际应该使用原子类型或 RwLock
        
        // 记录违规事件
        if let Ok(mut violations) = self.violations.write() {
            violations.push_back(event.clone());
            
            // 限制历史记录数量
            while violations.len() > self.max_history {
                violations.pop_front();
            }
        }
    }
    
    /// 记录内存违规
    pub fn record_memory_violation(&mut self, description: &str) {
        self.stats.memory_violations += 1;
        self.stats.violation_count += 1;
        
        let event = ViolationEvent::new(ViolationType::MemoryLimit, description.to_string());
        if let Ok(mut violations) = self.violations.write() {
            violations.push_back(event);
            while violations.len() > self.max_history {
                violations.pop_front();
            }
        }
    }
    
    /// 记录时间违规
    pub fn record_time_violation(&mut self, description: &str) {
        self.stats.time_violations += 1;
        self.stats.violation_count += 1;
        
        let event = ViolationEvent::new(ViolationType::TimeLimit, description.to_string());
        if let Ok(mut violations) = self.violations.write() {
            violations.push_back(event);
            while violations.len() > self.max_history {
                violations.pop_front();
            }
        }
    }
    
    /// 记录函数调用违规
    pub fn record_function_violation(&mut self, description: &str) {
        self.stats.function_violations += 1;
        self.stats.violation_count += 1;
        
        let event = ViolationEvent::new(ViolationType::FunctionCall, description.to_string());
        if let Ok(mut violations) = self.violations.write() {
            violations.push_back(event);
            while violations.len() > self.max_history {
                violations.pop_front();
            }
        }
    }
    
    /// 记录文件访问违规
    pub fn record_file_violation(&mut self, description: &str) {
        self.stats.file_violations += 1;
        self.stats.violation_count += 1;
        
        let event = ViolationEvent::new(ViolationType::FileAccess, description.to_string());
        if let Ok(mut violations) = self.violations.write() {
            violations.push_back(event);
            while violations.len() > self.max_history {
                violations.pop_front();
            }
        }
    }
    
    /// 记录网络访问违规
    pub fn record_network_violation(&mut self, description: &str) {
        self.stats.network_violations += 1;
        self.stats.violation_count += 1;
        
        let event = ViolationEvent::new(ViolationType::NetworkAccess, description.to_string());
        if let Ok(mut violations) = self.violations.write() {
            violations.push_back(event);
            while violations.len() > self.max_history {
                violations.pop_front();
            }
        }
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> MonitorStats {
        self.stats.clone()
    }
    
    /// 获取违规事件列表
    pub fn get_violations(&self) -> Vec<ViolationEvent> {
        if let Ok(violations) = self.violations.read() {
            violations.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }
    
    /// 获取最近的违规事件
    /// 
    /// # 参数
    /// - `limit`: 最大返回数量
    pub fn get_recent_violations(&self, limit: usize) -> Vec<ViolationEvent> {
        if let Ok(violations) = self.violations.read() {
            violations
                .iter()
                .rev()
                .take(limit)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// 获取指定类型的违规事件
    /// 
    /// # 参数
    /// - `violation_type`: 违规类型
    pub fn get_violations_by_type(&self, violation_type: ViolationType) -> Vec<ViolationEvent> {
        if let Ok(violations) = self.violations.read() {
            violations
                .iter()
                .filter(|v| v.violation_type == violation_type)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// 清空历史记录
    pub fn clear(&self) {
        if let Ok(mut violations) = self.violations.write() {
            violations.clear();
        }
    }
    
    /// 重置统计信息
    pub fn reset(&mut self) {
        self.stats = MonitorStats::default();
        self.clear();
    }
    
    /// 检查是否有违规
    pub fn has_violations(&self) -> bool {
        if let Ok(violations) = self.violations.read() {
            !violations.is_empty()
        } else {
            false
        }
    }
    
    /// 获取违规数量
    pub fn violation_count(&self) -> usize {
        if let Ok(violations) = self.violations.read() {
            violations.len()
        } else {
            0
        }
    }
}

impl Default for SandboxMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试违规事件创建
    #[test]
    fn test_violation_event() {
        let event = ViolationEvent::new(
            ViolationType::MemoryLimit,
            "内存超限".to_string(),
        );
        
        assert_eq!(event.violation_type, ViolationType::MemoryLimit);
        assert_eq!(event.description, "内存超限");
        assert!(event.timestamp > 0);
    }
    
    /// 测试监控器
    #[test]
    fn test_monitor() {
        let mut monitor = SandboxMonitor::new();
        
        // 记录执行
        monitor.record_execution_start();
        monitor.record_success(100, 1024);
        
        let stats = monitor.get_stats();
        assert_eq!(stats.total_executions, 1);
        assert_eq!(stats.success_count, 1);
    }
    
    /// 测试违规记录
    #[test]
    fn test_violation_recording() {
        let mut monitor = SandboxMonitor::new();
        
        // 记录违规
        monitor.record_memory_violation("内存超限");
        monitor.record_time_violation("执行超时");
        
        let violations = monitor.get_violations();
        assert_eq!(violations.len(), 2);
        
        let stats = monitor.get_stats();
        assert_eq!(stats.violation_count, 2);
        assert_eq!(stats.memory_violations, 1);
        assert_eq!(stats.time_violations, 1);
    }
    
    /// 测试统计计算
    #[test]
    fn test_stats_calculation() {
        let mut stats = MonitorStats::default();
        
        stats.total_executions = 10;
        stats.success_count = 8;
        stats.failure_count = 2;
        stats.total_execution_time_ms = 1000;
        stats.max_execution_time_ms = 200;
        
        assert!((stats.success_rate() - 0.8).abs() < f64::EPSILON);
        assert!((stats.avg_execution_time() - 100.0).abs() < f64::EPSILON);
    }
}
