//! 只读属性模块
//!
//! 实现 PHP 8.1+ 的只读属性功能支持
//! 只读属性只能在声明时或构造函数中初始化一次，之后不能再修改
//!
//! # 功能特性
//! - 只读属性声明
//! - 构造函数中的初始化
//! - 防止后续修改
//! - 只读属性继承
//!
//! # 使用示例
//! ```php
//! class User {
//!     public readonly int $id;
//!     public readonly string $name;
//!     
//!     public function __construct(int $id, string $name) {
//!         $this->id = $id;
//!         $this->name = $name;
//!     }
//! }
//! 
//! $user = new User(1, 'think');
//! $user->id = 2;  // Error: Cannot modify readonly property
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 属性定义
/// 表示一个类属性的定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDefinition {
    /// 属性名称
    pub name: String,
    
    /// 属性类型（可选）
    /// 例如：int, string, ?string, User
    pub type_hint: Option<String>,
    
    /// 是否为只读属性
    pub is_readonly: bool,
    
    /// 可见性
    pub visibility: PropertyVisibility,
    
    /// 是否为静态属性
    pub is_static: bool,
    
    /// 默认值（JSON 序列化存储）
    pub default_value_json: Option<String>,
    
    /// 是否已初始化
    /// 用于跟踪只读属性的初始化状态
    pub is_initialized: bool,
    
    /// 属性所在行号
    pub line: usize,
    
    /// 文档注释
    pub doc_comment: Option<String>,
}

/// 属性可见性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertyVisibility {
    /// 公开
    Public,
    /// 受保护
    Protected,
    /// 私有
    Private,
}

/// 属性访问结果
/// 表示属性访问操作的结果
#[derive(Debug, Clone)]
pub enum PropertyAccessResult {
    /// 访问成功
    Success,
    /// 属性不存在
    NotFound(String),
    /// 只读属性不能修改
    ReadonlyViolation(String),
    /// 访问权限不足
    AccessDenied(String),
    /// 类型不匹配
    TypeMismatch(String),
    /// 静态属性访问方式错误
    StaticAccessError(String),
}

/// 只读属性管理器
/// 用于管理类中的只读属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadonlyPropertyManager {
    /// 类名
    pub class_name: String,
    
    /// 属性定义列表
    /// 键为属性名，值为属性定义
    pub properties: HashMap<String, PropertyDefinition>,
    
    /// 已初始化的只读属性集合
    /// 用于跟踪哪些只读属性已经被初始化
    pub initialized_readonly: HashMap<String, bool>,
}

impl PropertyDefinition {
    /// 创建新的属性定义
    ///
    /// # 参数
    /// - `name`: 属性名称
    /// - `type_hint`: 类型提示（可选）
    /// - `is_readonly`: 是否为只读属性
    /// - `visibility`: 可见性
    /// - `is_static`: 是否为静态属性
    /// - `line`: 所在行号
    ///
    /// # 返回
    /// 返回新创建的属性定义
    pub fn new(
        name: String,
        type_hint: Option<String>,
        is_readonly: bool,
        visibility: PropertyVisibility,
        is_static: bool,
        line: usize,
    ) -> Self {
        Self {
            // 设置属性名称
            name,
            // 设置类型提示
            type_hint,
            // 设置是否为只读属性
            is_readonly,
            // 设置可见性
            visibility,
            // 设置是否为静态属性
            is_static,
            // 默认值初始为空
            default_value_json: None,
            // 初始化状态默认为 false
            is_initialized: false,
            // 设置行号
            line,
            // 文档注释初始为空
            doc_comment: None,
        }
    }
    
    /// 创建公开只读属性
    ///
    /// # 参数
    /// - `name`: 属性名称
    /// - `type_hint`: 类型提示
    /// - `line`: 所在行号
    ///
    /// # 返回
    /// 返回公开只读属性定义
    pub fn public_readonly(name: String, type_hint: String, line: usize) -> Self {
        Self::new(
            name,
            Some(type_hint),
            true,
            PropertyVisibility::Public,
            false,
            line,
        )
    }
    
    /// 创建公开属性
    ///
    /// # 参数
    /// - `name`: 属性名称
    /// - `type_hint`: 类型提示（可选）
    /// - `line`: 所在行号
    ///
    /// # 返回
    /// 返回公开属性定义
    pub fn public(name: String, type_hint: Option<String>, line: usize) -> Self {
        Self::new(
            name,
            type_hint,
            false,
            PropertyVisibility::Public,
            false,
            line,
        )
    }
    
