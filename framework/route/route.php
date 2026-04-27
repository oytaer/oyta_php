<?php

/**
 * OYTAPHP 路由定义文件
 * 
 * 本文件用于定义URL路由规则
 * 路由将URL映射到对应的控制器方法
 * 
 * 注意：Route Facade 由 Rust 运行时提供
 * 
 * 路由类型：
 * - Route::get()      GET请求路由
 * - Route::post()     POST请求路由
 * - Route::put()      PUT请求路由
 * - Route::delete()   DELETE请求路由
 * - Route::rule()     任意请求类型路由
 * - Route::resource() RESTful资源路由
 * - Route::group()    路由分组
 * - Route::miss()     404路由
 * 
 * 参数绑定：
 * - :name     动态参数
 * - <id:\d+>  带正则约束的参数
 */

use oyta\facade\Route;

// 示例：GET /hello/xxx 映射到 IndexController::hello($name)
Route::get('hello/:name', 'index/hello');

// 404路由：未匹配到任何路由时执行
Route::miss(function () {
    return '404 Not Found';
});
