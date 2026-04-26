# OYTAPHP 完整设计方案

**文档版本**：4.0  
**日期**：2026年4月26日  
**项目名称**：OYTAPHP（oyta_cli + oyta_core）  
**核心定位**：用 Rust 重写的 ThinkPHP 8.0 运行时——同等能力，更简更高效  
**核心目标**：用户项目目录结构 **100% 与 ThinkPHP 8.0 一致**，零 Rust 痕迹，零 PHP 运行环境依赖，改完即生效，一条命令启动。  
**命令前缀**：所有命令统一使用 `oyta` 前缀（如 `oyta run`、`oyta build`、`oyta make:controller`），与 ThinkPHP 8.0 的 `php think` 命令体系一一对应。

---

## 1. 项目愿景与设计哲学

### 1.1 OYTAPHP 是什么

OYTAPHP 是一个 **Rust 驱动的 PHP 框架运行时**。用户完全按照 ThinkPHP 8.0 的写法编写 PHP 代码，但底层由 Rust 二进制解析执行。无需安装 PHP、Nginx、Composer，一条命令即可启动完整的 Web 服务。

### 1.2 设计哲学

| 原则 | 说明 |
|------|------|
| **同等能力** | ThinkPHP 8.0 支持的功能，OYTAPHP 全部支持 |
| **更简** | 零环境依赖、零配置安装、开箱即用 |
| **更高效** | Rust 原生性能，比 PHP-FPM 响应快 10 倍+，内存低 5 倍+ |
| **兼容优先** | 用户代码无需任何修改即可运行 |
| **大道至简** | 约定优于配置，最小化用户心智负担 |

### 1.3 与 ThinkPHP 8.0 的关系

| 维度 | ThinkPHP 8.0 | OYTAPHP |
|------|-------------|---------|
| 运行时 | PHP 8.0+ | Rust 二进制 |
| Web 服务器 | 需要 Nginx/Apache | 内置 Axum |
| 依赖管理 | Composer | 无需（二进制自包含） |
| 热重载 | 需要额外工具 | 内置 notify |
| 性能 | 基准 | 10x+ 响应速度 |
| 用户代码 | PHP | **完全相同的 PHP 代码** |
| 目录结构 | ThinkPHP 标准 | **更简洁**（框架内置替代手动维护） |

---

## 2. 项目总体架构

### 2.1 Cargo Workspace 结构

```
oyta_php/                          # 开发仓库
├── Cargo.toml                     # Workspace 根配置
├── oyta_core/                     # 核心运行时 crate
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                # 二进制入口
│       ├── cli/                   # CLI 命令模块
│       ├── parser/                # PHP 解析器模块
│       ├── interpreter/           # PHP 解释器模块
│       ├── runtime/               # 运行时核心（Facade/容器/事件等）
│       ├── server/                # Web 服务模块
│       ├── router/                # 路由模块
│       ├── middleware/             # 中间件模块
│       ├── database/              # 数据库模块
│       ├── cache/                 # 缓存模块
│       ├── session/               # Session 模块
│       ├── template/              # 模板引擎模块
│       ├── filesystem/            # 文件系统模块
│       ├── validate/              # 验证器模块
│       ├── event/                 # 事件模块
│       ├── console/               # 控制台/日志模块
│       ├── i18n/                  # 多语言模块
│       ├── security/              # 安全模块
│       └── hot_reload/            # 热重载模块
└── oyta_cli/                      # 全局 CLI 工具 crate
    ├── Cargo.toml
    └── src/
        ├── main.rs                # CLI 入口
        ├── commands/              # 命令实现
        └── template/              # 项目模板
```

### 2.2 双 Crate 职责

| Crate | 角色 | 编译产物 | 分发方式 |
|-------|------|----------|----------|
| `oyta_core` | 核心运行时 | 单个二进制 `oyta` | 存放于用户项目 `vendor/oyta_php/bin/oyta` |
| `oyta_cli` | 全局 CLI 工具 | 二进制 `oyta-cli` | `cargo install oyta-cli` |

### 2.3 用户项目目录结构（与 ThinkPHP 8.0 完全一致）

#### 单应用模式

```
myapp/                             # 用户项目根目录
├── app/                           # 应用目录
│   ├── controller/                # 控制器目录
│   │   └── Index.php              # IndexController
│   ├── model/                     # 模型目录
│   │   └── User.php               # User Model
│   ├── middleware/                 # 中间件目录
│   │   └── Auth.php               # Auth Middleware
│   ├── validate/                  # 验证器目录
│   ├── event/                     # 事件监听目录
│   └── common.php                 # 公共函数文件（可选）
├── config/                        # 配置目录
│   ├── app.php                    # 应用配置
│   ├── cache.php                  # 缓存配置
│   ├── console.php                # 控制台配置
│   ├── cookie.php                 # Cookie 配置
│   ├── database.php               # 数据库配置
│   ├── event.php                  # 事件定义
│   ├── exception.php              # 异常处理配置
│   ├── filesystem.php             # 文件磁盘配置
│   ├── lang.php                   # 多语言配置
│   ├── log.php                    # 日志配置
│   ├── middleware.php             # 中间件定义
│   ├── provider.php               # 服务提供定义
│   ├── route.php                  # URL 和路由配置
│   ├── session.php                # Session 配置
│   ├── trace.php                  # Trace 配置
│   └── view.php                   # 视图配置
├── route/                         # 路由定义目录
│   ├── route.php                  # 路由定义文件
│   └── api.php                    # API 路由定义
├── public/                        # WEB 目录（对外访问）
│   ├── index.php                  # 入口文件（兼容标识）
│   ├── router.php                 # 快速测试文件
│   ├── static/                    # 静态资源
│   └── uploads/                   # 上传文件目录
├── view/                          # 视图目录
│   └── index/                     # 按控制器分组
│       └── index.html             # 模板文件
├── extend/                        # 扩展类库目录
├── runtime/                       # 运行时目录（可写）
│   ├── cache/                     # 缓存文件
│   ├── log/                       # 日志文件
│   └── session/                   # Session 文件
├── vendor/                        # 框架核心目录
│   └── oyta_php/
│       └── bin/
│           └── oyta              # ← OYTAPHP 核心 Rust 二进制
├── .env                           # 环境变量
├── .example.env                   # 环境变量示例
├── composer.json                  # Composer 定义文件（兼容标识）
└── oyta                          # 软链接 → vendor/oyta_php/bin/oyta
```

#### 多应用模式

```
myapp/
├── app/
│   ├── index/                     # index 应用
│   │   ├── controller/
│   │   ├── model/
│   │   ├── middleware/
│   │   └── common.php             # 应用公共函数（可选）
│   ├── admin/                     # admin 应用
│   │   ├── controller/
│   │   ├── model/
│   │   ├── middleware/
│   │   └── common.php             # 应用公共函数（可选）
│   └── common.php                 # 全局公共函数（可选）
├── config/                        # 全局配置目录
├── route/                         # 全局路由目录
├── public/
├── view/                          # 全局视图目录
├── runtime/
├── vendor/oyta_php/bin/oyta
├── .env
└── oyta
```

### 2.4 比 ThinkPHP 8.0 更简洁的 app 目录

OYTAPHP 的 `app/` 目录比 ThinkPHP 8.0 更简洁，原因如下：

| ThinkPHP 8.0 文件 | OYTAPHP 处理方式 | 说明 |
|-------------------|-----------------|------|
| `BaseController.php` | **内置到框架** | 默认基础控制器由 Rust 二进制提供，用户无需维护 |
| `ExceptionHandle.php` | **内置到框架** | 异常处理由框架自动完成，可通过配置自定义 |
| `Request.php` | **内置到框架** | 请求对象由 Rust 自动构建，无需用户定义 |
| `event.php` | **配置化** | 事件定义移至 `config/event.php`，统一管理 |
| `middleware.php` | **配置化** | 中间件定义移至 `config/middleware.php`，统一管理 |
| `provider.php` | **配置化** | 服务提供移至 `config/provider.php`，统一管理 |
| `service/` 目录 | **按需创建** | 不再预创建空目录，用户需要时 `oyta make:service` 生成 |
| `view/` 子目录（多应用） | **全局统一** | 视图统一放在项目根 `view/` 下，按应用名分组 |
| `config/` 子目录（多应用） | **全局统一** | 配置统一放在项目根 `config/` 下 |
| `route/` 子目录（多应用） | **全局统一** | 路由统一放在项目根 `route/` 下 |

**核心原则**：框架能自动处理的，不让用户手动维护；配置能统一管理的，不让用户分散定义。

### 2.5 关键约束

- 用户项目中 **没有 Cargo.toml**、**没有 src/main.rs**、**没有 Rust 源码**
- 所有逻辑由 `vendor/oyta_php/bin/oyta` 二进制完成
- 二进制启动后自动以当前目录为项目根，扫描 `app/` 等目录下的 .php 文件
- 用户代码 100% PHP 语法，无需学习 Rust

---

## 3. 依赖库用途映射

### 3.1 PHP 语言层

| 依赖 | 版本 | 用途 |
|------|------|------|
| `php-rs-parser` | 0.9.2 | PHP 词法/语法分析，生成 CST |
| `php-ast` | 0.9.2 | PHP 抽象语法树构建与遍历 |

### 3.2 Web 服务层

| 依赖 | 版本 | 用途 |
|------|------|------|
| `axum` | 0.8.9 | HTTP 服务器框架（JSON/WS/Multipart） |
| `axum-ws-broadcaster` | 0.11.0 | WebSocket 房间广播 |
| `tower` | 0.5.3 | 中间件抽象层 |
| `tower-http` | 0.6.8 | HTTP 中间件（静态文件/CORS/Trace/压缩） |
| `actix-cors` | 0.7.1 | CORS 跨域处理 |
| `reqwest` | 0.13.2 | HTTP 客户端（用于代理/HTTP 请求） |

### 3.3 异步运行时

| 依赖 | 版本 | 用途 |
|------|------|------|
| `tokio` | 1.52.1 | 异步运行时（full features） |
| `tokio-util` | 0.7.18 | Tokio 辅助（IO 适配器） |
| `futures` | 0.3.32 | 异步工具集 |

### 3.4 数据库层

| 依赖 | 版本 | 用途 |
|------|------|------|
| `sea-orm` | 1.1.20 | 异步 ORM（MySQL/PostgreSQL/SQLite） |
| `sqlx` | 0.8.6 | 底层数据库驱动 + Query Builder |
| `rust_decimal` | 1.41.0 | 高精度十进制数（金额计算） |

### 3.5 缓存层

| 依赖 | 版本 | 用途 |
|------|------|------|
| `redis` | 1.2.0 | Redis 客户端（异步/连接池/JSON） |
| `deadpool-redis` | 0.23.0 | Redis 连接池管理 |
| `moka` | 0.12.15 | 高性能内存缓存（L1 同步+异步） |
| `cached` | 0.59.0 | 函数级缓存注解 |

### 3.6 配置层

| 依赖 | 版本 | 用途 |
|------|------|------|
| `figment` | 0.10.19 | 多源配置合并（ENV/TOML/YAML/PHP） |
| `dotenvy` | 0.15.7 | .env 文件加载 |

### 3.7 模板引擎

| 依赖 | 版本 | 用途 |
|------|------|------|
| `tera` | 1.20.1 | Jinja2 风格模板引擎（view/ 渲染） |

### 3.8 CLI 工具

| 依赖 | 版本 | 用途 |
|------|------|------|
| `clap` | 4.6.1 | 命令行参数解析 |
| `cargo-generate` | 0.23.8 | 项目模板生成 |
| `dialoguer` | 0.12.0 | 交互式命令行提示 |
| `tempfile` | 3.27.0 | 临时文件处理 |
| `assert_cmd` | 2.2.1 | CLI 集成测试 |

### 3.9 序列化

| 依赖 | 版本 | 用途 |
|------|------|------|
| `serde` | 1.0.228 | 通用序列化框架 |
| `serde_json` | 1.0.149 | JSON 序列化/反序列化 |

### 3.10 并发与同步

| 依赖 | 版本 | 用途 |
|------|------|------|
| `dashmap` | 6.1.0 | 并发 HashMap（全局符号表） |
| `parking_lot` | 0.12.5 | 高性能锁（RwLock/Mutex） |
| `once_cell` | 1.21.4 | 单次初始化（全局单例） |

### 3.11 加密与安全

| 依赖 | 版本 | 用途 |
|------|------|------|
| `sha2` | 0.11.0 | SHA-256/512 哈希 |
| `sha3` | 0.11.0 | SHA-3 哈希 |
| `aes-gcm` | 0.10.3 | AES-GCM 加密（Cookie/Session） |
| `hex` | 0.4.3 | 十六进制编解码 |
| `base64` | 0.22.1 | Base64 编解码 |

