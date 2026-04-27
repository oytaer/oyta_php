<?php

/**
 * OYTAPHP API路由定义文件
 * 
 * 本文件用于定义API相关的路由规则
 * 通常用于前后端分离项目的API接口
 * 
 * RESTful风格示例：
 * - GET    /api/user      获取用户列表
 * - POST   /api/user      创建用户
 * - GET    /api/user/:id  获取单个用户
 * - PUT    /api/user/:id  更新用户
 * - DELETE /api/user/:id  删除用户
 */

use oyta\facade\Route;

// API路由分组
Route::group('api', function () {
    // 用户相关API
    Route::get('user', 'api.User/index');        // 获取用户列表
    Route::post('user', 'api.User/save');        // 创建用户
    Route::put('user/:id', 'api.User/update');   // 更新用户
    Route::delete('user/:id', 'api.User/delete'); // 删除用户
});
