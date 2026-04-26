//! 项目根目录检测器
//!
//! 负责确定当前工作目录是否为有效的 OYTAPHP 项目
//! 并提供项目内各目录的路径访问

use std::path::{Path, PathBuf};
use anyhow::{bail, Context, Result};

/// OYTAPHP 项目的标志文件/目录
/// 至少存在以下之二即可判定为有效项目
const PROJECT_MARKERS: &[&str] = &[
    "app",              // 应用目录
    "config",           // 配置目录
    "public",           // 公共目录
    ".env",             // 环境变量文件
    "composer.json",    // Composer 定义文件
];

/// 项目必须存在的核心目录
/// 缺少这些目录将导致项目验证失败
const REQUIRED_DIRS: &[&str] = &[
    "app",      // 应用目录
    "config",   // 配置目录
];

/// 项目结构定义
/// 包含项目根目录及所有子目录的路径
/// 所有路径均为绝对路径
#[derive(Debug, Clone)]
pub struct Project {
    /// 项目根目录的绝对路径
    pub root: PathBuf,

    /// 应用目录（app/）
    pub app_dir: PathBuf,

    /// 配置目录（config/）
    pub config_dir: PathBuf,

    /// 路由定义目录（route/）
    pub route_dir: PathBuf,

    /// 公共目录（public/）
    pub public_dir: PathBuf,

    /// 视图目录（view/）
    pub view_dir: PathBuf,

    /// 运行时目录（runtime/）
    pub runtime_dir: PathBuf,

    /// 扩展类库目录（extend/）
    pub extend_dir: PathBuf,

    /// Vendor 目录（vendor/）
    pub vendor_dir: PathBuf,

    /// OYTAPHP 核心二进制所在目录（vendor/oyta_php/bin/）
    pub bin_dir: PathBuf,

    /// .env 文件路径
    pub env_file: PathBuf,

    /// .example.env 文件路径
    pub example_env_file: PathBuf,

    /// composer.json 文件路径
    pub composer_json: PathBuf,

    /// composer.lock 文件路径
    pub composer_lock: PathBuf,
}

impl Project {
    /// 从指定路径检测并创建 Project 实例
    /// 会验证路径是否为有效的 OYTAPHP 项目
    ///
    /// # 参数
    /// - `path`: 待检测的项目根目录路径
    ///
    /// # 返回
    /// - 成功：Project 实例
    /// - 失败：项目验证不通过的详细错误信息
    pub fn detect(path: &Path) -> Result<Self> {
        // 将路径转为绝对路径
        let root = path.canonicalize()
            .with_context(|| format!("路径不存在或无法访问: {}", path.display()))?;

        // 验证是否为目录
        if !root.is_dir() {
            bail!("指定路径不是目录: {}", root.display());
        }

        // 验证必须存在的目录
        for dir_name in REQUIRED_DIRS {
            let dir_path = root.join(dir_name);
            if !dir_path.is_dir() {
                bail!(
                    "缺少必需的目录: {}（项目根目录: {}）\n\
                    提示：使用 oyta build 创建项目目录结构",
                    dir_name,
                    root.display()
                );
            }
        }

        // 计算标志文件匹配数
        let marker_count = PROJECT_MARKERS
            .iter()
            .filter(|marker| root.join(marker).exists())
            .count();

        // 至少匹配 2 个标志才认为是有效项目
        if marker_count < 2 {
            bail!(
                "当前目录不是有效的 OYTAPHP 项目: {}\n\
                提示：使用 oyta-cli new <name> 创建新项目",
                root.display()
            );
        }

        // 构建 Project 实例，所有路径基于 root 计算
        Ok(Self {
            app_dir: root.join("app"),
            config_dir: root.join("config"),
            route_dir: root.join("route"),
            public_dir: root.join("public"),
            view_dir: root.join("view"),
            runtime_dir: root.join("runtime"),
            extend_dir: root.join("extend"),
            vendor_dir: root.join("vendor"),
            bin_dir: root.join("vendor/oyta_php/bin"),
            env_file: root.join(".env"),
            example_env_file: root.join(".example.env"),
            composer_json: root.join("composer.json"),
            composer_lock: root.join("composer.lock"),
            root,
        })
    }

