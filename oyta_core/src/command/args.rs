//! 命令行参数定义模块
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
    /// 子命令
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// 所有支持的子命令枚举
/// 按功能分组：服务器、代码生成、路由、缓存、队列、服务、配置、优化、Composer、信息
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ========================================================================
    // 服务器相关命令
    // ========================================================================
    
    /// 启动 HTTP 服务器（对应 php think run）
    /// 配置优先级：命令行参数 > config/app.php > .env 环境变量 > 默认值
    Run {
        /// 监听主机地址（可选，默认从 config/app.php 读取）
        #[arg(short = 'H', long)]
        host: Option<String>,

        /// 监听端口号（可选，默认从 config/app.php 读取）
        #[arg(short, long)]
        port: Option<u16>,

        /// 是否以守护进程模式运行
        #[arg(long, default_value_t = false)]
        daemon: bool,

        /// 是否开启调试模式（可选，默认从 config/app.php 读取）
        #[arg(long)]
        debug: Option<bool>,

        /// 工作进程数量
        #[arg(short = 'w', long)]
        workers: Option<usize>,
    },

    /// 启动 FastCGI 服务器（用于宝塔面板/Nginx 部署）
    Fastcgi {
        /// Unix Socket 路径
        #[arg(short, long, default_value = "runtime/oyta.sock")]
        socket: String,

        /// 是否开启调试模式
        #[arg(long, default_value_t = false)]
        debug: bool,
    },

    // ========================================================================
    // 项目初始化命令
    // ========================================================================

    /// 创建新项目（在当前目录下创建新的项目目录）
    New {
        /// 项目名称
        name: String,

        /// 应用名称（可选，默认为 index）
        #[arg(short = 'a', long)]
        app: Option<String>,
    },

    /// 初始化项目（在当前目录生成配置文件、环境变量、composer.json 等）
    Init {
        /// 应用名称（可选，默认为 index）
        name: Option<String>,
    },

    /// 安装 oyta 到系统 PATH（需要管理员权限）
    Install {
        /// 安装路径（默认为 /usr/local/bin）
        #[arg(short, long, default_value = "/usr/local/bin")]
        path: String,
    },

    /// 卸载系统 PATH 中的 oyta
    Uninstall {
        /// 安装路径（默认为 /usr/local/bin）
        #[arg(short, long, default_value = "/usr/local/bin")]
        path: String,
    },

    // ========================================================================
    // 代码生成命令
    // ========================================================================

    /// 自动生成应用目录和文件（对应 php think build）
    Build {
        /// 应用名称
        name: String,
    },

    /// 创建控制器（对应 php think make:controller）
    #[command(name = "make:controller")]
    MakeController {
        /// 控制器名称
        name: String,
        /// 创建空控制器
        #[arg(long, default_value_t = false)]
        plain: bool,
        /// 创建 API 控制器
        #[arg(long, default_value_t = false)]
        api: bool,
    },

    /// 创建模型（对应 php think make:model）
    #[command(name = "make:model")]
    MakeModel {
        /// 模型名称
        name: String,
    },

    /// 创建中间件（对应 php think make:middleware）
    #[command(name = "make:middleware")]
    MakeMiddleware {
        /// 中间件名称
        name: String,
    },

    /// 创建验证器（对应 php think make:validate）
    #[command(name = "make:validate")]
    MakeValidate {
        /// 验证器名称
        name: String,
    },

    /// 创建事件类（对应 php think make:event）
    #[command(name = "make:event")]
    MakeEvent {
        /// 事件名称
        name: String,
    },

    /// 创建事件监听器（对应 php think make:listener）
    #[command(name = "make:listener")]
    MakeListener {
        /// 监听器名称
        name: String,
    },

    /// 创建事件订阅者（对应 php think make:subscribe）
    #[command(name = "make:subscribe")]
    MakeSubscribe {
        /// 订阅者名称
        name: String,
    },

    /// 创建服务类（对应 php think make:service）
    #[command(name = "make:service")]
    MakeService {
        /// 服务类名称
        name: String,
    },

    /// 创建自定义命令（对应 php think make:command）
    #[command(name = "make:command")]
    MakeCommand {
        /// 命令名称
        name: String,
    },

    /// 创建队列任务类
    #[command(name = "make:job")]
    MakeJob {
        /// 任务名称
        name: String,
    },

    /// 创建定时任务类
    #[command(name = "make:task")]
    MakeTask {
        /// 任务名称
        name: String,
    },

    // ========================================================================
    // 路由相关命令
    // ========================================================================

    /// 查看路由列表（对应 php think route:list）
    #[command(name = "route:list")]
    RouteList,

    /// 缓存路由（对应 php think route:cache）
    #[command(name = "route:cache")]
    RouteCache,

    /// 清除路由缓存
    #[command(name = "route:clear")]
    RouteClear,

    // ========================================================================
    // 缓存管理命令（新增）
    // ========================================================================

    /// 清除缓存（对应 php think cache:clear）
    #[command(name = "cache:clear")]
    CacheClear {
        /// 指定缓存标签
        #[arg(short, long)]
        tag: Option<String>,
    },

    /// 删除指定缓存键（对应 php think cache:forget）
    #[command(name = "cache:forget")]
    CacheForget {
        /// 缓存键名
        key: String,
    },

    /// 查看缓存状态
    #[command(name = "cache:status")]
    CacheStatus,

    // ========================================================================
    // 队列管理命令（新增）
    // ========================================================================

    /// 查看失败任务列表（对应 php think queue:failed）
    #[command(name = "queue:failed")]
    QueueFailed {
        /// 指定队列名称
        #[arg(short, long)]
        queue: Option<String>,
    },

    /// 重试失败任务（对应 php think queue:retry）
    #[command(name = "queue:retry")]
    QueueRetry {
        /// 任务ID，不指定则重试所有失败任务
        id: Option<String>,
        /// 指定队列名称
        #[arg(short, long)]
        queue: Option<String>,
    },

    /// 清除所有失败任务（对应 php think queue:flush）
    #[command(name = "queue:flush")]
    QueueFlush {
        /// 指定队列名称
        #[arg(short, long)]
        queue: Option<String>,
    },

    /// 查看队列状态
    #[command(name = "queue:status")]
    QueueStatus {
        /// 指定队列名称
        #[arg(short, long)]
        queue: Option<String>,
    },

    // ========================================================================
    // 服务管理命令（新增）
    // ========================================================================

    /// 查看所有服务状态
    #[command(name = "service:list")]
    ServiceList,

    /// 查看指定服务详情
    #[command(name = "service:show")]
    ServiceShow {
        /// 服务名称
        name: String,
    },

    // ========================================================================
    // 配置管理命令（新增）
    // ========================================================================

    /// 缓存配置（对应 php think config:cache）
    #[command(name = "config:cache")]
    ConfigCache,

    /// 清除配置缓存
    #[command(name = "config:clear")]
    ConfigClear,

    /// 查看配置值
    #[command(name = "config:get")]
    ConfigGet {
        /// 配置键名，如 app.debug
        key: String,
    },

    /// 设置配置值
    #[command(name = "config:set")]
    ConfigSet {
        /// 配置键名
        key: String,
        /// 配置值
        value: String,
    },

    // ========================================================================
    // 优化相关命令
    // ========================================================================

    /// 优化应用（对应 php think optimize）
    Optimize,

    /// 生成路由缓存
    #[command(name = "optimize:route")]
    OptimizeRoute,

    /// 生成数据表字段缓存
    #[command(name = "optimize:schema")]
    OptimizeSchema,

    /// 清除运行时缓存（对应 php think clear）
    Clear,

    // ========================================================================
    // Composer 相关命令
    // ========================================================================

    /// 安装依赖
    #[command(name = "composer:install")]
    ComposerInstall {
        #[arg(long, default_value_t = false)]
        no_dev: bool,
        #[arg(short, long, default_value_t = false)]
        optimize: bool,
    },

    /// 更新依赖
    #[command(name = "composer:update")]
    ComposerUpdate {
        packages: Vec<String>,
        #[arg(long, default_value_t = false)]
        no_dev: bool,
    },

    /// 添加依赖
    #[command(name = "composer:require")]
    ComposerRequire {
        packages: Vec<String>,
        #[arg(long, default_value_t = false)]
        dev: bool,
    },

    /// 移除依赖
    #[command(name = "composer:remove")]
    ComposerRemove {
        packages: Vec<String>,
    },

    /// 重新生成自动加载
    #[command(name = "composer:dump-autoload")]
    ComposerDumpAutoload {
        #[arg(short, long, default_value_t = false)]
        optimize: bool,
    },

    /// 查看已安装的包
    #[command(name = "composer:show")]
    ComposerShow {
        package: Option<String>,
        #[arg(long, default_value_t = false)]
        direct: bool,
    },

    /// 查看过时的包
    #[command(name = "composer:outdated")]
    ComposerOutdated,

    /// 验证 composer.json
    #[command(name = "composer:validate")]
    ComposerValidate,

    /// 配置 Composer
    #[command(name = "composer:config")]
    ComposerConfig {
        args: Vec<String>,
        #[arg(short, long, default_value_t = false)]
        list: bool,
    },

    /// 清除 Composer 缓存
    #[command(name = "composer:clear-cache")]
    ComposerClearCache,

    /// 诊断 Composer 问题
    #[command(name = "composer:diagnose")]
    ComposerDiagnose,

    // ========================================================================
    // 扩展包相关命令
    // ========================================================================

    /// 自动注册扩展包系统服务
    #[command(name = "service:discover")]
    ServiceDiscover,

    /// 发布扩展配置文件
    #[command(name = "vendor:publish")]
    VendorPublish {
        #[arg(long)]
        package: Option<String>,
        #[arg(long, default_value_t = false)]
        force: bool,
    },

    // ========================================================================
    // 信息查看命令
    // ========================================================================

    /// 查看版本信息
    Version,

    /// 列出所有可用命令
    List,
}

