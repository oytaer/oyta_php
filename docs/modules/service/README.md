# 服务管理模块 (Service)

## 模块概述

服务管理模块提供完整的服务生命周期管理，支持注解驱动服务声明、独立配置文件、按需启动（惰性初始化）、服务依赖管理。

## 模块结构

```
service/
├── mod.rs          # 模块入口
├── traits.rs       # 服务 trait
├── registry.rs     # 服务注册表
├── config.rs       # 服务配置
├── annotation.rs   # 注解解析
└── bootstrap.rs    # 服务启动
```

## 主要类型

| 类型 | 说明 |
|------|------|
| Service | 服务 trait |
| BackgroundService | 后台服务 trait |
| ServiceRegistry | 服务注册表 |
| ServiceBootstrap | 服务启动器 |
| ServiceConfig | 服务配置 |
| ServiceState | 服务状态 |

## 注解类型

| 注解 | 说明 |
|------|------|
| @Service | 服务声明 |
| @TimerTask | 定时任务 |
| @QueueJob | 队列任务 |
| @WebSocketHandler | WebSocket 处理器 |

## 使用示例

### 注解声明服务

```php
<?php
#[Service(name: 'websocket', enable: true, config: ['port' => 9501])]
class WebSocketService
{
    public function start()
    {
        // 启动 WebSocket 服务
    }
}
```

### 配置文件声明

```php
<?php
// config/websocket.php
return [
    'enable' => true,
    'port' => 9501,
];
```

### 惰性初始化

```php
<?php
// 门面调用时自动启动服务
Cache::set('key', 'value');  // 自动启动缓存服务
```

## 技术实现

- 注解驱动服务声明
- 独立配置文件
- 按需启动
- 服务依赖管理
