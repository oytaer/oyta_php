//! 网络隔离模块
//!
//! 提供沙箱的网络访问控制

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 网络权限
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkPermission {
    /// 允许所有网络访问
    AllowAll,
    /// 仅允许出站连接
    OutboundOnly,
    /// 仅允许入站连接
    InboundOnly,
    /// 禁止所有网络访问
    DenyAll,
}

/// 网络规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRule {
    /// 规则名称
    pub name: String,
    /// 目标主机（支持通配符）
    pub host: String,
    /// 端口范围（开始）
    pub port_start: u16,
    /// 端口范围（结束）
    pub port_end: u16,
    /// 是否允许
    pub allowed: bool,
}

impl NetworkRule {
    /// 创建新的网络规则
    pub fn new(name: &str, host: &str, port_start: u16, port_end: u16, allowed: bool) -> Self {
        Self {
            name: name.to_string(),
            host: host.to_string(),
            port_start,
            port_end,
            allowed,
        }
    }
    
    /// 检查是否匹配
    pub fn matches(&self, host: &str, port: u16) -> bool {
        // 检查端口范围
        if port < self.port_start || port > self.port_end {
            return false;
        }
        
        // 检查主机匹配
        if self.host == "*" || self.host == host {
            return true;
        }
        
        // 支持通配符后缀匹配
        if self.host.starts_with("*.") {
            let suffix = &self.host[1..]; // 去掉 *
            if host.ends_with(suffix) {
                return true;
            }
        }
        
        false
    }
}

/// 网络隔离
/// 控制沙箱的网络访问
#[derive(Debug, Clone)]
pub struct NetworkIsolation {
    /// 是否启用网络
    enabled: bool,
    /// 默认权限
    default_permission: NetworkPermission,
    /// 网络规则列表
    rules: Vec<NetworkRule>,
    /// 允许的域名列表
    allowed_domains: Vec<String>,
    /// 禁止的域名列表
    denied_domains: Vec<String>,
    /// 允许的端口列表
    allowed_ports: Vec<u16>,
    /// 禁止的端口列表
    denied_ports: Vec<u16>,
}

