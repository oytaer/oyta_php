//! 项目初始化命令模块
//!
//! 提供完整的项目初始化功能
//! 自动生成所有配置文件、环境变量文件、composer.json 等

use anyhow::Result;

use crate::project;

/// 处理 init 命令
///
/// 初始化完整的 OYTAPHP 项目结构
/// 包括：配置文件、环境变量、composer.json、目录结构等
///
/// # 参数
/// - `app_name`: 应用名称（可选，不指定则为单应用模式）
pub async fn handle_init(app_name: Option<&str>) -> Result<()> {
    // 检测项目根目录
    let project = project::detector::Project::detect_from_cwd()?;

    let is_multi_app = app_name.is_some();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  OYTAPHP 项目初始化");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // 1. 创建目录结构
    println!("  [1/5] 创建目录结构...");
    create_directories(&project, app_name)?;
    println!("  ✓ 目录结构创建完成");

    // 2. 生成配置文件
    println!("\n  [2/5] 生成配置文件...");
    create_config_files(&project)?;
    println!("  ✓ 配置文件生成完成");

    // 3. 生成 .env 文件
    println!("\n  [3/5] 生成环境变量文件...");
    create_env_file(&project)?;
    println!("  ✓ .env 文件生成完成");

    // 4. 生成 composer.json
    println!("\n  [4/5] 生成 composer.json...");
    create_composer_json(&project)?;
    println!("  ✓ composer.json 生成完成");

    // 5. 创建默认控制器
    println!("\n  [5/5] 创建默认控制器...");
    create_default_controller(&project, app_name)?;
    println!("  ✓ 默认控制器创建完成");

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  项目初始化完成！");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    if is_multi_app {
        println!("  项目结构 (多应用模式):");
        println!("    app/{}/", app_name.unwrap());
    } else {
        println!("  项目结构 (单应用模式):");
        println!("    app/");
    }
    println!("    ├── controller/");
    println!("    ├── model/");
    println!("    ├── middleware/");
    println!("    ├── validate/");
    println!("    └── view/");
    println!("    config/");
    println!("    route/");
    println!("    runtime/");
    println!("    storage/");
    println!("    vendor/");
    println!("    .env");
    println!("    composer.json");
    println!();
    println!("  启动服务器:");
    println!("    oyta run");
    println!();

    Ok(())
}

/// 创建目录结构
/// 默认创建单应用模式（app/controller/）
/// 如果指定了 app_name，则创建多应用模式（app/{app_name}/controller/）
fn create_directories(project: &project::detector::Project, app_name: Option<&str>) -> Result<()> {
    // 应用目录
    let app_dir = if let Some(name) = app_name {
        // 多应用模式: app/{app_name}/
        project.root.join("app").join(name)
    } else {
        // 单应用模式: app/
        project.root.join("app")
    };
    
    std::fs::create_dir_all(app_dir.join("controller"))?;
    std::fs::create_dir_all(app_dir.join("model"))?;
    std::fs::create_dir_all(app_dir.join("middleware"))?;
    std::fs::create_dir_all(app_dir.join("validate"))?;
    std::fs::create_dir_all(app_dir.join("view"))?;

    // 配置目录
    std::fs::create_dir_all(&project.config_dir)?;

    // 路由目录
    std::fs::create_dir_all(project.root.join("route"))?;

    // 运行时目录
    std::fs::create_dir_all(project.runtime_dir.join("cache"))?;
    std::fs::create_dir_all(project.runtime_dir.join("log"))?;
    std::fs::create_dir_all(project.runtime_dir.join("temp"))?;
    std::fs::create_dir_all(project.runtime_dir.join("session"))?;

    // 公共目录
    std::fs::create_dir_all(&project.public_dir)?;
    std::fs::create_dir_all(project.public_dir.join("static"))?;

    // 存储目录
    std::fs::create_dir_all(project.root.join("storage").join("app").join("public"))?;

    // Vendor 目录（Composer 依赖）
    std::fs::create_dir_all(project.root.join("vendor"))?;

    Ok(())
}

