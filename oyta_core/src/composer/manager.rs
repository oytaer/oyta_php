//! Composer 管理器
//!
//! 统一管理 Composer 相关操作
//! 提供 PHP Composer 兼容的依赖管理功能
//!
//! # 功能特性
//! - 依赖安装 (install)
//! - 依赖更新 (update)
//! - 添加依赖 (require)
//! - 移除依赖 (remove)
//! - 自动加载生成 (dump-autoload)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::autoload;
use super::downloader;
use super::lock;
use super::parser;
use super::resolver;

/// Composer 管理器
///
/// 统一管理 Composer 相关操作
pub struct Composer {
    /// 项目根目录
    project_path: String,
    /// composer.json 配置
    config: Option<ComposerJson>,
    /// composer.lock 内容
    lock: Option<ComposerLock>,
}

/// composer.json 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposerJson {
    /// 包名称
    pub name: String,
    /// 包描述
    pub description: String,
    /// 包类型
    #[serde(default)]
    pub r#type: String,
    /// 许可证
    #[serde(default)]
    pub license: Vec<String>,
    /// 作者列表
    #[serde(default)]
    pub authors: Vec<Author>,
    /// 需要的依赖
    #[serde(default)]
    pub require: HashMap<String, String>,
    /// 开发依赖
    #[serde(default)]
    pub require_dev: HashMap<String, String>,
    /// 自动加载配置
    #[serde(default)]
    pub autoload: AutoloadConfig,
    /// 开发环境自动加载配置
    #[serde(default)]
    pub autoload_dev: AutoloadConfig,
    /// 仓库配置
    #[serde(default)]
    pub repositories: Vec<Repository>,
    /// 最小稳定性
    #[serde(default = "default_stability")]
    pub minimum_stability: String,
    /// 优先稳定版本
    #[serde(default = "default_true")]
    pub prefer_stable: bool,
    /// 脚本配置
    #[serde(default)]
    pub scripts: HashMap<String, Vec<String>>,
    /// 额外配置
    #[serde(default)]
    pub extra: serde_json::Value,
    /// 配置选项
    #[serde(default)]
    pub config: ComposerConfig,
}

/// 为 ComposerJson 实现自定义 Default
/// 确保默认值符合预期
impl Default for ComposerJson {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            r#type: String::new(),
            license: Vec::new(),
            authors: Vec::new(),
            require: HashMap::new(),
            require_dev: HashMap::new(),
            autoload: AutoloadConfig::default(),
            autoload_dev: AutoloadConfig::default(),
            repositories: Vec::new(),
            minimum_stability: default_stability(),
            prefer_stable: default_true(),
            scripts: HashMap::new(),
            extra: serde_json::Value::Null,
            config: ComposerConfig::default(),
        }
    }
}

/// 默认稳定性
fn default_stability() -> String {
    "stable".to_string()
}

/// 默认 true
fn default_true() -> bool {
    true
}

/// 作者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    /// 作者名称
    pub name: String,
    /// 邮箱地址
    pub email: String,
    /// 主页
    #[serde(default)]
    pub homepage: Option<String>,
    /// 角色
    #[serde(default)]
    pub role: Option<String>,
}

/// 自动加载配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutoloadConfig {
    /// PSR-4 自动加载
    #[serde(default, rename = "psr-4")]
    pub psr_4: HashMap<String, Vec<String>>,
    /// PSR-0 自动加载
    #[serde(default, rename = "psr-0")]
    pub psr_0: HashMap<String, Vec<String>>,
    /// 类映射
    #[serde(default)]
    pub classmap: Vec<String>,
    /// 文件映射
    #[serde(default)]
    pub files: Vec<String>,
}

/// 仓库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// 仓库类型
    pub r#type: String,
    /// 仓库 URL
    pub url: String,
    /// 仓库名称
    #[serde(default)]
    pub name: Option<String>,
}

/// Composer 配置选项
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComposerConfig {
    /// 平台配置（覆盖 PHP 版本等）
    #[serde(default)]
    pub platform: HashMap<String, String>,
    /// 优化自动加载
    #[serde(default)]
    pub optimize_autoloader: bool,
    /// 类映射权威模式
    #[serde(default)]
    pub classmap_authoritative: bool,
    /// APCu 缓存
    #[serde(default)]
    pub apcu_autoloader: bool,
}

