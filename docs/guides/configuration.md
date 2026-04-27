# 配置说明

## 概述

OYTAPHP 使用多源配置系统，支持 PHP 文件、环境变量、JSON、TOML 等多种配置格式。

## 配置文件

所有配置文件位于 `config/` 目录：

```
config/
├── app.php           # 应用配置
├── cache.php         # 缓存配置
├── database.php      # 数据库配置
├── log.php           # 日志配置
├── middleware.php    # 中间件配置
├── queue.php         # 队列配置
├── route.php         # 路由配置
├── session.php       # Session 配置
├── view.php          # 视图配置
└── provider.php      # 服务提供者
```

## 应用配置

`config/app.php`：

```php
<?php
return [
    // 应用名称
    'name' => env('APP_NAME', 'OYTAPHP'),
    
    // 应用环境
    'env' => env('APP_ENV', 'production'),
    
    // 调试模式
    'debug' => env('APP_DEBUG', false),
    
    // 时区
    'timezone' => 'Asia/Shanghai',
    
    // 默认语言
    'locale' => 'zh_CN',
    
    // 备用语言
    'fallback_locale' => 'en',
    
    // 加密密钥
    'key' => env('APP_KEY'),
    
    // 加密算法
    'cipher' => 'AES-256-GCM',
    
    // 服务提供者
    'providers' => [
        \app\provider\AppServiceProvider::class,
    ],
];
```

## 数据库配置

`config/database.php`：

```php
<?php
return [
    // 默认连接
    'default' => env('DB_CONNECTION', 'mysql'),
    
    // 连接配置
    'connections' => [
        'mysql' => [
            'driver' => 'mysql',
            'host' => env('DB_HOST', 'localhost'),
            'port' => env('DB_PORT', 3306),
            'database' => env('DB_DATABASE', 'myapp'),
            'username' => env('DB_USERNAME', 'root'),
            'password' => env('DB_PASSWORD', ''),
            'charset' => 'utf8mb4',
            'collation' => 'utf8mb4_unicode_ci',
            'prefix' => '',
            'strict' => true,
            'engine' => null,
            'pool' => [
                'max_connections' => 100,
                'min_connections' => 5,
                'connection_timeout' => 30,
                'idle_timeout' => 600,
            ],
        ],
        
        'pgsql' => [
            'driver' => 'pgsql',
            'host' => env('DB_HOST', 'localhost'),
            'port' => env('DB_PORT', 5432),
            'database' => env('DB_DATABASE', 'myapp'),
            'username' => env('DB_USERNAME', 'postgres'),
            'password' => env('DB_PASSWORD', ''),
            'charset' => 'utf8',
            'schema' => 'public',
        ],
        
        'sqlite' => [
            'driver' => 'sqlite',
            'database' => database_path('database.sqlite'),
            'prefix' => '',
        ],
    ],
    
    // 读写分离
    'read_write_split' => [
        'enabled' => false,
        'read_connections' => ['mysql_read'],
        'write_connection' => 'mysql',
    ],
    
    // 集群配置
    'clusters' => [
        'default' => [
            'nodes' => [
                ['host' => '192.168.1.1', 'port' => 3306],
                ['host' => '192.168.1.2', 'port' => 3306],
            ],
            'strategy' => 'round_robin',
        ],
    ],
];
```

## 缓存配置

`config/cache.php`：

```php
<?php
return [
    // 默认缓存驱动
    'default' => env('CACHE_DRIVER', 'redis'),
    
    // 缓存存储
    'stores' => [
        'redis' => [
            'driver' => 'redis',
            'connection' => 'cache',
            'lock_connection' => 'cache',
            'prefix' => env('CACHE_PREFIX', 'oyta:'),
        ],
        
        'file' => [
            'driver' => 'file',
            'path' => runtime_path('cache'),
        ],
        
        'memory' => [
            'driver' => 'memory',
            'max_capacity' => 10000,
            'ttl' => 3600,
        ],
        
        'multi' => [
            'driver' => 'multi',
            'l1' => 'memory',
            'l2' => 'redis',
        ],
    ],
    
    // 缓存预热
    'warmup' => [
        'enabled' => true,
        'concurrency' => 10,
        'schedule' => '0 3 * * *',
    ],
];
```

## 日志配置

`config/log.php`：

```php
<?php
return [
    // 默认日志通道
    'default' => env('LOG_CHANNEL', 'stack'),
    
    // 日志通道
    'channels' => [
        'stack' => [
            'driver' => 'stack',
            'channels' => ['daily', 'stderr'],
            'ignore_exceptions' => false,
        ],
        
        'daily' => [
            'driver' => 'daily',
            'path' => runtime_path('log/app.log'),
            'level' => env('LOG_LEVEL', 'debug'),
            'days' => 14,
            'permission' => 0644,
        ],
        
        'stderr' => [
            'driver' => 'stderr',
            'level' => env('LOG_LEVEL', 'debug'),
        ],
        
        'syslog' => [
            'driver' => 'syslog',
            'level' => env('LOG_LEVEL', 'debug'),
            'facility' => LOG_USER,
        ],
    ],
    
    // 日志格式
    'format' => '[{timestamp}] {level}: {message} {context}',
    
    // 时间格式
    'date_format' => 'Y-m-d H:i:s',
];
```

