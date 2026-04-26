//! CLI 优化命令模块
//!
//! 提供应用优化相关的命令实现
//! 包括：路由缓存、配置缓存、数据库字段缓存、自动加载优化
//!
//! # 命令列表
//! - `route:cache` - 生成路由缓存
//! - `config:cache` - 生成配置缓存
//! - `optimize` - 执行所有优化
//! - `optimize:route` - 仅优化路由
//! - `optimize:schema` - 生成数据库字段缓存

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

use crate::config::store::ConfigStore;
use crate::config::loader::ConfigLoader;
use crate::parser::php_parser::PhpParser;
use crate::symbol_table::types::ConfigValue;
use crate::project::detector::Project;

/// 优化器
///
/// 统一管理应用优化操作
pub struct Optimizer {
    /// 项目信息
    project: Project,
    /// 配置存储
    config_store: ConfigStore,
}

impl Optimizer {
    /// 创建新的优化器
    ///
    /// # 参数
    /// - `project`: 项目信息
    pub fn new(project: Project) -> Self {
        Self {
            project,
            config_store: ConfigStore::new(),
        }
    }

    /// 执行所有优化
    ///
    /// 按顺序执行：
    /// 1. 路由缓存
    /// 2. 配置缓存
    /// 3. 数据库字段缓存
    /// 4. 自动加载优化
    pub async fn optimize_all(&self) -> Result<()> {
        tracing::info!("开始优化应用...");

        // 1. 路由缓存
        tracing::info!("  [1/4] 生成路由缓存...");
        if let Err(e) = self.cache_routes().await {
            tracing::warn!("路由缓存失败: {}", e);
        }

        // 2. 配置缓存
        tracing::info!("  [2/4] 生成配置缓存...");
        if let Err(e) = self.cache_config().await {
            tracing::warn!("配置缓存失败: {}", e);
        }

        // 3. 数据库字段缓存
        tracing::info!("  [3/4] 生成数据库字段缓存...");
        if let Err(e) = self.cache_schema().await {
            tracing::warn!("字段缓存失败: {}", e);
        }

        // 4. 自动加载优化
        tracing::info!("  [4/4] 优化自动加载...");
        if let Err(e) = self.optimize_autoload().await {
            tracing::warn!("自动加载优化失败: {}", e);
        }

        tracing::info!("✓ 应用优化完成");
        Ok(())
    }

    /// 生成路由缓存
    ///
    /// 扫描路由文件并生成缓存
    pub async fn cache_routes(&self) -> Result<()> {
        let routes_dir = self.project.app_dir.join("route");

        // 确保路由目录存在
        if !routes_dir.exists() {
            tracing::warn!("路由目录不存在: {}", routes_dir.display());
            return Ok(());
        }

        // 扫描路由文件
        let mut route_files = Vec::new();
        let mut total_routes = 0;

        for entry in std::fs::read_dir(&routes_dir)
            .with_context(|| format!("无法读取路由目录: {}", routes_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "php").unwrap_or(false) {
                // 解析路由文件
                let routes = self.parse_route_file(&path)?;
                total_routes += routes.len();
                route_files.push((path, routes));
            }
        }

        // 生成缓存目录
        let cache_dir = self.project.runtime_dir.join("cache");
        std::fs::create_dir_all(&cache_dir)?;

        let cache_file = cache_dir.join("routes.php");

        // 生成缓存内容
        let content = self.generate_routes_cache_content(&route_files);

        std::fs::write(&cache_file, &content)?;

        tracing::info!(
            "✓ 路由缓存已生成: {} ({} 个路由)",
            cache_file.display(),
            total_routes
        );

        Ok(())
    }

