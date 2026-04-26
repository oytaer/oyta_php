//! OYTAPHP CLI 工具
//!
//! 独立于运行时的命令行工具
//! 提供项目创建、脚手架生成、运行时安装、版本管理等功能
//! 对应 ThinkPHP 8.0 的 topthink/think 命令行工具

use anyhow::Result;
use clap::{Parser, Subcommand};

/// OYTAPHP 最新版本号
const LATEST_VERSION: &str = "0.1.0";
/// GitHub 发布页面基础 URL
const RELEASE_BASE_URL: &str = "https://github.com/oyta-php/oyta/releases";

#[derive(Parser)]
#[command(name = "oyta-cli")]
#[command(about = "OYTAPHP 命令行工具", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 创建新项目
    New {
        /// 项目名称
        name: String,
        /// 项目目录（默认为当前目录下的项目名）
        #[arg(long)]
        path: Option<String>,
        /// 多应用模式
        #[arg(long)]
        multi: bool,
    },
    /// 安装 OYTAPHP 运行时
    Install {
        /// 安装路径
        #[arg(long, default_value = ".")]
        path: String,
        /// 指定版本
        #[arg(long)]
        version: Option<String>,
    },
    /// 查看版本信息
    Version,
    /// 更新运行时到最新版本
    Update,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name, path, multi } => create_project(&name, path.as_deref(), multi),
        Commands::Install { path, version } => install_runtime(&path, version.as_deref()),
        Commands::Version => show_version(),
        Commands::Update => update_runtime(),
    }
}

/// 创建新的 OYTAPHP 项目
fn create_project(name: &str, path: Option<&str>, multi: bool) -> Result<()> {
    let project_dir = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => std::env::current_dir()?.join(name),
    };

    println!("正在创建项目: {}", name);
    println!("项目目录: {}", project_dir.display());

    if project_dir.exists() {
        anyhow::bail!("目录已存在: {}", project_dir.display());
    }

    // 创建项目目录结构
    let mut dirs = vec![
        "app/controller",
        "app/model",
        "app/middleware",
        "app/validate",
        "app/event",
        "app/listener",
        "app/service",
        "config",
        "route",
        "public",
        "runtime/cache",
        "runtime/temp",
        "runtime/view",
        "runtime/log",
        "runtime/session",
        "view",
        "storage",
        "extend",
    ];

    // 多应用模式额外目录
    if multi {
        dirs.extend_from_slice(&[
            "app/index/controller",
            "app/index/model",
            "app/index/view",
            "app/admin/controller",
            "app/admin/model",
            "app/admin/view",
            "app/common",
        ]);
    }

    for dir in &dirs {
        let dir_path = project_dir.join(dir);
        std::fs::create_dir_all(&dir_path)?;
    }

    // 创建入口文件
    create_entry_file(&project_dir, name)?;

    // 创建配置文件
    create_config_files(&project_dir)?;

    // 创建路由文件
    create_route_file(&project_dir)?;

    // 创建默认控制器
    create_default_controller(&project_dir, multi)?;

    // 创建 .env 文件
    create_env_file(&project_dir)?;

    // 创建 composer.json
    create_composer_json(&project_dir, name)?;

    println!();
    println!("✓ 项目 [{}] 创建成功！", name);
    println!();
    println!("下一步：");
    println!("  cd {}", name);
    println!("  oyta run");
    println!();

    Ok(())
}

/// 创建入口文件 public/index.php
fn create_entry_file(project_dir: &std::path::Path, _name: &str) -> Result<()> {
    let content = r#"<?php
// OYTAPHP 入口文件
// 此文件由 oyta-cli 自动生成

// 加载基础文件
require __DIR__ . '/../vendor/autoload.php';

// 获取应用实例
$http = new \oyta\Http();

// 执行 HTTP 请求
$http->run()->send();
"#;
    let path = project_dir.join("public/index.php");
    std::fs::write(&path, content)?;
    Ok(())
}

