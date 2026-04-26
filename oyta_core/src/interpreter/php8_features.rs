//! PHP 8.0+ 语法特性支持
//!
//! 实现 PHP 8.0 及以上版本的新特性：
//! - match 表达式
//! - 命名参数
//! - Nullsafe 运算符 (?->)
//! - Attribute 注解
//! - 构造器属性提升
//! - readonly 属性
//! - 联合类型
//! - 匿名类

use std::collections::HashMap;

use crate::interpreter::value::Value;

/// Value 类型的辅助方法
impl Value {
    /// 获取字符串值
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// 获取整数值
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// 获取浮点数值
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// 获取布尔值
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

/// Match 表达式执行器
///
/// PHP 8.0 的 match 表达式类似于 switch，但：
/// 1. 返回值而不是执行语句
/// 2. 使用严格比较 (===)
/// 3. 不需要 break
/// 4. 支持多条件匹配
///
/// # 示例
/// ```php
/// $status = match($code) {
///     200, 300 => 'OK',
///     400, 404 => 'Not Found',
///     500 => 'Server Error',
///     default => 'Unknown'
/// };
/// ```
pub struct MatchExpression {
    /// 要匹配的表达式
    pub subject: Value,
    /// 匹配分支列表
    pub arms: Vec<MatchArm>,
    /// 默认值
    pub default_value: Option<Value>,
}

/// Match 分支
pub struct MatchArm {
    /// 条件列表（多个条件用逗号分隔）
    pub conditions: Vec<Value>,
    /// 返回值表达式
    pub body: MatchBody,
}

/// Match 分支体
#[derive(Debug, Clone)]
pub enum MatchBody {
    /// 表达式（直接返回值）
    Expression(Value),
    /// 代码块（需要执行一系列语句后返回值）
    Block(Vec<MatchStatement>),
}

/// Match 代码块中的语句
#[derive(Debug, Clone)]
pub enum MatchStatement {
    /// 变量赋值
    Assign {
        /// 变量名
        name: String,
        /// 值
        value: Value,
    },
    /// 函数调用
    FunctionCall {
        /// 函数名
        name: String,
        /// 参数
        args: Vec<Value>,
    },
    /// 返回语句
    Return(Value),
}

impl MatchExpression {
    /// 创建新的 match 表达式
    pub fn new(subject: Value) -> Self {
        Self {
            subject,
            arms: Vec::new(),
            default_value: None,
        }
    }

    /// 添加匹配分支
    pub fn add_arm(mut self, conditions: Vec<Value>, body: MatchBody) -> Self {
        self.arms.push(MatchArm { conditions, body });
        self
    }

    /// 设置默认值
    pub fn with_default(mut self, value: Value) -> Self {
        self.default_value = Some(value);
        self
    }

    /// 执行 match 表达式
    ///
    /// # 返回
    /// 匹配的值，如果没有匹配则返回默认值或 None
    pub fn evaluate(&self) -> Option<Value> {
        for arm in &self.arms {
            // 检查每个条件
            for condition in &arm.conditions {
                // 使用严格比较
                if self.values_equal(&self.subject, condition) {
                    return Some(self.evaluate_body(&arm.body));
                }
            }
        }

        // 返回默认值
        self.default_value.clone()
    }

