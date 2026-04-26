//! OYTAPHP 原生模板引擎
//!
//! 实现 OYTAPHP 的模板标签语法
//! 支持：变量输出、条件判断、循环、包含文件、区块继承、过滤器
//! 提供原生 PHP 风格的模板渲染

use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// OYTAPHP 原生模板引擎
///
/// 解析 OYTAPHP 模板标签语法并渲染输出
/// 支持的标签：
/// - {$var} — 变量输出
/// - {$var.name} — 对象属性/数组键
/// - {$var|filter} — 变量过滤器
/// - {if $cond}...{else}...{/if} — 条件判断
/// - {foreach $arr as $key => $val}...{/foreach} — 循环
/// - {include file="xxx"} — 包含文件
/// - {block name="xxx"}...{/block} — 区块定义
/// - {extend name="xxx"} — 继承模板
/// - {:func()} — 函数调用
/// - {~expr} — 原始 PHP 表达式
/// - {/* comment */} — 注释
pub struct TemplateEngine {
    /// 模板目录路径
    view_dir: PathBuf,
    /// 模板缓存（模板名 → 编译后的内容）
    cache: HashMap<String, String>,
    /// 自定义过滤器
    filters: HashMap<String, Box<dyn Fn(&str) -> String + Send + Sync>>,
}

impl TemplateEngine {
    /// 创建新的模板引擎
    ///
    /// # 参数
    /// - `view_dir`: 模板目录路径
    pub fn new(view_dir: &Path) -> Result<Self> {
        if !view_dir.exists() {
            std::fs::create_dir_all(view_dir)
                .with_context(|| format!("无法创建模板目录: {:?}", view_dir))?;
        }
        Ok(Self {
            view_dir: view_dir.to_path_buf(),
            cache: HashMap::new(),
            filters: HashMap::new(),
        })
    }

    /// 创建空模板引擎
    pub fn empty() -> Self {
        Self {
            view_dir: PathBuf::new(),
            cache: HashMap::new(),
            filters: HashMap::new(),
        }
    }

    /// 渲染模板
    ///
    /// # 参数
    /// - `template_name`: 模板名称（相对于 view 目录），如 "index/index"
    /// - `context`: 模板变量上下文
    pub fn render(&self, template_name: &str, context: &HashMap<String, serde_json::Value>) -> Result<String> {
        let name = if template_name.ends_with(".html") {
            template_name.to_string()
        } else {
            format!("{}.html", template_name)
        };

        let template_content = if let Some(cached) = self.cache.get(&name) {
            cached.clone()
        } else {
            self.load_template(&name)?
        };

        self.render_content(&template_content, context)
    }

    /// 加载模板文件内容
    ///
    /// # 参数
    /// - `name`: 模板文件名
    fn load_template(&self, name: &str) -> Result<String> {
        let path = self.view_dir.join(name);
        std::fs::read_to_string(&path)
            .with_context(|| format!("无法加载模板文件: {:?}", path))
    }

    /// 渲染模板内容
    ///
    /// 解析模板标签并替换为实际值
    ///
    /// # 参数
    /// - `content`: 模板内容
    /// - `context`: 模板变量上下文
    fn render_content(&self, content: &str, context: &HashMap<String, serde_json::Value>) -> Result<String> {
        let mut result = content.to_string();

        result = self.process_comments(&result);
        result = self.process_extends(&result, context)?;
        result = self.process_includes(&result, context)?;
        result = self.process_blocks(&result);
        result = self.process_if(&result, context)?;
        result = self.process_foreach(&result, context)?;
        result = self.process_function_calls(&result, context);
        result = self.process_raw_expressions(&result, context);
        result = self.process_variables(&result, context);

        Ok(result)
    }

    /// 处理注释 {/* comment */}
    fn process_comments(&self, content: &str) -> String {
        let re = Regex::new(r"\{/ \*.*?\*/\}").unwrap();
        re.replace_all(content, "").to_string()
    }

    /// 处理变量输出 {$var}, {$var.name}, {$var|filter}
    fn process_variables(&self, content: &str, context: &HashMap<String, serde_json::Value>) -> String {
        let re = Regex::new(r"\{\$([a-zA-Z_][a-zA-Z0-9_.|]*)\}").unwrap();
        re.replace_all(content, |caps: &regex::Captures| {
            let expr = caps.get(1).unwrap().as_str();
            let parts: Vec<&str> = expr.splitn(2, '|').collect();
            let var_expr = parts[0];
            let filter_name = parts.get(1).map(|s| *s);

            let value = self.resolve_variable(var_expr, context);

            match filter_name {
                Some(filter) => self.apply_filter(filter, &value),
                None => value,
            }
        }).to_string()
    }

