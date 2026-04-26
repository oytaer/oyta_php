//! 符号表类型定义模块
//!
//! 定义了 OYTAPHP 运行时需要的所有符号类型
//! 包括：类定义、方法定义、属性定义、函数定义、常量定义等
//! 这些类型用于在解析 PHP 文件后存储符号信息，供后续路由、中间件、依赖注入等模块使用

use serde::{Deserialize, Serialize};
use std::fmt;

/// 可见性枚举
/// 对应 PHP 的 public / protected / private 访问修饰符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Visibility {
    /// public - 公开，任何地方都可访问
    Public,
    /// protected - 受保护，仅本类及子类可访问
    Protected,
    /// private - 私有，仅本类可访问
    Private,
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Visibility::Public => write!(f, "public"),
            Visibility::Protected => write!(f, "protected"),
            Visibility::Private => write!(f, "private"),
        }
    }
}

/// 参数定义
/// 对应 PHP 函数/方法的参数声明
/// 如: public function index(string $name = 'think')
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDef {
    /// 参数名称（含 $ 前缀），如 "$name"
    pub name: String,
    /// 参数类型提示，如 "string"、"int"、"App\\User"
    /// None 表示无类型提示
    pub type_hint: Option<String>,
    /// 参数默认值（PHP 代码字符串形式）
    /// None 表示无默认值（必填参数）
    pub default_value: Option<String>,
    /// 是否引用传递（&$var）
    pub by_ref: bool,
    /// 是否可变参数（...$vars）
    pub variadic: bool,
}

/// 方法定义
/// 对应 PHP 类中的方法声明
/// 如: public function index(): string { ... }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodDef {
    /// 方法名称，如 "index"、"hello"
    pub name: String,
    /// 可见性修饰符
    pub visibility: Visibility,
    /// 是否为静态方法（static）
    pub is_static: bool,
    /// 是否为抽象方法（abstract），无方法体
    pub is_abstract: bool,
    /// 是否为最终方法（final），不可被子类覆盖
    pub is_final: bool,
    /// 返回类型提示，如 "string"、"void"
    /// None 表示无返回类型声明
    pub return_type: Option<String>,
    /// 参数列表
    pub params: Vec<ParamDef>,
    /// 方法体语句数量（用于判断方法是否为空方法）
    /// None 表示抽象方法，无方法体
    pub body_stmt_count: Option<usize>,
}

/// 属性定义
/// 对应 PHP 类中的属性声明
/// 如: protected $name = 'user';
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDef {
    /// 属性名称（含 $ 前缀），如 "$name"
    pub name: String,
    /// 可见性修饰符
    pub visibility: Visibility,
    /// 是否为静态属性（static）
    pub is_static: bool,
    /// 是否为只读属性（readonly，PHP 8.1+）
    pub is_readonly: bool,
    /// 属性类型提示，如 "string"、"int"
    /// None 表示无类型提示
    pub type_hint: Option<String>,
    /// 属性默认值（PHP 代码字符串形式）
    /// None 表示无默认值
    pub default_value: Option<String>,
}

/// 类常量定义
/// 对应 PHP 类中的 const 声明
/// 如: const MAX_AGE = 100;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassConstDef {
    /// 常量名称，如 "MAX_AGE"
    pub name: String,
    /// 可见性修饰符（PHP 8.1+ 支持常量可见性）
    pub visibility: Option<Visibility>,
    /// 常量值（PHP 代码字符串形式）
    pub value: Option<String>,
}

/// 类定义
/// 对应 PHP 中的 class 声明
/// 存储类的完整信息，包括命名空间、继承、实现、成员等
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDef {
    /// 完整类名（含命名空间），如 "app\\controller\\Index"
    pub full_name: String,
    /// 类名（不含命名空间），如 "Index"
    pub name: String,
    /// 命名空间，如 "app\\controller"
    /// None 表示全局命名空间
    pub namespace: Option<String>,
    /// 是否为抽象类
    pub is_abstract: bool,
    /// 是否为最终类
    pub is_final: bool,
    /// 是否为只读类（PHP 8.2+）
    pub is_readonly: bool,
    /// 继承的父类名称（含命名空间）
    /// None 表示不继承（或隐式继承 stdClass）
    pub extends: Option<String>,
    /// 实现的接口列表（含命名空间）
    pub implements: Vec<String>,
    /// 使用的 trait 列表（含命名空间）
    pub traits: Vec<String>,
    /// 方法定义列表
    pub methods: Vec<MethodDef>,
    /// 属性定义列表
    pub properties: Vec<PropertyDef>,
    /// 类常量定义列表
    pub constants: Vec<ClassConstDef>,
    /// 源文件路径（绝对路径）
    pub file_path: String,
}

