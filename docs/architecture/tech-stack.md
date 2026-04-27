# 技术栈

## 核心依赖库

### PHP 语言层

| 依赖 | 版本 | 用途 |
|------|------|------|
| `php-rs-parser` | 0.9.2 | PHP 词法/语法分析，生成 CST |
| `php-ast` | 0.9.2 | PHP 抽象语法树构建与遍历 |

### Web 服务层

| 依赖 | 版本 | 用途 |
|------|------|------|
| `axum` | 0.8.9 | HTTP 服务器框架（JSON/WS/Multipart） |
| `axum-ws-broadcaster` | 0.11.0 | WebSocket 房间广播 |
| `tower` | 0.5.3 | 中间件抽象层 |
| `tower-http` | 0.6.8 | HTTP 中间件（静态文件/CORS/Trace） |
| `actix-cors` | 0.7.1 | CORS 跨域处理 |
| `reqwest` | 0.13.2 | HTTP 客户端 |

### 异步运行时

| 依赖 | 版本 | 用途 |
|------|------|------|
| `tokio` | 1.52.1 | 异步运行时（full features） |
| `tokio-util` | 0.7.18 | Tokio 辅助工具 |
| `futures` | 0.3.32 | 异步工具集 |
| `async-trait` | 0.1 | 异步 trait 支持 |

### 数据库层

| 依赖 | 版本 | 用途 |
|------|------|------|
| `sea-orm` | 1.1.20 | 异步 ORM（MySQL/PostgreSQL/SQLite） |
| `sqlx` | 0.8.6 | 底层数据库驱动 + Query Builder |
| `rust_decimal` | 1.41.0 | 高精度十进制数 |

### 缓存层

| 依赖 | 版本 | 用途 |
|------|------|------|
| `redis` | 1.2.0 | Redis 客户端（异步/连接池/JSON） |
| `deadpool-redis` | 0.23.0 | Redis 连接池管理 |
| `moka` | 0.12.15 | 高性能内存缓存（L1 同步+异步） |
| `cached` | 0.59.0 | 函数级缓存注解 |

### 配置层

| 依赖 | 版本 | 用途 |
|------|------|------|
| `figment` | 0.10.19 | 多源配置合并（ENV/TOML/YAML/PHP） |
| `dotenvy` | 0.15.7 | .env 文件加载 |

### 模板引擎

| 依赖 | 版本 | 用途 |
|------|------|------|
| `tera` | 1.20.0 | Jinja2 风格模板引擎 |

### CLI 工具

| 依赖 | 版本 | 用途 |
|------|------|------|
| `clap` | 4.6.1 | 命令行参数解析 |
| `dialoguer` | 0.12.0 | 交互式命令行提示 |

### 序列化

| 依赖 | 版本 | 用途 |
|------|------|------|
| `serde` | 1.0.228 | 通用序列化框架 |
| `serde_json` | 1.0.149 | JSON 序列化/反序列化 |
| `bincode` | 1.3.3 | Bincode 二进制序列化 |
| `rmp-serde` | 1.3.0 | MessagePack 序列化 |
| `serde_cbor` | 0.11.2 | CBOR 序列化 |
| `postcard` | 1.1.1 | Postcard 序列化 |

### 并发与同步

| 依赖 | 版本 | 用途 |
|------|------|------|
| `dashmap` | 6.1.0 | 并发 HashMap（全局符号表） |
| `parking_lot` | 0.12.5 | 高性能锁（RwLock/Mutex） |
| `once_cell` | 1.21.4 | 单次初始化（全局单例） |

### 加密与安全

| 依赖 | 版本 | 用途 |
|------|------|------|
| `sha2` | 0.10.8 | SHA-256/512 哈希 |
| `sha3` | 0.10.8 | SHA-3 哈希 |
| `hmac` | 0.12.1 | HMAC 消息认证 |
| `pbkdf2` | 0.12.2 | PBKDF2 密钥派生 |
| `aes-gcm` | 0.10.3 | AES-GCM 加密 |
| `base64` | 0.22.1 | Base64 编解码 |
| `hex` | 0.4.3 | 十六进制编解码 |
| `blake3` | 1.5 | BLAKE3 哈希 |
| `xxhash-rust` | 0.8 | xxHash 哈希 |

