//! 项目信息结构体
//!
//! 用于 version 和 info 命令的输出
//! 包含项目的基本信息，如根目录、应用模式、文件存在状态等

use std::fmt;

/// 项目信息结构体
/// 包含项目的各种状态信息，用于显示给用户
#[derive(Debug)]
pub struct ProjectInfo {
    /// 项目根目录路径字符串
    pub root: String,

    /// 是否为多应用模式
    pub is_multi_app: bool,

    /// 应用名称列表（多应用模式下有值）
    pub app_names: Vec<String>,

    /// 是否存在 .env 文件
    pub has_env_file: bool,

    /// 是否存在 composer.json 文件
    pub has_composer_json: bool,

    /// 是否存在 vendor 目录
    pub has_vendor: bool,
}

/// 实现 Display trait，用于格式化输出项目信息
/// 输出格式为中文，方便用户阅读
impl fmt::Display for ProjectInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 输出标题
        writeln!(f, "OYTAPHP 项目信息")?;
        writeln!(f, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;

        // 输出项目根目录
        writeln!(f, "项目根目录: {}", self.root)?;

        // 输出应用模式
        writeln!(f, "应用模式: {}", if self.is_multi_app { "多应用" } else { "单应用" })?;

        // 多应用模式下输出应用列表
        if self.is_multi_app && !self.app_names.is_empty() {
            writeln!(f, "应用列表: {}", self.app_names.join(", "))?;
        }

        // 输出文件存在状态，使用 ✓/✗ 标记
        writeln!(f, ".env 文件: {}", if self.has_env_file { "✓" } else { "✗" })?;
        writeln!(f, "composer.json: {}", if self.has_composer_json { "✓" } else { "✗" })?;
        writeln!(f, "vendor 目录: {}", if self.has_vendor { "✓" } else { "✗" })?;

        Ok(())
    }
}
