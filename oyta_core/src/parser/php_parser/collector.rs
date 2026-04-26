//! 符号收集器模块
//!
//! 实现 ScopeVisitor trait，在遍历 AST 时收集符号信息

use php_ast::ast::{Stmt, StmtKind};
use php_ast::visitor::{Scope, ScopeVisitor, ScopeWalker};
use std::ops::ControlFlow;

use crate::symbol_table::types::{
    ClassDef, ConstDef, EnumDef, FileParseResult, FuncDef, InterfaceDef, TraitDef, UseItemDef,
};

use super::converters::convert_use_kind;
use super::extractors::{
    extract_class_def, extract_const_def, extract_enum_def, extract_func_def,
    extract_interface_def, extract_trait_def,
};

/// 符号收集器
///
/// 实现 ScopeVisitor trait，在遍历 AST 时收集符号信息
/// ScopeVisitor 会自动维护当前命名空间、类名、函数名等作用域信息
pub struct SymbolCollector {
    /// 当前文件路径
    pub file_path: String,
    /// 收集到的命名空间
    pub namespace: Option<String>,
    /// 收集到的 use 导入项
    pub use_items: Vec<UseItemDef>,
    /// 收集到的类定义列表
    pub classes: Vec<ClassDef>,
    /// 收集到的接口定义列表
    pub interfaces: Vec<InterfaceDef>,
    /// 收集到的 trait 定义列表
    pub traits: Vec<TraitDef>,
    /// 收集到的枚举定义列表
    pub enums: Vec<EnumDef>,
    /// 收集到的函数定义列表
    pub functions: Vec<FuncDef>,
    /// 收集到的常量定义列表
    pub constants: Vec<ConstDef>,
    /// 当前命名空间下的 use 项（临时存储）
    current_use_items: Vec<UseItemDef>,
}

impl SymbolCollector {
    /// 创建新的符号收集器
    ///
    /// # 参数
    /// - `file_path`: 当前解析的文件路径
    ///
    /// # 返回
    /// 新的 SymbolCollector 实例
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            namespace: None,
            use_items: Vec::new(),
            classes: Vec::new(),
            interfaces: Vec::new(),
            traits: Vec::new(),
            enums: Vec::new(),
            functions: Vec::new(),
            constants: Vec::new(),
            current_use_items: Vec::new(),
        }
    }
}

impl<'arena, 'src> ScopeVisitor<'arena, 'src> for SymbolCollector {
    /// 访问语句节点
    ///
    /// 根据语句类型提取不同的符号信息
    ///
    /// # 参数
    /// - `stmt`: AST 语句节点
    /// - `scope`: 当前作用域信息
    ///
    /// # 返回
    /// ControlFlow::Continue(()) 表示继续遍历
    fn visit_stmt(
        &mut self,
        stmt: &Stmt<'arena, 'src>,
        scope: &Scope<'src>,
    ) -> ControlFlow<()> {
        match &stmt.kind {
            // 命名空间声明
            StmtKind::Namespace(ns_decl) => {
                self.namespace = ns_decl.name.as_ref().map(|name| name.to_string_repr().to_string());
                // 将当前收集的 use 项保存
                self.use_items = self.current_use_items.clone();
            }

            // use 导入声明
            StmtKind::Use(use_decl) => {
                for use_item in use_decl.uses.iter() {
                    let kind = convert_use_kind(use_decl.kind);
                    let full_name = use_item.name.to_string_repr().to_string();
                    let alias = use_item.alias.map(|s| s.to_string());
                    self.current_use_items.push(UseItemDef {
                        full_name,
                        alias,
                        kind,
                    });
                }
            }

            // 类声明
            StmtKind::Class(class_decl) => {
                if let Some(class_def) = extract_class_def(
                    class_decl,
                    scope,
                    &self.file_path,
                ) {
                    self.classes.push(class_def);
                }
            }

            // 接口声明
            StmtKind::Interface(iface_decl) => {
                if let Some(iface_def) = extract_interface_def(
                    iface_decl,
                    scope,
                    &self.file_path,
                ) {
                    self.interfaces.push(iface_def);
                }
            }

            // Trait 声明
            StmtKind::Trait(trait_decl) => {
                if let Some(trait_def) = extract_trait_def(
                    trait_decl,
                    scope,
                    &self.file_path,
                ) {
                    self.traits.push(trait_def);
                }
            }

            // 枚举声明
            StmtKind::Enum(enum_decl) => {
                if let Some(enum_def) = extract_enum_def(
                    enum_decl,
                    scope,
                    &self.file_path,
                ) {
                    self.enums.push(enum_def);
                }
            }

            // 函数声明
            StmtKind::Function(func_decl) => {
                let func_def = extract_func_def(func_decl, scope, &self.file_path);
                self.functions.push(func_def);
            }

            // 顶层常量声明
            StmtKind::Const(const_items) => {
                for const_item in const_items.iter() {
                    let const_def = extract_const_def(const_item, scope, &self.file_path);
                    self.constants.push(const_def);
                }
            }

            _ => {}
        }

        ControlFlow::Continue(())
    }
}

/// 从 ScopeWalker 中提取收集结果
///
/// # 参数
/// - `walker`: ScopeWalker 实例
///
/// # 返回
/// 文件解析结果
pub fn collect_result(walker: ScopeWalker<'_, SymbolCollector>) -> FileParseResult {
    let collected = walker.into_inner();

    FileParseResult {
        file_path: collected.file_path,
        namespace: collected.namespace,
        use_items: collected.use_items,
        classes: collected.classes,
        interfaces: collected.interfaces,
        traits: collected.traits,
        enums: collected.enums,
        functions: collected.functions,
        constants: collected.constants,
        errors: Vec::new(),
    }
}
