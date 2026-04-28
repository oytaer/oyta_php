# OYTAPHP

**用 Rust 重写的 ThinkPHP 8.0 运行时**

---

## 项目简介

OYTAPHP 是一个 Rust 驱动的 PHP 框架运行时。完全按照 ThinkPHP 8.0 的写法编写 PHP 代码，底层由 Rust 二进制解析执行。无需安装 PHP、Nginx、Composer，一条命令即可启动完整的 Web 服务。

### 核心特性

- **100% 兼容**：项目目录结构与 ThinkPHP 8.0 完全一致
- **零依赖**：无需安装 PHP、Nginx、Composer，单二进制运行
- **高性能**：Rust 原生性能，比 PHP-FPM 响应更快
- **热重载**：修改 PHP 代码即时生效，无需重启

---

## 项目结构

```
oyta_php/
├── Cargo.toml                 # Workspace 配置
├── Cargo.lock                 # 依赖锁定文件
├── README.md                  # 项目说明文档
├── install.sh                 # Linux/macOS 安装脚本
├── install.ps1                # Windows 安装脚本
├── Cross.toml                 # 交叉编译配置
├── .cargo/config.toml         # Cargo 配置
├── .github/workflows/         # GitHub Actions 工作流
│   ├── ci.yml                 # CI 工作流
│   └── build-release.yml      # 发布构建工作流
├── docs/                      # 文档目录
└── oyta_core/                 # 核心运行时
    ├── Cargo.toml
    └── src/
        ├── main.rs            # 程序入口
        ├── cache/             # 缓存模块
        ├── cluster/           # 集群模块
        ├── command/           # 命令行模块
        ├── composer/          # Composer 模块
        ├── config/            # 配置模块
        ├── container/         # 容器模块
        ├── coroutine/         # 协程模块
        ├── database/          # 数据库模块
        ├── datetime/          # 日期时间模块
        ├── debug/             # 调试模块
        ├── embedded/          # 内嵌服务模块
        ├── env_loader/        # 环境变量加载器
        ├── error/             # 错误处理模块
        ├── eval/              # 动态执行模块
        ├── event/             # 事件模块
        ├── fastcgi/           # FastCGI 模块
        ├── gd/                # GD 图形处理模块
        ├── helpers/           # 辅助函数模块
        ├── http/              # HTTP 服务模块
        ├── http3/             # HTTP/3 模块
        ├── i18n/              # 国际化模块
        ├── interpreter/       # PHP 解释器模块
        ├── logging/           # 日志模块
        ├── mbstring/          # 多字节字符串模块
        ├── middleware/        # 中间件模块
        ├── microservice/      # 微服务模块
        ├── monitor/           # 监控模块
        ├── output/            # 输出缓冲模块
        ├── parser/            # PHP 解析器模块
        ├── phar/              # Phar 归档模块
        ├── project/           # 项目管理模块
        ├── reflection/        # 反射模块
        ├── router/            # 路由模块
        ├── sandbox/           # 沙箱模块
        ├── search/            # 搜索模块
        ├── security/          # 安全模块
        ├── serialize/         # 序列化模块
        ├── service/           # 服务模块
        ├── session/           # Session 模块
        ├── simd/              # SIMD 加速模块
        ├── storage/           # 存储模块
        ├── symbol_table/      # 符号表模块
        ├── template/          # 模板引擎模块
        ├── timer/             # 定时器模块
        ├── validator/         # 验证器模块
        ├── watcher/           # 文件监视模块
        └── xml/               # XML 处理模块
```

---

## 命令行工具

OYTAPHP 提供完整的命令行工具，与 ThinkPHP 8.0 的 `php think` 命令体系对应：

### 服务器命令

```bash
oyta run                              # 启动 HTTP 服务器 (默认 0.0.0.0:8000)
oyta run -H 127.0.0.1 -p 8080         # 指定地址和端口
oyta run --daemon                     # 守护进程模式
oyta run --debug                      # 调试模式
oyta fastcgi                          # 启动 FastCGI 服务器
```

### 项目初始化命令

```bash
oyta new myapp                        # 创建新项目
oyta new myapp -a admin               # 创建多应用项目
oyta init                             # 在当前目录初始化项目
oyta install                          # 安装 oyta 到系统 PATH
oyta uninstall                        # 卸载系统 PATH 中的 oyta
```

