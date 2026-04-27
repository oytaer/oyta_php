# 缓存模块

## 模块概述

缓存模块提供多级缓存系统，支持 Redis、内存缓存等多种缓存驱动，并提供缓存预热、缓存标签等高级功能。

## 文件结构

```
cache/
├── mod.rs              # 模块入口
├── manager.rs          # 缓存管理器
├── driver.rs           # 缓存驱动接口
├── redis_driver.rs     # Redis 缓存驱动
├── multi_level.rs      # 多级缓存
├── tag.rs              # 缓存标签
├── facade.rs           # Facade 门面
└── warmup/             # 缓存预热
    ├── mod.rs
    ├── analyzer.rs     # 访问分析器
    ├── warmer.rs       # 预热执行器
    └── invalidator.rs  # 缓存失效器
```

## 核心组件

### CacheManager

缓存管理器，统一管理所有缓存驱动：

```rust
pub struct CacheManager {
    /// 默认驱动名称
    default_driver: String,
    /// 已注册的驱动
    drivers: DashMap<String, Box<dyn CacheDriver>>,
    /// 缓存配置
    config: CacheConfig,
}
```

### CacheDriver Trait

缓存驱动接口：

```rust
#[async_trait]
pub trait CacheDriver: Send + Sync {
    /// 获取缓存值
    async fn get(&self, key: &str) -> Option<Vec<u8>>;
    
    /// 设置缓存值
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()>;
    
    /// 删除缓存
    async fn delete(&self, key: &str) -> Result<()>;
    
    /// 检查缓存是否存在
    async fn has(&self, key: &str) -> bool;
    
    /// 清空缓存
    async fn clear(&self) -> Result<()>;
    
    /// 批量获取
    async fn many(&self, keys: &[&str]) -> Vec<Option<Vec<u8>>>;
    
    /// 批量设置
    async fn put_many(&self, items: &[(&str, &[u8])], ttl: Option<Duration>) -> Result<()>;
    
    /// 自增
    async fn increment(&self, key: &str, value: i64) -> Result<i64>;
    
    /// 自减
    async fn decrement(&self, key: &str, value: i64) -> Result<i64>;
}
```

## 缓存驱动

### Redis 驱动

使用 `deadpool-redis` 实现连接池管理：

```rust
pub struct RedisDriver {
    /// Redis 连接池
    pool: Pool<Connection>,
    /// 键前缀
    prefix: String,
}
```

**配置示例**：

```php
// config/cache.php
return [
    'default' => 'redis',
    'stores' => [
        'redis' => [
            'driver' => 'redis',
            'connection' => 'cache',
            'prefix' => 'oyta:',
        ],
    ],
];
```

### 内存驱动

使用 `moka` 高性能缓存库：

```rust
pub struct MemoryDriver {
    /// Moka 缓存实例
    cache: Cache<String, Vec<u8>>,
    /// 最大容量
    max_capacity: u64,
    /// TTL
    ttl: Option<Duration>,
}
```

### 多级缓存

L1（内存）+ L2（Redis）多级缓存：

```rust
pub struct MultiLevelCache {
    /// L1 内存缓存
    l1: MemoryDriver,
    /// L2 Redis 缓存
    l2: RedisDriver,
    /// L1 命中统计
    l1_hits: AtomicU64,
    /// L2 命中统计
    l2_hits: AtomicU64,
    /// 未命中统计
    misses: AtomicU64,
}
```

**读取流程**：
1. 先查 L1 内存缓存
2. L1 未命中则查 L2 Redis
3. L2 命中则回填 L1
4. 都未命中则返回 None

**写入流程**：
1. 同时写入 L1 和 L2
2. 保证两级缓存一致性

## 缓存标签

支持按标签批量管理缓存：

```php
// 设置带标签的缓存
\oyta\facade\Cache::tags(['user', 'profile'])->put('user:1:profile', $profile, 3600);

// 清除指定标签的所有缓存
\oyta\facade\Cache::tags(['user'])->flush();
```

**实现原理**：
- 每个标签维护一个键集合
- 设置缓存时，将键添加到所有相关标签
- 清除标签时，删除标签下所有键

## 缓存预热

### 访问分析器

分析热点数据访问模式：

