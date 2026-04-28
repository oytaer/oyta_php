# 环境变量加载模块 (Env Loader)

## 模块概述

环境变量加载模块提供 .env 文件加载和解析功能。

## 模块结构

```
env_loader/
├── mod.rs          # 模块入口
└── loader.rs       # 环境变量加载器
```

## 功能特性

| 功能 | 说明 |
|------|------|
| .env 加载 | 加载 .env 文件 |
| 变量解析 | 解析环境变量 |
| 类型转换 | 变量类型转换 |

## .env 文件格式

```ini
# 应用配置
APP_NAME=OYTAPHP
APP_DEBUG=true
APP_ENV=production

# 数据库配置
DB_HOST=127.0.0.1
DB_PORT=3306
DB_NAME=test
DB_USER=root
DB_PASS=
```

## 技术实现

- .env 文件解析
- 环境变量加载
- 类型转换
