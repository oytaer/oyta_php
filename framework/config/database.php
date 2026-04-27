<?php

/**
 * OYTAPHP 数据库配置文件
 * 
 * 支持MySQL、PostgreSQL、SQLite等多种数据库
 * 内置连接池，支持读写分离和集群部署
 * 
 * 使用示例：
 * $users = Db::table('user')->where('status', 1)->select();
 * $user = User::find(1);
 */

return [
    // 默认数据库连接
    'default' => env('DB_TYPE', 'mysql'),
    
    // 数据库连接配置
    'connections' => [
        // MySQL连接配置
        'mysql' => [
            'type' => 'mysql',                              // 数据库类型
            'hostname' => env('DB_HOST', '127.0.0.1'),      // 服务器地址
            'database' => env('DB_NAME', 'test'),           // 数据库名
            'username' => env('DB_USER', 'root'),           // 用户名
            'password' => env('DB_PASS', ''),               // 密码
            'hostport' => env('DB_PORT', 3306),             // 端口
            'charset' => env('DB_CHARSET', 'utf8mb4'),      // 字符集
            'prefix' => env('DB_PREFIX', ''),               // 表前缀
            'debug' => env('APP_DEBUG', false),             // 调试模式（记录SQL）
            
            // 连接池配置
            'pool' => [
                'min_connections' => 1,                     // 最小连接数
                'max_connections' => 10,                    // 最大连接数
                'connect_timeout' => 10,                    // 连接超时（秒）
                'idle_timeout' => 60,                       // 空闲超时（秒）
            ],
        ],
        
        // PostgreSQL连接配置
        'pgsql' => [
            'type' => 'pgsql',
            'hostname' => env('DB_HOST', '127.0.0.1'),
            'database' => env('DB_NAME', 'test'),
            'username' => env('DB_USER', 'postgres'),
            'password' => env('DB_PASS', ''),
            'hostport' => env('DB_PORT', 5432),
            'charset' => env('DB_CHARSET', 'utf8'),
            'prefix' => env('DB_PREFIX', ''),
            'debug' => env('APP_DEBUG', false),
        ],
        
        // SQLite连接配置
        'sqlite' => [
            'type' => 'sqlite',
            'database' => root_path() . 'database.db',      // 数据库文件路径
            'debug' => env('APP_DEBUG', false),
        ],
    ],
];
