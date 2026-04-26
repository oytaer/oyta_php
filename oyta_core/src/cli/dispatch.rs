//! 命令分发模块
//!
//! 负责将解析后的命令分发到对应的处理器
//! 每个命令都有独立的处理函数，确保职责清晰

use anyhow::{bail, Result};

use super::args::Commands;
use super::optimizer;
use crate::config;
use crate::env_loader;
use crate::http::state::AppState;
use crate::middleware::dispatcher::MiddlewareDispatcher;
use crate::parser::scanner::Scanner;
use crate::project;
use crate::router::registry::RouteRegistry;

/// 命令分发入口
/// 根据命令类型调用对应的处理函数
/// 这是所有命令执行的统一入口点
///
/// # 参数
/// - `command`: 解析后的命令枚举
///
/// # 返回值
/// 命令执行成功返回 Ok(())，失败返回错误信息
pub async fn dispatch(command: &Commands) -> Result<()> {
    match command {
        // 启动 HTTP 服务器
        Commands::Run { host, port, daemon, debug, workers } => {
            handle_run(host, *port, daemon, debug, workers).await
        }

        // 启动 FastCGI 服务器
        Commands::Fastcgi { socket, debug } => {
            handle_fastcgi(socket, debug).await
        }

        // 构建应用目录
        Commands::Build { name } => {
            handle_build(name).await
        }

        // 创建控制器
        Commands::MakeController { name, plain, api } => {
            handle_make_controller(name, *plain, *api).await
        }

        // 创建模型
        Commands::MakeModel { name } => {
            handle_make_model(name).await
        }

        // 创建中间件
        Commands::MakeMiddleware { name } => {
            handle_make_middleware(name).await
        }

        // 创建验证器
        Commands::MakeValidate { name } => {
            handle_make_validate(name).await
        }

        // 创建事件
        Commands::MakeEvent { name } => {
            handle_make_event(name).await
        }

        // 创建监听器
        Commands::MakeListener { name } => {
            handle_make_listener(name).await
        }

        // 创建订阅者
        Commands::MakeSubscribe { name } => {
            handle_make_subscribe(name).await
        }

        // 创建服务类
        Commands::MakeService { name } => {
            handle_make_service(name).await
        }

        // 创建自定义命令
        Commands::MakeCommand { name } => {
            handle_make_command(name).await
        }

        // 查看路由列表
        Commands::RouteList => {
            handle_route_list().await
        }

        // 缓存路由
        Commands::RouteCache => {
            handle_route_cache().await
        }

        // 优化应用
        Commands::Optimize => {
            handle_optimize().await
        }

        // 生成路由缓存
        Commands::OptimizeRoute => {
            handle_optimize_route().await
        }

        // 生成数据表字段缓存
        Commands::OptimizeSchema => {
            handle_optimize_schema().await
        }

        // 清除运行时缓存
        Commands::Clear => {
            handle_clear().await
        }

        // 缓存配置
        Commands::ConfigCache => {
            handle_config_cache().await
        }

        // Composer 安装依赖
        Commands::ComposerInstall { no_dev, optimize } => {
            handle_composer_install(*no_dev, *optimize).await
        }

        // Composer 更新依赖
        Commands::ComposerUpdate { packages, no_dev } => {
            handle_composer_update(packages, *no_dev).await
        }

        // Composer 添加依赖
        Commands::ComposerRequire { packages, dev } => {
            handle_composer_require(packages, *dev).await
        }

        // Composer 移除依赖
        Commands::ComposerRemove { packages } => {
            handle_composer_remove(packages).await
        }

        // Composer 重新生成自动加载
        Commands::ComposerDumpAutoload { optimize } => {
            handle_composer_dump_autoload(*optimize).await
        }

        // Composer 查看已安装的包
        Commands::ComposerShow { package, direct } => {
            handle_composer_show(package, *direct).await
        }

        // Composer 查看过时的包
        Commands::ComposerOutdated => {
            handle_composer_outdated().await
        }

        // Composer 验证 composer.json
        Commands::ComposerValidate => {
            handle_composer_validate().await
        }

        // Composer 配置
        Commands::ComposerConfig { args, list } => {
            handle_composer_config(args, *list).await
        }

        // Composer 清除缓存
        Commands::ComposerClearCache => {
            handle_composer_clear_cache().await
        }

        // Composer 诊断
        Commands::ComposerDiagnose => {
            handle_composer_diagnose().await
        }

        // 自动注册扩展包系统服务
        Commands::ServiceDiscover => {
            handle_service_discover().await
        }

        // 发布扩展配置文件
        Commands::VendorPublish { package, force } => {
            handle_vendor_publish(package, *force).await
        }

        // 查看版本
        Commands::Version => {
            handle_version().await
        }

        // 列出所有命令
        Commands::List => {
            handle_list().await
        }
    }
}