impl ClassDef {
    /// 查找指定名称的方法
    /// 方法名不区分大小写（与 PHP 行为一致）
    pub fn find_method(&self, name: &str) -> Option<&MethodDef> {
        let name_lower = name.to_lowercase();
        self.methods.iter().find(|m| m.name.to_lowercase() == name_lower)
    }

    /// 查找指定名称的属性
    /// 属性名含 $ 前缀，如 "$name"
    pub fn find_property(&self, name: &str) -> Option<&PropertyDef> {
        self.properties.iter().find(|p| p.name == name)
    }

    /// 判断是否继承自指定类
    /// 检查 extends 字段是否匹配（不区分大小写）
    pub fn extends_class(&self, class_name: &str) -> bool {
        self.extends.as_ref().map_or(false, |e| e.to_lowercase() == class_name.to_lowercase())
    }

    /// 判断是否实现了指定接口
    pub fn implements_interface(&self, interface_name: &str) -> bool {
        self.implements.iter().any(|i| i.to_lowercase() == interface_name.to_lowercase())
    }

    /// 判断是否使用了指定 trait
    pub fn uses_trait(&self, trait_name: &str) -> bool {
        self.traits.iter().any(|t| t.to_lowercase() == trait_name.to_lowercase())
    }

    /// 判断是否为控制器类
    /// 检查命名空间是否包含 "controller" 或继承自控制器基类
    pub fn is_controller(&self) -> bool {
        if let Some(ns) = &self.namespace {
            ns.to_lowercase().contains("controller")
        } else {
            false
        }
    }

    /// 判断是否为模型类
    /// 检查命名空间是否包含 "model" 或继承自 Model 基类
    pub fn is_model(&self) -> bool {
        if let Some(ns) = &self.namespace {
            ns.to_lowercase().contains("model")
        } else {
            false
        }
    }

    /// 判断是否为中间件类
    /// 检查命名空间是否包含 "middleware"
    pub fn is_middleware(&self) -> bool {
        if let Some(ns) = &self.namespace {
            ns.to_lowercase().contains("middleware")
        } else {
            false
        }
    }

    /// 判断是否为监听器类
    /// 通过命名空间或类名中包含 "listener" 来判断
    pub fn is_listener(&self) -> bool {
        if let Some(ns) = &self.namespace {
            ns.to_lowercase().contains("listener")
        } else {
            self.name.to_lowercase().contains("listener")
        }
    }
}

/// 函数定义
/// 对应 PHP 中的全局函数声明
/// 如: function json_encode($data): string { ... }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuncDef {
    /// 函数名称
    pub name: String,
    /// 所属命名空间
    /// None 表示全局命名空间
    pub namespace: Option<String>,
    /// 完整函数名（含命名空间），如 "app\\common\\json_success"
    pub full_name: String,
    /// 返回类型提示
    pub return_type: Option<String>,
    /// 参数列表
    pub params: Vec<ParamDef>,
    /// 是否引用返回
    pub by_ref: bool,
    /// 源文件路径
    pub file_path: String,
}

/// 常量定义
/// 对应 PHP 中的顶层 const 声明
/// 如: const APP_VERSION = '1.0.0';
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstDef {
    /// 常量名称
    pub name: String,
    /// 所属命名空间
    pub namespace: Option<String>,
    /// 常量值（PHP 代码字符串形式）
    pub value: Option<String>,
    /// 源文件路径
    pub file_path: String,
}

/// Use 导入项
/// 对应 PHP 中的 use 语句
/// 如: use oyta\Model; 或 use oyta\Model as BaseModel;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseItemDef {
    /// 导入的完整名称，如 "oyta\\Model"
    pub full_name: String,
    /// 别名（as 后的名称），如 "BaseModel"
    /// None 表示无别名，使用原始名称
    pub alias: Option<String>,
    /// 导入类型
    pub kind: UseKind,
}

/// Use 导入类型
/// 对应 PHP 中 use / use function / use const
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UseKind {
    /// 普通导入（类/接口/trait/枚举）
    Normal,
    /// 函数导入（use function）
    Function,
    /// 常量导入（use const）
    Const,
}

/// 命名空间定义
/// 对应 PHP 中的 namespace 声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceDef {
    /// 命名空间名称，如 "app\\controller"
    /// None 表示全局命名空间
    pub name: Option<String>,
    /// 该命名空间下的 use 导入列表
    pub use_items: Vec<UseItemDef>,
    /// 源文件路径
    pub file_path: String,
}

