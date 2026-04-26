//! PHP 解析器主体模块
//!
//! 封装 php-rs-parser，提供高级解析接口

use anyhow::{Context, Result};
use php_rs_parser::{ParserContext, PhpVersion};
use php_ast::ast::{Stmt, StmtKind};
use std::path::Path;

use crate::symbol_table::types::{ConfigValue, FileParseResult};

use super::collector::{collect_result, SymbolCollector};
use super::config_parser::eval_config_expr;

/// PHP 解析器
///
/// 封装 php-rs-parser，提供高级解析接口
/// 内部使用 ParserContext 复用 Arena，提升批量解析性能
pub struct PhpParser {
    /// 可复用的解析上下文
    /// 内部持有 bumpalo::Bump arena，避免每次解析都重新分配内存
    context: ParserContext,
}

impl PhpParser {
    /// 创建新的 PHP 解析器实例
    ///
    /// # 返回
    /// 新的 PhpParser 实例
    pub fn new() -> Self {
        Self {
            context: ParserContext::new(),
        }
    }

    /// 解析 PHP 源码字符串
    ///
    /// 返回解析后的文件符号结果
    ///
    /// # 参数
    /// - `source`: PHP 源码字符串
    /// - `file_path`: 源文件路径（用于记录符号来源）
    ///
    /// # 返回
    /// 文件解析结果，包含类、函数、常量等符号信息
    pub fn parse_source(&mut self, source: &str, file_path: &str) -> FileParseResult {
        // 使用 ParserContext 解析源码
        // 默认使用 PHP 8.5 语法（最新版本，兼容所有 PHP 8.x 语法）
        let result = self.context.reparse_versioned(source, PhpVersion::Php85);

        // 收集解析错误
        let errors: Vec<String> = result.errors.iter().map(|e| e.to_string()).collect();

        // 如果有解析错误，记录日志但不中断解析
        // php-rs-parser 是容错的，即使有错误也会生成 AST
        if !errors.is_empty() {
            tracing::debug!(
                "PHP 文件解析有 {} 个警告: {}",
                errors.len(),
                file_path
            );
        }

        // 使用 ScopeVisitor 遍历 AST，提取符号信息
        let collector = SymbolCollector::new(file_path);
        let mut walker = php_ast::visitor::ScopeWalker::new(source, collector);
        let _ = walker.walk(&result.program);

        // 获取收集结果
        let mut parse_result = collect_result(walker);
        parse_result.errors = errors;

        parse_result
    }

    /// 解析 PHP 文件
    ///
    /// 读取文件内容并调用 parse_source
    ///
    /// # 参数
    /// - `path`: PHP 文件路径
    ///
    /// # 返回
    /// 文件解析结果
    pub fn parse_file(&mut self, path: &Path) -> Result<FileParseResult> {
        // 读取文件内容
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("无法读取 PHP 文件: {}", path.display()))?;

        let file_path = path.to_string_lossy().to_string();
        Ok(self.parse_source(&source, &file_path))
    }

    /// 解析 PHP 配置文件
    ///
    /// 配置文件格式为 `return [...]`
    /// 返回配置值树
    ///
    /// # 参数
    /// - `source`: PHP 配置文件源码
    ///
    /// # 返回
    /// 配置值
    pub fn parse_config(&mut self, source: &str) -> Result<ConfigValue> {
        let result = self.context.reparse_versioned(source, PhpVersion::Php85);

        // 记录解析错误
        for err in &result.errors {
            tracing::warn!("配置文件解析警告: {}", err);
        }

        // 在顶层语句中查找 return 语句
        for stmt in result.program.stmts.iter() {
            if let StmtKind::Return(Some(expr)) = &stmt.kind {
                return Ok(eval_config_expr(expr));
            }
        }

        // 没有找到 return 语句，返回空关联数组
        tracing::warn!("配置文件未找到 return 语句");
        Ok(ConfigValue::AssociativeArray(Vec::new()))
    }

    /// 解析 PHP 配置文件（从文件路径）
    ///
    /// # 参数
    /// - `path`: 配置文件路径
    ///
    /// # 返回
    /// 配置值
    pub fn parse_config_file(&mut self, path: &Path) -> Result<ConfigValue> {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("无法读取配置文件: {}", path.display()))?;
        self.parse_config(&source)
    }
}

impl Default for PhpParser {
    fn default() -> Self {
        Self::new()
    }
}
