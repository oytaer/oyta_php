# 解释器模块

## 模块概述

解释器模块是 OYTAPHP 的核心，负责 PHP 代码的解析与执行。该模块实现了完整的 PHP 8.x 语法支持，包括 Generator、Fiber、枚举类型、只读属性等新特性。

## 文件结构

```
interpreter/
├── mod.rs              # 模块入口
├── engine.rs           # 解释器引擎
├── value.rs            # PHP 值类型定义
├── generator.rs        # Generator/yield 支持
├── fiber.rs            # Fiber 协程支持
├── enum_type.rs        # 枚举类型支持
├── readonly.rs         # 只读属性支持
└── php8_features/      # PHP 8.x 新特性
    ├── mod.rs
    ├── match_expr.rs   # match 表达式
    ├── named_args.rs   # 命名参数
    ├── nullsafe.rs     # Nullsafe 运算符
    ├── union_type.rs   # 联合类型
    └── attribute.rs    # Attribute 注解
```

## 核心组件

### Value 类型

PHP 值类型定义，支持所有 PHP 数据类型：

```rust
pub enum Value {
    /// null 值
    Null,
    /// 布尔值
    Bool(bool),
    /// 整数值
    Int(i64),
    /// 浮点数值
    Float(f64),
    /// 字符串值
    String(String),
    /// 索引数组
    IndexedArray(Vec<Value>),
    /// 关联数组
    AssociativeArray(Vec<(String, Value)>),
    /// 对象实例
    Object(ObjectInstance),
    /// 可调用对象
    Callable(Callable),
    /// 资源
    Resource(String),
    /// Generator
    Generator(GeneratorValue),
    /// Fiber
    Fiber(FiberValue),
}
```

### Generator 支持

实现 PHP 的 Generator/yield 功能：

```php
function getUsers(): Generator {
    foreach (User::chunk(100) as $users) {
        foreach ($users as $user) {
            yield $user;
        }
    }
}
```

**Generator 状态**：
- `Created`：已创建但未启动
- `Running`：正在运行
- `Suspended`：已挂起（yield 后）
- `Closed`：已关闭

**Generator 方法**：
| 方法 | 说明 |
|------|------|
| `current()` | 获取当前值 |
| `key()` | 获取当前键 |
| `next()` | 前进到下一个值 |
| `rewind()` | 重置生成器 |
| `valid()` | 检查是否有效 |
| `send($value)` | 发送值到生成器 |
| `throw($exception)` | 抛出异常 |
| `getReturn()` | 获取返回值 |

### Fiber 协程

实现 PHP 8.1+ 的 Fiber 协程功能：

```php
$fiber = new Fiber(function() {
    $value = Fiber::suspend('suspended');
    return "received: " . $value;
});

$fiber->start();  // 返回 'suspended'
$fiber->resume('resumed');  // 返回 'received: resumed'
```

**Fiber 状态**：
- `Created`：已创建但未启动
- `Started`：已启动
- `Suspended`：已挂起
- `Terminated`：已终止

**Fiber 方法**：
| 方法 | 说明 |
|------|------|
| `start()` | 启动 Fiber |
| `resume($value)` | 恢复执行 |
| `throw($exception)` | 抛出异常 |
| `getReturn()` | 获取返回值 |
| `isStarted()` | 是否已启动 |
| `isSuspended()` | 是否已挂起 |
| `isTerminated()` | 是否已终止 |

### 枚举类型

实现 PHP 8.1+ 的枚举功能：

```php
// 纯枚举
enum Status {
    case Active;
    case Inactive;
    case Pending;
}

// 回退枚举
enum UserRole: string {
    case Admin = 'admin';
    case User = 'user';
    case Guest = 'guest';
}
```

**枚举特性**：
- 纯枚举（Pure Enum）
- 回退枚举（Backed Enum）
- 枚举方法
- 枚举常量
- 枚举接口实现

### 只读属性

实现 PHP 8.1+ 的只读属性功能：

```php
class User {
    public readonly int $id;
    public readonly string $name;
    
    public function __construct(int $id, string $name) {
        $this->id = $id;
        $this->name = $name;
    }
}

$user = new User(1, 'think');
$user->id = 2;  // Error: Cannot modify readonly property
```

**只读属性特性**：
- 只能在声明时或构造函数中初始化
- 初始化后不能再修改
- 支持类型提示
- 支持构造函数属性提升

### PHP 8.x 新特性

#### match 表达式

```php
$status = match($code) {
    200 => 'OK',
    404 => 'Not Found',
    500 => 'Server Error',
    default => 'Unknown',
};
```

#### 命名参数

```php
function createUser(string $name, int $age, string $email) {}

createUser(
    email: 'user@example.com',
    name: 'think',
    age: 25
);
```

#### Nullsafe 运算符

```php
$name = $user?->profile?->name;
```

#### 联合类型

```php
function process(int|string $value): int|float {
    return is_int($value) ? $value * 2 : strlen($value);
}
```

#### Attribute 注解

```php
#[Route('/hello/:name', method: 'GET')]
public function hello(string $name) {
    return 'Hello, ' . $name;
}
```

## 符号表

使用 `dashmap` 实现并发安全的符号表：

```rust
struct SymbolTable {
    // 类定义：namespace\ClassName → ClassDef
    classes: DashMap<String, ClassDef>,
    // 函数定义：function_name → FuncDef
    functions: DashMap<String, FuncDef>,
    // 常量定义：CONST_NAME → Value
    constants: DashMap<String, Value>,
    // 路由定义：HTTP_METHOD:path → RouteDef
    routes: DashMap<String, RouteDef>,
    // 中间件定义：middleware_name → MiddlewareDef
    middlewares: DashMap<String, MiddlewareDef>,
    // 事件定义：event_name → Vec<EventListener>
    events: DashMap<String, Vec<EventListener>>,
}
```

## 支持的 PHP 语法

### 基础语法

| 特性 | 状态 |
|------|------|
| 变量与赋值 | ✅ |
| 数据类型 | ✅ |
| 运算符 | ✅ |
| 字符串操作 | ✅ |
| 数组 | ✅ |
| 控制结构 | ✅ |
| 函数定义与调用 | ✅ |
| 闭包/Lambda | ✅ |
| 命名空间 | ✅ |
| use 导入 | ✅ |

### 面向对象

| 特性 | 状态 |
|------|------|
| 类定义 | ✅ |
| 构造函数 | ✅ |
| 方法定义与调用 | ✅ |
| 静态方法 | ✅ |
| 属性 | ✅ |
| 继承 | ✅ |
| 接口 | ✅ |
| 抽象类 | ✅ |
| Trait | ✅ |
| 访问修饰符 | ✅ |
| $this 引用 | ✅ |
| static/self/parent | ✅ |
| 魔术方法 | ✅ |
| 常量 | ✅ |
| 类型提示 | ✅ |

### PHP 8.x 特性

| 特性 | 状态 |
|------|------|
| 命名参数 | ✅ |
| 联合类型 | ✅ |
| match 表达式 | ✅ |
| Nullsafe 运算符 | ✅ |
| Attribute 注解 | ✅ |
| 构造器属性提升 | ✅ |
| readonly 属性 | ✅ |
| 枚举类型 | ✅ |
| Fiber 协程 | ✅ |
| Generator/yield | ✅ |
