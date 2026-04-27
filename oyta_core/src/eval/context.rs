//! 执行上下文模块
//! 
//! 本模块实现 eval() 执行的上下文管理，包括：
//! - 变量作用域
//! - 函数和类定义
//! - 常量管理
//! - 超全局变量

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

/// eval 变量类型
#[derive(Debug, Clone, PartialEq)]
pub enum EvalVariable {
    /// 整数
    Integer(i64),
    /// 浮点数
    Float(f64),
    /// 字符串
    String(String),
    /// 布尔值
    Boolean(bool),
    /// 数组
    Array(Vec<EvalVariable>),
    /// 关联数组
    AssociativeArray(HashMap<String, EvalVariable>),
    /// Null
    Null,
    /// 资源（句柄）
    Resource(usize),
    /// 对象
    Object {
        /// 类名
        class_name: String,
        /// 属性
        properties: HashMap<String, EvalVariable>,
    },
    /// 可调用
    Callable(String),
}

impl EvalVariable {
    /// 创建整数变量
    pub fn integer(val: i64) -> Self {
        EvalVariable::Integer(val)
    }

    /// 创建浮点数变量
    pub fn float(val: f64) -> Self {
        EvalVariable::Float(val)
    }

    /// 创建字符串变量
    pub fn string(val: &str) -> Self {
        EvalVariable::String(val.to_string())
    }

    /// 创建布尔值变量
    pub fn boolean(val: bool) -> Self {
        EvalVariable::Boolean(val)
    }

    /// 创建数组变量
    pub fn array(val: Vec<EvalVariable>) -> Self {
        EvalVariable::Array(val)
    }

    /// 创建关联数组变量
    pub fn associative_array(val: HashMap<String, EvalVariable>) -> Self {
        EvalVariable::AssociativeArray(val)
    }

    /// 创建 null 变量
    pub fn null() -> Self {
        EvalVariable::Null
    }

    /// 转换为布尔值
    pub fn to_bool(&self) -> bool {
        match self {
            EvalVariable::Integer(i) => *i != 0,
            EvalVariable::Float(f) => *f != 0.0,
            EvalVariable::String(s) => !s.is_empty() && s != "0",
            EvalVariable::Boolean(b) => *b,
            EvalVariable::Array(a) => !a.is_empty(),
            EvalVariable::AssociativeArray(a) => !a.is_empty(),
            EvalVariable::Null => false,
            EvalVariable::Resource(_) => true,
            EvalVariable::Object { .. } => true,
            EvalVariable::Callable(_) => true,
        }
    }

    /// 转换为字符串
    pub fn to_string(&self) -> String {
        match self {
            EvalVariable::Integer(i) => i.to_string(),
            EvalVariable::Float(f) => f.to_string(),
            EvalVariable::String(s) => s.clone(),
            EvalVariable::Boolean(b) => if *b { "1" } else { "" }.to_string(),
            EvalVariable::Array(_) => "Array".to_string(),
            EvalVariable::AssociativeArray(_) => "Array".to_string(),
            EvalVariable::Null => "".to_string(),
            EvalVariable::Resource(id) => format!("Resource id #{}", id),
            EvalVariable::Object { class_name, .. } => format!("{} Object", class_name),
            EvalVariable::Callable(name) => format!("{}()", name),
        }
    }

    /// 转换为整数
    pub fn to_integer(&self) -> i64 {
        match self {
            EvalVariable::Integer(i) => *i,
            EvalVariable::Float(f) => *f as i64,
            EvalVariable::String(s) => s.parse().unwrap_or(0),
            EvalVariable::Boolean(b) => if *b { 1 } else { 0 },
            EvalVariable::Array(a) => a.len() as i64,
            EvalVariable::AssociativeArray(a) => a.len() as i64,
            EvalVariable::Null => 0,
            EvalVariable::Resource(id) => *id as i64,
            EvalVariable::Object { .. } => 1,
            EvalVariable::Callable(_) => 1,
        }
    }

    /// 转换为浮点数
    pub fn to_float(&self) -> f64 {
        match self {
            EvalVariable::Integer(i) => *i as f64,
            EvalVariable::Float(f) => *f,
            EvalVariable::String(s) => s.parse().unwrap_or(0.0),
            EvalVariable::Boolean(b) => if *b { 1.0 } else { 0.0 },
            EvalVariable::Array(a) => a.len() as f64,
            EvalVariable::AssociativeArray(a) => a.len() as f64,
            EvalVariable::Null => 0.0,
            EvalVariable::Resource(id) => *id as f64,
            EvalVariable::Object { .. } => 1.0,
            EvalVariable::Callable(_) => 1.0,
        }
    }

