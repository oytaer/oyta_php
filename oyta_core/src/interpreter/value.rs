//! 解释器值类型模块
//!
//! 定义解释器中使用的所有值类型和执行上下文
//! PHP 是动态类型语言，所有值在运行时都表示为 Value 枚举
//! 支持完整的 PHP 值类型：null/bool/int/float/string/array/object/callable/resource/generator/fiber

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 前向声明 Generator 和 Fiber 类型
// 这些类型在 generator.rs 和 fiber.rs 中定义
use super::generator::GeneratorValue;
use super::fiber::FiberValue;

/// 解释器值类型
/// PHP 的所有值在运行时都表示为此枚举的变体
/// 使用 Box 包装 Generator 和 Fiber 以避免递归类型无限大小
#[derive(Debug, Clone, PartialEq)]
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
    /// Generator 生成器（PHP 5.5+ yield 支持）
    Generator(GeneratorValue),
    /// Fiber 协程（PHP 8.1+ 协程支持）
    Fiber(FiberValue),
}

/// 为 Value 实现 Serialize trait
/// Generator 和 Fiber 序列化为字符串表示
impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // 根据值类型进行序列化
        match self {
            // 基本类型直接序列化
            Value::Null => serializer.serialize_none(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Int(i) => serializer.serialize_i64(*i),
            Value::Float(f) => serializer.serialize_f64(*f),
            Value::String(s) => serializer.serialize_str(s),
            // 数组类型序列化为序列
            Value::IndexedArray(arr) => arr.serialize(serializer),
            Value::AssociativeArray(map) => {
                // 使用 serde_json 的 map 格式序列化关联数组
                use serde::ser::SerializeMap;
                let mut map_ser = serializer.serialize_map(Some(map.len()))?;
                for (k, v) in map {
                    map_ser.serialize_entry(k, v)?;
                }
                map_ser.end()
            }
            // 对象序列化为包含类名的结构体
            Value::Object(obj) => obj.serialize(serializer),
            // 可调用序列化为字符串表示
            Value::Callable(_) => serializer.serialize_str("Callable"),
            // 资源序列化为字符串表示
            Value::Resource(r) => serializer.serialize_str(&format!("Resource({})", r)),
            // Generator 序列化为字符串表示
            Value::Generator(_) => serializer.serialize_str("Generator"),
            // Fiber 序列化为字符串表示
            Value::Fiber(_) => serializer.serialize_str("Fiber"),
        }
    }
}

/// 为 Value 实现 Deserialize trait
/// Generator 和 Fiber 反序列化为默认值（无法从 JSON 恢复）
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // 使用访问者模式反序列化
        use serde::de::{self, Visitor, MapAccess, SeqAccess};
        
        /// Value 反序列化访问者
        struct ValueVisitor;
        
        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;
            
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid PHP value type")
            }
            
            // 反序列化 null
            fn visit_none<E>(self) -> Result<Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Null)
            }
            
            // 反序列化布尔值
            fn visit_bool<E>(self, v: bool) -> Result<Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Bool(v))
            }
            
            // 反序列化整数
            fn visit_i64<E>(self, v: i64) -> Result<Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Int(v))
            }
            
            // 反序列化浮点数
            fn visit_f64<E>(self, v: f64) -> Result<Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Float(v))
            }
            
            // 反序列化字符串
            fn visit_str<E>(self, v: &str) -> Result<Value, E>
            where
                E: de::Error,
            {
                // 检查特殊字符串值
                match v {
                    "Generator" => Ok(Value::Null), // Generator 无法从 JSON 恢复
                    "Fiber" => Ok(Value::Null),     // Fiber 无法从 JSON 恢复
                    "Callable" => Ok(Value::Null),  // Callable 无法从 JSON 恢复
                    _ if v.starts_with("Resource(") => Ok(Value::Resource(v.to_string())),
                    _ => Ok(Value::String(v.to_string())),
                }
            }
            
            // 反序列化字符串（拥有所有权）
            fn visit_string<E>(self, v: String) -> Result<Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&v)
            }
            
            // 反序列化序列（索引数组）
            fn visit_seq<A>(self, mut seq: A) -> Result<Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut arr = Vec::new();
                // 遍历序列中的每个元素
                while let Some(elem) = seq.next_element()? {
                    arr.push(elem);
                }
                Ok(Value::IndexedArray(arr))
            }
            
            // 反序列化映射（关联数组或对象）
            fn visit_map<A>(self, mut map: A) -> Result<Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut entries = Vec::new();
                let mut has_class = false;
                let mut class_name = String::new();
                
                // 遍历映射中的每个键值对
                while let Some(key) = map.next_key::<String>()? {
                    let value: Value = map.next_value()?;
                    
                    // 检查是否是对象（包含 _class 字段）
                    if key == "_class" {
                        has_class = true;
                        if let Value::String(name) = value {
                            class_name = name;
                        }
                    } else {
                        entries.push((key, value));
                    }
                }
                
                // 如果有 _class 字段，则反序列化为对象
                if has_class {
                    let mut properties = HashMap::new();
                    for (k, v) in entries {
                        properties.insert(k, v);
                    }
                    Ok(Value::Object(ObjectInstance {
                        class_name,
                        properties,
                    }))
                } else {
                    // 否则反序列化为关联数组
                    Ok(Value::AssociativeArray(entries))
                }
            }
        }
        
        // 使用访问者反序列化
        deserializer.deserialize_any(ValueVisitor)
    }
}

