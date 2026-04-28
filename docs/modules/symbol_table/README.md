# 符号表模块 (Symbol Table)

## 模块概述

符号表模块存储和管理 PHP 文件解析后的所有符号信息，包括类、函数、常量、命名空间等定义。

## 模块结构

```
symbol_table/
├── mod.rs          # 模块入口
├── registry.rs     # 符号注册表
└── types.rs        # 符号类型
```

## 存储的符号

| 符号类型 | 说明 |
|----------|------|
| Class | 类定义 |
| Interface | 接口定义 |
| Trait | Trait 定义 |
| Function | 函数定义 |
| Constant | 常量定义 |
| Namespace | 命名空间 |

## 使用示例

### 查找类

```php
<?php
// 自动加载时查找类定义
$class = SymbolTable::findClass('App\\Controller\\UserController');
```

### 查找函数

```php
<?php
// 查找函数定义
$function = SymbolTable::findFunction('App\\Helpers\\formatDate');
```

## 技术实现

- 符号注册表
- 命名空间管理
- 自动加载支持
- 增量更新
