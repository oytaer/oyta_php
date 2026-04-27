//! 枚举类型模块
//!
//! 实现 PHP 8.1+ 的枚举（Enum）功能支持
//! 枚举是 PHP 8.1 引入的特性，用于定义一组命名的常量
//!
//! # 功能特性
//! - 纯枚举（Pure Enum）
//! - 回退枚举（Backed Enum）
//! - 枚举方法
//! - 枚举常量
//! - 枚举接口实现
//!
//! # 使用示例
//! ```php
//! // 纯枚举
//! enum Status {
//!     case Active;
//!     case Inactive;
//!     case Pending;
//! }
//!
//! // 回退枚举
//! enum UserRole: string {
//!     case Admin = 'admin';
//!     case User = 'user';
//!     case Guest = 'guest';
//! }
//!
//! // 枚举方法
//! enum Status {
//!     case Active;
//!     case Inactive;
//!     
//!     public function label(): string {
//!         return match($this) {
//!             self::Active => '激活',
//!             self::Inactive => '禁用',
//!         };
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 枚举类型
/// 表示一个 PHP 枚举的定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumType {
    /// 枚举名称
    /// 例如：Status, UserRole
    pub name: String,
    
    /// 枚举命名空间
    /// 例如：App\Enums
    pub namespace: Option<String>,
    
    /// 完整类名（包含命名空间）
    /// 例如：App\Enums\Status
    pub full_name: String,
    
    /// 枚举类型
    /// 纯枚举或回退枚举
    pub backing_type: EnumBackingType,
    
    /// 枚举成员列表
    /// 键为成员名称，值为枚举成员
    pub cases: HashMap<String, EnumCase>,
    
    /// 枚举方法列表
    /// 键为方法名，值为方法定义
    pub methods: HashMap<String, EnumMethod>,
    
    /// 枚举常量列表
    /// 键为常量名，值为常量值
    pub constants: HashMap<String, String>,
    
    /// 实现的接口列表
    pub implements: Vec<String>,
    
    /// 使用的 trait 列表
    pub uses: Vec<String>,
    
    /// 枚举所在文件路径
    pub file_path: String,
    
    /// 枚举起始行号
    pub start_line: usize,
    
    /// 枚举结束行号
    pub end_line: usize,
}

/// 枚举回退类型
/// 表示枚举的值类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnumBackingType {
    /// 纯枚举（无回退值）
    /// 例如：enum Status { case Active; }
    Pure,
    
    /// 整数回退枚举
    /// 例如：enum Status: int { case Active = 1; }
    Int,
    
    /// 字符串回退枚举
    /// 例如：enum Status: string { case Active = 'active'; }
    String,
}

/// 枚举成员
/// 表示枚举中的一个 case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumCase {
    /// 成员名称
    /// 例如：Active, Inactive
    pub name: String,
    
    /// 回退值（JSON 序列化存储）
    /// 对于纯枚举，此值为 None
    /// 对于回退枚举，此值存储实际的值
    pub value_json: Option<String>,
    
    /// 成员所在行号
    pub line: usize,
    
    /// 成员文档注释
    pub doc_comment: Option<String>,
}

/// 枚举方法
/// 表示枚举中定义的方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumMethod {
    /// 方法名称
    pub name: String,
    
    /// 方法参数（JSON 序列化存储）
    pub params_json: String,
    
    /// 返回类型
    pub return_type: Option<String>,
    
    /// 方法体代码
    pub body: String,
    
    /// 是否为静态方法
    pub is_static: bool,
    
    /// 可见性
    pub visibility: EnumVisibility,
    
    /// 方法所在行号
    pub line: usize,
}

/// 枚举可见性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnumVisibility {
    /// 公开
    Public,
    /// 受保护
    Protected,
    /// 私有
    Private,
}

/// 枚举成员实例
/// 表示枚举的一个具体实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumInstance {
    /// 枚举类型名称
    pub enum_name: String,
    
    /// 枚举完整类名
    pub enum_full_name: String,
    
    /// 成员名称
    pub case_name: String,
    
    /// 回退值（JSON 序列化存储）
    pub value_json: Option<String>,
}

