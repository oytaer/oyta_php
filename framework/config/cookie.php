<?php

/**
 * OYTAPHP Cookie配置文件
 * 
 * 配置Cookie的全局默认属性
 * 可在代码中通过 Cookie::set() 方法覆盖这些默认值
 */

return [
    // Cookie名称前缀
    'prefix' => 'oyta_',
    
    // Cookie默认过期时间（秒），0表示会话Cookie
    'expires' => 0,
    
    // Cookie有效路径
    'path' => '/',
    
    // Cookie有效域名
    'domain' => '',
    
    // 是否仅通过HTTPS传输
    'secure' => false,
    
    // 是否仅通过HTTP协议访问（防止JavaScript读取）
    'httponly' => true,
    
    // SameSite属性，防止CSRF攻击
    // 可选值：Strict、Lax、None
    'samesite' => 'Lax',
];
