# 解析器模块 (Parser)

## 模块概述

解析器模块提供 PHP 源码解析、配置文件解析、文件扫描等功能，基于 php-rs-parser 和 php-ast 库实现。

## 模块结构

```
parser/
├── mod.rs          # 模块入口
├── php_parser.rs   # PHP 解析器
├── config_parser.rs # 配置解析器
└── scanner.rs      # 文件扫描器
```

## 主要功能

| 功能 | 说明 |
|------|------|
| PHP 源码解析 | 解析 PHP 文件生成 AST |
| 配置文件解析 | 解析 PHP 配置文件 |
| 文件扫描 | 扫描项目目录结构 |

## 支持的 PHP 语法

- 类和接口定义
- 函数和方法定义
- 变量和常量
- 控制结构（if/else/for/foreach/while/switch）
- 命名空间
- Trait
- 匿名函数
- 箭头函数
- 属性（Attribute）
- 枚举（Enum）

## 技术实现

- 基于 php-rs-parser 库
- 生成 AST 抽象语法树
- 支持增量解析
- 错误恢复机制
