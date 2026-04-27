//! 反射元数据管理模块
//!
//! 提供类、方法、属性、函数等元数据的存储和查询

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// 反射元数据管理器
///
/// 存储和管理所有反射相关的元数据
#[derive(Debug)]
pub struct ReflectionMetadata {
    /// 类元数据映射
    classes: DashMap<String, ClassMetadata>,
    /// 函数元数据映射
    functions: DashMap<String, FunctionMetadata>,
    /// 接口元数据映射
    interfaces: DashMap<String, InterfaceMetadata>,
    /// Trait 元数据映射
    traits: DashMap<String, TraitMetadata>,
    /// 枚举元数据映射
    enums: DashMap<String, EnumMetadata>,
}

impl ReflectionMetadata {
    /// 创建新的元数据管理器
    pub fn new() -> Self {
        Self {
            classes: DashMap::new(),
            functions: DashMap::new(),
            interfaces: DashMap::new(),
            traits: DashMap::new(),
            enums: DashMap::new(),
        }
    }
    
    /// 注册类元数据
    pub fn register_class(&self, metadata: ClassMetadata) {
        self.classes.insert(metadata.name.clone(), metadata);
    }
    
    /// 注册函数元数据
    pub fn register_function(&self, metadata: FunctionMetadata) {
        self.functions.insert(metadata.name.clone(), metadata);
    }
    
    /// 注册接口元数据
    pub fn register_interface(&self, metadata: InterfaceMetadata) {
        self.interfaces.insert(metadata.name.clone(), metadata);
    }
    
    /// 注册 Trait 元数据
    pub fn register_trait(&self, metadata: TraitMetadata) {
        self.traits.insert(metadata.name.clone(), metadata);
    }
    
    /// 注册枚举元数据
    pub fn register_enum(&self, metadata: EnumMetadata) {
        self.enums.insert(metadata.name.clone(), metadata);
    }
    
    /// 获取类元数据
    pub fn get_class(&self, name: &str) -> Option<ClassMetadata> {
        self.classes.get(name).map(|r| r.value().clone())
    }
    
    /// 获取函数元数据
    pub fn get_function(&self, name: &str) -> Option<FunctionMetadata> {
        self.functions.get(name).map(|r| r.value().clone())
    }
    
    /// 获取接口元数据
    pub fn get_interface(&self, name: &str) -> Option<InterfaceMetadata> {
        self.interfaces.get(name).map(|r| r.value().clone())
    }
    
    /// 获取 Trait 元数据
    pub fn get_trait(&self, name: &str) -> Option<TraitMetadata> {
        self.traits.get(name).map(|r| r.value().clone())
    }
    
    /// 获取枚举元数据
    pub fn get_enum(&self, name: &str) -> Option<EnumMetadata> {
        self.enums.get(name).map(|r| r.value().clone())
    }
    
    /// 检查类是否存在
    pub fn has_class(&self, name: &str) -> bool {
        self.classes.contains_key(name)
    }
    
    /// 检查函数是否存在
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
    
    /// 检查接口是否存在
    pub fn has_interface(&self, name: &str) -> bool {
        self.interfaces.contains_key(name)
    }
    
    /// 检查 Trait 是否存在
    pub fn has_trait(&self, name: &str) -> bool {
        self.traits.contains_key(name)
    }
    
    /// 检查枚举是否存在
    pub fn has_enum(&self, name: &str) -> bool {
        self.enums.contains_key(name)
    }
    
    /// 获取所有类名
    pub fn all_class_names(&self) -> Vec<String> {
        self.classes.iter().map(|r| r.key().clone()).collect()
    }
    
    /// 获取所有函数名
    pub fn all_function_names(&self) -> Vec<String> {
        self.functions.iter().map(|r| r.key().clone()).collect()
    }
    
    /// 清空所有元数据
    pub fn clear(&self) {
        self.classes.clear();
        self.functions.clear();
        self.interfaces.clear();
        self.traits.clear();
        self.enums.clear();
    }
}

