//! 解释器值类型模块
//!
//! 定义解释器中使用的所有值类型和执行上下文
//! PHP 是动态类型语言，所有值在运行时都表示为 Value 枚举
//! 支持完整的 PHP 值类型：null/bool/int/float/string/array/object/callable/resource

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 解释器值类型
/// PHP 的所有值在运行时都表示为此枚举的变体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    /// null 值
    Null,
    /// 布尔值
    Bool(bool),
    /// 整数值（i64 对应 PHP 的 int）
    Int(i64),
    /// 浮点数值（f64 对应 PHP 的 float）
    Float(f64),
    /// 字符串值
    String(String),
    /// 索引数组（PHP 的 [1, 2, 3]）
    IndexedArray(Vec<Value>),
    /// 关联数组（PHP 的 ['key' => 'value']）
    AssociativeArray(Vec<(String, Value)>),
    /// 对象实例
    Object(ObjectInstance),
    /// 可调用（闭包或函数引用）
    Callable(CallableValue),
    /// 资源类型（文件句柄、数据库连接等）
    Resource(String),
}

/// 对象实例
/// 运行时创建的对象，包含类名和属性值
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObjectInstance {
    /// 类名（含命名空间）
    pub class_name: String,
    /// 属性值映射
    pub properties: HashMap<String, Value>,
}

/// 可调用值
/// 表示一个可以被调用的函数或方法
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CallableValue {
    /// 命名函数调用
    Function {
        /// 函数名（含命名空间）
        name: String,
    },
    /// 类方法调用
    Method {
        /// 类名
        class_name: String,
        /// 方法名
        method_name: String,
    },
    /// 闭包（捕获上下文变量）
    Closure {
        /// 闭包唯一标识
        id: String,
        /// 闭包参数名列表
        params: Vec<String>,
        /// 捕获的变量（use 子句）
        captured: HashMap<String, Value>,
        /// 闭包来源文件路径
        file_path: String,
        /// 是否按引用传递
        by_ref: bool,
    },
    /// 箭头函数（fn($x) => expr）
    Arrow {
        /// 箭头函数唯一标识
        id: String,
        /// 参数名列表
        params: Vec<String>,
        /// 捕获的变量（自动捕获外层变量）
        captured: HashMap<String, Value>,
        /// 箭头函数来源文件路径
        file_path: String,
    },
}

impl Value {
    /// 创建 null 值
    pub fn null() -> Self {
        Value::Null
    }

    /// 创建布尔值
    pub fn bool(b: bool) -> Self {
        Value::Bool(b)
    }

    /// 创建整数值
    pub fn int(i: i64) -> Self {
        Value::Int(i)
    }

    /// 创建浮点数值
    pub fn float(f: f64) -> Self {
        Value::Float(f)
    }

    /// 创建字符串值
    pub fn string(s: &str) -> Self {
        Value::String(s.to_string())
    }

