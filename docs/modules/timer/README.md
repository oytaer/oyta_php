# 定时器模块 (Timer)

## 模块概述

定时器模块实现高性能的多级时间轮定时器（Hierarchical Timing Wheel），提供毫秒级精度的定时任务调度，支持从毫秒到天级别的超时范围。

## 模块结构

```
timer/
├── mod.rs          # 模块入口
├── wheel.rs        # 时间轮
├── level.rs        # 层级定义
├── slot.rs         # 槽位
├── task.rs         # 定时任务
├── scheduler.rs    # 调度器
├── callback.rs     # 回调处理
├── stats.rs        # 统计信息
└── error.rs        # 错误定义
```

## 架构设计

```
多级时间轮结构（4级层级设计）：

Level 0: 毫秒轮 (Millisecond Wheel)
[0][1][2][3]...[63]  ← 64 槽位，每槽 1ms，周期 64ms

Level 1: 秒轮 (Second Wheel)
[0][1][2]...[59]  ← 60 槽位，每槽 1s，周期 60s

Level 2: 分钟轮 (Minute Wheel)
[0][1][2]...[59]  ← 60 槽位，每槽 1min，周期 60min

Level 3: 小时轮 (Hour Wheel)
[0][1][2]...[23]  ← 24 槽位，每槽 1hour，周期 24h
```

## Timer 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `after` | delay_ms: u64, callback: Fn | TaskId | 一次性定时任务 |
| `every` | interval_ms: u64, callback: Fn | TaskId | 周期性定时任务 |
| `cancel` | task_id: TaskId | bool | 取消定时任务 |

## 使用示例

### 一次性任务

```php
<?php
use oyta\facade\Timer;

// 1秒后执行
$taskId = Timer::after(1000, function() {
    echo "1秒后执行";
});

// 取消任务
Timer::cancel($taskId);
```

### 周期性任务

```php
<?php
use oyta\facade\Timer;

// 每2秒执行一次
$taskId = Timer::every(2000, function() {
    echo "每2秒执行一次\n";
});
```

### 定时器配置

```php
<?php
// config/timer.php
return [
    'levels' => [
        ['tick_ms' => 1, 'slots' => 64],      // 毫秒轮
        ['tick_ms' => 64, 'slots' => 60],     // 秒轮
        ['tick_ms' => 3840, 'slots' => 60],   // 分钟轮
        ['tick_ms' => 230400, 'slots' => 24], // 小时轮
    ],
];
```

## 技术实现

- 4级层级设计
- O(1) 添加/删除任务复杂度
- 支持一次性任务和周期性任务
- 任务状态追踪和统计
