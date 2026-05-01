//! 内置类类型定义模块
//!
//! 定义内置类的核心类型和方法签名

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
        // 查找类定义
        let definition = self.classes.get(class_name)?;
        
        // 首先在当前类中查找静态方法
        if let Some(method) = definition.static_methods.get(method_name) {
            // 创建临时实例用于调用，包含调用类名
            let temp_instance = ObjectInstance {
                class_name: class_name.to_string(),
                properties: HashMap::new(),
            };
            return Some(method(&temp_instance, args));
        }
        
        // 如果当前类没有该方法，查找父类（支持继承）
        if let Some(parent_class) = &definition.parent_class {
            return self.call_static_method(parent_class, method_name, args);
        }
        
        None
    }
    
    /// 调用实例方法（支持继承）
    ///
    /// # 参数
    /// - `instance`: 对象实例
    /// - `method_name`: 方法名
    /// - `args`: 参数列表
    ///
    /// # 返回
    /// 方法返回值
    pub fn call_method_with_inheritance(
        &self,
        instance: &ObjectInstance,
        method_name: &str,
        args: &[Value],
    ) -> Option<anyhow::Result<Value>> {
        // 查找类定义
        let definition = self.classes.get(&instance.class_name)?;
        
        // 首先在当前类中查找方法
        if let Some(method) = definition.methods.get(method_name) {
            return Some(method(instance, args));
        }
        
        // 如果当前类没有该方法，查找父类（支持继承）
        if let Some(parent_class) = &definition.parent_class {
            // 创建一个带有父类名的临时实例
            let temp_instance = ObjectInstance {
                class_name: parent_class.clone(),
                properties: instance.properties.clone(),
            };
            return self.call_method_with_inheritance(&temp_instance, method_name, args);
        }
        
        None
    }
}

impl Default for BuiltinClassRegistry {
    fn default() -> Self {
        Self::new()
    }
}
