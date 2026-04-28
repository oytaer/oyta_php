# 解释器模块 (Interpreter)

## 模块概述

解释器模块提供 PHP 代码的解释执行能力，基于 php-ast 的 AST 节点进行遍历执行，支持变量操作、控制流、函数调用、类方法调用等。

## 模块结构

```
interpreter/
├── mod.rs          # 模块入口
├── engine/         # 解释器引擎
│   ├── mod.rs
│   ├── builtins.rs # 内置函数
│   └── builtin_facades.rs # PHP 门面
├── generator.rs    # Generator 支持
├── fiber.rs        # Fiber 协程
├── enum_type.rs    # 枚举类型
├── readonly.rs     # 只读属性
├── php8_features/  # PHP 8.0+ 特性
└── value.rs        # 值类型
```

## PHP 8.0+ 特性支持

| 特性 | 说明 |
|------|------|
| match 表达式 | 模式匹配 |
| 命名参数 | 参数按名传递 |
| Nullsafe 运算符 | ?-> 安全调用 |
| Attribute 注解 | 声明式元数据 |
| 构造器属性提升 | 简化属性声明 |
| 联合类型 | 多类型声明 |

## PHP 8.1+ 特性支持

| 特性 | 说明 |
|------|------|
| Generator / yield | 生成器 |
| Fiber 协程 | 原生协程 |
| 枚举类型 | Enum |
| 只读属性 | readonly |

## 值类型

| 类型 | 说明 |
|------|------|
| Null | 空值 |
| Bool | 布尔值 |
| Int | 整数 |
| Float | 浮点数 |
| String | 字符串 |
| IndexedArray | 索引数组 |
| AssociativeArray | 关联数组 |
| Object | 对象 |
| Callable | 可调用 |
| Resource | 资源 |

## 技术实现

- 基于 AST 遍历执行
- 支持变量作用域
- 支持闭包和匿名函数
- 支持异常处理