/// 创建所有配置文件
fn create_config_files(project: &project::detector::Project) -> Result<()> {
    let config_dir = &project.config_dir;

    // app.php - 应用配置
    std::fs::write(
        config_dir.join("app.php"),
        generate_app_config(),
    )?;

    // database.php - 数据库配置
    std::fs::write(
        config_dir.join("database.php"),
        generate_database_config(),
    )?;

    // cache.php - 缓存配置
    std::fs::write(
        config_dir.join("cache.php"),
        generate_cache_config(),
    )?;

    // session.php - Session 配置
    std::fs::write(
        config_dir.join("session.php"),
        generate_session_config(),
    )?;

    // log.php - 日志配置
    std::fs::write(
        config_dir.join("log.php"),
        generate_log_config(),
    )?;

    // queue.php - 队列配置
    std::fs::write(
        config_dir.join("queue.php"),
        generate_queue_config(),
    )?;

    // websocket.php - WebSocket 配置
    std::fs::write(
        config_dir.join("websocket.php"),
        generate_websocket_config(),
    )?;

    // timer.php - 定时器配置
    std::fs::write(
        config_dir.join("timer.php"),
        generate_timer_config(),
    )?;

    // crypt.php - 加密配置
    std::fs::write(
        config_dir.join("crypt.php"),
        generate_crypt_config(),
    )?;

    // cors.php - CORS 配置
    std::fs::write(
        config_dir.join("cors.php"),
        generate_cors_config(),
    )?;

    // template.php - 模板配置
    std::fs::write(
        config_dir.join("template.php"),
        generate_template_config(),
    )?;

    // i18n.php - 国际化配置
    std::fs::write(
        config_dir.join("i18n.php"),
        generate_i18n_config(),
    )?;

    // event.php - 事件配置
    std::fs::write(
        config_dir.join("event.php"),
        generate_event_config(),
    )?;

    // filesystem.php - 文件系统配置
    std::fs::write(
        config_dir.join("filesystem.php"),
        generate_filesystem_config(),
    )?;

    // middleware.php - 中间件配置
    std::fs::write(
        config_dir.join("middleware.php"),
        generate_middleware_config(),
    )?;

    Ok(())
}

/// 生成 app.php 配置
fn generate_app_config() -> String {
    r#"<?php
/**
 * 应用配置文件
 * 
 * 包含应用的基础配置信息
 */

return [
    // 应用名称
    'name' => env('APP_NAME', 'OYTAPHP'),
    
    // 运行环境：development, production, testing
    'env' => env('APP_ENV', 'production'),
    
    // 调试模式
    'debug' => env('APP_DEBUG', false),
    
    // 应用地址（对外访问的 URL，用于生成链接）
    // 例如：http://example.com 或 https://www.example.com
    'url' => env('APP_URL', 'http://localhost'),
    
    // 服务器监听地址（内部监听）
    // 0.0.0.0 表示监听所有网络接口，127.0.0.1 表示只监听本地
    'host' => env('APP_HOST', '0.0.0.0'),
    
    // 服务器监听端口
    'port' => env('APP_PORT', 8000),
    
    // 工作进程数量
    'workers' => env('APP_WORKERS', 2),
    
    // 时区
    'timezone' => env('APP_TIMEZONE', 'Asia/Shanghai'),
    
    // 默认语言
    'locale' => env('APP_LOCALE', 'zh-cn'),
    
    // 字符集
    'charset' => 'UTF-8',
];
"#.to_string()
}

/// 生成 database.php 配置
fn generate_database_config() -> String {
    r#"<?php
/**
 * 数据库配置文件
 * 
 * 支持 MySQL、PostgreSQL、SQLite
 */

return [
    // 默认数据库连接
    'default' => env('DB_CONNECTION', 'mysql'),
    
    // 数据库连接配置
    'connections' => [
        'mysql' => [
            'type' => 'mysql',
            'hostname' => env('DB_HOST', '127.0.0.1'),
            'port' => env('DB_PORT', 3306),
            'database' => env('DB_NAME', 'test'),
            'username' => env('DB_USER', 'root'),
            'password' => env('DB_PASS', ''),
            'charset' => 'utf8mb4',
            'prefix' => env('DB_PREFIX', ''),
            'pool_size' => env('DB_POOL_SIZE', 10),
            'debug' => env('APP_DEBUG', false),
        ],
        
        'pgsql' => [
            'type' => 'pgsql',
            'hostname' => env('DB_HOST', '127.0.0.1'),
            'port' => env('DB_PORT', 5432),
            'database' => env('DB_NAME', 'test'),
            'username' => env('DB_USER', 'postgres'),
            'password' => env('DB_PASS', ''),
            'charset' => 'utf8',
            'prefix' => env('DB_PREFIX', ''),
            'pool_size' => env('DB_POOL_SIZE', 10),
        ],
        
        'sqlite' => [
            'type' => 'sqlite',
            'database' => env('DB_PATH', 'runtime/database.sqlite'),
            'prefix' => env('DB_PREFIX', ''),
        ],
    ],
];
"#.to_string()
}