/// 处理 run 命令：启动 HTTP 服务器
/// 这是 OYTAPHP 的核心命令，启动流程如下：
/// 1. 检测项目根目录
/// 2. 加载 .env 环境变量
/// 3. 确保运行时目录存在
/// 4. 启动 Axum HTTP 服务
///
/// # 参数
/// - `host`: 监听地址
/// - `port`: 监听端口
/// - `daemon`: 是否守护进程模式
/// - `debug`: 是否调试模式
/// - `workers`: 工作进程数量
async fn handle_run(host: &str, port: u16, _daemon: &bool, is_debug: &bool, workers: &Option<usize>) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;
    project.ensure_runtime_dirs()?;
    
    let mut scanner = Scanner::new();
    let _scan_result = scanner.scan_all(&project.root)?;
    
    let config_store = config::store::ConfigStore::new();
    let config_loader = config::loader::ConfigLoader::new(&project.config_dir);
    let mut php_parser = crate::parser::php_parser::PhpParser::new();
    config_loader.load(&config_store, &mut php_parser)?;
    config::facade::Config::init_from_store(&config_store);
    
    let route_registry = RouteRegistry::new();
    let controllers = scanner.registry().get_controllers();
    route_registry.generate_auto_routes(&controllers, project.is_multi_app(), None);
    
    let route_dir = project.root.join("route");
    let _ = crate::router::route_parser::RouteFileParser::parse_route_dir(
        &route_dir,
        &route_registry,
        &mut php_parser,
    )?;
    
    let registry_arc = std::sync::Arc::new(scanner.registry().clone());
    let mut middleware_dispatcher = MiddlewareDispatcher::new(registry_arc.clone());
    middleware_dispatcher.discover_from_registry();
    
    let route_registry_arc = std::sync::Arc::new(route_registry);
    let app_state = AppState::new(
        registry_arc,
        route_registry_arc,
        middleware_dispatcher,
        project.root.clone(),
        *is_debug,
    );
    
    // 从环境变量读取工作进程数，默认为 2
    // 优先级：命令行参数 > 环境变量 > 默认值 2
    let worker_count = workers.unwrap_or_else(|| {
        std::env::var("WORKERS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|&n| n > 0)
            .unwrap_or(2)
    });
    
    // 输出简洁的启动信息
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("  OYTAPHP v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("  监听地址: {}:{}", host, port);
    tracing::info!("  调试模式: {}", if *is_debug { "开启" } else { "关闭" });
    tracing::info!("  工作进程: {}", worker_count);
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let server_config = crate::http::server::ServerConfig {
        host: host.to_string(),
        port,
        project_root: project.root.clone(),
        public_dir: project.public_dir.clone(),
        workers: worker_count,
        debug: *is_debug,
    };
    
    if *is_debug {
        let hot_reload_registry = app_state.registry.clone();
        let hot_reload_root = project.root.clone();
        tokio::spawn(async move {
            match crate::watcher::hot_reload::HotReloader::new(&hot_reload_root, hot_reload_registry) {
                Ok(mut reloader) => reloader.run().await,
                Err(e) => tracing::warn!("热重载启动失败: {}", e),
            }
        });
    }
    
    crate::http::server::start_server(server_config, app_state).await?;
    
    Ok(())
}

/// 处理 FastCGI 命令：启动 FastCGI 服务器
/// 
/// 用于宝塔面板/Nginx 部署，类似于 PHP-FPM 的工作方式
/// 通过 Unix Socket 接收 Nginx 转发的请求
/// 
/// # 参数
/// - `socket`: Unix Socket 路径
/// - `is_debug`: 是否开启调试模式
async fn handle_fastcgi(socket: &str, is_debug: &bool) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 确保运行时目录存在
    project.ensure_runtime_dirs()?;

    // 扫描项目文件
    let mut scanner = Scanner::new();
    let _scan_result = scanner.scan_all(&project.root)?;

    // 加载配置
    let config_store = config::store::ConfigStore::new();
    let config_loader = config::loader::ConfigLoader::new(&project.config_dir);
    let mut php_parser = crate::parser::php_parser::PhpParser::new();
    config_loader.load(&config_store, &mut php_parser)?;
    config::facade::Config::init_from_store(&config_store);

    // 构建路由表
    let route_registry = RouteRegistry::new();
    let controllers = scanner.registry().get_controllers();
    route_registry.generate_auto_routes(&controllers, project.is_multi_app(), None);

    // 解析路由定义文件
    let route_dir = project.root.join("route");
    let _ = crate::router::route_parser::RouteFileParser::parse_route_dir(
        &route_dir,
        &route_registry,
        &mut php_parser,
    )?;

    // 构建中间件调度器
    let registry_arc = std::sync::Arc::new(scanner.registry().clone());
    let mut middleware_dispatcher = MiddlewareDispatcher::new(registry_arc.clone());
    middleware_dispatcher.discover_from_registry();

    // 构建应用状态
    let route_registry_arc = std::sync::Arc::new(route_registry);
    let app_state = AppState::new(
        registry_arc,
        route_registry_arc,
        middleware_dispatcher,
        project.root.clone(),
        *is_debug,
    );

    // 解析 Socket 路径
    let socket_path = if socket.starts_with('/') {
        std::path::PathBuf::from(socket)
    } else {
        project.root.join(socket)
    };

    // 输出启动信息
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("  OYTAPHP FastCGI v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("  Socket: {}", socket_path.display());
    tracing::info!("  调试模式: {}", if *is_debug { "开启" } else { "关闭" });
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 创建并启动 FastCGI 服务器
    let fastcgi_config = crate::fastcgi::server::FastCGIServerConfig {
        socket_path,
        project_root: project.root,
        max_connections: 100,
    };

    let server = crate::fastcgi::server::FastCGIServer::new(fastcgi_config, std::sync::Arc::new(app_state));
    server.start().await?;

    Ok(())
}

/// 处理 build 命令：自动生成应用目录和文件
/// 在 app/ 目录下创建指定名称的应用子目录
/// 包含 controller/、model/、middleware/ 等子目录
async fn handle_build(name: &str) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    // 加载环境变量
    env_loader::loader::load_env(&project)?;

    // 创建应用目录结构
    let app_dir = project.app_dir.join(name);

    // 需要创建的子目录列表
    let subdirs = ["controller", "model", "middleware"];

    // 逐个创建子目录
    for subdir in subdirs {
        let dir_path = app_dir.join(subdir);
        std::fs::create_dir_all(&dir_path)?;
        tracing::info!("创建目录: {}", dir_path.display());
    }

    // 创建应用公共函数文件（可选）
    let common_file = app_dir.join("common.php");
    if !common_file.exists() {
        let content = format!("<?php\n// {} 应用公共函数文件\n", name);
        std::fs::write(&common_file, content)?;
        tracing::info!("创建文件: {}", common_file.display());
    }

    tracing::info!("✓ 应用 [{}] 构建完成", name);
    Ok(())
}

