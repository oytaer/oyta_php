//! Validate 门面

use std::collections::HashMap;

use super::validator::Validator;

/// Validate 门面
pub struct Validate;

impl Validate {
    /// 创建验证器
    pub fn make() -> Validator {
        Validator::new()
    }

    /// 快速验证数据
    pub fn check(data: &HashMap<String, String>, rules: &[(&str, &str)]) -> Result<(), HashMap<String, String>> {
        let mut validator = Validator::new();
        for (field, rule) in rules {
            validator = validator.rule(field, rule);
        }
        if validator.check(data) {
            Ok(())
        } else {
            Err(validator.errors().clone())
        }
    }
}
