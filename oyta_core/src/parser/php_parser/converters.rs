//! 类型转换函数模块
//!
//! 提供 php-ast 类型到 OYTAPHP 内部类型的转换

use php_ast::ast::{UseKind as AstUseKind, Visibility as AstVisibility};

use crate::symbol_table::types::{UseKind, Visibility};

/// 转换 php-ast 的 UseKind 为 OYTAPHP 内部的 UseKind
///
/// # 参数
/// - `kind`: php-ast 的 UseKind 枚举
///
/// # 返回
/// OYTAPHP 内部的 UseKind 枚举
pub fn convert_use_kind(kind: AstUseKind) -> UseKind {
    match kind {
        // 普通类/接口/ trait 导入
        AstUseKind::Normal => UseKind::Normal,
        // 函数导入
        AstUseKind::Function => UseKind::Function,
        // 常量导入
        AstUseKind::Const => UseKind::Const,
    }
}

/// 转换 php-ast 的 Visibility 为 OYTAPHP 内部的 Visibility
///
/// # 参数
/// - `v`: php-ast 的 Visibility 枚举
///
/// # 返回
/// OYTAPHP 内部的 Visibility 枚举
pub fn convert_visibility(v: AstVisibility) -> Visibility {
    match v {
        // 公开
        AstVisibility::Public => Visibility::Public,
        // 受保护
        AstVisibility::Protected => Visibility::Protected,
        // 私有
        AstVisibility::Private => Visibility::Private,
    }
}
