# 微服务模块

## 模块概述

微服务模块实现了内置微服务框架，提供服务注册与发现、负载均衡、熔断器、API 网关和配置中心等核心功能。

## 文件结构

```
microservice/
├── mod.rs              # 模块入口
├── registry.rs         # 服务注册与发现
├── loadbalancer.rs     # 负载均衡
├── circuit_breaker.rs  # 熔断器
├── gateway.rs          # API 网关
└── config_center.rs    # 配置中心
```

## 核心设计

### 微服务架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        微服务框架                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                      API Gateway                          │   │
│   │   ┌─────────────────────────────────────────────────┐   │   │
│   │   │  路由 │ 限流 │ 认证 │ 日志 │ 监控 │ 熔断 │       │   │   │
│   │   └─────────────────────────────────────────────────┘   │   │
│   └─────────────────────────────────────────────────────────┘   │
│                              │                                    │
│              ┌───────────────┼───────────────┐                   │
│              │               │               │                   │
│              ▼               ▼               ▼                   │
│   ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │
│   │ Load Balancer│ │Load Balancer │ │Load Balancer │            │
│   └──────────────┘ └──────────────┘ └──────────────┘            │
│         │                │                │                      │
│         ▼                ▼                ▼                      │
│   ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │
│   │  Service A   │ │  Service B   │ │  Service C   │            │
│   │  Instance 1  │ │  Instance 1  │ │  Instance 1  │            │
│   │  Instance 2  │ │  Instance 2  │ │  Instance 2  │            │
│   └──────────────┘ └──────────────┘ └──────────────┘            │
│                                                                   │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │              Service Registry (注册中心)                  │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                   │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │              Config Center (配置中心)                     │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## 核心组件

### ServiceRegistry

服务注册与发现：

```rust
pub struct ServiceRegistry {
    /// 服务实例存储
    services: DashMap<String, Vec<ServiceInstance>>,
    /// 健康检查器
    health_checker: HealthChecker,
    /// 心跳间隔（秒）
    heartbeat_interval_secs: u64,
    /// 实例过期时间（秒）
    instance_expire_secs: u64,
}

pub struct ServiceInstance {
    /// 实例 ID
    pub id: String,
    /// 服务名称
    pub service_name: String,
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 权重
    pub weight: u32,
    /// 状态
    pub status: InstanceStatus,
    /// 注册时间
    pub registered_at: Instant,
    /// 最后心跳时间
    pub last_heartbeat: Instant,
}

pub enum InstanceStatus {
    Up,
    Down,
    Starting,
    OutOfService,
}
```

**API**：

```rust
impl ServiceRegistry {
    /// 注册服务实例
    pub async fn register(&self, instance: ServiceInstance) -> Result<()>;
    
    /// 注销服务实例
    pub async fn deregister(&self, instance_id: &str) -> Result<()>;
    
    /// 发现服务实例
    pub async fn discover(&self, service_name: &str) -> Vec<ServiceInstance>;
    
    /// 发送心跳
    pub async fn heartbeat(&self, instance_id: &str) -> Result<()>;
    
    /// 更新实例状态
    pub async fn update_status(&self, instance_id: &str, status: InstanceStatus) -> Result<()>;
}
```

### LoadBalancer

负载均衡器：

```rust
pub trait LoadBalancer: Send + Sync {
    /// 选择一个实例
    fn select(&self, instances: &[ServiceInstance]) -> Option<&ServiceInstance>;
    
    /// 负载均衡策略名称
    fn name(&self) -> &'static str;
}
```

**支持的负载均衡策略**：

| 策略 | 说明 |
|------|------|
| RoundRobin | 轮询，依次选择每个实例 |
| WeightedRoundRobin | 加权轮询，按权重分配 |
| LeastConnections | 最少连接，选择连接数最少的实例 |
| Random | 随机选择 |
| ConsistentHash | 一致性哈希，相同请求路由到相同实例 |

```rust
/// 轮询负载均衡
pub struct RoundRobinLoadBalancer {
    counter: AtomicUsize,
}

/// 加权轮询负载均衡
pub struct WeightedRoundRobinLoadBalancer {
    counters: DashMap<String, AtomicUsize>,
}

/// 最少连接负载均衡
pub struct LeastConnectionsLoadBalancer {
    connections: DashMap<String, AtomicUsize>,
}

/// 一致性哈希负载均衡
pub struct ConsistentHashLoadBalancer {
    ring: HashRing,
    virtual_nodes: usize,
}
```

### CircuitBreaker

熔断器：

```rust
pub struct CircuitBreaker {
    /// 熔断器名称
    name: String,
    /// 当前状态
    state: Atomic<CircuitState>,
    /// 失败计数
    failure_count: AtomicUsize,
    /// 成功计数
    success_count: AtomicUsize,
    /// 配置
    config: CircuitBreakerConfig,
    /// 最后一次状态变更时间
    last_state_change: Atomic<Instant>,
}

pub enum CircuitState {
    /// 关闭状态（正常）
    Closed,
    /// 打开状态（熔断）
    Open,
    /// 半开状态（探测）
    HalfOpen,
}

pub struct CircuitBreakerConfig {
    /// 触发熔断的失败阈值
    pub failure_threshold: usize,
    /// 熔断持续时间（毫秒）
    pub open_duration_ms: u64,
    /// 半开状态下允许的请求数
    pub half_open_requests: usize,
    /// 成功阈值（半开状态恢复）
    pub success_threshold: usize,
    /// 滑动窗口大小
    pub sliding_window_size: usize,
}
```

**状态转换**：