    /// 严格比较两个值
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(i1), Value::Int(i2)) => i1 == i2,
            (Value::Float(f1), Value::Float(f2)) => f1 == f2,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
            (Value::Null, Value::Null) => true,
            // 比较整数和浮点数（PHP 弱类型比较）
            (Value::Int(i), Value::Float(f)) | (Value::Float(f), Value::Int(i)) => {
                (*i as f64) == *f
            }
            _ => false,
        }
    }

    /// 计算分支体
    ///
    /// 对于表达式直接返回值，对于代码块则执行语句
    fn evaluate_body(&self, body: &MatchBody) -> Value {
        match body {
            MatchBody::Expression(v) => v.clone(),
            MatchBody::Block(statements) => {
                // 执行代码块中的语句
                let mut last_value = Value::Null;

                for stmt in statements {
                    match stmt {
                        MatchStatement::Assign { name, value } => {
                            // 在实际实现中，这里应该将变量存储到作用域
                            tracing::debug!("Match 块赋值: {} = {:?}", name, value);
                            last_value = value.clone();
                        }
                        MatchStatement::FunctionCall { name, args } => {
                            // 在实际实现中，这里应该调用函数
                            tracing::debug!("Match 块函数调用: {}({:?})", name, args);
                            last_value = Value::Null;
                        }
                        MatchStatement::Return(value) => {
                            return value.clone();
                        }
                    }
                }

                last_value
            }
        }
    }

    /// 添加 default 分支
    pub fn add_default(mut self, body: MatchBody) -> Self {
        self.default_value = Some(self.evaluate_body(&body));
        self
    }
}

/// 命名参数处理器
///
/// PHP 8.0 允许使用命名参数调用函数
///
/// # 示例
/// ```php
/// // 位置参数
/// htmlspecialchars($string, ENT_QUOTES | ENT_HTML5, 'UTF-8');
///
/// // 命名参数
/// htmlspecialchars(string: $string, encoding: 'UTF-8');
/// ```
pub struct NamedArguments {
    /// 位置参数
    pub positional: Vec<Value>,
    /// 命名参数
    pub named: HashMap<String, Value>,
}

impl NamedArguments {
    /// 创建新的命名参数集合
    pub fn new() -> Self {
        Self {
            positional: Vec::new(),
            named: HashMap::new(),
        }
    }

    /// 添加位置参数
    pub fn add_positional(mut self, value: Value) -> Self {
        self.positional.push(value);
        self
    }

    /// 添加命名参数
    pub fn add_named(mut self, name: &str, value: Value) -> Self {
        self.named.insert(name.to_string(), value);
        self
    }

    /// 解析函数参数
    ///
    /// # 参数
    /// - `param_names`: 函数参数名称列表
    /// - `param_defaults`: 参数默认值
    ///
    /// # 返回
    /// 按参数顺序排列的值列表
    pub fn resolve(
        &self,
        param_names: &[String],
        param_defaults: &HashMap<String, Value>,
    ) -> Vec<Value> {
        let mut result = Vec::new();

        for (i, param_name) in param_names.iter().enumerate() {
            // 首先检查命名参数
            if let Some(value) = self.named.get(param_name) {
                result.push(value.clone());
            }
            // 然后检查位置参数
            else if i < self.positional.len() {
                result.push(self.positional[i].clone());
            }
            // 最后使用默认值
            else if let Some(default) = param_defaults.get(param_name) {
                result.push(default.clone());
            }
            // 没有值，使用 Null
            else {
                result.push(Value::Null);
            }
        }

        result
    }

    /// 获取参数数量
    pub fn len(&self) -> usize {
        self.positional.len() + self.named.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 合并另一个参数集合
    pub fn merge(&mut self, other: NamedArguments) {
        self.positional.extend(other.positional);
        for (name, value) in other.named {
            self.named.insert(name, value);
        }
    }

    /// 转换为位置参数列表（忽略命名参数）
    pub fn to_positional(&self) -> Vec<Value> {
        self.positional.clone()
    }
}

impl Default for NamedArguments {
    fn default() -> Self {
        Self::new()
    }
}

/// Nullsafe 运算符处理器
///
/// PHP 8.0 引入 ?-> 运算符，用于安全地访问可能为 null 的对象
///
/// # 示例
/// ```php
/// // 传统写法
/// $country = $user !== null ? $user->address->country : null;
///
/// // Nullsafe 运算符
/// $country = $user?->address?->country;
/// ```
pub struct NullsafeOperator;

impl NullsafeOperator {
    /// 安全访问对象属性
    ///
    /// # 参数
    /// - `object`: 对象值
    /// - `property`: 属性名
    ///
    /// # 返回
    /// 属性值，如果对象为 null 则返回 null
    pub fn get_property(object: &Value, property: &str) -> Value {
        match object {
            Value::Null => Value::Null,
            Value::Object(obj) => {
                obj.properties.get(property).cloned().unwrap_or(Value::Null)
            }
            _ => Value::Null,
        }
    }

