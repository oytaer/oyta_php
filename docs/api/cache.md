# Cache 缓存模块

## 模块概述

Cache 模块提供高性能缓存系统，支持多种缓存驱动和高级缓存策略。对应 ThinkPHP 8.0 的 Cache 门面。

## 模块结构

```
cache/
├── mod.rs           # 模块入口
├── driver.rs        # 缓存驱动接口和实现（内存、文件）
├── manager.rs       # 缓存管理器
├── redis_driver.rs  # Redis 缓存驱动
├── multi_level.rs   # 多级缓存驱动
├── tag.rs           # 缓存标签管理
├── facade.rs        # 门面类
└── warmup/          # 缓存预热子模块
    ├── mod.rs
    ├── analyzer.rs  # 访问分析器
    ├── warmer.rs    # 预热执行器
    └── invalidator.rs # 失效管理器
```

## 缓存驱动接口 (CacheDriver Trait)

所有缓存驱动必须实现以下方法：

```rust
pub trait CacheDriver: Send + Sync {
    fn get(&self, key: &str) -> Option<Value>;
    fn set(&self, key: &str, value: Value, ttl: Option<u64>) -> bool;
    fn delete(&self, key: &str) -> bool;
    fn has(&self, key: &str) -> bool;
    fn clear(&self) -> bool;
    fn increment(&self, key: &str, step: i64) -> Option<i64>;
    fn decrement(&self, key: &str, step: i64) -> Option<i64>;
    fn get_multiple(&self, keys: &[&str]) -> HashMap<String, Value>;
    fn set_multiple(&self, items: &[(String, Value)], ttl: Option<u64>) -> bool;
    fn delete_multiple(&self, keys: &[&str]) -> bool;
    fn ttl(&self, key: &str) -> Option<i64>;
    fn set_ttl(&self, key: &str, ttl: u64) -> bool;
}
```

## 内存缓存驱动 (MemoryCacheDriver)

使用 moka 库实现高性能内存缓存：

```rust
pub struct MemoryCacheDriver {
    cache: Cache<String, CacheEntry>,
    max_capacity: u64,
}

struct CacheEntry {
    value: Value,
    expires_at: Option<Instant>,
}
```

### 特性

- 可配置的最大容量（默认 10000）
- TTL 过期时间支持
- 自动过期清理
- 高性能并发访问

### PHP 使用

```php
<?php
// 使用内存缓存
Cache::store('memory')->set('key', 'value', 3600);
$value = Cache::store('memory')->get('key');
```

## 文件缓存驱动 (FileCacheDriver)

将缓存数据持久化到文件系统：

```rust
pub struct FileCacheDriver {
    cache_dir: PathBuf,
    prefix: String,
}
```

### 特性

- 使用 MD5 哈希作为文件名
- 支持过期时间
- 自动清理过期文件
- 支持序列化复杂类型

### PHP 使用

```php
<?php
// 使用文件缓存
Cache::store('file')->set('key', 'value', 3600);
$value = Cache::store('file')->get('key');
```

## Redis 缓存驱动 (RedisCacheDriver)

基于 Redis 的高性能分布式缓存：

```rust
pub struct RedisCacheDriver {
    pool: Pool<RedisConnectionManager>,
    prefix: String,
}
```

### 特性

- 支持连接池
- 支持键前缀
- 支持 TTL
- 支持批量操作
- 支持自增/自减

### PHP 使用

```php
<?php
// 使用 Redis 缓存
Cache::store('redis')->set('key', 'value', 3600);
$value = Cache::store('redis')->get('key');

// 批量操作
Cache::store('redis')->mset(['key1' => 'value1', 'key2' => 'value2']);
$values = Cache::store('redis')->mget(['key1', 'key2']);

// 自增
$newValue = Cache::store('redis')->increment('counter', 1);
```

## 多级缓存驱动 (MultiLevelCacheDriver)

组合多个缓存驱动形成多级缓存：

```rust
pub struct MultiLevelCacheDriver {
    drivers: Vec<Box<dyn CacheDriver>>,
}
```

### 特性

- L1 通常为内存缓存（快速）
- L2 通常为 Redis/文件缓存（持久化）
- 读取时自动回填
- 写入时同时写入所有层级
- 删除时从所有层级删除

### PHP 使用

```php
<?php
// 使用多级缓存
Cache::store('multi')->set('key', 'value', 3600);
$value = Cache::store('multi')->get('key');
```

## 缓存标签 (Tag)