### 3.12 日志与追踪

| 依赖 | 版本 | 用途 |
|------|------|------|
| `tracing` | 0.1.44 | 结构化日志 |
| `tracing-subscriber` | 0.3.23 | 日志订阅器（env-filter） |

### 3.13 文件监听

| 依赖 | 版本 | 用途 |
|------|------|------|
| `notify` | 8.2.0 | 文件系统变更监听（热重载） |

### 3.14 工具库

| 依赖 | 版本 | 用途 |
|------|------|------|
| `bytes` | 1.11.1 | 字节缓冲区 |
| `chrono` | 0.4.44 | 时间日期处理 |
| `uuid` | 1.23.1 | UUID 生成 |
| `anyhow` | 1.0.102 | 错误处理 |
| `thiserror` | 2.0.18 | 自定义错误类型 |
| `mime` | 0.3.17 | MIME 类型判断 |

### 3.15 Composer 支持

| 依赖 | 版本 | 用途 |
|------|------|------|
| `mago-composer` | 1.24.0 | composer.json 解析与建模 |
| `semver` | 1.0.28 | 语义版本号解析与比较 |
| `zip` | 8.5.1 | ZIP 压缩包解压 |
| `flate2` | 1.1.9 | gzip 解压 |
| `tar` | 0.4.45 | tar 归档解压 |

---

## 4. 核心二进制（oyta_core）设计

### 4.1 编译与产物

```bash
cargo build --release --package oyta_core --bin oyta
# 产物：target/release/oyta
```

### 4.2 命令行体系（与 ThinkPHP 8.0 对齐）

OYTAPHP 的命令行体系与 ThinkPHP 8.0 的 `php think` 命令**一一对应**，只需将 `php think` 替换为 `oyta`：

| ThinkPHP 8.0 | OYTAPHP | 说明 |
|-------------|---------|------|
| `php think run` | `oyta run` | 启动服务 |
| `php think build demo` | `oyta build demo` | 生成应用目录 |
| `php think make:controller` | `oyta make:controller` | 创建控制器 |
| `php think make:model` | `oyta make:model` | 创建模型 |
| `php think version` | `oyta version` | 查看版本 |
| `php think clear` | `oyta clear` | 清除缓存 |
| `php think route:list` | `oyta route:list` | 查看路由 |
| `php think list` | `oyta list` | 命令列表 |

完整命令列表：

```bash
# ===== 运行服务 =====
oyta run                            # 启动服务 0.0.0.0:8000
oyta run -p 8080                    # 指定端口启动
oyta run --host=0.0.0.0 --port=80   # 指定地址和端口
oyta run --daemon                    # 守护进程模式

# ===== 项目构建 =====
oyta build demo                      # 自动生成应用目录和文件

# ===== 代码生成（make:*） =====
oyta make:controller Index           # 创建控制器
oyta make:controller admin/Index     # 创建多级控制器
oyta make:controller Index --plain   # 创建空控制器
oyta make:controller Index --api     # 创建 API 控制器
oyta make:model User                 # 创建模型
oyta make:model admin/User           # 创建多应用模型
oyta make:middleware Auth             # 创建中间件
oyta make:validate User              # 创建验证器
oyta make:event UserEvent            # 创建事件
oyta make:listener UserListener      # 创建事件监听器
oyta make:subscribe UserSubscribe    # 创建事件订阅者
oyta make:service UserService        # 创建服务类
oyta make:command CustomCommand      # 创建自定义命令

# ===== 路由管理 =====
oyta route:list                      # 查看路由列表
oyta route:cache                     # 缓存路由

# ===== 优化管理 =====
oyta optimize                        # 优化应用
oyta optimize:route                  # 生成路由缓存
oyta optimize:schema                 # 生成数据表字段缓存

# ===== 缓存管理 =====
oyta clear                           # 清除运行时缓存
oyta config:cache                    # 缓存配置

# ===== Composer 依赖管理 =====
oyta composer install                # 安装依赖
oyta composer update                 # 更新依赖
oyta composer require vendor/package # 添加依赖
oyta composer remove vendor/package  # 移除依赖
oyta composer dump-autoload          # 重新生成自动加载
oyta composer show                   # 查看已安装包
oyta composer outdated               # 查看过时包
oyta composer validate               # 验证 composer.json

# ===== 其他 =====
oyta version                         # 查看版本
oyta list                            # 命令列表
oyta help                            # 帮助信息
oyta service:discover                # 自动注册扩展包系统服务
oyta vendor:publish                  # 发布扩展配置文件
```

### 4.3 内嵌式服务（WebSocket/定时器/队列）

OYTAPHP 的核心理念是**更简**——WebSocket、定时器、队列等服务**不需要通过命令行单独启动**，而是在代码中声明即可自动运行。

#### 在 public/index.php 中声明内嵌服务

```php
// public/index.php
// OYTAPHP 入口文件 —— 在此声明需要自动运行的服务

// WebSocket 服务（自动随主服务启动）
\oyta\WebSocket::start('/ws', function ($ws) {
    $ws->on('open', function ($connection) {
        $ws->join('room_1', $connection);
    });

    $ws->on('message', function ($connection, $data) {
        $ws->broadcast('room_1', $data);
    });

    $ws->on('close', function ($connection) {
        $ws->leave('room_1', $connection);
    });
});

// 定时任务（自动随主服务启动）
\oyta\Timer::every(60, function () {
    // 每60秒执行一次
    \oyta\facade\Cache::clear('temp');
});

\oyta\Timer::cron('0 2 * * *', function () {
    // 每天凌晨2点执行
    \app\service\CleanupService::run();
});

\oyta\Timer::daily('03:00', function () {
    // 每天3点执行
    \app\service\ReportService::generate();
});

// 异步队列（自动随主服务启动）
\oyta\Queue::process('emails', function ($job) {
    \app\service\MailService::send($job->data);
    $job->ack();
});
```

#### 内嵌服务工作原理

```
oyta run 启动
  │
  ├─ 1. 解析 public/index.php
  │     ├─ 检测 \oyta\WebSocket::start() 声明
  │     ├─ 检测 \oyta\Timer::* 声明
  │     └─ 检测 \oyta\Queue::process() 声明
  │
  ├─ 2. 启动 Axum HTTP 服务（主服务）
  │
  ├─ 3. 启动内嵌 WebSocket 服务（独立 tokio task）
  │     └─ 注册到 Axum Router 的 /ws 路径
  │
  ├─ 4. 启动内嵌定时器（独立 tokio task）
  │     ├─ every → tokio::time::interval
  │     ├─ cron → cron 表达式调度器
  │     └─ daily → 每日定时触发
  │
  └─ 5. 启动内嵌队列消费者（独立 tokio task）
        └─ Redis BRPOP 监听队列
```

#### 内嵌服务 API

| API | 说明 | 示例 |
|-----|------|------|
| `\oyta\WebSocket::start($path, $handler)` | 启动 WebSocket 服务 | `WebSocket::start('/ws', fn($ws) => ...)` |
| `\oyta\Timer::every($seconds, $callback)` | 间隔执行 | `Timer::every(60, fn() => ...)` |
| `\oyta\Timer::cron($expression, $callback)` | Cron 表达式 | `Timer::cron('0 2 * * *', fn() => ...)` |
| `\oyta\Timer::daily($time, $callback)` | 每日定时 | `Timer::daily('03:00', fn() => ...)` |
| `\oyta\Timer::once($delay, $callback)` | 延迟执行一次 | `Timer::once(30, fn() => ...)` |
| `\oyta\Queue::process($queue, $handler)` | 处理队列任务 | `Queue::process('emails', fn($job) => ...)` |
| `\oyta\Queue::dispatch($queue, $data)` | 分发队列任务 | `Queue::dispatch('emails', $data)` |

### 4.4 启动流程

```
oyta run
  │
  ├─ 1. 确定项目根目录（当前工作目录）
  │
  ├─ 2. 加载环境变量
  │     ├─ 读取 .env 文件（dotenvy）
  │     └─ 合并系统环境变量
  │
  ├─ 3. 加载配置
  │     ├─ 解析 config/*.php（解释器执行 return 数组）
  │     └─ 合并 .env 覆盖配置
  │
  ├─ 4. 初始化运行时组件
  │     ├─ 数据库连接池（SeaORM）
  │     ├─ 缓存驱动（moka L1 + Redis L2）
  │     ├─ Session 驱动
  │     ├─ 日志系统（tracing）
  │     └─ 模板引擎（Tera）
  │
  ├─ 5. 扫描与解析 PHP 文件
  │     ├─ 扫描 app/controller/**/*.php → 控制器符号表
  │     ├─ 扫描 app/model/**/*.php → 模型符号表
  │     ├─ 扫描 app/middleware/**/*.php → 中间件符号表
  │     ├─ 扫描 app/validate/**/*.php → 验证器符号表
  │     ├─ 扫描 app/event/**/*.php → 事件符号表
  │     ├─ 扫描 app/service/**/*.php → 服务符号表
  │     ├─ 解析 route/**/*.php → 路由定义
  │     └─ 解析 app/common.php → 公共函数
  │
  ├─ 6. 构建路由表
  │     ├─ 自动路由（Controller/Method 映射）
  │     ├─ 定义路由（route/*.php 中的 Route::rule 等）
  │     └─ 资源路由（Route::resource）
  │
  ├─ 7. 注册中间件
  │     ├─ 全局中间件
  │     ├─ 路由中间件
  │     └─ 控制器中间件
  │
  ├─ 8. 解析内嵌服务声明
  │     ├─ 解析 public/index.php 中的 \oyta\WebSocket::start()
  │     ├─ 解析 \oyta\Timer::* 定时器声明
  │     └─ 解析 \oyta\Queue::process() 队列声明
  │
  ├─ 9. 启动热重载监视器
  │     └─ notify 监视 app/、config/、route/ 下 .php 文件变更
  │
  └─ 10. 启动 Axum HTTP 服务
        ├─ 注册路由
        ├─ 注册中间件链
        ├─ 注册静态文件服务（public/）
        ├─ 注册 WebSocket 端点（如有声明）
        ├─ 启动定时器任务（如有声明）
        ├─ 启动队列消费者（如有声明）
        └─ 监听 0.0.0.0:8000
```

---

## 5. PHP 解析器与解释器

### 5.1 解析器设计

使用 `php-rs-parser` + `php-ast` 构建 PHP 代码解析管线：

```
.php 源码 → php-rs-parser（词法+语法分析）→ CST → php-ast（AST 构建）→ OYTAPHP AST（语义增强）
```

### 5.2 支持的 PHP 语法（完整覆盖 ThinkPHP 8.0 所需）

#### 基础语法

| 特性 | 示例 | 优先级 |
|------|------|--------|
| 变量与赋值 | `$name = 'think';` | P0 |
| 数据类型 | `string/int/float/bool/array/null/object` | P0 |
| 运算符 | 算术/比较/逻辑/位/赋值/三元/null合并 | P0 |
| 字符串操作 | 拼接/插值/heredoc/nowdoc | P0 |
| 数组 | 索引数组/关联数组/多维数组/数组解构 | P0 |
| 控制结构 | `if/else/elseif/switch/for/foreach/while/do-while` | P0 |
| 函数定义与调用 | `function foo($a, $b): string {}` | P0 |
| 闭包/Lambda | `fn($x) => $x * 2` / `function($x) use ($y) {}` | P0 |
| 返回值 | `return` / `return []` | P0 |
| 命名空间 | `namespace app\controller;` | P0 |
| use 导入 | `use oyta\Model;` / `use app\model\User;` | P0 |

#### 面向对象

| 特性 | 示例 | 优先级 |
|------|------|--------|
| 类定义 | `class Index {}` | P0 |
| 构造函数 | `public function __construct() {}` | P0 |
| 方法定义与调用 | `public function index() {}` | P0 |
| 静态方法 | `public static function make() {}` | P0 |
| 属性 | `public $name;` / `protected int $id;` | P0 |
| 继承 | `class User extends Model {}` | P0 |
| 接口 | `interface Renderable {}` | P1 |
| 抽象类 | `abstract class Base {}` | P1 |
| Trait | `use SoftDelete;` | P1 |
| 访问修饰符 | `public/protected/private` | P0 |
| $this 引用 | `$this->name` | P0 |
| static/self/parent | `static::class` / `parent::__construct()` | P0 |
| 魔术方法 | `__get/__set/__call/__callStatic/__toString` | P1 |
| 常量 | `const VERSION = '1.0';` | P0 |
| 类型提示 | `public function show(int $id): string` | P0 |
| 匿名类 | `new class {}` | P2 |