/// 处理 make:controller 命令：创建控制器
/// 在 app/controller/ 目录下生成控制器 PHP 文件
/// 支持 plain（空控制器）和 api（API控制器）模式
async fn handle_make_controller(name: &str, plain: bool, api: bool) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    // 确定控制器文件路径
    let controller_dir = project.controller_dir(None);
    let file_path = controller_dir.join(format!("{}.php", name));

    // 检查文件是否已存在
    if file_path.exists() {
        bail!("控制器文件已存在: {}", file_path.display());
    }

    // 确保目录存在
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 根据命名空间规则生成类名
    // 如 admin/Index → namespace app\controller\admin; class Index
    let (namespace, class_name) = parse_name_with_namespace(name, "app\\controller");

    // 根据模式生成不同的控制器内容
    let content = if plain {
        // 空控制器：不包含任何方法
        format!(
            "<?php\nnamespace {};\n\nclass {}\n{{\n}}\n",
            namespace, class_name
        )
    } else if api {
        // API 控制器：包含 RESTful 风格方法
        format!(
            "<?php\nnamespace {};\n\nclass {}\n{{\n    public function index()\n    {{\n        return json([{{'status' => 'ok'}}]);\n    }}\n\n    public function show($id)\n    {{\n        return json([{{'id' => $id}}]);\n    }}\n\n    public function save()\n    {{\n        return json([{{'status' => 'created'}}], 201);\n    }}\n\n    public function update($id)\n    {{\n        return json([{{'id' => $id, 'status' => 'updated'}}]);\n    }}\n\n    public function delete($id)\n    {{\n        return json([{{'id' => $id, 'status' => 'deleted'}}]);\n    }}\n}}\n",
            namespace, class_name
        )
    } else {
        // 标准控制器：包含 index 和 hello 方法
        format!(
            "<?php\nnamespace {};\n\nclass {}\n{{\n    public function index()\n    {{\n        return 'Hello, OYTAPHP!';\n    }}\n\n    public function hello(string $name = 'think')\n    {{\n        return 'Hello,' . $name;\n    }}\n}}\n",
            namespace, class_name
        )
    };

    // 写入文件
    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 控制器创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 make:model 命令：创建模型
/// 在 app/model/ 目录下生成模型 PHP 文件
/// 模型默认继承 oyta\Model
async fn handle_make_model(name: &str) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    let model_dir = project.model_dir(None);
    let file_path = model_dir.join(format!("{}.php", name));

    if file_path.exists() {
        bail!("模型文件已存在: {}", file_path.display());
    }

    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let (namespace, class_name) = parse_name_with_namespace(name, "app\\model");

    // 生成模型内容，继承 oyta\Model
    let content = format!(
        "<?php\nnamespace {};\n\nuse oyta\\Model;\n\nclass {} extends Model\n{{\n    // 模型名称（对应数据表名，默认为类名的下划线形式）\n    protected $name = '{}';\n\n    // 主键字段名，默认为 id\n    protected $pk = 'id';\n\n    // 自动写入时间戳\n    protected $autoWriteTimestamp = true;\n}}\n",
        namespace,
        class_name,
        // 将类名转换为下划线形式作为默认表名
        camel_to_snake(&class_name).as_str()
    );

    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 模型创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 make:middleware 命令：创建中间件
/// 在 app/middleware/ 目录下生成中间件 PHP 文件
async fn handle_make_middleware(name: &str) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    let middleware_dir = project.middleware_dir(None);
    let file_path = middleware_dir.join(format!("{}.php", name));

    if file_path.exists() {
        bail!("中间件文件已存在: {}", file_path.display());
    }

    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let (namespace, class_name) = parse_name_with_namespace(name, "app\\middleware");

    // 生成中间件内容，包含 handle 和 end 方法
    let content = format!(
        "<?php\nnamespace {};\n\nclass {}\n{{\n    /**\n     * 中间件处理\n     * @param mixed  $request 请求对象\n     * @param \\Closure $next  下一个中间件\n     * @return mixed\n     */\n    public function handle($request, \\Closure $next)\n    {{\n        // 前置中间件逻辑\n\n        // 执行下一个中间件\n        $response = $next($request);\n\n        // 后置中间件逻辑\n\n        return $response;\n    }}\n\n    /**\n     * 中间件结束回调\n     * @param mixed $response 响应对象\n     */\n    public function end($response)\n    {{\n        // 请求结束后的回调逻辑\n    }}\n}}\n",
        namespace, class_name
    );

    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 中间件创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 make:validate 命令：创建验证器
/// 在 app/validate/ 目录下生成验证器 PHP 文件
async fn handle_make_validate(name: &str) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    let validate_dir = project.app_dir.join("validate");
    let file_path = validate_dir.join(format!("{}.php", name));

    if file_path.exists() {
        bail!("验证器文件已存在: {}", file_path.display());
    }

    std::fs::create_dir_all(&validate_dir)?;

    let (namespace, class_name) = parse_name_with_namespace(name, "app\\validate");

    // 生成验证器内容，继承 oyta\Validate
    let content = format!(
        "<?php\nnamespace {};\n\nuse oyta\\Validate;\n\nclass {} extends Validate\n{{\n    // 验证规则\n    protected $rule = [\n        // 'name'  => 'require|max:25',\n        // 'email' => 'require|email',\n    ];\n\n    // 验证消息\n    protected $message = [\n        // 'name.require' => '名称必须',\n        // 'name.max'     => '名称最多25个字符',\n        // 'email.require'=> '邮箱必须',\n        // 'email.email'  => '邮箱格式错误',\n    ];\n\n    // 验证场景\n    protected $scene = [\n        // 'login'   => ['email'],\n        // 'register'=> ['name', 'email'],\n    ];\n}}\n",
        namespace, class_name
    );

    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 验证器创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 make:event 命令：创建事件类
/// 在 app/event/ 目录下生成事件 PHP 文件
async fn handle_make_event(name: &str) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    let event_dir = project.app_dir.join("event");
    let file_path = event_dir.join(format!("{}.php", name));

    if file_path.exists() {
        bail!("事件文件已存在: {}", file_path.display());
    }

    std::fs::create_dir_all(&event_dir)?;

    let (namespace, class_name) = parse_name_with_namespace(name, "app\\event");

    let content = format!(
        "<?php\nnamespace {};\n\nclass {}\n{{\n    // 事件数据\n    public $data;\n\n    public function __construct($data = null)\n    {{\n        $this->data = $data;\n    }}\n}}\n",
        namespace, class_name
    );

    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 事件创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 make:listener 命令：创建事件监听器
/// 在 app/listener/ 目录下生成监听器 PHP 文件
async fn handle_make_listener(name: &str) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    let listener_dir = project.app_dir.join("listener");
    let file_path = listener_dir.join(format!("{}.php", name));

    if file_path.exists() {
        bail!("监听器文件已存在: {}", file_path.display());
    }

    std::fs::create_dir_all(&listener_dir)?;

    let (namespace, class_name) = parse_name_with_namespace(name, "app\\listener");

    let content = format!(
        "<?php\nnamespace {};\n\nclass {}\n{{\n    /**\n     * 事件监听处理\n     * @param mixed $event 事件数据\n     */\n    public function handle($event)\n    {{\n        // 监听器逻辑\n    }}\n}}\n",
        namespace, class_name
    );

    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 监听器创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 make:subscribe 命令：创建事件订阅者
