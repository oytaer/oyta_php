# 缓存模块 (Cache)

## 模块概述

缓存模块提供 OYTAPHP 的缓存管理功能，对应 ThinkPHP 8.0 的 Cache 门面。支持多种缓存驱动，提供统一的缓存操作接口。

## 模块结构

```
cache/
├── mod.rs          # 模块入口
├── facade.rs       # Cache 门面
├── driver.rs       # 缓存驱动 trait
├── manager.rs      # 缓存管理器
├── file.rs         # 文件缓存驱动
├── redis.rs        # Redis 缓存驱动
└── memory.rs       # 内存缓存驱动
```

## 支持的缓存驱动

| 驱动 | 说明 | 适用场景 |
|------|------|----------|
| file | 文件缓存 | 单机部署、简单场景 |
| redis | Redis 缓存 | 分布式部署、高性能场景 |
| memory | 内存缓存 | 临时缓存、高速访问 |

## Cache 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `get` | key: &str | Option<String> | 获取缓存值 |
| `get_or` | key: &str, default: &str | String | 获取缓存值，不存在则返回默认值 |
| `set` | key: &str, value: &str, ttl: u64 | Result<()> | 设置缓存值 |
| `forever` | key: &str, value: &str | Result<()> | 永久设置缓存 |
| `delete` | key: &str | Result<()> | 删除缓存 |
| `has` | key: &str | bool | 检查缓存是否存在 |
| `increment` | key: &str, step: i64 | Result<i64> | 自增 |
| `decrement` | key: &str, step: i64 | Result<i64> | 自减 |
| `clear` | - | Result<()> | 清空所有缓存 |
| `remember` | key: &str, ttl: u64, value: &str | Result<String> | 记住缓存（不存在则设置） |

## 使用示例

### PHP 风格调用

```php
<?php
use oyta\facade\Cache;

// 设置缓存
Cache::set('user_1', json_encode(['name' => '张三']), 3600);

// 获取缓存
$user = Cache::get('user_1');

// 获取缓存，不存在则返回默认值
$name = Cache::get_or('name', '默认名称');

// 检查缓存是否存在
if (Cache::has('user_1')) {
    echo "缓存存在";
}

// 自增
Cache::increment('counter', 1);

// 自减
Cache::decrement('counter', 1);

// 删除缓存
Cache::delete('user_1');

// 清空所有缓存
Cache::clear();

// 记住缓存
$value = Cache::remember('config', 3600, json_encode(['debug' => true]));
```

### 配置示例

```php
<?php
// config/cache.php
return [
    'default' => 'file',
    'stores' => [
        'file' => [
            'driver' => 'file',
            'path' => runtime_path('cache'),
            'expire' => 3600,
        ],
        'redis' => [
            'driver' => 'redis',
            'host' => '127.0.0.1',
            'port' => 6379,
            'password' => '',
            'select' => 0,
            'timeout' => 0,
            'persistent' => false,
        ],
        'memory' => [
            'driver' => 'memory',
            'max_items' => 10000,
        ],
    ],
];
```

## 缓存驱动实现

### 文件缓存驱动

- 使用 tokio::fs 进行异步文件操作
- 支持过期时间检查
- 自动清理过期缓存

### Redis 缓存驱动

- 使用 deadpool-redis 连接池
- 支持分布式部署
- 高性能读写

### 内存缓存驱动

- 使用 moka 高性能缓存库
- 支持 LRU 淘汰策略
- 适合临时缓存场景
