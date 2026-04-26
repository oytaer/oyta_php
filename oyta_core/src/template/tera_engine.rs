//! Tera 模板引擎适配器
//!
//! 提供 Tera 模板引擎的集成支持
//! Tera 是一个受 Jinja2/Django 模板语言启发的 Rust 模板引擎
//! 当用户需要使用 Tera 语法（{{ }} / {% %}）时可以使用此适配器
//!
//! # Tera 语法示例
//! ```html
//! <h1>{{ title }}</h1>
//! {% for item in items %}
//!   <li>{{ item.name }}</li>
//! {% endfor %}
//! {% extends "base.html" %}
//! {% block content %}...{% endblock %}
//! ```
//!
//! # 使用方式
//! 在 config/view.php 中配置：
//! ```php
//! return [
//!     'type' => 'tera',
//! ];
//! ```

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tera::Tera;

/// Tera 模板引擎适配器
///
/// 封装 Tera 引擎，提供与 OYTAPHP 模板系统兼容的接口
/// 支持 Tera 的所有特性：继承、包含、宏、过滤器等
pub struct TeraTemplateEngine {
    /// Tera 引擎实例
    tera: Tera,
    /// 模板目录路径
    view_dir: PathBuf,
    /// 全局变量
    globals: HashMap<String, serde_json::Value>,
}

impl TeraTemplateEngine {
    /// 创建新的 Tera 模板引擎
    ///
    /// # 参数
    /// - `view_dir`: 模板目录路径
    pub fn new(view_dir: &Path) -> Result<Self> {
        let view_dir_str = view_dir.to_string_lossy();
        let glob_pattern = format!("{}/**/*.html", view_dir_str);

        let tera = Tera::new(&glob_pattern)
            .with_context(|| format!("初始化 Tera 模板引擎失败，模板目录: {:?}", view_dir))?;

        Ok(Self {
            tera,
            view_dir: view_dir.to_path_buf(),
            globals: HashMap::new(),
        })
    }

    /// 创建空的 Tera 模板引擎
    pub fn empty() -> Self {
        Self {
            tera: Tera::default(),
            view_dir: PathBuf::new(),
            globals: HashMap::new(),
        }
    }

    /// 渲染模板
    ///
    /// # 参数
    /// - `template_name`: 模板名称（相对于 view 目录），如 "index/index.html"
    /// - `context`: 模板变量上下文
    pub fn render(
        &self,
        template_name: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        // 合并全局变量
        let mut full_context = self.globals.clone();
        full_context.extend(context.clone());

        let name = if template_name.ends_with(".html") {
            template_name.to_string()
        } else {
            format!("{}.html", template_name)
        };

        self.tera
            .render(&name, &tera::Context::from_serialize(&full_context)?)
            .with_context(|| format!("渲染模板失败: {}", name))
    }

    /// 注册全局变量
    /// 全局变量在所有模板中可用
    ///
    /// # 参数
    /// - `name`: 变量名
    /// - `value`: 变量值
    pub fn register_global(&mut self, name: &str, value: serde_json::Value) {
        self.globals.insert(name.to_string(), value);
    }

    /// 注册自定义过滤器
    ///
    /// # 参数
    /// - `name`: 过滤器名称
    /// - `filter`: 过滤器函数
    pub fn register_filter(
        &mut self,
        name: &str,
        filter: impl Fn(&serde_json::Value, &HashMap<String, serde_json::Value>) -> tera::Result<serde_json::Value> + Send + Sync + 'static,
    ) {
        self.tera.register_filter(name, filter);
    }

    /// 注册自定义测试函数
    ///
    /// # 参数
    /// - `name`: 测试名称
    /// - `test`: 测试函数
    pub fn register_test(
        &mut self,
        name: &str,
        test: impl tera::Test + 'static,
    ) {
        self.tera.register_tester(name, test);
    }

    /// 注册自定义函数
    ///
    /// # 参数
    /// - `name`: 函数名称
    /// - `func`: 函数实现
    pub fn register_function(
        &mut self,
        name: &str,
        func: impl tera::Function + 'static,
    ) {
        self.tera.register_function(name, func);
    }

    /// 添加原始模板字符串
    /// 用于动态创建模板（不通过文件系统）
    ///
    /// # 参数
    /// - `name`: 模板名称
    /// - `content`: 模板内容
    pub fn add_template(&mut self, name: &str, content: &str) -> Result<()> {
        self.tera
            .add_raw_template(name, content)
            .with_context(|| format!("添加模板失败: {}", name))
    }

    /// 重新加载模板文件
    /// 在开发模式下，文件修改后需要重新加载
    pub fn reload(&mut self) -> Result<()> {
        let view_dir_str = self.view_dir.to_string_lossy();
        let glob_pattern = format!("{}/**/*.html", view_dir_str);

        self.tera
            .full_reload()
            .with_context(|| format!("重新加载模板失败: {}", glob_pattern))
    }

    /// 获取模板目录路径
    pub fn view_dir(&self) -> &Path {
        &self.view_dir
    }

    /// 获取所有已注册的模板名称
    pub fn get_template_names(&self) -> Vec<String> {
        self.tera.get_template_names().map(|s| s.to_string()).collect()
    }
}

/// Tera 辅助函数：生成 URL
/// 在模板中使用 {{ url(path="/hello") }}
pub struct UrlFunction;

impl tera::Function for UrlFunction {
    fn call(&self, args: &HashMap<String, serde_json::Value>) -> tera::Result<serde_json::Value> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("/");
        Ok(serde_json::Value::String(path.to_string()))
    }
}

/// Tera 辅助函数：生成静态资源 URL
/// 在模板中使用 {{ asset(path="/css/style.css") }}
pub struct AssetFunction;

impl tera::Function for AssetFunction {
    fn call(&self, args: &HashMap<String, serde_json::Value>) -> tera::Result<serde_json::Value> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("/");
        Ok(serde_json::Value::String(format!("/static{}", path)))
    }
}