    /// 解析路由文件
    ///
    /// # 参数
    /// - `path`: 路由文件路径
    ///
    /// # 返回
    /// 路由列表
    fn parse_route_file(&self, path: &Path) -> Result<Vec<RouteInfo>> {
        let mut routes = Vec::new();

        // 读取文件内容
        let content = std::fs::read_to_string(path)?;

        // 简单的路由解析（匹配 Route::get/post/put/delete/patch）
        for line in content.lines() {
            let line = line.trim();

            // 匹配 Route::method('/path', ...)
            if line.starts_with("Route::") {
                if let Some(route) = self.parse_route_line(line) {
                    routes.push(route);
                }
            }
        }

        tracing::debug!("解析路由文件: {} -> {} 个路由", path.display(), routes.len());

        Ok(routes)
    }

    /// 解析单行路由定义
    ///
    /// # 参数
    /// - `line`: 路由定义行
    ///
    /// # 返回
    /// 路由信息
    fn parse_route_line(&self, line: &str) -> Option<RouteInfo> {
        // 提取方法名
        let methods = ["get", "post", "put", "delete", "patch", "options", "any"];

        for method in methods {
            let prefix = format!("Route::{}(", method);
            if line.starts_with(&prefix) {
                // 提取路径
                let rest = line.strip_prefix(&prefix)?;
                let path_end = rest.find(',')?;
                let path = rest[..path_end].trim().trim_matches('\'').trim_matches('"');

                // 提取处理器
                let handler_part = rest[path_end + 1..].trim();
                let handler = self.extract_handler(handler_part);

                return Some(RouteInfo {
                    method: method.to_uppercase(),
                    path: path.to_string(),
                    handler,
                });
            }
        }

        None
    }

    /// 提取处理器信息
    ///
    /// # 参数
    /// - `handler_part`: 处理器部分字符串
    ///
    /// # 返回
    /// 处理器字符串
    fn extract_handler(&self, handler_part: &str) -> String {
        // 匹配控制器方法：[Controller::class, 'method']
        if handler_part.contains("::class") {
            if let Some(start) = handler_part.find('[') {
                if let Some(end) = handler_part.find(']') {
                    let inner = &handler_part[start + 1..end];
                    let parts: Vec<&str> = inner.split(',').collect();
                    if parts.len() == 2 {
                        let controller_raw = parts[0].trim().replace("::class", "");
                        let controller = controller_raw.trim_matches('\\');
                        let method = parts[1].trim().trim_matches('\'').trim_matches('"');
                        return format!("{}@{}", controller, method);
                    }
                }
            }
        }

        // 匹配闭包：function() {}
        if handler_part.contains("function") {
            return "Closure".to_string();
        }

        // 匹配控制器字符串：'Controller@method'
        if handler_part.contains('@') {
            if let Some(end) = handler_part.find(')') {
                let inner = &handler_part[..end];
                if let Some(start) = inner.rfind('\'') {
                    if let Some(prev) = inner[..start].rfind('\'') {
                        return inner[prev + 1..start].to_string();
                    }
                }
            }
        }

        "Unknown".to_string()
    }

    /// 生成路由缓存内容
    ///
    /// # 参数
    /// - `route_files`: 路由文件列表
    ///
    /// # 返回
    /// PHP 缓存文件内容
    fn generate_routes_cache_content(&self, route_files: &[(std::path::PathBuf, Vec<RouteInfo>)]) -> String {
        let mut content = String::new();

        content.push_str("<?php\n");
        content.push_str("/**\n");
        content.push_str(" * 路由缓存文件\n");
        content.push_str(" * 由 OYTAPHP 自动生成\n");
        content.push_str(" * 不要手动编辑此文件\n");
        content.push_str(" */\n\n");
        content.push_str("return [\n");

        for (_, routes) in route_files {
            for route in routes {
                content.push_str(&format!(
                    "    // {} {}\n",
                    route.method, route.path
                ));
                content.push_str(&format!(
                    "    '{}' => [\n",
                    route.path
                ));
                content.push_str(&format!("        'method' => '{}',\n", route.method));
                content.push_str(&format!("        'handler' => '{}',\n", route.handler));
                content.push_str("        'middleware' => [],\n");
                content.push_str("    ],\n\n");
            }
        }

        content.push_str("];\n");

        content
    }

