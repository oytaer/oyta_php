//! Attribute 注解模块
//!
//! PHP 8.0 引入原生注解支持
//! 注解可以应用于类、方法、属性、参数等目标

use std::collections::HashMap;

use crate::interpreter::value::Value;

/// Attribute 注解
///
/// PHP 8.0 引入原生注解支持
///
/// # 示例
/// ```php
/// #[Route('/api/users', methods: ['GET'])]
/// class UserController {
///     #[Inject]
///     private UserService $userService;
///
///     #[Route('/{id}', methods: ['GET'])]
///     public function show(int $id) {}
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Attribute {
    /// 注解类名
    pub name: String,
    /// 位置参数
    pub arguments: Vec<Value>,
    /// 命名参数
    pub named_arguments: HashMap<String, Value>,
    /// 目标（类/方法/属性/参数等）
    pub target: AttributeTarget,
}

/// 注解目标
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeTarget {
    /// 类
    Class,
    /// 方法
    Method,
    /// 属性
    Property,
    /// 参数
    Parameter,
    /// 类常量
    ClassConstant,
    /// 函数
    Function,
    /// 匿名类
    AnonymousClass,
}

impl Attribute {
    /// 创建新的注解
    ///
    /// # 参数
    /// - `name`: 注解类名
    /// - `target`: 注解目标
    ///
    /// # 返回值
    /// 新的 Attribute 实例
    pub fn new(name: &str, target: AttributeTarget) -> Self {
        Self {
            name: name.to_string(),
            arguments: Vec::new(),
            named_arguments: HashMap::new(),
            target,
        }
    }

    /// 添加位置参数（链式调用）
    ///
    /// # 参数
    /// - `value`: 参数值
    ///
    /// # 返回值
    /// 返回自身，支持链式调用
    pub fn with_argument(mut self, value: Value) -> Self {
        self.arguments.push(value);
        self
    }

    /// 添加命名参数（链式调用）
    ///
    /// # 参数
    /// - `name`: 参数名
    /// - `value`: 参数值
    ///
    /// # 返回值
    /// 返回自身，支持链式调用
    pub fn with_named_argument(mut self, name: &str, value: Value) -> Self {
        self.named_arguments.insert(name.to_string(), value);
        self
    }

    /// 添加位置参数（可变引用）
    ///
    /// # 参数
    /// - `value`: 参数值
    pub fn add_argument(&mut self, value: Value) {
        self.arguments.push(value);
    }

    /// 添加命名参数（可变引用）
    ///
    /// # 参数
    /// - `name`: 参数名
    /// - `value`: 参数值
    pub fn add_named_argument(&mut self, name: &str, value: Value) {
        self.named_arguments.insert(name.to_string(), value);
    }

    /// 获取位置参数
    ///
    /// # 参数
    /// - `index`: 参数索引
    ///
    /// # 返回值
    /// 参数值引用
    pub fn get_argument(&self, index: usize) -> Option<&Value> {
        self.arguments.get(index)
    }

    /// 获取命名参数
    ///
    /// # 参数
    /// - `name`: 参数名
    ///
    /// # 返回值
    /// 参数值引用
    pub fn get_named_argument(&self, name: &str) -> Option<&Value> {
        self.named_arguments.get(name)
    }

    /// 获取字符串参数
    ///
    /// # 参数
    /// - `index`: 参数索引
    ///
    /// # 返回值
    /// 字符串切片
    pub fn get_string_argument(&self, index: usize) -> Option<&str> {
        self.arguments.get(index).and_then(|v| v.as_str())
    }

    /// 获取整数参数
    ///
    /// # 参数
    /// - `index`: 参数索引
    ///
    /// # 返回值
    /// 整数值
    pub fn get_int_argument(&self, index: usize) -> Option<i64> {
        self.arguments.get(index).and_then(|v| v.as_i64())
    }

    /// 检查是否有指定名称的命名参数
    ///
    /// # 参数
    /// - `name`: 参数名
    ///
    /// # 返回值
    /// 如果存在返回 true
    pub fn has_named_argument(&self, name: &str) -> bool {
        self.named_arguments.contains_key(name)
    }
}

/// 注解解析器
pub struct AttributeParser {
    /// 已解析的注解列表
    attributes: Vec<Attribute>,
}

