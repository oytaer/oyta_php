//! 配置缓存模块
//!
//! 提供配置文件的缓存生成功能

use std::collections::HashMap;

use crate::symbol_table::types::ConfigValue;

/// 配置缓存生成器
pub struct ConfigCacheGenerator;

impl ConfigCacheGenerator {
    /// 生成配置缓存内容
    ///
    /// # 参数
    /// - `config_store`: 配置存储引用
    ///
    /// # 返回
    /// PHP 缓存文件内容
    pub fn generate_cache_content(
        config_store: &crate::config::store::ConfigStore,
    ) -> String {
        let mut content = String::new();

        content.push_str("<?php\n");
        content.push_str("/**\n");
        content.push_str(" * 配置缓存文件\n");
        content.push_str(" * 由 OYTAPHP 自动生成\n");
        content.push_str(" * 不要手动编辑此文件\n");
        content.push_str(" */\n\n");
        content.push_str("return [\n");

        // 获取所有配置键
        let keys = config_store.keys();

        // 按前缀分组
        let mut groups: HashMap<String, Vec<(String, ConfigValue)>> = HashMap::new();

        for key in keys {
            let parts: Vec<&str> = key.splitn(2, '.').collect();
            let group_name = parts[0].to_string();

            if let Some(value) = config_store.get(&key) {
                groups
                    .entry(group_name)
                    .or_default()
                    .push((key.clone(), value.clone()));
            }
        }

        // 生成每个配置组
        for (group_name, items) in groups {
            content.push_str(&format!("    // {} 配置\n", group_name));
            content.push_str(&format!("    '{}' => [\n", group_name));

            for (key, value) in items {
                // 提取子键
                let sub_key = key.splitn(2, '.').nth(1).unwrap_or(&key);
                let php_value = Self::config_value_to_php(&value, 3);
                content.push_str(&format!("        '{}' => {},\n", sub_key, php_value));
            }

            content.push_str("    ],\n\n");
        }

        content.push_str("];\n");

        content
    }

    /// 将配置值转换为 PHP 格式
    ///
    /// # 参数
    /// - `value`: 配置值
    /// - `indent`: 缩进级别
    ///
    /// # 返回
    /// PHP 格式的值字符串
    fn config_value_to_php(value: &ConfigValue, indent: usize) -> String {
        let indent_str = " ".repeat(indent * 4);

        match value {
            // 字符串类型
            ConfigValue::String(s) => format!("'{}'", s.replace('\'', "\\'")),
            // 整数类型
            ConfigValue::Int(i) => i.to_string(),
            // 浮点类型
            ConfigValue::Float(f) => f.to_string(),
            // 布尔类型
            ConfigValue::Bool(b) => {
                if *b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            // null 类型
            ConfigValue::Null => "null".to_string(),
            // 索引数组
            ConfigValue::IndexedArray(arr) => {
                let mut result = String::from("[\n");
                for item in arr {
                    result.push_str(&format!(
                        "{}{},\n",
                        indent_str,
                        Self::config_value_to_php(item, indent + 1)
                    ));
                }
                result.push_str(&format!("{}]", " ".repeat((indent - 1) * 4)));
                result
            }
            // 关联数组
            ConfigValue::AssociativeArray(map) => {
                let mut result = String::from("[\n");
                for (k, v) in map {
                    result.push_str(&format!(
                        "{}'{}' => {},\n",
                        indent_str,
                        k.replace('\'', "\\'"),
                        Self::config_value_to_php(v, indent + 1)
                    ));
                }
                result.push_str(&format!("{}]", " ".repeat((indent - 1) * 4)));
                result
            }
        }
    }
}