    /// 创建私有只读属性
    ///
    /// # 参数
    /// - `name`: 属性名称
    /// - `type_hint`: 类型提示
    /// - `line`: 所在行号
    ///
    /// # 返回
    /// 返回私有只读属性定义
    pub fn private_readonly(name: String, type_hint: String, line: usize) -> Self {
        Self::new(
            name,
            Some(type_hint),
            true,
            PropertyVisibility::Private,
            false,
            line,
        )
    }
    
    /// 创建受保护只读属性
    ///
    /// # 参数
    /// - `name`: 属性名称
    /// - `type_hint`: 类型提示
    /// - `line`: 所在行号
    ///
    /// # 返回
    /// 返回受保护只读属性定义
    pub fn protected_readonly(name: String, type_hint: String, line: usize) -> Self {
        Self::new(
            name,
            Some(type_hint),
            true,
            PropertyVisibility::Protected,
            false,
            line,
        )
    }
    
    /// 设置默认值
    ///
    /// # 参数
    /// - `value_json`: 默认值的 JSON 字符串
    pub fn set_default_value(&mut self, value_json: String) {
        self.default_value_json = Some(value_json);
        // 有默认值则标记为已初始化
        self.is_initialized = true;
    }
    
    /// 检查是否可以修改
    ///
    /// # 返回
    /// 如果属性可以被修改返回 true，否则返回 false
    pub fn can_modify(&self) -> bool {
        // 只读属性已初始化后不能修改
        if self.is_readonly && self.is_initialized {
            return false;
        }
        true
    }
    
    /// 标记为已初始化
    pub fn mark_initialized(&mut self) {
        self.is_initialized = true;
    }
    
    /// 获取可见性字符串
    ///
    /// # 返回
    /// 返回可见性的字符串表示
    pub fn visibility_string(&self) -> &str {
        match self.visibility {
            PropertyVisibility::Public => "public",
            PropertyVisibility::Protected => "protected",
            PropertyVisibility::Private => "private",
        }
    }
}

impl ReadonlyPropertyManager {
    /// 创建新的只读属性管理器
    ///
    /// # 参数
    /// - `class_name`: 类名
    ///
    /// # 返回
    /// 返回新创建的管理器实例
    pub fn new(class_name: String) -> Self {
        Self {
            // 设置类名
            class_name,
            // 初始化属性列表为空
            properties: HashMap::new(),
            // 初始化已初始化列表为空
            initialized_readonly: HashMap::new(),
        }
    }
    
    /// 添加属性定义
    ///
    /// # 参数
    /// - `property`: 属性定义
    ///
    /// # 返回
    /// 成功返回 Ok(())，如果属性已存在返回错误
    pub fn add_property(&mut self, property: PropertyDefinition) -> Result<(), String> {
        // 检查属性是否已存在
        if self.properties.contains_key(&property.name) {
            return Err(format!(
                "Property '{}' already exists in class '{}'",
                property.name, self.class_name
            ));
        }
        
        // 如果是只读属性且有默认值，标记为已初始化
        if property.is_readonly && property.default_value_json.is_some() {
            self.initialized_readonly.insert(property.name.clone(), true);
        }
        
        // 添加属性
        let name = property.name.clone();
        self.properties.insert(name, property);
        
        Ok(())
    }
    
    /// 获取属性定义
    ///
    /// # 参数
    /// - `name`: 属性名称
    ///
    /// # 返回
    /// 返回属性定义，如果不存在返回 None
    pub fn get_property(&self, name: &str) -> Option<&PropertyDefinition> {
        self.properties.get(name)
    }
    
    /// 获取属性定义（可变引用）
    ///
    /// # 参数
    /// - `name`: 属性名称
    ///
    /// # 返回
    /// 返回属性定义的可变引用，如果不存在返回 None
    pub fn get_property_mut(&mut self, name: &str) -> Option<&mut PropertyDefinition> {
        self.properties.get_mut(name)
    }
    
    /// 检查属性是否为只读
    ///
    /// # 参数
    /// - `name`: 属性名称
    ///
    /// # 返回
    /// 如果属性是只读属性返回 true，否则返回 false
    pub fn is_readonly(&self, name: &str) -> bool {
        // 查找属性定义
        match self.properties.get(name) {
            Some(prop) => prop.is_readonly,
            None => false,
        }
    }
    
    /// 检查只读属性是否已初始化
    ///
    /// # 参数
    /// - `name`: 属性名称
    ///
    /// # 返回
    /// 如果只读属性已初始化返回 true，否则返回 false
    pub fn is_initialized(&self, name: &str) -> bool {
        // 从已初始化列表中查找
        self.initialized_readonly.get(name).copied().unwrap_or(false)
    }
    