    /// 检查是否为 null
    pub fn is_null(&self) -> bool {
        matches!(self, EvalVariable::Null)
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        match self {
            EvalVariable::String(s) => s.is_empty(),
            EvalVariable::Array(a) => a.is_empty(),
            EvalVariable::AssociativeArray(a) => a.is_empty(),
            EvalVariable::Null => true,
            EvalVariable::Integer(i) => *i == 0,
            EvalVariable::Float(f) => *f == 0.0,
            EvalVariable::Boolean(b) => !b,
            _ => false,
        }
    }

    /// 获取类型名称
    pub fn type_name(&self) -> &'static str {
        match self {
            EvalVariable::Integer(_) => "integer",
            EvalVariable::Float(_) => "double",
            EvalVariable::String(_) => "string",
            EvalVariable::Boolean(_) => "boolean",
            EvalVariable::Array(_) => "array",
            EvalVariable::AssociativeArray(_) => "array",
            EvalVariable::Null => "NULL",
            EvalVariable::Resource(_) => "resource",
            EvalVariable::Object { .. } => "object",
            EvalVariable::Callable(_) => "callable",
        }
    }
}

impl Default for EvalVariable {
    fn default() -> Self {
        EvalVariable::Null
    }
}

/// eval 作用域
#[derive(Debug, Clone)]
pub struct EvalScope {
    /// 变量表
    pub variables: HashMap<String, EvalVariable>,
    /// 引用变量
    pub references: HashMap<String, String>,
    /// 静态变量
    pub static_vars: HashMap<String, EvalVariable>,
    /// 父作用域
    pub parent: Option<Rc<RefCell<EvalScope>>>,
    /// 作用域名称
    pub name: String,
    /// 是否为函数作用域
    pub is_function: bool,
}

impl EvalScope {
    /// 创建新的作用域
    pub fn new(name: &str) -> Self {
        EvalScope {
            variables: HashMap::new(),
            references: HashMap::new(),
            static_vars: HashMap::new(),
            parent: None,
            name: name.to_string(),
            is_function: false,
        }
    }

    /// 创建函数作用域
    pub fn function_scope(name: &str, parent: Rc<RefCell<EvalScope>>) -> Self {
        EvalScope {
            variables: HashMap::new(),
            references: HashMap::new(),
            static_vars: HashMap::new(),
            parent: Some(parent),
            name: name.to_string(),
            is_function: true,
        }
    }

    /// 设置变量
    pub fn set(&mut self, name: &str, value: EvalVariable) {
        // 检查是否是引用
        if let Some(ref_name) = self.references.get(name) {
            self.variables.insert(ref_name.clone(), value);
        } else {
            self.variables.insert(name.to_string(), value);
        }
    }

    /// 获取变量
    pub fn get(&self, name: &str) -> Option<EvalVariable> {
        if let Some(val) = self.variables.get(name) {
            return Some(val.clone());
        }
        // 检查引用
        if let Some(ref_name) = self.references.get(name) {
            return self.variables.get(ref_name).cloned();
        }
        // 查找父作用域
        if let Some(ref parent) = self.parent {
            if let Ok(parent_ref) = parent.try_borrow() {
                if let Some(val) = parent_ref.get(name) {
                    return Some(val);
                }
            }
        }
        None
    }

    /// 获取变量可变引用
    pub fn get_mut(&mut self, name: &str) -> Option<&mut EvalVariable> {
        self.variables.get_mut(name)
    }

    /// 检查变量是否存在
    pub fn has(&self, name: &str) -> bool {
        if self.variables.contains_key(name) {
            return true;
        }
        if let Some(ref parent) = self.parent {
            if let Ok(parent_ref) = parent.try_borrow() {
                return parent_ref.has(name);
            }
        }
        false
    }

    /// 删除变量
    pub fn remove(&mut self, name: &str) -> Option<EvalVariable> {
        self.variables.remove(name)
    }

    /// 设置引用
    pub fn set_reference(&mut self, name: &str, target: &str) {
        self.references.insert(name.to_string(), target.to_string());
    }

    /// 设置静态变量
    pub fn set_static(&mut self, name: &str, value: EvalVariable) {
        self.static_vars.insert(name.to_string(), value);
    }

