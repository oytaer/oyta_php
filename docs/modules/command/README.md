# 命令行模块 (Command)

## 模块概述

命令行模块提供 OYTAPHP 的 CLI 命令功能，支持服务器启动、项目初始化、代码生成、缓存管理、队列管理、Composer 操作等命令。

## 模块结构

```
command/
├── mod.rs          # 模块入口
├── args.rs         # 命令参数定义
├── server.rs       # 服务器命令
├── init.rs         # 初始化命令
├── make.rs         # 代码生成命令
├── cache.rs        # 缓存命令
├── config.rs       # 配置命令
├── route.rs        # 路由命令
├── queue.rs        # 队列命令
├── service.rs      # 服务命令
├── optimize.rs     # 优化命令
├── composer.rs     # Composer 命令
└── info.rs         # 信息命令
```

## 命令列表

### 服务器命令

| 命令 | 说明 |
|------|------|
| `oyta run` | 启动 HTTP 服务器 |
| `oyta fastcgi` | 启动 FastCGI 服务器 |

### 项目初始化命令

| 命令 | 说明 |
|------|------|
| `oyta new <name>` | 创建新项目 |
| `oyta init` | 初始化当前目录 |
| `oyta install` | 安装 oyta 到系统 PATH |
| `oyta uninstall` | 卸载系统 PATH 中的 oyta |

### 代码生成命令

| 命令 | 说明 |
|------|------|
| `oyta build <name>` | 构建应用 |
| `oyta make:controller <name>` | 创建控制器 |
| `oyta make:model <name>` | 创建模型 |
| `oyta make:middleware <name>` | 创建中间件 |
| `oyta make:validate <name>` | 创建验证器 |
| `oyta make:event <name>` | 创建事件 |
| `oyta make:listener <name>` | 创建监听器 |
| `oyta make:service <name>` | 创建服务类 |
| `oyta make:command <name>` | 创建自定义命令 |
| `oyta make:job <name>` | 创建队列任务 |
| `oyta make:task <name>` | 创建定时任务 |

### 路由命令

| 命令 | 说明 |
|------|------|
| `oyta route:list` | 查看路由列表 |
| `oyta route:cache` | 缓存路由 |
| `oyta route:clear` | 清除路由缓存 |

### 缓存命令

| 命令 | 说明 |
|------|------|
| `oyta cache:clear` | 清除缓存 |
| `oyta cache:forget <key>` | 删除指定缓存 |
| `oyta cache:status` | 查看缓存状态 |

### 队列命令

| 命令 | 说明 |
|------|------|
| `oyta queue:failed` | 查看失败任务 |
| `oyta queue:retry <id>` | 重试指定任务 |
| `oyta queue:flush` | 清除所有失败任务 |
| `oyta queue:status` | 查看队列状态 |

### 配置命令

| 命令 | 说明 |
|------|------|
| `oyta config:cache` | 缓存配置 |
| `oyta config:clear` | 清除配置缓存 |
| `oyta config:get <key>` | 查看配置值 |
| `oyta config:set <key> <value>` | 设置配置值 |

### 优化命令

| 命令 | 说明 |
|------|------|
| `oyta optimize` | 优化应用 |
| `oyta optimize:route` | 生成路由缓存 |
| `oyta optimize:schema` | 生成数据表字段缓存 |
| `oyta clear` | 清除运行时缓存 |

### Composer 命令

| 命令 | 说明 |
|------|------|
| `oyta composer:install` | 安装依赖 |
| `oyta composer:update` | 更新依赖 |
| `oyta composer:require <package>` | 添加依赖 |
| `oyta composer:remove <package>` | 移除依赖 |
| `oyta composer:dump-autoload` | 重新生成自动加载 |
| `oyta composer:show` | 查看已安装的包 |
| `oyta composer:outdated` | 查看过时的包 |
| `oyta composer:validate` | 验证 composer.json |
| `oyta composer:config` | 查看 Composer 配置 |
| `oyta composer:clear-cache` | 清除 Composer 缓存 |
| `oyta composer:diagnose` | 诊断 Composer 问题 |

### 信息命令

| 命令 | 说明 |
|------|------|
| `oyta version` | 查看版本 |
| `oyta list` | 列出所有命令 |

## 使用示例

```bash
# 启动服务器
oyta run -p 8080

# 创建新项目
oyta new myapp

# 创建控制器
oyta make:controller Index

# 清除缓存
oyta cache:clear

# 查看路由列表
oyta route:list
```

## 技术实现

- 基于 clap 库实现命令行解析
- 支持命令别名
- 支持命令参数
- 支持命令分组
