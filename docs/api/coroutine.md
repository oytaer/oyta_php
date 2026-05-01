# Coroutine 协程模块

## 模块结构

```
coroutine/
├── mod.rs       # 模块入口
├── scheduler.rs # 协程调度器
├── task.rs      # 协程任务
├── sync.rs      # 同步原语
└── channel.rs   # 协程通道
```

## 基本使用

```php
<?php
// 创建协程
$coroutineId = Coroutine::create(function() {
    // 协程代码
    Coroutine::sleep(1000); // 毫秒
    echo "Hello from coroutine\n";
});

// 等待协程完成
Coroutine::join($coroutineId);

// 获取当前协程 ID
$id = Coroutine::id();

// 并发执行
$results = Coroutine::parallel([
    fn() => Http::get('http://api1.example.com'),
    fn() => Http::get('http://api2.example.com'),
    fn() => Http::get('http://api3.example.com'),
]);
```

## 协程通道

```php
<?php
// 创建通道
$channel = new Channel(100);

// 发送数据
$channel->push($data);

// 接收数据
$data = $channel->pop();

// 关闭通道
$channel->close();
```

## 协程锁

```php
<?php
$lock = new Coroutine\Lock();

$lock->lock();
try {
    // 临界区代码
} finally {
    $lock->unlock();
}
```

## 协程调度

```php
<?php
// 让出执行权
Coroutine::yield();

// 恢复执行
Coroutine::resume($coroutineId);

// 设置最大协程数
Coroutine::setMaxOpen(10000);
```
