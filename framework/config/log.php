<?php

/**
 * OYTAPHP 日志配置文件
 * 
 * 支持多种日志驱动：file（文件）、stdout（标准输出）
 * 可配置日志级别、文件大小限制、保留天数等
 * 
 * 使用示例：
 * Log::info('用户登录', ['user_id' => 1]);
 * Log::error('数据库错误', ['error' => $e->getMessage()]);
 * Log::warning('缓存未命中', ['key' => 'config']);
 */

return [
    // 默认日志驱动
    'default' => 'file',
    
    // 日志通道配置
    'channels' => [
        // 文件日志驱动
        'file' => [
            'type' => 'file',                              // 驱动类型
            'path' => runtime_path() . 'log/',             // 日志存储路径
            'level' => ['info', 'error', 'warning', 'debug'],  // 记录的日志级别
            'max_files' => 30,                             // 最大保留文件数
            'file_size' => 20971520,                       // 单文件最大大小（20MB）
        ],
        
        // 标准输出日志驱动
        'stdout' => [
            'type' => 'stdout',
            'level' => ['info', 'error', 'warning', 'debug'],
        ],
    ],
    
    // 日志格式化模板
    // 支持变量：{date}、{level}、{message}、{context}
    'formatter' => '[{date}] [{level}] {message} {context}',
];
