<?php

/**
 * OYTAPHP 中间件配置文件
 * 
 * 配置全局中间件和中间件别名
 * 中间件按照洋葱模型执行，支持前置和后置处理
 * 
 * 中间件类型：
 * - 全局中间件：所有请求都会执行
 * - 路由中间件：在路由定义时指定
 * - 控制器中间件：在控制器中定义
 * 
 * 使用示例：
 * // 定义路由中间件
 * Route::get('admin', 'admin.Index/index')->middleware('auth');
 */

return [
    // 全局中间件
    // 所有请求都会执行这些中间件
    'global' => [
        // \oyta\middleware\SessionInit::class,
        // \oyta\middleware\LoadLangPack::class,
    ],
    
    // 中间件别名
    // 在路由中可以使用别名引用中间件
    'alias' => [
        // 'auth' => \app\middleware\Auth::class,
        // 'cors' => \oyta\middleware\CORS::class,
    ],
];
