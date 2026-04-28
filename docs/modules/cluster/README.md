# 集群模块 (Cluster)

## 模块概述

集群模块提供分布式部署所需的功能，包括分布式 Session、分布式缓存、分布式锁、服务发现与注册等。

## 模块结构

```
cluster/
├── mod.rs              # 模块入口
├── distributed_cache.rs # 分布式缓存
├── distributed_lock.rs # 分布式锁
├── distributed_session.rs # 分布式 Session
└── service_discovery.rs # 服务发现
```

## 功能特性

| 功能 | 说明 |
|------|------|
| 分布式 Session | Redis Session 共享 |
| 分布式缓存 | 缓存一致性 |
| 分布式锁 | Redlock 算法 |
| 服务发现 | 服务健康检查 |

## 使用示例

### 分布式锁

```php
<?php
use oyta\cluster\DistributedLock;

// 获取锁
$lock = new DistributedLock('resource_key', 30);

if ($lock->acquire()) {
    try {
        // 执行业务逻辑
    } finally {
        $lock->release();
    }
}
```

### 分布式 Session

```php
<?php
// config/session.php
return [
    'driver' => 'redis',
    'redis' => [
        'host' => '127.0.0.1',
        'port' => 6379,
    ],
];
```

## 技术实现

- Redis 分布式锁
- Session 共享
- 缓存一致性
- 服务健康检查