/// 生成 cache.php 配置
fn generate_cache_config() -> String {
    r#"<?php
/**
 * 缓存配置文件
 * 
 * 支持：file（文件缓存）、memory（内存缓存）、redis（Redis缓存）
 */

return [
    // 默认缓存驱动
    'default' => env('CACHE_DRIVER', 'file'),
    
    // 缓存键前缀
    'prefix' => env('CACHE_PREFIX', 'cache_'),
    
    // 是否启用缓存
    'enable' => env('CACHE_ENABLE', true),
    
    // 缓存驱动配置
    'stores' => [
        // 文件缓存（默认）
        'file' => [
            'type' => 'file',
            'path' => env('CACHE_PATH', 'runtime/cache'),
            'expire' => env('CACHE_EXPIRE', 3600),
        ],
        
        // 内存缓存
        'memory' => [
            'type' => 'memory',
            'max_size' => env('CACHE_MAX_SIZE', 1000),
        ],
        
        // Redis 缓存
        'redis' => [
            'type' => 'redis',
            'host' => env('REDIS_HOST', '127.0.0.1'),
            'port' => env('REDIS_PORT', 6379),
            'password' => env('REDIS_PASSWORD', ''),
            'database' => env('REDIS_DATABASE', 0),
            'timeout' => 5,
        ],
    ],
];
"#.to_string()
}

/// 生成 session.php 配置
fn generate_session_config() -> String {
    r#"<?php
/**
 * Session 配置文件
 * 
 * 支持：file（文件）、memory（内存）、redis（Redis）
 */

return [
    // Session 驱动
    'driver' => env('SESSION_DRIVER', 'file'),
    
    // Session 名称
    'name' => env('SESSION_NAME', 'OYTA_SESSION'),
    
    // Session 存储路径
    'save_path' => env('SESSION_PATH', 'runtime/session'),
    
    // Session 生命周期（秒）
    'lifetime' => env('SESSION_LIFETIME', 7200),
    
    // Redis 配置（当 driver 为 redis 时使用）
    'redis' => [
        'host' => env('REDIS_HOST', '127.0.0.1'),
        'port' => env('REDIS_PORT', 6379),
        'password' => env('REDIS_PASSWORD', ''),
        'database' => env('REDIS_DATABASE', 0),
    ],
    
    // Cookie 配置
    'cookie' => [
        'lifetime' => env('SESSION_COOKIE_LIFETIME', 7200),
        'path' => '/',
        'domain' => env('SESSION_DOMAIN', ''),
        'secure' => env('SESSION_SECURE', false),
        'httponly' => true,
        'samesite' => 'lax',
    ],
];
"#.to_string()
}

/// 生成 log.php 配置
fn generate_log_config() -> String {
    r#"<?php
/**
 * 日志配置文件
 * 
 * 支持：file（文件）、stdout（标准输出）、stderr（标准错误）
 */

return [
    // 默认日志通道
    'default' => env('LOG_CHANNEL', 'file'),
    
    // 日志级别：emergency, alert, critical, error, warning, notice, info, debug
    'level' => env('LOG_LEVEL', 'info'),
    
    // 日志通道配置
    'channels' => [
        // 文件日志
        'file' => [
            'type' => 'file',
            'path' => env('LOG_PATH', 'runtime/log'),
            'level' => env('LOG_LEVEL', 'info'),
            'max_files' => env('LOG_MAX_FILES', 30),
        ],
        
        // 标准输出
        'stdout' => [
            'type' => 'stdout',
            'level' => env('LOG_LEVEL', 'debug'),
        ],
        
        // 标准错误
        'stderr' => [
            'type' => 'stderr',
            'level' => 'error',
        ],
    ],
];
"#.to_string()
}

