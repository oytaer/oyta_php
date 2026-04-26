//! CLI 命令行参数定义模块
//!
//! 使用 clap derive 模式定义 OYTAPHP 支持的所有命令和参数
//! 与 ThinkPHP 8.0 的 php think 命令体系一一对应
//! 所有命令统一使用 oyta 前缀

use clap::{Parser, Subcommand};

/// OYTAPHP 核心运行时命令行参数结构体
/// 通过 clap derive 宏自动生成参数解析代码
#[derive(Parser, Debug)]
#[command(
    name = "oyta",
    version,
    about = "OYTAPHP - 用 Rust 重写的 ThinkPHP 8.0 运行时",
    long_about = "OYTAPHP 是一个 Rust 驱动的 PHP 框架运行时。\n\
                  用户完全按照 ThinkPHP 8.0 的写法编写 PHP 代码，\n\
                  底层由 Rust 二进制解析执行，无需安装 PHP 运行环境。"
)]
pub struct Cli {
    /// 子命令：run / build / make:* / composer / version 等
    /// 如果不指定子命令，将显示帮助信息
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// 所有支持的子命令枚举
/// 每个变体对应一个具体的命令
/// 与 ThinkPHP 8.0 的 php think 命令一一对应
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 启动 HTTP 服务器（对应 php think run）
    /// 这是 OYTAPHP 的核心命令，启动内置的 Axum Web 服务器
    Run {
        /// 监听主机地址，默认绑定所有网卡 0.0.0.0
        #[arg(short = 'H', long, default_value = "0.0.0.0")]
        host: String,

        /// 监听端口号，默认 8000
        #[arg(short, long, default_value_t = 8000)]
        port: u16,

        /// 是否以守护进程模式运行
        /// 守护进程模式下，进程将在后台运行，日志输出到文件
        #[arg(long, default_value_t = false)]
        daemon: bool,

        /// 是否开启调试模式
        /// 调试模式下会显示详细错误信息、SQL 日志、Trace 面板
        /// 此参数会覆盖 .env 中的 APP_DEBUG 设置
        #[arg(long, default_value_t = false)]
        debug: bool,

        /// 工作进程数量
        /// 默认为 CPU 核心数，建议生产环境设置为 CPU 核心数的 1-2 倍
        #[arg(short = 'w', long)]
        workers: Option<usize>,
    },

    /// 启动 FastCGI 服务器（用于宝塔面板/Nginx 部署）
    /// 类似于 PHP-FPM 的工作方式，通过 Unix Socket 接收 Nginx 请求
    Fastcgi {
        /// Unix Socket 路径
        /// 默认为项目 runtime 目录下的 oyta.sock
        #[arg(short, long, default_value = "runtime/oyta.sock")]
        socket: String,

        /// 是否开启调试模式
        #[arg(long, default_value_t = false)]
        debug: bool,
    },

    /// 自动生成应用目录和文件（对应 php think build）
    /// 根据指定的应用名称创建完整的目录结构和默认文件
    Build {
        /// 应用名称，如 demo、admin
        /// 将在 app/ 目录下创建对应的子目录
        name: String,
    },

    /// 创建控制器（对应 php think make:controller）
    /// 在 app/controller/ 目录下生成控制器 PHP 文件
    #[command(name = "make:controller")]
    MakeController {
        /// 控制器名称，支持多级命名
        /// 如 Index、admin/Index、api/v1/User
        name: String,

        /// 创建空控制器（不包含默认的 index/hello 方法）
        #[arg(long, default_value_t = false)]
        plain: bool,

        /// 创建 API 控制器（继承 ApiController，不含 HTML 视图相关方法）
        #[arg(long, default_value_t = false)]
        api: bool,
    },

    /// 创建模型（对应 php think make:model）
    /// 在 app/model/ 目录下生成模型 PHP 文件
    #[command(name = "make:model")]
    MakeModel {
        /// 模型名称，如 User、admin/User
        name: String,
    },

    /// 创建中间件（对应 php think make:middleware）
    /// 在 app/middleware/ 目录下生成中间件 PHP 文件
    #[command(name = "make:middleware")]
    MakeMiddleware {
        /// 中间件名称，如 Auth、CORS
        name: String,
    },

    /// 创建验证器（对应 php think make:validate）
    /// 在 app/validate/ 目录下生成验证器 PHP 文件
    #[command(name = "make:validate")]
    MakeValidate {
        /// 验证器名称，如 User
        name: String,
    },

    /// 创建事件类（对应 php think make:event）
    /// 在 app/event/ 目录下生成事件 PHP 文件
    #[command(name = "make:event")]
    MakeEvent {
        /// 事件名称，如 UserLogin
        name: String,
    },

    /// 创建事件监听器（对应 php think make:listener）
    /// 在 app/listener/ 目录下生成监听器 PHP 文件
    #[command(name = "make:listener")]
    MakeListener {
        /// 监听器名称，如 UserLoginListener
        name: String,
    },

