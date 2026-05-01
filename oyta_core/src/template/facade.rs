//! View 门面
//!
//! 对应 ThinkPHP 8.0 的 View 门面
//! 提供静态方法访问模板渲染功能
//!
//! # 功能特性
//! - 模板渲染
//! - 模板变量分配
//! - 全局变量管理
//! - 模板缓存
//!
//! # 使用示例
//! ```php
//! // PHP 风格调用
//! View::assign('name', 'value');
//! View::assign(['key1' => 'value1', 'key2' => 'value2']);
//! echo View::fetch('index/index');
//! ```

use anyhow::Result;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;

use super::engine::TemplateEngine;

/// 全局模板引擎实例
static TEMPLATE_ENGINE: Lazy<RwLock<Option<TemplateEngine>>> = Lazy::new(|| {
    RwLock::new(None)
});

/// 全局模板变量存储
///
/// 存储通过 assign() 方法分配的全局变量
/// 这些变量在所有模板渲染时都可用
static GLOBAL_VARIABLES: Lazy<RwLock<HashMap<String, serde_json::Value>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

/// 模板配置
static TEMPLATE_CONFIG: Lazy<RwLock<TemplateConfig>> = Lazy::new(|| {
    RwLock::new(TemplateConfig::default())
});

/// 模板配置结构
#[derive(Debug, Clone)]
pub struct TemplateConfig {
    /// 模板目录
    pub view_dir: PathBuf,
    /// 模板后缀
    pub view_suffix: String,
    /// 是否开启模板缓存
    pub cache_enabled: bool,
    /// 模板主题
    pub theme: String,
    /// 默认模板引擎
    pub engine: String,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            view_dir: PathBuf::from("app/view"),
            view_suffix: ".html".to_string(),
            cache_enabled: true,
            theme: "default".to_string(),
            engine: "native".to_string(),
        }
    }
}

/// View 门面
///
/// 提供静态方法访问模板渲染功能
pub struct View;

impl View {
    /// 初始化模板引擎
    ///
    /// # 参数
    /// - `view_dir`: 模板目录路径
    ///
    /// # 示例
    /// ```rust
    /// View::init(std::path::Path::new("app/view"))?;
    /// ```
    pub fn init(view_dir: &std::path::Path) -> Result<()> {
        let engine = TemplateEngine::new(view_dir)?;

        // 保存配置
        {
            let mut config = TEMPLATE_CONFIG.write();
            config.view_dir = view_dir.to_path_buf();
        }

        let mut guard = TEMPLATE_ENGINE.write();
        *guard = Some(engine);

        tracing::info!("模板引擎已初始化: {}", view_dir.display());
        Ok(())
    }

    /// 渲染模板
    ///
    /// # 参数
    /// - `template_name`: 模板名称（不含后缀）
    /// - `data`: 局部变量
    ///
    /// # 返回
    /// 渲染后的 HTML 字符串
    ///
    /// # 示例
    /// ```rust
    /// let mut data = HashMap::new();
    /// data.insert("title".to_string(), serde_json::Value::String("首页".to_string()));
    /// let html = View::fetch("index/index", &data)?;
    /// ```
    pub fn fetch(template_name: &str, data: &HashMap<String, serde_json::Value>) -> Result<String> {
        // 合并全局变量和局部变量
        let merged_data = Self::merge_variables(data);

        let guard = TEMPLATE_ENGINE.read();
        match guard.as_ref() {
            Some(engine) => engine.render(template_name, &merged_data),
            None => anyhow::bail!("模板引擎未初始化"),
        }
    }

    /// 渲染模板并返回 HTML 响应
    ///
    /// # 参数
    /// - `template_name`: 模板名称
    /// - `data`: 模板变量
    ///
    /// # 返回
    /// HTTP 响应对象
    pub fn display(template_name: &str, data: &HashMap<String, serde_json::Value>) -> Result<crate::http::response::OytaResponse> {
        let html = Self::fetch(template_name, data)?;
        Ok(crate::http::response::OytaResponse::html(html))
    }

    /// 分配模板变量（单个）
    ///
    /// 将变量存储到全局变量池，在所有模板渲染时可用
    ///
    /// # 参数
    /// - `name`: 变量名
    /// - `value`: 变量值
    ///
    /// # 示例
    /// ```rust
    /// View::assign("title", serde_json::Value::String("网站标题".to_string()));
    /// View::assign("user", serde_json::Value::Object(user_map));
    /// ```
    pub fn assign(name: &str, value: serde_json::Value) {
        let mut vars = GLOBAL_VARIABLES.write();
        vars.insert(name.to_string(), value);

        tracing::debug!("分配模板变量: {}", name);
    }

