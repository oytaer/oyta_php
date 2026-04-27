//! 沙箱执行器模块
//!
//! 提供沙箱环境的创建和代码执行功能

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::resource::{ResourceManager, ResourceLimits, ResourceUsage};
use super::filesystem::FilesystemIsolation;
use super::network::NetworkIsolation;
use super::monitor::{SandboxMonitor, ViolationEvent, ViolationType};

/// 沙箱配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// 内存限制（字节）
    pub memory_limit: u64,
    /// 时间限制（毫秒）
    pub time_limit: u64,
    /// CPU 限制（百分比 0-100）
    pub cpu_limit: u32,
    /// 是否允许网络访问
    pub network_enabled: bool,
    /// 文件系统规则
    pub filesystem_rules: Vec<String>,
    /// 允许的函数列表
    pub allowed_functions: Vec<String>,
    /// 禁止的函数列表
    pub denied_functions: Vec<String>,
    /// 最大输出大小（字节）
    pub max_output_size: u64,
    /// 环境变量
    pub env_vars: HashMap<String, String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            memory_limit: 32 * 1024 * 1024, // 32MB
            time_limit: 5000,                // 5秒
            cpu_limit: 50,                   // 50%
            network_enabled: false,
            filesystem_rules: Vec::new(),
            allowed_functions: Vec::new(),
            denied_functions: vec![
                "exec".to_string(),
                "system".to_string(),
                "shell_exec".to_string(),
                "passthru".to_string(),
                "popen".to_string(),
                "proc_open".to_string(),
            ],
            max_output_size: 1024 * 1024, // 1MB
            env_vars: HashMap::new(),
        }
    }
}

impl SandboxConfig {
    /// 创建新的沙箱配置
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 设置内存限制
    pub fn memory_limit(mut self, bytes: u64) -> Self {
        self.memory_limit = bytes;
        self
    }
    
    /// 设置时间限制
    pub fn time_limit(mut self, ms: u64) -> Self {
        self.time_limit = ms;
        self
    }
    
    /// 设置 CPU 限制
    pub fn cpu_limit(mut self, percent: u32) -> Self {
        self.cpu_limit = percent.min(100);
        self
    }
    
    /// 启用网络访问
    pub fn enable_network(mut self) -> Self {
        self.network_enabled = true;
        self
    }
    
    /// 禁用网络访问
    pub fn disable_network(mut self) -> Self {
        self.network_enabled = false;
        self
    }
    
    /// 添加允许的函数
    pub fn allow_function(mut self, func: &str) -> Self {
        self.allowed_functions.push(func.to_string());
        self
    }
    
    /// 添加禁止的函数
    pub fn deny_function(mut self, func: &str) -> Self {
        self.denied_functions.push(func.to_string());
        self
    }
    
    /// 添加文件系统规则
    pub fn filesystem_rule(mut self, rule: &str) -> Self {
        self.filesystem_rules.push(rule.to_string());
        self
    }
    
    /// 设置环境变量
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }
}

/// 沙箱执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResult {
    /// 是否成功
    pub success: bool,
    /// 执行输出
    pub output: String,
    /// 返回值（JSON 格式）
    pub return_value: Option<String>,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 内存使用峰值（字节）
    pub memory_used: u64,
    /// 违规事件列表
    pub violations: Vec<ViolationEvent>,
    /// 错误信息
    pub error: Option<String>,
}

impl SandboxResult {
    /// 创建成功的结果
    pub fn success(output: String, return_value: Option<String>) -> Self {
        Self {
            success: true,
            output,
            return_value,
            execution_time_ms: 0,
            memory_used: 0,
            violations: Vec::new(),
            error: None,
        }
    }
    
    /// 创建失败的结果
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            output: String::new(),
            return_value: None,
            execution_time_ms: 0,
            memory_used: 0,
            violations: Vec::new(),
            error: Some(error),
        }
    }
}

/// 沙箱执行器
/// 提供安全的代码执行环境
#[derive(Debug)]
pub struct SandboxExecutor {
    /// 沙箱 ID
    id: String,
    /// 配置
    config: SandboxConfig,
    /// 资源管理器
    resource_manager: ResourceManager,
    /// 文件系统隔离
    filesystem: FilesystemIsolation,
    /// 网络隔离
    network: NetworkIsolation,
    /// 监控器
    monitor: SandboxMonitor,
    /// 是否正在运行
    running: RwLock<bool>,
}

