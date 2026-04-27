<?php

/**
 * OYTAPHP 应用配置文件
 * 
 * 本文件包含应用的基础配置项，如时区、默认语言、默认控制器等
 * 所有配置项都可以通过 .env 文件进行覆盖
 */

return [
    // 默认时区
    'default_timezone' => 'Asia/Shanghai',
    
    // 默认语言
    'default_lang' => 'zh-cn',
    
    // 默认模块（多应用模式有效）
    'default_module' => 'index',
    
    // 默认控制器
    'default_controller' => 'Index',
    
    // 默认操作方法
    'default_action' => 'index',
    
    // 默认返回类型（html/json/xml）
    'default_return_type' => 'html',
    
    // 默认AJAX请求返回类型
    'default_ajax_return' => 'json',
    
    // JSONP处理的回调函数名
    'default_jsonp_handler' => 'jsonpReturn',
    
    // JSONP处理的回调参数名
    'default_jsonp_handler_param' => 'callback',
    
    // 默认过滤器（全局过滤函数）
    'default_filter' => null,
    
    // 默认异常处理类（为空则使用框架内置）
    'default_exception_handler' => '',
    
    // 语言切换变量名（URL参数）
    'default_lang_switch_var' => 'lang',
    
    // 语言自动检测变量名
    'default_lang_detect_var' => 'lang',
    
    // 语言Cookie变量名
    'default_lang_cookie_var' => 'oyta_lang',
    
    // 语言Header变量名
    'default_lang_header_var' => 'oyta-lang',
    
    // 语言Cookie有效期（秒）
    'default_lang_cookie_expire' => 31536000,
    
    // 语言Cookie域名
    'default_lang_cookie_domain' => '',
    
    // 语言Cookie路径
    'default_lang_cookie_path' => '/',
    
    // 语言Cookie是否仅HTTPS
    'default_lang_cookie_secure' => false,
    
    // 语言Cookie是否HttpOnly
    'default_lang_cookie_httponly' => false,
    
    // 语言Cookie SameSite属性
    'default_lang_cookie_samesite' => 'Lax',
];