#### PHP 8.0+ 特性

| 特性 | 示例 | 优先级 |
|------|------|--------|
| 命名参数 | `foo(name: 'think')` | P1 |
| 联合类型 | `int\|string` | P1 |
| match 表达式 | `match($status) { 1 => 'ok' }` | P1 |
| Null 安全运算符 | `$user?->name` | P1 |
| 属性（Attribute） | `#[Route('/hello')]` | P2 |
| 构造器属性提升 | `public function __construct(private int $id) {}` | P2 |
| readonly 属性 | `public readonly string $name` | P2 |

#### 动态特性

| 特性 | 示例 | 优先级 |
|------|------|--------|
| 可变变量 | `$$name` | P1 |
| 可变函数 | `$func()` | P1 |
| 可变方法 | `$obj->$method()` | P1 |
| call_user_func | `call_user_func($callback, $arg)` | P2 |
| 数组访问 | `$obj['key']`（实现 ArrayAccess） | P1 |

### 5.3 解释器架构

```
┌─────────────────────────────────────────────────┐
│                  OYTAPHP 解释器                    │
├─────────────────────────────────────────────────┤
│                                                   │
│  ┌───────────┐   ┌───────────┐   ┌───────────┐  │
│  │  AST 节点  │──▶│  语义分析  │──▶│  执行引擎  │  │
│  └───────────┘   └───────────┘   └───────────┘  │
│                                       │          │
│                    ┌──────────────────┼──────┐   │
│                    ▼                  ▼      ▼   │
│              ┌──────────┐  ┌──────────┐ ┌─────┐ │
│              │ 变量作用域 │  │ 函数调用栈│ │对象栈│ │
│              └──────────┘  └──────────┘ └─────┘ │
│                                                   │
│  ┌─────────────────────────────────────────────┐ │
│  │              Facade 调度层                     │ │
│  ├─────────┬─────────┬──────────┬──────────────┤ │
│  │ Db::    │Cache::  │Session:: │ Config::     │ │
│  │ →SeaORM │→moka/   │→moka/    │ →figment     │ │
│  │         │ Redis   │ Redis    │              │ │
│  ├─────────┼─────────┼──────────┼──────────────┤ │
│  │Log::    │Event::  │Validate::│ Filesystem:: │ │
│  │→tracing │→tokio   │→Rust验证 │ →tokio::fs   │ │
│  │         │broadcast│引擎      │              │ │
│  ├─────────┼─────────┼──────────┼──────────────┤ │
│  │Request::│Response::│Env::    │ Lang::       │ │
│  │→axum    │→axum    │→dotenvy  │ →i18n模块    │ │
│  │Extractor│Response │          │              │ │
│  └─────────┴─────────┴──────────┴──────────────┘ │
│                                                   │
│  ┌─────────────────────────────────────────────┐ │
│  │              内置函数库                        │ │
│  ├─────────────────────────────────────────────┤ │
│  │ 字符串：strlen/substr/explode/implode/trim/  │ │
│  │         str_replace/json_encode/json_decode  │ │
│  │ 数组：  array_push/array_merge/in_array/     │ │
│  │         array_map/array_filter/count/sort    │ │
│  │ 时间：  date/time/strtotime/date_format      │ │
│  │ 数学：  intval/floatval/round/ceil/floor     │ │
│  │ 文件：  file_get_contents/file_put_contents  │ │
│  │ 助手：  input/dump/dd/halt/redirect/json/    │ │
│  │         url/cache/session/config/env         │ │
│  └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

### 5.4 符号表设计

使用 `dashmap` 存储全局符号表，支持并发读写：

```rust
// 符号表结构
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

struct ClassDef {
    namespace: String,
    name: String,
    parent: Option<String>,         // 父类
    traits: Vec<String>,            // Trait 列表
    methods: HashMap<String, MethodDef>,
    properties: HashMap<String, PropertyDef>,
    constants: HashMap<String, Value>,
    ast: PhpAst,                    // 缓存的 AST
    file_path: PathBuf,
    last_modified: SystemTime,
}
```

---

## 6. Web 服务与路由系统

### 6.1 路由模式（与 ThinkPHP 8.0 完全对齐）

#### 普通模式（自动路由）

```
URL: /index/hello/name/think
映射: app\controller\Index::hello($name = 'think')
```

#### 混合模式（定义路由 + 自动路由）

```php
// route/route.php
use oyta\facade\Route;

Route::rule('hello/:name', 'index/hello');
Route::get('api/users', 'api.User/index');
Route::post('api/users', 'api.User/save');
```

#### 强制路由模式

```php
// config/route.php
return [
    'url_route_must' => true,
];
```

#### RESTful 资源路由

```php
Route::resource('blog', 'Blog');
// 自动生成：GET /blog → index
//           GET /blog/create → create
//           POST /blog → save
//           GET /blog/:id → read
//           GET /blog/:id/edit → edit
//           PUT /blog/:id → update
//           DELETE /blog/:id → delete
```

#### 路由分组

```php
Route::group('admin', function () {
    Route::rule('index', 'admin.Index/index');
    Route::rule('user', 'admin.User/index');
})->middleware('auth');
```

#### 注解路由

```php
namespace app\controller;

use oyta\annotation\Route;

class Index
{
    #[Route('hello/:name', method: 'GET')]
    public function hello(string $name)
    {
        return 'Hello,' . $name;
    }
}
```

#### MISS 路由

```php
Route::miss(function () {
    return '404 Not Found';
});
```

### 6.2 路由实现

```
Axum Router
  │
  ├─ 静态路由匹配（O(1) HashMap 查找）
  │    例：GET /api/users → 精确匹配
  │
  ├─ 动态路由匹配（Trie 前缀树 O(k) 匹配）
  │    例：GET /hello/:name → 参数提取
  │
  ├─ 正则路由匹配（编译为 Rust Regex）
  │    例：GET /hello/<name:\w+> → 正则约束
  │
  └─ 兜底路由（MISS 路由 / 默认控制器路由）
```

### 6.3 请求处理流程

```
HTTP 请求到达
  │
  ├─ 1. Axum 接收请求 → 构建 Request 对象
  │
  ├─ 2. 路由匹配 → 确定 Controller@Method
  │
  ├─ 3. 中间件链执行（前置）
  │     ├─ 全局中间件
  │     ├─ 应用中间件（多应用模式）
  │     ├─ 路由中间件
  │     └─ 控制器中间件
  │
  ├─ 4. 参数绑定与类型转换
  │     ├─ URL 参数 → 方法参数
  │     ├─ 请求体 → 方法参数（JSON/表单）
  │     └─ 依赖注入（Request 对象等）
  │
  ├─ 5. 解释器执行 Controller 方法
  │     ├─ 创建控制器实例
  │     ├─ 执行方法体
  │     ├─ Facade 调用 → Rust 实现
  │     └─ 返回结果
  │
  ├─ 6. 响应构建
  │     ├─ 字符串 → HTML Response
  │     ├─ 数组 → JSON Response
  │     ├─ View::fetch() → 模板渲染 → HTML Response
  │     └─ redirect() → 302 Response
  │
  ├─ 7. 中间件链执行（后置）
  │
  └─ 8. 发送响应
```

### 6.4 静态文件服务

```rust
// tower-http ServeDir 处理 public/ 目录
// 优先级：动态路由 > 静态文件
// 支持：ETag / Last-Modified / Range（断点续传）
// 安全：禁止路径穿越（../）
```

---

## 7. 中间件系统

### 7.1 中间件类型（与 ThinkPHP 8.0 对齐）

| 类型 | 定义位置 | 执行顺序 | 作用域 |
|------|----------|----------|--------|
| 全局中间件 | `config/middleware.php` | 第1 | 所有请求 |
| 应用中间件 | `app/{app}/middleware/` | 第2 | 当前应用 |
| 路由中间件 | `Route::rule()->middleware()` | 第3 | 匹配路由 |
| 控制器中间件 | 控制器属性定义 | 第4 | 当前控制器 |

### 7.2 中间件执行流程（洋葱模型）

```
请求 → 全局中间件1 → 全局中间件2 → 路由中间件1 → 控制器 → 路由中间件1 → 全局中间件2 → 全局中间件1 → 响应
```

### 7.3 中间件定义方式

```php
// app/middleware/Auth.php
namespace app\middleware;

class Auth
{
    public function handle($request, \Closure $next)
    {
        if (!session('user_id')) {
            return redirect('/login');
        }
        return $next($request);
    }

    public function end(\oyta\Response $response)
    {
        // 请求结束后的回调
    }
}
```

### 7.4 内置中间件

| 中间件 | 功能 |
|--------|------|
| SessionInit | Session 初始化 |
| RequestCache | 请求缓存 |
| LoadLangPack | 多语言加载 |
| CORS | 跨域处理 |
| Trace | 调试追踪 |

---

## 8. 数据库与模型系统

### 8.1 数据库支持

| 数据库 | 驱动 | 状态 |
|--------|------|------|
| MySQL 5.6+ | sqlx + SeaORM | P0 |
| PostgreSQL | sqlx + SeaORM | P0 |
| SQLite | sqlx + SeaORM | P0 |

### 8.2 Db Facade（查询构造器）

```php
// 查询
$users = Db::table('user')->where('status', 1)->select();
$user  = Db::table('user')->where('id', 1)->find();
$count = Db::table('user')->where('status', 1)->count();

// 插入
Db::table('user')->insert(['name' => 'think', 'age' => 25]);
Db::table('user')->insertGetId(['name' => 'think', 'age' => 25]);

// 更新
Db::table('user')->where('id', 1)->update(['name' => 'oyta']);

// 删除
Db::table('user')->where('id', 1)->delete();

// 链式操作
Db::table('user')
    ->alias('u')
    ->join('profile p', 'u.id = p.user_id')
    ->field('u.name, p.avatar')
    ->where('u.status', 1)
    ->order('u.id', 'desc')
    ->limit(10)
    ->select();

// 聚合查询
Db::table('order')->count();
Db::table('order')->max('amount');
Db::table('order')->min('amount');
Db::table('order')->avg('amount');
Db::table('order')->sum('amount');

// 事务
Db::transaction(function () {
    Db::table('user')->insert([...]);
    Db::table('account')->insert([...]);
});

Db::startTrans();
try {
    Db::table('user')->insert([...]);
    Db::commit();
} catch (\Exception $e) {
    Db::rollback();
}

// 原生 SQL
Db::query('SELECT * FROM user WHERE id = ?', [1]);
Db::execute('UPDATE user SET name = ? WHERE id = ?', ['oyta', 1]);
```

### 8.3 Model（Active Record 模式）

```php
namespace app\model;

use oyta\Model;

class User extends Model
{
    protected $pk = 'id';
    protected $name = 'user';
    protected $autoWriteTimestamp = true;

    // 获取器
    public function getStatusAttr($value)
    {
        $status = [0 => '禁用', 1 => '正常'];
        return $status[$value] ?? '未知';
    }

    // 修改器
    public function setPasswordAttr($value)
    {
        return password_hash($value, PASSWORD_DEFAULT);
    }

    // 搜索器
    public function searchNameAttr($query, $value)
    {
        $query->where('name', 'like', '%' . $value . '%');
    }

    // 关联
    public function profile()
    {
        return $this->hasOne(Profile::class);
    }

    public function orders()
    {
        return $this->hasMany(Order::class);
    }

    public function roles()
    {
        return $this->belongsToMany(Role::class);
    }
}

// 使用
$user = User::find(1);
$user = User::where('name', 'think')->find();
$users = User::where('status', 1)->select();
$user->name = 'oyta';
$user->save();
$user->delete();
```

### 8.4 关联关系

| 关联类型 | 方法 | 说明 |
|----------|------|------|
| 一对一 | `hasOne()` | User → Profile |
| 一对一（反向） | `belongsTo()` | Profile → User |
| 一对多 | `hasMany()` | User → Orders |
| 一对多（反向） | `belongsTo()` | Order → User |
| 多对多 | `belongsToMany()` | User ↔ Role |
| 远程一对一 | `hasOneThrough()` | User → Book（通过 Profile） |
| 远程一对多 | `hasManyThrough()` | User → Books（通过 Profile） |
| 多态一对一 | `morphOne()` | Comment → Image/Video |
| 多态一对多 | `morphMany()` | Comment → Images/Videos |
| 多态多对多 | `morphToMany()` | Tag → Articles/Videos |

### 8.5 模型特性

| 特性 | 说明 |
|------|------|
| 自动时间戳 | `create_time` / `update_time` 自动填充 |
| 软删除 | `delete_time` 字段，查询自动排除 |
| JSON 字段 | 自动序列化/反序列化 JSON 字段 |
| 类型转换 | `$schema = ['id' => 'int', 'price' => 'float']` |
| 获取器/修改器 | 自定义字段读写逻辑 |
| 搜索器 | 简化搜索条件构建 |
| 模型事件 | `before_insert/after_insert/before_update/...` |
| 查询范围 | `scope()` 定义常用查询条件 |

---

## 9. 缓存系统

### 9.1 Cache Facade

```php
// 基本操作
Cache::set('name', $value, 3600);
Cache::get('name');
Cache::get('name', 'default');
Cache::delete('name');
Cache::clear();
Cache::has('name');

