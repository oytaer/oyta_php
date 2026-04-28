# 事件模块 (Event)

## 模块概述

事件模块提供 OYTAPHP 的事件监听和触发功能，对应 ThinkPHP 8.0 的 Event 类。支持事件监听、事件触发、事件订阅等功能。

## 模块结构

```
event/
├── mod.rs          # 模块入口
├── facade.rs       # Event 门面
├── dispatcher.rs   # 事件调度器
└── types.rs        # 类型定义
```

## Event 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `dispatch` | event: &str, payload: Vec<Value> | - | 触发事件 |
| `listen` | event: &str, listener: &str | - | 注册监听器 |
| `on` | event: &str, listener: &str | - | 注册监听器（别名） |
| `has` | event: &str | bool | 检查事件是否有监听器 |
| `forget` | event: &str | - | 移除事件监听器 |

## 使用示例

### 定义事件

```php
<?php
namespace app\event;

class UserEvent
{
    public $user;
    
    public function __construct($user)
    {
        $this->user = $user;
    }
}
```

### 定义监听器

```php
<?php
namespace app\listener;

class SendEmailListener
{
    public function handle($event)
    {
        // 发送邮件逻辑
        mail($event->user['email'], '欢迎注册', '欢迎加入我们！');
    }
}
```

### 注册监听器

```php
<?php
// config/event.php
return [
    'listen' => [
        'UserRegistered' => [
            \app\listener\SendEmailListener::class,
            \app\listener\SendSmsListener::class,
        ],
    ],
    'subscribe' => [
        \app\subscribe\UserSubscribe::class,
    ],
];
```

### 触发事件

```php
<?php
use oyta\facade\Event;

// 触发事件
Event::dispatch('UserRegistered', [$user]);

// 或使用 on 方法注册监听器
Event::on('OrderPaid', \app\listener\UpdateInventoryListener::class);
```

### 事件订阅者

```php
<?php
namespace app\subscribe;

class UserSubscribe
{
    public function onUserRegistered($event)
    {
        // 处理用户注册事件
    }
    
    public function onUserLogin($event)
    {
        // 处理用户登录事件
    }
    
    public function subscribe()
    {
        return [
            'UserRegistered' => 'onUserRegistered',
            'UserLogin' => 'onUserLogin',
        ];
    }
}
```

## 配置示例

```php
<?php
// config/event.php
return [
    'listen' => [
        'UserRegistered' => [
            \app\listener\SendEmailListener::class,
        ],
    ],
    'subscribe' => [
        \app\subscribe\UserSubscribe::class,
    ],
];
```

## 技术实现

- 基于观察者模式
- 支持多个监听器
- 支持事件订阅者
- 线程安全的事件调度