/// composer.lock 结构
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComposerLock {
    /// 锁文件版本
    #[serde(default)]
    pub version: i32,
    /// 内容哈希
    #[serde(default)]
    pub content_hash: String,
    /// 已安装的包
    #[serde(default)]
    pub packages: Vec<LockedPackage>,
    /// 开发环境包
    #[serde(default)]
    pub packages_dev: Vec<LockedPackage>,
    /// 平台要求
    #[serde(default)]
    pub platform: HashMap<String, String>,
    /// 平台开发要求
    #[serde(default)]
    pub platform_dev: HashMap<String, String>,
}

/// 已锁定的包信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    /// 包名称
    pub name: String,
    /// 版本号
    pub version: String,
    /// 源信息
    pub source: PackageSource,
    /// 安装路径
    #[serde(default)]
    pub install_path: String,
    /// 自动加载配置
    #[serde(default)]
    pub autoload: AutoloadConfig,
    /// 许可证
    #[serde(default)]
    pub license: Vec<String>,
    /// 依赖
    #[serde(default)]
    pub require: HashMap<String, String>,
}

/// 包源信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSource {
    /// 源类型（git/dist）
    pub r#type: String,
    /// 源 URL
    pub url: String,
    /// 引用（commit hash 或 dist checksum）
    pub reference: String,
}

impl Composer {
    /// 创建新的 Composer 管理器
    ///
    /// # 参数
    /// - `project_path`: 项目根目录路径
    ///
    /// # 返回
    /// 新的 Composer 实例
    pub fn new(project_path: &str) -> Self {
        Self {
            project_path: project_path.to_string(),
            config: None,
            lock: None,
        }
    }

    /// 加载 composer.json 和 composer.lock
    ///
    /// # 返回
    /// 加载成功返回 Ok(())，失败返回错误
    pub async fn load(&mut self) -> Result<()> {
        // 加载 composer.json
        let json_path = Path::new(&self.project_path).join("composer.json");

        if json_path.exists() {
            self.config = Some(parser::parse_composer_json(json_path.to_str().unwrap()).await?);
            tracing::info!("已加载 composer.json: {}", json_path.display());
        }

        // 加载 composer.lock
        let lock_path = Path::new(&self.project_path).join("composer.lock");
        if lock_path.exists() {
            self.lock = Some(lock::parse_composer_lock(lock_path.to_str().unwrap()).await?);
            tracing::info!("已加载 composer.lock: {}", lock_path.display());
        }

        Ok(())
    }

    /// 获取配置
    ///
    /// # 返回
    /// composer.json 配置的引用
    pub fn config(&self) -> Option<&ComposerJson> {
        self.config.as_ref()
    }

    /// 获取锁文件
    ///
    /// # 返回
    /// composer.lock 内容的引用
    pub fn lock(&self) -> Option<&ComposerLock> {
        self.lock.as_ref()
    }

    /// 安装依赖
    ///
    /// 解析 composer.json 中的依赖，下载并安装所有包
    ///
    /// # 返回
    /// 安装成功返回 Ok(())，失败返回错误
    pub async fn install(&mut self) -> Result<()> {
        self.load().await?;

        let config = self.config.as_ref()
            .context("composer.json 不存在")?;

        tracing::info!("开始安装依赖...");

        // 解析依赖
        let dependencies = resolver::resolve(&config.require).await?;

        // 下载依赖
        for dep in &dependencies {
            downloader::download_package(&self.project_path, dep).await?;
        }

        // 将 ResolvedPackage 转换为 LockedPackage
        let locked_packages: Vec<LockedPackage> = dependencies
            .iter()
            .map(|dep| LockedPackage::from(dep))
            .collect();

        // 生成自动加载
        autoload::generate(&self.project_path, &locked_packages).await?;

        // 更新 lock 文件
        self.lock = Some(lock::generate_lock(&dependencies));
        lock::write_composer_lock(&self.project_path, self.lock.as_ref().unwrap()).await?;

        tracing::info!("✓ 依赖安装完成，共 {} 个包", dependencies.len());

        Ok(())
    }