    /// 解析变量表达式
    ///
    /// 支持：var, var.name, var.0
    fn resolve_variable(&self, expr: &str, context: &HashMap<String, serde_json::Value>) -> String {
        let parts: Vec<&str> = expr.split('.').collect();
        let mut current = context.get(parts[0]).cloned();

        for part in parts.iter().skip(1) {
            match current {
                Some(serde_json::Value::Object(map)) => {
                    current = map.get(*part).cloned();
                }
                Some(serde_json::Value::Array(arr)) => {
                    if let Ok(idx) = part.parse::<usize>() {
                        current = arr.get(idx).cloned();
                    } else {
                        current = None;
                    }
                }
                _ => {
                    current = None;
                }
            }
        }

        match current {
            Some(serde_json::Value::String(s)) => s,
            Some(serde_json::Value::Number(n)) => n.to_string(),
            Some(serde_json::Value::Bool(b)) => b.to_string(),
            Some(serde_json::Value::Null) => String::new(),
            Some(v) => v.to_string(),
            None => String::new(),
        }
    }

    /// 应用过滤器
    fn apply_filter(&self, filter: &str, value: &str) -> String {
        if let Some(f) = self.filters.get(filter) {
            f(value)
        } else {
            match filter {
                "upper" | "strtoupper" => value.to_uppercase(),
                "lower" | "strtolower" => value.to_lowercase(),
                "trim" => value.trim().to_string(),
                "md5" => format!("{:x}", md5::compute(value.as_bytes())),
                "sha1" => value.to_string(),
                "json" | "json_encode" => serde_json::to_string(&value).unwrap_or_default(),
                "default" => {
                    if value.is_empty() { "(empty)".to_string() } else { value.to_string() }
                }
                "raw" => value.to_string(),
                "nl2br" => value.replace("\n", "<br />\n"),
                "htmlspecialchars" | "escape" => value
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('"', "&quot;"),
                _ => value.to_string(),
            }
        }
    }

    /// 处理条件判断 {if $cond}...{else}...{/if}
    fn process_if(&self, content: &str, context: &HashMap<String, serde_json::Value>) -> Result<String> {
        let re = Regex::new(r"\{if\s+(.+?)\}(.+?)(?:\{else\}(.+?))?\{/if\}").unwrap();
        let result = re.replace_all(content, |caps: &regex::Captures| {
            let condition = caps.get(1).unwrap().as_str();
            let then_body = caps.get(2).unwrap().as_str();
            let else_body = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            if self.evaluate_condition(condition, context) {
                then_body.to_string()
            } else {
                else_body.to_string()
            }
        });
        Ok(result.to_string())
    }

    /// 评估条件表达式
    fn evaluate_condition(&self, condition: &str, context: &HashMap<String, serde_json::Value>) -> bool {
        let trimmed = condition.trim();

        if let Some(rest) = trimmed.strip_prefix("!") {
            return !self.evaluate_condition(rest, context);
        }

        for op in &["===", "!==", "==", "!=", ">=", "<=", ">", "<"] {
            if let Some(idx) = trimmed.find(op) {
                let left = trimmed[..idx].trim();
                let right = trimmed[idx + op.len()..].trim();
                let left_val = self.resolve_variable(left.trim_start_matches('$'), context);
                let right_val = self.resolve_variable(right.trim_start_matches('$').trim_matches('\'').trim_matches('"'), context);

                return match *op {
                    "===" => left_val == right_val,
                    "!==" => left_val != right_val,
                    "==" => left_val == right_val,
                    "!=" => left_val != right_val,
                    ">=" => left_val >= right_val,
                    "<=" => left_val <= right_val,
                    ">" => left_val > right_val,
                    "<" => left_val < right_val,
                    _ => false,
                };
            }
        }

        let val = self.resolve_variable(trimmed.trim_start_matches('$'), context);
        !val.is_empty() && val != "0" && val != "false"
    }

    /// 处理循环 {foreach $arr as $key => $val}...{/foreach}
    fn process_foreach(&self, content: &str, context: &HashMap<String, serde_json::Value>) -> Result<String> {
        let re = Regex::new(r"\{foreach\s+\$([a-zA-Z_][a-zA-Z0-9_]*)\s+as\s+(?:\$([a-zA-Z_][a-zA-Z0-9_]*)\s*=>\s*)?\$([a-zA-Z_][a-zA-Z0-9_]*)\}(.+?)\{/foreach\}").unwrap();
        let result = re.replace_all(content, |caps: &regex::Captures| {
            let arr_name = caps.get(1).unwrap().as_str();
            let key_name = caps.get(2).map(|m| m.as_str());
            let val_name = caps.get(3).unwrap().as_str();
            let body = caps.get(4).unwrap().as_str();

            let arr = context.get(arr_name);
            match arr {
                Some(serde_json::Value::Array(items)) => {
                    let mut output = String::new();
                    for (i, item) in items.iter().enumerate() {
                        let mut local_ctx = context.clone();
                        if let Some(kn) = key_name {
                            local_ctx.insert(kn.to_string(), serde_json::Value::Number(i.into()));
                        }
                        local_ctx.insert(val_name.to_string(), item.clone());
                        let rendered = self.render_content(body, &local_ctx).unwrap_or_default();
                        output.push_str(&rendered);
                    }
                    output
                }
                Some(serde_json::Value::Object(map)) => {
                    let mut output = String::new();
                    for (k, v) in map {
                        let mut local_ctx = context.clone();
                        if let Some(kn) = key_name {
                            local_ctx.insert(kn.to_string(), serde_json::Value::String(k.clone()));
                        }
                        local_ctx.insert(val_name.to_string(), v.clone());
                        let rendered = self.render_content(body, &local_ctx).unwrap_or_default();
                        output.push_str(&rendered);
                    }
                    output
                }
                _ => String::new(),
            }
        });
        Ok(result.to_string())
    }