impl AttributeParser {
    /// 创建新的注解解析器
    ///
    /// # 返回值
    /// 新的 AttributeParser 实例
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }

    /// 解析注解字符串
    ///
    /// # 参数
    /// - `input`: 注解字符串（如 `#[Route('/path')]`）
    /// - `target`: 注解目标
    ///
    /// # 返回值
    /// 解析后的注解
    pub fn parse(&mut self, input: &str, target: AttributeTarget) -> Result<Attribute, String> {
        // 移除 #[ 和 ]
        let content = input.trim();
        if !content.starts_with("#[") || !content.ends_with(']') {
            return Err("无效的注解格式".to_string());
        }

        // 提取内容部分
        let content = &content[2..content.len() - 1].trim();

        // 解析类名
        let (name, args_str) = if let Some(pos) = content.find('(') {
            let name = content[..pos].trim().to_string();
            let args = content[pos..].trim();
            (name, Some(args))
        } else {
            (content.to_string(), None)
        };

        // 创建注解实例
        let mut attribute = Attribute::new(&name, target);

        // 解析参数
        if let Some(args) = args_str {
            self.parse_arguments(args, &mut attribute)?;
        }

        // 保存并返回
        self.attributes.push(attribute.clone());
        Ok(attribute)
    }

    /// 解析参数
    ///
    /// # 参数
    /// - `args`: 参数字符串
    /// - `attribute`: 注解实例
    fn parse_arguments(&self, args: &str, attribute: &mut Attribute) -> Result<(), String> {
        // 移除括号
        let args = args.trim();
        if !args.starts_with('(') || !args.ends_with(')') {
            return Err("无效的参数格式".to_string());
        }

        // 提取参数内容
        let args = &args[1..args.len() - 1].trim();
        if args.is_empty() {
            return Ok(());
        }

        // 解析参数（支持嵌套）
        self.parse_argument_list(args, attribute)?;

        Ok(())
    }

    /// 解析参数列表（支持嵌套结构和数组）
    ///
    /// # 参数
    /// - `args`: 参数列表字符串
    /// - `attribute`: 注解实例
    fn parse_argument_list(&self, args: &str, attribute: &mut Attribute) -> Result<(), String> {
        let mut depth = 0;
        let mut current = String::new();
        let mut in_string = false;
        let mut string_char = ' ';

        // 遍历每个字符
        for ch in args.chars() {
            // 处理字符串
            if (ch == '"' || ch == '\'') && depth == 0 {
                if !in_string {
                    in_string = true;
                    string_char = ch;
                } else if ch == string_char {
                    in_string = false;
                }
                current.push(ch);
                continue;
            }

            // 字符串内的字符直接添加
            if in_string {
                current.push(ch);
                continue;
            }

            // 处理嵌套结构
            match ch {
                '[' | '(' | '{' => {
                    depth += 1;
                    current.push(ch);
                }
                ']' | ')' | '}' => {
                    depth -= 1;
                    current.push(ch);
                }
                ',' if depth == 0 => {
                    // 分隔符，解析当前参数
                    self.parse_single_argument(current.trim(), attribute)?;
                    current.clear();
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        // 解析最后一个参数
        if !current.trim().is_empty() {
            self.parse_single_argument(current.trim(), attribute)?;
        }

        Ok(())
    }

    /// 解析单个参数
    ///
    /// # 参数
    /// - `arg`: 参数字符串
    /// - `attribute`: 注解实例
    fn parse_single_argument(&self, arg: &str, attribute: &mut Attribute) -> Result<(), String> {
        let arg = arg.trim();
        if arg.is_empty() {
            return Ok(());
        }

        // 检查是否为命名参数
        if arg.contains(':') && !arg.starts_with('[') {
            let mut iter = arg.splitn(2, ':');
            let name = iter.next().unwrap().trim();
            if let Some(value_str) = iter.next() {
                // 确保名称是有效的标识符
                if name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    let value = self.parse_value(value_str.trim())?;
                    attribute.add_named_argument(name, value);
                    return Ok(());
                }
            }
        }

        // 位置参数
        let value = self.parse_value(arg)?;
        attribute.add_argument(value);

        Ok(())
    }

    /// 解析值
    ///
    /// # 参数
    /// - `value`: 值字符串
    ///
    /// # 返回值
    /// 解析后的 Value
    fn parse_value(&self, value: &str) -> Result<Value, String> {
        let value = value.trim();

        // 字符串
        if (value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\''))
        {
            return Ok(Value::String(value[1..value.len() - 1].to_string()));
        }

        // 数组
        if value.starts_with('[') && value.ends_with(']') {
            return self.parse_array(value);
        }

        // 整数
        if let Ok(i) = value.parse::<i64>() {
            return Ok(Value::Int(i));
        }

        // 浮点数
        if let Ok(f) = value.parse::<f64>() {
            return Ok(Value::Float(f));
        }

        // 布尔值
        match value.to_lowercase().as_str() {
            "true" => return Ok(Value::Bool(true)),
            "false" => return Ok(Value::Bool(false)),
            "null" => return Ok(Value::Null),
            _ => {}
        }

        // 常量引用或类名
        Ok(Value::String(value.to_string()))
    }

    /// 解析数组
    ///
    /// # 参数
    /// - `value`: 数组字符串
    ///
    /// # 返回值
    /// 解析后的 Value
    fn parse_array(&self, value: &str) -> Result<Value, String> {
        // 提取数组内容
        let content = &value[1..value.len() - 1].trim();

        // 空数组
        if content.is_empty() {
            return Ok(Value::IndexedArray(Vec::new()));
        }

        // 尝试解析为索引数组或关联数组
        let mut indexed = Vec::new();
        let mut associative = HashMap::new();
        let mut is_associative = false;

        let mut depth = 0;
        let mut current = String::new();
        let mut in_string = false;
        let mut string_char = ' ';

        // 遍历每个字符
        for ch in content.chars() {
            // 处理字符串
            if (ch == '"' || ch == '\'') && depth == 0 {
                if !in_string {
                    in_string = true;
                    string_char = ch;
                } else if ch == string_char {
                    in_string = false;
                }
                current.push(ch);
                continue;
            }

            // 字符串内的字符直接添加
            if in_string {
                current.push(ch);
                continue;
            }

            // 处理嵌套结构
            match ch {
                '[' | '(' | '{' => {
                    depth += 1;
                    current.push(ch);
                }
                ']' | ')' | '}' => {
                    depth -= 1;
                    current.push(ch);
                }
                ',' if depth == 0 => {
                    // 分隔符
                    if let Some((key, val)) = self.parse_array_item(current.trim()) {
                        if let Some(k) = key {
                            is_associative = true;
                            associative.insert(k, val);
                        } else {
                            indexed.push(val);
                        }
                    }
                    current.clear();
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        // 解析最后一个元素
        if !current.trim().is_empty() {
            if let Some((key, val)) = self.parse_array_item(current.trim()) {
                if let Some(k) = key {
                    is_associative = true;
                    associative.insert(k, val);
                } else {
                    indexed.push(val);
                }
            }
        }

        // 根据类型返回
        if is_associative {
            // 将 HashMap 转换为 Vec<(String, Value)>
            let vec: Vec<(String, Value)> = associative.into_iter().collect();
            Ok(Value::AssociativeArray(vec))
        } else {
            Ok(Value::IndexedArray(indexed))
        }
    }

    /// 解析数组元素
    ///
    /// # 参数
    /// - `item`: 元素字符串
    ///
    /// # 返回值
    /// 元组的键（可选）和值
    fn parse_array_item(&self, item: &str) -> Option<(Option<String>, Value)> {
        let item = item.trim();
        if item.is_empty() {
            return None;
        }

        // 检查是否有关联键
        if item.contains("=>") {
            let mut parts = item.splitn(2, "=>");
            let key_str = parts.next()?.trim();
            let value_str = parts.next()?.trim();

            // 解析键
            let key = if key_str.starts_with('"') || key_str.starts_with('\'') {
                key_str[1..key_str.len() - 1].to_string()
            } else {
                key_str.to_string()
            };

            // 解析值
            let value = self.parse_value(value_str).ok()?;
            return Some((Some(key), value));
        }

        // 索引数组元素
        let value = self.parse_value(item).ok()?;
        Some((None, value))
    }

    /// 获取所有注解
    ///
    /// # 返回值
    /// 注解列表引用
    pub fn get_attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    /// 按名称获取注解
    ///
    /// # 参数
    /// - `name`: 注解名称
    ///
    /// # 返回值
    /// 匹配的注解列表
    pub fn get_by_name(&self, name: &str) -> Vec<&Attribute> {
        self.attributes
            .iter()
            .filter(|a| a.name == name)
            .collect()
    }

    /// 按目标获取注解
    ///
    /// # 参数
    /// - `target`: 注解目标
    ///
    /// # 返回值
    /// 匹配的注解列表
    pub fn get_by_target(&self, target: AttributeTarget) -> Vec<&Attribute> {
        self.attributes
            .iter()
            .filter(|a| a.target == target)
            .collect()
    }

    /// 清空已解析的注解
    pub fn clear(&mut self) {
        self.attributes.clear();
    }
}

impl Default for AttributeParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试基本的注解解析
    #[test]
    fn test_attribute_parser() {
        let mut parser = AttributeParser::new();
        let attr = parser
            .parse("#[Route('/api/users')]", AttributeTarget::Class)
            .unwrap();

        assert_eq!(attr.name, "Route");
        assert_eq!(attr.arguments.len(), 1);
    }

    /// 测试带命名参数的注解解析
    #[test]
    fn test_attribute_parser_with_named() {
        let mut parser = AttributeParser::new();
        let attr = parser
            .parse(
                "#[Route(path: '/api/users', methods: ['GET', 'POST'])]",
                AttributeTarget::Class,
            )
            .unwrap();

        assert_eq!(attr.name, "Route");
        assert!(attr.has_named_argument("path"));
        assert!(attr.has_named_argument("methods"));
    }

    /// 测试带数组参数的注解解析
    #[test]
    fn test_attribute_parser_array() {
        let mut parser = AttributeParser::new();
        let attr = parser
            .parse("#[Route(methods: ['GET', 'POST'])]", AttributeTarget::Method)
            .unwrap();

        assert_eq!(attr.name, "Route");
        let methods = attr.get_named_argument("methods").unwrap();
        if let Value::IndexedArray(arr) = methods {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected IndexedArray");
        }
    }
}