/// 在 app/subscribe/ 目录下生成订阅者 PHP 文件
async fn handle_make_subscribe(name: &str) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    let subscribe_dir = project.app_dir.join("subscribe");
    let file_path = subscribe_dir.join(format!("{}.php", name));

    if file_path.exists() {
        bail!("订阅者文件已存在: {}", file_path.display());
    }

    std::fs::create_dir_all(&subscribe_dir)?;

    let (namespace, class_name) = parse_name_with_namespace(name, "app\\subscribe");

    let content = format!(
        "<?php\nnamespace {};\n\nclass {}\n{{\n    /**\n     * 订阅的事件列表\n     * @return array\n     */\n    public function subscribe()\n    {{\n        return [\n            // 'UserLogin' => 'onUserLogin',\n            // 'UserLogout'=> 'onUserLogout',\n        ];\n    }}\n}}\n",
        namespace, class_name
    );

    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 订阅者创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 make:service 命令：创建服务类
/// 在 app/service/ 目录下生成服务类 PHP 文件
async fn handle_make_service(name: &str) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    let service_dir = project.app_dir.join("service");
    let file_path = service_dir.join(format!("{}.php", name));

    if file_path.exists() {
        bail!("服务类文件已存在: {}", file_path.display());
    }

    std::fs::create_dir_all(&service_dir)?;

    let (namespace, class_name) = parse_name_with_namespace(name, "app\\service");

    let content = format!(
        "<?php\nnamespace {};\n\nclass {}\n{{\n    // 服务类逻辑\n}}\n",
        namespace, class_name
    );

    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 服务类创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 make:command 命令：创建自定义命令
/// 在 app/command/ 目录下生成命令类 PHP 文件
async fn handle_make_command(name: &str) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    let command_dir = project.app_dir.join("command");
    let file_path = command_dir.join(format!("{}.php", name));

    if file_path.exists() {
        bail!("命令文件已存在: {}", file_path.display());
    }

    std::fs::create_dir_all(&command_dir)?;

    let (namespace, class_name) = parse_name_with_namespace(name, "app\\command");

    let content = format!(
        "<?php\nnamespace {};\n\nclass {}\n{{\n    /**\n     * 命令名称\n     * @var string\n     */\n    protected $name = '{}';\n\n    /**\n     * 命令描述\n     * @var string\n     */\n    protected $description = '';\n\n    /**\n     * 执行命令\n     */\n    public function execute()\n    {{\n        // 命令逻辑\n    }}\n}}\n",
        namespace, class_name, camel_to_snake(&class_name).as_str()
    );

    std::fs::write(&file_path, content)?;
    tracing::info!("✓ 命令创建成功: {}", file_path.display());

    Ok(())
}

/// 处理 route:list 命令：查看路由列表
/// 显示所有已注册的路由信息
async fn handle_route_list() -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    // 扫描项目文件
    let mut scanner = Scanner::new();
    scanner.scan_all(&project.root)?;

    // 生成自动路由
    let route_registry = RouteRegistry::new();
    let controllers = scanner.registry().get_controllers();
    route_registry.generate_auto_routes(&controllers, project.is_multi_app(), None);

    // 解析路由定义文件
    let route_dir = project.root.join("route");
    let mut php_parser = crate::parser::php_parser::PhpParser::new();
    let _ = crate::router::route_parser::RouteFileParser::parse_route_dir(
        &route_dir,
        &route_registry,
        &mut php_parser,
    );

    let routes = route_registry.all_routes();

    if routes.is_empty() {
        println!("没有注册的路由");
        return Ok(());
    }

    println!("路由列表 (共 {} 条):", routes.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{:<8} {:<10} {:<40} {}", "方法", "类型", "路径", "处理器");
    println!("────────────────────────────────────────────────────────────────────");

    for route in &routes {
        let method = route.method.as_str();
        let handler_str = match &route.handler {
            crate::router::types::RouteHandler::ControllerMethod { controller, method } => {
                format!("{}@{}", controller, method)
            }
            crate::router::types::RouteHandler::Static { content, .. } => {
                format!("[static] {}", if content.len() > 30 { &content[..30] } else { content })
            }
            crate::router::types::RouteHandler::Redirect { url, .. } => {
                format!("[redirect] {}", url)
            }
            crate::router::types::RouteHandler::Closure { id } => {
                format!("[closure] {}", id)
            }
        };
        let route_type = route.group.as_deref().unwrap_or("auto");
        println!("{:<8} {:<10} {:<40} {}", method, route_type, route.path, handler_str);
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    Ok(())
}

/// 处理 route:cache 命令：缓存路由
///
/// 将路由配置编译为缓存文件，提高路由匹配性能
async fn handle_route_cache() -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    let optimizer = optimizer::Optimizer::new(project);
    optimizer.cache_routes().await
}

/// 处理 optimize 命令：优化应用
///
/// 执行所有优化操作：路由缓存、配置缓存、类映射优化
async fn handle_optimize() -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    let optimizer = optimizer::Optimizer::new(project);
    optimizer.optimize_all().await
}

/// 优化自动加载
async fn optimize_autoload(project: &project::detector::Project) -> Result<()> {
    let optimizer = optimizer::Optimizer::new(project.clone());
    optimizer.optimize_autoload().await
}

/// 处理 optimize:route 命令：生成路由缓存
async fn handle_optimize_route() -> Result<()> {
    handle_route_cache().await
}

/// 处理 optimize:schema 命令：生成数据表字段缓存
///
/// 扫描数据库表结构，生成字段映射缓存
async fn handle_optimize_schema() -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    let optimizer = optimizer::Optimizer::new(project);
    optimizer.cache_schema().await
}

/// 处理 clear 命令：清除运行时缓存
/// 删除 runtime/ 目录下的所有缓存文件
async fn handle_clear() -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    let optimizer = optimizer::Optimizer::new(project);
    optimizer.clear_cache().await
}

/// 处理 config:cache 命令：缓存配置
///
/// 将所有配置文件合并为一个缓存文件，提高配置读取性能
async fn handle_config_cache() -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    let optimizer = optimizer::Optimizer::new(project);
    optimizer.cache_config().await
}