    /// 尝试设置属性值
    /// 检查只读属性的修改权限
    ///
    /// # 参数
    /// - `name`: 属性名称
    /// - `in_constructor`: 是否在构造函数中
    ///
    /// # 返回
    /// 返回属性访问结果
    pub fn try_set_property(&mut self, name: &str, in_constructor: bool) -> PropertyAccessResult {
        // 查找属性定义
        let property = match self.properties.get(name) {
            Some(prop) => prop,
            None => return PropertyAccessResult::NotFound(format!(
                "Property '{}' does not exist in class '{}'",
                name, self.class_name
            )),
        };
        
        // 检查是否为只读属性
        if property.is_readonly {
            // 检查是否已初始化
            if self.is_initialized(name) {
                // 只读属性已初始化后不能修改
                return PropertyAccessResult::ReadonlyViolation(format!(
                    "Cannot modify readonly property {}::${}",
                    self.class_name, name
                ));
            }
            
            // 在构造函数中可以初始化只读属性
            if in_constructor {
                // 标记为已初始化
                self.initialized_readonly.insert(name.to_string(), true);
                return PropertyAccessResult::Success;
            }
            
            // 不在构造函数中，不能初始化只读属性
            return PropertyAccessResult::ReadonlyViolation(format!(
                "Cannot initialize readonly property {}::${} outside of constructor",
                self.class_name, name
            ));
        }
        
        // 非只读属性可以自由修改
        PropertyAccessResult::Success
    }
    
    /// 标记只读属性为已初始化
    ///
    /// # 参数
    /// - `name`: 属性名称
    ///
    /// # 返回
    /// 成功返回 Ok(())，如果属性不存在返回错误
    pub fn mark_readonly_initialized(&mut self, name: &str) -> Result<(), String> {
        // 检查属性是否存在
        if !self.properties.contains_key(name) {
            return Err(format!("Property '{}' does not exist", name));
        }
        
        // 标记为已初始化
        self.initialized_readonly.insert(name.to_string(), true);
        
        Ok(())
    }
    
    /// 获取所有只读属性名称
    ///
    /// # 返回
    /// 返回所有只读属性的名称列表
    pub fn get_readonly_properties(&self) -> Vec<String> {
        self.properties
            .iter()
            .filter(|(_, prop)| prop.is_readonly)
            .map(|(name, _)| name.clone())
            .collect()
    }
    
    /// 获取所有未初始化的只读属性名称
    ///
    /// # 返回
    /// 返回所有未初始化的只读属性名称列表
    pub fn get_uninitialized_readonly(&self) -> Vec<String> {
        self.properties
            .iter()
            .filter(|(_, prop)| prop.is_readonly && !prop.is_initialized)
            .map(|(name, _)| name.clone())
            .collect()
    }
    
    /// 验证所有只读属性是否已初始化
    /// 在构造函数结束时调用
    ///
    /// # 返回
    /// 如果所有只读属性都已初始化返回 Ok(())，否则返回错误信息
    pub fn validate_all_readonly_initialized(&self) -> Result<(), String> {
        // 获取未初始化的只读属性
        let uninitialized: Vec<String> = self.properties
            .iter()
            .filter(|(_, prop)| prop.is_readonly && !prop.default_value_json.is_some() && !self.is_initialized(&prop.name))
            .map(|(name, _)| name.clone())
            .collect();
        
        // 如果有未初始化的属性，返回错误
        if !uninitialized.is_empty() {
            return Err(format!(
                "Readonly properties must be initialized: {}",
                uninitialized.join(", ")
            ));
        }
        
        Ok(())
    }
    
    /// 获取类名
    pub fn get_class_name(&self) -> &str {
        &self.class_name
    }
    
    /// 获取属性数量
    pub fn property_count(&self) -> usize {
        self.properties.len()
    }
    
    /// 获取只读属性数量
    pub fn readonly_count(&self) -> usize {
        self.properties.values().filter(|p| p.is_readonly).count()
    }
}

/// 构造函数属性提升
/// PHP 8.0+ 允许在构造函数参数中直接声明属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructorPropertyPromotion {
    /// 参数名称
    pub param_name: String,
    
    /// 参数类型
    pub param_type: Option<String>,
    
    /// 属性可见性
    pub visibility: PropertyVisibility,
    
    /// 是否为只读属性
    pub is_readonly: bool,
    
    /// 默认值（JSON 序列化存储）
    pub default_value_json: Option<String>,
}

impl ConstructorPropertyPromotion {
    /// 创建新的构造函数属性提升
    ///
    /// # 参数
    /// - `param_name`: 参数名称
    /// - `param_type`: 参数类型（可选）
    /// - `visibility`: 属性可见性
    /// - `is_readonly`: 是否为只读属性
    ///
    /// # 返回
    /// 返回新创建的构造函数属性提升
    pub fn new(
        param_name: String,
        param_type: Option<String>,
        visibility: PropertyVisibility,
        is_readonly: bool,
    ) -> Self {
        Self {
            param_name,
            param_type,
            visibility,
            is_readonly,
            default_value_json: None,
        }
    }
    