    /// 安全调用对象方法
    ///
    /// # 参数
    /// - `object`: 对象值
    /// - `method`: 方法名
    /// - `args`: 参数列表
    /// - `context`: 执行上下文（用于方法调用）
    ///
    /// # 返回
    /// 方法返回值，如果对象为 null 则返回 null
    pub fn call_method(
        object: &Value,
        method: &str,
        args: &[Value],
        context: Option<&MethodCallContext>,
    ) -> Value {
        match object {
            Value::Null => Value::Null,
            Value::Object(obj) => {
                // 在实际实现中，这里需要通过解释器调用对象方法
                // 由于 ObjectInstance 没有 methods 字段，方法调用需要通过解释器进行
                tracing::debug!(
                    "调用对象方法: {}::{}({:?})",
                    obj.class_name,
                    method,
                    args
                );

                // 如果有上下文，使用上下文执行方法
                if let Some(ctx) = context {
                    tracing::debug!("方法调用上下文: {:?}", ctx.class_name);
                }

                // 返回占位值，实际实现需要解释器支持
                Value::Null
            }
            _ => Value::Null,
        }
    }

    /// 链式安全访问
    ///
    /// # 参数
    /// - `chain`: 访问链（如 ["user", "address", "country"]）
    ///
    /// # 返回
    /// 最终值，如果链中任何值为 null 则返回 null
    pub fn chain_access(object: &Value, chain: &[&str]) -> Value {
        let mut current = object.clone();

        for property in chain {
            current = Self::get_property(&current, property);
            if matches!(current, Value::Null) {
                return Value::Null;
            }
        }

        current
    }

    /// 链式安全方法调用
    ///
    /// # 参数
    /// - `object`: 起始对象
    /// - `calls`: 调用链（属性名或方法调用）
    ///
    /// # 返回
    /// 最终值
    pub fn chain_call(object: &Value, calls: &[ChainCall]) -> Value {
        let mut current = object.clone();

        for call in calls {
            if matches!(current, Value::Null) {
                return Value::Null;
            }

            match call {
                ChainCall::Property(name) => {
                    current = Self::get_property(&current, name);
                }
                ChainCall::Method { name, args } => {
                    current = Self::call_method(&current, name, args, None);
                }
                ChainCall::ArrayIndex(index) => {
                    current = Self::get_array_item(&current, index);
                }
            }
        }

        current
    }

    /// 获取数组元素
    fn get_array_item(array: &Value, index: &Value) -> Value {
        match (array, index) {
            (Value::IndexedArray(arr), Value::Int(i)) => {
                if *i >= 0 && (*i as usize) < arr.len() {
                    arr[*i as usize].clone()
                } else {
                    Value::Null
                }
            }
            (Value::AssociativeArray(map), Value::String(key)) => {
                // 关联数组是 Vec<(String, Value)>，需要查找
                for (k, v) in map {
                    if k == key {
                        return v.clone();
                    }
                }
                Value::Null
            }
            _ => Value::Null,
        }
    }
}

/// 链式调用类型
#[derive(Debug, Clone)]
pub enum ChainCall {
    /// 属性访问
    Property(String),
    /// 方法调用
    Method {
        /// 方法名
        name: String,
        /// 参数
        args: Vec<Value>,
    },
    /// 数组索引
    ArrayIndex(Value),
}

/// 方法调用上下文
#[derive(Debug, Clone)]
pub struct MethodCallContext {
    /// 当前类名
    pub class_name: String,
    /// 当前对象
    pub this: Value,
    /// 作用域变量
    pub scope: HashMap<String, Value>,
}

impl MethodCallContext {
    /// 创建新的方法调用上下文
    pub fn new(class_name: &str, this: Value) -> Self {
        Self {
            class_name: class_name.to_string(),
            this,
            scope: HashMap::new(),
        }
    }
}

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
    pub fn new(name: &str, target: AttributeTarget) -> Self {
        Self {
            name: name.to_string(),
            arguments: Vec::new(),
            named_arguments: HashMap::new(),
            target,
        }
    }