// 自增/自减
Cache::inc('counter');
Cache::inc('counter', 3);
Cache::dec('counter');
Cache::dec('counter', 3);

// 不存在则写入
Cache::remember('config', function () {
    return Db::table('config')->select();
}, 3600);

// 获取并删除
Cache::pull('token');

// 追加
Cache::set('list', [1, 2, 3]);
Cache::push('list', 4);

// 缓存标签
Cache::tag('user')->set('user_1', $data);
Cache::tag('user')->set('user_2', $data);
Cache::tag('user')->clear();

// 切换缓存驱动
Cache::store('redis')->set('name', $value);
Cache::store('file')->get('name');
```

### 9.2 缓存驱动

| 驱动 | Rust 实现 | 说明 |
|------|-----------|------|
| file | tokio::fs + serde | 文件缓存（默认） |
| redis | deadpool-redis | Redis 缓存 |
| memory | moka | 内存缓存（L1） |

### 9.3 多级缓存架构

```
请求 → moka L1 内存缓存（纳秒级）
         │ miss
         ▼
      Redis L2 缓存（毫秒级）
         │ miss
         ▼
      数据库查询（SeaORM）
         │
         ▼
      回填 L1 + L2 缓存
```

### 9.4 缓存配置

```php
// config/cache.php
return [
    'default' => 'file',
    'stores'  => [
        'file'   => [
            'type'   => 'file',
            'path'   => runtime_path() . 'cache/',
            'expire' => 0,
        ],
        'redis'  => [
            'type'       => 'redis',
            'host'       => '127.0.0.1',
            'port'       => 6379,
            'password'   => '',
            'select'     => 0,
            'timeout'    => 0,
            'persistent' => false,
            'prefix'     => 'oyta_',
        ],
        'memory' => [
            'type'   => 'memory',
            'max_capacity' => 10000,
            'ttl'    => 3600,
        ],
    ],
];
```

---

## 10. Session 系统

### 10.1 Session Facade

```php
Session::set('user_id', 1);
Session::get('user_id');
Session::get('user_id', 0);
Session::delete('user_id');
Session::clear();
Session::has('user_id');
Session::push('list', 'item');
Session::flash('message', '操作成功');
```

### 10.2 Session 驱动

| 驱动 | Rust 实现 | 说明 |
|------|-----------|------|
| file | tokio::fs | 文件存储（默认） |
| redis | deadpool-redis | Redis 存储 |
| cache | Cache Facade | 使用缓存驱动 |

### 10.3 Session 配置

```php
// config/session.php
return [
    'type'       => 'file',
    'store'      => null,
    'expire'     => 1440,
    'prefix'     => '',
    'auto_start' => true,
    'cookie'     => [
        'name'     => 'OYTA_SESSION_ID',
        'lifetime' => 1440,
        'path'     => '/',
        'domain'   => '',
        'secure'   => false,
        'httponly' => true,
        'samesite' => 'Lax',
    ],
];
```

---

## 11. 模板引擎

### 11.1 View Facade

```php
// 控制器中使用
return View::fetch('index/hello', [
    'name' => 'think',
    'list' => $users,
]);

return View::assign('name', 'think')->fetch();

// 模板布局
return View::layout('layout/main')->fetch('index/hello');
```

### 11.2 模板语法（Tera 引擎，兼容 ThinkPHP 风格）

```
{-- 变量输出 --}
{{ name }}
{{ user.name }}
{{ user.getName() }}

{-- 条件判断 --}
{% if status == 1 %}
    正常
{% elseif status == 0 %}
    禁用
{% else %}
    未知
{% endif %}

{-- 循环 --}
{% for item in list %}
    {{ item.name }}
{% endfor %}

{-- 包含子模板 --}
{% include "public/header" %}

{-- 模板继承 --}
{% extends "layout/main" %}
{% block content %}
    页面内容
{% endblock %}

{-- 原始输出（不转义） --}
{{ content | safe }}

{-- 默认值 --}
{{ name | default(value="默认值") }}

{-- 日期格式化 --}
{{ create_time | date(format="%Y-%m-%d %H:%M:%S") }}

{-- URL 生成 --}
{{ url('index/hello', {name: 'think'}) }}
```

### 11.3 模板配置

```php
// config/view.php
return [
    'type'          => 'Tera',
    'view_path'     => '',
    'view_suffix'   => 'html',
    'view_depr'     => '/',
    'tpl_begin'     => '{',
    'tpl_end'       => '}',
    'taglib_begin'  => '{',
    'taglib_end'    => '}',
    'layout_on'     => false,
    'layout_name'   => 'layout/index',
    'layout_item'   => '{__CONTENT__}',
    'cache_prefix'  => 'tpl_',
];
```

---

## 12. 验证器系统

### 12.1 Validate 类

```php
namespace app\validate;

use oyta\Validate;

class UserValidate extends Validate
{
    protected $rule = [
        'name'     => 'require|max:25',
        'email'    => 'require|email',
        'age'      => 'require|between:1,120',
        'password' => 'require|min:6',
        'phone'    => 'require|regex:/^1[3-9]\d{9}$/',
    ];

    protected $message = [
        'name.require'     => '名称必须',
        'name.max'         => '名称最多25个字符',
        'email.require'    => '邮箱必须',
        'email.email'      => '邮箱格式错误',
        'age.between'      => '年龄必须在1-120之间',
        'password.min'     => '密码至少6位',
        'phone.regex'      => '手机号格式错误',
    ];

    protected $scene = [
        'login'  => ['email', 'password'],
        'register' => ['name', 'email', 'password', 'phone'],
    ];
}
```

### 12.2 验证规则（与 ThinkPHP 8.0 完全对齐）

| 规则 | 说明 | 示例 |
|------|------|------|
| `require` | 必须验证 | `'name' => 'require'` |
| `max` | 最大长度 | `'name' => 'max:25'` |
| `min` | 最小长度 | `'password' => 'min:6'` |
| `between` | 范围验证 | `'age' => 'between:1,120'` |
| `email` | 邮箱格式 | `'email' => 'email'` |
| `url` | URL 格式 | `'website' => 'url'` |
| `ip` | IP 地址 | `'ip' => 'ip'` |
| `integer` | 整数 | `'id' => 'integer'` |
| `float` | 浮点数 | `'price' => 'float'` |
| `alpha` | 纯字母 | `'code' => 'alpha'` |
| `alphaNum` | 字母数字 | `'code' => 'alphaNum'` |
| `alphaDash` | 字母数字下划线 | `'username' => 'alphaDash'` |
| `regex` | 正则验证 | `'phone' => 'regex:/^1[3-9]\d{9}$/'` |
| `in` | 在范围内 | `'status' => 'in:0,1,2'` |
| `notIn` | 不在范围内 | `'status' => 'notIn:-1,-2'` |
| `unique` | 唯一验证 | `'email' => 'unique:user'` |
| `confirm` | 字段确认 | `'password' => 'confirm:repassword'` |
| `date` | 日期格式 | `'date' => 'date:Y-m-d'` |
| `dateFormat` | 日期格式 | `'date' => 'dateFormat:Y-m-d H:i:s'` |
| `startWith` | 以...开头 | `'name' => 'startWith:user_'` |
| `endWith` | 以...结尾 | `'email' => 'endWith:@example.com'` |
| `contain` | 包含 | `'content' => 'contain:重要'` |
| `different` | 不等于 | `'new_pwd' => 'different:old_pwd'` |
| `egt` / `gt` | 大于等于/大于 | `'age' => 'egt:18'` |
| `elt` / `lt` | 小于等于/小于 | `'age' => 'elt:120'` |
| `length` | 长度验证 | `'name' => 'length:2,25'` |
| `number` | 纯数字 | `'id' => 'number'` |
| `mobile` | 手机号 | `'phone' => 'mobile'` |
| `idCard` | 身份证 | `'idcard' => 'idCard'` |
| `chs` | 纯中文 | `'name' => 'chs'` |
| `chsAlpha` | 中文+字母 | `'name' => 'chsAlpha'` |
| `file` | 文件上传验证 | `'avatar' => 'file'` |
| `image` | 图片验证 | `'avatar' => 'image'` |

### 12.3 验证场景

```php
$validate = new UserValidate();
$result = $validate->scene('login')->check($data);
if (!$result) {
    return json($validate->getError());
}
```

---

## 13. 事件系统

### 13.1 事件定义

```php
// config/event.php
return [
    'bind' => [
        'UserLogin' => 'app\event\UserLogin',
    ],
    'listen' => [
        'UserLogin' => ['app\listener\UserLogin'],
        'UserRegister' => ['app\listener\SendWelcomeEmail', 'app\listener\InitUserStats'],
    ],
    'subscribe' => [
        'app\subscribe\User',
    ],
];
```

### 13.2 事件触发与监听

```php
// 触发事件
Event::trigger('UserLogin', $user);
Event::listen('UserLogin', function ($user) {
    Log::info('用户登录：' . $user->name);
});

// 事件订阅
namespace app\subscribe;

class User
{
    public function onUserLogin($user) { ... }
    public function onUserLogout($user) { ... }
}
```

### 13.3 Rust 实现

```
Event::trigger → tokio::sync::broadcast 发送事件
Event::listen  → 注册异步监听器
事件调度器运行在独立 tokio task
```

---

## 14. 容器与依赖注入

### 14.1 容器

```php
// 绑定
App::bind('UserInterface', UserModel::class);

// 解析
$user = App::make('UserInterface');

// 自动依赖注入
class UserController
{
    public function index(Request $request, UserModel $user)
    {
        // $request 和 $user 自动注入
    }
}
```

### 14.2 服务提供者

```php
// config/provider.php
return [
    'app\service\DatabaseService',
    'app\service\CacheService',
];

// 服务类
namespace app\service;

class DatabaseService
{
    public function register()
    {
        App::bind('db', function () {
            return new DatabaseConnection();
        });
    }
}
```

---

## 15. Facade 门面系统（完整清单）

### 15.1 核心 Facade

| Facade | Rust 实现 | 说明 |
|--------|-----------|------|
| `App` | 运行时上下文 | 应用实例 |
| `Config` | figment | 配置管理 |
| `Env` | dotenvy | 环境变量 |
| `Request` | axum::extract::Request | 请求对象 |
| `Response` | axum::response::Response | 响应对象 |
| `Route` | 路由模块 | 路由定义 |

### 15.2 数据 Facade

| Facade | Rust 实现 | 说明 |
|--------|-----------|------|
| `Db` | SeaORM + sqlx | 数据库查询 |
| `Cache` | moka + Redis | 缓存操作 |
| `Session` | moka/Redis/file | Session 管理 |

### 15.3 服务 Facade

| Facade | Rust 实现 | 说明 |
|--------|-----------|------|
| `Log` | tracing | 日志记录 |
| `Event` | tokio::broadcast | 事件调度 |
| `Validate` | Rust 验证引擎 | 数据验证 |
| `Filesystem` | tokio::fs | 文件操作 |
| `Lang` | i18n 模块 | 多语言 |

### 15.4 视图 Facade

| Facade | Rust 实现 | 说明 |
|--------|-----------|------|
| `View` | Tera | 模板渲染 |

### 15.5 控制台 Facade

| Facade | Rust 实现 | 说明 |
|--------|-----------|------|
| `Console` | CLI 模块 | 命令行输出 |

---

## 16. 日志系统

### 16.1 Log Facade

```php
Log::info('用户登录', ['user_id' => 1]);
Log::error('数据库错误', ['error' => $e->getMessage()]);
Log::warning('缓存未命中', ['key' => 'config']);
Log::debug('调试信息', ['data' => $data]);
Log::write('自定义级别', '消息内容');
```

### 16.2 日志驱动

| 驱动 | Rust 实现 | 说明 |
|------|-----------|------|
| file | tokio::fs + tracing | 文件日志（默认） |
| stdout | tracing-subscriber | 标准输出 |

### 16.3 日志配置

```php
// config/log.php
return [
    'default' => 'file',
    'channels' => [
        'file' => [
            'type'  => 'file',
            'path'  => runtime_path() . 'log/',
            'level' => ['info', 'error', 'warning'],
            'max_files' => 30,
            'file_size' => 20971520,
        ],
    ],
    'formatter' => '[{date}] [{level}] {message} {context}',
];
```

---

## 17. 多语言系统

### 17.1 Lang Facade

```php
Lang::set('hello', '你好', 'zh-cn');
Lang::get('hello');
Lang::get('hello', [], 'en');