impl Default for ReflectionMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// 类元数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassMetadata {
    /// 类名（完整名称，含命名空间）
    pub name: String,
    /// 短类名（不含命名空间）
    pub short_name: String,
    /// 命名空间
    pub namespace: Option<String>,
    /// 是否为抽象类
    pub is_abstract: bool,
    /// 是否为最终类
    pub is_final: bool,
    /// 是否为只读类
    pub is_readonly: bool,
    /// 父类名
    pub parent_class: Option<String>,
    /// 实现的接口列表
    pub interfaces: Vec<String>,
    /// 使用的 Trait 列表
    pub traits: Vec<String>,
    /// 方法定义列表
    pub methods: Vec<MethodMetadata>,
    /// 属性定义列表
    pub properties: Vec<PropertyMetadata>,
    /// 类常量定义列表
    pub constants: Vec<ConstantMetadata>,
    /// 注解列表
    pub attributes: Vec<AttributeMetadata>,
    /// DocBlock 注释
    pub doc_comment: Option<String>,
    /// 源文件路径
    pub file_name: Option<String>,
    /// 起始行号
    pub start_line: Option<usize>,
    /// 结束行号
    pub end_line: Option<usize>,
}

impl ClassMetadata {
    /// 创建新的类元数据
    pub fn new(name: &str) -> Self {
        let (namespace, short_name) = Self::parse_name(name);
        Self {
            name: name.to_string(),
            short_name,
            namespace,
            is_abstract: false,
            is_final: false,
            is_readonly: false,
            parent_class: None,
            interfaces: Vec::new(),
            traits: Vec::new(),
            methods: Vec::new(),
            properties: Vec::new(),
            constants: Vec::new(),
            attributes: Vec::new(),
            doc_comment: None,
            file_name: None,
            start_line: None,
            end_line: None,
        }
    }
    
    /// 解析类名
    fn parse_name(full_name: &str) -> (Option<String>, String) {
        if let Some(pos) = full_name.rfind('\\') {
            (Some(full_name[..pos].to_string()), full_name[pos + 1..].to_string())
        } else {
            (None, full_name.to_string())
        }
    }
    
    /// 查找方法
    pub fn find_method(&self, name: &str) -> Option<&MethodMetadata> {
        let name_lower = name.to_lowercase();
        self.methods.iter().find(|m| m.name.to_lowercase() == name_lower)
    }
    
    /// 查找属性
    pub fn find_property(&self, name: &str) -> Option<&PropertyMetadata> {
        // 属性名可能带 $ 前缀
        let name = name.trim_start_matches('$');
        self.properties.iter().find(|p| p.name == name || p.name == format!("${}", name))
    }
    
    /// 查找常量
    pub fn find_constant(&self, name: &str) -> Option<&ConstantMetadata> {
        self.constants.iter().find(|c| c.name == name)
    }
    
    /// 检查是否继承自指定类
    pub fn extends(&self, class_name: &str) -> bool {
        self.parent_class.as_ref().map(|p| p.eq_ignore_ascii_case(class_name)).unwrap_or(false)
    }
    
    /// 检查是否实现了指定接口
    pub fn implements(&self, interface_name: &str) -> bool {
        self.interfaces.iter().any(|i| i.eq_ignore_ascii_case(interface_name))
    }
    
    /// 检查是否使用了指定 Trait
    pub fn uses_trait(&self, trait_name: &str) -> bool {
        self.traits.iter().any(|t| t.eq_ignore_ascii_case(trait_name))
    }
}

/// 方法元数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodMetadata {
    /// 方法名
    pub name: String,
    /// 所属类名
    pub class_name: String,
    /// 可见性
    pub visibility: Visibility,
    /// 是否为静态方法
    pub is_static: bool,
    /// 是否为抽象方法
    pub is_abstract: bool,
    /// 是否为最终方法
    pub is_final: bool,
    /// 返回类型
    pub return_type: Option<TypeMetadata>,
    /// 参数列表
    pub parameters: Vec<ParameterMetadata>,
    /// 注解列表
    pub attributes: Vec<AttributeMetadata>,
    /// DocBlock 注释
    pub doc_comment: Option<String>,
    /// 起始行号
    pub start_line: Option<usize>,
    /// 结束行号
    pub end_line: Option<usize>,
}

impl MethodMetadata {
    /// 创建新的方法元数据
    pub fn new(name: &str, class_name: &str) -> Self {
        Self {
            name: name.to_string(),
            class_name: class_name.to_string(),
            visibility: Visibility::Public,
            is_static: false,
            is_abstract: false,
            is_final: false,
            return_type: None,
            parameters: Vec::new(),
            attributes: Vec::new(),
            doc_comment: None,
            start_line: None,
            end_line: None,
        }
    }
    
    /// 检查是否为构造函数
    pub fn is_constructor(&self) -> bool {
        self.name.eq_ignore_ascii_case("__construct")
    }
    