/// 配置值类型
/// 用于解析 PHP 配置文件（return [] 格式）后的值表示
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    /// 字符串值
    String(String),
    /// 整数值
    Int(i64),
    /// 浮点数值
    Float(f64),
    /// 布尔值
    Bool(bool),
    /// null 值
    Null,
    /// 数组值（索引数组）
    /// 对应 PHP 的 [1, 2, 3] 格式
    IndexedArray(Vec<ConfigValue>),
    /// 关联数组值
    /// 对应 PHP 的 ['key' => 'value'] 格式
    AssociativeArray(Vec<(String, ConfigValue)>),
}

impl ConfigValue {
    /// 尝试将配置值转换为字符串
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// 尝试将配置值转换为整数
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ConfigValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// 尝试将配置值转换为布尔值
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// 尝试获取关联数组中指定键的值
    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        match self {
            ConfigValue::AssociativeArray(map) => {
                map.iter().find(|(k, _)| k == key).map(|(_, v)| v)
            }
            _ => None,
        }
    }

    /// 判断是否为空值（null 或空数组）
    pub fn is_empty(&self) -> bool {
        match self {
            ConfigValue::Null => true,
            ConfigValue::IndexedArray(arr) => arr.is_empty(),
            ConfigValue::AssociativeArray(map) => map.is_empty(),
            ConfigValue::String(s) => s.is_empty(),
            _ => false,
        }
    }
}

/// 接口定义
/// 对应 PHP 中的 interface 声明
/// 如: interface Renderable { public function render(): string; }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceDef {
    /// 完整接口名（含命名空间），如 "app\\contracts\\Renderable"
    pub full_name: String,
    /// 接口名（不含命名空间），如 "Renderable"
    pub name: String,
    /// 命名空间，如 "app\\contracts"
    /// None 表示全局命名空间
    pub namespace: Option<String>,
    /// 继承的父接口列表（含命名空间）
    /// PHP 接口支持多继承：interface A extends B, C
    pub extends: Vec<String>,
    /// 接口中定义的方法签名列表
    /// 接口方法默认为 public abstract
    pub methods: Vec<MethodDef>,
    /// 接口中定义的常量列表
    /// PHP 接口可以包含常量：interface A { const VERSION = '1.0'; }
    pub constants: Vec<ClassConstDef>,
    /// 源文件路径
    pub file_path: String,
}

impl InterfaceDef {
    /// 查找指定名称的方法签名
    /// 方法名不区分大小写（与 PHP 行为一致）
    pub fn find_method(&self, name: &str) -> Option<&MethodDef> {
        let name_lower = name.to_lowercase();
        self.methods.iter().find(|m| m.name.to_lowercase() == name_lower)
    }

    /// 判断是否继承了指定接口
    pub fn extends_interface(&self, interface_name: &str) -> bool {
        self.extends.iter().any(|e| e.to_lowercase() == interface_name.to_lowercase())
    }
}

/// Trait 定义
/// 对应 PHP 中的 trait 声明
/// 如: trait SoftDelete { protected function delete() { ... } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitDef {
    /// 完整 Trait 名（含命名空间），如 "app\\traits\\SoftDelete"
    pub full_name: String,
    /// Trait 名（不含命名空间），如 "SoftDelete"
    pub name: String,
    /// 命名空间，如 "app\\traits"
    /// None 表示全局命名空间
    pub namespace: Option<String>,
    /// Trait 使用的其他 trait 列表
    pub traits: Vec<String>,
    /// Trait 中定义的方法列表
    pub methods: Vec<MethodDef>,
    /// Trait 中定义的属性列表
    pub properties: Vec<PropertyDef>,
    /// Trait 中定义的常量列表（PHP 8.2+）
    pub constants: Vec<ClassConstDef>,
    /// 源文件路径
    pub file_path: String,
}

impl TraitDef {
    /// 查找指定名称的方法
    /// 方法名不区分大小写
    pub fn find_method(&self, name: &str) -> Option<&MethodDef> {
        let name_lower = name.to_lowercase();
        self.methods.iter().find(|m| m.name.to_lowercase() == name_lower)
    }

    /// 查找指定名称的属性
    pub fn find_property(&self, name: &str) -> Option<&PropertyDef> {
        self.properties.iter().find(|p| p.name == name)
    }

    /// 判断是否使用了指定 trait
    pub fn uses_trait(&self, trait_name: &str) -> bool {
        self.traits.iter().any(|t| t.to_lowercase() == trait_name.to_lowercase())
    }
}

