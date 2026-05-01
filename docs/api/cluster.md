# Cluster 分布式集群模块

## 模块概述

Cluster 模块提供分布式系统支持，包括服务发现、分布式锁、分布式 Session 和分布式缓存。对应 ThinkPHP 8.0 的分布式部署能力。

## 模块结构

```
cluster/
├── mod.rs                  # 模块入口
├── service_discovery.rs    # 服务发现与注册
├── distributed_session.rs  # 分布式 Session
├── distributed_lock.rs     # 分布式锁
└── distributed_cache.rs    # 分布式缓存
```

## 服务发现 (ServiceDiscovery)

### 支持的注册中心

- **Memory**: 内存注册（单机开发）
- **Consul**: HashiCorp Consul
- **etcd**: CoreOS etcd
- **Nacos**: Alibaba Nacos

### 服务实例

```php
<?php
// 注册服务
$instance = ServiceDiscovery::register('user-service', '192.168.1.100', 8080);

// 发现服务
$instances = ServiceDiscovery::discover('user-service');

// 获取单个实例（负载均衡）
$instance = ServiceDiscovery::getOne('user-service');

// 发送心跳
ServiceDiscovery::heartbeat($instanceId);

// 注销服务
ServiceDiscovery::deregister($instanceId);
```

### 健康检查

```php
<?php
// 执行健康检查
$result = ServiceDiscovery::health();

// 返回结果
// [
//     'total' => 10,
//     'healthy' => 8,
//     'unhealthy' => 2
// ]
```

## 分布式锁 (Lock)

### 基本使用

```php
<?php
// 获取锁
$lock = Lock::acquire('resource_key', 30000); // 30秒超时

if ($lock) {
    try {
        // 执行临界区代码
        // ...
    } finally {
        // 释放锁
        Lock::release($lock);
    }
}

// 尝试获取锁（非阻塞）
$lock = Lock::tryAcquire('resource_key', 30000);
```

### 自动续期

```php
<?php
// 使用锁守卫，自动续期和释放
Lock::guard('resource_key', 30000, function() {
    // 执行临界区代码
    // 锁会自动续期
});
```

### 锁操作

```php
<?php
// 检查锁是否存在
if (Lock::isLocked('resource_key')) {
    // 资源被锁定
}

// 强制释放锁（管理操作）
Lock::forceRelease('resource_key');
```

## 分布式 Session

### 配置

```php
<?php
// config/session.php
return [
    'type' => 'redis',
    'redis' => [
        'url' => env('REDIS_URL', 'redis://127.0.0.1:6379/0'),
        'prefix' => 'session:',
        'ttl' => 7200,
    ],
];
```

### 使用

```php
<?php
// 获取 Session 数据
$data = Session::get('session_id');
$field = Session::getField('session_id', 'user_id');

// 设置 Session 数据
Session::set('session_id', ['user_id' => 1, 'name' => 'John']);
Session::setField('session_id', 'user_id', 1);

// 删除字段
Session::deleteField('session_id', 'temp_data');

// 销毁 Session
Session::destroy('session_id');

// 刷新过期时间
Session::refresh('session_id');

// 获取剩余过期时间
$ttl = Session::ttl('session_id');
```

### Session 分布式锁

```php
<?php
// 获取 Session 锁
if (Session::acquireLock('session_id', 5000)) {
    try {
        // 安全操作 Session
    } finally {
        Session::releaseLock('session_id');
    }
}
```

## 分布式缓存

### 基本操作

```php
<?php
// 获取缓存
$value = Cache::get('key');

// 设置缓存
Cache::set('key', 'value', 3600);

// 删除缓存
Cache::delete('key');

// 检查是否存在
if (Cache::exists('key')) {
    // ...
}
```

### 带标签的缓存

```php
<?php
// 设置带标签的缓存
Cache::setWithTags('user_1', $data, ['user', 'active'], 3600);

// 清除标签下的所有缓存
$count = Cache::clearTag('user');
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
```

### 自增/自减

```php
<?php
// 自增
$newValue = Cache::increment('counter', 1);

// 自减
$newValue = Cache::decrement('counter', 1);
```

## 配置示例

```php
<?php
// config/cluster.php
return [
    // 服务发现配置
    'service_discovery' => [
        'type' => env('SD_TYPE', 'memory'),
        'endpoint' => env('SD_ENDPOINT', '127.0.0.1:8500'),
        'namespace' => 'default',
        'heartbeat_interval' => 10,
        'health_check_interval' => 30,
        'instance_ttl' => 60,
    ],
    
    // 分布式锁配置
    'lock' => [
        'redis_url' => env('REDIS_URL', 'redis://127.0.0.1:6379/0'),
        'prefix' => 'lock:',
        'default_timeout' => 30000,
        'retry_interval' => 100,
        'retry_count' => 3,
    ],
    
    // 分布式 Session 配置
    'session' => [
        'redis_url' => env('REDIS_URL', 'redis://127.0.0.1:6379/0'),
        'prefix' => 'session:',
        'ttl' => 7200,
    ],
    
    // 分布式缓存配置
    'cache' => [
        'redis_url' => env('REDIS_URL', 'redis://127.0.0.1:6379/0'),
        'prefix' => 'cache:',
        'ttl' => 3600,
    ],
];
```

## 高可用特性

- **多注册中心支持**：Consul、etcd、Nacos
- **自动健康检查**：定时心跳检测
- **负载均衡**：轮询选择服务实例
- **锁自动续期**：防止业务执行期间锁过期
- **Session 复制**：支持多节点共享 Session