    /// 检查是否为析构函数
    pub fn is_destructor(&self) -> bool {
        self.name.eq_ignore_ascii_case("__destruct")
    }
}

/// 属性元数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyMetadata {
    /// 属性名（含 $ 前缀）
    pub name: String,
    /// 所属类名
    pub class_name: String,
    /// 可见性
    pub visibility: Visibility,
    /// 是否为静态属性
    pub is_static: bool,
    /// 是否为只读属性
    pub is_readonly: bool,
    /// 类型
    pub property_type: Option<TypeMetadata>,
    /// 默认值
    pub default_value: Option<String>,
    /// 注解列表
    pub attributes: Vec<AttributeMetadata>,
    /// DocBlock 注释
    pub doc_comment: Option<String>,
}

impl PropertyMetadata {
    /// 创建新的属性元数据
    pub fn new(name: &str, class_name: &str) -> Self {
        let name = if name.starts_with('$') {
            name.to_string()
        } else {
            format!("${}", name)
        };
        
        Self {
            name,
            class_name: class_name.to_string(),
            visibility: Visibility::Public,
            is_static: false,
            is_readonly: false,
            property_type: None,
            default_value: None,
            attributes: Vec::new(),
            doc_comment: None,
        }
    }
    
    /// 获取属性名（不含 $ 前缀）
    pub fn name_without_dollar(&self) -> &str {
        self.name.trim_start_matches('$')
    }
}

/// 参数元数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterMetadata {
    /// 参数名（含 $ 前缀）
    pub name: String,
    /// 参数位置（从 0 开始）
    pub position: usize,
    /// 类型
    pub parameter_type: Option<TypeMetadata>,
    /// 是否可选
    pub is_optional: bool,
    /// 是否引用传递
    pub is_passed_by_reference: bool,
    /// 是否可变参数
    pub is_variadic: bool,
    /// 默认值
    pub default_value: Option<String>,
    /// 注解列表
    pub attributes: Vec<AttributeMetadata>,
}

impl ParameterMetadata {
    /// 创建新的参数元数据
    pub fn new(name: &str, position: usize) -> Self {
        let name = if name.starts_with('$') {
            name.to_string()
        } else {
            format!("${}", name)
        };
        
        Self {
            name,
            position,
            parameter_type: None,
            is_optional: false,
            is_passed_by_reference: false,
            is_variadic: false,
            default_value: None,
            attributes: Vec::new(),
        }
    }
    
    /// 获取参数名（不含 $ 前缀）
    pub fn name_without_dollar(&self) -> &str {
        self.name.trim_start_matches('$')
    }
}

/// 常量元数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstantMetadata {
    /// 常量名
    pub name: String,
    /// 所属类名（对于类常量）
    pub class_name: Option<String>,
    /// 可见性
    pub visibility: Visibility,
    /// 值
    pub value: Option<String>,
    /// 注解列表
    pub attributes: Vec<AttributeMetadata>,
}

impl ConstantMetadata {
    /// 创建新的常量元数据
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            class_name: None,
            visibility: Visibility::Public,
            value: None,
            attributes: Vec::new(),
        }
    }
}

/// 函数元数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionMetadata {
    /// 函数名（完整名称，含命名空间）
    pub name: String,
    /// 短函数名（不含命名空间）
    pub short_name: String,
    /// 命名空间
    pub namespace: Option<String>,
    /// 返回类型
    pub return_type: Option<TypeMetadata>,
    /// 参数列表
    pub parameters: Vec<ParameterMetadata>,
    /// 是否引用返回
    pub returns_reference: bool,
    /// 注解列表
    pub attributes: Vec<AttributeMetadata>,
    /// DocBlock 注释
    pub doc_comment: Option<String>,
    /// 源文件路径
    pub file_name: Option<String>,
    /// 起始行号
    pub start_line: Option<usize>,
    /// 结束行号
    pub end_line: Option<usize>,
}

impl FunctionMetadata {
    /// 创建新的函数元数据
    pub fn new(name: &str) -> Self {
        let (namespace, short_name) = if let Some(pos) = name.rfind('\\') {
            (Some(name[..pos].to_string()), name[pos + 1..].to_string())
        } else {
            (None, name.to_string())
        };
        
        Self {
            name: name.to_string(),
            short_name,
            namespace,
            return_type: None,
            parameters: Vec::new(),
            returns_reference: false,
            attributes: Vec::new(),
            doc_comment: None,
            file_name: None,
            start_line: None,
            end_line: None,
        }
    }
}