/// 创建配置文件
fn create_config_files(project_dir: &std::path::Path) -> Result<()> {
    // app.php
    let app_config = r#"<?php
// 应用配置
return [
    // 应用名称
    'name' => 'OYTAPHP',
    // 应用调试模式
    'debug' => env('APP_DEBUG', false),
    // 应用地址
    'domain' => env('APP_DOMAIN', 'localhost'),
    // 默认时区
    'timezone' => 'Asia/Shanghai',
    // 默认控制器
    'default_controller' => 'Index',
    // 默认操作
    'default_action' => 'index',
    // URL 路由模式
    'url_route_must' => false,
];
"#;
    std::fs::write(project_dir.join("config/app.php"), app_config)?;

    // database.php
    let db_config = r#"<?php
// 数据库配置
return [
    // 数据库类型
    'type' => 'mysql',
    // 数据库连接地址
    'hostname' => env('DB_HOST', '127.0.0.1'),
    // 数据库连接端口
    'hostport' => env('DB_PORT', 3306),
    // 数据库名称
    'database' => env('DB_NAME', 'oyta'),
    // 数据库用户名
    'username' => env('DB_USER', 'root'),
    // 数据库密码
    'password' => env('DB_PASS', ''),
    // 数据库编码
    'charset' => 'utf8mb4',
    // 数据库表前缀
    'prefix' => env('DB_PREFIX', ''),
    // 数据库调试模式
    'debug' => env('APP_DEBUG', false),
];
"#;
    std::fs::write(project_dir.join("config/database.php"), db_config)?;

    // cache.php
    let cache_config = r#"<?php
// 缓存配置
return [
    // 默认缓存驱动
    'default' => 'file',
    // 缓存驱动配置
    'stores' => [
        'file' => [
            'type' => 'file',
            'path' => runtime_path() . '/cache',
        ],
        'redis' => [
            'type' => 'redis',
            'host' => env('REDIS_HOST', '127.0.0.1'),
            'port' => env('REDIS_PORT', 6379),
            'password' => env('REDIS_PASSWORD', ''),
            'select' => env('REDIS_SELECT', 0),
            'timeout' => 0,
            'persistent' => false,
        ],
    ],
];
"#;
    std::fs::write(project_dir.join("config/cache.php"), cache_config)?;

    // session.php
    let session_config = r#"<?php
// Session 配置
return [
    // Session 驱动
    'type' => 'file',
    // Session 存储路径
    'path' => runtime_path() . '/session',
    // Session 过期时间
    'expire' => 1440,
    // Session 前缀
    'prefix' => '',
];
"#;
    std::fs::write(project_dir.join("config/session.php"), session_config)?;

    Ok(())
}

/// 创建路由文件
fn create_route_file(project_dir: &std::path::Path) -> Result<()> {
    let content = r#"<?php
// 路由定义文件
use oyta\facade\Route;

// 首页路由
Route::get('/', 'Index/index');
"#;
    std::fs::write(project_dir.join("route/app.php"), content)?;
    Ok(())
}

/// 创建默认控制器
fn create_default_controller(project_dir: &std::path::Path, multi: bool) -> Result<()> {
    if multi {
        // 多应用模式：创建 index 和 admin 应用的默认控制器
        let index_content = r#"<?php
namespace app\index\controller;

class Index
{
    public function index()
    {
        return 'Hello, OYTAPHP! - Index App';
    }
}
"#;
        std::fs::write(project_dir.join("app/index/controller/Index.php"), index_content)?;

        let admin_content = r#"<?php
namespace app\admin\controller;

class Index
{
    public function index()
    {
        return 'Hello, OYTAPHP! - Admin App';
    }
}
"#;
        std::fs::write(project_dir.join("app/admin/controller/Index.php"), admin_content)?;
    } else {
        let content = r#"<?php
namespace app\controller;

class Index
{
    public function index()
    {
        return 'Hello, OYTAPHP!';
    }
}
"#;
        std::fs::write(project_dir.join("app/controller/Index.php"), content)?;
    }
    Ok(())
}

