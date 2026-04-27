<?php

/**
 * OYTAPHP 事件配置文件
 * 
 * 用于定义事件绑定、监听器和订阅者
 * 支持事件触发、监听和订阅模式
 * 
 * 使用示例：
 * // 触发事件
 * Event::trigger('UserLogin', $user);
 * 
 * // 监听事件
 * Event::listen('UserLogin', function($user) {
 *     Log::info('用户登录：' . $user->name);
 * });
 */

return [
    // 事件绑定（事件别名映射）
    // 将事件类映射到事件名称
    'bind' => [
        // 'UserLogin' => 'app\event\UserLogin',
        // 'UserRegister' => 'app\event\UserRegister',
    ],
    
    // 事件监听器
    // 一个事件可以有多个监听器，按顺序执行
    'listen' => [
        // 'UserLogin' => ['app\listener\UserLogin'],
        // 'UserRegister' => [
        //     'app\listener\SendWelcomeEmail',
        //     'app\listener\InitUserStats',
        // ],
    ],
    
    // 事件订阅者
    // 订阅者可以在一个类中监听多个事件
    'subscribe' => [
        // 'app\subscribe\User',
    ],
];
