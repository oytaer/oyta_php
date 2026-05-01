# Timer 定时器模块

## 模块结构

```
timer/
├── mod.rs       # 模块入口
└── scheduler.rs # 定时器调度器
```

## 基本使用

```php
<?php
// 延迟执行
$timerId = Timer::after(1000, function() {
    echo "1秒后执行\n";
});

// 定时执行
$timerId = Timer::tick(1000, function() {
    echo "每秒执行\n";
});

// 取消定时器
Timer::clear($timerId);

// 立即执行
Timer::defer(function() {
    echo "下一个事件循环执行\n";
});
```

## Cron 表达式

```php
<?php
// 使用 Cron 表达式
$timerId = Cron::add('*/5 * * * *', function() {
    echo "每5分钟执行\n";
});

// 移除 Cron 任务
Cron::remove($timerId);

// 列出所有任务
$tasks = Cron::list();

// 手动运行
Cron::run();
```
