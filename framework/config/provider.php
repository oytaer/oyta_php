<?php

/**
 * OYTAPHP 服务提供者配置文件
 * 
 * 服务提供者用于注册服务和绑定依赖
 * 在应用启动时会自动调用服务提供者的 register 方法
 * 
 * 使用示例：
 * // app\service\DatabaseService.php
 * class DatabaseService
 * {
 *     public function register()
 *     {
 *         App::bind('db', function () {
 *             return new DatabaseConnection();
 *         });
 *     }
 * }
 */

return [
    // 服务提供者列表
    // 按顺序注册，先注册的服务优先级更高
    // [
    //     'app\service\DatabaseService',
    //     'app\service\CacheService',
    // ],
];
