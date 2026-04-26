//! 验证器核心

use std::collections::HashMap;

use super::rules::{self, Rule};

/// 验证器
pub struct Validator {
    /// 验证规则列表
    rules: Vec<Rule>,
    /// 验证错误
    errors: HashMap<String, String>,
    /// 验证场景
    scene: Option<String>,
    /// 场景规则映射
    scene_rules: HashMap<String, Vec<Rule>>,
}

impl Validator {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            errors: HashMap::new(),
            scene: None,
            scene_rules: HashMap::new(),
        }
    }

    /// 添加验证规则
    pub fn rule(mut self, field: &str, rule: &str) -> Self {
        // 支持多规则：rule1|rule2|rule3
        for r in rule.split('|') {
            self.rules.push(Rule::new(field, r));
        }
        self
    }

    /// 设置验证场景
    pub fn scene(mut self, name: &str) -> Self {
        self.scene = Some(name.to_string());
        self
    }

    /// 执行验证
    pub fn check(&mut self, data: &HashMap<String, String>) -> bool {
        self.errors.clear();

        let rules_to_check = if let Some(scene) = &self.scene {
            self.scene_rules.get(scene).unwrap_or(&self.rules)
        } else {
            &self.rules
        };

        for rule in rules_to_check {
            let value = data.get(&rule.field).map(|s| s.as_str()).unwrap_or("");
            if let Err(msg) = rules::validate_rule(&rule.rule, value, &rule.params) {
                let error_msg = rule.message.clone().unwrap_or(msg);
                self.errors.insert(rule.field.clone(), error_msg);
            }
        }

        self.errors.is_empty()
    }

    /// 获取验证错误
    pub fn errors(&self) -> &HashMap<String, String> {
        &self.errors
    }

    /// 获取第一个错误
    pub fn first_error(&self) -> Option<&String> {
        self.errors.values().next()
    }

    /// 是否有错误
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}