/// 生成 queue.php 配置
fn generate_queue_config() -> String {
    r#"<?php
/**
 * 队列配置文件
 * 
 * 支持：sync（同步）、database（数据库）、redis（Redis）
 */

return [
    // 默认队列驱动
    'default' => env('QUEUE_DRIVER', 'sync'),
    
    // 是否启用队列
    'enable' => env('QUEUE_ENABLE', false),
    
    // 队列驱动配置
    'connections' => [
        // 同步驱动（直接执行，不入队）
        'sync' => [
            'type' => 'sync',
        ],
        
        // 数据库驱动
        'database' => [
            'type' => 'database',
            'table' => 'jobs',
            'retry_after' => 90,
        ],
        
        // Redis 驱动
        'redis' => [
            'type' => 'redis',
            'host' => env('REDIS_HOST', '127.0.0.1'),
            'port' => env('REDIS_PORT', 6379),
            'password' => env('REDIS_PASSWORD', ''),
            'database' => env('REDIS_DATABASE', 0),
            'queue' => 'default',
            'retry_after' => 90,
        ],
    ],
    
    // 失败任务配置
    'failed' => [
        'database' => 'mysql',
        'table' => 'failed_jobs',
    ],
];
"#.to_string()
}

/// 生成 websocket.php 配置
fn generate_websocket_config() -> String {
    r#"<?php
/**
 * WebSocket 配置文件
 * 
 * 默认禁用，需要时手动启用
 */

return [
    // 是否启用 WebSocket 服务
    'enable' => env('WEBSOCKET_ENABLE', false),
    
    // 监听地址
    'host' => env('WEBSOCKET_HOST', '0.0.0.0'),
    
    // 监听端口
    'port' => env('WEBSOCKET_PORT', 9501),
    
    // 最大连接数
    'max_connections' => env('WEBSOCKET_MAX_CONNECTIONS', 1000),
    
    // 心跳间隔（秒）
    'heartbeat_interval' => env('WEBSOCKET_HEARTBEAT_INTERVAL', 30),
    
    // 心跳超时（秒）
    'heartbeat_timeout' => env('WEBSOCKET_HEARTBEAT_TIMEOUT', 60),
    
    // 最大消息大小（字节）
    'max_message_size' => env('WEBSOCKET_MAX_MESSAGE_SIZE', 1048576),
];
"#.to_string()
}

/// 生成 timer.php 配置
fn generate_timer_config() -> String {
    r#"<?php
/**
 * 定时器配置文件
 * 
 * 默认禁用，需要时手动启用
 */

return [
    // 是否启用定时器服务
    'enable' => env('TIMER_ENABLE', false),
    
    // 时间轮精度（毫秒）
    'tick_ms' => env('TIMER_TICK_MS', 100),
    
    // 每层槽数
    'slots_per_wheel' => env('TIMER_SLOTS_PER_WHEEL', 60),
    
    // 层数
    'wheels' => env('TIMER_WHEELS', 4),
    
    // 最大任务数
    'max_tasks' => env('TIMER_MAX_TASKS', 10000),
];
"#.to_string()
}