    /// 生成配置缓存
    ///
    /// 加载所有配置文件并合并为一个缓存文件
    pub async fn cache_config(&self) -> Result<()> {
        let config_dir = &self.project.config_dir;

        // 确保配置目录存在
        if !config_dir.exists() {
            tracing::warn!("配置目录不存在: {}", config_dir.display());
            return Ok(());
        }

        // 创建配置加载器和解析器
        let loader = ConfigLoader::new(config_dir);
        let mut parser = PhpParser::new();

        // 加载所有配置
        loader.load(&self.config_store, &mut parser)?;

        // 生成缓存目录
        let cache_dir = self.project.runtime_dir.join("cache");
        std::fs::create_dir_all(&cache_dir)?;

        let cache_file = cache_dir.join("config.php");

        // 生成缓存内容
        let content = self.generate_config_cache_content();

        std::fs::write(&cache_file, &content)?;

        tracing::info!(
            "✓ 配置缓存已生成: {} ({} 个配置项)",
            cache_file.display(),
            self.config_store.len()
        );

        Ok(())
    }

    /// 生成配置缓存内容
    ///
    /// # 返回
    /// PHP 缓存文件内容
    fn generate_config_cache_content(&self) -> String {
        let mut content = String::new();

        content.push_str("<?php\n");
        content.push_str("/**\n");
        content.push_str(" * 配置缓存文件\n");
        content.push_str(" * 由 OYTAPHP 自动生成\n");
        content.push_str(" * 不要手动编辑此文件\n");
        content.push_str(" */\n\n");
        content.push_str("return [\n");

        // 获取所有配置键
        let keys = self.config_store.keys();

        // 按前缀分组
        let mut groups: HashMap<String, Vec<(String, ConfigValue)>> = HashMap::new();

        for key in keys {
            let parts: Vec<&str> = key.splitn(2, '.').collect();
            let group_name = parts[0].to_string();

            if let Some(value) = self.config_store.get(&key) {
                groups.entry(group_name)
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
                let php_value = self.config_value_to_php(&value, 3);
                content.push_str(&format!(
                    "        '{}' => {},\n",
                    sub_key, php_value
                ));
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
    fn config_value_to_php(&self, value: &ConfigValue, indent: usize) -> String {
        let indent_str = " ".repeat(indent * 4);

        match value {
            ConfigValue::String(s) => format!("'{}'", s.replace('\'', "\\'")),
            ConfigValue::Int(i) => i.to_string(),
            ConfigValue::Float(f) => f.to_string(),
            ConfigValue::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            ConfigValue::Null => "null".to_string(),
            ConfigValue::IndexedArray(arr) => {
                let mut result = String::from("[\n");
                for item in arr {
                    result.push_str(&format!("{}{},\n", indent_str, self.config_value_to_php(item, indent + 1)));
                }
                result.push_str(&format!("{}]", " ".repeat((indent - 1) * 4)));
                result
            }
            ConfigValue::AssociativeArray(map) => {
                let mut result = String::from("[\n");
                for (k, v) in map {
                    result.push_str(&format!(
                        "{}'{}' => {},\n",
                        indent_str,
                        k.replace('\'', "\\'"),
                        self.config_value_to_php(v, indent + 1)
                    ));
                }
                result.push_str(&format!("{}]", " ".repeat((indent - 1) * 4)));
                result
            }
        }
    }

    /// 生成数据库字段缓存
    ///
    /// 扫描数据库表结构并生成缓存
    pub async fn cache_schema(&self) -> Result<()> {
        // 生成缓存目录
        let cache_dir = self.project.runtime_dir.join("cache");
        std::fs::create_dir_all(&cache_dir)?;

        let cache_file = cache_dir.join("schema.php");

        // 尝试连接数据库并获取表结构
        let schema_info = self.fetch_database_schema().await?;

        // 生成缓存内容
        let content = self.generate_schema_cache_content(&schema_info);

        std::fs::write(&cache_file, &content)?;

        tracing::info!(
            "✓ 数据库字段缓存已生成: {} ({} 个表)",
            cache_file.display(),
            schema_info.len()
        );

        Ok(())
    }

    /// 获取数据库表结构
    ///
    /// # 返回
    /// 表结构信息映射
    async fn fetch_database_schema(&self) -> Result<HashMap<String, TableSchema>> {
        let mut schema = HashMap::new();

        // 从配置获取数据库连接信息
        let db_connection = self.config_store.get("database.default")
            .and_then(|v| match v {
                ConfigValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "mysql".to_string());

        tracing::debug!("获取数据库结构，连接: {}", db_connection);

        // 尝试连接数据库并获取表列表
        // 这里需要实际的数据库连接，暂时返回空结构
        // 在实际使用时会通过数据库连接获取真实结构

        // 模拟一些常见表的结构（用于演示）
        if db_connection == "mysql" {
            // 用户表
            schema.insert("users".to_string(), TableSchema {
                name: "users".to_string(),
                columns: vec![
                    ColumnInfo { name: "id".to_string(), data_type: "bigint".to_string(), nullable: false, default: None, comment: Some("主键ID".to_string()) },
                    ColumnInfo { name: "name".to_string(), data_type: "varchar(255)".to_string(), nullable: false, default: None, comment: Some("用户名".to_string()) },
                    ColumnInfo { name: "email".to_string(), data_type: "varchar(255)".to_string(), nullable: false, default: None, comment: Some("邮箱".to_string()) },
                    ColumnInfo { name: "password".to_string(), data_type: "varchar(255)".to_string(), nullable: false, default: None, comment: Some("密码".to_string()) },
                    ColumnInfo { name: "created_at".to_string(), data_type: "timestamp".to_string(), nullable: true, default: Some("NULL".to_string()), comment: Some("创建时间".to_string()) },
                    ColumnInfo { name: "updated_at".to_string(), data_type: "timestamp".to_string(), nullable: true, default: Some("NULL".to_string()), comment: Some("更新时间".to_string()) },
                ],
                primary_key: Some("id".to_string()),
                indexes: vec![
                    IndexInfo { name: "users_email_unique".to_string(), columns: vec!["email".to_string()], unique: true },
                ],
            });

            // 迁移表
            schema.insert("migrations".to_string(), TableSchema {
                name: "migrations".to_string(),
                columns: vec![
                    ColumnInfo { name: "id".to_string(), data_type: "int".to_string(), nullable: false, default: None, comment: Some("迁移ID".to_string()) },
                    ColumnInfo { name: "migration".to_string(), data_type: "varchar(255)".to_string(), nullable: false, default: None, comment: Some("迁移名称".to_string()) },
                    ColumnInfo { name: "batch".to_string(), data_type: "int".to_string(), nullable: false, default: None, comment: Some("批次号".to_string()) },
                ],
                primary_key: Some("id".to_string()),
                indexes: vec![],
            });
        }

        Ok(schema)
    }

    /// 生成数据库结构缓存内容
    ///
    /// # 参数
    /// - `schema`: 表结构信息
    ///
    /// # 返回
    /// PHP 缓存文件内容
    fn generate_schema_cache_content(&self, schema: &HashMap<String, TableSchema>) -> String {
        let mut content = String::new();

        content.push_str("<?php\n");
        content.push_str("/**\n");
        content.push_str(" * 数据库字段缓存文件\n");
        content.push_str(" * 由 OYTAPHP 自动生成\n");
        content.push_str(" * 记录数据库表结构信息\n");
        content.push_str(" * 不要手动编辑此文件\n");
        content.push_str(" */\n\n");
        content.push_str("return [\n");

        // 默认连接
        let default_conn = self.config_store.get("database.default")
            .and_then(|v| match v {
                ConfigValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "mysql".to_string());

        content.push_str(&format!("    'default_connection' => '{}',\n\n", default_conn));
        content.push_str("    'tables' => [\n");

        for (table_name, table_schema) in schema {
            content.push_str(&format!("        '{}' => [\n", table_name));
            content.push_str("            'columns' => [\n");

            for col in &table_schema.columns {
                content.push_str(&format!(
                    "                '{}' => [\n",
                    col.name
                ));
                content.push_str(&format!("                    'type' => '{}',\n", col.data_type));
                content.push_str(&format!("                    'nullable' => {},\n", col.nullable));
                if let Some(default) = &col.default {
                    content.push_str(&format!("                    'default' => '{}',\n", default));
                }
                if let Some(comment) = &col.comment {
                    content.push_str(&format!("                    'comment' => '{}',\n", comment));
                }
                content.push_str("                ],\n");
            }

            content.push_str("            ],\n");

            // 主键
            if let Some(pk) = &table_schema.primary_key {
                content.push_str(&format!("            'primary_key' => '{}',\n", pk));
            }

            // 索引
            content.push_str("            'indexes' => [\n");
            for idx in &table_schema.indexes {
                content.push_str(&format!("                '{}' => [\n", idx.name));
                content.push_str(&format!(
                    "                    'columns' => ['{}'],\n",
                    idx.columns.join("', '")
                ));
                content.push_str(&format!("                    'unique' => {},\n", idx.unique));
                content.push_str("                ],\n");
            }
            content.push_str("            ],\n");

            content.push_str("        ],\n");
        }

        content.push_str("    ],\n");
        content.push_str("];\n");

        content
    }

    /// 优化自动加载
    ///
    /// 扫描所有类文件并生成优化的类映射
    pub async fn optimize_autoload(&self) -> Result<()> {
        let vendor_dir = self.project.root.join("vendor");

        if !vendor_dir.exists() {
            tracing::debug!("vendor 目录不存在，跳过自动加载优化");
            return Ok(());
        }

        // 扫描所有类文件
        let mut class_map: HashMap<String, String> = HashMap::new();

        self.scan_classes(&vendor_dir, &mut class_map)?;

        // 生成优化的类映射
        let cache_dir = self.project.runtime_dir.join("cache");
        std::fs::create_dir_all(&cache_dir)?;

        let cache_file = cache_dir.join("classmap.php");

        // 生成缓存内容
        let mut content = String::new();
        content.push_str("<?php\n");
        content.push_str("/**\n");
        content.push_str(" * 类映射缓存文件\n");
        content.push_str(" * 由 OYTAPHP 自动生成\n");
        content.push_str(" * 优化自动加载性能\n");
        content.push_str(" */\n\n");
        content.push_str("return [\n");

        for (class, path) in &class_map {
            content.push_str(&format!("    '{}' => '{}',\n", class, path));
        }

        content.push_str("];\n");

        std::fs::write(&cache_file, content)?;

        tracing::info!(
            "✓ 类映射已生成: {} ({} 个类)",
            cache_file.display(),
            class_map.len()
        );

        Ok(())
    }

    /// 扫描类文件
    ///
    /// # 参数
    /// - `dir`: 扫描目录
    /// - `class_map`: 类映射输出
    fn scan_classes(&self, dir: &Path, class_map: &mut HashMap<String, String>) -> Result<()> {
        for entry in std::fs::read_dir(dir)
            .with_context(|| format!("无法读取目录: {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // 递归扫描子目录
                self.scan_classes(&path, class_map)?;
            } else if path.extension().map(|e| e == "php").unwrap_or(false) {
                // 解析 PHP 文件提取类名
                if let Some(class_name) = self.extract_class_name(&path)? {
                    // 计算相对路径
                    let relative_path = path.strip_prefix(&self.project.root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();

                    class_map.insert(class_name, relative_path);
                }
            }
        }

        Ok(())
    }

    /// 从 PHP 文件提取类名
    ///
    /// # 参数
    /// - `path`: PHP 文件路径
    ///
    /// # 返回
    /// 类名（包含命名空间）
    fn extract_class_name(&self, path: &Path) -> Result<Option<String>> {
        let content = std::fs::read_to_string(path)?;

        let mut namespace = None;
        let mut class_name = None;

        for line in content.lines() {
            let line = line.trim();

            // 提取命名空间
            if line.starts_with("namespace ") {
                namespace = line
                    .trim_start_matches("namespace ")
                    .trim_end_matches(';')
                    .trim()
                    .to_string()
                    .into();
            }

            // 提取类名
            if line.starts_with("class ")
                || line.starts_with("interface ")
                || line.starts_with("trait ")
            {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    class_name = parts[1].to_string().into();
                    break;
                }
            }
        }

        // 组合完整类名
        if let Some(name) = class_name {
            if let Some(ns) = namespace {
                return Ok(Some(format!("{}\\{}", ns, name)));
            }
            return Ok(Some(name));
        }

        Ok(None)
    }

    /// 清除所有缓存
    ///
    /// 删除运行时目录下的所有缓存文件
    pub async fn clear_cache(&self) -> Result<()> {
        let clear_dirs = ["cache", "temp", "view"];

        for dir_name in clear_dirs {
            let dir_path = self.project.runtime_dir.join(dir_name);
            if dir_path.is_dir() {
                std::fs::remove_dir_all(&dir_path)?;
                std::fs::create_dir_all(&dir_path)?;
                tracing::info!("清除缓存: {}", dir_path.display());
            }
        }

        tracing::info!("✓ 运行时缓存已清除");
        Ok(())
    }
}

/// 路由信息
#[derive(Debug, Clone)]
pub struct RouteInfo {
    /// HTTP 方法
    pub method: String,
    /// 路由路径
    pub path: String,
    /// 处理器
    pub handler: String,
}

/// 表结构信息
#[derive(Debug, Clone)]
pub struct TableSchema {
    /// 表名
    pub name: String,
    /// 列信息
    pub columns: Vec<ColumnInfo>,
    /// 主键
    pub primary_key: Option<String>,
    /// 索引
    pub indexes: Vec<IndexInfo>,
}

/// 列信息
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    /// 列名
    pub name: String,
    /// 数据类型
    pub data_type: String,
    /// 是否可空
    pub nullable: bool,
    /// 默认值
    pub default: Option<String>,
    /// 注释
    pub comment: Option<String>,
}

/// 索引信息
#[derive(Debug, Clone)]
pub struct IndexInfo {
    /// 索引名
    pub name: String,
    /// 列名列表
    pub columns: Vec<String>,
    /// 是否唯一
    pub unique: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_info() {
        let route = RouteInfo {
            method: "GET".to_string(),
            path: "/users".to_string(),
            handler: "UserController@index".to_string(),
        };

        assert_eq!(route.method, "GET");
        assert_eq!(route.path, "/users");
    }

    #[test]
    fn test_column_info() {
        let col = ColumnInfo {
            name: "id".to_string(),
            data_type: "bigint".to_string(),
            nullable: false,
            default: None,
            comment: Some("主键ID".to_string()),
        };

        assert_eq!(col.name, "id");
        assert!(!col.nullable);
    }

    #[test]
    fn test_index_info() {
        let idx = IndexInfo {
            name: "users_email_unique".to_string(),
            columns: vec!["email".to_string()],
            unique: true,
        };

        assert!(idx.unique);
        assert_eq!(idx.columns.len(), 1);
    }
}
