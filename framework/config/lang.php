<?php

/**
 * OYTAPHP 多语言配置文件
 * 
 * 支持多语言切换，可通过URL参数、Cookie、Header等方式切换语言
 * 语言文件放置在 app/lang/ 目录下
 * 
 * 使用示例：
 * // 获取翻译
 * Lang::get('hello');
 * 
 * // 带变量翻译
 * Lang::get('welcome', ['name' => 'OYTAPHP']);
 */

return [
    // 默认语言
    'default_lang' => 'zh-cn',
    
    // 允许的语言列表
    'allow_lang_list' => ['zh-cn', 'en'],
    
    // 语言检测变量名（URL参数）
    'detect_var' => 'lang',
    
    // 语言Cookie变量名
    'cookie_var' => 'oyta_lang',
    
    // 语言Header变量名
    'header_var' => 'oyta-lang',
    
    // 扩展语言包
    'extend_list' => [],
    
    // Accept-Language映射
    'accept_language' => [
        'zh-cn' => 'zh-cn',
        'en' => 'en',
    ],
];
