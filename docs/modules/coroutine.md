# 协程模块

## 模块概述

协程模块实现了原生协程调度器，提供轻量级的并发编程能力，支持工作窃取调度、协程通道、同步原语等功能。

## 文件结构

```
coroutine/
├── mod.rs          # 模块入口
├── scheduler.rs    # 协程调度器
├── worker.rs       # 工作线程
├── task.rs         # 协程任务
├── channel.rs      # 协程通道
└── sync.rs         # 同步原语
```

## 核心设计

### 调度器架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        协程调度器                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                    全局任务队列                           │   │
│   │   ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐            │   │
│   │   │ T │ T │ T │ T │ T │ T │ T │ T │ T │ T │ ...        │   │
│   │   └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘            │   │
│   └─────────────────────────────────────────────────────────┘   │
│                              │                                    │
│              ┌───────────────┼───────────────┐                   │
│              │               │               │                   │
│              ▼               ▼               ▼                   │
│   ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │
│   │   Worker 0   │ │   Worker 1   │ │   Worker N   │            │
│   │ ┌──────────┐ │ │ ┌──────────┐ │ │ ┌──────────┐ │            │
│   │ │ 本地队列 │ │ │ │ 本地队列 │ │ │ │ 本地队列 │ │            │
│   │ │ T T T T  │ │ │ │ T T T    │ │ │ │ T T T T  │ │            │
│   │ └──────────┘ │ │ └──────────┘ │ │ └──────────┘ │            │
│   └──────────────┘ └──────────────┘ └──────────────┘            │
│         │                │                │                      │
│         └────────────────┴────────────────┘                      │
│                          │                                        │
│                   工作窃取 │                                        │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

### 工作窃取调度

当工作线程的本地队列为空时，会从其他线程窃取任务：

1. 先从本地队列获取任务
2. 本地队列为空，从全局队列获取
3. 全局队列也为空，从其他工作线程窃取

## 核心组件

### CoroutineScheduler

协程调度器：

```rust
pub struct CoroutineScheduler {
    /// 工作线程数量
    worker_count: usize,
    /// 工作线程列表
    workers: Vec<Arc<Worker>>,
    /// 全局任务队列
    global_queue: Arc<Mutex<VecDeque<CoroutineTask>>>,
    /// 协程 ID 计数器
    next_coroutine_id: AtomicU64,
    /// 是否正在运行
    running: AtomicBool,
}
```

### Worker

工作线程：

```rust
pub struct Worker {
    /// 工作线程 ID
    pub id: usize,
    /// 本地任务队列
    local_queue: Mutex<VecDeque<CoroutineTask>>,
    /// 调度器引用
    scheduler: Arc<CoroutineScheduler>,
    /// 线程句柄
    thread: Option<JoinHandle<()>>,
}
```

### CoroutineTask

协程任务：

```rust
pub struct CoroutineTask {
    /// 协程 ID
    pub id: u64,
    /// 协程名称
    pub name: String,
    /// 协程状态
    pub state: CoroutineState,
    /// 协程入口函数
    pub entry: Box<dyn FnOnce() + Send>,
    /// 协程栈大小
    pub stack_size: usize,
    /// 创建时间
    pub created_at: Instant,
    /// 优先级
    pub priority: Priority,
}

pub enum CoroutineState {
    /// 已创建
    Created,
    /// 等待执行
    Pending,
    /// 正在执行
    Running,
    /// 已挂起
    Suspended,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
}

pub enum Priority {
    Low,
    Normal,
    High,
    RealTime,
}
```

## 协程通道

### 有缓冲通道

```rust
pub struct BufferedChannel<T> {
    /// 缓冲区
    buffer: Mutex<VecDeque<T>>,
    /// 缓冲区容量
    capacity: usize,
    /// 发送等待队列
    send_waiters: Mutex<VecDeque<Condvar>>,
    /// 接收等待队列
    recv_waiters: Mutex<VecDeque<Condvar>>,
}
```

### 无缓冲通道

```rust
pub struct UnbufferedChannel<T> {
    /// 当前值
    value: Mutex<Option<T>>,
    /// 是否有值
    has_value: AtomicBool,
    /// 发送等待
    send_waiter: Mutex<Condvar>,
    /// 接收等待
    recv_waiter: Mutex<Condvar>,
}
```

## 同步原语

### 协程互斥锁

```rust
pub struct CoroutineMutex {
    /// 锁状态
    locked: AtomicBool,
    /// 等待队列
    waiters: Mutex<VecDeque<Condvar>>,
}
```

### 协程读写锁

```rust
pub struct CoroutineRwLock {
    /// 读计数
    read_count: AtomicUsize,
    /// 写锁状态
    write_locked: AtomicBool,
    /// 读等待队列
    read_waiters: Mutex<VecDeque<Condvar>>,
    /// 写等待队列
    write_waiters: Mutex<VecDeque<Condvar>>,
}
```

### 协程信号量

```rust
pub struct CoroutineSemaphore {
    /// 当前许可数
    permits: AtomicUsize,
    /// 最大许可数
    max_permits: usize,
    /// 等待队列
    waiters: Mutex<VecDeque<Condvar>>,
}
```

## API 使用

### 创建协程

```php
// 创建协程
$coroutineId = \oyta\Coroutine::create(function () {
    // 协程代码
    $result = doSomething();
    return $result;
});

// 创建带名称的协程
$coroutineId = \oyta\Coroutine::create('my_coroutine', function () {
    // 协程代码
});
```

### 协程通信

```php
// 创建通道
$channel = new \oyta\Channel(100); // 缓冲区大小 100

// 发送数据
$channel->push($data);

// 接收数据
$data = $channel->pop();

// 关闭通道
$channel->close();
```

### 协程同步

```php
// 创建互斥锁
$mutex = new \oyta\Coroutine\Mutex();

// 加锁
$mutex->lock();
try {
    // 临界区代码
} finally {
    // 解锁
    $mutex->unlock();
}

// 使用 tryLock
if ($mutex->tryLock()) {
    try {
        // 临界区代码
    } finally {
        $mutex->unlock();
    }
}
```

### 协程等待

```php
// 等待协程完成
$result = \oyta\Coroutine::join($coroutineId);

// 等待多个协程
$results = \oyta\Coroutine::joinAll([$id1, $id2, $id3]);

// 等待任意一个协程完成
$result = \oyta\Coroutine::select([$id1, $id2, $id3]);
```

## 性能特点

### 资源开销

| 项目 | 协程 | 线程 |
|------|------|------|
| 内存占用 | ~2KB | ~8MB |
| 创建开销 | ~1μs | ~100μs |
| 切换开销 | ~0.1μs | ~1μs |

### 并发能力

- 单机支持百万级协程
- 工作窃取实现负载均衡
- 无锁队列减少竞争

## 使用场景

### 并发请求

```php
$urls = ['http://api1.com', 'http://api2.com', 'http://api3.com'];

$results = \oyta\Coroutine::map($urls, function ($url) {
    return file_get_contents($url);
});
```

### 生产者-消费者

```php
$channel = new \oyta\Channel(100);

// 生产者
\oyta\Coroutine::create(function () use ($channel) {
    for ($i = 0; $i < 1000; $i++) {
        $channel->push($i);
    }
    $channel->close();
});

// 消费者
for ($i = 0; $i < 10; $i++) {
    \oyta\Coroutine::create(function () use ($channel) {
        while (($data = $channel->pop()) !== null) {
            process($data);
        }
    });
}
```

### 并行计算

```php
$results = \oyta\Coroutine::parallel([
    fn() => calculateA(),
    fn() => calculateB(),
    fn() => calculateC(),
]);
```
