//! 门面类类型定义模块
//!
//! 定义门面类的核心类型：FacadeClassDefinition, FacadeClassRegistry, FacadeMethod

use std::collections::HashMap;

use crate::interpreter::value::{ObjectInstance, Value};

/// 门面类方法类型
///
/// 定义门面类方法的函数签名
pub type FacadeMethod = fn(&ObjectInstance, &[Value]) -> anyhow::Result<Value>;

/// 门面类定义
///
/// 包含类名和静态方法列表
#[derive(Debug, Clone)]
pub struct FacadeClassDefinition {
    /// 类名
    pub name: String,
    /// 类描述
    pub description: String,
    /// 静态方法
    pub static_methods: HashMap<String, FacadeMethod>,
    /// 常量
    pub constants: HashMap<String, Value>,
}

impl FacadeClassDefinition {
    /// 创建新的门面类定义
    ///
    /// # 参数
    /// - `name`: 类名
    /// - `description`: 类描述
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            static_methods: HashMap::new(),
            constants: HashMap::new(),
        }
    }

    /// 添加静态方法
    ///
    /// # 参数
    /// - `name`: 方法名
    /// - `method`: 方法实现
    pub fn add_static_method(mut self, name: &str, method: FacadeMethod) -> Self {
        self.static_methods.insert(name.to_string(), method);
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
}

/// 门面类注册表
///
/// 存储所有门面类的定义
pub struct FacadeClassRegistry {
    /// 类定义映射
    classes: HashMap<String, FacadeClassDefinition>,
}

impl FacadeClassRegistry {
    /// 创建新的门面类注册表
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
        }
    }

    /// 注册门面类
    ///
    /// # 参数
    /// - `definition`: 类定义
    pub fn register(&mut self, definition: FacadeClassDefinition) {
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
    pub fn get_class(&self, name: &str) -> Option<&FacadeClassDefinition> {
        self.classes.get(name)
    }

    /// 获取所有类名
    ///
    /// # 返回
    /// 类名列表
    pub fn get_class_names(&self) -> Vec<&String> {
        self.classes.keys().collect()
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

impl Default for FacadeClassRegistry {
    fn default() -> Self {
        Self::new()
    }
}