// 带变量
Lang::get('welcome', ['name' => 'think']);
// 对应：'welcome' => '欢迎，{:name}'
```

### 17.2 语言文件

```
app/lang/
├── zh-cn/
│   └── index.php       # return ['hello' => '你好'];
└── en/
    └── index.php       # return ['hello' => 'Hello'];
```

---

## 18. 文件系统

### 18.1 Filesystem Facade

```php
Filesystem::put('test.txt', 'content');
Filesystem::get('test.txt');
Filesystem::delete('test.txt');
Filesystem::exists('test.txt');
Filesystem::size('test.txt');
Filesystem::move('old.txt', 'new.txt');
Filesystem::copy('src.txt', 'dst.txt');
```

### 18.2 文件上传

```php
// 控制器中处理上传
public function upload(Request $request)
{
    $file = $request->file('avatar');
    $path = $file->store('uploads/avatar');
    return json(['path' => $path]);
}

// 验证上传
$validate = new Validate([
    'avatar' => 'file|image|max:2048',
]);
```

---

## 19. Cookie 系统

```php
Cookie::set('name', 'value', 3600);
Cookie::get('name');
Cookie::get('name', 'default');
Cookie::delete('name');
Cookie::has('name');
Cookie::forever('name', 'value');
```

---

## 20. 热重载系统

### 20.1 监视范围

| 目录 | 监视内容 | 触发动作 |
|------|----------|----------|
| `app/**/*.php` | 控制器/模型/中间件等 | 重新解析 → 更新符号表 |
| `config/**/*.php` | 配置文件 | 重新加载配置 |
| `route/**/*.php` | 路由定义 | 重新构建路由表 |
| `view/**/*` | 模板文件 | 清除模板缓存 |

### 20.2 热重载流程

```
notify 监视器（独立 tokio task）
  │
  ├─ 检测到 .php 文件变更
  │
  ├─ 读取文件内容
  │
  ├─ php-rs-parser 重新解析
  │
  ├─ 更新 dashmap 符号表
  │     ├─ 新增类 → 插入
  │     ├─ 修改类 → 替换
  │     └─ 删除类 → 移除
  │
  ├─ 如为路由文件 → 重建路由表
  │
  ├─ 如为配置文件 → 重新加载配置
  │
  └─ 日志输出：[HotReload] Reloaded: app/controller/Index.php
```

### 20.3 热重载安全

- 解析失败时保留旧版本符号表，不影响正在运行的请求
- 日志输出解析错误信息，方便调试
- 支持配置忽略目录（如 `.git/`、`runtime/`、`vendor/`）

---

## 21. WebSocket 与内嵌服务系统

### 21.1 内嵌式设计理念

OYTAPHP 将 WebSocket、定时器、队列等服务设计为**内嵌式**——在 `public/index.php` 中声明即可，`oyta run` 启动时自动识别并启动这些服务，**无需通过命令行单独启动**。

这与 ThinkPHP 8.0 需要单独启动 WebSocket 服务、定时任务的方式不同，OYTAPHP 更简更高效。

### 21.2 WebSocket 服务

```php
// public/index.php 中声明 WebSocket
\oyta\WebSocket::start('/ws', function ($ws) {
    $ws->on('open', function ($connection) {
        $ws->join('room_1', $connection);
    });

    $ws->on('message', function ($connection, $data) {
        $ws->broadcast('room_1', $data);
    });

    $ws->on('close', function ($connection) {
        $ws->leave('room_1', $connection);
    });
});
```

### 21.3 定时器服务

```php
// public/index.php 中声明定时任务
\oyta\Timer::every(60, function () {
    // 每60秒执行
    \oyta\facade\Cache::clear('temp');
});

\oyta\Timer::cron('0 2 * * *', function () {
    // Cron 表达式：每天凌晨2点
    \app\service\CleanupService::run();
});

\oyta\Timer::daily('03:00', function () {
    // 每天固定时间
    \app\service\ReportService::generate();
});

\oyta\Timer::once(30, function () {
    // 延迟30秒执行一次
    \app\service\InitService::setup();
});
```

### 21.4 异步队列服务

```php
// public/index.php 中声明队列消费者
\oyta\Queue::process('emails', function ($job) {
    \app\service\MailService::send($job->data);
    $job->ack();
});

// 在控制器中分发队列任务
\oyta\Queue::dispatch('emails', [
    'to' => 'user@example.com',
    'subject' => '欢迎',
    'body' => '...',
]);
```

### 21.5 Rust 实现

```
内嵌服务管理器（EmbeddedServiceManager）
  │
  ├─ WebSocket 服务
  │   └─ axum-ws-broadcaster
  │       ├─ 房间管理（创建/加入/离开/销毁）
  │       ├─ 消息广播（房间内广播/全局广播）
  │       ├─ 心跳检测（可配置间隔）
  │       └─ 连接管理（最大连接数/超时断开）
  │
  ├─ 定时器服务
  │   └─ tokio::time
  │       ├─ every → tokio::time::interval
  │       ├─ cron → cron 表达式调度器
  │       ├─ daily → 每日定时触发
  │       └─ once → tokio::time::sleep
  │
  └─ 队列服务
      └─ Redis BRPOP
          ├─ 多队列并发消费
          ├─ 任务确认/重试机制
          ├─ 延迟队列支持
          └─ 死信队列处理
```

---

## 22. 安全系统

### 22.1 安全机制

| 安全项 | 实现方式 |
|--------|----------|
| SQL 注入防护 | SeaORM 参数化查询 + 查询构造器自动转义 |
| XSS 防护 | 模板引擎默认 HTML 转义 |
| CSRF 防护 | Session + Token 验证 |
| 文件上传安全 | 文件类型白名单/大小限制/路径穿越防护 |
| 路径遍历防护 | 静态文件服务禁止 `../` |
| Cookie 安全 | HttpOnly/Secure/SameSite 属性 |
| Session 安全 | 随机 Session ID + 过期机制 |
| 加密 | AES-GCM 加密敏感数据 |
| 哈希 | SHA-256/SHA-3 密码哈希 |
| 输入过滤 | `input()` 函数自动类型转换 |
| 请求频率限制 | 中间件实现 Rate Limiting |
| CORS | 可配置跨域策略 |

### 22.2 输入过滤

```php
// input() 助手函数自动过滤
$id = input('id/d');          // 强制整数
$name = input('name/s');      // 强制字符串
$price = input('price/f');    // 强制浮点数
$page = input('page/d', 1);   // 带默认值
```

类型标识：

| 标识 | 类型 | 说明 |
|------|------|------|
| `/s` | string | 字符串 |
| `/d` | int | 整数 |
| `/f` | float | 浮点数 |
| `/b` | bool | 布尔值 |
| `/a` | array | 数组 |

---

## 23. 错误处理与调试

### 23.1 错误处理

OYTAPHP 默认内置异常处理器，用户也可通过配置自定义：

```php
// config/exception.php
return [
    'handle' => app\ExceptionHandle::class,
];

// app/ExceptionHandle.php（可选，按需创建）
namespace app;

use oyta\exception\Handle;

class ExceptionHandle extends Handle
{
    public function render($request, \Throwable $e)
    {
        if ($e instanceof ValidateException) {
            return json($e->getError(), 422);
        }
        if ($e instanceof HttpException) {
            return response($e->getMessage(), $e->getStatusCode());
        }
        return parent::render($request, $e);
    }
}
```

### 23.2 调试模式

```env
# .env
APP_DEBUG = true
```

调试模式开启时：
- 显示详细错误堆栈（文件名+行号+代码片段）
- 显示 SQL 查询日志
- 显示 Trace 调试面板
- 显示内存使用和执行时间
- 禁用路由/配置缓存

### 23.3 错误页面

| 模式 | 行为 |
|------|------|
| 调试模式 | 显示详细错误信息（文件/行号/堆栈/SQL） |
| 生产模式 | 显示友好错误页面，记录日志 |

---

## 24. 助手函数

### 24.1 内置助手函数（与 ThinkPHP 8.0 对齐）

| 函数 | 说明 |
|------|------|
| `input($key, $default)` | 获取输入参数 |
| `config($key, $default)` | 获取配置值 |
| `env($key, $default)` | 获取环境变量 |
| `url($path, $params)` | 生成 URL |
| `json($data, $code)` | 返回 JSON 响应 |
| `redirect($url)` | 重定向 |
| `cache($key, $value, $ttl)` | 缓存操作 |
| `session($key, $value)` | Session 操作 |
| `cookie($key, $value)` | Cookie 操作 |
| `dump($var)` | 打印变量 |
| `dd($var)` | 打印并终止 |
| `halt($var)` | 中断执行 |
| `app()` | 获取应用实例 |
| `request()` | 获取请求对象 |
| `response($data, $code)` | 创建响应 |
| `view($template, $data)` | 渲染模板 |
| `event($name, $data)` | 触发事件 |
| `validate($data, $rule)` | 数据验证 |
| `db($table)` | 数据库查询 |
| `model($class)` | 获取模型实例 |
| `runtime_path()` | 运行时目录 |
| `root_path()` | 项目根目录 |
| `app_path()` | 应用目录 |
| `config_path()` | 配置目录 |
| `public_path()` | 公共目录 |
| `vendor_path()` | Vendor 目录 |

---

## 25. oyta_cli 全局 CLI 工具

### 25.1 两个二进制的区别

| 二进制 | 来源 | 安装方式 | 使用场景 |
|--------|------|----------|----------|
| `oyta-cli` | oyta_cli crate | `cargo install oyta-cli` | 全局工具：项目创建、核心二进制管理 |
| `oyta` | oyta_core crate | 项目内 `vendor/oyta_php/bin/oyta` | 项目内工具：运行服务、代码生成、依赖管理 |

**注意**：`oyta-cli` 是全局安装的独立工具，`oyta` 是项目内的核心运行时。两者命令有重叠但职责不同：
- `oyta-cli new myapp`：在任意位置创建新项目
- `oyta run`：在项目内启动服务
- `oyta make:controller`：在项目内生成代码

### 25.2 命令列表

```bash
# ===== oyta-cli 全局命令 =====
oyta-cli new <name>                # 创建新项目
oyta-cli update                    # 更新核心二进制
oyta-cli version                   # 查看版本
oyta-cli help                      # 帮助

# ===== oyta 项目内命令 =====
# 运行服务
oyta run                            # 启动服务 0.0.0.0:8000
oyta run -p 8080                    # 指定端口启动

# 项目构建
oyta build demo                      # 自动生成应用目录和文件

# 代码生成
oyta make:controller <name>        # 创建控制器
oyta make:model <name>             # 创建模型
oyta make:middleware <name>        # 创建中间件
oyta make:validate <name>          # 创建验证器
oyta make:event <name>             # 创建事件
oyta make:listener <name>          # 创建事件监听器
oyta make:service <name>           # 创建服务类
oyta make:command <name>           # 创建自定义命令

