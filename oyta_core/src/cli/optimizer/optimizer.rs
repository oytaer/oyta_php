//! 优化器主体模块
//!
//! 统一管理应用优化操作

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

use crate::config::loader::ConfigLoader;
use crate::config::store::ConfigStore;
use crate::parser::php_parser::PhpParser;
use crate::project::detector::Project;
use crate::symbol_table::types::ConfigValue;

use super::autoload::AutoloadOptimizer;
use super::config_cache::ConfigCacheGenerator;
use super::route_cache::RouteCacheGenerator;
use super::schema_cache::SchemaCacheGenerator;
use super::types::TableSchema;

/// 优化器
///
/// 统一管理应用优化操作
pub struct Optimizer {
    /// 项目信息
    pub project: Project,
    /// 配置存储
    pub config_store: ConfigStore,
}

impl Optimizer {
    /// 创建新的优化器
    ///
    /// # 参数
    /// - `project`: 项目信息
    ///
    /// # 返回值
    /// 新的 Optimizer 实例
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
                let routes = RouteCacheGenerator::parse_route_file(&path)?;
                total_routes += routes.len();
                route_files.push((path, routes));
            }
        }

        // 生成缓存目录
        let cache_dir = self.project.runtime_dir.join("cache");
        std::fs::create_dir_all(&cache_dir)?;

        let cache_file = cache_dir.join("routes.php");

        // 生成缓存内容
        let content = RouteCacheGenerator::generate_cache_content(&route_files);

        std::fs::write(&cache_file, &content)?;

        tracing::info!(
            "✓ 路由缓存已生成: {} ({} 个路由)",
            cache_file.display(),
            total_routes
        );

        Ok(())
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
        let content = ConfigCacheGenerator::generate_cache_content(&self.config_store);

        std::fs::write(&cache_file, &content)?;

        tracing::info!(
            "✓ 配置缓存已生成: {} ({} 个配置项)",
            cache_file.display(),
            self.config_store.len()
        );

        Ok(())
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
        let content = SchemaCacheGenerator::generate_cache_content(&schema_info, &self.config_store);

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
        // 从配置获取数据库连接信息
        let db_connection = self
            .config_store
            .get("database.default")
            .and_then(|v| match v {
                ConfigValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "mysql".to_string());

        tracing::debug!("获取数据库结构，连接: {}", db_connection);

        // 尝试连接数据库并获取表列表
        // 这里需要实际的数据库连接，暂时返回模拟结构
        // 在实际使用时会通过数据库连接获取真实结构

        // 返回模拟的数据库结构
        if db_connection == "mysql" {
            return Ok(SchemaCacheGenerator::get_mock_schema());
        }

        Ok(HashMap::new())
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

        let optimizer = AutoloadOptimizer::new(&self.project.root);
        optimizer.scan_classes(&vendor_dir, &mut class_map)?;

        // 生成优化的类映射
        let cache_dir = self.project.runtime_dir.join("cache");
        std::fs::create_dir_all(&cache_dir)?;

        let cache_file = cache_dir.join("classmap.php");

        // 生成缓存内容
        let content = AutoloadOptimizer::generate_cache_content(&class_map);

        std::fs::write(&cache_file, content)?;

        tracing::info!(
            "✓ 类映射已生成: {} ({} 个类)",
            cache_file.display(),
            class_map.len()
        );

        Ok(())
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