    /// 转换为属性定义
    ///
    /// # 参数
    /// - `line`: 所在行号
    ///
    /// # 返回
    /// 返回对应的属性定义
    pub fn to_property_definition(&self, line: usize) -> PropertyDefinition {
        PropertyDefinition::new(
            self.param_name.clone(),
            self.param_type.clone(),
            self.is_readonly,
            self.visibility,
            false,  // 构造函数属性提升不支持静态属性
            line,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试属性定义创建
    #[test]
    fn test_property_creation() {
        // 创建只读属性
        let prop = PropertyDefinition::public_readonly(
            "id".to_string(),
            "int".to_string(),
            10,
        );
        
        // 验证
        assert_eq!(prop.name, "id");
        assert!(prop.is_readonly);
        assert_eq!(prop.visibility, PropertyVisibility::Public);
        assert!(!prop.is_static);
    }
    
    /// 测试只读属性管理器
    #[test]
    fn test_readonly_manager() {
        // 创建管理器
        let mut manager = ReadonlyPropertyManager::new("User".to_string());
        
        // 添加只读属性
        let prop = PropertyDefinition::public_readonly(
            "id".to_string(),
            "int".to_string(),
            10,
        );
        manager.add_property(prop).unwrap();
        
        // 验证
        assert!(manager.is_readonly("id"));
        assert!(!manager.is_initialized("id"));
        
        // 在构造函数中初始化
        let result = manager.try_set_property("id", true);
        assert!(matches!(result, PropertyAccessResult::Success));
        assert!(manager.is_initialized("id"));
        
        // 尝试再次修改
        let result = manager.try_set_property("id", true);
        assert!(matches!(result, PropertyAccessResult::ReadonlyViolation(_)));
    }
    
    /// 测试只读属性不能在构造函数外初始化
    #[test]
    fn test_readonly_outside_constructor() {
        // 创建管理器
        let mut manager = ReadonlyPropertyManager::new("User".to_string());
        
        // 添加只读属性
        let prop = PropertyDefinition::public_readonly(
            "id".to_string(),
            "int".to_string(),
            10,
        );
        manager.add_property(prop).unwrap();
        
        // 在构造函数外尝试初始化
        let result = manager.try_set_property("id", false);
        assert!(matches!(result, PropertyAccessResult::ReadonlyViolation(_)));
    }
    
    /// 测试非只读属性可以自由修改
    #[test]
    fn test_non_readonly_can_modify() {
        // 创建管理器
        let mut manager = ReadonlyPropertyManager::new("User".to_string());
        
        // 添加非只读属性
        let prop = PropertyDefinition::public(
            "name".to_string(),
            Some("string".to_string()),
            10,
        );
        manager.add_property(prop).unwrap();
        
        // 验证可以修改
        let result = manager.try_set_property("name", false);
        assert!(matches!(result, PropertyAccessResult::Success));
    }
    
    /// 测试构造函数属性提升
    #[test]
    fn test_constructor_property_promotion() {
        // 创建构造函数属性提升
        let promotion = ConstructorPropertyPromotion::new(
            "id".to_string(),
            Some("int".to_string()),
            PropertyVisibility::Public,
            true,
        );
        
        // 转换为属性定义
        let prop = promotion.to_property_definition(10);
        
        // 验证
        assert_eq!(prop.name, "id");
        assert!(prop.is_readonly);
        assert_eq!(prop.visibility, PropertyVisibility::Public);
    }
    
    /// 测试验证所有只读属性已初始化
    #[test]
    fn test_validate_all_readonly_initialized() {
        // 创建管理器
        let mut manager = ReadonlyPropertyManager::new("User".to_string());
        
        // 添加只读属性（无默认值）
        let prop1 = PropertyDefinition::public_readonly(
            "id".to_string(),
            "int".to_string(),
            10,
        );
        manager.add_property(prop1).unwrap();
        
        // 验证应该失败
        assert!(manager.validate_all_readonly_initialized().is_err());
        
        // 初始化属性
        manager.mark_readonly_initialized("id").unwrap();
        
        // 验证应该成功
        assert!(manager.validate_all_readonly_initialized().is_ok());
    }
    
    /// 测试带默认值的只读属性
    #[test]
    fn test_readonly_with_default_value() {
        // 创建只读属性
        let mut prop = PropertyDefinition::public_readonly(
            "status".to_string(),
            "string".to_string(),
            10,
        );
        
        // 设置默认值
        prop.set_default_value("\"active\"".to_string());
        
        // 验证已初始化
        assert!(prop.is_initialized);
        assert!(!prop.can_modify());
    }
}