    /// 创建事件订阅者（对应 php think make:subscribe）
    /// 在 app/subscribe/ 目录下生成订阅者 PHP 文件
    #[command(name = "make:subscribe")]
    MakeSubscribe {
        /// 订阅者名称，如 UserSubscribe
        name: String,
    },

    /// 创建服务类（对应 php think make:service）
    /// 在 app/service/ 目录下生成服务类 PHP 文件
    #[command(name = "make:service")]
    MakeService {
        /// 服务类名称，如 UserService
        name: String,
    },

    /// 创建自定义命令（对应 php think make:command）
    /// 在 app/command/ 目录下生成命令类 PHP 文件
    #[command(name = "make:command")]
    MakeCommand {
        /// 命令名称，如 CustomCommand
        name: String,
    },

    /// 查看路由列表（对应 php think route:list）
    /// 显示所有已注册的路由，包括自动路由和定义路由
    #[command(name = "route:list")]
    RouteList,

    /// 缓存路由（对应 php think route:cache）
    /// 将路由表编译缓存，提升生产环境路由匹配性能
    #[command(name = "route:cache")]
    RouteCache,

    /// 优化应用（对应 php think optimize）
    /// 一键执行所有优化操作：路由缓存、配置缓存、字段缓存
    Optimize,

    /// 生成路由缓存
    /// 将路由定义编译为二进制缓存文件
    #[command(name = "optimize:route")]
    OptimizeRoute,

    /// 生成数据表字段缓存
    /// 缓存数据库表结构信息，避免运行时重复查询
    #[command(name = "optimize:schema")]
    OptimizeSchema,

    /// 清除运行时缓存（对应 php think clear）
    /// 删除 runtime/ 目录下的所有缓存文件
    Clear,

    /// 缓存配置（对应 php think config:cache）
    /// 将所有配置文件合并编译为缓存，提升配置读取性能
    #[command(name = "config:cache")]
    ConfigCache,

    /// Composer 依赖管理 - 安装依赖
    /// 根据 composer.lock 安装所有依赖包
    /// 如果 composer.lock 不存在，则根据 composer.json 解析并安装
    #[command(name = "composer:install")]
    ComposerInstall {
        /// 不安装开发依赖（require-dev 部分）
        #[arg(long, default_value_t = false)]
        no_dev: bool,

        /// 优化自动加载（生成完整 classmap）
        #[arg(short, long, default_value_t = false)]
        optimize: bool,
    },

    /// Composer 依赖管理 - 更新依赖
    /// 重新解析 composer.json 并更新依赖到最新兼容版本
    #[command(name = "composer:update")]
    ComposerUpdate {
        /// 只更新指定的包，不指定则更新所有包
        packages: Vec<String>,

        /// 不更新开发依赖
        #[arg(long, default_value_t = false)]
        no_dev: bool,
    },

    /// Composer 依赖管理 - 添加依赖
    /// 将包添加到 composer.json 的 require 中并安装
    #[command(name = "composer:require")]
    ComposerRequire {
        /// 包名，支持版本约束
        /// 如 vendor/package 或 vendor/package:^2.0
        packages: Vec<String>,

        /// 添加为开发依赖（require-dev）
        #[arg(long, default_value_t = false)]
        dev: bool,
    },

    /// Composer 依赖管理 - 移除依赖
    /// 从 composer.json 中移除包并卸载
    #[command(name = "composer:remove")]
    ComposerRemove {
        /// 要移除的包名列表
        packages: Vec<String>,
    },

    /// Composer 依赖管理 - 重新生成自动加载
    /// 扫描 vendor/ 目录，重新生成 PSR-4 映射文件
    #[command(name = "composer:dump-autoload")]
    ComposerDumpAutoload {
        /// 优化自动加载（生成完整 classmap 而非仅 PSR-4 规则）
        #[arg(short, long, default_value_t = false)]
        optimize: bool,
    },

    /// Composer 依赖管理 - 查看已安装的包
    /// 列出所有已安装的包及其版本
    #[command(name = "composer:show")]
    ComposerShow {
        /// 查看指定包的详细信息
        package: Option<String>,

        /// 只显示直接依赖（不显示传递依赖）
        #[arg(long, default_value_t = false)]
        direct: bool,
    },

    /// Composer 依赖管理 - 查看过时的包
    /// 检查已安装的包是否有新版本可用
    #[command(name = "composer:outdated")]
    ComposerOutdated,

    /// Composer 依赖管理 - 验证 composer.json
    /// 检查 composer.json 格式是否正确
    #[command(name = "composer:validate")]
    ComposerValidate,