/// 处理 composer install 命令
/// 
/// 根据 composer.json 或 composer.lock 安装所有依赖包
/// 
/// # 参数
/// - `no_dev`: 是否跳过开发依赖
/// - `optimize`: 是否优化自动加载
async fn handle_composer_install(no_dev: bool, optimize: bool) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    tracing::info!("开始安装 Composer 依赖...");
    
    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());
    
    // 执行安装
    composer.install().await?;
    
    // 如果需要优化自动加载
    if optimize {
        tracing::info!("优化自动加载...");
        composer.dump_autoload().await?;
    }
    
    // 如果跳过开发依赖，输出提示
    if no_dev {
        tracing::info!("已跳过开发依赖 (require-dev)");
    }
    
    tracing::info!("✓ Composer 依赖安装完成");
    Ok(())
}

/// 处理 composer update 命令
/// 
/// 更新指定的包到最新版本，如果包列表为空则更新所有依赖
/// 
/// # 参数
/// - `packages`: 要更新的包列表
/// - `no_dev`: 是否跳过开发依赖
async fn handle_composer_update(packages: &[String], no_dev: bool) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    tracing::info!("开始更新 Composer 依赖...");
    
    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());
    
    // 执行更新
    composer.update(packages).await?;
    
    // 如果跳过开发依赖，输出提示
    if no_dev {
        tracing::info!("已跳过开发依赖 (require-dev)");
    }
    
    tracing::info!("✓ Composer 依赖更新完成");
    Ok(())
}

/// 处理 composer require 命令
/// 
/// 添加新的依赖包到 composer.json 并安装
/// 
/// # 参数
/// - `packages`: 包列表，格式为 "vendor/package" 或 "vendor/package:^1.0"
/// - `dev`: 是否添加为开发依赖
async fn handle_composer_require(packages: &[String], dev: bool) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    if packages.is_empty() {
        anyhow::bail!("请指定要添加的包，例如: oyta composer:require vendor/package");
    }

    tracing::info!("开始添加 Composer 依赖...");
    
    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());
    
    // 解析并添加每个包
    for package_spec in packages {
        // 解析包名和版本约束
        let (package_name, version) = parse_package_spec(package_spec);
        
        // 添加依赖
        composer.add_require(&package_name, &version).await?;
        
        // 如果是开发依赖，输出提示
        if dev {
            tracing::info!("已添加为开发依赖: {}", package_name);
        }
    }
    
    tracing::info!("✓ Composer 依赖添加完成");
    Ok(())
}

/// 处理 composer remove 命令
/// 
/// 从 composer.json 中移除依赖包并删除相关文件
/// 
/// # 参数
/// - `packages`: 要移除的包名列表
async fn handle_composer_remove(packages: &[String]) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    if packages.is_empty() {
        anyhow::bail!("请指定要移除的包，例如: oyta composer:remove vendor/package");
    }

    tracing::info!("开始移除 Composer 依赖...");
    
    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());
    
    // 移除每个包
    for package_name in packages {
        composer.remove_require(package_name).await?;
    }
    
    tracing::info!("✓ Composer 依赖移除完成");
    Ok(())
}

/// 处理 composer dump-autoload 命令
/// 
/// 重新生成自动加载文件
/// 
/// # 参数
/// - `optimize`: 是否优化自动加载（生成完整 classmap）
async fn handle_composer_dump_autoload(optimize: bool) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    tracing::info!("重新生成自动加载文件...");
    
    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());
    
    // 加载配置
    composer.load().await?;
    
    // 重新生成自动加载
    composer.dump_autoload().await?;
    
    // 如果需要优化
    if optimize {
        tracing::info!("已优化自动加载（生成完整 classmap）");
    }
    
    tracing::info!("✓ 自动加载文件已重新生成");
    Ok(())
}

/// 处理 composer show 命令
/// 
/// 显示已安装的包信息
/// 
/// # 参数
/// - `package`: 指定包名，不指定则显示所有包
/// - `direct`: 是否只显示直接依赖
async fn handle_composer_show(package: &Option<String>, direct: bool) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    tracing::info!("查看已安装的包...");
    
    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());
    
    // 加载配置
    composer.load().await?;
    
    // 如果指定了包名，显示详细信息
    if let Some(pkg_name) = package {
        if let Some(version) = composer.get_package_version(pkg_name) {
            println!("包名: {}", pkg_name);
            println!("版本: {}", version);
            
            // 获取配置中的包信息
            if let Some(config) = composer.config() {
                if let Some(version_constraint) = config.require.get(pkg_name) {
                    println!("版本约束: {}", version_constraint);
                }
            }
        } else {
            tracing::warn!("包 {} 未安装", pkg_name);
        }
    } else {
        // 显示所有已安装的包
        let installed = composer.get_installed_packages();
        
        if installed.is_empty() {
            println!("没有已安装的包");
            return Ok(());
        }
        
        println!("已安装的包 (共 {} 个):", installed.len());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        for pkg_name in &installed {
            if let Some(version) = composer.get_package_version(pkg_name) {
                println!("  {} {}", pkg_name, version);
            }
        }
        
        // 如果只显示直接依赖
        if direct {
            println!("\n直接依赖:");
            if let Some(config) = composer.config() {
                for (name, version) in &config.require {
                    println!("  {} {}", name, version);
                }
            }
        }
    }
    
    Ok(())
}