    /// 获取静态变量
    pub fn get_static(&self, name: &str) -> Option<&EvalVariable> {
        self.static_vars.get(name)
    }

    /// 获取所有变量名
    pub fn variable_names(&self) -> Vec<&str> {
        self.variables.keys().map(|s| s.as_str()).collect()
    }

    /// 清空变量
    pub fn clear(&mut self) {
        self.variables.clear();
        self.references.clear();
    }
}

impl Default for EvalScope {
    fn default() -> Self {
        Self::new("global")
    }
}

/// eval 执行上下文
#[derive(Debug)]
pub struct EvalContext {
    /// 全局作用域
    pub global_scope: Rc<RefCell<EvalScope>>,
    /// 当前作用域栈
    pub scope_stack: Vec<Rc<RefCell<EvalScope>>>,
    /// 函数定义
    pub functions: HashMap<String, EvalFunction>,
    /// 类定义
    pub classes: HashMap<String, EvalClass>,
    /// 常量
    pub constants: HashMap<String, EvalVariable>,
    /// 超全局变量
    pub super_globals: HashMap<String, EvalVariable>,
    /// 包含路径
    pub include_paths: Vec<String>,
    /// 当前文件路径
    pub current_file: String,
    /// 当前行号
    pub current_line: usize,
    /// 错误处理器
    pub error_handler: Option<String>,
    /// 异常处理器
    pub exception_handler: Option<String>,
    /// 是否在严格模式
    pub strict_mode: bool,
    /// 执行时间限制（毫秒）
    pub time_limit: Option<u64>,
    /// 内存限制（字节）
    pub memory_limit: Option<usize>,
}

/// eval 函数定义
#[derive(Debug, Clone)]
pub struct EvalFunction {
    /// 函数名
    pub name: String,
    /// 参数列表
    pub parameters: Vec<EvalParameter>,
    /// 函数体
    pub body: String,
    /// 是否返回引用
    pub returns_reference: bool,
    /// 是否为内置函数
    pub is_builtin: bool,
    /// 静态变量
    pub static_vars: HashMap<String, EvalVariable>,
}

/// eval 参数定义
#[derive(Debug, Clone)]
pub struct EvalParameter {
    /// 参数名
    pub name: String,
    /// 类型提示
    pub type_hint: Option<String>,
    /// 默认值
    pub default_value: Option<EvalVariable>,
    /// 是否通过引用传递
    pub by_reference: bool,
    /// 是否可选
    pub optional: bool,
    /// 是否可变参数
    pub variadic: bool,
}

impl EvalParameter {
    /// 创建新的参数
    pub fn new(name: &str) -> Self {
        EvalParameter {
            name: name.to_string(),
            type_hint: None,
            default_value: None,
            by_reference: false,
            optional: false,
            variadic: false,
        }
    }

    /// 设置类型提示
    pub fn with_type(mut self, type_hint: &str) -> Self {
        self.type_hint = Some(type_hint.to_string());
        self
    }

    /// 设置默认值
    pub fn with_default(mut self, value: EvalVariable) -> Self {
        self.default_value = Some(value);
        self.optional = true;
        self
    }

    /// 设置为引用传递
    pub fn by_ref(mut self) -> Self {
        self.by_reference = true;
        self
    }

    /// 设置为可变参数
    pub fn as_variadic(mut self) -> Self {
        self.variadic = true;
        self
    }
}

/// eval 类定义
#[derive(Debug, Clone)]
pub struct EvalClass {
    /// 类名
    pub name: String,
    /// 父类名
    pub parent: Option<String>,
    /// 实现的接口
    pub interfaces: Vec<String>,
    /// 属性
    pub properties: HashMap<String, EvalProperty>,
    /// 方法
    pub methods: HashMap<String, EvalFunction>,
    /// 常量
    pub constants: HashMap<String, EvalVariable>,
    /// 是否为抽象类
    pub is_abstract: bool,
    /// 是否为最终类
    pub is_final: bool,
    /// 是否为接口
    pub is_interface: bool,
    /// 是否为 trait
    pub is_trait: bool,
    /// 使用的 traits
    pub traits: Vec<String>,
}

impl EvalClass {
    /// 创建新的类
    pub fn new(name: &str) -> Self {
        EvalClass {
            name: name.to_string(),
            parent: None,
            interfaces: Vec::new(),
            properties: HashMap::new(),
            methods: HashMap::new(),
            constants: HashMap::new(),
            is_abstract: false,
            is_final: false,
            is_interface: false,
            is_trait: false,
            traits: Vec::new(),
        }
    }