/// 枚举定义
/// 对应 PHP 8.1+ 中的 enum 声明
/// 如: enum Status: int { case Active = 1; case Inactive = 0; }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumDef {
    /// 完整枚举名（含命名空间），如 "app\\enums\\Status"
    pub full_name: String,
    /// 枚举名（不含命名空间），如 "Status"
    pub name: String,
    /// 命名空间，如 "app\\enums"
    /// None 表示全局命名空间
    pub namespace: Option<String>,
    /// 枚举的底层类型（Backed Enum）
    /// None 表示 Unit Enum（无底层类型）
    /// Some("int") 表示 int-backed enum
    /// Some("string") 表示 string-backed enum
    pub backing_type: Option<String>,
    /// 实现的接口列表
    pub implements: Vec<String>,
    /// 使用的 trait 列表
    pub traits: Vec<String>,
    /// 枚举的 case 列表
    pub cases: Vec<EnumCaseDef>,
    /// 枚举中定义的方法列表
    pub methods: Vec<MethodDef>,
    /// 枚举中定义的常量列表
    pub constants: Vec<ClassConstDef>,
    /// 源文件路径
    pub file_path: String,
}

/// 枚举 case 定义
/// 对应 PHP 8.1+ 中的 enum case 声明
/// 如: case Active = 1; 或 case Pending;（Unit Enum）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumCaseDef {
    /// case 名称，如 "Active"
    pub name: String,
    /// case 的值（仅 Backed Enum 有值）
    /// None 表示 Unit Enum 的 case
    pub value: Option<String>,
}

impl EnumDef {
    /// 查找指定名称的 case
    pub fn find_case(&self, name: &str) -> Option<&EnumCaseDef> {
        self.cases.iter().find(|c| c.name == name)
    }

    /// 查找指定名称的方法
    pub fn find_method(&self, name: &str) -> Option<&MethodDef> {
        let name_lower = name.to_lowercase();
        self.methods.iter().find(|m| m.name.to_lowercase() == name_lower)
    }

    /// 判断是否为 Backed Enum（有底层类型的枚举）
    pub fn is_backed(&self) -> bool {
        self.backing_type.is_some()
    }

    /// 判断是否实现了指定接口
    pub fn implements_interface(&self, interface_name: &str) -> bool {
        self.implements.iter().any(|i| i.to_lowercase() == interface_name.to_lowercase())
    }
}

/// 注解（Attribute）定义
/// 对应 PHP 8.0+ 中的 #[...] 声明
/// 如: #[Route('/hello', method: 'GET')]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDef {
    /// 注解的完整类名，如 "oyta\\annotation\\Route"
    /// 如果使用短名称，需要通过 use 导入解析
    pub full_name: String,
    /// 注解名称（可能是短名称），如 "Route"
    pub name: String,
    /// 位置参数列表
    /// 如 #[Route('/hello', method: 'GET')] 中的 '/hello'
    pub positional_args: Vec<String>,
    /// 命名参数列表
    /// 如 #[Route('/hello', method: 'GET')] 中的 method: 'GET'
    pub named_args: Vec<(String, String)>,
    /// 注解所附加的目标类型
    pub target: AttributeTarget,
}

/// 注解目标类型
/// 表示注解附加在哪种语法元素上
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttributeTarget {
    /// 附加在类上
    Class,
    /// 附加在方法上
    Method,
    /// 附加在属性上
    Property,
    /// 附加在函数上
    Function,
    /// 附加在参数上
    Parameter,
    /// 附加在类常量上
    ClassConst,
}

/// 文件解析结果
/// 单个 PHP 文件解析后产生的所有符号信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileParseResult {
    /// 源文件路径
    pub file_path: String,
    /// 文件中的命名空间声明
    pub namespace: Option<String>,
    /// 文件中的 use 导入列表
    pub use_items: Vec<UseItemDef>,
    /// 文件中定义的类列表
    pub classes: Vec<ClassDef>,
    /// 文件中定义的接口列表
    pub interfaces: Vec<InterfaceDef>,
    /// 文件中定义的 trait 列表
    pub traits: Vec<TraitDef>,
    /// 文件中定义的枚举列表
    pub enums: Vec<EnumDef>,
    /// 文件中定义的函数列表
    pub functions: Vec<FuncDef>,
    /// 文件中定义的常量列表
    pub constants: Vec<ConstDef>,
    /// 解析时遇到的错误列表
    pub errors: Vec<String>,
}