    /// 批量分配模板变量
    ///
    /// # 参数
    /// - `variables`: 变量映射
    ///
    /// # 示例
    /// ```rust
    /// let mut vars = HashMap::new();
    /// vars.insert("title".to_string(), serde_json::Value::String("标题".to_string()));
    /// vars.insert("keywords".to_string(), serde_json::Value::String("关键词".to_string()));
    /// View::assign_batch(&vars);
    /// ```
    pub fn assign_batch(variables: &HashMap<String, serde_json::Value>) {
        let mut vars = GLOBAL_VARIABLES.write();
        for (name, value) in variables {
            vars.insert(name.clone(), value.clone());
        }

        tracing::debug!("批量分配模板变量: {} 个", variables.len());
    }

    /// 获取已分配的变量
    ///
    /// # 参数
    /// - `name`: 变量名
    ///
    /// # 返回
    /// 变量值的克隆，如果不存在返回 None
    pub fn get(name: &str) -> Option<serde_json::Value> {
        let vars = GLOBAL_VARIABLES.read();
        vars.get(name).cloned()
    }

    /// 检查变量是否已分配
    ///
    /// # 参数
    /// - `name`: 变量名
    ///
    /// # 返回
    /// 是否存在
    pub fn has(name: &str) -> bool {
        let vars = GLOBAL_VARIABLES.read();
        vars.contains_key(name)
    }

    /// 移除已分配的变量
    ///
    /// # 参数
    /// - `name`: 变量名
    pub fn remove(name: &str) {
        let mut vars = GLOBAL_VARIABLES.write();
        vars.remove(name);

        tracing::debug!("移除模板变量: {}", name);
    }

    /// 清空所有已分配的变量
    pub fn clear() {
        let mut vars = GLOBAL_VARIABLES.write();
        vars.clear();

        tracing::debug!("清空所有模板变量");
    }

    /// 获取所有全局变量
    ///
    /// # 返回
    /// 全局变量映射的克隆
    pub fn all() -> HashMap<String, serde_json::Value> {
        let vars = GLOBAL_VARIABLES.read();
        vars.clone()
    }

    /// 合并全局变量和局部变量
    ///
    /// 局部变量优先级高于全局变量
    ///
    /// # 参数
    /// - `local_vars`: 局部变量
    ///
    /// # 返回
    /// 合并后的变量映射
    fn merge_variables(local_vars: &HashMap<String, serde_json::Value>) -> HashMap<String, serde_json::Value> {
        let mut merged = HashMap::new();

        // 先添加全局变量
        {
            let global_vars = GLOBAL_VARIABLES.read();
            for (name, value) in global_vars.iter() {
                merged.insert(name.clone(), value.clone());
            }
        }

        // 再添加局部变量（覆盖同名全局变量）
        for (name, value) in local_vars {
            merged.insert(name.clone(), value.clone());
        }

        merged
    }

    /// 重新加载模板
    ///
    /// 清空模板缓存
    pub fn reload() -> Result<()> {
        let mut guard = TEMPLATE_ENGINE.write();
        match guard.as_mut() {
            Some(engine) => engine.reload(),
            None => anyhow::bail!("模板引擎未初始化"),
        }
    }

    /// 设置模板主题
    ///
    /// # 参数
    /// - `theme`: 主题名称
    pub fn set_theme(theme: &str) {
        let mut config = TEMPLATE_CONFIG.write();
        config.theme = theme.to_string();

        tracing::info!("设置模板主题: {}", theme);
    }

    /// 获取当前主题
    ///
    /// # 返回
    /// 主题名称
    pub fn get_theme() -> String {
        let config = TEMPLATE_CONFIG.read();
        config.theme.clone()
    }

    /// 获取模板配置
    ///
    /// # 返回
    /// 配置克隆
    pub fn get_config() -> TemplateConfig {
        let config = TEMPLATE_CONFIG.read();
        config.clone()
    }

    /// 注册自定义过滤器
    ///
    /// # 参数
    /// - `name`: 过滤器名称
    /// - `filter`: 过滤器函数
    ///
    /// # 示例
    /// ```rust
    /// View::register_filter("my_filter", |value| {
    ///     format!("[{}]", value)
    /// });
    /// ```
    pub fn register_filter(name: &str, filter: impl Fn(&str) -> String + Send + Sync + 'static) {
        let mut guard = TEMPLATE_ENGINE.write();
        if let Some(engine) = guard.as_mut() {
            engine.register_filter(name, filter);
            tracing::debug!("注册模板过滤器: {}", name);
        }
    }

    /// 注册模板函数
    ///
    /// # 参数
    /// - `name`: 函数名称
    /// - `func`: 函数实现
    pub fn register_function(name: &str, func: Box<dyn Fn(&HashMap<String, serde_json::Value>) -> String + Send + Sync>) {
        let mut guard = TEMPLATE_ENGINE.write();
        if let Some(engine) = guard.as_mut() {
            engine.register_function(name, func);
            tracing::debug!("注册模板函数: {}", name);
        }
    }