/// 对象实例
/// 运行时创建的对象，包含类名和属性值
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectInstance {
    /// 类名（含命名空间）
    pub class_name: String,
    /// 属性值映射
    pub properties: HashMap<String, Value>,
}

/// 为 ObjectInstance 实现 Serialize trait
impl Serialize for ObjectInstance {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // 使用 serde 的 map 格式序列化对象
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.properties.len() + 1))?;
        // 添加类名字段
        map.serialize_entry("_class", &self.class_name)?;
        // 添加所有属性
        for (key, value) in &self.properties {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

/// 为 ObjectInstance 实现 Deserialize trait
impl<'de> Deserialize<'de> for ObjectInstance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // 使用访问者模式反序列化
        use serde::de::{self, MapAccess, Visitor};
        
        /// ObjectInstance 反序列化访问者
        struct ObjectInstanceVisitor;
        
        impl<'de> Visitor<'de> for ObjectInstanceVisitor {
            type Value = ObjectInstance;
            
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an object instance with _class field")
            }
            
            fn visit_map<A>(self, mut map: A) -> Result<ObjectInstance, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut class_name = String::new();
                let mut properties = HashMap::new();
                
                // 遍历映射中的每个键值对
                while let Some(key) = map.next_key::<String>()? {
                    if key == "_class" {
                        class_name = map.next_value()?;
                    } else {
                        let value: Value = map.next_value()?;
                        properties.insert(key, value);
                    }
                }
                
                Ok(ObjectInstance { class_name, properties })
            }
        }
        
        deserializer.deserialize_map(ObjectInstanceVisitor)
    }
}

/// 可调用值
/// 表示一个可以被调用的函数或方法
#[derive(Debug, Clone, PartialEq)]
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

/// 为 CallableValue 实现 Serialize trait
impl Serialize for CallableValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // 使用 serde 的 map 格式序列化可调用值
        use serde::ser::SerializeMap;
        
        match self {
            // 函数调用序列化为 { "type": "function", "name": "..." }
            CallableValue::Function { name } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "function")?;
                map.serialize_entry("name", name)?;
                map.end()
            }
            // 方法调用序列化为 { "type": "method", "class": "...", "method": "..." }
            CallableValue::Method { class_name, method_name } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("type", "method")?;
                map.serialize_entry("class", class_name)?;
                map.serialize_entry("method", method_name)?;
                map.end()
            }
            // 闭包序列化为 { "type": "closure", "id": "...", "params": [...], "captured": {...}, "file": "...", "by_ref": true/false }
            CallableValue::Closure { id, params, captured, file_path, by_ref } => {
                let mut map = serializer.serialize_map(Some(6))?;
                map.serialize_entry("type", "closure")?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("params", params)?;
                map.serialize_entry("captured", captured)?;
                map.serialize_entry("file", file_path)?;
                map.serialize_entry("by_ref", by_ref)?;
                map.end()
            }
            // 箭头函数序列化为 { "type": "arrow", "id": "...", "params": [...], "captured": {...}, "file": "..." }
            CallableValue::Arrow { id, params, captured, file_path } => {
                let mut map = serializer.serialize_map(Some(5))?;
                map.serialize_entry("type", "arrow")?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("params", params)?;
                map.serialize_entry("captured", captured)?;
                map.serialize_entry("file", file_path)?;
                map.end()
            }
        }
    }
}

