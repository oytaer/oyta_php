//! 内置类注册模块
//!
//! 本模块实现 PHP 内置类的注册机制
//! 包括：DateTime, DateTimeImmutable, DateTimeZone, DateInterval, DatePeriod,
//! DOMDocument, SimpleXMLElement, GdImage, Phar, PharData 等

use std::collections::HashMap;

use crate::interpreter::value::{ObjectInstance, Value};

/// 内置类方法类型
///
/// 定义内置类方法的函数签名
pub type BuiltinMethod = fn(&ObjectInstance, &[Value]) -> anyhow::Result<Value>;

/// 内置类定义
///
/// 包含类名、方法列表和属性列表
#[derive(Debug, Clone)]
pub struct BuiltinClassDefinition {
    /// 类名
    pub name: String,
    /// 是否为最终类
    pub is_final: bool,
    /// 是否为抽象类
    pub is_abstract: bool,
    /// 是否为接口
    pub is_interface: bool,
    /// 是否为 trait
    pub is_trait: bool,
    /// 父类名
    pub parent_class: Option<String>,
    /// 实现的接口列表
    pub interfaces: Vec<String>,
    /// 使用的 trait 列表
    pub traits: Vec<String>,
    /// 类常量
    pub constants: HashMap<String, Value>,
    /// 静态属性
    pub static_properties: HashMap<String, Value>,
    /// 静态方法
    pub static_methods: HashMap<String, BuiltinMethod>,
    /// 实例方法
    pub methods: HashMap<String, BuiltinMethod>,
    /// 默认属性值
    pub default_properties: HashMap<String, Value>,
}

impl BuiltinClassDefinition {
    /// 创建新的内置类定义
    ///
    /// # 参数
    /// - `name`: 类名
    ///
    /// # 返回
    /// 新的内置类定义实例
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_final: false,
            is_abstract: false,
            is_interface: false,
            is_trait: false,
            parent_class: None,
            interfaces: Vec::new(),
            traits: Vec::new(),
            constants: HashMap::new(),
            static_properties: HashMap::new(),
            static_methods: HashMap::new(),
            methods: HashMap::new(),
            default_properties: HashMap::new(),
        }
    }
    
    /// 设置为最终类
    pub fn set_final(mut self) -> Self {
        self.is_final = true;
        self
    }
    
    /// 设置为抽象类
    pub fn set_abstract(mut self) -> Self {
        self.is_abstract = true;
        self
    }
    
    /// 设置父类
    ///
    /// # 参数
    /// - `parent`: 父类名
    pub fn set_parent(mut self, parent: &str) -> Self {
        self.parent_class = Some(parent.to_string());
        self
    }
    
    /// 添加接口
    ///
    /// # 参数
    /// - `interface`: 接口名
    pub fn add_interface(mut self, interface: &str) -> Self {
        self.interfaces.push(interface.to_string());
        self
    }
    
    /// 添加常量
    ///
    /// # 参数
    /// - `name`: 常量名
    /// - `value`: 常量值
    pub fn add_constant(mut self, name: &str, value: Value) -> Self {
        self.constants.insert(name.to_string(), value);
        self
    }
    
    /// 添加静态属性
    ///
    /// # 参数
    /// - `name`: 属性名
    /// - `value`: 属性值
    pub fn add_static_property(mut self, name: &str, value: Value) -> Self {
        self.static_properties.insert(name.to_string(), value);
        self
    }
    
    /// 添加静态方法
    ///
    /// # 参数
    /// - `name`: 方法名
    /// - `method`: 方法实现
    pub fn add_static_method(mut self, name: &str, method: BuiltinMethod) -> Self {
        self.static_methods.insert(name.to_string(), method);
        self
    }
    
    /// 添加实例方法
    ///
    /// # 参数
    /// - `name`: 方法名
    /// - `method`: 方法实现
    pub fn add_method(mut self, name: &str, method: BuiltinMethod) -> Self {
        self.methods.insert(name.to_string(), method);
        self
    }
    
    /// 添加默认属性
    ///
    /// # 参数
    /// - `name`: 属性名
    /// - `value`: 默认值
    pub fn add_default_property(mut self, name: &str, value: Value) -> Self {
        self.default_properties.insert(name.to_string(), value);
        self
    }
}

/// 内置类注册表
///
/// 存储所有内置类的定义
pub struct BuiltinClassRegistry {
    /// 类定义映射
    classes: HashMap<String, BuiltinClassDefinition>,
}