    /// 设置父类
    pub fn extends(mut self, parent: &str) -> Self {
        self.parent = Some(parent.to_string());
        self
    }

    /// 添加接口
    pub fn implements(mut self, interface: &str) -> Self {
        self.interfaces.push(interface.to_string());
        self
    }

    /// 添加属性
    pub fn add_property(&mut self, name: &str, property: EvalProperty) {
        self.properties.insert(name.to_string(), property);
    }

    /// 添加方法
    pub fn add_method(&mut self, name: &str, method: EvalFunction) {
        self.methods.insert(name.to_string(), method);
    }

    /// 添加常量
    pub fn add_constant(&mut self, name: &str, value: EvalVariable) {
        self.constants.insert(name.to_string(), value);
    }

    /// 检查方法是否存在
    pub fn has_method(&self, name: &str) -> bool {
        if self.methods.contains_key(name) {
            return true;
        }
        // 检查父类
        if let Some(ref parent) = self.parent {
            // 这里需要访问类注册表
        }
        false
    }

    /// 检查属性是否存在
    pub fn has_property(&self, name: &str) -> bool {
        self.properties.contains_key(name)
    }
}

/// eval 属性定义
#[derive(Debug, Clone)]
pub struct EvalProperty {
    /// 属性名
    pub name: String,
    /// 类型提示
    pub type_hint: Option<String>,
    /// 默认值
    pub default_value: Option<EvalVariable>,
    /// 可见性
    pub visibility: EvalVisibility,
    /// 是否为静态
    pub is_static: bool,
    /// 是否只读
    pub is_readonly: bool,
}

/// 可见性枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalVisibility {
    /// 公开
    Public,
    /// 保护
    Protected,
    /// 私有
    Private,
}

impl EvalProperty {
    /// 创建公开属性
    pub fn public(name: &str) -> Self {
        EvalProperty {
            name: name.to_string(),
            type_hint: None,
            default_value: None,
            visibility: EvalVisibility::Public,
            is_static: false,
            is_readonly: false,
        }
    }

    /// 创建保护属性
    pub fn protected(name: &str) -> Self {
        EvalProperty {
            name: name.to_string(),
            type_hint: None,
            default_value: None,
            visibility: EvalVisibility::Protected,
            is_static: false,
            is_readonly: false,
        }
    }

    /// 创建私有属性
    pub fn private(name: &str) -> Self {
        EvalProperty {
            name: name.to_string(),
            type_hint: None,
            default_value: None,
            visibility: EvalVisibility::Private,
            is_static: false,
            is_readonly: false,
        }
    }

    /// 设置类型
    pub fn with_type(mut self, type_hint: &str) -> Self {
        self.type_hint = Some(type_hint.to_string());
        self
    }

    /// 设置默认值
    pub fn with_default(mut self, value: EvalVariable) -> Self {
        self.default_value = Some(value);
        self
    }

    /// 设置为静态
    pub fn as_static(mut self) -> Self {
        self.is_static = true;
        self
    }

    /// 设置为只读
    pub fn as_readonly(mut self) -> Self {
        self.is_readonly = true;
        self
    }
}

impl EvalContext {
    /// 创建新的执行上下文
    pub fn new() -> Self {
        let global_scope = Rc::new(RefCell::new(EvalScope::new("global")));
        EvalContext {
            global_scope: global_scope.clone(),
            scope_stack: vec![global_scope],
            functions: HashMap::new(),
            classes: HashMap::new(),
            constants: HashMap::new(),
            super_globals: HashMap::new(),
            include_paths: vec![".".to_string()],
            current_file: "eval".to_string(),
            current_line: 1,
            error_handler: None,
            exception_handler: None,
            strict_mode: false,
            time_limit: None,
            memory_limit: None,
        }
    }

    /// 获取当前作用域
    pub fn current_scope(&self) -> Rc<RefCell<EvalScope>> {
        self.scope_stack.last().unwrap().clone()
    }

    /// 进入新作用域
    pub fn push_scope(&mut self, name: &str) {
        let parent = self.current_scope();
        let new_scope = Rc::new(RefCell::new(EvalScope::function_scope(name, parent)));
        self.scope_stack.push(new_scope);
    }

    /// 退出作用域
    pub fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    /// 设置变量
    pub fn set_variable(&mut self, name: &str, value: EvalVariable) {
        let scope = self.current_scope();
        scope.borrow_mut().set(name, value);
    }