    /// 判断是否为 null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// 判断是否为真值（PHP 的 truthy 规则）
    /// - null, false, 0, 0.0, "", "0", 空数组 为假
    /// - 其他为真
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty() && s != "0",
            Value::IndexedArray(arr) => !arr.is_empty(),
            Value::AssociativeArray(map) => !map.is_empty(),
            Value::Object(_) => true,
            Value::Callable(_) => true,
            Value::Resource(_) => true,
        }
    }

    /// 转换为字符串
    pub fn to_string_value(&self) -> String {
        match self {
            Value::Null => "".to_string(),
            Value::Bool(b) => if *b { "1" } else { "" }.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => format!("{}", f),
            Value::String(s) => s.clone(),
            Value::IndexedArray(arr) => format!("Array({})", arr.len()),
            Value::AssociativeArray(map) => format!("Array({})", map.len()),
            Value::Object(obj) => format!("{} Object", obj.class_name),
            Value::Callable(_) => "Callable".to_string(),
            Value::Resource(r) => format!("Resource({})", r),
        }
    }

    /// 转换为整数
    pub fn to_int(&self) -> i64 {
        match self {
            Value::Null => 0,
            Value::Bool(b) => if *b { 1 } else { 0 },
            Value::Int(i) => *i,
            Value::Float(f) => *f as i64,
            Value::String(s) => s.trim().parse().unwrap_or(0),
            _ => 0,
        }
    }

    /// 转换为浮点数
    pub fn to_float(&self) -> f64 {
        match self {
            Value::Null => 0.0,
            Value::Bool(b) => if *b { 1.0 } else { 0.0 },
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            Value::String(s) => s.trim().parse().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    /// 转换为布尔值
    pub fn to_bool(&self) -> bool {
        self.is_truthy()
    }

    /// 获取关联数组中的值
    pub fn get_by_key(&self, key: &str) -> Option<Value> {
        match self {
            Value::AssociativeArray(map) => {
                map.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
            }
            _ => None,
        }
    }

    /// 设置关联数组中的值
    pub fn set_by_key(&mut self, key: &str, value: Value) {
        if let Value::AssociativeArray(map) = self {
            if let Some(entry) = map.iter_mut().find(|(k, _)| k == key) {
                entry.1 = value;
            } else {
                map.push((key.to_string(), value));
            }
        }
    }

    /// 获取数组长度
    pub fn count(&self) -> usize {
        match self {
            Value::IndexedArray(arr) => arr.len(),
            Value::AssociativeArray(map) => map.len(),
            Value::Null => 0,
            _ => 1,
        }
    }

    /// 类型名称（用于错误信息）
    pub fn type_name(&self) -> &str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::IndexedArray(_) | Value::AssociativeArray(_) => "array",
            Value::Object(_) => "object",
            Value::Callable(_) => "callable",
            Value::Resource(_) => "resource",
        }
    }

    /// 转换为 serde_json::Value
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            Value::Null => serde_json::Value::Null,
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::Int(i) => serde_json::json!(*i),
            Value::Float(f) => serde_json::json!(*f),
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::IndexedArray(arr) => {
                serde_json::Value::Array(arr.iter().map(|v| v.to_json_value()).collect())
            }
            Value::AssociativeArray(map) => {
                let mut obj = serde_json::Map::new();
                for (key, val) in map {
                    obj.insert(key.clone(), val.to_json_value());
                }
                serde_json::Value::Object(obj)
            }
            Value::Object(obj) => {
                let mut m = serde_json::Map::new();
                m.insert("_class".to_string(), serde_json::Value::String(obj.class_name.clone()));
                for (key, val) in &obj.properties {
                    m.insert(key.clone(), val.to_json_value());
                }
                serde_json::Value::Object(m)
            }
            Value::Callable(_) => serde_json::Value::String("Callable".to_string()),
            Value::Resource(r) => serde_json::Value::String(format!("Resource({})", r)),
        }
    }

    /// 转换为 JSON 字符串
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(&self.to_json_value()).unwrap_or_else(|_| "null".to_string())
    }

    /// 类型转换（对应 PHP 的 (int)$x 等强制转换）
    pub fn cast(&self, kind: &CastKind) -> Value {
        match kind {
            CastKind::Int => Value::Int(self.to_int()),
            CastKind::Float => Value::Float(self.to_float()),
            CastKind::String => Value::String(self.to_string_value()),
            CastKind::Bool => Value::Bool(self.is_truthy()),
            CastKind::Array => match self {
                Value::Null => Value::IndexedArray(Vec::new()),
                Value::IndexedArray(_) | Value::AssociativeArray(_) => self.clone(),
                v => Value::IndexedArray(vec![v.clone()]),
            },
            CastKind::Object => match self {
                Value::Object(_) => self.clone(),
                Value::Null => Value::Object(ObjectInstance {
                    class_name: "stdClass".to_string(),
                    properties: HashMap::new(),
                }),
                Value::AssociativeArray(map) => {
                    let mut props = HashMap::new();
                    for (k, v) in map {
                        props.insert(k.clone(), v.clone());
                    }
                    Value::Object(ObjectInstance {
                        class_name: "stdClass".to_string(),
                        properties: props,
                    })
                }
                v => Value::Object(ObjectInstance {
                    class_name: "stdClass".to_string(),
                    properties: {
                        let mut m = HashMap::new();
                        m.insert("scalar".to_string(), v.clone());
                        m
                    },
                }),
            },
            CastKind::Unset => Value::Null,
        }
    }
}