impl BuiltinClassRegistry {
    /// 创建新的内置类注册表
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
        }
    }
    
    /// 注册内置类
    ///
    /// # 参数
    /// - `definition`: 类定义
    pub fn register(&mut self, definition: BuiltinClassDefinition) {
        self.classes.insert(definition.name.clone(), definition);
    }
    
    /// 检查类是否存在
    ///
    /// # 参数
    /// - `name`: 类名
    ///
    /// # 返回
    /// 类是否存在
    pub fn has_class(&self, name: &str) -> bool {
        self.classes.contains_key(name)
    }
    
    /// 获取类定义
    ///
    /// # 参数
    /// - `name`: 类名
    ///
    /// # 返回
    /// 类定义引用
    pub fn get_class(&self, name: &str) -> Option<&BuiltinClassDefinition> {
        self.classes.get(name)
    }
    
    /// 获取所有类名
    ///
    /// # 返回
    /// 类名列表
    pub fn get_class_names(&self) -> Vec<&String> {
        self.classes.keys().collect()
    }
    
    /// 创建类实例
    ///
    /// # 参数
    /// - `name`: 类名
    ///
    /// # 返回
    /// 新的对象实例
    pub fn create_instance(&self, name: &str) -> Option<ObjectInstance> {
        let definition = self.classes.get(name)?;
        
        // 创建属性映射
        let mut properties = HashMap::new();
        for (prop_name, prop_value) in &definition.default_properties {
            properties.insert(prop_name.clone(), prop_value.clone());
        }
        
        Some(ObjectInstance {
            class_name: definition.name.clone(),
            properties,
        })
    }
    
    /// 调用实例方法
    ///
    /// # 参数
    /// - `instance`: 对象实例
    /// - `method_name`: 方法名
    /// - `args`: 参数列表
    ///
    /// # 返回
    /// 方法返回值
    pub fn call_method(
        &self,
        instance: &ObjectInstance,
        method_name: &str,
        args: &[Value],
    ) -> Option<anyhow::Result<Value>> {
        let definition = self.classes.get(&instance.class_name)?;
        let method = definition.methods.get(method_name)?;
        Some(method(instance, args))
    }
    
    /// 调用静态方法
    ///
    /// # 参数
    /// - `class_name`: 类名
    /// - `method_name`: 方法名
    /// - `args`: 参数列表
    ///
    /// # 返回
    /// 方法返回值
    pub fn call_static_method(
        &self,
        class_name: &str,
        method_name: &str,
        args: &[Value],
    ) -> Option<anyhow::Result<Value>> {
        let definition = self.classes.get(class_name)?;
        let method = definition.static_methods.get(method_name)?;
        
        // 创建临时实例用于调用
        let temp_instance = ObjectInstance {
            class_name: class_name.to_string(),
            properties: HashMap::new(),
        };
        
        Some(method(&temp_instance, args))
    }
}

impl Default for BuiltinClassRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 内置类方法实现
// ============================================================================

/// DateTime::format 方法实现
fn datetime_format(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    // 获取格式字符串
    let format = args.first()
        .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
        .unwrap_or("Y-m-d H:i:s");
    
    // 从实例属性获取时间戳
    let timestamp = instance.properties.get("timestamp")
        .and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None })
        .unwrap_or(0);
    
    // 创建日期时间
    let dt = chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    
    // 格式化
    let result = format
        .replace('Y', &dt.format("%Y").to_string())
        .replace('m', &dt.format("%m").to_string())
        .replace('d', &dt.format("%d").to_string())
        .replace('H', &dt.format("%H").to_string())
        .replace('i', &dt.format("%M").to_string())
        .replace('s', &dt.format("%S").to_string());
    
    Ok(Value::String(result))
}

/// DateTime::modify 方法实现
fn datetime_modify(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    // 获取修改字符串
    let _modifier = args.first()
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();
    
    // 简化实现：返回当前实例的克隆
    Ok(Value::Object(instance.clone()))
}

/// DateTime::getTimestamp 方法实现
fn datetime_get_timestamp(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let timestamp = instance.properties.get("timestamp")
        .and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None })
        .unwrap_or(0);
    
    Ok(Value::Int(timestamp))
}

/// DateTime::setTimestamp 方法实现
fn datetime_set_timestamp(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    let timestamp = args.first()
        .and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None })
        .unwrap_or(0);
    
    // 创建新实例
    let mut new_instance = instance.clone();
    new_instance.properties.insert("timestamp".to_string(), Value::Int(timestamp));
    
    Ok(Value::Object(new_instance))
}

/// DateTimeImmutable::format 方法实现（与 DateTime 相同）
fn datetime_immutable_format(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    datetime_format(instance, args)
}

