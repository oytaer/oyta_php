# Event 事件模块

## 模块结构

```
event/
├── mod.rs        # 模块入口
├── types.rs      # 事件类型定义
├── dispatcher.rs # 事件分发器
└── facade.rs     # 事件门面
```

## 基本使用

```php
<?php
// 定义事件
class UserRegistered
{
    public $user;
    
    public function __construct($user)
    {
        $this->user = $user;
    }
}

// 定义监听器
class SendWelcomeEmail
{
    public function handle($event)
    {
        Mail::to($event->user->email)->send(new WelcomeEmail());
    }
}

// 注册监听器
Event::listen(UserRegistered::class, SendWelcomeEmail::class);

// 触发事件
Event::dispatch(new UserRegistered($user));
```

## 订阅者

```php
<?php
class UserEventSubscriber
{
    public function onUserRegistered($event) {}
    public function onUserLogin($event) {}
    
    public function subscribe($events)
    {
        $events->listen(
            UserRegistered::class,
            [self::class, 'onUserRegistered']
        );
        
        $events->listen(
            UserLogin::class,
            [self::class, 'onUserLogin']
        );
    }
}

// 注册订阅者
Event::subscribe(UserEventSubscriber::class);
```
