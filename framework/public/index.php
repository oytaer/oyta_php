<?php

/**
 * OYTAPHP 入口文件
 * 
 * 本文件是应用的入口点，用于兼容传统PHP框架的目录结构
 * 实际运行时，由 Rust 二进制文件解析和执行PHP代码
 * 
 * ============================================
 * 重要说明：
 * ============================================
 * 
 * OYTAPHP 是一个 Rust 驱动的 PHP 运行时，核心功能全部由 Rust 实现：
 * 
 * 1. PHP 解释器 - Rust 直接解析执行 PHP 代码
 * 2. 内置类库 - Controller、Model、Validate 等由 Rust 提供
 * 3. Facade 类 - Config、Cache、Session、Db、View 等由 Rust 实现
 * 4. 助手函数 - env()、config()、cache()、view() 等由 Rust 提供
 * 
 * 用户只需要编写业务代码（app/controller/*.php 等）
 * 
 * ============================================
 * 内嵌服务声明
 * ============================================
 * 
 * OYTAPHP 支持在入口文件中声明内嵌服务，无需单独启动
 * 
 * 支持的内嵌服务：
 * - \oyta\WebSocket  WebSocket 服务
 * - \oyta\Timer      定时任务服务
 * - \oyta\Queue      异步队列服务
 * 
 * @see https://oyta.dev/docs
 */

// ==================== 内嵌服务声明示例 ====================
// 取消注释以下代码以启用相应服务

// WebSocket 服务
// \oyta\WebSocket::start('/ws', function ($ws) {
//     $ws->on('open', function ($connection) {
//         $ws->join('room_1', $connection);
//     });
//     $ws->on('message', function ($connection, $data) {
//         $ws->broadcast('room_1', $data);
//     });
//     $ws->on('close', function ($connection) {
//         $ws->leave('room_1', $connection);
//     });
// });

// 定时任务
// \oyta\Timer::every(60, function () {
//     // 每60秒执行一次
//     \oyta\facade\Cache::clear('temp');
// });

// \oyta\Timer::cron('0 2 * * *', function () {
//     // 每天凌晨2点执行（Cron表达式）
//     \app\service\CleanupService::run();
// });

// 异步队列
// \oyta\Queue::process('emails', function ($job) {
//     // 处理邮件队列任务
//     \app\service\MailService::send($job->data);
//     $job->ack(); // 确认任务完成
// });