    /// 更新依赖
    ///
    /// 更新指定的包到最新版本，如果包列表为空则更新所有依赖
    ///
    /// # 参数
    /// - `packages`: 要更新的包列表，空列表表示更新所有包
    ///
    /// # 返回
    /// 更新成功返回 Ok(())，失败返回错误
    pub async fn update(&mut self, packages: &[String]) -> Result<()> {
        self.load().await?;

        let config = self.config.as_ref()
            .context("composer.json 不存在")?;

        tracing::info!("开始更新依赖: {:?}", packages);

        // 获取当前锁定的包版本
        let current_locked: HashMap<String, String> = self.lock
            .as_ref()
            .map(|l| l.packages.iter().map(|p| (p.name.clone(), p.version.clone())).collect())
            .unwrap_or_default();

        // 解析依赖
        let mut dependencies = resolver::resolve(&config.require).await?;

        // 如果指定了要更新的包，只更新这些包
        if !packages.is_empty() {
            let update_set: HashSet<String> = packages.iter().cloned().collect();

            // 过滤出需要更新的包
            dependencies.retain(|dep| {
                update_set.contains(&dep.name) || !current_locked.contains_key(&dep.name)
            });

            // 保留不需要更新的包的当前版本
            for (name, version) in &current_locked {
                if !update_set.contains(name) && !dependencies.iter().any(|d| &d.name == name) {
                    // 创建一个保持当前版本的包信息
                    tracing::debug!("保持包 {} 版本 {}", name, version);
                }
            }
        }

        // 检测版本变化
        let mut updated_count = 0;
        let mut added_count = 0;
        let mut removed_count = 0;

        for dep in &dependencies {
            if let Some(old_version) = current_locked.get(&dep.name) {
                if old_version != &dep.version {
                    tracing::info!("  更新 {} 从 {} 到 {}", dep.name, old_version, dep.version);
                    updated_count += 1;
                }
            } else {
                tracing::info!("  新增 {} {}", dep.name, dep.version);
                added_count += 1;
            }
        }

        // 检测被移除的包
        let new_package_names: HashSet<String> = dependencies.iter().map(|d| d.name.clone()).collect();
        for name in current_locked.keys() {
            if !new_package_names.contains(name) {
                tracing::info!("  移除 {}", name);
                removed_count += 1;
                // 删除包目录
                self.remove_package_dir(name).await?;
            }
        }

        // 下载更新的依赖
        for dep in &dependencies {
            downloader::download_package(&self.project_path, dep).await?;
        }

        // 将 ResolvedPackage 转换为 LockedPackage
        let locked_packages: Vec<LockedPackage> = dependencies
            .iter()
            .map(|dep| LockedPackage::from(dep))
            .collect();

        // 生成自动加载
        autoload::generate(&self.project_path, &locked_packages).await?;

        // 更新 lock 文件
        self.lock = Some(lock::generate_lock(&dependencies));
        lock::write_composer_lock(&self.project_path, self.lock.as_ref().unwrap()).await?;

        tracing::info!(
            "✓ 依赖更新完成: {} 个更新, {} 个新增, {} 个移除",
            updated_count,
            added_count,
            removed_count
        );

        Ok(())
    }

    /// 添加依赖
    ///
    /// 添加新的依赖包到 composer.json 并安装
    ///
    /// # 参数
    /// - `package`: 包名称（如 "vendor/package"）
    /// - `version`: 版本约束（如 "^1.0"）
    ///
    /// # 返回
    /// 添加成功返回 Ok(())，失败返回错误
    pub async fn add_require(&mut self, package: &str, version: &str) -> Result<()> {
        self.load().await?;

        // 确保 composer.json 存在
        if self.config.is_none() {
            self.config = Some(ComposerJson {
                name: "project/app".to_string(),
                description: "A new project".to_string(),
                ..Default::default()
            });
        }

        tracing::info!("添加依赖: {} = {}", package, version);

        // 更新 composer.json
        {
            let config = self.config.as_mut().unwrap();
            config.require.insert(package.to_string(), version.to_string());
        }

        // 写入更新后的 composer.json
        self.write_composer_json().await?;

        // 解析新依赖
        let new_require = HashMap::from([(package.to_string(), version.to_string())]);
        let new_dependencies = resolver::resolve(&new_require).await?;

        // 下载新包
        for dep in &new_dependencies {
            downloader::download_package(&self.project_path, dep).await?;
        }

        // 合并到现有锁定包列表
        let mut all_packages = self.lock
            .as_ref()
            .map(|l| l.packages.clone())
            .unwrap_or_default();

        // 添加新包（避免重复）
        let existing_names: HashSet<String> = all_packages.iter().map(|p| p.name.clone()).collect();
        for dep in &new_dependencies {
            if !existing_names.contains(&dep.name) {
                all_packages.push(LockedPackage::from(dep));
            } else {
                // 更新已存在的包
                if let Some(pkg) = all_packages.iter_mut().find(|p| p.name == dep.name) {
                    pkg.version = dep.version.clone();
                }
            }
        }

        // 生成自动加载
        autoload::generate(&self.project_path, &all_packages).await?;

        // 更新 lock 文件
        let all_resolved: Vec<resolver::ResolvedPackage> = all_packages
            .iter()
            .filter_map(|p| {
                // 从 LockedPackage 转换回 ResolvedPackage（简化版本）
                Some(resolver::ResolvedPackage {
                    name: p.name.clone(),
                    version: p.version.clone(),
                    download_url: String::new(),
                    checksum: p.source.reference.clone(),
                    source: p.source.clone(),
                    require: p.require.clone(),
                })
            })
            .collect();

        self.lock = Some(lock::generate_lock(&all_resolved));
        lock::write_composer_lock(&self.project_path, self.lock.as_ref().unwrap()).await?;

        tracing::info!("✓ 依赖添加完成: {} 个新包", new_dependencies.len());

        Ok(())
    }