/// 类型转换种类
/// 对应 PHP 的 (int)$x, (string)$x 等强制类型转换
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastKind {
    Int,
    Float,
    String,
    Bool,
    Array,
    Object,
    Unset,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_value())
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

/// 执行上下文
/// 保存当前执行环境中的变量、引用、命名空间等
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// 局部变量表
    pub variables: HashMap<String, Value>,
    /// 当前对象实例（$this）
    pub this: Option<ObjectInstance>,
    /// 当前命名空间
    pub namespace: Option<String>,
    /// use 导入映射（别名 → 完整名称）
    pub use_map: HashMap<String, String>,
    /// 父级上下文引用（用于闭包捕获变量）
    pub parent: Option<Box<ExecutionContext>>,
}

impl ExecutionContext {
    /// 创建新的执行上下文
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            this: None,
            namespace: None,
            use_map: HashMap::new(),
            parent: None,
        }
    }

    /// 创建带命名空间的执行上下文
    pub fn with_namespace(namespace: &str) -> Self {
        Self {
            variables: HashMap::new(),
            this: None,
            namespace: Some(namespace.to_string()),
            use_map: HashMap::new(),
            parent: None,
        }
    }

    /// 获取变量值（支持向上查找父级上下文）
    pub fn get_var(&self, name: &str) -> Option<&Value> {
        match self.variables.get(name) {
            Some(v) => Some(v),
            None => {
                if let Some(ref parent) = self.parent {
                    parent.get_var(name)
                } else {
                    None
                }
            }
        }
    }

    /// 设置变量值
    pub fn set_var(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    /// 检查变量是否存在
    pub fn has_var(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// 移除变量
    pub fn remove_var(&mut self, name: &str) -> Option<Value> {
        self.variables.remove(name)
    }

    /// 捕获当前上下文中指定变量的值
    /// 用于闭包的 use 子句
    pub fn capture_vars(&self, var_names: &[String]) -> HashMap<String, Value> {
        let mut captured = HashMap::new();
        for name in var_names {
            if let Some(value) = self.get_var(name) {
                captured.insert(name.clone(), value.clone());
            }
        }
        captured
    }

    /// 捕获当前上下文中所有变量
    /// 用于箭头函数的自动变量捕获
    pub fn capture_all_vars(&self) -> HashMap<String, Value> {
        let mut captured = self.variables.clone();
        if let Some(ref parent) = self.parent {
            for (k, v) in parent.capture_all_vars() {
                if !captured.contains_key(&k) {
                    captured.insert(k, v);
                }
            }
        }
        captured
    }

    /// 解析类名（处理命名空间和 use 别名）
    pub fn resolve_class_name(&self, name: &str) -> String {
        if name.contains('\\') {
            return name.to_string();
        }
        if let Some(full) = self.use_map.get(name) {
            return full.clone();
        }
        if let Some(ref ns) = self.namespace {
            return format!("{}\\{}", ns, name);
        }
        name.to_string()
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行结果
/// 表示一段代码执行后的结果
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// 正常执行完成，返回值
    Value(Value),
    /// return 语句
    Return(Value),
    /// break 语句（可选层级）
    Break(Option<i32>),
    /// continue 语句（可选层级）
    Continue(Option<i32>),
    /// throw 语句（异常值）
    Throw(Value),
    /// 函数调用请求
    FunctionCall {
        /// 函数/方法名
        name: String,
        /// 参数列表
        args: Vec<Value>,
    },
}

impl ExecutionResult {
    /// 提取返回值
    pub fn into_value(self) -> Value {
        match self {
            ExecutionResult::Value(v) | ExecutionResult::Return(v) => v,
            _ => Value::Null,
        }
    }

    /// 判断是否为 return
    pub fn is_return(&self) -> bool {
        matches!(self, ExecutionResult::Return(_))
    }

    /// 判断是否为 break
    pub fn is_break(&self) -> bool {
        matches!(self, ExecutionResult::Break(_))
    }

    /// 判断是否为 continue
    pub fn is_continue(&self) -> bool {
        matches!(self, ExecutionResult::Continue(_))
    }

    /// 判断是否为 throw
    pub fn is_throw(&self) -> bool {
        matches!(self, ExecutionResult::Throw(_))
    }
}
