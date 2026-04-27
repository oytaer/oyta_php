<?php

/**
 * OYTAPHP 调试追踪配置文件
 * 
 * 调试模式下会显示详细的请求信息、SQL查询、内存使用等
 * 仅在开发环境启用，生产环境请关闭
 */

return [
    // 是否启用调试追踪
    'enable' => env('APP_DEBUG', false),
    
    // 追踪显示类型（html/console）
    'type' => 'html',
    
    // 显示的标签页
    'tabs' => ['base', 'file', 'sql', 'debug', 'user'],
    
    // 最大显示文件数
    'file_max' => 100,
];