/// DateTimeZone::getName 方法实现
fn datetimezone_get_name(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let name = instance.properties.get("name")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_else(|| "UTC".to_string());
    
    Ok(Value::String(name))
}

/// DOMDocument::saveXML 方法实现
fn domdocument_save_xml(_instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    // 简化实现：返回空 XML
    Ok(Value::String("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n".to_string()))
}

/// DOMDocument::loadXML 方法实现
fn domdocument_load_xml(instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    // 获取 XML 字符串
    let _xml = args.first()
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();
    
    // 简化实现：返回当前实例
    Ok(Value::Object(instance.clone()))
}

/// SimpleXMLElement::asXML 方法实现
fn simplexmlelement_as_xml(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let xml = instance.properties.get("xml")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();
    
    Ok(Value::String(xml))
}

/// SimpleXMLElement::xpath 方法实现
fn simplexmlelement_xpath(_instance: &ObjectInstance, args: &[Value]) -> anyhow::Result<Value> {
    // 获取 XPath 表达式
    let _xpath = args.first()
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_default();
    
    // 简化实现：返回空数组
    Ok(Value::IndexedArray(vec![]))
}

/// GdImage::getWidth 方法实现
fn gdimage_get_width(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let width = instance.properties.get("width")
        .and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None })
        .unwrap_or(0);
    
    Ok(Value::Int(width))
}

/// GdImage::getHeight 方法实现
fn gdimage_get_height(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let height = instance.properties.get("height")
        .and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None })
        .unwrap_or(0);
    
    Ok(Value::Int(height))
}

/// Phar::getSignature 方法实现
fn phar_get_signature(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let has_signature = instance.properties.get("hasSignature")
        .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
        .unwrap_or(false);
    
    if has_signature {
        Ok(Value::IndexedArray(vec![
            Value::String("hash".to_string()),
            Value::String(String::new()),
        ]))
    } else {
        Ok(Value::Bool(false))
    }
}

/// Phar::getVersion 方法实现
fn phar_get_version(instance: &ObjectInstance, _args: &[Value]) -> anyhow::Result<Value> {
    let version = instance.properties.get("version")
        .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
        .unwrap_or_else(|| "1.0.0".to_string());
    
    Ok(Value::String(version))
}

// ============================================================================
// 注册所有内置类
// ============================================================================

