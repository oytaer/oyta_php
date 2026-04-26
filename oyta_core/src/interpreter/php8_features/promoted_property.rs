//! 构造器属性提升模块
//!
//! PHP 8.0 允许在构造器参数中直接声明属性

use crate::interpreter::value::Value;

/// 构造器属性提升
///
/// PHP 8.0 允许在构造器参数中直接声明属性
///
/// # 示例
/// ```php
/// // PHP 7.x
/// class User {
///     public string $name;
///     public int $age;
///
///     public function __construct(string $name, int $age) {
///         $this->name = $name;
///         $this->age = $age;
///     }
/// }
///
/// // PHP 8.0 构造器属性提升
/// class User {
///     public function __construct(
///         public string $name,
///         public int $age
///     ) {}
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PromotedProperty {
    /// 属性名
    pub name: String,
    /// 类型
    pub type_hint: String,
    /// 可见性
    pub visibility: PropertyVisibility,
    /// 是否只读
    pub readonly: bool,
    /// 默认值
    pub default_value: Option<Value>,
}

/// 属性可见性
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropertyVisibility {
    /// 公开
    Public,
    /// 受保护
    Protected,
    /// 私有
    Private,
}

impl PromotedProperty {
    /// 创建新的提升属性
    ///
    /// # 参数
    /// - `name`: 属性名
    /// - `type_hint`: 类型提示
    /// - `visibility`: 可见性
    ///
    /// # 返回值
    /// 新的 PromotedProperty 实例
    pub fn new(name: &str, type_hint: &str, visibility: PropertyVisibility) -> Self {
        Self {
            name: name.to_string(),
            type_hint: type_hint.to_string(),
            visibility,
            readonly: false,
            default_value: None,
        }
    }

    /// 设置为只读
    ///
    /// # 返回值
    /// 返回自身，支持链式调用
    pub fn with_readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    /// 设置默认值
    ///
    /// # 参数
    /// - `value`: 默认值
    ///
    /// # 返回值
    /// 返回自身，支持链式调用
    pub fn with_default(mut self, value: Value) -> Self {
        self.default_value = Some(value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试提升属性
    #[test]
    fn test_promoted_property() {
        let prop = PromotedProperty::new("name", "string", PropertyVisibility::Public)
            .with_readonly()
            .with_default(Value::String("default".to_string()));

        assert_eq!(prop.name, "name");
        assert!(prop.readonly);
        assert!(prop.default_value.is_some());
    }
}