/// 生成 crypt.php 配置
fn generate_crypt_config() -> String {
    // 生成随机密钥
    let key = crate::security::crypto::random_hex(32);
    
    format!(r#"<?php
/**
 * 加密配置文件
 * 
 * 用于 AES-256-GCM 加密解密
 * 密钥必须是 32 字节（64 个十六进制字符）
 */

return [
    // 加密密钥（请妥善保管，不要泄露）
    'key' => env('APP_KEY', '{}'),
    
    // 加密算法
    'cipher' => 'AES-256-GCM',
];
"#, key)
}

/// 生成 cors.php 配置
fn generate_cors_config() -> String {
    r#"<?php
/**
 * CORS 跨域配置文件
 */

return [
    // 允许的源
    'allowed_origins' => ['*'],
    
    // 允许的源模式
    'allowed_origins_patterns' => [],
    
    // 允许的请求头
    'allowed_headers' => ['*'],
    
    // 允许的请求方法
    'allowed_methods' => ['*'],
    
    // 是否允许携带凭证
    'supports_credentials' => false,
    
    // 预检请求缓存时间（秒）
    'max_age' => 0,
    
    // 暴露的响应头
    'exposed_headers' => [],
];
"#.to_string()
}

/// 生成 template.php 配置
fn generate_template_config() -> String {
    r#"<?php
/**
 * 模板配置文件
 */

return [
    // 模板引擎：native（原生）、tera
    'engine' => env('TEMPLATE_ENGINE', 'native'),
    
    // 模板目录
    'view_dir' => env('TEMPLATE_VIEW_DIR', 'app/view'),
    
    // 模板后缀
    'view_suffix' => env('TEMPLATE_SUFFIX', '.html'),
    
    // 模板主题
    'theme' => env('TEMPLATE_THEME', 'default'),
    
    // 是否开启模板缓存
    'cache_enabled' => env('TEMPLATE_CACHE', true),
    
    // 模板标签配置
    'taglib_begin' => '{',
    'taglib_end' => '}',
];
"#.to_string()
}

/// 生成 i18n.php 配置
fn generate_i18n_config() -> String {
    r#"<?php
/**
 * 国际化配置文件
 */

return [
    // 默认语言
    'default_locale' => env('APP_LOCALE', 'zh-cn'),
    
    // 备用语言
    'fallback_locale' => 'en',
    
    // 语言包目录
    'lang_dir' => 'lang',
    
    // 支持的语言列表
    'available_locales' => [
        'zh-cn' => '简体中文',
        'en' => 'English',
    ],
];
"#.to_string()
}

/// 生成 event.php 配置
fn generate_event_config() -> String {
    r#"<?php
/**
 * 事件配置文件
 * 
 * 定义事件与监听器的映射关系
 */

return [
    // 事件监听器映射
    'listen' => [
        // 'UserCreated' => [
        //     'app\listener\UserCreated',
        // ],
    ],
    
    // 事件订阅者
    'subscribe' => [
        // 'app\subscribe\UserEventSubscriber',
    ],
];
"#.to_string()
}

/// 生成 middleware.php 配置
fn generate_middleware_config() -> String {
    r#"<?php
/**
 * 中间件配置文件
 */

return [
    // 全局中间件
    'global' => [
        // \app\middleware\Cors::class,
    ],
    
    // 路由中间件
    'route' => [
        // 'auth' => \app\middleware\Auth::class,
    ],
];
"#.to_string()
}

/// 生成 filesystem.php 配置
fn generate_filesystem_config() -> String {
    r#"<?php
/**
 * 文件系统配置文件
 * 
 * 支持多种存储驱动：local、s3、oss、cos、ftp
 * 默认使用本地文件存储
 */

return [
    // 默认文件系统磁盘
    'default' => env('FILESYSTEM_DRIVER', 'local'),
    
    // 文件系统磁盘配置
    'disks' => [
        // 本地存储（默认）
        'local' => [
            'driver' => 'local',
            'root' => env('FILESYSTEM_ROOT', 'storage/app'),
            'url' => env('FILESYSTEM_URL', '/storage'),
            'visibility' => 'public',
        ],
        
        // 公共存储（用于公开访问的文件）
        'public' => [
            'driver' => 'local',
            'root' => 'storage/app/public',
            'url' => '/storage',
            'visibility' => 'public',
        ],
        
        // AWS S3 存储（需要配置）
        // 's3' => [
        //     'driver' => 's3',
        //     'key' => env('AWS_ACCESS_KEY_ID'),
        //     'secret' => env('AWS_SECRET_ACCESS_KEY'),
        //     'region' => env('AWS_DEFAULT_REGION', 'us-east-1'),
        //     'bucket' => env('AWS_BUCKET'),
        //     'url' => env('AWS_URL'),
        //     'endpoint' => env('AWS_ENDPOINT'),
        //     'use_https' => true,
        //     'path_style' => false,
        // ],
        
        // 阿里云 OSS 存储（需要配置）
        // 'oss' => [
        //     'driver' => 'oss',
        //     'access_key' => env('OSS_ACCESS_KEY_ID'),
        //     'secret_key' => env('OSS_ACCESS_KEY_SECRET'),
        //     'bucket' => env('OSS_BUCKET'),
        //     'endpoint' => env('OSS_ENDPOINT', 'oss-cn-hangzhou.aliyuncs.com'),
        //     'custom_domain' => env('OSS_CUSTOM_DOMAIN'),
        //     'use_https' => true,
        // ],
        
        // 腾讯云 COS 存储（需要配置）
        // 'cos' => [
        //     'driver' => 'cos',
        //     'secret_id' => env('COS_SECRET_ID'),
        //     'secret_key' => env('COS_SECRET_KEY'),
        //     'bucket' => env('COS_BUCKET'),
        //     'region' => env('COS_REGION', 'ap-guangzhou'),
        //     'custom_domain' => env('COS_CUSTOM_DOMAIN'),
        //     'use_https' => true,
        // ],
        
        // FTP 存储（需要配置）
        // 'ftp' => [
        //     'driver' => 'ftp',
        //     'host' => env('FTP_HOST'),
        //     'port' => env('FTP_PORT', 21),
        //     'username' => env('FTP_USERNAME'),
        //     'password' => env('FTP_PASSWORD'),
        //     'root' => env('FTP_ROOT', '/'),
        //     'passive' => true,
        //     'ssl' => false,
        //     'timeout' => 30,
        // ],
    ],
];
"#.to_string()
}

/// 创建 .env 文件
fn create_env_file(project: &project::detector::Project) -> Result<()> {
    let content = r#"# ===========================================
# OYTAPHP 环境变量配置文件
# ===========================================

# 应用配置
APP_NAME=OYTAPHP
APP_ENV=development
APP_DEBUG=true
APP_URL=http://localhost

# 服务器配置
APP_HOST=0.0.0.0
APP_PORT=8000
APP_WORKERS=2

# 数据库配置
DB_CONNECTION=mysql
DB_HOST=127.0.0.1
DB_PORT=3306
DB_NAME=test
DB_USER=root
DB_PASS=
DB_PREFIX=

# Redis 配置
REDIS_HOST=127.0.0.1
REDIS_PORT=6379
REDIS_PASSWORD=
REDIS_DATABASE=0
"#;

    std::fs::write(&project.env_file, content)?;

    // 创建 .env.example
    std::fs::write(&project.example_env_file, content)?;

    Ok(())
}

/// 创建 composer.json 文件
fn create_composer_json(project: &project::detector::Project) -> Result<()> {
    let content = r#"{
    "name": "oyta/app",
    "type": "project",
    "description": "OYTAPHP Application",
    "keywords": ["oyta", "php", "framework", "thinkphp"],
    "license": "MIT",
    "require": {
        "php": ">=8.0"
    },
    "autoload": {
        "psr-4": {
            "app\\": "app/"
        }
    },
    "config": {
        "optimize-autoloader": true,
        "preferred-install": "dist",
        "sort-packages": true
    },
    "minimum-stability": "stable",
    "prefer-stable": true
}
"#;

    std::fs::write(&project.composer_json, content)?;

    Ok(())
}