    /// Composer 依赖管理 - 配置
    /// 读取或修改 composer.json 中的配置项
    #[command(name = "composer:config")]
    ComposerConfig {
        /// 配置参数
        /// 格式1: oyta composer config <key>（读取）
        /// 格式2: oyta composer config <key> <value>（设置）
        /// 如: oyta composer config repo.packagist composer https://mirrors.aliyun.com/composer/
        args: Vec<String>,

        /// 列出所有配置项
        #[arg(short, long, default_value_t = false)]
        list: bool,
    },

    /// Composer 依赖管理 - 清除缓存
    /// 清除本地缓存的包元数据和下载文件
    #[command(name = "composer:clear-cache")]
    ComposerClearCache,

    /// Composer 依赖管理 - 诊断问题
    /// 检查 Composer 运行环境是否正常
    #[command(name = "composer:diagnose")]
    ComposerDiagnose,

    /// 自动注册扩展包系统服务（对应 php think service:discover）
    /// 扫描 vendor/ 下所有包的服务提供者并自动注册
    #[command(name = "service:discover")]
    ServiceDiscover,

    /// 发布扩展配置文件（对应 php think vendor:publish）
    /// 将扩展包的配置文件复制到项目的 config/ 目录
    #[command(name = "vendor:publish")]
    VendorPublish {
        /// 指定要发布的扩展包名称
        /// 不指定则发布所有可发布的配置
        #[arg(long)]
        package: Option<String>,

        /// 强制覆盖已有的配置文件
        #[arg(long, default_value_t = false)]
        force: bool,
    },

    /// 查看版本信息（对应 php think version）
    /// 显示 OYTAPHP 版本号、编译信息、项目信息
    Version,

    /// 列出所有可用命令（对应 php think list）
    /// 显示所有已注册的命令及其简短说明
    List,
}

/// 解析命令行参数
/// 从 std::env::args() 获取参数并解析为 Cli 结构体
/// 如果参数格式不正确，clap 会自动显示错误信息并退出
///
/// # 返回值
/// 解析成功返回 Cli 结构体
pub fn parse() -> Cli {
    Cli::parse()
}

/// 获取命令的简短中文描述
/// 用于日志输出，方便追踪用户执行了哪个命令
///
/// # 参数
/// - `cmd`: 命令枚举引用
///
/// # 返回值
/// 命令的中文描述字符串
pub fn command_description(cmd: &Commands) -> String {
    match cmd {
        Commands::Run { host, port, .. } => format!("启动服务 {}:{}", host, port),
        Commands::Fastcgi { socket, .. } => format!("启动 FastCGI 服务: {}", socket),
        Commands::Build { name } => format!("构建应用: {}", name),
        Commands::MakeController { name, .. } => format!("创建控制器: {}", name),
        Commands::MakeModel { name } => format!("创建模型: {}", name),
        Commands::MakeMiddleware { name } => format!("创建中间件: {}", name),
        Commands::MakeValidate { name } => format!("创建验证器: {}", name),
        Commands::MakeEvent { name } => format!("创建事件: {}", name),
        Commands::MakeListener { name } => format!("创建监听器: {}", name),
        Commands::MakeSubscribe { name } => format!("创建订阅者: {}", name),
        Commands::MakeService { name } => format!("创建服务类: {}", name),
        Commands::MakeCommand { name } => format!("创建命令: {}", name),
        Commands::RouteList => "查看路由列表".to_string(),
        Commands::RouteCache => "缓存路由".to_string(),
        Commands::Optimize => "优化应用".to_string(),
        Commands::OptimizeRoute => "生成路由缓存".to_string(),
        Commands::OptimizeSchema => "生成数据表字段缓存".to_string(),
        Commands::Clear => "清除运行时缓存".to_string(),
        Commands::ConfigCache => "缓存配置".to_string(),
        Commands::ComposerInstall { .. } => "安装 Composer 依赖".to_string(),
        Commands::ComposerUpdate { .. } => "更新 Composer 依赖".to_string(),
        Commands::ComposerRequire { .. } => "添加 Composer 依赖".to_string(),
        Commands::ComposerRemove { .. } => "移除 Composer 依赖".to_string(),
        Commands::ComposerDumpAutoload { .. } => "重新生成自动加载".to_string(),
        Commands::ComposerShow { .. } => "查看已安装的包".to_string(),
        Commands::ComposerOutdated => "查看过时的包".to_string(),
        Commands::ComposerValidate => "验证 composer.json".to_string(),
        Commands::ComposerConfig { .. } => "配置 Composer".to_string(),
        Commands::ComposerClearCache => "清除 Composer 缓存".to_string(),
        Commands::ComposerDiagnose => "诊断 Composer 问题".to_string(),
        Commands::ServiceDiscover => "自动注册扩展包系统服务".to_string(),
        Commands::VendorPublish { .. } => "发布扩展配置文件".to_string(),
        Commands::Version => "查看版本".to_string(),
        Commands::List => "列出所有命令".to_string(),
    }
}
