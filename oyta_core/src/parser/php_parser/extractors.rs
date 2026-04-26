//! 符号提取函数模块
//!
//! 提供从 AST 节点提取符号信息的函数

use php_ast::ast::{ClassDecl, ClassMemberKind, EnumMemberKind, Expr, FunctionDecl, InterfaceDecl, MethodDecl, Param, PropertyDecl, TraitDecl};
use php_ast::visitor::Scope;

use crate::symbol_table::types::{
    ClassConstDef, ClassDef, ConstDef, EnumCaseDef, EnumDef, FuncDef, InterfaceDef, MethodDef,
    ParamDef, PropertyDef, TraitDef,
};

use super::converters::{convert_use_kind, convert_visibility};
use super::expr::{expr_to_string, type_hint_to_string};

/// 提取类定义
///
/// 从 AST 的 ClassDecl 转换为内部的 ClassDef
///
/// # 参数
/// - `class_decl`: AST 类声明节点
/// - `scope`: 当前作用域
/// - `file_path`: 源文件路径
///
/// # 返回
/// 类定义，匿名类返回 None
pub fn extract_class_def<'arena, 'src>(
    class_decl: &ClassDecl<'arena, 'src>,
    scope: &Scope<'src>,
    file_path: &str,
) -> Option<ClassDef> {
    // 匿名类不注册到符号表
    let name = class_decl.name?;

    // 构建完整类名
    let namespace = scope.namespace.map(|s| s.to_string());
    let full_name = match &namespace {
        Some(ns) => format!("{}\\{}", ns, name),
        None => name.to_string(),
    };

    // 提取继承信息
    let extends = class_decl.extends.as_ref().map(|e| e.to_string_repr().to_string());

    // 提取实现的接口
    let implements: Vec<String> = class_decl.implements
        .iter()
        .map(|i| i.to_string_repr().to_string())
        .collect();

    // 提取成员信息
    let mut methods = Vec::new();
    let mut properties = Vec::new();
    let mut constants = Vec::new();
    let mut traits = Vec::new();

    for member in class_decl.members.iter() {
        match &member.kind {
            ClassMemberKind::Method(method_decl) => {
                methods.push(extract_method_def(method_decl));
            }
            ClassMemberKind::Property(prop_decl) => {
                properties.push(extract_property_def(prop_decl));
            }
            ClassMemberKind::ClassConst(const_decl) => {
                // ClassConstDecl 包含单个常量定义
                constants.push(ClassConstDef {
                    name: const_decl.name.to_string(),
                    visibility: const_decl.visibility.map(|v| convert_visibility(v)),
                    value: Some(expr_to_string(&const_decl.value)),
                });
            }
            ClassMemberKind::TraitUse(trait_use) => {
                for trait_name in trait_use.traits.iter() {
                    traits.push(trait_name.to_string_repr().to_string());
                }
            }
        }
    }

    Some(ClassDef {
        full_name,
        name: name.to_string(),
        namespace,
        is_abstract: class_decl.modifiers.is_abstract,
        is_final: class_decl.modifiers.is_final,
        is_readonly: class_decl.modifiers.is_readonly,
        extends,
        implements,
        traits,
        methods,
        properties,
        constants,
        file_path: file_path.to_string(),
    })
}

/// 提取方法定义
///
/// # 参数
/// - `method`: AST 方法声明节点
///
/// # 返回
/// 方法定义
pub fn extract_method_def<'arena, 'src>(method: &MethodDecl<'arena, 'src>) -> MethodDef {
    let visibility = method.visibility
        .map(|v| convert_visibility(v))
        .unwrap_or(crate::symbol_table::types::Visibility::Public);
    let return_type = method.return_type.as_ref().map(|t| type_hint_to_string(t));
    let params: Vec<ParamDef> = method.params.iter().map(extract_param_def).collect();
    let body_stmt_count = method.body.as_ref().map(|body| body.len());

    MethodDef {
        name: method.name.to_string(),
        visibility,
        is_static: method.is_static,
        is_abstract: method.is_abstract,
        is_final: method.is_final,
        return_type,
        params,
        body_stmt_count,
    }
}

/// 提取属性定义
///
/// # 参数
/// - `prop`: AST 属性声明节点
///
/// # 返回
/// 属性定义
pub fn extract_property_def<'arena, 'src>(prop: &PropertyDecl<'arena, 'src>) -> PropertyDef {
    let visibility = prop.visibility
        .map(|v| convert_visibility(v))
        .unwrap_or(crate::symbol_table::types::Visibility::Public);
    let type_hint = prop.type_hint.as_ref().map(|t| type_hint_to_string(t));
    let default_value = prop.default.as_ref().map(|v| expr_to_string(v));

    PropertyDef {
        name: prop.name.to_string(),
        visibility,
        is_static: prop.is_static,
        is_readonly: prop.is_readonly,
        type_hint,
        default_value,
    }
}

