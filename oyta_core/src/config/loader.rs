//! 配置加载器模块
//!
//! 负责从多个来源加载配置：
//! 1. PHP 配置文件（config/*.php，return [] 格式）
//! 2. 环境变量（.env 文件 + 系统环境变量）
//! 3. figment 框架（支持 toml/yaml 等额外格式）
//! 加载优先级：环境变量 > PHP 配置文件 > 默认值

use anyhow::{Context, Result};
use std::path::Path;

use super::store::ConfigStore;
use crate::parser::php_parser::PhpParser;
use crate::symbol_table::types::ConfigValue;

/// 配置加载器
/// 负责从项目目录加载所有配置并存储到 ConfigStore
pub struct ConfigLoader {
    /// 配置目录路径
    config_dir: std::path::PathBuf,
}

impl ConfigLoader {
    /// 创建新的配置加载器
    pub fn new(config_dir: &Path) -> Self {
        Self {
            config_dir: config_dir.to_path_buf(),
        }
    }

    /// 加载所有配置
    /// 按顺序加载：
    /// 1. PHP 配置文件
    /// 2. 环境变量覆盖
    /// 3. figment 额外配置（如果存在）
    pub fn load(&self, store: &ConfigStore, parser: &mut PhpParser) -> Result<()> {
        // 第一步：加载 PHP 配置文件
        self.load_php_configs(store, parser)?;

        // 第二步：应用环境变量覆盖
        self.apply_env_overrides(store);

        // 第三步：加载 figment 额外配置（如果存在）
        self.load_figment_configs(store)?;

        tracing::debug!("配置加载完成: {} 个配置项", store.len());
        Ok(())
    }

    /// 加载 PHP 配置文件
    /// 扫描 config/ 目录下的所有 .php 文件
    /// 文件名作为配置的顶级键名
    fn load_php_configs(&self, store: &ConfigStore, parser: &mut PhpParser) -> Result<()> {
        if !self.config_dir.is_dir() {
            tracing::warn!("配置目录不存在: {}", self.config_dir.display());
            return Ok(());
        }

        let entries = std::fs::read_dir(&self.config_dir)
            .with_context(|| format!("无法读取配置目录: {}", self.config_dir.display()))?;

        for entry in entries.flatten() {
            let path = entry.path();

            // 只处理 .php 文件
            if path.extension().and_then(|e| e.to_str()) != Some("php") {
                continue;
            }

            // 获取文件名作为配置键名
            let config_name = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            // 解析 PHP 配置文件
            match parser.parse_config_file(&path) {
                Ok(ConfigValue::AssociativeArray(items)) => {
                    // 将关联数组展开为点号路径
                    store.set_from_associative(&config_name, &items);
                    tracing::debug!("配置文件加载成功: {}", config_name);
                }
                Ok(other) => {
                    // 非关联数组的配置值，直接存储
                    store.set(&config_name, other);
                    tracing::debug!("配置文件加载成功(非数组): {}", config_name);
                }
                Err(e) => {
                    tracing::warn!("配置文件解析失败: {} - {}", path.display(), e);
                }
            }
        }

        Ok(())
    }

