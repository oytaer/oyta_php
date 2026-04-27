# 定时器模块

## 模块概述

定时器模块实现了高性能的多级时间轮定时器，支持一次性任务、周期性任务、延迟任务等多种定时场景。

## 文件结构

```
timer/
├── mod.rs          # 模块入口
├── wheel.rs        # 时间轮核心实现
├── level.rs        # 时间轮层级
├── slot.rs         # 时间槽位
├── task.rs         # 定时任务定义
├── scheduler.rs    # 任务调度器
├── callback.rs     # 回调函数封装
├── stats.rs        # 统计信息
└── error.rs        # 错误定义
```

## 核心设计

### 多级时间轮架构

采用 4 级时间轮设计，覆盖从毫秒到小时的时间范围：

```
┌─────────────────────────────────────────────────────────────┐
│                      多级时间轮                               │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│   Level 0: 毫秒级 (1ms - 999ms)                               │
│   ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐                │
│   │ 0 │ 1 │ 2 │ 3 │...│997│998│999│   │   │  1000 槽位     │
│   └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘                │
│                                                               │
│   Level 1: 秒级 (1s - 59s)                                    │
│   ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐                │
│   │ 0 │ 1 │ 2 │ 3 │...│57 │58 │59 │   │   │  60 槽位       │
│   └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘                │
│                                                               │
│   Level 2: 分钟级 (1m - 59m)                                  │
│   ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐                │
│   │ 0 │ 1 │ 2 │ 3 │...│57 │58 │59 │   │   │  60 槽位       │
│   └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘                │
│                                                               │
│   Level 3: 小时级 (1h - 24h)                                  │
│   ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐                │
│   │ 0 │ 1 │ 2 │ 3 │...│22 │23 │24 │   │   │  25 槽位       │
│   └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘                │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### 时间轮层级

| 层级 | 时间范围 | 槽位数量 | 精度 |
|------|----------|----------|------|
| Level 0 | 1ms - 999ms | 1000 | 1ms |
| Level 1 | 1s - 59s | 60 | 1s |
| Level 2 | 1m - 59m | 60 | 1m |
| Level 3 | 1h - 24h | 25 | 1h |

## 核心组件

### TimingWheel

时间轮核心实现：

```rust
pub struct TimingWheel {
    /// 4 级时间轮
    levels: [Level; 4],
    /// 当前时间戳
    current_time: AtomicU64,
    /// 任务计数器
    task_count: AtomicU64,
    /// 是否正在运行
    running: AtomicBool,
}
```

### TimerTask

定时任务定义：

```rust
pub struct TimerTask {
    /// 任务唯一 ID
    pub id: u64,
    /// 任务名称
    pub name: String,
    /// 延迟时间（毫秒）
    pub delay_ms: u64,
    /// 是否周期性
    pub is_periodic: bool,
    /// 周期间隔（毫秒）
    pub period_ms: Option<u64>,
    /// 回调函数
    pub callback: TimerCallback,
    /// 任务状态
    pub state: TaskState,
    /// 创建时间
    pub created_at: u64,
    /// 下次执行时间
    pub next_execute_time: u64,
}
```

### TaskState

任务状态：

```rust
pub enum TaskState {
    /// 等待执行
    Pending,
    /// 正在执行
    Running,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 执行失败
    Failed,
}
```

## API 使用

### 添加定时任务

```rust
// 一次性任务（延迟 1000ms 后执行）
let task_id = wheel.add_task(TimerTask {
    name: "one_time_task".to_string(),
    delay_ms: 1000,
    is_periodic: false,
    callback: TimerCallback::Sync(|| {
        println!("One time task executed!");
    }),
    ..Default::default()
});

// 周期性任务（每 5000ms 执行一次）
let task_id = wheel.add_task(TimerTask {
    name: "periodic_task".to_string(),
    delay_ms: 5000,
    is_periodic: true,
    period_ms: Some(5000),
    callback: TimerCallback::Sync(|| {
        println!("Periodic task executed!");
    }),
    ..Default::default()
});
```

### 取消任务

```rust
wheel.cancel_task(task_id);
```

### 启动/停止时间轮

```rust
// 启动
wheel.start();

// 停止
wheel.stop();
```

## 性能特点

### 时间复杂度

| 操作 | 时间复杂度 |
|------|------------|
| 添加任务 | O(1) |
| 取消任务 | O(1) |
| 查找到期任务 | O(1) 平均 |

### 内存效率

- 槽位按需分配，不预分配所有槽位
- 任务使用链表存储，支持大量任务
- 使用 Arc 共享任务，减少内存复制

## 统计信息

```rust
pub struct TimerStats {
    /// 总任务数
    pub total_tasks: u64,
    /// 已完成任务数
    pub completed_tasks: u64,
    /// 已取消任务数
    pub cancelled_tasks: u64,
    /// 执行失败任务数
    pub failed_tasks: u64,
    /// 当前待执行任务数
    pub pending_tasks: u64,
    /// 平均执行时间（微秒）
    pub avg_execution_time_us: u64,
}
```

## 使用场景

### 内嵌定时器

在 `public/index.php` 中声明定时任务：

```php
// 每 60 秒执行一次
\oyta\Timer::every(60, function () {
    \oyta\facade\Cache::clear('temp');
});

// Cron 表达式：每天凌晨 2 点执行
\oyta\Timer::cron('0 2 * * *', function () {
    \app\service\CleanupService::run();
});

// 每天固定时间执行
\oyta\Timer::daily('03:00', function () {
    \app\service\ReportService::generate();
});

// 延迟 30 秒执行一次
\oyta\Timer::once(30, function () {
    \app\service\InitService::setup();
});
```

## 错误处理

```rust
pub enum TimerError {
    /// 任务已存在
    TaskAlreadyExists(u64),
    /// 任务不存在
    TaskNotFound(u64),
    /// 时间轮未启动
    WheelNotRunning,
    /// 时间轮已停止
    WheelAlreadyStopped,
    /// 无效的延迟时间
    InvalidDelay(u64),
    /// 回调执行失败
    CallbackFailed(String),
}
```
