# 项目目录结构

## 开发仓库结构

```
oyta_php/                          # 开发仓库
├── Cargo.toml                     # Workspace 根配置
├── Cargo.lock                     # 依赖锁定文件
├── Cross.toml                     # 交叉编译配置
├── README.md                      # 项目说明
├── ROADMAP.md                     # 开发路线图
│
├── .cargo/                        # Cargo 配置
│   └── config.toml               # 编译配置
│
├── .github/                       # GitHub 配置
│   └── workflows/                # CI/CD 工作流
│       ├── ci.yml                # 持续集成
│       └── build-release.yml     # 发布构建
│
├── docs/                          # 文档目录
│   ├── README.md                 # 文档首页
│   ├── architecture/             # 架构文档
│   ├── modules/                  # 模块文档
│   ├── api/                      # API 文档
│   └── guides/                   # 使用指南
│
├── oyta_core/                     # 核心运行时 crate
│   ├── Cargo.toml               # 包配置
│   └── src/                     # 源代码
│       ├── main.rs              # 二进制入口
│       │
│       ├── cli/                 # CLI 命令模块
│       │   ├── mod.rs
│       │   ├── args.rs          # 参数解析
│       │   ├── dispatch/        # 命令分发
│       │   └── optimizer/       # 优化命令
│       │
│       ├── interpreter/         # PHP 解释器
│       │   ├── mod.rs
│       │   ├── engine.rs        # 解释器引擎
│       │   ├── value.rs         # 值类型
│       │   ├── generator.rs     # Generator 支持
│       │   ├── fiber.rs         # Fiber 支持
│       │   ├── enum_type.rs     # 枚举类型
│       │   ├── readonly.rs      # 只读属性
│       │   └── php8_features/   # PHP 8.x 特性
│       │
│       ├── timer/               # 定时器模块
│       │   ├── mod.rs
│       │   ├── wheel.rs         # 时间轮
│       │   ├── level.rs         # 层级
│       │   ├── slot.rs          # 槽位
│       │   ├── task.rs          # 任务
│       │   ├── scheduler.rs     # 调度器
│       │   ├── callback.rs      # 回调
│       │   ├── stats.rs         # 统计
│       │   └── error.rs         # 错误
│       │
│       ├── monitor/             # 监控模块
│       │   ├── mod.rs
│       │   ├── collector.rs     # 收集器
│       │   ├── dashboard.rs     # 面板
│       │   └── metrics.rs       # 指标
│       │
│       ├── cache/               # 缓存模块
│       │   ├── mod.rs
│       │   ├── manager.rs       # 管理器
│       │   ├── driver.rs        # 驱动接口
│       │   ├── redis_driver.rs  # Redis 驱动
│       │   ├── multi_level.rs   # 多级缓存
│       │   ├── tag.rs           # 缓存标签
│       │   ├── facade.rs        # Facade
│       │   └── warmup/          # 缓存预热
│       │       ├── mod.rs
│       │       ├── analyzer.rs  # 访问分析
│       │       ├── warmer.rs    # 预热器
│       │       └── invalidator.rs # 失效器
│       │
│       ├── coroutine/           # 协程模块
│       │   ├── mod.rs
│       │   ├── scheduler.rs     # 调度器
│       │   ├── worker.rs        # 工作线程
│       │   ├── task.rs          # 任务
│       │   ├── channel.rs       # 通道
│       │   └── sync.rs          # 同步原语
│       │
│       ├── search/              # 搜索模块
│       │   ├── mod.rs
│       │   ├── index.rs         # 倒排索引
│       │   ├── query.rs         # 查询解析
│       │   ├── ranker.rs        # 排序
│       │   ├── tokenizer.rs     # 分词
│       │   └── storage.rs       # 存储
│       │
│       ├── simd/                # SIMD 模块
│       │   ├── mod.rs
│       │   ├── engine.rs        # 引擎
│       │   ├── string_ops.rs    # 字符串操作
│       │   ├── array_ops.rs     # 数组操作
│       │   ├── json_parse.rs    # JSON 解析
│       │   └── hash.rs          # 哈希
│       │
│       ├── serialize/           # 序列化模块
│       │   ├── mod.rs
│       │   ├── engine.rs        # 引擎
│       │   ├── bincode_format.rs
│       │   ├── msgpack_format.rs
│       │   ├── cbor_format.rs
│       │   ├── postcard_format.rs
│       │   └── stream.rs        # 流式处理
│       │
│       ├── sandbox/             # 沙箱模块
│       │   ├── mod.rs
│       │   ├── executor.rs      # 执行器
│       │   ├── resource.rs      # 资源管理
│       │   ├── filesystem.rs    # 文件系统
│       │   ├── network.rs       # 网络
│       │   └── monitor.rs       # 监控
│       │
│       ├── microservice/        # 微服务模块
│       │   ├── mod.rs
│       │   ├── registry.rs      # 服务注册
│       │   ├── loadbalancer.rs  # 负载均衡
│       │   ├── circuit_breaker.rs # 熔断器
│       │   ├── gateway.rs       # API 网关
│       │   └── config_center.rs # 配置中心
│       │
│       ├── http3/               # HTTP/3 模块
│       │   ├── mod.rs
│       │   ├── quic.rs          # QUIC 协议
│       │   ├── h3.rs            # HTTP/3
│       │   ├── stream.rs        # 流管理
│       │   └── connection.rs    # 连接管理
│       │
│       ├── database/            # 数据库模块
│       │   ├── mod.rs
│       │   ├── connection.rs    # 连接管理
│       │   ├── pool_trait.rs    # 连接池接口
│       │   ├── query_builder.rs # 查询构建
│       │   ├── executor.rs      # 执行器
│       │   ├── cluster.rs       # 集群
│       │   ├── read_write_split.rs # 读写分离
│       │   ├── postgres_pool.rs # PostgreSQL
│       │   └── sqlite_pool.rs   # SQLite
│       │
│       ├── router/              # 路由模块
│       │   ├── mod.rs
│       │   ├── registry.rs      # 路由注册
│       │   ├── route_parser.rs  # 路由解析
│       │   ├── annotation.rs    # 注解路由
│       │   ├── cache.rs         # 路由缓存
│       │   └── types.rs         # 类型定义
│       │
│       ├── middleware/          # 中间件模块
│       │   ├── mod.rs
│       │   ├── dispatcher.rs    # 分发器
│       │   ├── builtin.rs       # 内置中间件
│       │   └── types.rs         # 类型定义
│       │
│       ├── config/              # 配置模块
│       │   ├── mod.rs
│       │   ├── loader.rs        # 加载器
│       │   ├── store.rs         # 存储
│       │   └── facade.rs        # Facade
│       │
│       ├── container/           # 容器模块
│       │   ├── mod.rs
│       │   ├── container.rs     # IoC 容器
│       │   ├── service_provider.rs # 服务提供者
│       │   └── facade.rs        # Facade
│       │
│       ├── event/               # 事件模块
│       │   ├── mod.rs
│       │   ├── dispatcher.rs    # 分发器
│       │   └── types.rs         # 类型定义
│       │
│       ├── logging/             # 日志模块
│       │   ├── mod.rs
│       │   ├── setup.rs         # 初始化
│       │   ├── channel.rs       # 日志通道
│       │   └── file_writer.rs   # 文件写入
│       │
│       ├── security/            # 安全模块
│       │   ├── mod.rs
│       │   ├── crypto.rs        # 加密
│       │   ├── hash.rs          # 哈希
│       │   ├── csrf.rs          # CSRF
│       │   ├── xss.rs           # XSS 防护
│       │   └── sql_injection.rs # SQL 注入防护
│       │
│       ├── validator/           # 验证器模块
│       │   ├── mod.rs
│       │   ├── validator.rs     # 验证器
│       │   ├── rules.rs         # 验证规则
│       │   └── facade.rs        # Facade
│       │
│       ├── template/            # 模板模块
│       │   ├── mod.rs
│       │   ├── engine.rs        # 模板引擎
│       │   ├── tera_engine.rs   # Tera 引擎
│       │   └── facade.rs        # Facade
│       │
│       ├── parser/              # 解析器模块
│       │   ├── mod.rs
│       │   ├── config_parser.rs # 配置解析
│       │   ├── scanner.rs       # 扫描器
│       │   └── php_parser/      # PHP 解析器
│       │
│       ├── composer/            # Composer 模块
│       │   ├── mod.rs
│       │   ├── parser.rs        # 解析器
│       │   ├── resolver.rs      # 依赖解析
│       │   ├── downloader.rs    # 下载器
│       │   ├── autoload.rs      # 自动加载
│       │   ├── lock.rs          # 锁文件
│       │   ├── packagist.rs     # Packagist API
│       │   ├── extractor.rs     # 解压器
│       │   └── manager/         # 管理器
│       │
│       ├── embedded/            # 内嵌服务模块
│       │   ├── mod.rs
│       │   ├── websocket.rs     # WebSocket
│       │   ├── queue.rs         # 队列
│       │   ├── queue_redis.rs   # Redis 队列
│       │   ├── cron.rs          # Cron
│       │   ├── upload.rs        # 上传
│       │   └── timer/           # 定时器
│       │
│       ├── watcher/             # 文件监听模块
│       │   ├── mod.rs
│       │   ├── watcher.rs       # 监听器
│       │   └── hot_reload.rs    # 热重载
│       │
│       ├── fastcgi/             # FastCGI 模块
│       │   ├── mod.rs
│       │   ├── server.rs        # 服务器
│       │   ├── handler.rs       # 处理器
│       │   └── protocol.rs      # 协议
│       │
│       ├── env_loader/          # 环境变量模块
│       │   ├── mod.rs
│       │   └── loader.rs        # 加载器
│       │
│       ├── project/             # 项目模块
│       │   ├── mod.rs
│       │   ├── detector.rs      # 检测器
│       │   └── info.rs          # 项目信息
│       │
│       ├── symbol_table/        # 符号表模块
│       │   ├── mod.rs
│       │   ├── registry.rs      # 注册表
│       │   └── types.rs         # 类型定义
│       │
│       ├── debug/               # 调试模块
│       │   ├── mod.rs
│       │   └── trace.rs         # 追踪
│       │
│       └── helpers/             # 助手函数
│           └── mod.rs
│
└── oyta_cli/                     # 全局 CLI 工具 crate
    ├── Cargo.toml               # 包配置
    └── src/
        └── main.rs              # CLI 入口
```