```
         失败数 >= 阈值
    ┌─────────────────────┐
    │                     │
    │                     ▼
┌───────┐           ┌─────────┐
│ Closed│           │  Open   │
└───────┘           └─────────┘
    ▲                     │
    │                     │ 超时后
    │ 成功数 >= 阈值       ▼
    │              ┌──────────┐
    └──────────────│ HalfOpen │
                   └──────────┘
                        │
                        │ 失败数 >= 阈值
                        └───────────────┐
                                        │
                                        ▼
                                   ┌─────────┐
                                   │  Open   │
                                   └─────────┘
```

### ApiGateway

API 网关：

```rust
pub struct ApiGateway {
    /// 路由配置
    routes: Vec<RouteConfig>,
    /// 服务注册表
    registry: Arc<ServiceRegistry>,
    /// 负载均衡器
    load_balancer: Arc<dyn LoadBalancer>,
    /// 熔断器
    circuit_breakers: DashMap<String, Arc<CircuitBreaker>>,
    /// 限流器
    rate_limiters: DashMap<String, RateLimiter>,
}

pub struct RouteConfig {
    /// 路由 ID
    pub id: String,
    /// 路径模式
    pub path: String,
    /// 目标服务
    pub service_name: String,
    /// 路径重写
    pub rewrite: Option<String>,
    /// 超时时间（毫秒）
    pub timeout_ms: u64,
    /// 重试次数
    pub retry_count: u32,
    /// 是否启用熔断
    pub enable_circuit_breaker: bool,
    /// 限流配置
    pub rate_limit: Option<RateLimitConfig>,
}

pub struct RateLimitConfig {
    /// 请求数限制
    pub requests: u64,
    /// 时间窗口（秒）
    pub window_secs: u64,
}
```

### ConfigCenter

配置中心：

```rust
pub struct ConfigCenter {
    /// 配置存储
    configs: DashMap<String, ConfigEntry>,
    /// 配置版本
    versions: DashMap<String, u64>,
    /// 配置监听器
    watchers: DashMap<String, Vec<Box<dyn ConfigWatcher>>>,
}

pub struct ConfigEntry {
    /// 配置键
    pub key: String,
    /// 配置值
    pub value: String,
    /// 配置版本
    pub version: u64,
    /// 创建时间
    pub created_at: Instant,
    /// 更新时间
    pub updated_at: Instant,
}

pub trait ConfigWatcher: Send + Sync {
    /// 配置变更回调
    fn on_change(&self, key: &str, old_value: &str, new_value: &str);
}
```

## PHP API

### 服务注册

```php
// 注册服务
\oyta\Microservice::register([
    'service_name' => 'user-service',
    'host' => '192.168.1.100',
    'port' => 8080,
    'metadata' => [
        'version' => '1.0.0',
        'region' => 'us-west',
    ],
    'weight' => 100,
]);

// 发送心跳
\oyta\Microservice::heartbeat('instance-id');

// 注销服务
\oyta\Microservice::deregister('instance-id');
```

### 服务发现

```php
// 发现服务实例
$instances = \oyta\Microservice::discover('user-service');

foreach ($instances as $instance) {
    echo $instance['host'] . ':' . $instance['port'] . "\n";
}
```

### 负载均衡调用

```php
// 使用负载均衡调用服务
$response = \oyta\Microservice::call('user-service', '/api/users/1', [
    'method' => 'GET',
    'load_balancer' => 'round_robin', // round_robin, weighted, least_connections, random, consistent_hash
]);

// 使用熔断器保护
$response = \oyta\Microservice::call('user-service', '/api/users/1', [
    'circuit_breaker' => [
        'enabled' => true,
        'failure_threshold' => 5,
        'open_duration_ms' => 30000,
    ],
]);
```

### API 网关配置

```php
// 配置路由
\oyta\Gateway::route([
    'id' => 'user-service-route',
    'path' => '/api/users/*',
    'service_name' => 'user-service',
    'rewrite' => '/users/$1',
    'timeout_ms' => 5000,
    'retry_count' => 3,
    'rate_limit' => [
        'requests' => 100,
        'window_secs' => 60,
    ],
]);
```

### 配置中心

```php
// 获取配置
$value = \oyta\ConfigCenter::get('database.host');

// 设置配置
\oyta\ConfigCenter::set('database.host', 'localhost');

// 监听配置变更
\oyta\ConfigCenter::watch('database.host', function ($key, $oldValue, $newValue) {
    echo "Config changed: $key = $newValue\n";
});
```

## 配置说明

```php
// config/microservice.php
return [
    // 服务注册配置
    'registry' => [
        'heartbeat_interval_secs' => 10,
        'instance_expire_secs' => 30,
    ],
    
    // 负载均衡配置
    'load_balancer' => [
        'default' => 'round_robin',
    ],
    
    // 熔断器配置
    'circuit_breaker' => [
        'default' => [
            'failure_threshold' => 5,
            'open_duration_ms' => 30000,
            'half_open_requests' => 3,
            'success_threshold' => 2,
        ],
    ],
    
    // 网关配置
    'gateway' => [
        'default_timeout_ms' => 5000,
        'default_retry_count' => 3,
    ],
];
```

## 使用场景

### 微服务架构

```php
// 用户服务
class UserService {
    public function getUser($id) {
        return \oyta\Microservice::call('user-service', "/api/users/{$id}");
    }
}

// 订单服务
class OrderService {
    public function createOrder($data) {
        return \oyta\Microservice::call('order-service', '/api/orders', [
            'method' => 'POST',
            'body' => $data,
        ]);
    }
}
```

### 服务降级

```php
// 使用熔断器实现服务降级
$response = \oyta\Microservice::call('recommendation-service', '/api/recommend', [
    'fallback' => function () {
        return ['items' => []]; // 降级返回空推荐
    },
]);
```