impl EnumType {
    /// 创建新的枚举类型
    ///
    /// # 参数
    /// - `name`: 枚举名称
    /// - `namespace`: 命名空间（可选）
    /// - `backing_type`: 回退类型
    /// - `file_path`: 文件路径
    /// - `start_line`: 起始行号
    ///
    /// # 返回
    /// 返回新创建的枚举类型实例
    pub fn new(
        name: String,
        namespace: Option<String>,
        backing_type: EnumBackingType,
        file_path: String,
        start_line: usize,
    ) -> Self {
        // 计算完整类名
        let full_name = match &namespace {
            Some(ns) => format!("{}\\{}", ns, name),
            None => name.clone(),
        };
        
        // 创建枚举类型实例
        Self {
            // 设置枚举名称
            name,
            // 设置命名空间
            namespace,
            // 设置完整类名
            full_name,
            // 设置回退类型
            backing_type,
            // 初始化成员列表为空
            cases: HashMap::new(),
            // 初始化方法列表为空
            methods: HashMap::new(),
            // 初始化常量列表为空
            constants: HashMap::new(),
            // 初始化接口列表为空
            implements: Vec::new(),
            // 初始化 trait 列表为空
            uses: Vec::new(),
            // 设置文件路径
            file_path,
            // 设置起始行号
            start_line,
            // 结束行号默认为 0，后续更新
            end_line: 0,
        }
    }
    
    /// 添加枚举成员
    ///
    /// # 参数
    /// - `name`: 成员名称
    /// - `value_json`: 回退值（JSON 字符串，可选）
    /// - `line`: 所在行号
    /// - `doc_comment`: 文档注释（可选）
    ///
    /// # 返回
    /// 成功返回 Ok(())，如果成员已存在返回错误
    pub fn add_case(
        &mut self,
        name: String,
        value_json: Option<String>,
        line: usize,
        doc_comment: Option<String>,
    ) -> Result<(), String> {
        // 检查成员是否已存在
        if self.cases.contains_key(&name) {
            return Err(format!("Enum case '{}' already exists", name));
        }
        
        // 验证回退值
        if value_json.is_some() && self.backing_type == EnumBackingType::Pure {
            return Err("Pure enum cannot have backed values".to_string());
        }
        
        // 验证纯枚举必须有回退值
        if value_json.is_none() && self.backing_type != EnumBackingType::Pure {
            return Err("Backed enum must have a value for each case".to_string());
        }
        
        // 创建枚举成员
        let case = EnumCase {
            // 设置成员名称
            name: name.clone(),
            // 设置回退值
            value_json,
            // 设置行号
            line,
            // 设置文档注释
            doc_comment,
        };
        
        // 添加到成员列表
        self.cases.insert(name, case);
        
        Ok(())
    }
    
    /// 添加枚举方法
    ///
    /// # 参数
    /// - `name`: 方法名称
    /// - `params_json`: 参数列表（JSON 字符串）
    /// - `return_type`: 返回类型（可选）
    /// - `body`: 方法体代码
    /// - `is_static`: 是否为静态方法
    /// - `visibility`: 可见性
    /// - `line`: 所在行号
    ///
    /// # 返回
    /// 成功返回 Ok(())，如果方法已存在返回错误
    pub fn add_method(
        &mut self,
        name: String,
        params_json: String,
        return_type: Option<String>,
        body: String,
        is_static: bool,
        visibility: EnumVisibility,
        line: usize,
    ) -> Result<(), String> {
        // 检查方法是否已存在
        if self.methods.contains_key(&name) {
            return Err(format!("Enum method '{}' already exists", name));
        }
        
        // 创建枚举方法
        let method = EnumMethod {
            // 设置方法名称
            name: name.clone(),
            // 设置参数
            params_json,
            // 设置返回类型
            return_type,
            // 设置方法体
            body,
            // 设置是否为静态方法
            is_static,
            // 设置可见性
            visibility,
            // 设置行号
            line,
        };
        
        // 添加到方法列表
        self.methods.insert(name, method);
        
        Ok(())
    }
    