### 代码生成命令

```bash
oyta build demo                       # 构建应用
oyta make:controller Index            # 创建控制器
oyta make:controller Index --plain    # 创建空控制器
oyta make:controller Index --api      # 创建 API 控制器
oyta make:model User                  # 创建模型
oyta make:middleware Auth             # 创建中间件
oyta make:validate User               # 创建验证器
oyta make:event UserEvent             # 创建事件
oyta make:listener UserListener       # 创建监听器
oyta make:subscribe UserSubscribe     # 创建订阅者
oyta make:service UserService         # 创建服务类
oyta make:command CustomCommand       # 创建自定义命令
oyta make:job SendEmail               # 创建队列任务
oyta make:task CleanupTask            # 创建定时任务
```

### 路由命令

```bash
oyta route:list                       # 查看路由列表
oyta route:cache                      # 缓存路由
oyta route:clear                      # 清除路由缓存
```

### 缓存命令

```bash
oyta cache:clear                      # 清除缓存
oyta cache:clear --tag=user           # 清除指定标签缓存
oyta cache:forget key                 # 删除指定缓存
oyta cache:status                     # 查看缓存状态
```

### 队列命令

```bash
oyta queue:failed                     # 查看失败任务
oyta queue:retry 1                    # 重试指定任务
oyta queue:flush                      # 清除所有失败任务
oyta queue:status                     # 查看队列状态
```

### 配置命令

```bash
oyta config:cache                     # 缓存配置
oyta config:clear                     # 清除配置缓存
oyta config:get app.debug             # 查看配置值
oyta config:set app.debug true        # 设置配置值
```

### 优化命令

```bash
oyta optimize                         # 优化应用
oyta optimize:route                   # 生成路由缓存
oyta optimize:schema                  # 生成数据表字段缓存
oyta clear                            # 清除运行时缓存
```

### Composer 命令

```bash
oyta composer:install                 # 安装依赖
oyta composer:install --no-dev        # 不安装开发依赖
oyta composer:update                  # 更新依赖
oyta composer:require vendor/package  # 添加依赖
oyta composer:remove vendor/package   # 移除依赖
oyta composer:dump-autoload           # 重新生成自动加载
oyta composer:show                    # 查看已安装的包
oyta composer:outdated                # 查看过时的包
oyta composer:validate                # 验证 composer.json
oyta composer:config -l               # 查看 Composer 配置
oyta composer:clear-cache             # 清除 Composer 缓存
oyta composer:diagnose                # 诊断 Composer 问题
```

### 服务命令

```bash
oyta service:list                     # 查看服务列表
oyta service:show cache               # 查看服务详情
oyta service:discover                 # 自动注册扩展包服务
oyta vendor:publish                   # 发布扩展配置文件
```

### 信息命令

```bash
oyta version                          # 查看版本
oyta list                             # 列出所有命令
```

---

## PHP 门面系统

OYTAPHP 实现了完整的 ThinkPHP 8.0 门面系统：

| 门面 | 说明 | 方法 |
|------|------|------|
| **Cache** | 缓存门面 | get, set, delete, has, clear, increment, decrement, forever, remember |
| **Session** | 会话门面 | get, set, delete, has, clear, destroy |
| **Config** | 配置门面 | get, set, has |
| **Env** | 环境变量门面 | get, has |
| **Validate** | 验证门面 | check, make |
| **Log** | 日志门面 | info, error, warning, warn, debug |
| **Event** | 事件门面 | dispatch, listen, on |
| **Request** | 请求门面 | get, post, method, ip |
| **Response** | 响应门面 | json, redirect |
| **Crypt** | 加密门面 | encrypt, decrypt |
| **Coroutine** | 协程门面 | create, go, wait |
| **Timer** | 定时器门面 | after, every, cancel |
| **Search** | 搜索门面 | index, query |
| **WebSocket** | WebSocket门面 | join, leave, broadcast, broadcastToRoom, send, emit, onlineCount, getRooms, roomCount |
| **Queue** | 队列门面 | push, later, pop, len, size, clear, failedJobs, retry |
| **Upload** | 文件上传门面 | validate, move, save, exists, delete, url, path |
| **Route** | 路由门面 | get, post, put, delete, patch, any, group, resource, miss |
| **Lang** | 多语言门面 | get, set, has, locale, setLocale |
| **Hash** | 哈希门面 | make, check, needsRehash |
| **Csrf** | CSRF门面 | token, verify, refresh |
| **Date** | 日期门面 | now, format, parse, timestamp |
| **Middleware** | 中间件门面 | add, remove, has, clear |
| **Db** | 数据库门面 | query, execute, table, find, value, insertGetId |
| **Storage** | 存储门面 | exists, get, put, delete, copy, move, size, url, files, makeDirectory, deleteDirectory |
| **View** | 视图门面 | fetch, assign, display, has, get, clear |
| **App** | 应用容器门面 | make, bind, singleton, has, instance, alias |