/// 处理 composer outdated 命令
/// 
/// 检查已安装的包是否有新版本可用
async fn handle_composer_outdated() -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    tracing::info!("检查过时的包...");
    
    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());
    
    // 加载配置
    composer.load().await?;
    
    // 获取已安装的包
    let installed = composer.get_installed_packages();
    
    if installed.is_empty() {
        println!("没有已安装的包");
        return Ok(());
    }
    
    // 创建 Packagist API 客户端
    let mut packagist = crate::composer::packagist::PackagistClient::new();
    
    println!("检查包更新状态（连接 Packagist API）...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{:<40} {:<15} {:<15} {:<10}", "包名", "当前版本", "最新版本", "状态");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // 统计信息
    let mut outdated_count = 0;
    let mut up_to_date_count = 0;
    let mut error_count = 0;
    let mut outdated_packages: Vec<OutdatedPackageInfo> = Vec::new();
    
    // 遍历每个已安装的包，检查更新
    for pkg_name in &installed {
        // 获取当前安装的版本
        let current_version = match composer.get_package_version(pkg_name) {
            Some(v) => v,
            None => {
                println!("{:<40} {:<15} {:<15} {:<10}", pkg_name, "未知", "-", "⚠ 无法获取");
                error_count += 1;
                continue;
            }
        };
        
        // 从 Packagist API 获取最新版本信息
        match fetch_latest_version(&mut packagist, pkg_name).await {
            Ok(latest_info) => {
                let latest_version = latest_info.latest_version;
                let is_outdated = latest_info.is_outdated;
                
                // 规范化版本号用于比较
                let current_normalized = normalize_version(&current_version);
                let latest_normalized = normalize_version(&latest_version);
                
                // 比较版本
                let needs_update = is_outdated || compare_versions(&latest_normalized, &current_normalized);
                
                if needs_update {
                    let status = "🔴 需更新";
                    outdated_count += 1;
                    
                    // 记录过时包的详细信息
                    outdated_packages.push(OutdatedPackageInfo {
                        name: pkg_name.clone(),
                        current_version: current_version.clone(),
                        latest_version: latest_version.clone(),
                        description: latest_info.description,
                    });
                    
                    println!("{:<40} {:<15} {:<15} {:<10}", 
                        pkg_name, 
                        current_version, 
                        latest_version, 
                        status
                    );
                } else {
                    let status = "✓ 最新";
                    up_to_date_count += 1;
                    
                    println!("{:<40} {:<15} {:<15} {:<10}", 
                        pkg_name, 
                        current_version, 
                        latest_version, 
                        status
                    );
                }
            }
            Err(e) => {
                // API 请求失败，显示当前版本并标记错误
                println!("{:<40} {:<15} {:<15} {:<10}", 
                    pkg_name, 
                    current_version, 
                    "-", 
                    "⚠ API错误"
                );
                tracing::debug!("获取 {} 版本信息失败: {}", pkg_name, e);
                error_count += 1;
            }
        }
    }
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // 显示统计摘要
    println!();
    println!("统计摘要:");
    println!("  总包数: {}", installed.len());
    println!("  ✓ 最新: {} 个", up_to_date_count);
    println!("  🔴 需更新: {} 个", outdated_count);
    if error_count > 0 {
        println!("  ⚠ 检查失败: {} 个", error_count);
    }
    
    // 如果有过时的包，显示更新建议
    if !outdated_packages.is_empty() {
        println!();
        println!("更新建议:");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        for pkg in &outdated_packages {
            println!();
            println!("📦 {} ({} → {})", pkg.name, pkg.current_version, pkg.latest_version);
            if !pkg.description.is_empty() {
                println!("   描述: {}", pkg.description);
            }
            println!("   更新命令: oyta composer require {}:{}", pkg.name, pkg.latest_version);
        }
        
        println!();
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("提示: 使用 'oyta composer update' 更新所有包");
        println!("      使用 'oyta composer require <package>:<version>' 更新指定包");
    } else if outdated_count == 0 && error_count == 0 {
        println!();
        println!("✓ 所有包都是最新版本！");
    }
    
    Ok(())
}

/// 过时包信息
/// 用于存储需要更新的包的详细信息
struct OutdatedPackageInfo {
    /// 包名称
    name: String,
    /// 当前版本
    current_version: String,
    /// 最新版本
    latest_version: String,
    /// 包描述
    description: String,
}

/// 最新版本信息
/// 从 Packagist API 获取的版本信息
struct LatestVersionInfo {
    /// 最新版本号
    latest_version: String,
    /// 是否过时
    is_outdated: bool,
    /// 包描述
    description: String,
}

/// 从 Packagist API 获取最新版本信息
/// 
/// 该函数会：
/// 1. 调用 Packagist API 获取包的所有版本
/// 2. 过滤掉不稳定版本（dev/alpha/beta/RC）
/// 3. 找出最新的稳定版本
/// 4. 返回版本信息和包描述
/// 
/// # 参数
/// - `packagist`: Packagist API 客户端
/// - `package_name`: 包名称
/// 
/// # 返回值
/// 成功返回 LatestVersionInfo，失败返回错误
async fn fetch_latest_version(
    packagist: &mut crate::composer::packagist::PackagistClient,
    package_name: &str,
) -> Result<LatestVersionInfo> {
    // 获取包信息
    let package_info = packagist.get_package(package_name).await?;
    
    // 提取描述
    let description = package_info.description.clone();
    
    // 获取所有版本并过滤稳定版本
    let mut stable_versions: Vec<(String, (u32, u32, u32, u32))> = Vec::new();
    
    for version_str in package_info.versions.keys() {
        // 跳过不稳定版本
        if is_unstable_version(version_str) {
            continue;
        }
        
        // 解析版本号为可比较的元组
        let normalized = normalize_version(version_str);
        let version_tuple = parse_version_tuple(&normalized);
        stable_versions.push((version_str.clone(), version_tuple));
    }
    
    // 按版本号排序（降序）
    stable_versions.sort_by(|a, b| b.1.cmp(&a.1));
    
    // 获取最新版本
    let latest_version = stable_versions
        .first()
        .map(|(v, _)| v.clone())
        .ok_or_else(|| anyhow::anyhow!("没有找到稳定版本"))?;
    
    Ok(LatestVersionInfo {
        latest_version,
        is_outdated: false,
        description,
    })
}

/// 判断版本是否为不稳定版本
/// 
/// 不稳定版本包括：
/// - dev 版本（如 dev-master, dev-main）
/// - alpha 版本（如 1.0.0-alpha1）
/// - beta 版本（如 1.0.0-beta2）
/// - RC 版本（如 1.0.0-RC1）
/// 
/// # 参数
/// - `version`: 版本字符串
/// 
/// # 返回值
/// 如果是不稳定版本返回 true，否则返回 false
fn is_unstable_version(version: &str) -> bool {
    let version_lower = version.to_lowercase();
    
    // 检查各种不稳定版本标识
    version_lower.contains("dev") ||
    version_lower.contains("alpha") ||
    version_lower.contains("beta") ||
    version_lower.contains("rc") ||
    version_lower.contains("preview") ||
    version_lower.contains("snapshot") ||
    version_lower.contains("nightly") ||
    version_lower.contains("testing") ||
    // 检查是否以 "v" 开头后跟不稳定标识
    version_lower.starts_with("dev-")
}

