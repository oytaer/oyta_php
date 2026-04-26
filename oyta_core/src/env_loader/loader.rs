//! 环境变量加载器
//!
//! 负责加载 .env 文件中的环境变量到进程环境中
//! 使用 dotenvy 库实现，支持以下特性：
//! - 自动检测 .env 文件是否存在
//! - 不会覆盖已有的环境变量（系统环境变量优先）
//! - 支持 .example.env 作为模板参考
//! - 加载结果日志记录

use anyhow::Result;
use crate::project::detector::Project;

/// 宏：在环境变量加载失败时提供详细的错误信息
/// 封装 anyhow 的 bail! 宏，添加上下文信息
macro_rules! bail_with_context {
    ($($arg:tt)*) => {
        anyhow::bail!($($arg)*)
    };
}

/// 加载项目根目录下的 .env 文件
/// 如果 .env 文件不存在，不会报错，只输出警告信息
/// 因为开发环境可能不使用 .env 文件，而是直接设置系统环境变量
///
/// # 参数
/// - `project`: 项目结构体引用，用于获取 .env 文件路径
///
/// # 返回
/// - 成功：Ok(())
/// - 失败：.env 文件格式错误的详细信息
///
/// # 加载优先级
/// 1. 系统环境变量（最高优先级，不会被 .env 覆盖）
/// 2. .env 文件中定义的变量
/// 3. .example.env 仅作为参考，不会被加载
pub fn load_env(project: &Project) -> Result<()> {
    // 检查 .env 文件是否存在
    if project.env_file.exists() {
        // 使用 dotenvy 加载 .env 文件
        // dotenvy::from_path 不会覆盖已有的环境变量
        match dotenvy::from_path(&project.env_file) {
            Ok(_) => {
                tracing::debug!(".env 文件加载成功: {}", project.env_file.display());
            }
            Err(e) => {
                // 加载失败，可能是文件格式错误
                // 这里使用 context 添加更详细的错误信息
                bail_with_context!(
                    ".env 文件格式错误: {}（文件路径: {}）",
                    e,
                    project.env_file.display()
                );
            }
        }
    } else {
        // .env 文件不存在，检查是否有 .example.env 模板
        if project.example_env_file.exists() {
            tracing::warn!(
                ".env 文件不存在，但发现 .example.env 模板文件。\n\
                提示：复制 .example.env 为 .env 并修改配置：\n\
                cp {} {}",
                project.example_env_file.display(),
                project.env_file.display()
            );
        } else {
            // 既没有 .env 也没有 .example.env，输出调试信息
            tracing::debug!(".env 文件不存在，将使用系统环境变量");
        }
    }

    Ok(())
}

/// 获取环境变量的便捷函数
/// 优先从系统环境变量中获取，如果不存在则返回默认值
///
/// # 参数
/// - `key`: 环境变量名称
/// - `default`: 默认值
///
/// # 返回
/// 环境变量的值或默认值
///
/// # 示例
/// ```ignore
/// let debug = get_env("APP_DEBUG", "false");
/// let port = get_env("APP_PORT", "8000");
/// ```
pub fn get_env(key: &str, default: &str) -> String {
    // std::env::var 会自动从进程环境中获取
    // dotenvy 加载的变量也会出现在进程环境中
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// 获取环境变量的布尔值便捷函数
/// 将字符串 "true"/"1"/"yes"（不区分大小写）解析为 true
/// 其他值解析为 false
///
/// # 参数
/// - `key`: 环境变量名称
/// - `default`: 默认布尔值
///
/// # 返回
/// 解析后的布尔值
pub fn get_env_bool(key: &str, default: bool) -> bool {
    let value = get_env(key, "");
    if value.is_empty() {
        return default;
    }
    // 不区分大小写匹配
    matches!(value.to_lowercase().as_str(), "true" | "1" | "yes")
}

/// 获取环境变量的整数便捷函数
/// 如果解析失败则返回默认值
///
/// # 参数
/// - `key`: 环境变量名称
/// - `default`: 默认整数值
///
/// # 返回
/// 解析后的整数值或默认值
pub fn get_env_int(key: &str, default: i64) -> i64 {
    let value = get_env(key, "");
    if value.is_empty() {
        return default;
    }
    value.parse().unwrap_or(default)
}
