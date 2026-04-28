# Session 模块

## 模块概述

Session 模块提供 OYTAPHP 的会话管理功能，对应 ThinkPHP 8.0 的 Session 门面。支持多种存储驱动，提供会话数据的存取和管理。

## 模块结构

```
session/
├── mod.rs          # 模块入口
├── facade.rs       # Session 门面
├── manager.rs      # Session 管理器
├── handler.rs      # Session 处理器
└── store/          # 存储驱动
    ├── file.rs     # 文件存储
    └── redis.rs    # Redis 存储
```

## Session 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `start` | session_id: &str | Result<()> | 启动 Session |
| `get` | key: &str | Option<String> | 获取 Session 值 |
| `set` | key: &str, value: &str | Result<()> | 设置 Session 值 |
| `delete` | key: &str | Result<()> | 删除 Session 值 |
| `has` | key: &str | bool | 检查 Session 值是否存在 |
| `clear` | - | Result<()> | 清空 Session |
| `destroy` | - | Result<()> | 销毁 Session |

## 使用示例

```php
<?php
use oyta\facade\Session;

// 启动 Session
Session::start(session_id());

// 设置 Session 值
Session::set('user_id', '1');
Session::set('user_name', '张三');

// 获取 Session 值
$userId = Session::get('user_id');

// 检查 Session 值是否存在
if (Session::has('user_id')) {
    echo "用户已登录";
}

// 删除 Session 值
Session::delete('user_id');

// 清空 Session
Session::clear();

// 销毁 Session
Session::destroy();
```

## 配置示例

```php
<?php
// config/session.php
return [
    'driver' => 'file',
    'name' => 'OYTASESSID',
    'expire' => 1440,
    'prefix' => 'oyta_',
    'path' => runtime_path('session'),
    'domain' => '',
    'secure' => false,
    'httponly' => true,
    'cookie' => [
        'lifetime' => 0,
        'path' => '/',
        'domain' => '',
        'secure' => false,
        'httponly' => true,
        'samesite' => 'Lax',
    ],
];
```

## 存储驱动

| 驱动 | 说明 | 适用场景 |
|------|------|----------|
| file | 文件存储 | 单机部署 |
| redis | Redis 存储 | 分布式部署 |

## 技术实现

- 使用 tokio::fs 进行异步文件操作
- 支持 Session 过期自动清理
- 线程安全的会话管理