    /// 添加原始模板字符串
    ///
    /// # 参数
    /// - `name`: 模板名称
    /// - `content`: 模板内容
    pub fn add_template(name: &str, content: &str) -> Result<()> {
        let mut guard = TEMPLATE_ENGINE.write();
        match guard.as_mut() {
            Some(engine) => engine.add_template(name, content),
            None => anyhow::bail!("模板引擎未初始化"),
        }
    }

    /// 判断模板引擎是否已初始化
    ///
    /// # 返回
    /// 是否已初始化
    pub fn is_initialized() -> bool {
        let guard = TEMPLATE_ENGINE.read();
        guard.is_some()
    }

    /// 获取全局变量数量
    ///
    /// # 返回
    /// 变量数量
    pub fn count() -> usize {
        let vars = GLOBAL_VARIABLES.read();
        vars.len()
    }
}

/// 模板变量构建器
///
/// 提供链式调用方式分配模板变量
pub struct ViewBuilder {
    /// 变量存储
    variables: HashMap<String, serde_json::Value>,
}

impl ViewBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// 添加变量
    ///
    /// # 参数
    /// - `name`: 变量名
    /// - `value`: 变量值
    ///
    /// # 返回
    /// 构建器自身（支持链式调用）
    pub fn with(mut self, name: &str, value: serde_json::Value) -> Self {
        self.variables.insert(name.to_string(), value);
        self
    }

    /// 添加字符串变量
    pub fn with_string(self, name: &str, value: &str) -> Self {
        self.with(name, serde_json::Value::String(value.to_string()))
    }

    /// 添加整数变量
    pub fn with_int(self, name: &str, value: i64) -> Self {
        self.with(name, serde_json::Value::Number(value.into()))
    }

    /// 添加布尔变量
    pub fn with_bool(self, name: &str, value: bool) -> Self {
        self.with(name, serde_json::Value::Bool(value))
    }

    /// 添加数组变量
    pub fn with_array(self, name: &str, value: Vec<serde_json::Value>) -> Self {
        self.with(name, serde_json::Value::Array(value))
    }

    /// 添加对象变量
    pub fn with_object(self, name: &str, value: serde_json::Map<String, serde_json::Value>) -> Self {
        self.with(name, serde_json::Value::Object(value))
    }

    /// 渲染模板
    ///
    /// # 参数
    /// - `template_name`: 模板名称
    ///
    /// # 返回
    /// 渲染后的 HTML
    pub fn render(self, template_name: &str) -> Result<String> {
        View::fetch(template_name, &self.variables)
    }

    /// 渲染并返回响应
    ///
    /// # 参数
    /// - `template_name`: 模板名称
    ///
    /// # 返回
    /// HTTP 响应
    pub fn display(self, template_name: &str) -> Result<crate::http::response::OytaResponse> {
        View::display(template_name, &self.variables)
    }

    /// 应用到全局变量
    pub fn apply(self) {
        View::assign_batch(&self.variables);
    }

    /// 获取变量映射
    pub fn build(self) -> HashMap<String, serde_json::Value> {
        self.variables
    }
}

impl Default for ViewBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_view_assign() {
        View::clear();
        View::assign("title", serde_json::Value::String("测试标题".to_string()));

        assert!(View::has("title"));
        assert_eq!(
            View::get("title"),
            Some(serde_json::Value::String("测试标题".to_string()))
        );
    }

    #[test]
    #[serial]
    fn test_view_assign_batch() {
        View::clear();

        let mut vars = HashMap::new();
        vars.insert("key1".to_string(), serde_json::Value::String("value1".to_string()));
        vars.insert("key2".to_string(), serde_json::Value::String("value2".to_string()));

        View::assign_batch(&vars);

        assert_eq!(View::count(), 2);
    }

    #[test]
    #[serial]
    fn test_view_remove() {
        View::clear();
        View::assign("test", serde_json::Value::String("test".to_string()));

        assert!(View::has("test"));

        View::remove("test");

        assert!(!View::has("test"));
    }

    #[test]
    fn test_view_builder() {
        let vars = ViewBuilder::new()
            .with_string("name", "OYTAPHP")
            .with_int("version", 1)
            .with_bool("debug", true)
            .build();

        assert_eq!(vars.len(), 3);
        assert_eq!(vars.get("name"), Some(&serde_json::Value::String("OYTAPHP".to_string())));
    }

    #[test]
    fn test_template_config_default() {
        let config = TemplateConfig::default();

        assert_eq!(config.view_suffix, ".html");
        assert!(config.cache_enabled);
        assert_eq!(config.theme, "default");
    }
}