# 项目信息
oyta version                       # 查看版本
oyta info                          # 查看项目信息
oyta help                          # 帮助
```

### 25.3 项目创建流程

```
oyta-cli new myapp
  │
  ├─ 1. 交互式选择项目类型
  │     ├─ 单应用模式（默认）
  │     └─ 多应用模式
  │
  ├─ 2. 创建目录结构（ThinkPHP 8.0 标准）
  │
  ├─ 3. 生成默认文件
  │     ├─ app/controller/Index.php
  │     ├─ app/model/
  │     ├─ app/middleware/
  │     ├─ config/*.php（全部配置文件）
  │     ├─ route/route.php
  │     ├─ public/index.php
  │     ├─ view/index/index.html
  │     ├─ .env
  │     ├─ .example.env
  │     └─ composer.json
  │
  ├─ 4. 下载核心二进制
  │     ├─ 检测当前平台（OS + ARCH）
  │     ├─ 从 GitHub Release 下载预编译二进制
  │     ├─ 校验 SHA256 签名
  │     └─ 放置到 vendor/oyta_php/bin/oyta
  │
  └─ 5. 创建软链接
        └─ oyta → vendor/oyta_php/bin/oyta
```

---

## 26. 配置系统

### 26.1 配置加载顺序

```
1. .env 文件（dotenvy 加载）
2. config/*.php（解释器执行 return 数组）
3. .env 中的配置覆盖（APP_DEBUG → app.debug）
```

### 26.2 配置文件清单

| 文件 | 说明 |
|------|------|
| `app.php` | 应用配置（调试模式/时区/字符集等） |
| `cache.php` | 缓存配置 |
| `console.php` | 控制台配置 |
| `cookie.php` | Cookie 配置 |
| `database.php` | 数据库连接配置 |
| `event.php` | 事件定义（从 app/ 移入） |
| `exception.php` | 异常处理配置（从 app/ 移入） |
| `filesystem.php` | 文件磁盘配置 |
| `lang.php` | 多语言配置 |
| `log.php` | 日志配置 |
| `middleware.php` | 中间件定义（从 app/ 移入） |
| `provider.php` | 服务提供定义（从 app/ 移入） |
| `route.php` | URL 和路由配置 |
| `session.php` | Session 配置 |
| `trace.php` | Trace 调试配置 |
| `view.php` | 视图配置 |

### 26.3 Config Facade

```php
Config::get('app.default_timezone');
Config::get('database.connections.mysql');
Config::set('app.debug', true);
Config::has('app.debug');
```

---

## 27. 性能优化策略

### 27.1 编译时优化

| 策略 | 说明 |
|------|------|
| AST 缓存 | 解析后的 AST 缓存在 dashmap 中，热重载仅重新解析变更文件 |
| 路由编译 | 路由表编译为 Trie 前缀树，O(k) 匹配 |
| 配置缓存 | `oyta config:cache` 将配置序列化为二进制 |
| 路由缓存 | `oyta route:cache` 将路由表序列化 |

### 27.2 运行时优化

| 策略 | 说明 |
|------|------|
| 多级缓存 | moka L1（内存）+ Redis L2 |
| 数据库连接池 | SeaORM 内置连接池 |
| 模板编译缓存 | Tera 模板编译后缓存 |
| 静态文件缓存 | tower-http 内置 ETag/Last-Modified |
| 零拷贝响应 | bytes + axum 直接写入 socket |

### 27.3 性能目标

| 指标 | ThinkPHP 8.0 (PHP-FPM) | OYTAPHP (Rust) |
|------|------------------------|----------------|
| Hello World QPS | ~2,000 | >20,000 |
| DB 查询 QPS | ~800 | >5,000 |
| 内存占用 | ~50MB/进程 | ~10MB |
| 冷启动时间 | ~200ms | <50ms |
| 热重载时间 | 需重启 | <100ms |

---

## 28. 实现步骤（详细分阶段）

### 阶段 1：核心骨架（P0-1）

**目标**：二进制能启动，能解析命令

1. `oyta_core` 改为 bin 类型（`[[bin]] name = "oyta"`）
2. 实现 clap 命令解析（serve / make:* / help / version）
3. 项目根目录检测与验证
4. .env 文件加载（dotenvy）
5. 基础日志系统（tracing）

### 阶段 2：PHP 解析器集成（P0-2）

**目标**：能解析 PHP 文件，构建符号表

1. 集成 php-rs-parser + php-ast
2. 实现 .php 文件扫描与解析
3. 构建 dashmap 符号表
4. 支持 namespace、class、method、property 解析
5. 支持 `return []` 配置文件解析

### 阶段 3：配置系统（P0-3）

**目标**：完整配置加载

1. 实现 config/*.php 解析（解释器执行 return 数组）
2. 实现 figment 多源配置合并
3. 实现 .env 覆盖配置
4. 实现 Config Facade
5. 实现 Env Facade

### 阶段 4：Axum 服务基础（P0-4）

**目标**：HTTP 服务可启动

1. 实现 `oyta run` 启动 Axum
2. 实现静态文件服务（tower-http ServeDir）
3. 实现 Request/Response 对象
4. 实现 JSON 响应
5. 实现错误处理中间件

### 阶段 5：路由系统（P1-1）

**目标**：路由匹配与分发

1. 实现自动路由（Controller/Method 映射）
2. 实现定义路由（Route::rule/get/post 等）
3. 实现资源路由（Route::resource）
4. 实现路由分组
5. 实现路由缓存
6. 实现 MISS 路由

### 阶段 6：解释器核心（P1-2）

**目标**：能执行基础 PHP 代码

1. 实现变量作用域与赋值
2. 实现基础数据类型（string/int/float/bool/array/null）
3. 实现运算符
4. 实现控制结构（if/for/foreach/while/switch）
5. 实现函数定义与调用
6. 实现类实例化与方法调用
7. 实现 $this 引用
8. 实现静态方法调用
9. 实现命名空间与 use 导入

### 阶段 7：数据库集成（P1-3）

**目标**：Db Facade 可用

1. 集成 SeaORM + sqlx
2. 实现 Db Facade（查询构造器）
3. 实现 Model 基类（Active Record）
4. 实现数据库连接池
5. 实现事务支持
6. 实现数据库配置加载

### 阶段 8：缓存与 Session（P1-4）

**目标**：Cache/Session Facade 可用

1. 集成 moka 内存缓存
2. 集成 deadpool-redis
3. 实现 Cache Facade（file/redis/memory 驱动）
4. 实现 Session Facade（file/redis/cache 驱动）
5. 实现多级缓存策略

### 阶段 9：中间件系统（P2-1）

**目标**：中间件完整可用

1. 实现中间件定义与注册
2. 实现洋葱模型执行链
3. 实现全局/应用/路由/控制器中间件
4. 实现内置中间件（Session/CORS/Trace）
5. 实现中间件参数传递

### 阶段 10：热重载（P2-2）

**目标**：修改即生效

1. 集成 notify 文件监听
2. 实现 .php 文件变更检测
3. 实现符号表热更新
4. 实现路由表热重建
5. 实现配置热加载
6. 实现模板缓存清除

### 阶段 11：模板引擎（P2-3）

**目标**：View Facade 可用

1. 集成 Tera 模板引擎
2. 实现 View Facade
3. 实现模板布局
4. 实现模板继承
5. 实现模板缓存

### 阶段 12：验证器（P2-4）

**目标**：Validate 可用

1. 实现验证器基类
2. 实现全部验证规则
3. 实现验证场景
4. 实现自定义验证规则
5. 实现验证错误消息

### 阶段 13：事件系统（P2-5）

**目标**：Event Facade 可用

1. 实现事件定义与注册
2. 实现事件触发与监听
3. 实现事件订阅
4. 实现异步事件调度

### 阶段 14：容器与依赖注入（P2-6）

**目标**：IoC 容器可用

1. 实现容器绑定与解析
2. 实现自动依赖注入
3. 实现服务提供者
4. 实现 Facade 代理机制

### 阶段 15：内嵌服务与文件上传（P3-1）

**目标**：内嵌式高级功能可用

1. 实现 EmbeddedServiceManager（解析 index.php 中的 \oyta\ 声明）
2. 集成 axum-ws-broadcaster（内嵌 WebSocket）
3. 实现 \oyta\WebSocket API（start/on/join/leave/broadcast）
4. 实现 \oyta\Timer API（every/cron/daily/once）
5. 实现 \oyta\Queue API（process/dispatch）
6. 实现 multipart 文件上传
7. 实现文件验证

### 阶段 16：多语言与日志（P3-2）

**目标**：辅助功能完善

1. 实现多语言系统
2. 实现 Log Facade
3. 实现日志驱动
4. 实现助手函数

### 阶段 17：oyta_cli 工具（P3-3）

**目标**：CLI 工具完整

1. 实现 `oyta new` 项目创建
2. 实现 `oyta make:*` 代码生成
3. 实现核心二进制下载
4. 实现平台检测与二进制选择
5. 实现 SHA256 校验

### 阶段 18：调试与优化（P4）

**目标**：开发体验完善

1. 实现调试模式
2. 实现友好错误页面
3. 实现 Trace 调试面板
4. 实现路由/配置缓存命令
5. 实现性能优化

### 阶段 19：测试与发布（P5）

**目标**：生产就绪

1. 编写单元测试
2. 编写集成测试
3. 编写端到端测试
4. GitHub Actions CI/CD
5. 多平台预编译二进制
6. Docker 镜像
7. 文档编写

---

## 29. 多平台支持

### 29.1 支持平台

OYTAPHP 的核心二进制支持以下平台，覆盖主流操作系统和架构：

| 操作系统 | 架构 | Target Triple | 状态 |
|----------|------|---------------|------|
| Linux | x86_64 | `x86_64-unknown-linux-gnu` | ✅ 主要支持 |
| Linux | x86_64 (静态) | `x86_64-unknown-linux-musl` | ✅ 主要支持 |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | ✅ 主要支持 |
| Linux | ARM64 (静态) | `aarch64-unknown-linux-musl` | ✅ 主要支持 |
| macOS | Apple Silicon | `aarch64-apple-darwin` | ✅ 主要支持 |
| macOS | Intel | `x86_64-apple-darwin` | ✅ 主要支持 |
| Windows | x86_64 | `x86_64-pc-windows-msvc` | ✅ 主要支持 |
| Windows | ARM64 | `aarch64-pc-windows-msvc` | 🔄 后续支持 |

### 29.2 交叉编译

使用 `cross` 工具进行交叉编译：

```bash
# 安装 cross
cargo install cross --git https://github.com/cross-rs/cross

# 交叉编译 Linux ARM64
cross build --release --package oyta_core --target aarch64-unknown-linux-gnu
cross build --release --package oyta_cli --target aarch64-unknown-linux-gnu

# 交叉编译 Linux 静态链接（无 glibc 依赖）
cross build --release --package oyta_core --target x86_64-unknown-linux-musl
cross build --release --package oyta_cli --target x86_64-unknown-linux-musl

# 交叉编译 macOS（需要 macOS SDK）
cross build --release --package oyta_core --target aarch64-apple-darwin
```

### 29.3 本平台编译

```bash
# 在当前平台直接编译
cargo build --release --package oyta_core
cargo build --release --package oyta_cli

# 产物位置
# target/release/oyta       (Linux/macOS)
# target/release/oyta.exe   (Windows)
# target/release/oyta-cli   (Linux/macOS)
# target/release/oyta-cli.exe (Windows)
```

### 29.4 GitHub Actions CI/CD

项目内置 GitHub Actions 工作流（`.github/workflows/ci.yml`），自动完成：

| 功能 | 说明 |
|------|------|
| 多平台构建 | 自动在 Linux/macOS/Windows 上构建 |
| 多架构支持 | x86_64 + ARM64 |
| 自动测试 | 每次提交自动运行 `cargo test` |
| Release 发布 | 打 tag 时自动创建 GitHub Release |
| SHA256 校验 | 自动生成校验文件 |

**触发条件**：
- `push` 到 `main` 分支 → 构建和测试
- 创建 `v*` 标签 → 构建测试 + 发布 Release

```bash
# 发布新版本
git tag v0.1.0
git push origin v0.1.0
# GitHub Actions 自动构建并发布到 Release 页面
```

### 29.5 下载与安装

用户通过 `oyta-cli` 自动检测平台并下载对应二进制：

```bash
# 安装全局 CLI
cargo install oyta-cli

# 创建项目（自动下载对应平台的核心二进制）
oyta-cli new myapp
```

`oyta-cli new` 内部逻辑：

```
1. 检测当前平台
   ├─ OS: linux / macos / windows
   └─ ARCH: x86_64 / aarch64

2. 选择对应的二进制
   ├─ linux-x86_64   → oyta-linux-x86_64
   ├─ linux-aarch64  → oyta-linux-aarch64
   ├─ macos-aarch64  → oyta-macos-aarch64
   ├─ macos-x86_64   → oyta-macos-x86_64
   └─ windows-x86_64 → oyta-windows-x86_64.exe

3. 从 GitHub Release 下载
   └─ https://github.com/{owner}/oyta_php/releases/latest/download/oyta-{platform}

4. 校验 SHA256 签名

5. 放置到 vendor/oyta_php/bin/oyta
```

### 29.6 依赖库跨平台兼容性

| 依赖 | Linux | macOS | Windows | 说明 |
|------|-------|-------|---------|------|
| `axum` | ✅ | ✅ | ✅ | 纯 Rust |
| `tokio` | ✅ | ✅ | ✅ | 纯 Rust |
| `sea-orm` | ✅ | ✅ | ✅ | 纯 Rust（TLS 用 rustls） |
| `sqlx` | ✅ | ✅ | ✅ | 纯 Rust（TLS 用 rustls） |
| `redis` | ✅ | ✅ | ✅ | 纯 Rust |
| `moka` | ✅ | ✅ | ✅ | 纯 Rust |
| `notify` | ✅ inotify | ✅ FSEvents | ✅ ReadDirectoryChangesW | 各平台不同后端 |
| `reqwest` | ✅ | ✅ | ✅ | 纯 Rust（rustls 后端） |
| `zip`/`flate2`/`tar` | ✅ | ✅ | ✅ | 纯 Rust |
| `mago-composer` | ✅ | ✅ | ✅ | 纯 Rust |

**关键设计决策**：所有网络依赖使用 `rustls` 而非 `native-tls`/`openssl`，避免 Linux 上需要安装 OpenSSL 开发库，确保交叉编译和静态链接的便利性。

---

## 30. 潜在挑战与解决方案

| 挑战 | 解决方案 |
|------|----------|
| PHP 解释器完整性 | 渐进式实现：先覆盖 ThinkPHP 常用语法子集，逐步扩展 |
| 动态特性支持 | 对 `$$var`/`$obj->$method()` 等延迟实现，优先保证常用模式 |
| 配置文件解析 | 解释器必须先支持 `return []` 语法，这是配置加载的前置依赖 |
| 二进制分发 | GitHub Release 提供多平台预编译，oyta_cli 自动下载+SHA256 校验 |
| 跨平台兼容 | 使用 target triple 交叉编译，CI 自动构建 |
| 性能瓶颈 | AST 缓存 + 多级缓存 + 连接池 + 零拷贝 |
| 安全性 | 参数化查询 + 输入过滤 + 路径穿越防护 + 加密 |
| 多应用模式 | 先实现单应用模式，再扩展多应用支持 |
| 错误诊断 | 解析阶段友好报错（文件名+行号），运行时异常映射为 HTTP 错误 |

---

## 31. OYTAPHP vs ThinkPHP 8.0 功能对照表

| 功能 | ThinkPHP 8.0 | OYTAPHP | 说明 |
|------|-------------|---------|------|
| MVC 架构 | ✅ | ✅ | 完全支持 |
| 单应用模式 | ✅ | ✅ | 完全支持 |
| 多应用模式 | ✅ | ✅ | 完全支持 |
| 自动路由 | ✅ | ✅ | 完全支持 |
| 定义路由 | ✅ | ✅ | 完全支持 |
| 资源路由 | ✅ | ✅ | 完全支持 |
| 路由分组 | ✅ | ✅ | 完全支持 |
| 注解路由 | ✅ | ✅ | 完全支持 |
| MISS 路由 | ✅ | ✅ | 完全支持 |
| 路由缓存 | ✅ | ✅ | 完全支持 |
| 全局中间件 | ✅ | ✅ | 完全支持 |
| 应用中间件 | ✅ | ✅ | 完全支持 |
| 路由中间件 | ✅ | ✅ | 完全支持 |
| 控制器中间件 | ✅ | ✅ | 完全支持 |
| 中间件参数 | ✅ | ✅ | 完全支持 |
| Db 查询构造器 | ✅ | ✅ | 完全支持 |
| Model Active Record | ✅ | ✅ | 完全支持 |
| 关联关系 | ✅ | ✅ | 完全支持 |
| 获取器/修改器 | ✅ | ✅ | 完全支持 |
| 模型事件 | ✅ | ✅ | 完全支持 |
| 软删除 | ✅ | ✅ | 完全支持 |
| 自动时间戳 | ✅ | ✅ | 完全支持 |
| 事务支持 | ✅ | ✅ | 完全支持 |
| Cache Facade | ✅ | ✅ | 完全支持 |
| 多缓存驱动 | ✅ | ✅ | 完全支持 |
| 缓存标签 | ✅ | ✅ | 完全支持 |
| Session Facade | ✅ | ✅ | 完全支持 |
| 多 Session 驱动 | ✅ | ✅ | 完全支持 |
| 验证器 | ✅ | ✅ | 完全支持 |
| 验证场景 | ✅ | ✅ | 完全支持 |
| 自定义验证规则 | ✅ | ✅ | 完全支持 |
| 事件系统 | ✅ | ✅ | 完全支持 |
| 事件订阅 | ✅ | ✅ | 完全支持 |
| 容器/IoC | ✅ | ✅ | 完全支持 |
| 依赖注入 | ✅ | ✅ | 完全支持 |
| Facade 门面 | ✅ | ✅ | 完全支持 |
| 模板引擎 | ✅ | ✅ | Tera 替代 |
| 模板布局 | ✅ | ✅ | 完全支持 |
| 模板继承 | ✅ | ✅ | 完全支持 |
| 多语言 | ✅ | ✅ | 完全支持 |
| 日志系统 | ✅ | ✅ | 完全支持 |
| Cookie | ✅ | ✅ | 完全支持 |
| 文件系统 | ✅ | ✅ | 完全支持 |
| 文件上传 | ✅ | ✅ | 完全支持 |
| 热重载 | ❌ 需额外工具 | ✅ 内置 | **OYTAPHP 更优** |
| WebSocket | ❌ 需额外扩展 | ✅ 内嵌式 | **OYTAPHP 更优** |
| 定时任务 | ❌ 需系统 Cron | ✅ 内嵌式 | **OYTAPHP 更优** |
| 异步队列 | ❌ 需额外扩展 | ✅ 内嵌式 | **OYTAPHP 更优** |
| 内置 Web 服务器 | ✅ 简易版 | ✅ 生产级 | **OYTAPHP 更优** |
| 零环境依赖 | ❌ 需 PHP+Composer | ✅ 单二进制 | **OYTAPHP 更优** |
| 性能 | 基准 | 10x+ | **OYTAPHP 更优** |
| 内存占用 | 基准 | 1/5 | **OYTAPHP 更优** |
| 助手函数 | ✅ | ✅ | 完全支持 |
| 命令行工具 | ✅ | ✅ | 完全支持 |
| 代码生成 | ✅ | ✅ | 完全支持 |
| 调试模式 | ✅ | ✅ | 完全支持 |
| 错误处理 | ✅ | ✅ | 完全支持 |
| SQL 注入防护 | ✅ | ✅ | 完全支持 |
| XSS 防护 | ✅ | ✅ | 完全支持 |
| CSRF 防护 | ✅ | ✅ | 完全支持 |
| 跨域 CORS | ✅ | ✅ | 完全支持 |
| 数据库迁移 | ✅ | 🔄 后续版本 | 第二期实现 |
| 队列系统 | ✅ | ✅ 内嵌式 | 在 index.php 中声明即可 |
| 邮件系统 | ✅ | 🔄 后续版本 | 第二期实现 |
| Composer 依赖管理 | ✅ | ✅ | 内置 Rust 实现 |

---

## 32. Composer 依赖管理支持

### 32.1 设计理念

OYTAPHP 的核心承诺是**零 PHP 运行环境依赖**，但 Composer 是 PHP 生态的命脉——数以万计的 Packagist 包构成了开发者的日常工具链。因此 OYTAPHP 采用**内置 Rust 版 Composer 客户端**的方案，在二进制中原生支持 Composer 全部核心功能，用户无需安装 PHP 即可使用 `composer install/update/require` 等命令。

### 32.2 整体架构

```
┌──────────────────────────────────────────────────────────┐
│              OYTAPHP 内置 Composer 引擎                     │
├──────────────────────────────────────────────────────────┤
│                                                            │
│  ┌────────────────┐  ┌────────────────┐  ┌─────────────┐ │
│  │ composer.json  │  │ composer.lock  │  │ Packagist   │ │
│  │ 解析器         │  │ 读写器         │  │ API 客户端  │ │
│  │ (mago-composer)│  │ (serde_json)   │  │ (reqwest)   │ │
│  └───────┬────────┘  └───────┬────────┘  └──────┬──────┘ │
│          │                   │                   │        │
│          ▼                   ▼                   ▼        │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              依赖解析引擎（Sat Solver）               │ │
│  │  解析版本约束 → 构建依赖图 → 求解最优版本组合         │ │
│  └────────────────────────┬────────────────────────────┘ │
│                           │                               │
│          ┌────────────────┼────────────────┐              │
│          ▼                ▼                ▼              │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐     │
│  │ 下载器       │ │ 解压器       │ │ 自动加载     │     │
│  │ (reqwest     │ │ (zip/tar/   │ │ 生成器       │     │
│  │  + HTTP/2)   │ │  flate2)    │ │ (PSR-4)      │     │
│  └──────────────┘ └──────────────┘ └──────────────┘     │
│                                                            │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              vendor/ 目录管理                         │ │
│  ├─────────────────────────────────────────────────────┤ │
│  │ vendor/                                              │ │
│  │ ├── autoload.php          ← OYTAPHP 自动加载入口     │ │
│  │ ├── composer/                                       │ │
│  │ │   ├── autoload_classmap.php                       │ │
│  │ │   ├── autoload_namespaces.php                     │ │
│  │ │   ├── autoload_psr4.php                           │ │
│  │ │   ├── autoload_static.php                         │ │
│  │ │   ├── installed.json                              │ │
│  │ │   ├── installed.php                               │ │
│  │ │   └── platform_check.php                          │ │
│  │ ├── oyta_php/                                      │ │
│  │ │   └── bin/oyta         ← OYTAPHP 核心二进制      │ │
│  │ └── [vendor]/[package]/   ← 第三方 PHP 包           │ │
│  └─────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```

### 32.3 核心组件设计

#### 32.3.1 composer.json 解析器

使用 `mago-composer` crate 解析 composer.json，支持完整的 JSON Schema：

```rust
use mago_composer::ComposerJson;

struct OytaComposer {
    json: ComposerJson,
}

impl OytaComposer {
    fn load(project_root: &Path) -> Result<Self> {
        let path = project_root.join("composer.json");
        let content = tokio::fs::read_to_string(&path).await?;
        let json: ComposerJson = serde_json::from_str(&content)?;
        Ok(Self { json })
    }

    fn get_require(&self) -> &HashMap<String, String> {
        &self.json.require
    }

    fn get_autoload_psr4(&self) -> &HashMap<String, Vec<String>> {
        &self.json.autoload.psr4
    }

    fn get_repositories(&self) -> &Vec<Repository> {
        &self.json.repositories
    }
}
```

#### 32.3.2 Packagist API 客户端

```rust
struct PackagistClient {
    client: reqwest::Client,
    cache: DashMap<String, PackageMetadata>,
}

impl PackagistClient {
    async fn get_package_metadata(&self, name: &str) -> Result<PackageMetadata> {
        if let Some(cached) = self.cache.get(name) {
            return Ok(cached.clone());
        }
        let url = format!("https://repo.packagist.org/p2/{}.json", name);
        let resp = self.client.get(&url)
            .header("User-Agent", "OYTAPHP/1.0")
            .send().await?;
        let metadata: PackageMetadata = resp.json().await?;
        self.cache.insert(name.to_string(), metadata.clone());
        Ok(metadata)
    }

    async fn resolve_version(&self, name: &str, constraint: &str) -> Result<ResolvedVersion> {
        let metadata = self.get_package_metadata(name).await?;
        let constraint = semver::VersionReq::parse(constraint)?;
        let mut candidates: Vec<semver::Version> = metadata.versions.keys()
            .filter_map(|v| semver::Version::parse(v.strip_prefix('v').unwrap_or(v)).ok())
            .filter(|v| constraint.matches(v))
            .collect();
        candidates.sort();
        candidates.reverse();
        candidates.into_iter().next()
            .map(|v| ResolvedVersion { name: name.to_string(), version: v })
            .ok_or_else(|| anyhow!("No matching version for {}@{}", name, constraint))
    }
}
```

#### 32.3.3 依赖解析引擎

```rust
struct DependencyResolver {
    packagist: PackagistClient,
    resolved: DashMap<String, ResolvedPackage>,
}

impl DependencyResolver {
    async fn resolve(&self, requirements: &HashMap<String, String>) -> Result<Vec<ResolvedPackage>> {
        let mut queue = VecDeque::new();
        for (name, constraint) in requirements {
            queue.push_back((name.clone(), constraint.clone()));
        }
        while let Some((name, constraint)) = queue.pop_front() {
            if self.resolved.contains_key(&name) {
                continue;
            }
            let version = self.packagist.resolve_version(&name, &constraint).await?;
            let metadata = self.packagist.get_package_metadata(&name).await?;
            let pkg_version = metadata.versions.get(&version.to_string())
                .ok_or_else(|| anyhow!("Version not found"))?;
            if let Some(deps) = &pkg_version.require {
                for (dep_name, dep_constraint) in deps {
                    if !dep_name.starts_with("php") && !dep_name.starts_with("ext-") {
                        queue.push_back((dep_name.clone(), dep_constraint.clone()));
                    }
                }
            }
            self.resolved.insert(name.clone(), ResolvedPackage {
                name: name.clone(),
                version: version.clone(),
                dist: pkg_version.dist.clone(),
                source: pkg_version.source.clone(),
            });
        }
        Ok(self.resolved.iter().map(|r| r.value().clone()).collect())
    }
}
```

#### 32.3.4 包下载与解压

```rust
struct PackageDownloader {
    client: reqwest::Client,
    vendor_dir: PathBuf,
}

impl PackageDownloader {
    async fn download_and_extract(&self, pkg: &ResolvedPackage) -> Result<()> {
        let target_dir = self.vendor_dir.join(&pkg.name);
        if target_dir.exists() {
            return Ok(());
        }
        let dist = pkg.dist.as_ref()
            .ok_or_else(|| anyhow!("No dist for {}", pkg.name))?;
        let temp_dir = tempfile::tempdir()?;
        let archive_path = temp_dir.path().join(format!("archive.{}", dist.r#type));
        let mut resp = self.client.get(&dist.url).send().await?;
        let mut file = tokio::fs::File::create(&archive_path).await?;
        while let Some(chunk) = resp.chunk().await? {
            tokio::io::write_all(&mut file, &chunk).await?;
        }
        match dist.r#type.as_str() {
            "zip" => self.extract_zip(&archive_path, &target_dir).await?,
            "tar" | "tar.gz" | "tgz" => self.extract_tar(&archive_path, &target_dir).await?,
            _ => return Err(anyhow!("Unsupported archive type: {}", dist.r#type)),
        }
        Ok(())
    }

    async fn extract_zip(&self, archive: &Path, target: &Path) -> Result<()> {
        let file = std::fs::File::open(archive)?;
        let mut archive = zip::ZipArchive::new(file)?;
        archive.extract(target)?;
        Ok(())
    }

    async fn extract_tar(&self, archive: &Path, target: &Path) -> Result<()> {
        let file = std::fs::File::open(archive)?;
        let gz = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(gz);
        archive.unpack(target)?;
        Ok(())
    }
}
```

#### 32.3.5 PSR-4 自动加载生成器

```rust
struct AutoloadGenerator {
    vendor_dir: PathBuf,
}

impl AutoloadGenerator {
    async fn generate(&self, packages: &[ResolvedPackage]) -> Result<()> {
        let class_map = self.build_class_map(packages).await?;
        let psr4_map = self.build_psr4_map(packages).await?;
        self.write_autoload_files(&class_map, &psr4_map).await?;
        Ok(())
    }

    async fn build_class_map(&self, packages: &[ResolvedPackage]) -> Result<HashMap<String, PathBuf>> {
        let mut class_map = HashMap::new();
        for pkg in packages {
            let pkg_dir = self.vendor_dir.join(&pkg.name);
            self.scan_php_classes(&pkg_dir, &mut class_map).await?;
        }
        Ok(class_map)
    }

    async fn scan_php_classes(&self, dir: &Path, class_map: &mut HashMap<String, PathBuf>) -> Result<()> {
        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                self.scan_php_classes(&path, class_map).await?;
            } else if path.extension().map_or(false, |e| e == "php") {
                if let Some(class_name) = self.extract_class_name(&path).await? {
                    class_map.insert(class_name, path);
                }
            }
        }
        Ok(())
    }

    async fn write_autoload_files(&self, class_map: &HashMap<String, PathBuf>, psr4_map: &HashMap<String, Vec<String>>) -> Result<()> {
        let composer_dir = self.vendor_dir.join("composer");
        tokio::fs::create_dir_all(&composer_dir).await?;
        let autoload_php = self.generate_autoload_php(class_map, psr4_map)?;
        tokio::fs::write(self.vendor_dir.join("autoload.php"), autoload_php).await?;
        let classmap_php = self.generate_classmap_php(class_map)?;
        tokio::fs::write(composer_dir.join("autoload_classmap.php"), classmap_php).await?;
        let psr4_php = self.generate_psr4_php(psr4_map)?;
        tokio::fs::write(composer_dir.join("autoload_psr4.php"), psr4_php).await?;
        Ok(())
    }
}
```

### 32.4 命令行接口

OYTAPHP 内置 Composer 命令，通过 `oyta composer:*` 前缀调用：

```bash
# 安装依赖
oyta composer install                # 根据 composer.lock 安装
oyta composer install --no-dev      # 不安装开发依赖

# 更新依赖
oyta composer update                 # 更新所有依赖
oyta composer update vendor/pkg     # 更新指定包

# 添加依赖
oyta composer require vendor/package        # 添加包
oyta composer require vendor/package:^2.0   # 指定版本
oyta composer require --dev phpunit/phpunit # 添加开发依赖

# 移除依赖
oyta composer remove vendor/package

# 重新生成自动加载
oyta composer dump-autoload         # 重新生成 autoload 文件
oyta composer dump-autoload -o      # 优化（生成完整 classmap）

# 查看信息
oyta composer show                   # 列出已安装的包
oyta composer show vendor/package    # 查看包详情
oyta composer outdated               # 查看过时的包

# 验证
oyta composer validate               # 验证 composer.json

# 其他
oyta composer self-update            # 更新 OYTAPHP 自身
oyta composer clear-cache            # 清除缓存
oyta composer diagnose               # 诊断问题
```

### 32.5 自动加载集成

OYTAPHP 解释器在启动时自动加载 Composer 的自动加载映射：

```
oyta run 启动
  │
  ├─ 1. 检测 vendor/autoload.php 是否存在
  │
  ├─ 2. 读取 vendor/composer/autoload_psr4.php
  │     → 命名空间前缀 → 目录映射
  │     例："app\\" → ["/path/to/app/"]
  │     例："oyta\\" → ["/path/to/vendor/oyta_php/src/"]
  │
  ├─ 3. 读取 vendor/composer/autoload_classmap.php
  │     → 类名 → 文件路径映射
  │     例："PHPExcel" → "/path/to/vendor/phpoffice/phpexcel/PHPExcel.php"
  │
  ├─ 4. 读取 vendor/composer/autoload_namespaces.php
  │     → PEAR 风格命名空间映射
  │
  ├─ 5. 读取 vendor/composer/autoload_files.php
  │     → 需要自动加载的函数文件
  │
  └─ 6. 将所有映射注册到 OYTAPHP 符号表
        → 当解释器遇到 use/类实例化时
          先查符号表 → 未命中 → 查 PSR-4 映射 → 按需解析
```

### 32.6 PSR-4 自动加载规则

OYTAPHP 解释器内置 PSR-4 自动加载器，完全兼容 Composer 的自动加载规范：

```
命名空间前缀 → 基础目录 → 类文件路径

规则：完全限定的类名 → 前缀去除 → 替换命名空间分隔符为目录分隔符 → 追加 .php

示例：
  命名空间前缀："app\\controller\\" → "app/controller/"
  类名："app\controller\Index"
  → 文件：app/controller/Index.php

  命名空间前缀："oyta\facade\\" → "vendor/oyta_php/src/facade/"
  类名："oyta\facade\Db"
  → 文件：vendor/oyta_php/src/facade/Db.php
```

### 32.7 兼容性策略

OYTAPHP 的 Composer 支持采用**渐进式兼容**策略：

#### 第一层：完全兼容（零修改）

| 场景 | 说明 |
|------|------|
| 纯 PHP 库 | 只包含 PHP 代码的库，无需任何修改 |
| PSR-4 自动加载 | 完全兼容 Composer 的 autoload 规则 |
| 配置文件 | 兼容 `return []` 语法的配置文件 |
| 纯逻辑包 | 不依赖 PHP 扩展的纯逻辑包 |

#### 第二层：Facade 映射（自动转换）

| 原始 Composer 包 | OYTAPHP 映射 |
|------------------|---------------|
| `topthink/think-orm` | 内置 SeaORM + Db Facade |
| `topthink/think-cache` | 内置 moka + Redis + Cache Facade |
| `topthink/think-view` | 内置 Tera + View Facade |
| `topthink/think-session` | 内置 Session Facade |

OYTAPHP 内置了 ThinkPHP 核心包的 Rust 等价实现。当 `composer.json` 中 require 了这些包时，OYTAPHP 自动使用内置实现，**无需下载 PHP 版本**。

#### 第三层：扩展兼容（需要适配）

| 场景 | 说明 |
|------|------|
| PHP 扩展依赖 | `ext-json`/`ext-pdo` 等 → OYTAPHP 内置等价实现 |
| PHP 版本约束 | `php: ^8.0` → OYTAPHP 自动跳过 PHP 版本检查 |
| 平台包 | `composer/platform` → OYTAPHP 提供虚拟平台包 |

### 32.8 composer.json 支持

OYTAPHP 完全支持标准 composer.json 格式：

```json
{
    "name": "myapp/myapp",
    "description": "My OYTAPHP Application",
    "type": "project",
    "require": {
        "php": ">=8.0",
        "topthink/framework": "^8.0",
        "topthink/think-orm": "^3.0",
        "topthink/think-view": "^2.0",
        "vlucas/phpdotenv": "^5.0"
    },
    "require-dev": {
        "phpunit/phpunit": "^10.0"
    },
    "autoload": {
        "psr-4": {
            "app\\": "app/"
        },
        "files": [
            "app/common.php"
        ]
    },
    "autoload-dev": {
        "psr-4": {
            "tests\\": "tests/"
        }
    },
    "config": {
        "preferred-install": "dist",
        "sort-packages": true,
        "optimize-autoloader": true
    },
    "repositories": [
        {
            "type": "composer",
            "url": "https://mirrors.aliyun.com/composer/"
        }
    ]
}
```

### 32.9 依赖解析算法

OYTAPHP 使用基于**SAT 求解器**的依赖解析算法，与 Composer 2.x 兼容：

```
输入：composer.json 中的 require 列表
  │
  ├─ 1. 从 Packagist 获取每个包的元数据
  │
  ├─ 2. 构建版本约束图
  │     包A@^1.0 → [1.0, 1.1, 1.2, 2.0]
  │     包B@^2.0 → [2.0, 2.1]
  │     包B 依赖 包A@^1.1
  │
  ├─ 3. 求解最优版本组合
  │     → 包A@1.2 + 包B@2.1
  │
  ├─ 4. 递归解析传递依赖
  │
  ├─ 5. 冲突检测与解决
  │
  └─ 6. 生成 composer.lock
```

### 32.10 性能优化

| 优化项 | 说明 |
|--------|------|
| HTTP/2 连接池 | reqwest 复用连接，并发下载 |
| 元数据缓存 | 本地缓存 Packagist 元数据，减少网络请求 |
| 并行下载 | 多包并行下载（最多 10 并发） |
| 增量更新 | 仅下载变更的包，跳过已存在的包 |
| 锁文件 | composer.lock 确保可重复构建 |
| 镜像源 | 支持配置国内镜像（阿里云/腾讯云） |

### 32.11 镜像源配置

```bash
# 配置阿里云镜像
oyta composer config repo.packagist composer https://mirrors.aliyun.com/composer/

# 配置腾讯云镜像
oyta composer config repo.packagist composer https://mirrors.cloud.tencent.com/composer/

# 配置华为云镜像
oyta composer config repo.packagist composer https://repo.huaweicloud.com/repository/php/

# 查看当前配置
oyta composer config -l
```

### 32.12 新增依赖（Cargo.toml）

| 依赖 | 版本 | 用途 |
|------|------|------|
| `mago-composer` | 1.24.0 | composer.json 解析与建模 |
| `semver` | 1.0.28 | 语义版本号解析与比较 |
| `zip` | 8.5.1 | ZIP 压缩包解压 |
| `flate2` | 1.1.9 | gzip 解压 |
| `tar` | 0.4.45 | tar 归档解压 |

### 32.13 实现步骤

| 阶段 | 内容 | 优先级 |
|------|------|--------|
| **C1** | composer.json 解析（mago-composer）+ PSR-4 自动加载 | P1 |
| **C2** | Packagist API 客户端 + 版本解析 | P2 |
| **C3** | 依赖下载 + 解压 + vendor 目录管理 | P2 |
| **C4** | 依赖解析引擎（SAT Solver）+ composer.lock | P2 |
| **C5** | `oyta composer install/update/require/remove` 命令 | P2 |
| **C6** | 自动加载文件生成 + 符号表集成 | P2 |
| **C7** | ThinkPHP 核心包内置映射 | P3 |
| **C8** | 镜像源支持 + 缓存优化 + 并行下载 | P3 |

---

**OYTAPHP**：同等能力，更简更高效。ThinkPHP 8.0 的全部功能，Rust 级别的性能体验。