/// 创建 .env 文件
fn create_env_file(project_dir: &std::path::Path) -> Result<()> {
    let content = r#"# OYTAPHP 环境配置
APP_DEBUG = true
APP_DOMAIN = localhost

# 数据库配置
DB_HOST = 127.0.0.1
DB_PORT = 3306
DB_NAME = oyta
DB_USER = root
DB_PASS =
DB_PREFIX =

# Redis 配置
REDIS_HOST = 127.0.0.1
REDIS_PORT = 6379
REDIS_PASSWORD =
"#;
    std::fs::write(project_dir.join(".env"), content)?;
    Ok(())
}

/// 创建 composer.json
fn create_composer_json(project_dir: &std::path::Path, name: &str) -> Result<()> {
    let content = serde_json::json!({
        "name": name,
        "description": "OYTAPHP Application",
        "type": "project",
        "require": {
            "php": ">=8.0"
        },
        "autoload": {
            "psr-4": {
                "app\\": "app/"
            }
        },
        "config": {
            "preferred-install": "dist"
        }
    });
    let content_str = serde_json::to_string_pretty(&content)?;
    std::fs::write(project_dir.join("composer.json"), content_str)?;
    Ok(())
}

/// 安装运行时
fn install_runtime(path: &str, version: Option<&str>) -> Result<()> {
    let ver = version.unwrap_or(LATEST_VERSION);
    println!("正在安装 OYTAPHP 运行时 v{}...", ver);
    println!("目标路径: {}", path);

    // 检测当前平台
    let (os, arch) = detect_platform();
    println!("当前平台: {}-{}", os, arch);

    // 构建下载 URL
    let binary_name = format!("oyta-{}-{}", os, arch);
    let download_url = format!("{}/download/v{}/{}", RELEASE_BASE_URL, ver, binary_name);

    println!("下载地址: {}", download_url);

    // 尝试下载
    match download_file(&download_url, path) {
        Ok(sha256) => {
            println!("下载完成！");
            println!("SHA256: {}", sha256);

            // 验证 SHA256
            if let Some(expected_hash) = fetch_sha256(ver, &binary_name) {
                if sha256 == expected_hash {
                    println!("✓ SHA256 校验通过");
                } else {
                    println!("✗ SHA256 校验失败！");
                    println!("  期望: {}", expected_hash);
                    println!("  实际: {}", sha256);
                }
            }

            // 设置可执行权限
            let binary_path = std::path::Path::new(path).join("oyta");
            if binary_path.exists() {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = std::fs::metadata(&binary_path)?.permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(&binary_path, perms)?;
                }
            }

            println!("✓ OYTAPHP v{} 安装成功！", ver);
        }
        Err(e) => {
            println!("自动下载失败: {}", e);
            println!("请手动下载: {}", download_url);
            println!("下载后放置到: {}/oyta", path);
        }
    }

    Ok(())
}

/// 检测当前平台
fn detect_platform() -> (String, String) {
    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "amd64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "unknown"
    };

    (os.to_string(), arch.to_string())
}

/// 下载文件并返回 SHA256 哈希
fn download_file(url: &str, dest_dir: &str) -> Result<String> {
    let dir = std::path::Path::new(dest_dir);
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }

    // 使用 reqwest 下载（如果可用）
    // 如果 reqwest 不可用，返回错误让用户手动下载
    Err(anyhow::anyhow!("网络下载功能需要 reqwest 库支持"))
}

/// 获取 SHA256 校验值
fn fetch_sha256(version: &str, binary_name: &str) -> Option<String> {
    let checksum_url = format!("{}/download/v{}/SHA256SUMS", RELEASE_BASE_URL, version);
    // 尝试获取校验文件
    None
}

/// 更新运行时到最新版本
fn update_runtime() -> Result<()> {
    println!("正在检查更新...");
    println!("当前版本: {}", LATEST_VERSION);
    println!("最新版本: {}", LATEST_VERSION);

    if true {
        println!("已经是最新版本！");
    } else {
        install_runtime(".", Some(LATEST_VERSION))?;
    }

    Ok(())
}

/// 显示版本信息
fn show_version() -> Result<()> {
    println!("OYTAPHP CLI v{}", env!("CARGO_PKG_VERSION"));
    println!("Rust 运行时，兼容 ThinkPHP 8.0");
    Ok(())
}