/// 接口元数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceMetadata {
    /// 接口名（完整名称）
    pub name: String,
    /// 短接口名
    pub short_name: String,
    /// 命名空间
    pub namespace: Option<String>,
    /// 继承的父接口列表
    pub extends: Vec<String>,
    /// 方法定义列表
    pub methods: Vec<MethodMetadata>,
    /// 常量定义列表
    pub constants: Vec<ConstantMetadata>,
    /// 注解列表
    pub attributes: Vec<AttributeMetadata>,
    /// DocBlock 注释
    pub doc_comment: Option<String>,
    /// 源文件路径
    pub file_name: Option<String>,
}

impl InterfaceMetadata {
    /// 创建新的接口元数据
    pub fn new(name: &str) -> Self {
        let (namespace, short_name) = if let Some(pos) = name.rfind('\\') {
            (Some(name[..pos].to_string()), name[pos + 1..].to_string())
        } else {
            (None, name.to_string())
        };
        
        Self {
            name: name.to_string(),
            short_name,
            namespace,
            extends: Vec::new(),
            methods: Vec::new(),
            constants: Vec::new(),
            attributes: Vec::new(),
            doc_comment: None,
            file_name: None,
        }
    }
}

/// Trait 元数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitMetadata {
    /// Trait 名（完整名称）
    pub name: String,
    /// 短 Trait 名
    pub short_name: String,
    /// 命名空间
    pub namespace: Option<String>,
    /// 使用的其他 Trait 列表
    pub traits: Vec<String>,
    /// 方法定义列表
    pub methods: Vec<MethodMetadata>,
    /// 属性定义列表
    pub properties: Vec<PropertyMetadata>,
    /// 注解列表
    pub attributes: Vec<AttributeMetadata>,
    /// DocBlock 注释
    pub doc_comment: Option<String>,
    /// 源文件路径
    pub file_name: Option<String>,
}

impl TraitMetadata {
    /// 创建新的 Trait 元数据
    pub fn new(name: &str) -> Self {
        let (namespace, short_name) = if let Some(pos) = name.rfind('\\') {
            (Some(name[..pos].to_string()), name[pos + 1..].to_string())
        } else {
            (None, name.to_string())
        };
        
        Self {
            name: name.to_string(),
            short_name,
            namespace,
            traits: Vec::new(),
            methods: Vec::new(),
            properties: Vec::new(),
            attributes: Vec::new(),
            doc_comment: None,
            file_name: None,
        }
    }
}

/// 枚举元数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumMetadata {
    /// 枚举名（完整名称）
    pub name: String,
    /// 短枚举名
    pub short_name: String,
    /// 命名空间
    pub namespace: Option<String>,
    /// 底层类型（int 或 string）
    pub backing_type: Option<String>,
    /// 实现的接口列表
    pub interfaces: Vec<String>,
    /// 使用的 Trait 列表
    pub traits: Vec<String>,
    /// Case 列表
    pub cases: Vec<EnumCaseMetadata>,
    /// 方法定义列表
    pub methods: Vec<MethodMetadata>,
    /// 常量定义列表
    pub constants: Vec<ConstantMetadata>,
    /// 注解列表
    pub attributes: Vec<AttributeMetadata>,
    /// DocBlock 注释
    pub doc_comment: Option<String>,
    /// 源文件路径
    pub file_name: Option<String>,
}

impl EnumMetadata {
    /// 创建新的枚举元数据
    pub fn new(name: &str) -> Self {
        let (namespace, short_name) = if let Some(pos) = name.rfind('\\') {
            (Some(name[..pos].to_string()), name[pos + 1..].to_string())
        } else {
            (None, name.to_string())
        };
        
        Self {
            name: name.to_string(),
            short_name,
            namespace,
            backing_type: None,
            interfaces: Vec::new(),
            traits: Vec::new(),
            cases: Vec::new(),
            methods: Vec::new(),
            constants: Vec::new(),
            attributes: Vec::new(),
            doc_comment: None,
            file_name: None,
        }
    }
    
    /// 检查是否为 Backed Enum
    pub fn is_backed(&self) -> bool {
        self.backing_type.is_some()
    }
    
    /// 查找 Case
    pub fn find_case(&self, name: &str) -> Option<&EnumCaseMetadata> {
        self.cases.iter().find(|c| c.name == name)
    }
}

