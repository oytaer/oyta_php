# Session 会话模块

## 模块概述

Session 模块提供会话管理功能，支持多种存储驱动，包括内存、文件和 Redis。对应 ThinkPHP 8.0 的 Session 功能。

## 模块结构

```
session/
├── mod.rs       # 模块入口
├── driver.rs    # 会话驱动
├── manager.rs   # 会话管理器
└── facade.rs    # Session 门面
```

## 支持的驱动

### 1. 内存驱动

适合开发环境，数据存储在内存中。

### 2. 文件驱动

将会话数据持久化到文件系统。

### 3. Redis 驱动

支持分布式部署，使用 Redis 存储会话数据。

## 基本使用

```php
<?php
// 获取 Session 值
$value = Session::get('key');
$value = Session::get('key', 'default');

// 设置 Session 值
Session::set('key', 'value');

// 删除 Session 值
Session::delete('key');

// 检查是否存在
if (Session::has('key')) {
    // ...
}

// 清空 Session
Session::clear();

// 销毁 Session
Session::destroy();
```

## Flash 消息

```php
<?php
// 设置 flash 消息（只读一次）
Session::flash('message', '操作成功');

// 获取 flash 消息
$message = Session::getFlash('message');

// 保留 flash 消息
Session::reflash();

// 仅保留指定键
Session::keep(['message', 'error']);
```

## Session ID 管理

```php
<?php
// 获取 Session ID
$id = Session::getId();

// 重新生成 Session ID
Session::regenerate();

// 重新生成并删除旧数据
Session::regenerate(true);
```

## 配置示例

```php
<?php
// config/session.php
return [
    'type' => 'file',
    
    'store' => [
        'file' => [
            'path' => runtime_path('session'),
            'expire' => 7200,
        ],
        
        'redis' => [
            'host' => env('REDIS_HOST', '127.0.0.1'),
            'port' => env('REDIS_PORT', 6379),
            'password' => env('REDIS_PASSWORD', ''),
            'select' => 0,
            'prefix' => 'session:',
            'expire' => 7200,
        ],
    ],
    
    'name' => 'PHPSESSID',
    'cookie' => [
        'lifetime' => 7200,
        'path' => '/',
        'domain' => '',
        'secure' => false,
        'httponly' => true,
        'samesite' => 'Lax',
    ],
];
```