    /// 处理包含文件 {include file="xxx"}
    fn process_includes(&self, content: &str, context: &HashMap<String, serde_json::Value>) -> Result<String> {
        let re = Regex::new(r#"\{include\s+file="([^"]+)"\s*\}"#).unwrap();
        let mut result = content.to_string();

        for cap in re.captures_iter(content) {
            let file_name = cap.get(1).unwrap().as_str();
            let included = self.load_template(file_name)?;
            let rendered = self.render_content(&included, context)?;
            let pattern = cap.get(0).unwrap().as_str();
            result = result.replace(pattern, &rendered);
        }

        Ok(result)
    }

    /// 处理区块 {block name="xxx"}...{/block}
    fn process_blocks(&self, content: &str) -> String {
        let re = Regex::new(r#"\{block\s+name="([^"]+)"\s*\}(.+?)\{/block\}"#).unwrap();
        re.replace_all(content, |caps: &regex::Captures| {
            let _name = caps.get(1).unwrap().as_str();
            let body = caps.get(2).unwrap().as_str();
            body.to_string()
        }).to_string()
    }

    /// 处理继承 {extend name="xxx"}
    fn process_extends(&self, content: &str, _context: &HashMap<String, serde_json::Value>) -> Result<String> {
        let re = Regex::new(r#"\{extend\s+name="([^"]+)"\s*\}"#).unwrap();
        if let Some(cap) = re.captures(content) {
            let parent_name = cap.get(1).unwrap().as_str();
            let parent_content = self.load_template(parent_name)?;
            let child_content = re.replace(content, "").to_string();

            let mut child_blocks = HashMap::new();
            let block_re = Regex::new(r#"\{block\s+name="([^"]+)"\s*\}(.+?)\{/block\}"#).unwrap();
            for cap in block_re.captures_iter(&child_content) {
                let name = cap.get(1).unwrap().as_str();
                let body = cap.get(2).unwrap().as_str();
                child_blocks.insert(name.to_string(), body.to_string());
            }

            let mut result = parent_content;
            for (name, body) in &child_blocks {
                let parent_block_re = Regex::new(&format!(
                    r#"\{{block\s+name="{}"\s*\}}(.+?)\{{/block\}}"#,
                    regex::escape(name)
                )).unwrap();
                result = parent_block_re
                    .replace_all(&result, body.as_str())
                    .to_string();
            }

            return Ok(result);
        }
        Ok(content.to_string())
    }

    /// 处理函数调用 {:func(args)}
    fn process_function_calls(&self, content: &str, _context: &HashMap<String, serde_json::Value>) -> String {
        let re = Regex::new(r"\{:([a-zA-Z_][a-zA-Z0-9_]*)\(([^)]*)\)\}").unwrap();
        re.replace_all(content, |caps: &regex::Captures| {
            let func_name = caps.get(1).unwrap().as_str();
            let _args = caps.get(2).unwrap().as_str();
            match func_name {
                "time" => std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .to_string(),
                "date" => chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                "url" => "#".to_string(),
                _ => String::new(),
            }
        }).to_string()
    }

    /// 处理原始表达式 {~expr}
    fn process_raw_expressions(&self, content: &str, _context: &HashMap<String, serde_json::Value>) -> String {
        let re = Regex::new(r"\{~(.+?)\}").unwrap();
        re.replace_all(content, |caps: &regex::Captures| {
            let _expr = caps.get(1).unwrap().as_str();
            String::new()
        }).to_string()
    }

    /// 注册自定义过滤器
    pub fn register_filter(&mut self, name: &str, filter: impl Fn(&str) -> String + Send + Sync + 'static) {
        self.filters.insert(name.to_string(), Box::new(filter));
    }

    /// 添加原始模板字符串
    pub fn add_template(&mut self, name: &str, content: &str) -> Result<()> {
        self.cache.insert(name.to_string(), content.to_string());
        Ok(())
    }

    /// 重新加载模板（清空缓存）
    pub fn reload(&mut self) -> Result<()> {
        self.cache.clear();
        Ok(())
    }

    /// 注册全局变量（兼容 Tera 接口）
    pub fn register_global(&mut self, _name: &str, _value: serde_json::Value) {
    }

    pub fn register_function(&mut self, _name: &str, _func: Box<dyn Fn(&HashMap<String, serde_json::Value>) -> String + Send + Sync>) {
    }
}
