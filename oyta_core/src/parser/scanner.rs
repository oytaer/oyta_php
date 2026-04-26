//! PHP 文件扫描器模块
//!
//! 负责扫描项目目录下的所有 PHP 文件
//! 并调用解析器进行解析，将结果注册到符号表
//! 支持增量扫描和全量扫描两种模式

use anyhow::{Context, Result};
use std::path::Path;

use super::php_parser::PhpParser;
use crate::symbol_table::registry::SymbolRegistry;

/// PHP 文件扫描器
/// 扫描项目目录下的 PHP 文件并解析注册到符号表
pub struct Scanner {
    /// PHP 解析器实例
    parser: PhpParser,
    /// 符号表注册表
    registry: SymbolRegistry,
}

impl Scanner {
    /// 创建新的文件扫描器
    pub fn new() -> Self {
        Self {
            parser: PhpParser::new(),
            registry: SymbolRegistry::new(),
        }
    }

    /// 获取符号表注册表的引用
    /// 用于其他模块查询符号信息
    pub fn registry(&self) -> &SymbolRegistry {
        &self.registry
    }

    /// 获取符号表注册表的可变引用
    pub fn registry_mut(&mut self) -> &mut SymbolRegistry {
        &mut self.registry
    }

    /// 全量扫描项目目录
    /// 扫描 app/、config/、extend/ 目录下的所有 PHP 文件
    /// 清空现有符号表后重新注册
    ///
    /// # 参数
    /// - `project_root`: 项目根目录路径
    ///
    /// # 返回
    /// 扫描到的文件数量
    pub fn scan_all(&mut self, project_root: &Path) -> Result<ScanResult> {
        let start = std::time::Instant::now();
        let mut result = ScanResult::default();

        // 清空现有符号表
        self.registry.clear();

        // 需要扫描的目录列表
        let scan_dirs = [
            "app",       // 应用目录（控制器、模型、中间件等）
            "config",    // 配置目录
            "extend",    // 扩展类库目录
            "route",     // 路由定义目录
        ];

        for dir_name in &scan_dirs {
            let dir_path = project_root.join(dir_name);
            if dir_path.is_dir() {
                self.scan_directory(&dir_path, &mut result)?;
            }
        }

        // 扫描 public/index.php（入口文件，用于检测内嵌服务）
        let index_file = project_root.join("public/index.php");
        if index_file.is_file() {
            self.parse_and_register(&index_file, &mut result)?;
        }

        // 扫描 vendor 目录下的 oyta 相关包
        // 不扫描整个 vendor 目录（太慢），只扫描 vendor/ 下第一层目录
        let vendor_dir = project_root.join("vendor");
        if vendor_dir.is_dir() {
            self.scan_vendor_libraries(&vendor_dir, &mut result)?;
        }

        result.duration = start.elapsed();
        tracing::debug!(
            "项目扫描完成: {} 个文件, {} 个类, {} 个函数, 耗时 {:?}",
            result.file_count,
            result.class_count,
            result.function_count,
            result.duration
        );

        Ok(result)
    }

    /// 增量扫描指定文件
    /// 用于热重载场景，只重新解析修改过的文件
    ///
    /// # 参数
    /// - `file_path`: 修改的文件路径
    pub fn scan_file(&mut self, file_path: &Path) -> Result<FileParseResult> {
        // 先注销该文件的旧符号
        self.registry.unregister_file(&file_path.to_string_lossy());

        // 重新解析并注册
        let parse_result = self.parser.parse_file(file_path)
            .with_context(|| format!("解析文件失败: {}", file_path.display()))?;

        // 注册新符号
        self.registry.register_file(&parse_result);

        Ok(parse_result)
    }

    /// 递归扫描目录下的所有 PHP 文件
    fn scan_directory(&mut self, dir: &Path, result: &mut ScanResult) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir)
            .with_context(|| format!("无法读取目录: {}", dir.display()))?;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // 递归扫描子目录
                self.scan_directory(&path, result)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("php") {
                // 解析 PHP 文件
                self.parse_and_register(&path, result)?;
            }
        }

        Ok(())
    }

    /// 扫描 vendor 目录下的库
    /// 只扫描第一层包目录中的 src/ 目录
    fn scan_vendor_libraries(&mut self, vendor_dir: &Path, result: &mut ScanResult) -> Result<()> {
        if let Ok(entries) = std::fs::read_dir(vendor_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                // 跳过 bin 目录
                if path.file_name().and_then(|n| n.to_str()) == Some("bin") {
                    continue;
                }
                // 扫描 vendor/vendor_name/package_name/src/ 目录
                if let Ok(sub_entries) = std::fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.is_dir() {
                            let src_dir = sub_path.join("src");
                            if src_dir.is_dir() {
                                self.scan_directory(&src_dir, result)?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// 解析单个 PHP 文件并注册到符号表
    fn parse_and_register(&mut self, path: &Path, result: &mut ScanResult) -> Result<()> {
        match self.parser.parse_file(path) {
            Ok(parse_result) => {
                result.file_count += 1;
                result.class_count += parse_result.classes.len();
                result.function_count += parse_result.functions.len();
                result.constant_count += parse_result.constants.len();

                if !parse_result.errors.is_empty() {
                    result.error_count += parse_result.errors.len();
                    tracing::debug!(
                        "文件 {} 有 {} 个解析警告",
                        path.display(),
                        parse_result.errors.len()
                    );
                }

                // 注册到符号表
                self.registry.register_file(&parse_result);
                Ok(())
            }
            Err(e) => {
                result.error_count += 1;
                tracing::warn!("解析文件失败: {} - {}", path.display(), e);
                Ok(()) // 不中断扫描，继续处理其他文件
            }
        }
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

/// 扫描结果统计
#[derive(Debug, Default)]
pub struct ScanResult {
    /// 扫描到的文件数量
    pub file_count: usize,
    /// 发现的类数量
    pub class_count: usize,
    /// 发现的函数数量
    pub function_count: usize,
    /// 发现的常量数量
    pub constant_count: usize,
    /// 解析错误/警告数量
    pub error_count: usize,
    /// 扫描耗时
    pub duration: std::time::Duration,
}

/// 文件解析结果（从 php_parser 返回的类型重导出）
pub use crate::symbol_table::types::FileParseResult;
