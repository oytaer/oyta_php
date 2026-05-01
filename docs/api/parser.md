# Parser 解析器模块

## 模块结构

```
parser/
├── mod.rs           # 模块入口
├── config_parser.rs # 配置解析器
├── scanner.rs       # 词法扫描器
└── php_parser/      # PHP 解析器
    ├── mod.rs
    ├── parser.rs
    ├── expr.rs
    ├── collector.rs
    ├── extractors.rs
    └── config_parser.rs
```

## 解析流程

1. **词法分析**: PHP 源码 → Token 流
2. **语法分析**: Token 流 → AST
3. **语义分析**: 收集符号信息
4. **代码生成**: 生成可执行代码

## 支持的 PHP 语法

- 类、接口、Trait 定义
- 函数、方法定义
- 变量、常量定义
- 控制结构（if/else/for/foreach/while/switch）
- 表达式（算术、逻辑、比较、位运算）
- 闭包、箭头函数
- 匿名类
- PHP 8 新特性

## AST 节点类型

```rust
// 语句类型
enum Stmt {
    Class(ClassStmt),
    Function(FunctionStmt),
    If(IfStmt),
    For(ForStmt),
    While(WhileStmt),
    Return(ReturnStmt),
    Expression(ExpressionStmt),
    // ...
}

// 表达式类型
enum Expr {
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Call(CallExpr),
    Property(PropertyExpr),
    Array(ArrayExpr),
    // ...
}
```