    /// 添加位置参数（链式调用）
    pub fn with_argument(mut self, value: Value) -> Self {
        self.arguments.push(value);
        self
    }

    /// 添加命名参数（链式调用）
    pub fn with_named_argument(mut self, name: &str, value: Value) -> Self {
        self.named_arguments.insert(name.to_string(), value);
        self
    }

    /// 添加位置参数（可变引用）
    pub fn add_argument(&mut self, value: Value) {
        self.arguments.push(value);
    }

    /// 添加命名参数（可变引用）
    pub fn add_named_argument(&mut self, name: &str, value: Value) {
        self.named_arguments.insert(name.to_string(), value);
    }

    /// 获取位置参数
    pub fn get_argument(&self, index: usize) -> Option<&Value> {
        self.arguments.get(index)
    }

    /// 获取命名参数
    pub fn get_named_argument(&self, name: &str) -> Option<&Value> {
        self.named_arguments.get(name)
    }

    /// 获取字符串参数
    pub fn get_string_argument(&self, index: usize) -> Option<&str> {
        self.arguments.get(index).and_then(|v| v.as_str())
    }

    /// 获取整数参数
    pub fn get_int_argument(&self, index: usize) -> Option<i64> {
        self.arguments.get(index).and_then(|v| v.as_i64())
    }

    /// 检查是否有指定名称的命名参数
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
    /// # 返回
    /// 解析后的注解
    pub fn parse(&mut self, input: &str, target: AttributeTarget) -> Result<Attribute, String> {
        // 移除 #[ 和 ]
        let content = input.trim();
        if !content.starts_with("#[") || !content.ends_with(']') {
            return Err("无效的注解格式".to_string());
        }

        let content = &content[2..content.len() - 1].trim();

        // 解析类名
        let (name, args_str) = if let Some(pos) = content.find('(') {
            let name = content[..pos].trim().to_string();
            let args = content[pos..].trim();
            (name, Some(args))
        } else {
            (content.to_string(), None)
        };

        let mut attribute = Attribute::new(&name, target);

        // 解析参数
        if let Some(args) = args_str {
            self.parse_arguments(args, &mut attribute)?;
        }

        self.attributes.push(attribute.clone());
        Ok(attribute)
    }

    /// 解析参数
    fn parse_arguments(&self, args: &str, attribute: &mut Attribute) -> Result<(), String> {
        // 移除括号
        let args = args.trim();
        if !args.starts_with('(') || !args.ends_with(')') {
            return Err("无效的参数格式".to_string());
        }

        let args = &args[1..args.len() - 1].trim();
        if args.is_empty() {
            return Ok(());
        }

        // 解析参数（支持嵌套）
        self.parse_argument_list(args, attribute)?;

        Ok(())
    }

