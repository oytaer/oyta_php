# 内嵌服务模块 (Embedded)

## 模块概述

内嵌服务模块提供 WebSocket、定时器、队列、文件上传等内嵌服务，在 public/index.php 中声明，随应用一起启动。

## 模块结构

```
embedded/
├── mod.rs          # 模块入口
├── websocket.rs    # WebSocket 服务
├── timer.rs        # 定时器服务
├── queue.rs        # 队列服务
├── queue_redis.rs  # Redis 队列
├── upload.rs       # 文件上传
├── cron.rs         # 定时任务
└── queue_facade.rs # Queue 门面
```

## Queue 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `push` | job: &str, data: HashMap | - | 推送任务 |
| `later` | delay: u64, job: &str, data: HashMap | - | 延迟推送 |
| `pop` | queue: &str | Option<Job> | 弹出任务 |
| `len` | queue: &str | usize | 队列长度 |
| `size` | queue: &str | usize | 队列长度（别名） |
| `clear` | queue: &str | - | 清空队列 |
| `failedJobs` | queue: &str | Vec<Job> | 获取失败任务 |
| `retry` | id: &str, queue: &str | bool | 重试任务 |

## WebSocket 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `join` | room: &str, fd: u32 | - | 加入房间 |
| `leave` | room: &str, fd: u32 | - | 离开房间 |
| `broadcast` | message: &str | - | 广播消息 |
| `broadcastToRoom` | room: &str, message: &str | - | 房间广播 |
| `send` | fd: u32, message: &str | - | 发送消息 |
| `emit` | event: &str, data: HashMap | - | 发送事件 |
| `onlineCount` | - | usize | 在线人数 |
| `getRooms` | - | Vec<String> | 获取房间列表 |
| `roomCount` | room: &str | usize | 房间人数 |

## 使用示例

### 队列任务

```php
<?php
use oyta\facade\Queue;

// 推送任务
Queue::push('SendEmailJob', [
    'email' => 'user@example.com',
    'content' => '邮件内容',
]);

// 延迟任务
Queue::later(60, 'SendEmailJob', $data);
```

### WebSocket

```php
<?php
use oyta\facade\WebSocket;

// 加入房间
WebSocket::join('chat', $fd);

// 广播消息
WebSocket::broadcastToRoom('chat', 'Hello everyone');

// 发送消息
WebSocket::send($fd, 'Hello');
```

## 技术实现

- WebSocket 服务
- 定时器服务
- 队列服务
- 文件上传服务