---

## PHP 内置函数

### 字符串函数

```
strlen, strtolower, strtoupper, trim, ltrim, rtrim, substr, str_replace,
str_contains, str_starts_with, str_ends_with, strpos, str_repeat, str_pad,
explode, implode, join, strrev, str_shuffle, str_word_count, strrpos,
sprintf, printf, number_format, chr, ord, str_split, ucfirst, lcfirst,
ucwords, nl2br, htmlspecialchars, htmlentities, strip_tags
```

### 数组函数

```
count, sizeof, array_keys, array_values, array_merge, array_push, array_pop,
array_unshift, array_shift, array_map, array_filter, in_array, array_key_exists,
array_slice, array_splice, array_reverse, sort, usort, range, array_column,
array_unique, array_search, array_fill, array_combine, array_flip, array_sum,
array_product, array_rand, shuffle
```

### 编码函数

```
json_encode, json_decode, base64_encode, base64_decode, urlencode, urldecode
```

### 类型检测函数

```
is_null, is_array, is_string, is_int, is_float, is_bool, is_object,
is_callable, is_numeric, is_empty, isset, unset, gettype, settype
```

### 类型转换函数

```
intval, floatval, strval, boolval
```

### 数学函数

```
abs, max, min, floor, ceil, round
```

### 时间函数

```
time, date, microtime, strtotime
```

### 哈希函数

```
md5, sha1
```

### 调试函数

```
var_dump, print_r, json, dd, dump
```

### 类/对象函数

```
class_exists, method_exists, property_exists, get_class,
call_user_func, call_user_func_array, func_num_args
```

### 验证函数

```
filter_var, filter_input, validate_email, validate_url, validate_ip,
validate_regex, validate_length, validate_range, validate_required
```

### DateTime 函数

完整的 DateTime 类和相关函数支持

### mbstring 函数

多字节字符串处理函数

### GD 函数

完整的 GD 图形处理库支持

### XML 函数

XML 解析和处理函数

### Phar 函数

Phar 归档处理函数

### eval 函数

动态代码执行

---

## PHP 8.0+ 特性支持

- match 表达式
- 命名参数
- Nullsafe 运算符 (?->)
- Attribute 注解
- 构造器属性提升
- 联合类型
- Generator / yield
- Fiber 协程
- 枚举类型
- 只读属性

---

## 数据库支持

| 数据库 | 驱动 |
|--------|------|
| MySQL | sqlx + SeaORM |
| PostgreSQL | sqlx + SeaORM |
| SQLite | sqlx + SeaORM |

---

## 缓存驱动

| 驱动 | 实现 |
|------|------|
| file | tokio::fs |
| redis | deadpool-redis |
| memory | moka |

---

## 存储驱动

| 驱动 | 说明 |
|------|------|
| local | 本地文件系统 |
| s3 | AWS S3 / MinIO / Cloudflare R2 |
| aliyun | 阿里云 OSS |
| tencent | 腾讯云 COS |
| ftp | FTP/FTPS |

---

## 多平台支持

| 平台 | 架构 | 安装脚本 |
|------|------|----------|
| Linux | x86_64, aarch64 | `install.sh` |
| Windows | x86_64 | `install.ps1` |

### 安装方式

**Linux/macOS:**
```bash
chmod +x install.sh
./install.sh
```

**Windows (PowerShell):**
```powershell
Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned
.\install.ps1
```

安装完成后，`oyta` 二进制文件将自动添加到系统 PATH，可直接使用。

---

## 许可证

MIT License

---

## 相关资源

- [Rust 编程语言](https://www.rust-lang.org/)
- [PHP 8.0 文档](https://www.php.net/manual/zh/)
- [PHP 8.1 文档](https://www.php.net/manual/zh/)