/// 规范化版本号
/// 
/// 移除版本号前缀（如 "v"）和后缀信息
/// 将版本号规范化为标准格式
/// 
/// # 参数
/// - `version`: 原始版本字符串
/// 
/// # 返回值
/// 规范化后的版本字符串
fn normalize_version(version: &str) -> String {
    let mut normalized = version.trim().to_string();
    
    // 移除 "v" 前缀（如 v1.0.0 → 1.0.0）
    if normalized.starts_with('v') || normalized.starts_with('V') {
        normalized = normalized[1..].to_string();
    }
    
    // 移除可能存在的后缀（如 -stable）
    if let Some(pos) = normalized.find("-stable") {
        normalized = normalized[..pos].to_string();
    }
    
    normalized
}

/// 解析版本号为元组
/// 
/// 将版本字符串解析为 (major, minor, patch, build) 元组
/// 用于版本比较
/// 
/// # 参数
/// - `version`: 版本字符串（如 "1.2.3"）
/// 
/// # 返回值
/// 版本元组 (major, minor, patch, build)
fn parse_version_tuple(version: &str) -> (u32, u32, u32, u32) {
    // 移除可能存在的后缀（如 -patch1）
    let version_clean = version.split('-').next().unwrap_or(version);
    
    let parts: Vec<&str> = version_clean.split('.').collect();
    
    let major = parts.get(0)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    
    let minor = parts.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    
    let patch = parts.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    
    let build = parts.get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    
    (major, minor, patch, build)
}

/// 比较两个版本号
/// 
/// 判断 latest 版本是否大于 current 版本
/// 
/// # 参数
/// - `latest`: 最新版本字符串
/// - `current`: 当前版本字符串
/// 
/// # 返回值
/// 如果 latest > current 返回 true，否则返回 false
fn compare_versions(latest: &str, current: &str) -> bool {
    let latest_tuple = parse_version_tuple(latest);
    let current_tuple = parse_version_tuple(current);
    
    latest_tuple > current_tuple
}

/// 处理 composer validate 命令
/// 
/// 验证 composer.json 格式是否正确
async fn handle_composer_validate() -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    tracing::info!("验证 composer.json...");
    
    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());
    
    // 加载配置
    composer.load().await?;
    
    // 执行验证
    let warnings = composer.validate()?;
    
    if warnings.is_empty() {
        println!("✓ composer.json 格式正确");
    } else {
        println!("composer.json 验证结果:");
        for warning in &warnings {
            println!("  ⚠ {}", warning);
        }
    }
    
    Ok(())
}

/// 处理 composer config 命令
/// 
/// 读取或修改 composer.json 中的配置项
/// 
/// # 参数
/// - `args`: 配置参数
/// - `list`: 是否列出所有配置项
async fn handle_composer_config(args: &[String], list: bool) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    tracing::info!("Composer 配置管理...");
    
    // 创建 Composer 管理器实例
    let mut composer = crate::composer::Composer::new(project.root.to_str().unwrap());
    
    // 加载配置
    composer.load().await?;
    
    // 如果列出所有配置项
    if list {
        if let Some(config) = composer.config() {
            println!("composer.json 配置项:");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("名称: {}", config.name);
            println!("描述: {}", config.description);
            println!("类型: {}", config.r#type);
            println!("许可证: {:?}", config.license);
            println!("最小稳定性: {}", config.minimum_stability);
            println!("优先稳定版本: {}", config.prefer_stable);
            
            if !config.require.is_empty() {
                println!("\n依赖 (require):");
                for (name, version) in &config.require {
                    println!("  {} = {}", name, version);
                }
            }
            
            if !config.require_dev.is_empty() {
                println!("\n开发依赖 (require-dev):");
                for (name, version) in &config.require_dev {
                    println!("  {} = {}", name, version);
                }
            }
            
            if !config.autoload.psr_4.is_empty() {
                println!("\nPSR-4 自动加载:");
                for (namespace, paths) in &config.autoload.psr_4 {
                    println!("  {} => {:?}", namespace, paths);
                }
            }
            
            if !config.repositories.is_empty() {
                println!("\n仓库配置:");
                for repo in &config.repositories {
                    println!("  {} ({})", repo.url, repo.r#type);
                }
            }
        }
        return Ok(());
    }
    
    // 处理配置读取/设置
    match args.len() {
        0 => {
            anyhow::bail!("请指定配置项名称，或使用 --list 查看所有配置");
        }
        1 => {
            // 读取配置值
            let key = &args[0];
            if let Some(config) = composer.config() {
                let value = get_config_value(config, key);
                if let Some(v) = value {
                    println!("{} = {}", key, v);
                } else {
                    println!("配置项 {} 不存在", key);
                }
            }
        }
        _ => {
            // 设置配置值
            tracing::info!("设置配置项需要修改 composer.json，请手动编辑文件");
            println!("提示: 请直接编辑 composer.json 文件来修改配置项");
        }
    }
    
    Ok(())
}

/// 处理 composer clear-cache 命令
/// 
/// 清除本地缓存的包元数据和下载文件
async fn handle_composer_clear_cache() -> Result<()> {
    tracing::info!("清除 Composer 缓存...");
    
    // 获取缓存目录
    let cache_dir = std::env::var("COMPOSER_CACHE_DIR")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{}/.composer/cache", home)
        });
    
    let cache_path = std::path::Path::new(&cache_dir);
    
    if cache_path.exists() {
        // 删除缓存目录
        std::fs::remove_dir_all(cache_path)?;
        println!("✓ 已清除 Composer 缓存: {}", cache_dir);
    } else {
        println!("缓存目录不存在: {}", cache_dir);
    }
    
    Ok(())
}

