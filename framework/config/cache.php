<?php

/**
 * OYTAPHP 缓存配置文件
 * 
 * 支持多种缓存驱动：file（文件）、redis、memory（内存）
 * 可配置多级缓存策略，实现高性能数据缓存
 */

return [
    // 默认缓存驱动
    'default' => 'file',
    
    // 缓存存储配置
    'stores' => [
        // 文件缓存驱动
        'file' => [
            'type' => 'file',                           // 驱动类型
            'path' => runtime_path() . 'cache/',        // 缓存存储路径
            'expire' => 0,                              // 默认过期时间（0表示永不过期）
        ],
        
        // Redis缓存驱动
        'redis' => [
            'type' => 'redis',                          // 驱动类型
            'host' => env('REDIS_HOST', '127.0.0.1'),   // Redis服务器地址
            'port' => env('REDIS_PORT', 6379),          // Redis端口
            'password' => env('REDIS_PASSWORD', ''),    // Redis密码
            'select' => 0,                              // 数据库索引
            'timeout' => 0,                             // 连接超时时间
            'persistent' => false,                      // 是否持久连接
            'prefix' => 'oyta_',                        // 缓存键前缀
        ],
        
        // 内存缓存驱动（使用moka高性能内存缓存）
        'memory' => [
            'type' => 'memory',                         // 驱动类型
            'max_capacity' => 10000,                    // 最大缓存条目数
            'ttl' => 3600,                              // 默认过期时间（秒）
        ],
    ],
];
