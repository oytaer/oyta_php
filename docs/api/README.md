# OYTAPHP 模块文档

## 概述

OYTAPHP 是一个用 Rust 编写的高性能 PHP 解释器，兼容 ThinkPHP 8.0。本文档详细介绍了 OYTAPHP 的 47 个核心模块及其使用方法。

## 模块分类

### 核心运行时模块

| 模块 | 说明 | 文档 |
|------|------|------|
| interpreter | PHP 解释器引擎 | [interpreter.md](./interpreter.md) |
| parser | PHP 源码解析器 | [parser.md](./parser.md) |
| symbol_table | 符号表管理 | 内部模块 |
| value | 值类型系统 | 内部模块 |
| eval | 动态代码执行 | 内部模块 |

### Web 服务模块

| 模块 | 说明 | 文档 |
|------|------|------|
| http | HTTP 服务 | [http.md](./http.md) |
| http3 | HTTP/3 和 QUIC 支持 | 内部模块 |
| fastcgi | FastCGI 协议支持 | 内部模块 |
| router | 路由系统 | [router.md](./router.md) |
| middleware | 中间件系统 | [middleware.md](./middleware.md) |

### 数据处理模块

| 模块 | 说明 | 文档 |
|------|------|------|
| database | 数据库操作 | [database.md](./database.md) |
| cache | 缓存系统 | [cache.md](./cache.md) |
| session | 会话管理 | [session.md](./session.md) |
| storage | 文件存储 | [storage.md](./storage.md) |
| serialize | 序列化引擎 | [extensions.md](./extensions.md) |

### 安全模块

| 模块 | 说明 | 文档 |
|------|------|------|
| security | 安全功能 | [security.md](./security.md) |
| validator | 数据验证 | [validator.md](./validator.md) |
| sandbox | 沙箱隔离 | 内部模块 |

### 服务模块

| 模块 | 说明 | 文档 |
|------|------|------|
| service | 服务管理 | [other.md](./other.md) |
| container | 依赖注入容器 | [container.md](./container.md) |
| config | 配置管理 | [config.md](./config.md) |
| event | 事件系统 | [event.md](./event.md) |

### 异步处理模块

| 模块 | 说明 | 文档 |
|------|------|------|
| coroutine | 协程调度 | [coroutine.md](./coroutine.md) |
| timer | 定时器 | [timer.md](./timer.md) |
| embedded | 内嵌服务（队列） | 内部模块 |

### 扩展功能模块

| 模块 | 说明 | 文档 |
|------|------|------|
| gd | GD 图像处理 | [extensions.md](./extensions.md) |
| xml | XML 处理 | [extensions.md](./extensions.md) |
| mbstring | 多字节字符串 | [extensions.md](./extensions.md) |
| datetime | 日期时间 | [extensions.md](./extensions.md) |
| phar | Phar 归档 | [extensions.md](./extensions.md) |

### 开发工具模块

| 模块 | 说明 | 文档 |
|------|------|------|
| command | 命令行工具 | [command.md](./command.md) |
| composer | Composer 依赖管理 | [other.md](./other.md) |
| watcher | 文件监听与热重载 | [other.md](./other.md) |
| debug | 调试工具 | [other.md](./other.md) |
| monitor | 性能监控 | [other.md](./other.md) |
| reflection | 反射系统 | [other.md](./other.md) |

### 分布式模块

| 模块 | 说明 | 文档 |
|------|------|------|
| cluster | 分布式支持 | [cluster.md](./cluster.md) |
| microservice | 微服务框架 | 内部模块 |

### 其他模块

| 模块 | 说明 | 文档 |
|------|------|------|
| template | 模板引擎 | [template.md](./template.md) |
| i18n | 国际化 | [i18n.md](./i18n.md) |
| logging | 日志系统 | [logging.md](./logging.md) |
| error | 错误处理 | [other.md](./other.md) |
| helpers | 辅助函数 | [other.md](./other.md) |
| env_loader | 环境变量加载 | [other.md](./other.md) |
| output | 输出缓冲控制 | [other.md](./other.md) |
| search | 全文搜索 | [other.md](./other.md) |
| simd | SIMD 加速 | 内部模块 |
| embedded | 内嵌服务 | 内部模块 |
| project | 项目检测 | 内部模块 |

## 门面类一览

OYTAPHP 提供以下门面类（Facade），用于静态方法访问底层功能：

| 门面类 | 模块 | 说明 |
|--------|------|------|
| Cache | cache | 缓存操作 |
| Session | session | 会话管理 |
| Config | config | 配置管理 |
| Env | config | 环境变量 |
| Log | logging | 日志记录 |
| Event | event | 事件系统 |
| Request | http | 请求处理 |
| Response | http | 响应处理 |
| Route | router | 路由管理 |
| Db | database | 数据库操作 |
| Model | database | 模型操作 |
| View | template | 视图渲染 |
| Container | container | 依赖注入容器 |
| Lang | i18n | 多语言 |
| Crypt | security | 加密解密 |
| Hash | security | 哈希处理 |
| Csrf | security | CSRF 防护 |
| Validate | validator | 数据验证 |
| Middleware | middleware | 中间件 |
| Storage | storage | 文件存储 |
| Date | datetime | 日期处理 |
| Queue | embedded | 消息队列 |
| Timer | timer | 定时器 |
| Coroutine | coroutine | 协程管理 |
| Lock | cluster | 分布式锁 |
| ServiceDiscovery | cluster | 服务发现 |
| Composer | composer | 包管理 |
| Cron | embedded | 定时任务 |
| Monitor | monitor | 性能监控 |
| Serialize | serialize | 序列化 |

## 快速开始

### 安装

```bash
# 编译项目
cargo build --release

# 创建新项目
oyta new myproject

# 进入项目目录
cd myproject

# 启动开发服务器
oyta run
```

### 基本使用

```php
<?php
namespace app\controller;

class Index
{
    public function index()
    {
        return 'Hello, OYTAPHP!';
    }
    
    public function json()
    {
        return json([
            'code' => 200,
            'msg' => 'success',
            'data' => []
        ]);
    }
}
```

## 性能特性

- **原生 Rust 实现**：相比 PHP 解释器性能提升 5-10 倍
- **异步 I/O**：基于 Tokio 的高性能异步运行时
- **零拷贝序列化**：Bincode/MessagePack 等高性能序列化
- **SIMD 加速**：字符串操作、JSON 解析等场景自动使用 SIMD
- **协程支持**：原生协程调度，支持百万级并发

## 项目结构

```
myproject/
├── app/                    # 应用目录
│   ├── controller/         # 控制器
│   ├── model/              # 模型
│   ├── middleware/         # 中间件
│   ├── validate/           # 验证器
│   └── service/            # 服务类
├── config/                 # 配置文件
├── public/                 # 公共目录
│   └── index.php           # 入口文件
├── runtime/                # 运行时目录
│   ├── cache/              # 缓存
│   ├── log/                # 日志
│   └── session/            # 会话
├── route/                  # 路由定义
├── view/                   # 视图模板
├── .env                    # 环境变量
└── composer.json           # Composer 配置
```

## 命令行工具

```bash
# 创建新项目
oyta new myproject

# 启动开发服务器
oyta run

# 创建控制器
oyta make:controller UserController

# 创建模型
oyta make:model User

# 创建多应用
oyta build admin

# 清除缓存
oyta clear
```

## 版本信息

- **当前版本**: 0.1.0
- **PHP 兼容**: PHP 8.5
- **ThinkPHP 兼容**: ThinkPHP 8.0
- **Rust 版本**: 1.75+
