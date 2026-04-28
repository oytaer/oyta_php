# 微服务模块 (Microservice)

## 模块概述

微服务模块提供完整的微服务架构支持，包括服务注册与发现、负载均衡、熔断器、服务网关、配置中心等功能。

## 模块结构

```
microservice/
├── mod.rs          # 模块入口
├── registry.rs     # 服务注册与发现
├── loadbalancer.rs # 负载均衡
├── circuit_breaker.rs # 熔断器
├── gateway.rs      # 服务网关
└── config_center.rs # 配置中心
```

## 主要类型

| 类型 | 说明 |
|------|------|
| ServiceRegistry | 服务注册中心 |
| ServiceInstance | 服务实例 |
| LoadBalancer | 负载均衡器 |
| CircuitBreaker | 熔断器 |
| ApiGateway | API 网关 |
| ConfigCenter | 配置中心 |

## 负载均衡策略

| 策略 | 说明 |
|------|------|
| RoundRobin | 轮询 |
| WeightedRoundRobin | 加权轮询 |
| Random | 随机 |
| LeastConnections | 最少连接 |

## 使用示例

### 服务注册

```php
<?php
use oyta\microservice\ServiceRegistry;

// 注册服务
ServiceRegistry::register([
    'name' => 'user-service',
    'host' => '192.168.1.100',
    'port' => 8080,
    'weight' => 1,
    'metadata' => [
        'version' => '1.0.0',
    ],
]);

// 发现服务
$instances = ServiceRegistry::discover('user-service');
```

### 负载均衡

```php
<?php
use oyta\microservice\LoadBalancer;

// 创建负载均衡器
$lb = new LoadBalancer('round_robin');

// 获取服务实例
$instance = $lb->select('user-service');

// 调用服务
$response = $lb->call('user-service', '/api/users/1');
```

### 熔断器

```php
<?php
use oyta\microservice\CircuitBreaker;

// 创建熔断器
$cb = new CircuitBreaker([
    'failure_threshold' => 5,
    'timeout' => 30,
    'half_open_requests' => 3,
]);

// 执行请求
$result = $cb->execute(function() use ($instance) {
    return $instance->call('/api/users/1');
});
```

### API 网关

```php
<?php
use oyta\microservice\ApiGateway;

// 配置路由
$gateway = new ApiGateway([
    'routes' => [
        '/api/users/*' => 'user-service',
        '/api/orders/*' => 'order-service',
    ],
    'middleware' => ['auth', 'rate_limit'],
]);
```

## 技术实现

- 服务注册与发现
- 多种负载均衡策略
- 熔断器模式
- API 网关
- 配置中心