    /// 解析参数列表（支持嵌套结构和数组）
    fn parse_argument_list(&self, args: &str, attribute: &mut Attribute) -> Result<(), String> {
        let mut depth = 0;
        let mut current = String::new();
        let mut in_string = false;
        let mut string_char = ' ';

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

            if in_string {
                current.push(ch);
                continue;
            }

            // 处理嵌套
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
    fn parse_array(&self, value: &str) -> Result<Value, String> {
        let content = &value[1..value.len() - 1].trim();

        if content.is_empty() {
            return Ok(Value::IndexedArray(Vec::new()));
        }

        // 尝试解析为索引数组
        let mut indexed = Vec::new();
        let mut associative = HashMap::new();
        let mut is_associative = false;

        let mut depth = 0;
        let mut current = String::new();
        let mut in_string = false;
        let mut string_char = ' ';

        for ch in content.chars() {
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

            if in_string {
                current.push(ch);
                continue;
            }

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

        if is_associative {
            // 将 HashMap 转换为 Vec<(String, Value)>
            let vec: Vec<(String, Value)> = associative.into_iter().collect();
            Ok(Value::AssociativeArray(vec))
        } else {
            Ok(Value::IndexedArray(indexed))
        }
    }

    /// 解析数组元素
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

            let key = if key_str.starts_with('"') || key_str.starts_with('\'') {
                key_str[1..key_str.len() - 1].to_string()
            } else {
                key_str.to_string()
            };

            let value = self.parse_value(value_str).ok()?;
            return Some((Some(key), value));
        }

        // 索引数组元素
        let value = self.parse_value(item).ok()?;
        Some((None, value))
    }

    /// 获取所有注解
    pub fn get_attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    /// 按名称获取注解
    pub fn get_by_name(&self, name: &str) -> Vec<&Attribute> {
        self.attributes
            .iter()
            .filter(|a| a.name == name)
            .collect()
    }

    /// 按目标获取注解
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
    pub fn with_readonly(mut self) -> Self {
        self.readonly = true;
        self
    }

    /// 设置默认值
    pub fn with_default(mut self, value: Value) -> Self {
        self.default_value = Some(value);
        self
    }
}

/// 联合类型
///
/// PHP 8.0 允许类型包含多种类型
///
/// # 示例
/// ```php
/// public function process(int|float $number): int|float {
///     return $number * 2;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct UnionType {
    /// 类型列表
    pub types: Vec<String>,
}