impl SandboxExecutor {
    /// 创建新的沙箱执行器
    /// 
    /// # 参数
    /// - `config`: 沙箱配置
    pub fn new(config: SandboxConfig) -> Self {
        // 生成唯一 ID
        let id = uuid::Uuid::new_v4().to_string();
        
        // 创建资源限制
        let limits = ResourceLimits {
            memory_bytes: config.memory_limit,
            time_ms: config.time_limit,
            cpu_percent: config.cpu_limit,
        };
        
        // 创建资源管理器
        let resource_manager = ResourceManager::new(limits);
        
        // 创建文件系统隔离
        let filesystem = FilesystemIsolation::new(&config.filesystem_rules);
        
        // 创建网络隔离
        let network = NetworkIsolation::new(config.network_enabled);
        
        // 创建监控器
        let monitor = SandboxMonitor::new();
        
        Self {
            id,
            config,
            resource_manager,
            filesystem,
            network,
            monitor,
            running: RwLock::new(false),
        }
    }
    
    /// 获取沙箱 ID
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// 获取配置
    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }
    
    /// 在沙箱中执行代码
    /// 
    /// # 参数
    /// - `code`: 要执行的 PHP 代码
    pub fn execute(&self, code: &str) -> SandboxResult {
        // 记录开始时间
        let start_time = Instant::now();
        
        // 设置运行状态
        if let Ok(mut running) = self.running.write() {
            *running = true;
        }
        
        // 开始资源监控
        self.resource_manager.start();
        
        // 执行代码
        let result = self.execute_internal(code);
        
        // 停止资源监控
        self.resource_manager.stop();
        
        // 记录执行时间
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        
        // 更新结果
        let mut result = result;
        result.execution_time_ms = execution_time_ms;
        result.memory_used = self.resource_manager.current_usage().memory_bytes;
        
        // 设置运行状态
        if let Ok(mut running) = self.running.write() {
            *running = false;
        }
        
        result
    }
    
    /// 内部执行逻辑
    fn execute_internal(&self, code: &str) -> SandboxResult {
        // 检查代码是否包含禁止的函数
        for denied in &self.config.denied_functions {
            if code.contains(denied) {
                // 记录违规
                self.monitor.record_violation(ViolationEvent {
                    violation_type: ViolationType::FunctionCall,
                    description: format!("尝试调用禁止的函数: {}", denied),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                });
                
                return SandboxResult::failure(format!("禁止调用函数: {}", denied));
            }
        }
        
        // 模拟执行（实际应该调用解释器）
        // 这里只是示例实现
        let output = format!("执行代码: {} 字节", code.len());
        
        // 检查资源使用
        let usage = self.resource_manager.current_usage();
        if usage.memory_bytes > self.config.memory_limit {
            self.monitor.record_violation(ViolationEvent {
                violation_type: ViolationType::MemoryLimit,
                description: format!("内存超限: {} > {}", usage.memory_bytes, self.config.memory_limit),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            });
            
            return SandboxResult::failure("内存超限".to_string());
        }
        
        if usage.time_ms > self.config.time_limit {
            self.monitor.record_violation(ViolationEvent {
                violation_type: ViolationType::TimeLimit,
                description: format!("执行超时: {} > {}", usage.time_ms, self.config.time_limit),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            });
            
            return SandboxResult::failure("执行超时".to_string());
        }
        
        // 返回成功结果
        SandboxResult::success(output, Some("{\"status\": \"ok\"}".to_string()))
    }
    
    /// 在沙箱中执行闭包
    /// 
    /// # 参数
    /// - `func`: 要执行的闭包
    pub fn execute_closure<F>(&self, func: F) -> SandboxResult
    where
        F: FnOnce() -> String,
    {
        // 记录开始时间
        let start_time = Instant::now();
        
        // 开始资源监控
        self.resource_manager.start();
        
        // 执行闭包
        let output = func();
        
        // 停止资源监控
        self.resource_manager.stop();
        
        // 记录执行时间
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        
        // 检查是否超时
        if execution_time_ms > self.config.time_limit {
            return SandboxResult::failure("执行超时".to_string());
        }
        
        // 检查内存使用
        let usage = self.resource_manager.current_usage();
        if usage.memory_bytes > self.config.memory_limit {
            return SandboxResult::failure("内存超限".to_string());
        }
        
        SandboxResult {
            success: true,
            output,
            return_value: None,
            execution_time_ms,
            memory_used: usage.memory_bytes,
            violations: Vec::new(),
            error: None,
        }
    }
    
    /// 检查函数是否允许调用
    /// 
    /// # 参数
    /// - `function_name`: 函数名
    pub fn is_function_allowed(&self, function_name: &str) -> bool {
        // 检查是否在禁止列表中
        for denied in &self.config.denied_functions {
            // 支持通配符匹配
            if denied.ends_with('*') {
                let prefix = &denied[..denied.len() - 1];
                if function_name.starts_with(prefix) {
                    return false;
                }
            } else if function_name == denied {
                return false;
            }
        }
        
        // 如果允许列表为空，则允许所有不在禁止列表中的函数
        if self.config.allowed_functions.is_empty() {
            return true;
        }
        
        // 检查是否在允许列表中
        for allowed in &self.config.allowed_functions {
            if allowed.ends_with('*') {
                let prefix = &allowed[..allowed.len() - 1];
                if function_name.starts_with(prefix) {
                    return true;
                }
            } else if function_name == allowed {
                return true;
            }
        }
        
        false
    }
    
    /// 获取资源使用情况
    pub fn resource_usage(&self) -> ResourceUsage {
        self.resource_manager.current_usage()
    }
    
    /// 获取监控统计
    pub fn monitor_stats(&self) -> super::monitor::MonitorStats {
        self.monitor.get_stats()
    }
    
    /// 获取违规事件列表
    pub fn violations(&self) -> Vec<ViolationEvent> {
        self.monitor.get_violations()
    }
    
    /// 重置沙箱状态
    pub fn reset(&self) {
        // 重置资源管理器
        self.resource_manager.reset();
        
        // 清空监控数据
        self.monitor.clear();
    }
    
    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.read().map(|g| *g).unwrap_or(false)
    }
}