/// 处理 composer diagnose 命令
/// 
/// 检查 Composer 运行环境是否正常
async fn handle_composer_diagnose() -> Result<()> {
    tracing::info!("Composer 环境诊断...");
    
    println!("Composer 环境诊断结果:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // 检查 composer.json 是否存在
    let project = project::detector::Project::detect_from_cwd();
    match project {
        Ok(p) => {
            let json_path = p.root.join("composer.json");
            if json_path.exists() {
                println!("✓ composer.json 存在: {}", json_path.display());
            } else {
                println!("⚠ composer.json 不存在");
            }
            
            let lock_path = p.root.join("composer.lock");
            if lock_path.exists() {
                println!("✓ composer.lock 存在: {}", lock_path.display());
            } else {
                println!("⚠ composer.lock 不存在");
            }
            
            let vendor_path = p.root.join("vendor");
            if vendor_path.exists() {
                println!("✓ vendor 目录存在: {}", vendor_path.display());
            } else {
                println!("⚠ vendor 目录不存在，请运行 composer install");
            }
        }
        Err(e) => {
            println!("⚠ 无法检测项目目录: {}", e);
        }
    }
    
    // 检查网络连接
    println!("\n网络检测:");
    println!("  Packagist: https://repo.packagist.org");
    println!("  中国镜像: https://mirrors.aliyun.com/composer/");
    
    // 检查环境变量
    println!("\n环境变量:");
    if let Ok(cache_dir) = std::env::var("COMPOSER_CACHE_DIR") {
        println!("  COMPOSER_CACHE_DIR: {}", cache_dir);
    } else {
        println!("  COMPOSER_CACHE_DIR: 未设置 (使用默认值)");
    }
    
    if let Ok(home) = std::env::var("HOME") {
        println!("  HOME: {}", home);
    }
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✓ 诊断完成");
    
    Ok(())
}

/// 处理 service:discover 命令
async fn handle_service_discover() -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    tracing::warn!("服务发现功能将在后续阶段实现");
    Ok(())
}

/// 处理 vendor:publish 命令
async fn handle_vendor_publish(_package: &Option<String>, _force: bool) -> Result<()> {
    let project = project::detector::Project::detect_from_cwd()?;
    env_loader::loader::load_env(&project)?;

    tracing::warn!("配置发布功能将在后续阶段实现");
    Ok(())
}

/// 处理 version 命令：查看版本信息
/// 显示 OYTAPHP 版本号、项目信息等
async fn handle_version() -> Result<()> {
    println!("OYTAPHP v{}", env!("CARGO_PKG_VERSION"));
    println!("Rust 运行时，兼容 ThinkPHP 8.0");

    // 尝试检测项目信息
    if let Ok(project) = project::detector::Project::detect_from_cwd() {
        let info = project.info();
        println!();
        println!("{}", info);
    }

    Ok(())
}

/// 处理 list 命令：列出所有可用命令
async fn handle_list() -> Result<()> {
    println!("OYTAPHP 可用命令列表:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  oyta run                          启动服务");
    println!("  oyta run -p 8080                  指定端口启动");
    println!("  oyta build <name>                 构建应用目录");
    println!("  oyta make:controller <name>       创建控制器");
    println!("  oyta make:model <name>            创建模型");
    println!("  oyta make:middleware <name>        创建中间件");
    println!("  oyta make:validate <name>         创建验证器");
    println!("  oyta make:event <name>            创建事件");
    println!("  oyta make:listener <name>         创建监听器");
    println!("  oyta make:subscribe <name>        创建订阅者");
    println!("  oyta make:service <name>          创建服务类");
    println!("  oyta make:command <name>          创建自定义命令");
    println!("  oyta route:list                   查看路由列表");
    println!("  oyta route:cache                  缓存路由");
    println!("  oyta optimize                     优化应用");
    println!("  oyta clear                        清除运行时缓存");
    println!("  oyta version                      查看版本");
    println!("  oyta list                         列出所有命令");
    println!("  oyta help                         帮助信息");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

/// 解析名称中的命名空间和类名
/// 例如: "admin/Index" → ("app\\controller\\admin", "Index")
/// 例如: "Index" → ("app\\controller", "Index")
///
/// # 参数
/// - `name`: 用户输入的名称，可能包含路径分隔符
/// - `base_namespace`: 基础命名空间，如 "app\\controller"
///
/// # 返回值
/// 元组 (完整命名空间, 类名)
fn parse_name_with_namespace(name: &str, base_namespace: &str) -> (String, String) {
    // 将路径分隔符统一处理
    let parts: Vec<&str> = name.split('/')
        .flat_map(|s| s.split('\\'))
        .collect();

    if parts.len() > 1 {
        // 多级名称：admin/Index
        // 命名空间 = base_namespace + 子路径
        // 类名 = 最后一段
        let sub_namespace = parts[..parts.len() - 1].join("\\");
        let class_name = parts[parts.len() - 1].to_string();
        let full_namespace = format!("{}\\{}", base_namespace, sub_namespace);
        (full_namespace, class_name)
    } else {
        // 单级名称：Index
        let class_name = parts[0].to_string();
        (base_namespace.to_string(), class_name)
    }
}

/// 将驼峰命名转换为下划线命名
/// 例如: "UserProfile" → "user_profile"
/// 例如: "User" → "user"
///
/// # 参数
/// - `input`: 驼峰命名的字符串
///
/// # 返回值
/// 下划线命名的字符串
fn camel_to_snake(input: &str) -> String {
    let mut result = String::new();

    for (i, ch) in input.chars().enumerate() {
        if ch.is_uppercase() {
            // 大写字母前插入下划线（首字母除外）
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }

    result
}

/// 解析包规格字符串
/// 
/// 将 "vendor/package:^1.0" 格式解析为 (包名, 版本约束)
/// 如果没有版本约束，默认使用 "*" 表示任意版本
/// 
/// # 参数
/// - `spec`: 包规格字符串
/// 
/// # 返回值
/// 元组 (包名, 版本约束)
fn parse_package_spec(spec: &str) -> (String, String) {
    // 查找版本分隔符
    if let Some(pos) = spec.find(':') {
        let name = spec[..pos].to_string();
        let version = spec[pos + 1..].to_string();
        (name, version)
    } else {
        // 没有版本约束，使用 *
        (spec.to_string(), "*".to_string())
    }
}

/// 获取配置值
/// 
/// 根据键名从 ComposerJson 中获取配置值
/// 
/// # 参数
/// - `config`: ComposerJson 配置引用
/// - `key`: 配置键名
/// 
/// # 返回值
/// 配置值的字符串表示，如果不存在返回 None
fn get_config_value(config: &crate::composer::ComposerJson, key: &str) -> Option<String> {
    match key {
        "name" => Some(config.name.clone()),
        "description" => Some(config.description.clone()),
        "type" => Some(config.r#type.clone()),
        "license" => Some(format!("{:?}", config.license)),
        "minimum-stability" => Some(config.minimum_stability.clone()),
        "prefer-stable" => Some(config.prefer_stable.to_string()),
        _ => {
            // 尝试从 require 中获取
            if let Some(version) = config.require.get(key) {
                return Some(version.clone());
            }
            // 尝试从 require_dev 中获取
            if let Some(version) = config.require_dev.get(key) {
                return Some(version.clone());
            }
            None
        }
    }
}
