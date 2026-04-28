# eval 动态执行模块 (Eval)

## 模块概述

eval 动态执行模块实现 PHP 的 eval() 动态代码执行功能，包括代码解析和执行、执行上下文管理、沙箱安全机制等。

## 模块结构

```
eval/
├── mod.rs          # 模块入口
├── evaluator.rs    # 执行器
├── context.rs      # 执行上下文
└── sandbox.rs      # 沙箱安全
```

## 主要类型

| 类型 | 说明 |
|------|------|
| Eval | 执行器 |
| EvalContext | 执行上下文 |
| EvalSandbox | 沙箱环境 |
| EvalResult | 执行结果 |

## 使用示例

### 执行代码

```php
<?php
$result = eval('return 1 + 1;');
echo $result;  // 输出: 2
```

### 执行上下文

```php
<?php
$x = 10;
$result = eval('return $x * 2;');
echo $result;  // 输出: 20
```

## 安全机制

- 变量作用域隔离
- 执行时间限制
- 内存限制
- 危险函数限制

## 技术实现

- 动态代码解析
- 执行上下文管理
- 沙箱安全机制
- 变量作用域处理