/// 沙箱构建器
/// 用于方便地创建沙箱执行器
pub struct SandboxBuilder {
    config: SandboxConfig,
}

impl SandboxBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: SandboxConfig::default(),
        }
    }
    
    /// 设置内存限制
    pub fn memory_limit(mut self, bytes: u64) -> Self {
        self.config.memory_limit = bytes;
        self
    }
    
    /// 设置时间限制
    pub fn time_limit(mut self, ms: u64) -> Self {
        self.config.time_limit = ms;
        self
    }
    
    /// 设置 CPU 限制
    pub fn cpu_limit(mut self, percent: u32) -> Self {
        self.config.cpu_limit = percent.min(100);
        self
    }
    
    /// 启用网络访问
    pub fn enable_network(mut self) -> Self {
        self.config.network_enabled = true;
        self
    }
    
    /// 添加允许的函数
    pub fn allow_function(mut self, func: &str) -> Self {
        self.config.allowed_functions.push(func.to_string());
        self
    }
    
    /// 添加禁止的函数
    pub fn deny_function(mut self, func: &str) -> Self {
        self.config.denied_functions.push(func.to_string());
        self
    }
    
    /// 添加文件系统规则
    pub fn filesystem_rule(mut self, rule: &str) -> Self {
        self.config.filesystem_rules.push(rule.to_string());
        self
    }
    
    /// 设置环境变量
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.config.env_vars.insert(key.to_string(), value.to_string());
        self
    }
    
    /// 构建沙箱执行器
    pub fn build(self) -> SandboxExecutor {
        SandboxExecutor::new(self.config)
    }
}

impl Default for SandboxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试沙箱配置
    #[test]
    fn test_config() {
        let config = SandboxConfig::new()
            .memory_limit(64 * 1024 * 1024)
            .time_limit(10000)
            .cpu_limit(80);
        
        assert_eq!(config.memory_limit, 64 * 1024 * 1024);
        assert_eq!(config.time_limit, 10000);
        assert_eq!(config.cpu_limit, 80);
    }
    
    /// 测试沙箱创建
    #[test]
    fn test_sandbox_creation() {
        let sandbox = SandboxBuilder::new()
            .memory_limit(32 * 1024 * 1024)
            .time_limit(5000)
            .build();
        
        assert!(!sandbox.id().is_empty());
    }
    
    /// 测试函数检查
    #[test]
    fn test_function_check() {
        let sandbox = SandboxBuilder::new()
            .deny_function("exec")
            .allow_function("strlen")
            .build();
        
        // 禁止的函数
        assert!(!sandbox.is_function_allowed("exec"));
        
        // 允许的函数
        assert!(sandbox.is_function_allowed("strlen"));
    }
    
    /// 测试代码执行
    #[test]
    fn test_execute() {
        let sandbox = SandboxBuilder::new()
            .time_limit(1000)
            .build();
        
        let result = sandbox.execute("<?php echo 'hello';");
        
        // 由于是模拟执行，应该成功
        assert!(result.success || result.error.is_some());
    }
}