    /// 获取变量
    pub fn get_variable(&self, name: &str) -> Option<EvalVariable> {
        let scope = self.current_scope();
        scope.borrow().get(name)
    }

    /// 检查变量是否存在
    pub fn has_variable(&self, name: &str) -> bool {
        let scope = self.current_scope();
        scope.borrow().has(name)
    }

    /// 设置全局变量
    pub fn set_global(&mut self, name: &str, value: EvalVariable) {
        self.global_scope.borrow_mut().set(name, value);
    }

    /// 获取全局变量
    pub fn get_global(&self, name: &str) -> Option<EvalVariable> {
        self.global_scope.borrow().get(name)
    }

    /// 设置超全局变量
    pub fn set_super_global(&mut self, name: &str, value: EvalVariable) {
        self.super_globals.insert(name.to_string(), value);
    }

    /// 获取超全局变量
    pub fn get_super_global(&self, name: &str) -> Option<&EvalVariable> {
        self.super_globals.get(name)
    }

    /// 定义常量
    pub fn define_constant(&mut self, name: &str, value: EvalVariable) {
        self.constants.insert(name.to_string(), value);
    }

    /// 获取常量
    pub fn get_constant(&self, name: &str) -> Option<&EvalVariable> {
        self.constants.get(name)
    }

    /// 检查常量是否存在
    pub fn has_constant(&self, name: &str) -> bool {
        self.constants.contains_key(name)
    }

    /// 定义函数
    pub fn define_function(&mut self, func: EvalFunction) {
        self.functions.insert(func.name.clone(), func);
    }

    /// 获取函数
    pub fn get_function(&self, name: &str) -> Option<&EvalFunction> {
        self.functions.get(name)
    }

    /// 检查函数是否存在
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// 定义类
    pub fn define_class(&mut self, class: EvalClass) {
        self.classes.insert(class.name.clone(), class);
    }

    /// 获取类
    pub fn get_class(&self, name: &str) -> Option<&EvalClass> {
        self.classes.get(name)
    }

    /// 检查类是否存在
    pub fn has_class(&self, name: &str) -> bool {
        self.classes.contains_key(name)
    }

    /// 设置当前文件
    pub fn set_file(&mut self, file: &str) {
        self.current_file = file.to_string();
    }

    /// 设置当前行号
    pub fn set_line(&mut self, line: usize) {
        self.current_line = line;
    }

    /// 设置时间限制
    pub fn set_time_limit(&mut self, ms: u64) {
        self.time_limit = Some(ms);
    }

    /// 设置内存限制
    pub fn set_memory_limit(&mut self, bytes: usize) {
        self.memory_limit = Some(bytes);
    }

    /// 启用严格模式
    pub fn enable_strict_mode(&mut self) {
        self.strict_mode = true;
    }

    /// 禁用严格模式
    pub fn disable_strict_mode(&mut self) {
        self.strict_mode = false;
    }

    /// 添加包含路径
    pub fn add_include_path(&mut self, path: &str) {
        self.include_paths.push(path.to_string());
    }

    /// 获取所有变量
    pub fn get_all_variables(&self) -> HashMap<String, EvalVariable> {
        let scope = self.current_scope();
        scope.borrow().variables.clone()
    }

    /// 导入变量到当前作用域
    pub fn import_variables(&mut self, vars: HashMap<String, EvalVariable>) {
        let scope = self.current_scope();
        for (name, value) in vars {
            scope.borrow_mut().set(&name, value);
        }
    }
}

impl Default for EvalContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_variable() {
        let v = EvalVariable::integer(42);
        assert_eq!(v.to_integer(), 42);
        assert!(v.to_bool());
    }

    #[test]
    fn test_eval_variable_string() {
        let v = EvalVariable::string("hello");
        assert_eq!(v.to_string(), "hello");
        assert!(v.to_bool());
    }

    #[test]
    fn test_eval_scope() {
        let mut scope = EvalScope::new("test");
        scope.set("x", EvalVariable::integer(10));
        assert!(scope.has("x"));
        assert_eq!(scope.get("x").unwrap().to_integer(), 10);
    }

    #[test]
    fn test_eval_context() {
        let mut ctx = EvalContext::new();
        ctx.set_variable("a", EvalVariable::string("test"));
        assert!(ctx.has_variable("a"));
    }

    #[test]
    fn test_eval_class() {
        let mut class = EvalClass::new("TestClass");
        class.add_property("name", EvalProperty::public("name"));
        assert!(class.has_property("name"));
    }
}