/// 提取函数定义
///
/// # 参数
/// - `func`: AST 函数声明节点
/// - `scope`: 当前作用域
/// - `file_path`: 源文件路径
///
/// # 返回
/// 函数定义
pub fn extract_func_def<'arena, 'src>(
    func: &FunctionDecl<'arena, 'src>,
    scope: &Scope<'src>,
    file_path: &str,
) -> FuncDef {
    let namespace = scope.namespace.map(|s| s.to_string());
    let full_name = match &namespace {
        Some(ns) => format!("{}\\{}", ns, func.name),
        None => func.name.to_string(),
    };
    let return_type = func.return_type.as_ref().map(|t| type_hint_to_string(t));
    let params: Vec<ParamDef> = func.params.iter().map(extract_param_def).collect();

    FuncDef {
        name: func.name.to_string(),
        namespace,
        full_name,
        return_type,
        params,
        by_ref: func.by_ref,
        file_path: file_path.to_string(),
    }
}

/// 提取参数定义
///
/// # 参数
/// - `param`: AST 参数节点
///
/// # 返回
/// 参数定义
pub fn extract_param_def<'arena, 'src>(param: &Param<'arena, 'src>) -> ParamDef {
    let type_hint = param.type_hint.as_ref().map(|t| type_hint_to_string(t));
    let default_value = param.default.as_ref().map(|v| expr_to_string(v));

    ParamDef {
        name: param.name.to_string(),
        type_hint,
        default_value,
        by_ref: param.by_ref,
        variadic: param.variadic,
    }
}

/// 提取接口定义
///
/// 从 AST 的 InterfaceDecl 转换为内部的 InterfaceDef
/// PHP 接口可以包含方法签名和常量，方法默认为 public abstract
///
/// # 参数
/// - `iface_decl`: AST 接口声明节点
/// - `scope`: 当前作用域
/// - `file_path`: 源文件路径
///
/// # 返回
/// 接口定义
pub fn extract_interface_def<'arena, 'src>(
    iface_decl: &InterfaceDecl<'arena, 'src>,
    scope: &Scope<'src>,
    file_path: &str,
) -> Option<InterfaceDef> {
    let name = iface_decl.name;

    // 构建完整接口名
    let namespace = scope.namespace.map(|s| s.to_string());
    let full_name = match &namespace {
        Some(ns) => format!("{}\\{}", ns, name),
        None => name.to_string(),
    };

    // 提取继承的父接口列表
    // PHP 接口支持多继承：interface A extends B, C
    let extends: Vec<String> = iface_decl.extends
        .iter()
        .map(|e| e.to_string_repr().to_string())
        .collect();

    // 提取接口成员（方法和常量）
    let mut methods = Vec::new();
    let mut constants = Vec::new();

    for member in iface_decl.members.iter() {
        match &member.kind {
            ClassMemberKind::Method(method_decl) => {
                methods.push(extract_method_def(method_decl));
            }
            ClassMemberKind::ClassConst(const_decl) => {
                constants.push(ClassConstDef {
                    name: const_decl.name.to_string(),
                    visibility: const_decl.visibility.map(|v| convert_visibility(v)),
                    value: Some(expr_to_string(&const_decl.value)),
                });
            }
            _ => {}
        }
    }

    Some(InterfaceDef {
        full_name,
        name: name.to_string(),
        namespace,
        extends,
        methods,
        constants,
        file_path: file_path.to_string(),
    })
}

