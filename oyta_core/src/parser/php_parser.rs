//! PHP 解析器封装模块
//!
//! 封装 php-rs-parser 和 php-ast 库，提供统一的 PHP 源码解析接口
//! 核心功能：
//! 1. 解析 PHP 源码为 AST
//! 2. 遍历 AST 提取类、方法、属性、函数、命名空间等符号信息
//! 3. 将提取的符号信息转换为 OYTAPHP 内部的类型定义
//! 4. 支持解析 PHP 配置文件（return [] 格式）

use anyhow::{Context, Result};
use php_rs_parser::{ParserContext, PhpVersion};
use php_ast::ast::{ClassDecl, ClassMemberKind, Expr, ExprKind, FunctionDecl, MethodDecl, Param, PropertyDecl, Stmt, StmtKind, TypeHint, TypeHintKind, UnaryPrefixOp, UseKind as AstUseKind, Visibility as AstVisibility};
use php_ast::visitor::{ScopeVisitor, ScopeWalker, Scope};
use std::ops::ControlFlow;
use std::path::Path;

use crate::symbol_table::types::{
    ClassConstDef, ClassDef, ConfigValue, ConstDef, EnumCaseDef,
    EnumDef, FileParseResult, FuncDef, InterfaceDef, MethodDef, ParamDef, PropertyDef, TraitDef,
    UseItemDef, UseKind, Visibility,
};

/// PHP 解析器
/// 封装 php-rs-parser，提供高级解析接口
/// 内部使用 ParserContext 复用 Arena，提升批量解析性能
pub struct PhpParser {
    /// 可复用的解析上下文
    /// 内部持有 bumpalo::Bump arena，避免每次解析都重新分配内存
    context: ParserContext,
}

impl PhpParser {
    /// 创建新的 PHP 解析器实例
    pub fn new() -> Self {
        Self {
            context: ParserContext::new(),
        }
    }

    /// 解析 PHP 源码字符串
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
        let mut walker = ScopeWalker::new(source, collector);
        let _ = walker.walk(&result.program);

        // 获取收集结果
        let collected = walker.into_inner();

        FileParseResult {
            file_path: file_path.to_string(),
            namespace: collected.namespace,
            use_items: collected.use_items,
            classes: collected.classes,
            interfaces: collected.interfaces,
            traits: collected.traits,
            enums: collected.enums,
            functions: collected.functions,
            constants: collected.constants,
            errors,
        }
    }

    /// 解析 PHP 文件
    /// 读取文件内容并调用 parse_source
    ///
    /// # 参数
    /// - `path`: PHP 文件路径
    pub fn parse_file(&mut self, path: &Path) -> Result<FileParseResult> {
        // 读取文件内容
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("无法读取 PHP 文件: {}", path.display()))?;

        let file_path = path.to_string_lossy().to_string();
        Ok(self.parse_source(&source, &file_path))
    }

    /// 解析 PHP 配置文件
    /// 配置文件格式为 `return [...]`
    /// 返回配置值树
    ///
    /// # 参数
    /// - `source`: PHP 配置文件源码
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

/// 符号收集器
/// 实现 ScopeVisitor trait，在遍历 AST 时收集符号信息
/// ScopeVisitor 会自动维护当前命名空间、类名、函数名等作用域信息
struct SymbolCollector {
    /// 当前文件路径
    file_path: String,
    /// 收集到的命名空间
    namespace: Option<String>,
    /// 收集到的 use 导入项
    use_items: Vec<UseItemDef>,
    /// 收集到的类定义列表
    classes: Vec<ClassDef>,
    /// 收集到的接口定义列表
    interfaces: Vec<InterfaceDef>,
    /// 收集到的 trait 定义列表
    traits: Vec<TraitDef>,
    /// 收集到的枚举定义列表
    enums: Vec<EnumDef>,
    /// 收集到的函数定义列表
    functions: Vec<FuncDef>,
    /// 收集到的常量定义列表
    constants: Vec<ConstDef>,
    /// 当前命名空间下的 use 项（临时存储）
    current_use_items: Vec<UseItemDef>,
}

impl SymbolCollector {
    fn new(file_path: &str) -> Self {
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
    /// 根据语句类型提取不同的符号信息
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
                    // ConstItem.name 是 &'src str 类型
                    let name = const_item.name.to_string();
                    let value = Some(expr_to_string(&const_item.value));
                    let ns = scope.namespace.map(|s| s.to_string());
                    self.constants.push(ConstDef {
                        name,
                        namespace: ns,
                        value,
                        file_path: self.file_path.clone(),
                    });
                }
            }

            _ => {}
        }

        ControlFlow::Continue(())
    }
}