/// 创建并填充内置类注册表
///
/// # 返回
/// 包含所有内置类的注册表
pub fn create_builtin_class_registry() -> BuiltinClassRegistry {
    let mut registry = BuiltinClassRegistry::new();
    
    // 注册 DateTime 类
    let datetime_class = BuiltinClassDefinition::new("DateTime")
        .add_default_property("timestamp", Value::Int(0))
        .add_default_property("ustime", Value::Int(0))
        .add_default_property("timezone", Value::String("UTC".to_string()))
        .add_default_property("year", Value::Int(0))
        .add_default_property("month", Value::Int(0))
        .add_default_property("day", Value::Int(0))
        .add_default_property("hour", Value::Int(0))
        .add_default_property("minute", Value::Int(0))
        .add_default_property("second", Value::Int(0))
        .add_method("format", datetime_format)
        .add_method("modify", datetime_modify)
        .add_method("getTimestamp", datetime_get_timestamp)
        .add_method("setTimestamp", datetime_set_timestamp)
        .add_constant("ATOM", Value::String("Y-m-d\\TH:i:sP".to_string()))
        .add_constant("COOKIE", Value::String("l, d-M-Y H:i:s T".to_string()))
        .add_constant("ISO8601", Value::String("Y-m-d\\TH:i:sO".to_string()))
        .add_constant("RFC822", Value::String("D, d M y H:i:s O".to_string()))
        .add_constant("RFC2822", Value::String("D, d M Y H:i:s O".to_string()))
        .add_constant("RFC3339", Value::String("Y-m-d\\TH:i:sP".to_string()))
        .add_constant("RSS", Value::String("D, d M Y H:i:s O".to_string()))
        .add_constant("W3C", Value::String("Y-m-d\\TH:i:sP".to_string()));
    registry.register(datetime_class);
    
    // 注册 DateTimeImmutable 类
    let datetime_immutable_class = BuiltinClassDefinition::new("DateTimeImmutable")
        .set_parent("DateTime")
        .add_default_property("timestamp", Value::Int(0))
        .add_default_property("ustime", Value::Int(0))
        .add_default_property("timezone", Value::String("UTC".to_string()))
        .add_method("format", datetime_immutable_format);
    registry.register(datetime_immutable_class);
    
    // 注册 DateTimeZone 类
    let datetimezone_class = BuiltinClassDefinition::new("DateTimeZone")
        .add_default_property("name", Value::String("UTC".to_string()))
        .add_method("getName", datetimezone_get_name)
        .add_constant("AFRICA", Value::Int(1))
        .add_constant("AMERICA", Value::Int(2))
        .add_constant("ANTARCTICA", Value::Int(4))
        .add_constant("ARCTIC", Value::Int(8))
        .add_constant("ASIA", Value::Int(16))
        .add_constant("ATLANTIC", Value::Int(32))
        .add_constant("AUSTRALIA", Value::Int(64))
        .add_constant("EUROPE", Value::Int(128))
        .add_constant("INDIAN", Value::Int(256))
        .add_constant("PACIFIC", Value::Int(512))
        .add_constant("UTC", Value::Int(1024))
        .add_constant("ALL", Value::Int(2047))
        .add_constant("ALL_WITH_BC", Value::Int(4095));
    registry.register(datetimezone_class);
    
    // 注册 DateInterval 类
    let dateinterval_class = BuiltinClassDefinition::new("DateInterval")
        .add_default_property("y", Value::Int(0))
        .add_default_property("m", Value::Int(0))
        .add_default_property("d", Value::Int(0))
        .add_default_property("h", Value::Int(0))
        .add_default_property("i", Value::Int(0))
        .add_default_property("s", Value::Int(0))
        .add_default_property("invert", Value::Bool(false))
        .add_default_property("days", Value::Bool(false));
    registry.register(dateinterval_class);
    
    // 注册 DatePeriod 类
    let dateperiod_class = BuiltinClassDefinition::new("DatePeriod")
        .add_default_property("start", Value::Null)
        .add_default_property("end", Value::Null)
        .add_default_property("interval", Value::Null)
        .add_default_property("include_start_date", Value::Bool(true))
        .add_default_property("recurrences", Value::Int(0));
    registry.register(dateperiod_class);
    
    // 注册 DOMDocument 类
    let domdocument_class = BuiltinClassDefinition::new("DOMDocument")
        .add_default_property("version", Value::String("1.0".to_string()))
        .add_default_property("encoding", Value::String("UTF-8".to_string()))
        .add_default_property("documentElement", Value::Null)
        .add_default_property("formatOutput", Value::Bool(false))
        .add_default_property("preserveWhiteSpace", Value::Bool(true))
        .add_default_property("validateOnParse", Value::Bool(false))
        .add_method("saveXML", domdocument_save_xml)
        .add_method("loadXML", domdocument_load_xml);
    registry.register(domdocument_class);
    
    // 注册 SimpleXMLElement 类
    let simplexmlelement_class = BuiltinClassDefinition::new("SimpleXMLElement")
        .add_default_property("xml", Value::String(String::new()))
        .add_default_property("nodeName", Value::String(String::new()))
        .add_default_property("nodeValue", Value::String(String::new()))
        .add_default_property("nodeType", Value::Int(1))
        .add_default_property("attributes", Value::IndexedArray(vec![]))
        .add_default_property("children", Value::IndexedArray(vec![]))
        .add_method("asXML", simplexmlelement_as_xml)
        .add_method("xpath", simplexmlelement_xpath);
    registry.register(simplexmlelement_class);
    
    // 注册 GdImage 类
    let gdimage_class = BuiltinClassDefinition::new("GdImage")
        .add_default_property("width", Value::Int(0))
        .add_default_property("height", Value::Int(0))
        .add_default_property("trueColor", Value::Bool(true))
        .add_default_property("pixels", Value::String(String::new()))
        .add_default_property("colors", Value::IndexedArray(vec![]))
        .add_default_property("transparent", Value::Int(-1))
        .add_method("getWidth", gdimage_get_width)
        .add_method("getHeight", gdimage_get_height);
    registry.register(gdimage_class);
    
    // 注册 Phar 类
    let phar_class = BuiltinClassDefinition::new("Phar")
        .add_default_property("path", Value::String(String::new()))
        .add_default_property("alias", Value::String(String::new()))
        .add_default_property("version", Value::String("1.0.0".to_string()))
        .add_default_property("signature", Value::Int(0))
        .add_default_property("hasSignature", Value::Bool(false))
        .add_default_property("isCompressed", Value::Bool(false))
        .add_default_property("isWritable", Value::Bool(false))
        .add_default_property("entries", Value::IndexedArray(vec![]))
        .add_method("getSignature", phar_get_signature)
        .add_method("getVersion", phar_get_version)
        .add_constant("MD5", Value::Int(1))
        .add_constant("SHA1", Value::Int(2))
        .add_constant("SHA256", Value::Int(3))
        .add_constant("SHA512", Value::Int(4))
        .add_constant("OPENSSL", Value::Int(16));
    registry.register(phar_class);
    
    // 注册 PharData 类
    let phardata_class = BuiltinClassDefinition::new("PharData")
        .add_default_property("path", Value::String(String::new()))
        .add_default_property("alias", Value::String(String::new()))
        .add_default_property("entries", Value::IndexedArray(vec![]));
    registry.register(phardata_class);
    
    // 注册 PharException 类
    let pharexception_class = BuiltinClassDefinition::new("PharException")
        .set_parent("Exception")
        .add_default_property("message", Value::String(String::new()))
        .add_default_property("code", Value::Int(0))
        .add_default_property("file", Value::String(String::new()))
        .add_default_property("line", Value::Int(0));
    registry.register(pharexception_class);
    
    // 注册 XMLParser 类
    let xmlparser_class = BuiltinClassDefinition::new("XMLParser")
        .add_default_property("encoding", Value::String("UTF-8".to_string()))
        .add_default_property("buffer", Value::String(String::new()))
        .add_default_property("status", Value::Bool(true));
    registry.register(xmlparser_class);
    
    // 注册 ReflectionClass 类
    let reflectionclass_class = BuiltinClassDefinition::new("ReflectionClass")
        .add_default_property("name", Value::String(String::new()))
        .add_default_property("isInternal", Value::Bool(false))
        .add_default_property("isUserDefined", Value::Bool(true))
        .add_default_property("isInstantiable", Value::Bool(true))
        .add_default_property("isCloneable", Value::Bool(true))
        .add_default_property("isFinal", Value::Bool(false))
        .add_default_property("isAbstract", Value::Bool(false))
        .add_default_property("isInterface", Value::Bool(false))
        .add_default_property("isTrait", Value::Bool(false));
    registry.register(reflectionclass_class);
    
    // 注册 ReflectionMethod 类
    let reflectionmethod_class = BuiltinClassDefinition::new("ReflectionMethod")
        .add_default_property("name", Value::String(String::new()))
        .add_default_property("class", Value::String(String::new()))
        .add_default_property("isPublic", Value::Bool(true))
        .add_default_property("isPrivate", Value::Bool(false))
        .add_default_property("isProtected", Value::Bool(false))
        .add_default_property("isStatic", Value::Bool(false))
        .add_default_property("isFinal", Value::Bool(false))
        .add_default_property("isAbstract", Value::Bool(false));
    registry.register(reflectionmethod_class);
    
    // 注册 ReflectionProperty 类
    let reflectionproperty_class = BuiltinClassDefinition::new("ReflectionProperty")
        .add_default_property("name", Value::String(String::new()))
        .add_default_property("class", Value::String(String::new()))
        .add_default_property("isPublic", Value::Bool(true))
        .add_default_property("isPrivate", Value::Bool(false))
        .add_default_property("isProtected", Value::Bool(false))
        .add_default_property("isStatic", Value::Bool(false))
        .add_default_property("isDefault", Value::Bool(true));
    registry.register(reflectionproperty_class);
    
    // 注册 ReflectionParameter 类
    let reflectionparameter_class = BuiltinClassDefinition::new("ReflectionParameter")
        .add_default_property("name", Value::String(String::new()))
        .add_default_property("position", Value::Int(0))
        .add_default_property("isOptional", Value::Bool(false))
        .add_default_property("isDefaultValueAvailable", Value::Bool(false))
        .add_default_property("defaultValue", Value::Null)
        .add_default_property("type", Value::String(String::new()))
        .add_default_property("allowsNull", Value::Bool(true));
    registry.register(reflectionparameter_class);
    
    // 注册 ReflectionFunction 类
    let reflectionfunction_class = BuiltinClassDefinition::new("ReflectionFunction")
        .add_default_property("name", Value::String(String::new()))
        .add_default_property("isInternal", Value::Bool(false))
        .add_default_property("isUserDefined", Value::Bool(true))
        .add_default_property("isClosure", Value::Bool(false))
        .add_default_property("isDeprecated", Value::Bool(false))
        .add_default_property("isGenerator", Value::Bool(false))
        .add_default_property("isVariadic", Value::Bool(false));
    registry.register(reflectionfunction_class);
    
    registry
}

/// 全局内置类注册表
///
/// 使用懒加载初始化
pub fn get_builtin_class_registry() -> &'static BuiltinClassRegistry {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<BuiltinClassRegistry> = OnceLock::new();
    REGISTRY.get_or_init(create_builtin_class_registry)
}
