<?php

/**
 * OYTAPHP Session配置文件
 * 
 * 支持多种Session驱动：file（文件）、redis、cache
 * Session数据会自动加密存储，确保安全性
 * 
 * 使用示例：
 * // 设置Session
 * Session::set('user_id', 1);
 * 
 * // 获取Session
 * $userId = Session::get('user_id');
 * 
 * // 删除Session
 * Session::delete('user_id');
 * 
 * // 闪存数据（下次请求后自动删除）
 * Session::flash('message', '操作成功');
 */

return [
    // Session驱动类型
    // 可选：file、redis、cache
    'type' => 'file',
    
    // 使用的缓存存储（当type为cache时有效）
    'store' => null,
    
    // Session有效期（秒）
    'expire' => 1440,
    
    // Session名称前缀
    'prefix' => '',
    
    // 是否自动启动Session
    'auto_start' => true,
    
    // Session Cookie配置
    'cookie' => [
        'name' => 'OYTA_SESSION_ID',       // Cookie名称
        'lifetime' => 1440,                 // 有效期（秒）
        'path' => '/',                      // 有效路径
        'domain' => '',                     // 有效域名
        'secure' => false,                  // 仅HTTPS传输
        'httponly' => true,                 // 仅HTTP访问
        'samesite' => 'Lax',                // SameSite属性
    ],
];