    /// 添加枚举常量
    ///
    /// # 参数
    /// - `name`: 常量名称
    /// - `value_json`: 常量值（JSON 字符串）
    ///
    /// # 返回
    /// 成功返回 Ok(())，如果常量已存在返回错误
    pub fn add_constant(&mut self, name: String, value_json: String) -> Result<(), String> {
        // 检查常量是否已存在
        if self.constants.contains_key(&name) {
            return Err(format!("Enum constant '{}' already exists", name));
        }
        
        // 添加到常量列表
        self.constants.insert(name, value_json);
        
        Ok(())
    }
    
    /// 添加实现的接口
    ///
    /// # 参数
    /// - `interface_name`: 接口名称
    pub fn add_implements(&mut self, interface_name: String) {
        // 添加到接口列表
        self.implements.push(interface_name);
    }
    
    /// 添加使用的 trait
    ///
    /// # 参数
    /// - `trait_name`: trait 名称
    pub fn add_trait(&mut self, trait_name: String) {
        // 添加到 trait 列表
        self.uses.push(trait_name);
    }
    
    /// 获取枚举成员
    ///
    /// # 参数
    /// - `name`: 成员名称
    ///
    /// # 返回
    /// 返回枚举成员，如果不存在返回 None
    pub fn get_case(&self, name: &str) -> Option<&EnumCase> {
        // 从成员列表中查找
        self.cases.get(name)
    }
    
    /// 获取所有成员名称
    ///
    /// # 返回
    /// 返回所有成员名称的列表
    pub fn get_case_names(&self) -> Vec<String> {
        // 返回成员名称列表
        self.cases.keys().cloned().collect()
    }
    
    /// 获取枚举方法
    ///
    /// # 参数
    /// - `name`: 方法名称
    ///
    /// # 返回
    /// 返回枚举方法，如果不存在返回 None
    pub fn get_method(&self, name: &str) -> Option<&EnumMethod> {
        // 从方法列表中查找
        self.methods.get(name)
    }
    
    /// 创建枚举实例
    ///
    /// # 参数
    /// - `case_name`: 成员名称
    ///
    /// # 返回
    /// 成功返回枚举实例，失败返回错误
    pub fn create_instance(&self, case_name: &str) -> Result<EnumInstance, String> {
        // 查找枚举成员
        let case = self.cases.get(case_name).ok_or_else(|| {
            format!("Enum case '{}' not found in '{}'", case_name, self.full_name)
        })?;
        
        // 创建枚举实例
        Ok(EnumInstance {
            // 设置枚举类型名称
            enum_name: self.name.clone(),
            // 设置枚举完整类名
            enum_full_name: self.full_name.clone(),
            // 设置成员名称
            case_name: case.name.clone(),
            // 设置回退值
            value_json: case.value_json.clone(),
        })
    }
    
    /// 根据值查找成员
    /// 仅适用于回退枚举
    ///
    /// # 参数
    /// - `value_json`: 回退值（JSON 字符串）
    ///
    /// # 返回
    /// 返回对应的成员名称，如果不存在返回 None
    pub fn find_case_by_value(&self, value_json: &str) -> Option<String> {
        // 遍历所有成员
        for (name, case) in &self.cases {
            // 比较值
            if case.value_json.as_deref() == Some(value_json) {
                return Some(name.clone());
            }
        }
        None
    }
    
    /// 检查是否为回退枚举
    ///
    /// # 返回
    /// 如果是回退枚举返回 true，否则返回 false
    pub fn is_backed(&self) -> bool {
        // 检查回退类型
        self.backing_type != EnumBackingType::Pure
    }
    
    /// 获取枚举类型名称
    ///
    /// # 返回
    /// 返回枚举类型的字符串表示
    pub fn get_type_name(&self) -> &str {
        &self.name
    }
    
    /// 获取枚举完整类名
    ///
    /// # 返回
    /// 返回枚举的完整类名
    pub fn get_full_name(&self) -> &str {
        &self.full_name
    }
    
    /// 设置结束行号
    ///
    /// # 参数
    /// - `line`: 结束行号
    pub fn set_end_line(&mut self, line: usize) {
        self.end_line = line;
    }
}

impl EnumInstance {
    /// 创建新的枚举实例
    ///
    /// # 参数
    /// - `enum_name`: 枚举类型名称
    /// - `enum_full_name`: 枚举完整类名
    /// - `case_name`: 成员名称
    /// - `value_json`: 回退值（JSON 字符串，可选）
    ///
    /// # 返回
    /// 返回新创建的枚举实例
    pub fn new(
        enum_name: String,
        enum_full_name: String,
        case_name: String,
        value_json: Option<String>,
    ) -> Self {
        Self {
            enum_name,
            enum_full_name,
            case_name,
            value_json,
        }
    }
    