    /// 应用环境变量覆盖
    /// 环境变量格式：OYTA_ 前缀 + 大写键名 + 下划线分隔
    /// 如 OYTA_APP_DEBUG=true 覆盖 app.debug 配置
    fn apply_env_overrides(&self, store: &ConfigStore) {
        // 遍历所有配置键，检查是否有对应的环境变量覆盖
        let keys: Vec<String> = store.keys();

        for key in keys {
            // 将配置键转换为环境变量名
            // app.debug → OYTA_APP_DEBUG
            let env_key = format!("OYTA_{}", key.to_uppercase().replace('.', "_"));

            if let Ok(env_value) = std::env::var(&env_key) {
                // 根据原始值的类型推断环境变量的类型
                if let Some(original) = store.get(&key) {
                    let override_value = match original {
                        ConfigValue::Bool(_) => {
                            // 布尔值：true/false/1/0/yes/no
                            match env_value.to_lowercase().as_str() {
                                "true" | "1" | "yes" => ConfigValue::Bool(true),
                                "false" | "0" | "no" => ConfigValue::Bool(false),
                                _ => ConfigValue::String(env_value),
                            }
                        }
                        ConfigValue::Int(_) => {
                            // 整数值
                            env_value.parse::<i64>()
                                .map(ConfigValue::Int)
                                .unwrap_or(ConfigValue::String(env_value))
                        }
                        ConfigValue::Float(_) => {
                            // 浮点数值
                            env_value.parse::<f64>()
                                .map(ConfigValue::Float)
                                .unwrap_or(ConfigValue::String(env_value))
                        }
                        _ => ConfigValue::String(env_value),
                    };
                    store.set(&key, override_value);
                    tracing::debug!("环境变量覆盖配置: {} = {:?}", key, store.get(&key));
                }
            }
        }

        // 处理特殊的环境变量配置
        // APP_DEBUG → app.debug
        if let Ok(val) = std::env::var("APP_DEBUG") {
            let bool_val = matches!(val.to_lowercase().as_str(), "true" | "1" | "yes");
            store.set("app.debug", ConfigValue::Bool(bool_val));
        }

        // APP_ENV → app.env
        if let Ok(val) = std::env::var("APP_ENV") {
            store.set("app.env", ConfigValue::String(val));
        }
    }

    /// 加载 figment 额外配置
    /// 支持 .toml 和 .yaml 格式的配置文件
    fn load_figment_configs(&self, store: &ConfigStore) -> Result<()> {
        // 检查是否存在 config.toml
        let toml_path = self.config_dir.join("config.toml");
        if toml_path.exists() {
            self.load_figment_file(store, &toml_path, "toml")?;
        }

        // 检查是否存在 config.yaml
        let yaml_path = self.config_dir.join("config.yaml");
        if yaml_path.exists() {
            self.load_figment_file(store, &yaml_path, "yaml")?;
        }

        Ok(())
    }

    /// 使用 figment 加载单个配置文件
    fn load_figment_file(&self, store: &ConfigStore, path: &Path, format: &str) -> Result<()> {
        use figment::{Figment, providers::Format};

        let figment = match format {
            "toml" => {
                Figment::new()
                    .merge(figment::providers::Toml::file(path))
            }
            "yaml" => {
                Figment::new()
                    .merge(figment::providers::Yaml::file(path))
            }
            _ => return Ok(()),
        };

        // 将 figment 的配置数据提取为 JSON，然后转换为 ConfigValue
        let value: serde_json::Value = figment.extract()
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));

        if let serde_json::Value::Object(map) = value {
            for (key, val) in map {
                let config_val = json_to_config_value(val);
                match &config_val {
                    ConfigValue::AssociativeArray(items) => {
                        store.set_from_associative(&key, items);
                    }
                    _ => {
                        store.set(&key, config_val);
                    }
                }
            }
        }

        tracing::debug!("figment 配置文件加载成功: {} ({})", path.display(), format);
        Ok(())
    }
}

/// 将 JSON 值转换为 ConfigValue
fn json_to_config_value(val: serde_json::Value) -> ConfigValue {
    match val {
        serde_json::Value::String(s) => ConfigValue::String(s),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                ConfigValue::Int(i)
            } else if let Some(f) = n.as_f64() {
                ConfigValue::Float(f)
            } else {
                ConfigValue::String(n.to_string())
            }
        }
        serde_json::Value::Bool(b) => ConfigValue::Bool(b),
        serde_json::Value::Null => ConfigValue::Null,
        serde_json::Value::Array(arr) => {
            let items: Vec<ConfigValue> = arr.into_iter().map(json_to_config_value).collect();
            ConfigValue::IndexedArray(items)
        }
        serde_json::Value::Object(map) => {
            let items: Vec<(String, ConfigValue)> = map
                .into_iter()
                .map(|(k, v)| (k, json_to_config_value(v)))
                .collect();
            ConfigValue::AssociativeArray(items)
        }
    }
}