/// 为 CallableValue 实现 Deserialize trait
impl<'de> Deserialize<'de> for CallableValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // 使用访问者模式反序列化
        use serde::de::{self, MapAccess, Visitor};
        
        /// CallableValue 反序列化访问者
        struct CallableValueVisitor;
        
        impl<'de> Visitor<'de> for CallableValueVisitor {
            type Value = CallableValue;
            
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a callable value")
            }
            
            fn visit_map<A>(self, mut map: A) -> Result<CallableValue, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut callable_type: Option<String> = None;
                let mut name: Option<String> = None;
                let mut class_name: Option<String> = None;
                let mut method_name: Option<String> = None;
                let mut id: Option<String> = None;
                let mut params: Option<Vec<String>> = None;
                let mut captured: Option<HashMap<String, Value>> = None;
                let mut file_path: Option<String> = None;
                let mut by_ref: Option<bool> = None;
                
                // 遍历映射中的每个键值对
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "type" => callable_type = Some(map.next_value()?),
                        "name" => name = Some(map.next_value()?),
                        "class" => class_name = Some(map.next_value()?),
                        "method" => method_name = Some(map.next_value()?),
                        "id" => id = Some(map.next_value()?),
                        "params" => params = Some(map.next_value()?),
                        "captured" => captured = Some(map.next_value()?),
                        "file" => file_path = Some(map.next_value()?),
                        "by_ref" => by_ref = Some(map.next_value()?),
                        _ => { let _ = map.next_value::<serde::de::IgnoredAny>()?; }
                    }
                }
                
                // 根据类型构造对应的 CallableValue
                match callable_type.as_deref() {
                    Some("function") => Ok(CallableValue::Function {
                        name: name.ok_or_else(|| de::Error::missing_field("name"))?,
                    }),
                    Some("method") => Ok(CallableValue::Method {
                        class_name: class_name.ok_or_else(|| de::Error::missing_field("class"))?,
                        method_name: method_name.ok_or_else(|| de::Error::missing_field("method"))?,
                    }),
                    Some("closure") => Ok(CallableValue::Closure {
                        id: id.ok_or_else(|| de::Error::missing_field("id"))?,
                        params: params.ok_or_else(|| de::Error::missing_field("params"))?,
                        captured: captured.ok_or_else(|| de::Error::missing_field("captured"))?,
                        file_path: file_path.ok_or_else(|| de::Error::missing_field("file"))?,
                        by_ref: by_ref.unwrap_or(false),
                    }),
                    Some("arrow") => Ok(CallableValue::Arrow {
                        id: id.ok_or_else(|| de::Error::missing_field("id"))?,
                        params: params.ok_or_else(|| de::Error::missing_field("params"))?,
                        captured: captured.ok_or_else(|| de::Error::missing_field("captured"))?,
                        file_path: file_path.ok_or_else(|| de::Error::missing_field("file"))?,
                    }),
                    _ => Err(de::Error::custom("unknown callable type")),
                }
            }
        }
        
        deserializer.deserialize_map(CallableValueVisitor)
    }
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
    /// - 其他为真（包括 Generator 和 Fiber）
    pub fn is_truthy(&self) -> bool {
        match self {
            // null 值为假
            Value::Null => false,
            // 布尔值直接返回
            Value::Bool(b) => *b,
            // 非零整数为真
            Value::Int(i) => *i != 0,
            // 非零浮点数为真
            Value::Float(f) => *f != 0.0,
            // 非空字符串且不为 "0" 为真
            Value::String(s) => !s.is_empty() && s != "0",
            // 非空索引数组为真
            Value::IndexedArray(arr) => !arr.is_empty(),
            // 非空关联数组为真
            Value::AssociativeArray(map) => !map.is_empty(),
            // 对象始终为真
            Value::Object(_) => true,
            // 可调用始终为真
            Value::Callable(_) => true,
            // 资源始终为真
            Value::Resource(_) => true,
            // Generator 始终为真（即使已关闭）
            Value::Generator(_) => true,
            // Fiber 始终为真
            Value::Fiber(_) => true,
        }
    }

    /// 转换为字符串
    /// 用于 echo、字符串拼接等场景
    pub fn to_string_value(&self) -> String {
        match self {
            // null 转换为空字符串
            Value::Null => "".to_string(),
            // true 转换为 "1"，false 转换为 ""
            Value::Bool(b) => if *b { "1" } else { "" }.to_string(),
            // 整数转换为十进制字符串
            Value::Int(i) => i.to_string(),
            // 浮点数转换为字符串
            Value::Float(f) => format!("{}", f),
            // 字符串直接返回
            Value::String(s) => s.clone(),
            // 数组转换为 "Array(n)" 格式
            Value::IndexedArray(arr) => format!("Array({})", arr.len()),
            // 关联数组转换为 "Array(n)" 格式
            Value::AssociativeArray(map) => format!("Array({})", map.len()),
            // 对象转换为 "ClassName Object" 格式
            Value::Object(obj) => format!("{} Object", obj.class_name),
            // 可调用转换为 "Callable"
            Value::Callable(_) => "Callable".to_string(),
            // 资源转换为 "Resource(type)" 格式
            Value::Resource(r) => format!("Resource({})", r),
            // Generator 转换为 "Generator" 字符串
            Value::Generator(_) => "Generator".to_string(),
            // Fiber 转换为 "Fiber" 字符串
            Value::Fiber(_) => "Fiber".to_string(),
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

    /// 类型名称（用于错误信息和类型检查）
    /// 返回 PHP 风格的类型名称字符串
    pub fn type_name(&self) -> &str {
        match self {
            // null 类型
            Value::Null => "null",
            // 布尔类型
            Value::Bool(_) => "bool",
            // 整数类型
            Value::Int(_) => "int",
            // 浮点数类型
            Value::Float(_) => "float",
            // 字符串类型
            Value::String(_) => "string",
            // 数组类型（索引数组和关联数组都是 array）
            Value::IndexedArray(_) | Value::AssociativeArray(_) => "array",
            // 对象类型
            Value::Object(_) => "object",
            // 可调用类型
            Value::Callable(_) => "callable",
            // 资源类型
            Value::Resource(_) => "resource",
            // Generator 类型
            Value::Generator(_) => "Generator",
            // Fiber 类型
            Value::Fiber(_) => "Fiber",
        }
    }

    /// 转换为 serde_json::Value
    /// 用于 JSON 序列化
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            // null 转换为 JSON null
            Value::Null => serde_json::Value::Null,
            // 布尔值转换为 JSON boolean
            Value::Bool(b) => serde_json::Value::Bool(*b),
            // 整数转换为 JSON number
            Value::Int(i) => serde_json::json!(*i),
            // 浮点数转换为 JSON number
            Value::Float(f) => serde_json::json!(*f),
            // 字符串转换为 JSON string
            Value::String(s) => serde_json::Value::String(s.clone()),
            // 索引数组转换为 JSON array
            Value::IndexedArray(arr) => {
                // 递归转换每个元素
                serde_json::Value::Array(arr.iter().map(|v| v.to_json_value()).collect())
            }
            // 关联数组转换为 JSON object
            Value::AssociativeArray(map) => {
                let mut obj = serde_json::Map::new();
                // 遍历每个键值对
                for (key, val) in map {
                    obj.insert(key.clone(), val.to_json_value());
                }
                serde_json::Value::Object(obj)
            }
            // 对象转换为 JSON object（包含 _class 字段）
            Value::Object(obj) => {
                let mut m = serde_json::Map::new();
                // 添加类名字段
                m.insert("_class".to_string(), serde_json::Value::String(obj.class_name.clone()));
                // 添加所有属性
                for (key, val) in &obj.properties {
                    m.insert(key.clone(), val.to_json_value());
                }
                serde_json::Value::Object(m)
            }
            // 可调用转换为字符串表示
            Value::Callable(_) => serde_json::Value::String("Callable".to_string()),
            // 资源转换为字符串表示
            Value::Resource(r) => serde_json::Value::String(format!("Resource({})", r)),
            // Generator 转换为字符串表示
            Value::Generator(_) => serde_json::Value::String("Generator".to_string()),
            // Fiber 转换为字符串表示
            Value::Fiber(_) => serde_json::Value::String("Fiber".to_string()),
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