impl NetworkIsolation {
    /// 创建新的网络隔离
    /// 
    /// # 参数
    /// - `enabled`: 是否启用网络访问
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            default_permission: if enabled {
                NetworkPermission::OutboundOnly
            } else {
                NetworkPermission::DenyAll
            },
            rules: Vec::new(),
            allowed_domains: Vec::new(),
            denied_domains: Vec::new(),
            allowed_ports: Vec::new(),
            denied_ports: Vec::new(),
        }
    }
    
    /// 启用网络
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    /// 禁用网络
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    /// 设置默认权限
    pub fn set_default_permission(&mut self, permission: NetworkPermission) {
        self.default_permission = permission;
    }
    
    /// 添加网络规则
    pub fn add_rule(&mut self, rule: NetworkRule) {
        self.rules.push(rule);
    }
    
    /// 移除网络规则
    pub fn remove_rule(&mut self, name: &str) {
        self.rules.retain(|r| r.name != name);
    }
    
    /// 添加允许的域名
    pub fn allow_domain(&mut self, domain: &str) {
        self.allowed_domains.push(domain.to_string());
    }
    
    /// 添加禁止的域名
    pub fn deny_domain(&mut self, domain: &str) {
        self.denied_domains.push(domain.to_string());
    }
    
    /// 添加允许的端口
    pub fn allow_port(&mut self, port: u16) {
        self.allowed_ports.push(port);
    }
    
    /// 添加禁止的端口
    pub fn deny_port(&mut self, port: u16) {
        self.denied_ports.push(port);
    }
    
    /// 检查是否允许连接
    /// 
    /// # 参数
    /// - `host`: 目标主机
    /// - `port`: 目标端口
    pub fn can_connect(&self, host: &str, port: u16) -> bool {
        // 如果禁用网络，拒绝所有连接
        if !self.enabled {
            return false;
        }
        
        // 检查默认权限
        match self.default_permission {
            NetworkPermission::DenyAll => return false,
            NetworkPermission::AllowAll => {}
            NetworkPermission::OutboundOnly => {} // 出站连接允许
            NetworkPermission::InboundOnly => return false,
        }
        
        // 检查禁止的端口
        if self.denied_ports.contains(&port) {
            return false;
        }
        
        // 检查禁止的域名
        for denied in &self.denied_domains {
            if Self::domain_matches(host, denied) {
                return false;
            }
        }
        
        // 检查允许的端口（如果有定义）
        if !self.allowed_ports.is_empty() && !self.allowed_ports.contains(&port) {
            return false;
        }
        
        // 检查允许的域名（如果有定义）
        if !self.allowed_domains.is_empty() {
            let mut found = false;
            for allowed in &self.allowed_domains {
                if Self::domain_matches(host, allowed) {
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        
        // 检查规则
        for rule in &self.rules {
            if rule.matches(host, port) {
                return rule.allowed;
            }
        }
        
        // 默认根据权限决定
        true
    }
    
    /// 检查是否允许监听
    /// 
    /// # 参数
    /// - `port`: 监听端口
    pub fn can_listen(&self, port: u16) -> bool {
        // 如果禁用网络，拒绝所有监听
        if !self.enabled {
            return false;
        }
        
        // 检查默认权限
        match self.default_permission {
            NetworkPermission::DenyAll => return false,
            NetworkPermission::AllowAll => return true,
            NetworkPermission::OutboundOnly => return false,
            NetworkPermission::InboundOnly => return true,
        }
    }
    
    /// 检查域名是否匹配
    fn domain_matches(host: &str, pattern: &str) -> bool {
        // 精确匹配
        if host == pattern {
            return true;
        }
        
        // 通配符匹配
        if pattern.starts_with("*.") {
            let suffix = &pattern[1..];
            if host.ends_with(suffix) {
                return true;
            }
        }
        
        false
    }
    
    /// 获取所有规则
    pub fn get_rules(&self) -> &[NetworkRule] {
        &self.rules
    }
    
    /// 获取允许的域名列表
    pub fn get_allowed_domains(&self) -> &[String] {
        &self.allowed_domains
    }
    
    /// 获取禁止的域名列表
    pub fn get_denied_domains(&self) -> &[String] {
        &self.denied_domains
    }
    
    /// 获取网络状态摘要
    pub fn summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();
        
        summary.insert("enabled".to_string(), self.enabled.to_string());
        summary.insert("default_permission".to_string(), format!("{:?}", self.default_permission));
        summary.insert("rules_count".to_string(), self.rules.len().to_string());
        summary.insert("allowed_domains_count".to_string(), self.allowed_domains.len().to_string());
        summary.insert("denied_domains_count".to_string(), self.denied_domains.len().to_string());
        
        summary
    }
}

impl Default for NetworkIsolation {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试网络规则匹配
    #[test]
    fn test_rule_matching() {
        let rule = NetworkRule::new("allow-http", "example.com", 80, 80, true);
        
        assert!(rule.matches("example.com", 80));
        assert!(!rule.matches("example.com", 443));
        assert!(!rule.matches("other.com", 80));
    }
    
    /// 测试通配符规则
    #[test]
    fn test_wildcard_rule() {
        let rule = NetworkRule::new("allow-subdomains", "*.example.com", 443, 443, true);
        
        assert!(rule.matches("api.example.com", 443));
        assert!(rule.matches("www.example.com", 443));
        assert!(!rule.matches("example.com", 443));
        assert!(!rule.matches("other.com", 443));
    }
    
    /// 测试网络隔离
    #[test]
    fn test_isolation() {
        let mut isolation = NetworkIsolation::new(true);
        
        // 默认允许出站连接
        assert!(isolation.can_connect("example.com", 80));
        
        // 禁止特定域名
        isolation.deny_domain("blocked.com");
        assert!(!isolation.can_connect("blocked.com", 80));
        
        // 禁用网络
        isolation.disable();
        assert!(!isolation.can_connect("example.com", 80));
    }
    
    /// 测试端口限制
    #[test]
    fn test_port_restriction() {
        let mut isolation = NetworkIsolation::new(true);
        
        // 只允许特定端口
        isolation.allow_port(80);
        isolation.allow_port(443);
        
        assert!(isolation.can_connect("example.com", 80));
        assert!(isolation.can_connect("example.com", 443));
        assert!(!isolation.can_connect("example.com", 8080));
    }
    
    /// 测试监听权限
    #[test]
    fn test_listen_permission() {
        let mut isolation = NetworkIsolation::new(true);
        isolation.set_default_permission(NetworkPermission::InboundOnly);
        
        assert!(isolation.can_listen(8080));
        assert!(!isolation.can_connect("example.com", 80));
    }
}