## 用户项目结构

用户创建的项目目录结构与 ThinkPHP 8.0 完全一致：

```
myapp/                             # 用户项目根目录
├── app/                           # 应用目录
│   ├── controller/                # 控制器目录
│   │   └── Index.php              # IndexController
│   ├── model/                     # 模型目录
│   │   └── User.php               # User Model
│   ├── middleware/                # 中间件目录
│   │   └── Auth.php               # Auth Middleware
│   ├── validate/                  # 验证器目录
│   ├── event/                     # 事件监听目录
│   ├── service/                   # 服务类目录
│   └── common.php                 # 公共函数文件
│
├── config/                        # 配置目录
│   ├── app.php                    # 应用配置
│   ├── cache.php                  # 缓存配置
│   ├── database.php               # 数据库配置
│   ├── route.php                  # 路由配置
│   ├── session.php                # Session 配置
│   ├── log.php                    # 日志配置
│   ├── middleware.php             # 中间件定义
│   ├── event.php                  # 事件定义
│   └── provider.php               # 服务提供定义
│
├── route/                         # 路由定义目录
│   ├── route.php                  # 路由定义文件
│   └── api.php                    # API 路由定义
│
├── public/                        # WEB 目录
│   ├── index.php                  # 入口文件
│   ├── router.php                 # 快速测试文件
│   ├── static/                    # 静态资源
│   └── uploads/                   # 上传文件目录
│
├── view/                          # 视图目录
│   └── index/                     # 按控制器分组
│       └── index.html             # 模板文件
│
├── runtime/                       # 运行时目录
│   ├── cache/                     # 缓存文件
│   ├── log/                       # 日志文件
│   └── session/                   # Session 文件
│
├── vendor/                        # 框架核心目录
│   └── oyta_php/
│       └── bin/
│           └── oyta               # OYTAPHP 核心二进制
│
├── .env                           # 环境变量
├── .example.env                   # 环境变量示例
├── composer.json                  # Composer 定义文件
└── oyta                           # 软链接 → vendor/oyta_php/bin/oyta
```

## 关键文件说明

### 核心配置文件

| 文件 | 说明 |
|------|------|
| `Cargo.toml` | Workspace 根配置，定义成员和共享依赖 |
| `oyta_core/Cargo.toml` | 核心运行时包配置 |
| `oyta_cli/Cargo.toml` | CLI 工具包配置 |
| `Cross.toml` | 交叉编译配置 |

### 源代码入口

| 文件 | 说明 |
|------|------|
| `oyta_core/src/main.rs` | 核心二进制入口 |
| `oyta_cli/src/main.rs` | CLI 工具入口 |

### CI/CD 配置

| 文件 | 说明 |
|------|------|
| `.github/workflows/ci.yml` | 持续集成工作流 |
| `.github/workflows/build-release.yml` | 发布构建工作流 |
