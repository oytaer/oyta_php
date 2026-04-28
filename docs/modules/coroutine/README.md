# 协程模块 (Coroutine)

## 模块概述

协程模块提供基于 Rust async/await 的原生协程支持，相比 PHP 原生协程性能提升 5x+，支持工作窃取调度。

## 模块结构

```
coroutine/
├── mod.rs          # 模块入口
├── scheduler.rs    # 协程调度器
├── task.rs         # 协程任务
├── channel.rs      # 协程通道
└── sync.rs         # 协程同步原语
```

## 主要类型

| 类型 | 说明 |
|------|------|
| CoroutineScheduler | 协程调度器 |
| CoroutineTask | 协程任务 |
| CoroutineChannel | 协程通道 |
| CoroutineMutex | 协程互斥锁 |
| CoroutineRwLock | 协程读写锁 |
| CoroutineSemaphore | 协程信号量 |

## Coroutine 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `create` | task: Fn | TaskId | 创建协程 |
| `go` | task: Fn | TaskId | 创建协程（别名） |
| `wait` | task_id: TaskId | Result<Value> | 等待协程完成 |

## 使用示例

### 创建协程

```php
<?php
use oyta\facade\Coroutine;

// 创建协程
$taskId = Coroutine::create(function() {
    // 协程任务
    sleep(1);
    return "任务完成";
});

// 等待协程完成
$result = Coroutine::wait($taskId);
echo $result; // 输出: 任务完成
```

### 并发执行

```php
<?php
use oyta\facade\Coroutine;

// 创建多个协程
$task1 = Coroutine::create(function() {
    sleep(1);
    return "任务1完成";
});

$task2 = Coroutine::create(function() {
    sleep(1);
    return "任务2完成";
});

// 并发等待
$result1 = Coroutine::wait($task1);
$result2 = Coroutine::wait($task2);
```

### 协程通道

```php
<?php
use oyta\facade\Coroutine;

// 创建通道
$channel = Coroutine::channel(10);

// 发送数据
$channel->send("数据");

// 接收数据
$data = $channel->receive();
```

### 协程同步

```php
<?php
use oyta\facade\Coroutine;

// 创建互斥锁
$mutex = Coroutine::mutex();

// 加锁
$mutex->lock();
try {
    // 临界区代码
} finally {
    $mutex->unlock();
}
```

## 调度器配置

```php
<?php
// config/coroutine.php
return [
    'worker_num' => 4,           // 工作线程数
    'max_coroutine' => 10000,    // 最大协程数
    'stack_size' => 8192,        // 栈大小
    'enable_work_stealing' => true, // 启用工作窃取
];
```

## 技术实现

- 基于 Rust async/await
- 工作窃取调度算法
- 支持协程同步原语
- 高性能协程通道