### 日志与追踪

| 依赖 | 版本 | 用途 |
|------|------|------|
| `tracing` | 0.1.44 | 结构化日志 |
| `tracing-subscriber` | 0.3.23 | 日志订阅器 |

### 文件监听

| 依赖 | 版本 | 用途 |
|------|------|------|
| `notify` | 8.2.0 | 文件系统变更监听 |

### 工具库

| 依赖 | 版本 | 用途 |
|------|------|------|
| `bytes` | 1.11.1 | 字节缓冲区 |
| `chrono` | 0.4.44 | 时间日期处理 |
| `chrono-tz` | 0.10.0 | 时区支持 |
| `uuid` | 1.23.1 | UUID 生成 |
| `anyhow` | 1.0.102 | 错误处理 |
| `thiserror` | 2.0.18 | 自定义错误类型 |
| `mime` | 0.3.17 | MIME 类型判断 |
| `regex` | 1.11.1 | 正则表达式 |
| `md5` | 0.7.0 | MD5 哈希 |
| `urlencoding` | 2.1.3 | URL 编解码 |
| `rand` | 0.9.1 | 随机数生成 |
| `byteorder` | 1.5.0 | 字节序处理 |
| `num_cpus` | 1.16 | CPU 核心数检测 |
| `crc32fast` | 1.4 | CRC32 校验 |
| `html-escape` | 0.2 | HTML 实体编解码 |

### Composer 支持

| 依赖 | 版本 | 用途 |
|------|------|------|
| `mago-composer` | 1.24.0 | composer.json 解析与建模 |
| `semver` | 1.0.28 | 语义版本号解析与比较 |
| `zip` | 8.5.1 | ZIP 压缩包解压 |
| `flate2` | 1.1.9 | gzip 解压 |
| `tar` | 0.4.45 | tar 归档解压 |

## 路由匹配

| 依赖 | 版本 | 用途 |
|------|------|------|
| `matchit` | 0.8.4 | 高性能路由匹配（Trie 实现） |

## 技术选型原则

### 1. 纯 Rust 实现

所有依赖库优先选择纯 Rust 实现，避免系统依赖：

- 使用 `rustls` 替代 `native-tls`/`openssl`
- 使用 `tokio` 异步运行时
- 使用纯 Rust 数据库驱动

### 2. 高性能优先

选择经过性能验证的库：

- `dashmap`：并发 HashMap，比标准 RwLock<HashMap> 快数倍
- `moka`：高性能缓存库，支持异步
- `parking_lot`：比标准库锁更高效

### 3. 零拷贝设计

尽量减少数据复制：

- `bytes`：零拷贝字节缓冲区
- `axum`：零拷贝响应体

### 4. 跨平台兼容

确保在 Linux/macOS/Windows 上都能编译运行：

- 避免平台特定 API
- 使用跨平台文件监听（notify）
- 使用跨平台网络库

## 版本兼容性

| 依赖 | Linux | macOS | Windows | 说明 |
|------|-------|-------|---------|------|
| `axum` | ✅ | ✅ | ✅ | 纯 Rust |
| `tokio` | ✅ | ✅ | ✅ | 纯 Rust |
| `sea-orm` | ✅ | ✅ | ✅ | 纯 Rust（rustls） |
| `sqlx` | ✅ | ✅ | ✅ | 纯 Rust（rustls） |
| `redis` | ✅ | ✅ | ✅ | 纯 Rust |
| `moka` | ✅ | ✅ | ✅ | 纯 Rust |
| `notify` | ✅ | ✅ | ✅ | 各平台不同后端 |
| `reqwest` | ✅ | ✅ | ✅ | 纯 Rust（rustls） |