/// 转换 php-ast 的 UseKind 为 OYTAPHP 内部的 UseKind
fn convert_use_kind(kind: AstUseKind) -> UseKind {
    match kind {
        AstUseKind::Normal => UseKind::Normal,
        AstUseKind::Function => UseKind::Function,
        AstUseKind::Const => UseKind::Const,
    }
}

/// 转换 php-ast 的 Visibility 为 OYTAPHP 内部的 Visibility
fn convert_visibility(v: AstVisibility) -> Visibility {
    match v {
        AstVisibility::Public => Visibility::Public,
        AstVisibility::Protected => Visibility::Protected,
        AstVisibility::Private => Visibility::Private,
    }
}

/// 提取类定义
/// 从 AST 的 ClassDecl 转换为内部的 ClassDef
fn extract_class_def<'arena, 'src>(
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
fn extract_method_def<'arena, 'src>(method: &MethodDecl<'arena, 'src>) -> MethodDef {
    let visibility = method.visibility
        .map(|v| convert_visibility(v))
        .unwrap_or(Visibility::Public);
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
fn extract_property_def<'arena, 'src>(prop: &PropertyDecl<'arena, 'src>) -> PropertyDef {
    let visibility = prop.visibility
        .map(|v| convert_visibility(v))
        .unwrap_or(Visibility::Public);
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
fn extract_func_def<'arena, 'src>(
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
fn extract_param_def<'arena, 'src>(param: &Param<'arena, 'src>) -> ParamDef {
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

/// 将类型提示转换为字符串
fn type_hint_to_string<'arena, 'src>(type_hint: &TypeHint<'arena, 'src>) -> String {
    match &type_hint.kind {
        TypeHintKind::Named(name) => name.to_string_repr().to_string(),
        TypeHintKind::Keyword(builtin, _) => builtin.as_str().to_string(),
        TypeHintKind::Nullable(inner) => format!("?{}", type_hint_to_string(inner)),
        TypeHintKind::Union(types) => {
            types.iter()
                .map(|t| type_hint_to_string(t))
                .collect::<Vec<_>>()
                .join("|")
        }
        TypeHintKind::Intersection(types) => {
            types.iter()
                .map(|t| type_hint_to_string(t))
                .collect::<Vec<_>>()
                .join("&")
        }
    }
}

/// 将表达式转换为字符串表示（用于默认值等场景）
/// 这是一个简化实现，只处理常见的字面量表达式
fn expr_to_string<'arena, 'src>(expr: &Expr<'arena, 'src>) -> String {
    match &expr.kind {
        ExprKind::Int(i) => i.to_string(),
        ExprKind::Float(f) => f.to_string(),
        ExprKind::String(s) => format!("'{}'", s),
        ExprKind::Bool(b) => b.to_string(),
        ExprKind::Null => "null".to_string(),
        ExprKind::Array(elements) => {
            let items: Vec<String> = elements.iter().map(|elem| {
                let value_str = expr_to_string(&elem.value);
                match &elem.key {
                    Some(key) => format!("{} => {}", expr_to_string(key), value_str),
                    None => value_str,
                }
            }).collect();
            format!("[{}]", items.join(", "))
        }
        ExprKind::Variable(var) => {
            format!("${}", var.as_str())
        }
        ExprKind::Identifier(id) => id.as_str().to_string(),
        ExprKind::UnaryPrefix(unary) => {
            let op = match unary.op {
                UnaryPrefixOp::Negate => "-",
                UnaryPrefixOp::Plus => "+",
                UnaryPrefixOp::BooleanNot => "!",
                UnaryPrefixOp::BitwiseNot => "~",
                UnaryPrefixOp::PreIncrement => "++",
                UnaryPrefixOp::PreDecrement => "--",
            };
            format!("{}{}", op, expr_to_string(unary.operand))
        }
        ExprKind::Ternary(ternary) => {
            match &ternary.then_expr {
                Some(then) => format!(
                    "{} ? {} : {}",
                    expr_to_string(ternary.condition),
                    expr_to_string(then),
                    expr_to_string(ternary.else_expr)
                ),
                None => format!(
                    "{} ?: {}",
                    expr_to_string(ternary.condition),
                    expr_to_string(ternary.else_expr)
                ),
            }
        }
        ExprKind::NullCoalesce(nc) => {
            format!("{} ?? {}", expr_to_string(nc.left), expr_to_string(nc.right))
        }
        ExprKind::FunctionCall(call) => {
            let args: Vec<String> = call.args.iter().map(|arg| {
                expr_to_string(&arg.value)
            }).collect();
            format!("{}({})", expr_to_string(call.name), args.join(", "))
        }
        ExprKind::ClassConstAccess(access) => {
            format!("{}::{}", expr_to_string(&access.class), expr_to_string(&access.member))
        }
        ExprKind::StaticPropertyAccess(access) => {
            format!("{}::${}", expr_to_string(&access.class), expr_to_string(&access.member))
        }
        _ => {
            // 对于复杂的表达式，返回占位符
            // 后续阶段在解释器中会完整处理
            "/* complex expr */".to_string()
        }
    }
}

/// 评估配置表达式，将 AST 表达式转换为 ConfigValue
/// 递归处理嵌套数组和字面量
fn eval_config_expr<'arena, 'src>(expr: &Expr<'arena, 'src>) -> ConfigValue {
    match &expr.kind {
        // 字面量类型
        ExprKind::Int(i) => ConfigValue::Int(*i),
        ExprKind::Float(f) => ConfigValue::Float(*f),
        ExprKind::String(s) => ConfigValue::String(s.to_string()),
        ExprKind::Bool(b) => ConfigValue::Bool(*b),
        ExprKind::Null => ConfigValue::Null,

        // 数组类型 - 需要区分索引数组和关联数组
        ExprKind::Array(elements) => {
            // 检查是否所有元素都有字符串键（关联数组）
            let has_string_key = elements.iter().any(|elem| {
                matches!(&elem.key, Some(key) if matches!(&key.kind, ExprKind::String(_)))
            });

            if has_string_key {
                // 关联数组
                let mut map = Vec::new();
                for elem in elements.iter() {
                    let key = match &elem.key {
                        Some(key_expr) => expr_to_config_key(key_expr),
                        None => continue,
                    };
                    let value = eval_config_expr(&elem.value);
                    map.push((key, value));
                }
                ConfigValue::AssociativeArray(map)
            } else {
                // 索引数组
                let mut arr = Vec::new();
                for elem in elements.iter() {
                    let value = eval_config_expr(&elem.value);
                    arr.push(value);
                }
                ConfigValue::IndexedArray(arr)
            }
        }

        // 负数
        ExprKind::UnaryPrefix(unary) => {
            match unary.op {
                UnaryPrefixOp::Negate => {
                    match eval_config_expr(unary.operand) {
                        ConfigValue::Int(i) => ConfigValue::Int(-i),
                        ConfigValue::Float(f) => ConfigValue::Float(-f),
                        other => other,
                    }
                }
                _ => ConfigValue::Null,
            }
        }

        // 其他复杂表达式暂不处理
        _ => ConfigValue::Null,
    }
}

/// 将表达式转换为配置键名
fn expr_to_config_key<'arena, 'src>(expr: &Expr<'arena, 'src>) -> String {
    match &expr.kind {
        ExprKind::String(s) => s.to_string(),
        ExprKind::Int(i) => i.to_string(),
        _ => format!("{:?}", expr.kind),
    }
}

/// 提取接口定义
/// 从 AST 的 InterfaceDecl 转换为内部的 InterfaceDef
/// PHP 接口可以包含方法签名和常量，方法默认为 public abstract
fn extract_interface_def<'arena, 'src>(
    iface_decl: &php_ast::ast::InterfaceDecl<'arena, 'src>,
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
/// 从 AST 的 TraitDecl 转换为内部的 TraitDef
/// PHP Trait 可以包含方法、属性、常量（PHP 8.2+）和其他 trait 的使用
fn extract_trait_def<'arena, 'src>(
    trait_decl: &php_ast::ast::TraitDecl<'arena, 'src>,
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
/// 从 AST 的 EnumDecl 转换为内部的 EnumDef
/// PHP 8.1+ 支持枚举类型，分为 Unit Enum 和 Backed Enum
fn extract_enum_def<'arena, 'src>(
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
            php_ast::ast::EnumMemberKind::Case(enum_case) => {
                let value = enum_case.value.as_ref().map(|v| expr_to_string(v));
                cases.push(EnumCaseDef {
                    name: enum_case.name.to_string(),
                    value,
                });
            }
            php_ast::ast::EnumMemberKind::Method(method_decl) => {
                methods.push(extract_method_def(method_decl));
            }
            php_ast::ast::EnumMemberKind::ClassConst(const_decl) => {
                constants.push(ClassConstDef {
                    name: const_decl.name.to_string(),
                    visibility: const_decl.visibility.map(|v| convert_visibility(v)),
                    value: Some(expr_to_string(&const_decl.value)),
                });
            }
            php_ast::ast::EnumMemberKind::TraitUse(trait_use) => {
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