/// 枚举 Case 元数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumCaseMetadata {
    /// Case 名称
    pub name: String,
    /// Case 值（仅 Backed Enum）
    pub value: Option<String>,
    /// 注解列表
    pub attributes: Vec<AttributeMetadata>,
}

impl EnumCaseMetadata {
    /// 创建新的 Case 元数据
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: None,
            attributes: Vec::new(),
        }
    }
}

/// 类型元数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeMetadata {
    /// 类型名称
    pub name: String,
    /// 是否允许 null
    pub allows_null: bool,
    /// 是否为内置类型
    pub is_builtin: bool,
}

impl TypeMetadata {
    /// 创建新的类型元数据
    pub fn new(name: &str) -> Self {
        let is_builtin = Self::is_builtin_type(name);
        Self {
            name: name.to_string(),
            allows_null: false,
            is_builtin,
        }
    }
    
    /// 创建可空类型
    pub fn nullable(name: &str) -> Self {
        let is_builtin = Self::is_builtin_type(name);
        Self {
            name: name.to_string(),
            allows_null: true,
            is_builtin,
        }
    }
    
    /// 检查是否为内置类型
    fn is_builtin_type(name: &str) -> bool {
        matches!(
            name.to_lowercase().as_str(),
            "int" | "integer" |
            "float" | "double" |
            "string" |
            "bool" | "boolean" |
            "array" |
            "object" |
            "callable" |
            "iterable" |
            "void" |
            "null" |
            "mixed" |
            "never" |
            "true" | "false"
        )
    }
}

/// 注解元数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeMetadata {
    /// 注解类名
    pub name: String,
    /// 位置参数
    pub arguments: Vec<String>,
    /// 命名参数
    pub named_arguments: HashMap<String, String>,
    /// 目标类型
    pub target: AttributeTarget,
}

impl AttributeMetadata {
    /// 创建新的注解元数据
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            arguments: Vec::new(),
            named_arguments: HashMap::new(),
            target: AttributeTarget::Class,
        }
    }
}

/// 注解目标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttributeTarget {
    /// 类
    Class,
    /// 方法
    Method,
    /// 属性
    Property,
    /// 函数
    Function,
    /// 参数
    Parameter,
    /// 类常量
    ClassConstant,
    /// 枚举 Case
    EnumCase,
}

/// 可见性枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Visibility {
    /// public
    Public,
    /// protected
    Protected,
    /// private
    Private,
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Visibility::Public => write!(f, "public"),
            Visibility::Protected => write!(f, "protected"),
            Visibility::Private => write!(f, "private"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reflection_metadata() {
        let metadata = ReflectionMetadata::new();
        
        // 注册类
        let class = ClassMetadata::new("App\\Service\\UserService");
        metadata.register_class(class);
        
        // 检查类是否存在
        assert!(metadata.has_class("App\\Service\\UserService"));
        
        // 获取类
        let retrieved = metadata.get_class("App\\Service\\UserService");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().short_name, "UserService");
    }
    
    #[test]
    fn test_class_metadata() {
        let mut class = ClassMetadata::new("App\\Entity\\User");
        
        // 设置属性
        class.is_final = true;
        class.parent_class = Some("App\\Entity\\BaseEntity".to_string());
        class.interfaces.push("App\\Contract\\UserInterface".to_string());
        
        // 添加方法
        let method = MethodMetadata::new("getName", &class.name);
        class.methods.push(method);
        
        // 添加属性
        let property = PropertyMetadata::new("name", &class.name);
        class.properties.push(property);
        
        // 测试查找
        assert!(class.find_method("getName").is_some());
        assert!(class.find_property("name").is_some());
        assert!(class.implements("App\\Contract\\UserInterface"));
    }
    
    #[test]
    fn test_method_metadata() {
        let mut method = MethodMetadata::new("__construct", "App\\Service\\UserService");
        
        method.visibility = Visibility::Public;
        
        // 添加参数
        let param = ParameterMetadata::new("name", 0);
        method.parameters.push(param);
        
        assert!(method.is_constructor());
        assert!(!method.is_destructor());
    }
    
    #[test]
    fn test_type_metadata() {
        let int_type = TypeMetadata::new("int");
        assert!(int_type.is_builtin);
        assert!(!int_type.allows_null);
        
        let nullable_string = TypeMetadata::nullable("string");
        assert!(nullable_string.is_builtin);
        assert!(nullable_string.allows_null);
        
        let class_type = TypeMetadata::new("App\\Entity\\User");
        assert!(!class_type.is_builtin);
    }
}