支持按标签分组管理缓存：

```rust
pub struct TagManager {
    tags: HashMap<String, HashSet<String>>,
}

pub struct TaggedCacheBuilder<'a> {
    manager: &'a mut TagManager,
    tags: Vec<String>,
}
```

### PHP 使用

```php
<?php
// 设置带标签的缓存
Cache::tag('user')->set('user_1', $userData, 3600);
Cache::tag(['user', 'admin'])->set('admin_1', $adminData, 3600);

// 清除标签下的所有缓存
Cache::tag('user')->clear();

// 获取标签下的所有键
$keys = Cache::tag('user')->keys();
```

## 缓存管理器 (CacheManager)

```rust
pub struct CacheManager {
    drivers: HashMap<String, Box<dyn CacheDriver>>,
    default_driver: String,
}

impl CacheManager {
    pub fn register_driver(&mut self, name: &str, driver: Box<dyn CacheDriver>);
    pub fn driver(&self, name: Option<&str>) -> &dyn CacheDriver;
    pub fn set_default(&mut self, name: &str);
    pub fn init_global(&self);
}
```

## 缓存预热 (Warmup)

### 访问分析器 (AccessAnalyzer)

分析缓存访问模式，识别热点数据：

```rust
pub struct AccessAnalyzer {
    access_counts: HashMap<String, AccessInfo>,
    config: AnalyzerConfig,
}

struct AccessInfo {
    count: u64,
    last_access: Instant,
    access_pattern: AccessPattern,
}
```

### 预热执行器 (CacheWarmer)

执行缓存预热任务：

```rust
pub struct CacheWarmer {
    rules: Vec<WarmupRule>,
    history: Vec<WarmupRecord>,
}

struct WarmupRule {
    key_pattern: String,
    data_source: DataSource,
    refresh_interval: Duration,
}
```

### 失效管理器 (Invalidator)

管理事件驱动的缓存失效：

```rust
pub struct Invalidator {
    rules: Vec<InvalidationRule>,
    history: Vec<InvalidationRecord>,
}

struct InvalidationRule {
    event: String,
    key_patterns: Vec<String>,
    delay: Option<Duration>,
}
```

## Cache 门面 API

### 基本操作

```php
<?php
// 获取缓存
$value = Cache::get('key');
$value = Cache::get('key', 'default');

// 设置缓存（TTL 单位：秒）
Cache::set('key', 'value', 3600);

// 永久设置
Cache::forever('key', 'value');

// 删除缓存
Cache::delete('key');

// 检查是否存在
if (Cache::has('key')) {
    // ...
}

// 清空所有缓存
Cache::clear();
```

### 自增/自减

```php
<?php
// 自增
$newValue = Cache::increment('counter', 1);

// 自减
$newValue = Cache::decrement('counter', 1);
```

### 记住缓存

```php
<?php
// 如果缓存不存在则设置
$value = Cache::remember('key', 3600, 'default_value');
```

### 批量操作

```php
<?php
// 批量获取
$values = Cache::mget(['key1', 'key2', 'key3']);

// 批量设置
Cache::mset([
    'key1' => 'value1',
    'key2' => 'value2',
]);

// 批量删除
Cache::mdelete(['key1', 'key2']);
```

### TTL 操作

```php
<?php
// 获取剩余过期时间
$ttl = Cache::ttl('key');

// 设置过期时间
Cache::setTtl('key', 3600);
```

### 切换驱动

```php
<?php
// 使用指定驱动
Cache::store('redis')->set('key', 'value');
```

## 配置示例

```php
<?php
// config/cache.php
return [
    'default' => 'file',
    
    'stores' => [
        'file' => [
            'type' => 'file',
            'path' => runtime_path('cache'),
        ],
        
        'memory' => [
            'type' => 'memory',
            'max_capacity' => 10000,
        ],
        
        'redis' => [
            'type' => 'redis',
            'url' => env('REDIS_URL', 'redis://127.0.0.1:6379/0'),
            'prefix' => 'cache:',
        ],
        
        'multi' => [
            'type' => 'multi',
            'drivers' => ['memory', 'redis'],
        ],
    ],
];
```

## 性能特性

- **moka 内存缓存**：高性能并发缓存库
- **连接池**：Redis 连接池复用
- **多级缓存**：L1+L2 架构，兼顾速度和持久化
- **智能预热**：基于访问模式自动预热热点数据
- **事件驱动失效**：精准失效，减少缓存穿透