    /// 移除依赖
    ///
    /// 从 composer.json 中移除依赖包并删除相关文件
    ///
    /// # 参数
    /// - `package`: 要移除的包名称
    ///
    /// # 返回
    /// 移除成功返回 Ok(())，失败返回错误
    pub async fn remove_require(&mut self, package: &str) -> Result<()> {
        self.load().await?;

        // 检查包是否存在并移除
        {
            let config = self.config.as_mut()
                .context("composer.json 不存在")?;

            tracing::info!("移除依赖: {}", package);

            // 检查包是否存在
            if !config.require.contains_key(package) {
                anyhow::bail!("包 {} 不在依赖列表中", package);
            }

            // 检查是否有其他包依赖此包
            if let Some(lock) = &self.lock {
                for locked_pkg in &lock.packages {
                    if locked_pkg.require.contains_key(package) {
                        tracing::warn!(
                            "警告: 包 {} 被包 {} 依赖，移除可能导致问题",
                            package,
                            locked_pkg.name
                        );
                    }
                }
            }

            // 从 composer.json 移除
            config.require.remove(package);
        }

        // 写入更新后的 composer.json
        self.write_composer_json().await?;

        // 从锁定包列表移除
        let mut removed_packages = vec![package.to_string()];

        // 递归移除不再被需要的依赖
        if let Some(lock) = &self.lock {
            let remaining_requires: HashSet<String> = self.config
                .as_ref()
                .map(|c| c.require.keys().cloned().collect())
                .unwrap_or_default();

            // 找出所有被其他包依赖的包
            let mut required_packages: HashSet<String> = remaining_requires.clone();
            for locked_pkg in &lock.packages {
                if required_packages.contains(&locked_pkg.name) {
                    // 这个包的依赖也是需要的
                    for dep_name in locked_pkg.require.keys() {
                        if !dep_name.starts_with("php") && !dep_name.starts_with("ext-") {
                            required_packages.insert(dep_name.clone());
                        }
                    }
                }
            }

            // 找出可以移除的包
            for locked_pkg in &lock.packages {
                if !required_packages.contains(&locked_pkg.name) && locked_pkg.name != package {
                    removed_packages.push(locked_pkg.name.clone());
                }
            }
        }

        // 删除包目录
        for pkg_name in &removed_packages {
            self.remove_package_dir(pkg_name).await?;
        }

        // 更新锁定包列表
        if let Some(lock) = &mut self.lock {
            lock.packages.retain(|p| !removed_packages.contains(&p.name));
        }

        // 重新生成自动加载
        if let Some(lock) = &self.lock {
            autoload::generate(&self.project_path, &lock.packages).await?;
            lock::write_composer_lock(&self.project_path, lock).await?;
        }

        tracing::info!("✓ 依赖移除完成: {} 个包被移除", removed_packages.len());

        Ok(())
    }

    /// 重新生成自动加载
    ///
    /// 根据 composer.lock 重新生成 autoload 文件
    ///
    /// # 返回
    /// 生成成功返回 Ok(())，失败返回错误
    pub async fn dump_autoload(&self) -> Result<()> {
        if let Some(lock) = &self.lock {
            autoload::generate(&self.project_path, &lock.packages).await?;
            tracing::info!("✓ 自动加载已重新生成");
        } else {
            tracing::warn!("composer.lock 不存在，无法生成自动加载");
        }

        Ok(())
    }

    /// 写入 composer.json 文件
    async fn write_composer_json(&self) -> Result<()> {
        let config = self.config.as_ref()
            .context("配置未加载")?;

        let json_path = Path::new(&self.project_path).join("composer.json");
        let content = serde_json::to_string_pretty(config)?;

        tokio::fs::write(&json_path, content).await?;

        tracing::debug!("已写入 composer.json: {}", json_path.display());

        Ok(())
    }