/// 解析命令行参数
pub fn parse() -> Cli {
    Cli::parse()
}

/// 获取命令的简短中文描述
pub fn command_description(cmd: &Commands) -> String {
    match cmd {
        Commands::Run { host, port, .. } => {
            let h = host.as_deref().unwrap_or("0.0.0.0");
            let p = port.unwrap_or(8000);
            format!("启动服务 {}:{}", h, p)
        }
        Commands::Fastcgi { socket, .. } => format!("启动 FastCGI 服务: {}", socket),
        Commands::New { name, .. } => format!("创建新项目: {}", name),
        Commands::Init { name } => format!("初始化项目: {}", name.as_deref().unwrap_or("index")),
        Commands::Install { path, .. } => format!("安装 oyta 到: {}", path),
        Commands::Uninstall { path } => format!("从 {} 卸载 oyta", path),
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
        Commands::MakeJob { name } => format!("创建队列任务: {}", name),
        Commands::MakeTask { name } => format!("创建定时任务: {}", name),
        Commands::RouteList => "查看路由列表".to_string(),
        Commands::RouteCache => "缓存路由".to_string(),
        Commands::RouteClear => "清除路由缓存".to_string(),
        Commands::CacheClear { .. } => "清除缓存".to_string(),
        Commands::CacheForget { key } => format!("删除缓存: {}", key),
        Commands::CacheStatus => "查看缓存状态".to_string(),
        Commands::QueueFailed { .. } => "查看失败任务".to_string(),
        Commands::QueueRetry { .. } => "重试失败任务".to_string(),
        Commands::QueueFlush { .. } => "清除失败任务".to_string(),
        Commands::QueueStatus { .. } => "查看队列状态".to_string(),
        Commands::ServiceList => "查看服务列表".to_string(),
        Commands::ServiceShow { name } => format!("查看服务详情: {}", name),
        Commands::ConfigCache => "缓存配置".to_string(),
        Commands::ConfigClear => "清除配置缓存".to_string(),
        Commands::ConfigGet { key } => format!("查看配置: {}", key),
        Commands::ConfigSet { key, value } => format!("设置配置: {} = {}", key, value),
        Commands::Optimize => "优化应用".to_string(),
        Commands::OptimizeRoute => "生成路由缓存".to_string(),
        Commands::OptimizeSchema => "生成数据表字段缓存".to_string(),
        Commands::Clear => "清除运行时缓存".to_string(),
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