/// 创建默认控制器
/// 默认创建单应用模式（app/controller/Index.php）
/// 如果指定了 app_name，则创建多应用模式（app/{app_name}/controller/Index.php）
fn create_default_controller(project: &project::detector::Project, app_name: Option<&str>) -> Result<()> {
    // 控制器路径
    let controller_path = if let Some(name) = app_name {
        // 多应用模式
        project.root
            .join("app")
            .join(name)
            .join("controller")
            .join("Index.php")
    } else {
        // 单应用模式
        project.root
            .join("app")
            .join("controller")
            .join("Index.php")
    };

    let content = if let Some(name) = app_name {
        // 多应用模式
        format!(r#"<?php

namespace app\{}\controller;

/**
 * 默认控制器
 */
class Index
{{
    /**
     * 首页
     */
    public function index()
    {{
        return 'Hello, OYTAPHP!';
    }}
}}
"#, name)
    } else {
        // 单应用模式
        r#"<?php

namespace app\controller;

/**
 * 默认控制器
 */
class Index
{
    /**
     * 首页
     */
    public function index()
    {
        return 'Hello, OYTAPHP!';
    }
}
"#.to_string()
    };

    std::fs::write(&controller_path, content)?;

    // 创建路由文件
    let route_path = project.root.join("route").join("app.php");
    let route_content = r#"<?php
/**
 * 路由定义文件
 */

use oyta\facade\Route;

// 在此定义你的路由
// Route::get('hello/:name', 'index/hello');
"#;

    std::fs::write(&route_path, route_content)?;

    Ok(())
}

/// 处理 new 命令
///
/// 创建新的 OYTAPHP 项目
/// 在当前目录下创建一个完整的项目目录结构
///
/// # 参数
/// - `project_name`: 项目名称
/// - `app_name`: 应用名称（可选，默认为 index）
pub async fn handle_new(project_name: &str, app_name: Option<&str>) -> Result<()> {
    // 支持绝对路径和相对路径
    let project_root = if std::path::Path::new(project_name).is_absolute() {
        std::path::PathBuf::from(project_name)
    } else {
        let current_dir = std::env::current_dir()?;
        current_dir.join(project_name)
    };

    // 检查目录是否已存在
    if project_root.exists() {
        return Err(anyhow::anyhow!("目录 '{}' 已存在", project_root.display()));
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  OYTAPHP 创建新项目: {}", project_root.display());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // 创建项目结构
    create_project_structure(&project_root, app_name)?;

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  项目创建完成！");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("  进入项目目录:");
    println!("    cd {}", project_root.display());
    println!();
    println!("  启动服务器:");
    println!("    oyta run");
    println!();

    Ok(())
}

/// 创建完整的项目结构
/// 默认创建单应用模式（app/controller/）
/// 如果指定了 --app 参数，则创建多应用模式（app/{app_name}/controller/）
fn create_project_structure(project_root: &std::path::Path, app_name: Option<&str>) -> Result<()> {
    // 创建项目根目录
    std::fs::create_dir_all(project_root)?;

    // 判断是单应用还是多应用模式
    let is_multi_app = app_name.is_some();
    
    // 应用目录
    let app_dir = if let Some(name) = app_name {
        // 多应用模式: app/{app_name}/
        project_root.join("app").join(name)
    } else {
        // 单应用模式: app/
        project_root.join("app")
    };
    
    std::fs::create_dir_all(app_dir.join("controller"))?;
    std::fs::create_dir_all(app_dir.join("model"))?;
    std::fs::create_dir_all(app_dir.join("middleware"))?;
    std::fs::create_dir_all(app_dir.join("validate"))?;
    std::fs::create_dir_all(app_dir.join("view"))?;

    // 配置目录
    let config_dir = project_root.join("config");
    std::fs::create_dir_all(&config_dir)?;

    // 路由目录
    std::fs::create_dir_all(project_root.join("route"))?;

    // 运行时目录
    let runtime_dir = project_root.join("runtime");
    std::fs::create_dir_all(runtime_dir.join("cache"))?;
    std::fs::create_dir_all(runtime_dir.join("log"))?;
    std::fs::create_dir_all(runtime_dir.join("temp"))?;
    std::fs::create_dir_all(runtime_dir.join("session"))?;

    // 公共目录
    let public_dir = project_root.join("public");
    std::fs::create_dir_all(&public_dir)?;
    std::fs::create_dir_all(public_dir.join("static"))?;

    // 存储目录
    std::fs::create_dir_all(project_root.join("storage").join("app").join("public"))?;

    // Vendor 目录
    std::fs::create_dir_all(project_root.join("vendor").join("bin"))?;

    println!("  ✓ 目录结构创建完成");

    // 生成配置文件
    println!("  ✓ 生成配置文件...");
    std::fs::write(config_dir.join("app.php"), generate_app_config())?;
    std::fs::write(config_dir.join("database.php"), generate_database_config())?;
    std::fs::write(config_dir.join("cache.php"), generate_cache_config())?;
    std::fs::write(config_dir.join("session.php"), generate_session_config())?;
    std::fs::write(config_dir.join("log.php"), generate_log_config())?;
    std::fs::write(config_dir.join("queue.php"), generate_queue_config())?;
    std::fs::write(config_dir.join("websocket.php"), generate_websocket_config())?;
    std::fs::write(config_dir.join("timer.php"), generate_timer_config())?;
    std::fs::write(config_dir.join("crypt.php"), generate_crypt_config())?;
    std::fs::write(config_dir.join("cors.php"), generate_cors_config())?;
    std::fs::write(config_dir.join("template.php"), generate_template_config())?;
    std::fs::write(config_dir.join("i18n.php"), generate_i18n_config())?;
    std::fs::write(config_dir.join("event.php"), generate_event_config())?;
    std::fs::write(config_dir.join("filesystem.php"), generate_filesystem_config())?;
    std::fs::write(config_dir.join("middleware.php"), generate_middleware_config())?;

    // 生成 .env 文件
    let env_content = r#"# 应用配置
APP_NAME=OYTAPHP
APP_ENV=development
APP_DEBUG=true
APP_URL=http://localhost

# 服务器配置
APP_HOST=0.0.0.0
APP_PORT=8000
APP_WORKERS=2

# 数据库配置
DB_CONNECTION=mysql
DB_HOST=127.0.0.1
DB_PORT=3306
DB_NAME=test
DB_USER=root
DB_PASS=
DB_PREFIX=

# Redis 配置
REDIS_HOST=127.0.0.1
REDIS_PORT=6379
REDIS_PASSWORD=
REDIS_DATABASE=0
"#;
    std::fs::write(project_root.join(".env"), env_content)?;
    std::fs::write(project_root.join(".env.example"), env_content)?;

    // 生成 composer.json
    let composer_content = r#"{
    "name": "oyta/app",
    "type": "project",
    "description": "OYTAPHP Application",
    "keywords": ["oyta", "php", "framework", "thinkphp"],
    "license": "MIT",
    "require": {
        "php": ">=8.0"
    },
    "autoload": {
        "psr-4": {
            "app\\": "app/"
        }
    },
    "config": {
        "optimize-autoloader": true,
        "preferred-install": "dist",
        "sort-packages": true
    }
}
"#;
    std::fs::write(project_root.join("composer.json"), composer_content)?;

    // 创建默认控制器
    let controller_content = if let Some(name) = app_name {
        // 多应用模式
        format!(r#"<?php

namespace app\{}\controller;

/**
 * 默认控制器
 */
class Index
{{
    /**
     * 首页
     */
    public function index()
    {{
        return 'Hello, {}!';
    }}
}}
"#, name, name)
    } else {
        // 单应用模式
        r#"<?php

namespace app\controller;

/**
 * 默认控制器
 */
class Index
{
    /**
     * 首页
     */
    public function index()
    {
        return 'Hello, OYTAPHP!';
    }
}
"#.to_string()
    };
    std::fs::write(app_dir.join("controller").join("Index.php"), controller_content)?;

    // 创建路由文件
    std::fs::write(
        project_root.join("route").join("app.php"),
        r#"<?php

use oyta\facade\Route;

// 在此定义你的路由
// Route::get('hello/:name', 'index/hello');
"#,
    )?;

    println!("  ✓ 配置文件生成完成");

    Ok(())
}

/// 处理 install 命令
///
/// 将当前 oyta 安装到指定目录
/// 注意：用户通常应该通过 curl/apt 等方式安装，此命令用于特殊情况
///
/// # 参数
/// - `install_path`: 安装路径（默认为 /usr/local/bin）
pub async fn handle_install(install_path: &str) -> Result<()> {
    let install_dir = std::path::Path::new(install_path);
    let oyta_path = install_dir.join("oyta");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  OYTAPHP 安装到系统 PATH");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // 检查安装目录是否存在
    if !install_dir.exists() {
        return Err(anyhow::anyhow!(
            "安装目录 '{}' 不存在\n\n\
            提示: 如果您还没有安装 oyta，请使用以下方式安装:\n\
            \n\
              # Linux/macOS\n\
              curl -fsSL https://oyta.dev/install.sh | bash\n\
            \n\
              # Windows (PowerShell)\n\
              irm https://oyta.dev/install.ps1 | iex",
            install_path
        ));
    }

    // 获取当前二进制文件路径
    let current_exe = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("无法获取当前程序路径: {}", e))?;

    // 复制二进制文件到安装目录
    println!("  [1/2] 复制二进制文件...");
    std::fs::copy(&current_exe, &oyta_path)
        .map_err(|e| anyhow::anyhow!("复制失败: {}。请尝试使用 sudo 运行", e))?;

    // 设置可执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&oyta_path, std::fs::Permissions::from_mode(0o755))?;
    }

    println!("  ✓ 二进制文件已安装到: {}", oyta_path.display());

    println!();
    println!("  [2/2] 验证安装...");
    println!();
    println!("  安装完成！oyta 已添加到系统 PATH");
    println!();
    println!("  验证安装:");
    println!("    oyta --version");
    println!();
    println!("  创建新项目:");
    println!("    oyta new myapp");
    println!("    cd myapp");
    println!("    oyta run");
    println!();

    Ok(())
}

/// 处理 uninstall 命令
///
/// 从系统 PATH 卸载 oyta
///
/// # 参数
/// - `install_path`: 安装路径
pub async fn handle_uninstall(install_path: &str) -> Result<()> {
    let oyta_path = std::path::Path::new(install_path).join("oyta");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  OYTAPHP 系统卸载");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    if oyta_path.exists() {
        std::fs::remove_file(&oyta_path)
            .map_err(|e| anyhow::anyhow!("卸载失败: {}。请尝试使用 sudo 运行", e))?;
        println!("  ✓ 已从 {} 移除 oyta", install_path);
    } else {
        println!("  ! oyta 未安装在 {}", install_path);
    }

    println!();
    println!("  卸载完成！");
    println!();

    Ok(())
}