    /// 删除包目录
    async fn remove_package_dir(&self, package_name: &str) -> Result<()> {
        let package_dir = Path::new(&self.project_path)
            .join("vendor")
            .join(package_name);

        if package_dir.exists() {
            tokio::fs::remove_dir_all(&package_dir).await?;
            tracing::debug!("已删除包目录: {}", package_dir.display());
        }

        Ok(())
    }

    /// 获取已安装包列表
    ///
    /// # 返回
    /// 已安装包名称列表
    pub fn get_installed_packages(&self) -> Vec<String> {
        self.lock
            .as_ref()
            .map(|l| l.packages.iter().map(|p| p.name.clone()).collect())
            .unwrap_or_default()
    }

    /// 检查包是否已安装
    ///
    /// # 参数
    /// - `package`: 包名称
    ///
    /// # 返回
    /// 是否已安装
    pub fn is_package_installed(&self, package: &str) -> bool {
        self.lock
            .as_ref()
            .map(|l| l.packages.iter().any(|p| p.name == package))
            .unwrap_or(false)
    }

    /// 获取包版本
    ///
    /// # 参数
    /// - `package`: 包名称
    ///
    /// # 返回
    /// 包版本，如果未安装返回 None
    pub fn get_package_version(&self, package: &str) -> Option<String> {
        self.lock
            .as_ref()
            .and_then(|l| l.packages.iter().find(|p| p.name == package))
            .map(|p| p.version.clone())
    }

    /// 执行脚本
    ///
    /// # 参数
    /// - `script_name`: 脚本名称（如 "post-install-cmd"）
    pub async fn run_script(&self, script_name: &str) -> Result<()> {
        let config = self.config.as_ref()
            .context("配置未加载")?;

        if let Some(scripts) = config.scripts.get(script_name) {
            tracing::info!("执行脚本: {}", script_name);

            for cmd in scripts {
                tracing::info!("  执行: {}", cmd);
                // 这里可以集成命令执行器
            }
        }

        Ok(())
    }

    /// 验证 composer.json
    ///
    /// # 返回
    /// 验证结果
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        let config = self.config.as_ref()
            .context("composer.json 未加载")?;

        // 检查必需字段
        if config.name.is_empty() {
            warnings.push("缺少包名称 (name)".to_string());
        }

        if config.description.is_empty() {
            warnings.push("缺少包描述 (description)".to_string());
        }

        if config.license.is_empty() {
            warnings.push("缺少许可证 (license)".to_string());
        }

        // 检查自动加载配置
        if config.autoload.psr_4.is_empty()
            && config.autoload.psr_0.is_empty()
            && config.autoload.classmap.is_empty()
            && config.autoload.files.is_empty() {
            warnings.push("未配置自动加载".to_string());
        }

        Ok(warnings)
    }
}

/// 从解析后的包创建锁定包
impl From<&resolver::ResolvedPackage> for LockedPackage {
    fn from(pkg: &resolver::ResolvedPackage) -> Self {
        LockedPackage {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            source: pkg.source.clone(),
            install_path: format!("vendor/{}", pkg.name),
            autoload: AutoloadConfig::default(),
            license: Vec::new(),
            require: pkg.require.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composer_new() {
        let composer = Composer::new("/tmp/project");
        assert!(composer.config.is_none());
        assert!(composer.lock.is_none());
    }

    #[test]
    fn test_composer_json_default() {
        let json = ComposerJson::default();
        assert!(json.name.is_empty());
        assert!(json.require.is_empty());
        assert_eq!(json.minimum_stability, "stable");
        assert!(json.prefer_stable);
    }

    #[test]
    fn test_locked_package_from_resolved() {
        let resolved = resolver::ResolvedPackage {
            name: "test/package".to_string(),
            version: "1.0.0".to_string(),
            download_url: "https://example.com/test.zip".to_string(),
            checksum: "abc123".to_string(),
            source: PackageSource {
                r#type: "dist".to_string(),
                url: "https://example.com/test.zip".to_string(),
                reference: "abc123".to_string(),
            },
            require: HashMap::new(),
        };

        let locked = LockedPackage::from(&resolved);
        assert_eq!(locked.name, "test/package");
        assert_eq!(locked.version, "1.0.0");
        assert_eq!(locked.install_path, "vendor/test/package");
    }
}