    /// 从当前工作目录检测项目
    /// 这是最常用的入口，oyta 命令默认以当前目录为项目根
    ///
    /// # 返回
    /// - 成功：Project 实例
    /// - 失败：当前目录不是有效项目的错误信息
    pub fn detect_from_cwd() -> Result<Self> {
        let cwd = std::env::current_dir()
            .context("无法获取当前工作目录")?;
        Self::detect(&cwd)
    }

    /// 确保运行时目录存在（runtime/ 及其子目录）
    /// 运行时目录需要写权限，用于存放缓存、日志、Session 等文件
    /// 如果目录不存在会自动创建
    ///
    /// # 返回
    /// - 成功：Ok(())
    /// - 失败：无法创建目录的错误信息
    pub fn ensure_runtime_dirs(&self) -> Result<()> {
        // 需要确保存在的运行时子目录
        let runtime_subdirs = &[
            "cache",        // 缓存文件目录
            "log",          // 日志文件目录
            "session",      // Session 文件目录
            "temp",         // 临时文件目录
            "view",         // 模板编译缓存目录
        ];

        // 创建 runtime 根目录
        std::fs::create_dir_all(&self.runtime_dir)
            .with_context(|| format!("无法创建运行时目录: {}", self.runtime_dir.display()))?;

        // 创建 runtime 子目录
        for subdir in runtime_subdirs {
            let path = self.runtime_dir.join(subdir);
            std::fs::create_dir_all(&path)
                .with_context(|| format!("无法创建运行时子目录: {}", path.display()))?;
        }

        Ok(())
    }

    /// 获取应用目录下的控制器目录路径
    /// 单应用模式: app/controller/
    /// 多应用模式: app/{app_name}/controller/
    ///
    /// # 参数
    /// - `app_name`: 应用名称，None 表示单应用模式
    pub fn controller_dir(&self, app_name: Option<&str>) -> PathBuf {
        match app_name {
            Some(name) => self.app_dir.join(name).join("controller"),
            None => self.app_dir.join("controller"),
        }
    }

    /// 获取应用目录下的模型目录路径
    /// 单应用模式: app/model/
    /// 多应用模式: app/{app_name}/model/
    ///
    /// # 参数
    /// - `app_name`: 应用名称，None 表示单应用模式
    pub fn model_dir(&self, app_name: Option<&str>) -> PathBuf {
        match app_name {
            Some(name) => self.app_dir.join(name).join("model"),
            None => self.app_dir.join("model"),
        }
    }

    /// 获取应用目录下的中间件目录路径
    /// 单应用模式: app/middleware/
    /// 多应用模式: app/{app_name}/middleware/
    ///
    /// # 参数
    /// - `app_name`: 应用名称，None 表示单应用模式
    pub fn middleware_dir(&self, app_name: Option<&str>) -> PathBuf {
        match app_name {
            Some(name) => self.app_dir.join(name).join("middleware"),
            None => self.app_dir.join("middleware"),
        }
    }

    /// 判断是否为多应用模式
    /// 检测逻辑：
    /// - 如果 app/controller/ 直接存在 → 单应用模式
    /// - 如果 app/{name}/controller/ 存在 → 多应用模式
    pub fn is_multi_app(&self) -> bool {
        // 如果 app/controller/ 存在，则为单应用模式
        if self.app_dir.join("controller").is_dir() {
            return false;
        }

        // 检测 app/ 下的子目录是否包含 controller/
        if let Ok(entries) = std::fs::read_dir(&self.app_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    // 子目录下存在 controller/ 则为多应用模式
                    if entry.path().join("controller").is_dir() {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// 获取所有应用名称列表（多应用模式下使用）
    /// 返回 app/ 下包含 controller/ 子目录的目录名
    /// 结果按字母排序，确保顺序稳定
    pub fn app_names(&self) -> Vec<String> {
        let mut names = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.app_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                // 只收集包含 controller/ 子目录的目录
                if path.is_dir() && path.join("controller").is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        names.push(name.to_string());
                    }
                }
            }
        }

        // 按字母排序，确保顺序稳定
        names.sort();
        names
    }

    /// 获取项目的显示信息
    /// 用于 version 和 info 命令的输出
    pub fn info(&self) -> crate::project::info::ProjectInfo {
        crate::project::info::ProjectInfo {
            root: self.root.display().to_string(),
            is_multi_app: self.is_multi_app(),
            app_names: self.app_names(),
            has_env_file: self.env_file.exists(),
            has_composer_json: self.composer_json.exists(),
            has_vendor: self.vendor_dir.exists(),
        }
    }
}