```rust
pub struct AccessAnalyzer {
    /// 访问计数器
    access_counts: DashMap<String, AtomicU64>,
    /// 访问时间记录
    access_times: DashMap<String, Vec<u64>>,
    /// 时间衰减因子
    decay_factor: f64,
}
```

**时间衰减算法**：
```
score = access_count * e^(-decay_factor * time_elapsed)
```

### 预热执行器

执行缓存预热任务：

```rust
pub struct CacheWarmer {
    /// 分析器引用
    analyzer: Arc<AccessAnalyzer>,
    /// 缓存驱动
    driver: Arc<dyn CacheDriver>,
    /// 预热任务队列
    warmup_queue: Vec<WarmupTask>,
    /// 并发数
    concurrency: usize,
}
```

### 缓存失效器

管理缓存失效策略：

```rust
pub struct CacheInvalidator {
    /// 失效规则
    rules: Vec<InvalidationRule>,
    /// 依赖关系
    dependencies: DashMap<String, Vec<String>>,
}

pub struct InvalidationRule {
    /// 规则名称
    pub name: String,
    /// 触发条件
    pub trigger: Trigger,
    /// 失效模式
    pub mode: InvalidationMode,
}

pub enum Trigger {
    /// 数据更新
    OnUpdate(String),
    /// 数据删除
    OnDelete(String),
    /// 定时失效
    OnSchedule(String),
    /// 手动触发
    Manual,
}

pub enum InvalidationMode {
    /// 立即失效
    Immediate,
    /// 延迟失效
    Delayed(Duration),
    /// 懒失效
    Lazy,
}
```

## Facade 使用

### 基本操作

```php
// 获取缓存
$value = \oyta\facade\Cache::get('key');

// 设置缓存（永久）
\oyta\facade\Cache::put('key', 'value');

// 设置缓存（带过期时间，秒）
\oyta\facade\Cache::put('key', 'value', 3600);

// 设置缓存（带过期时间，DateTime）
\oyta\facade\Cache::put('key', 'value', \Carbon\Carbon::now()->addHour());

// 检查缓存是否存在
if (\oyta\facade\Cache::has('key')) {
    // ...
}

// 删除缓存
\oyta\facade\Cache::forget('key');

// 清空所有缓存
\oyta\facade\Cache::flush();
```

### 原子操作

```php
// 自增
$newValue = \oyta\facade\Cache::increment('counter', 1);

// 自减
$newValue = \oyta\facade\Cache::decrement('counter', 1);
```

### 记住模式

```php
// 获取或设置（闭包）
$value = \oyta\facade\Cache::remember('key', 3600, function () {
    return expensiveOperation();
});

// 永久记住
$value = \oyta\facade\Cache::rememberForever('key', function () {
    return expensiveOperation();
});
```

### 批量操作

```php
// 批量获取
$values = \oyta\facade\Cache::many(['key1', 'key2', 'key3']);

// 批量设置
\oyta\facade\Cache::putMany([
    'key1' => 'value1',
    'key2' => 'value2',
], 3600);
```

## 配置说明

```php
// config/cache.php
return [
    // 默认缓存驱动
    'default' => env('CACHE_DRIVER', 'redis'),
    
    // 缓存存储配置
    'stores' => [
        // Redis 缓存
        'redis' => [
            'driver' => 'redis',
            'connection' => 'cache',
            'lock_connection' => 'cache',
            'prefix' => env('CACHE_PREFIX', 'oyta:'),
        ],
        
        // 内存缓存
        'memory' => [
            'driver' => 'memory',
            'max_capacity' => 10000,
            'ttl' => 3600,
        ],
        
        // 多级缓存
        'multi' => [
            'driver' => 'multi',
            'l1' => 'memory',
            'l2' => 'redis',
        ],
    ],
    
    // 缓存预热配置
    'warmup' => [
        'enabled' => true,
        'concurrency' => 10,
        'schedule' => '0 3 * * *', // 每天凌晨 3 点
    ],
];
```

## 性能特点

| 操作 | Redis 驱动 | 内存驱动 | 多级缓存 |
|------|-----------|---------|---------|
| 读取 | ~1ms | ~1μs | L1: ~1μs, L2: ~1ms |
| 写入 | ~1ms | ~1μs | ~1ms |
| 批量读取 | ~10ms | ~10μs | 取决于命中率 |
| 命中率 | N/A | N/A | 通常 > 90% |
