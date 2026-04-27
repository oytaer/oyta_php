<?php

/**
 * OYTAPHP 文件系统配置文件
 * 
 * 配置文件存储磁盘，支持本地存储和云存储
 * 
 * 使用示例：
 * // 存储文件
 * Filesystem::put('test.txt', 'content');
 * 
 * // 读取文件
 * $content = Filesystem::get('test.txt');
 * 
 * // 上传文件
 * $path = $request->file('avatar')->store('uploads');
 */

return [
    // 默认磁盘
    'default' => 'local',
    
    // 磁盘配置
    'disks' => [
        // 本地存储磁盘
        'local' => [
            'type' => 'local',                             // 驱动类型
            'root' => runtime_path() . 'storage/',         // 存储根目录
        ],
        
        // 公共存储磁盘（用于用户上传文件）
        'public' => [
            'type' => 'local',
            'root' => public_path() . 'uploads/',          // 存储根目录
            'url' => '/uploads',                           // 公开访问URL
        ],
    ],
];