## Session 配置

`config/session.php`：

```php
<?php
return [
    // Session 驱动
    'driver' => env('SESSION_DRIVER', 'file'),
    
    // Session 生命周期（分钟）
    'lifetime' => env('SESSION_LIFETIME', 120),
    
    // 是否自动过期
    'expire_on_close' => false,
    
    // 加密 Session
    'encrypt' => false,
    
    // Session 文件路径
    'files' => runtime_path('session'),
    
    // Redis 连接
    'connection' => env('SESSION_CONNECTION', 'session'),
    
    // Session 表名（数据库驱动）
    'table' => 'sessions',
    
    // Session ID 长度
    'sid_length' => 32,
    
    // Cookie 配置
    'cookie' => [
        'name' => 'oyta_session',
        'path' => '/',
        'domain' => env('SESSION_DOMAIN'),
        'secure' => env('SESSION_SECURE_COOKIE'),
        'http_only' => true,
        'same_site' => 'lax',
    ],
];
```

## 队列配置

`config/queue.php`：

```php
<?php
return [
    // 默认队列连接
    'default' => env('QUEUE_CONNECTION', 'redis'),
    
    // 队列连接
    'connections' => [
        'redis' => [
            'driver' => 'redis',
            'connection' => 'queue',
            'queue' => env('REDIS_QUEUE', 'default'),
            'retry_after' => 90,
            'block_for' => null,
        ],
        
        'database' => [
            'driver' => 'database',
            'table' => 'jobs',
            'queue' => 'default',
            'retry_after' => 90,
        ],
        
        'sync' => [
            'driver' => 'sync',
        ],
    ],
    
    // 失败任务配置
    'failed' => [
        'driver' => env('QUEUE_FAILED_DRIVER', 'database'),
        'database' => env('DB_CONNECTION', 'mysql'),
        'table' => 'failed_jobs',
    ],
];
```

## 中间件配置

`config/middleware.php`：

```php
<?php
return [
    // 全局中间件
    'global' => [
        \oyta\middleware\CorsMiddleware::class,
        \oyta\middleware\TrustProxiesMiddleware::class,
    ],
    
    // 路由中间件
    'route' => [
        'auth' => \app\middleware\AuthMiddleware::class,
        'throttle' => \oyta\middleware\ThrottleMiddleware::class,
        'verified' => \app\middleware\VerifiedMiddleware::class,
    ],
    
    // 中间件组
    'groups' => [
        'web' => [
            \oyta\middleware\SessionMiddleware::class,
            \oyta\middleware\CsrfMiddleware::class,
        ],
        'api' => [
            'throttle:60,1',
        ],
    ],
];
```

## 视图配置

`config/view.php`：

```php
<?php
return [
    // 视图路径
    'paths' => [
        root_path('view'),
    ],
    
    // 编译缓存路径
    'compiled' => runtime_path('view'),
    
    // 模板引擎
    'engine' => 'tera',
    
    // Tera 配置
    'tera' => [
        'autoescape' => true,
        'strict_variables' => true,
    ],
];
```

## 服务提供者配置

`config/provider.php`：

```php
<?php
return [
    // 服务绑定
    'bindings' => [
        'cache' => \oyta\cache\CacheManager::class,
        'db' => \oyta\database\DatabaseManager::class,
        'log' => \oyta\logging\LogManager::class,
    ],
    
    // 单例绑定
    'singletons' => [
        'config' => \oyta\config\Config::class,
    ],
    
    // 服务提供者
    'providers' => [
        \app\provider\AppServiceProvider::class,
        \app\provider\EventServiceProvider::class,
    ],
];
```

## 环境变量

`.env` 文件：

```env
# 应用配置
APP_NAME=OYTAPHP
APP_ENV=local
APP_DEBUG=true
APP_KEY=base64:xxxxx

# 数据库配置
DB_CONNECTION=mysql
DB_HOST=127.0.0.1
DB_PORT=3306
DB_DATABASE=myapp
DB_USERNAME=root
DB_PASSWORD=

# Redis 配置
REDIS_HOST=127.0.0.1
REDIS_PASSWORD=null
REDIS_PORT=6379

# 缓存配置
CACHE_DRIVER=redis
CACHE_PREFIX=oyta:

# 队列配置
QUEUE_CONNECTION=redis

# Session 配置
SESSION_DRIVER=redis
SESSION_LIFETIME=120
```

## 访问配置

```php
use oyta\facade\Config;

// 获取配置
$value = Config::get('app.name');
$value = Config::get('database.connections.mysql.host', 'localhost');

// 检查配置
if (Config::has('app.timezone')) {
    // ...
}

// 设置配置
Config::set('app.debug', false);

// 获取所有配置
$all = Config::all();
```

## 配置缓存

生产环境建议缓存配置：

```bash
# 缓存配置
./oyta optimize:config

# 清除缓存
./oyta optimize:clear
```