/// 提取 Trait 定义
///
/// 从 AST 的 TraitDecl 转换为内部的 TraitDef
/// PHP Trait 可以包含方法、属性、常量（PHP 8.2+）和其他 trait 的使用
///
/// # 参数
/// - `trait_decl`: AST trait 声明节点
/// - `scope`: 当前作用域
/// - `file_path`: 源文件路径
///
/// # 返回
/// Trait 定义
pub fn extract_trait_def<'arena, 'src>(
    trait_decl: &TraitDecl<'arena, 'src>,
    scope: &Scope<'src>,
    file_path: &str,
) -> Option<TraitDef> {
    let name = trait_decl.name;

    // 构建完整 trait 名
    let namespace = scope.namespace.map(|s| s.to_string());
    let full_name = match &namespace {
        Some(ns) => format!("{}\\{}", ns, name),
        None => name.to_string(),
    };

    // 提取 trait 成员
    let mut methods = Vec::new();
    let mut properties = Vec::new();
    let mut constants = Vec::new();
    let mut traits = Vec::new();

    for member in trait_decl.members.iter() {
        match &member.kind {
            ClassMemberKind::Method(method_decl) => {
                methods.push(extract_method_def(method_decl));
            }
            ClassMemberKind::Property(prop_decl) => {
                properties.push(extract_property_def(prop_decl));
            }
            ClassMemberKind::ClassConst(const_decl) => {
                constants.push(ClassConstDef {
                    name: const_decl.name.to_string(),
                    visibility: const_decl.visibility.map(|v| convert_visibility(v)),
                    value: Some(expr_to_string(&const_decl.value)),
                });
            }
            ClassMemberKind::TraitUse(trait_use) => {
                for trait_name in trait_use.traits.iter() {
                    traits.push(trait_name.to_string_repr().to_string());
                }
            }
        }
    }

    Some(TraitDef {
        full_name,
        name: name.to_string(),
        namespace,
        traits,
        methods,
        properties,
        constants,
        file_path: file_path.to_string(),
    })
}

/// 提取枚举定义
///
/// 从 AST 的 EnumDecl 转换为内部的 EnumDef
/// PHP 8.1+ 支持枚举类型，分为 Unit Enum 和 Backed Enum
///
/// # 参数
/// - `enum_decl`: AST 枚举声明节点
/// - `scope`: 当前作用域
/// - `file_path`: 源文件路径
///
/// # 返回
/// 枚举定义
pub fn extract_enum_def<'arena, 'src>(
    enum_decl: &php_ast::ast::EnumDecl<'arena, 'src>,
    scope: &Scope<'src>,
    file_path: &str,
) -> Option<EnumDef> {
    let name = enum_decl.name;

    // 构建完整枚举名
    let namespace = scope.namespace.map(|s| s.to_string());
    let full_name = match &namespace {
        Some(ns) => format!("{}\\{}", ns, name),
        None => name.to_string(),
    };

    // 提取底层类型（Backed Enum）
    // enum Status: int → backing_type = Some("int")
    // enum Color → backing_type = None (Unit Enum)
    let backing_type = enum_decl.scalar_type
        .as_ref()
        .map(|t| t.to_string_repr().to_string());

    // 提取实现的接口列表
    let implements: Vec<String> = enum_decl.implements
        .iter()
        .map(|i| i.to_string_repr().to_string())
        .collect();

    // 提取枚举成员
    let mut cases = Vec::new();
    let mut methods = Vec::new();
    let mut constants = Vec::new();
    let mut traits = Vec::new();

    for member in enum_decl.members.iter() {
        match &member.kind {
            EnumMemberKind::Case(enum_case) => {
                let value = enum_case.value.as_ref().map(|v| expr_to_string(v));
                cases.push(EnumCaseDef {
                    name: enum_case.name.to_string(),
                    value,
                });
            }
            EnumMemberKind::Method(method_decl) => {
                methods.push(extract_method_def(method_decl));
            }
            EnumMemberKind::ClassConst(const_decl) => {
                constants.push(ClassConstDef {
                    name: const_decl.name.to_string(),
                    visibility: const_decl.visibility.map(|v| convert_visibility(v)),
                    value: Some(expr_to_string(&const_decl.value)),
                });
            }
            EnumMemberKind::TraitUse(trait_use) => {
                for trait_name in trait_use.traits.iter() {
                    traits.push(trait_name.to_string_repr().to_string());
                }
            }
        }
    }

    Some(EnumDef {
        full_name,
        name: name.to_string(),
        namespace,
        backing_type,
        implements,
        traits,
        cases,
        methods,
        constants,
        file_path: file_path.to_string(),
    })
}

/// 提取顶层常量定义
///
/// # 参数
/// - `const_item`: AST 常量项
/// - `scope`: 当前作用域
/// - `file_path`: 源文件路径
///
/// # 返回
/// 常量定义
pub fn extract_const_def<'arena, 'src>(
    const_item: &php_ast::ast::ConstItem<'arena, 'src>,
    scope: &Scope<'src>,
    file_path: &str,
) -> ConstDef {
    let name = const_item.name.to_string();
    let value = Some(expr_to_string(&const_item.value));
    let namespace = scope.namespace.map(|s| s.to_string());

    ConstDef {
        name,
        namespace,
        value,
        file_path: file_path.to_string(),
    }
}