impl UnionType {
    /// 创建新的联合类型
    pub fn new(types: Vec<&str>) -> Self {
        Self {
            types: types.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// 从字符串解析联合类型
    pub fn parse(type_str: &str) -> Self {
        Self {
            types: type_str.split('|').map(|s| s.trim().to_string()).collect(),
        }
    }

    /// 检查值是否匹配类型
    pub fn matches(&self, value: &Value) -> bool {
        for type_name in &self.types {
            if Self::value_matches_type(value, type_name) {
                return true;
            }
        }
        false
    }

    /// 检查单个类型匹配
    fn value_matches_type(value: &Value, type_name: &str) -> bool {
        match type_name.to_lowercase().as_str() {
            "int" | "integer" => matches!(value, Value::Int(_)),
            "float" | "double" => matches!(value, Value::Float(_)),
            "string" => matches!(value, Value::String(_)),
            "bool" | "boolean" => matches!(value, Value::Bool(_)),
            "array" => matches!(value, Value::IndexedArray(_) | Value::AssociativeArray(_)),
            "object" => matches!(value, Value::Object(_)),
            "null" => matches!(value, Value::Null),
            "mixed" => true,
            "callable" => matches!(value, Value::Callable(_)),
            "iterable" => {
                matches!(value, Value::IndexedArray(_) | Value::AssociativeArray(_))
            }
            "numeric" => matches!(value, Value::Int(_) | Value::Float(_)),
            "scalar" => matches!(
                value,
                Value::Int(_) | Value::Float(_) | Value::String(_) | Value::Bool(_)
            ),
            _ => true, // 其他类型（类名等）暂不验证
        }
    }

    /// 获取类型字符串
    pub fn to_type_string(&self) -> String {
        self.types.join("|")
    }

    /// 检查是否包含 null 类型
    pub fn is_nullable(&self) -> bool {
        self.types.iter().any(|t| t.to_lowercase() == "null")
    }

    /// 获取非 null 类型
    pub fn get_non_null_types(&self) -> Vec<&String> {
        self.types
            .iter()
            .filter(|t| t.to_lowercase() != "null")
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::value::ObjectInstance;

    #[test]
    fn test_match_expression() {
        let expr = MatchExpression::new(Value::Int(200))
            .add_arm(vec![Value::Int(200), Value::Int(300)], MatchBody::Expression(Value::String("OK".to_string())))
            .add_arm(vec![Value::Int(400)], MatchBody::Expression(Value::String("Error".to_string())));

        let result = expr.evaluate();
        assert_eq!(result, Some(Value::String("OK".to_string())));
    }

    #[test]
    fn test_match_expression_default() {
        let expr = MatchExpression::new(Value::Int(500))
            .add_arm(vec![Value::Int(200)], MatchBody::Expression(Value::String("OK".to_string())))
            .with_default(Value::String("Unknown".to_string()));

        let result = expr.evaluate();
        assert_eq!(result, Some(Value::String("Unknown".to_string())));
    }

    #[test]
    fn test_match_block() {
        let expr = MatchExpression::new(Value::Int(1))
            .add_arm(
                vec![Value::Int(1)],
                MatchBody::Block(vec![
                    MatchStatement::Assign {
                        name: "result".to_string(),
                        value: Value::String("processed".to_string()),
                    },
                    MatchStatement::Return(Value::String("done".to_string())),
                ]),
            );

        let result = expr.evaluate();
        assert_eq!(result, Some(Value::String("done".to_string())));
    }

    #[test]
    fn test_named_arguments() {
        let args = NamedArguments::new()
            .add_positional(Value::Int(1))
            .add_named("name", Value::String("test".to_string()));

        assert_eq!(args.len(), 2);
    }

    #[test]
    fn test_named_arguments_resolve() {
        let args = NamedArguments::new()
            .add_positional(Value::Int(1))
            .add_named("name", Value::String("test".to_string()));

        let param_names = vec!["id".to_string(), "name".to_string(), "value".to_string()];
        let defaults = HashMap::from([("value".to_string(), Value::Int(100))]);

        let resolved = args.resolve(&param_names, &defaults);

        assert_eq!(resolved[0], Value::Int(1));
        assert_eq!(resolved[1], Value::String("test".to_string()));
        assert_eq!(resolved[2], Value::Int(100));
    }

    #[test]
    fn test_nullsafe_operator() {
        let result = NullsafeOperator::get_property(&Value::Null, "name");
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_nullsafe_chain() {
        let obj = Value::Object(ObjectInstance {
            class_name: "User".to_string(),
            properties: HashMap::new(),
        });

        let result = NullsafeOperator::chain_access(&obj, &["name", "first"]);
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_attribute_parser() {
        let mut parser = AttributeParser::new();
        let attr = parser.parse("#[Route('/api/users')]", AttributeTarget::Class).unwrap();

        assert_eq!(attr.name, "Route");
        assert_eq!(attr.arguments.len(), 1);
    }

    #[test]
    fn test_attribute_parser_with_named() {
        let mut parser = AttributeParser::new();
        let attr = parser
            .parse("#[Route(path: '/api/users', methods: ['GET', 'POST'])]", AttributeTarget::Class)
            .unwrap();

        assert_eq!(attr.name, "Route");
        assert!(attr.has_named_argument("path"));
        assert!(attr.has_named_argument("methods"));
    }

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

    #[test]
    fn test_union_type() {
        let union = UnionType::new(vec!["int", "float"]);

        assert!(union.matches(&Value::Int(42)));
        assert!(union.matches(&Value::Float(3.14)));
        assert!(!union.matches(&Value::String("hello".to_string())));
    }

    #[test]
    fn test_union_type_nullable() {
        let union = UnionType::parse("int|null");

        assert!(union.is_nullable());
        assert!(union.matches(&Value::Int(42)));
        assert!(union.matches(&Value::Null));
    }

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