    /// 获取枚举成员名称
    ///
    /// # 返回
    /// 返回成员名称
    pub fn get_name(&self) -> &str {
        &self.case_name
    }
    
    /// 获取回退值
    ///
    /// # 返回
    /// 返回回退值的 JSON 字符串，如果是纯枚举返回 None
    pub fn get_value(&self) -> Option<&str> {
        self.value_json.as_deref()
    }
    
    /// 获取枚举类型名称
    ///
    /// # 返回
    /// 返回枚举类型名称
    pub fn get_enum_name(&self) -> &str {
        &self.enum_name
    }
    
    /// 获取枚举完整类名
    ///
    /// # 返回
    /// 返回枚举完整类名
    pub fn get_enum_full_name(&self) -> &str {
        &self.enum_full_name
    }
    
    /// 转换为字符串表示
    ///
    /// # 返回
    /// 返回枚举实例的字符串表示，格式为 "EnumName::CaseName"
    pub fn to_string(&self) -> String {
        format!("{}::{}", self.enum_name, self.case_name)
    }
}

/// 枚举工厂
/// 用于创建和管理枚举类型
pub struct EnumFactory {
    /// 已注册的枚举类型列表
    /// 键为完整类名，值为枚举类型
    enums: HashMap<String, EnumType>,
}

impl EnumFactory {
    /// 创建新的枚举工厂
    pub fn new() -> Self {
        Self {
            enums: HashMap::new(),
        }
    }
    
    /// 注册枚举类型
    ///
    /// # 参数
    /// - `enum_type`: 枚举类型
    ///
    /// # 返回
    /// 成功返回 Ok(())，如果枚举已存在返回错误
    pub fn register(&mut self, enum_type: EnumType) -> Result<(), String> {
        // 检查枚举是否已存在
        if self.enums.contains_key(&enum_type.full_name) {
            return Err(format!("Enum '{}' already registered", enum_type.full_name));
        }
        
        // 注册枚举类型
        self.enums.insert(enum_type.full_name.clone(), enum_type);
        
        Ok(())
    }
    
    /// 获取枚举类型
    ///
    /// # 参数
    /// - `full_name`: 枚举完整类名
    ///
    /// # 返回
    /// 返回枚举类型，如果不存在返回 None
    pub fn get(&self, full_name: &str) -> Option<&EnumType> {
        self.enums.get(full_name)
    }
    
    /// 获取枚举类型（可变引用）
    ///
    /// # 参数
    /// - `full_name`: 枚举完整类名
    ///
    /// # 返回
    /// 返回枚举类型的可变引用，如果不存在返回 None
    pub fn get_mut(&mut self, full_name: &str) -> Option<&mut EnumType> {
        self.enums.get_mut(full_name)
    }
    
    /// 检查枚举是否存在
    ///
    /// # 参数
    /// - `full_name`: 枚举完整类名
    ///
    /// # 返回
    /// 如果枚举存在返回 true，否则返回 false
    pub fn exists(&self, full_name: &str) -> bool {
        self.enums.contains_key(full_name)
    }
    
    /// 获取所有已注册的枚举名称
    ///
    /// # 返回
    /// 返回所有枚举完整类名的列表
    pub fn get_all_names(&self) -> Vec<String> {
        self.enums.keys().cloned().collect()
    }
    
    /// 注销枚举类型
    ///
    /// # 参数
    /// - `full_name`: 枚举完整类名
    ///
    /// # 返回
    /// 成功返回被注销的枚举类型，如果不存在返回 None
    pub fn unregister(&mut self, full_name: &str) -> Option<EnumType> {
        self.enums.remove(full_name)
    }
    
    /// 获取已注册的枚举数量
    ///
    /// # 返回
    /// 返回已注册的枚举数量
    pub fn count(&self) -> usize {
        self.enums.len()
    }
}

impl Default for EnumFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试纯枚举创建
    #[test]
    fn test_pure_enum_creation() {
        // 创建纯枚举
        let mut enum_type = EnumType::new(
            "Status".to_string(),
            None,
            EnumBackingType::Pure,
            "test.php".to_string(),
            1,
        );
        
        // 添加成员
        enum_type.add_case("Active".to_string(), None, 2, None).unwrap();
        enum_type.add_case("Inactive".to_string(), None, 3, None).unwrap();
        
        // 验证
        assert_eq!(enum_type.name, "Status");
        assert_eq!(enum_type.cases.len(), 2);
        assert!(!enum_type.is_backed());
    }
    
    /// 测试回退枚举创建
    #[test]
    fn test_backed_enum_creation() {
        // 创建字符串回退枚举
        let mut enum_type = EnumType::new(
            "UserRole".to_string(),
            Some("App\\Enums".to_string()),
            EnumBackingType::String,
            "test.php".to_string(),
            1,
        );
        
        // 添加成员
        enum_type.add_case("Admin".to_string(), Some("\"admin\"".to_string()), 2, None).unwrap();
        enum_type.add_case("User".to_string(), Some("\"user\"".to_string()), 3, None).unwrap();
        
        // 验证
        assert_eq!(enum_type.full_name, "App\\Enums\\UserRole");
        assert!(enum_type.is_backed());
        
        // 根据值查找成员
        let case = enum_type.find_case_by_value("\"admin\"");
        assert_eq!(case, Some("Admin".to_string()));
    }
    
    /// 测试枚举实例创建
    #[test]
    fn test_enum_instance() {
        // 创建枚举
        let mut enum_type = EnumType::new(
            "Status".to_string(),
            None,
            EnumBackingType::Pure,
            "test.php".to_string(),
            1,
        );
        
        // 添加成员
        enum_type.add_case("Active".to_string(), None, 2, None).unwrap();
        
        // 创建实例
        let instance = enum_type.create_instance("Active").unwrap();
        
        // 验证
        assert_eq!(instance.get_name(), "Active");
        assert_eq!(instance.to_string(), "Status::Active");
    }
    
    /// 测试枚举方法
    #[test]
    fn test_enum_method() {
        // 创建枚举
        let mut enum_type = EnumType::new(
            "Status".to_string(),
            None,
            EnumBackingType::Pure,
            "test.php".to_string(),
            1,
        );
        
        // 添加方法
        enum_type.add_method(
            "label".to_string(),
            "[]".to_string(),
            Some("string".to_string()),
            "return 'label';".to_string(),
            false,
            EnumVisibility::Public,
            5,
        ).unwrap();
        
        // 验证
        assert!(enum_type.get_method("label").is_some());
        assert_eq!(enum_type.methods.len(), 1);
    }
    
    /// 测试枚举工厂
    #[test]
    fn test_enum_factory() {
        // 创建工厂
        let mut factory = EnumFactory::new();
        
        // 创建枚举
        let enum_type = EnumType::new(
            "Status".to_string(),
            None,
            EnumBackingType::Pure,
            "test.php".to_string(),
            1,
        );
        
        // 注册枚举
        factory.register(enum_type).unwrap();
        
        // 验证
        assert!(factory.exists("Status"));
        assert_eq!(factory.count(), 1);
        
        // 注销枚举
        factory.unregister("Status");
        assert!(!factory.exists("Status"));
    }
    
    /// 测试纯枚举不能有回退值
    #[test]
    fn test_pure_enum_cannot_have_value() {
        // 创建纯枚举
        let mut enum_type = EnumType::new(
            "Status".to_string(),
            None,
            EnumBackingType::Pure,
            "test.php".to_string(),
            1,
        );
        
        // 尝试添加带值的成员，应该失败
        let result = enum_type.add_case("Active".to_string(), Some("\"value\"".to_string()), 2, None);
        assert!(result.is_err());
    }
    
    /// 测试回退枚举必须有值
    #[test]
    fn test_backed_enum_must_have_value() {
        // 创建回退枚举
        let mut enum_type = EnumType::new(
            "UserRole".to_string(),
            None,
            EnumBackingType::String,
            "test.php".to_string(),
            1,
        );
        
        // 尝试添加不带值的成员，应该失败
        let result = enum_type.add_case("Admin".to_string(), None, 2, None);
        assert!(result.is_err());
    }
}
