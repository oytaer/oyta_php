# HTTP/3 模块 (HTTP3)

## 模块概述

HTTP/3 模块提供基于 QUIC 协议的 HTTP/3 实现，包括 QUIC 连接管理、HTTP/3 请求/响应处理、流管理、0-RTT 支持等功能。

## 模块结构

```
http3/
├── mod.rs          # 模块入口
├── quic.rs         # QUIC 协议
├── h3.rs           # HTTP/3 实现
├── stream.rs       # 流管理
└── connection.rs   # 连接管理
```

## 主要类型

| 类型 | 说明 |
|------|------|
| QuicConfig | QUIC 配置 |
| QuicConnection | QUIC 连接 |
| H3Server | HTTP/3 服务器 |
| H3Request | HTTP/3 请求 |
| H3Response | HTTP/3 响应 |

## 功能特性

| 特性 | 说明 |
|------|------|
| 0-RTT | 零往返时间连接 |
| 连接迁移 | 网络切换不断连 |
| 多路复用 | 无队头阻塞 |
| 前向纠错 | 丢包恢复 |

## 使用示例

### 启动 HTTP/3 服务器

```php
<?php
// 启用 HTTP/3
// config/app.php
return [
    'http3' => [
        'enabled' => true,
        'port' => 443,
        'cert' => '/path/to/cert.pem',
        'key' => '/path/to/key.pem',
    ],
];
```

### 配置示例

```php
<?php
// config/http3.php
return [
    'enabled' => true,
    'port' => 443,
    'cert' => env('HTTP3_CERT'),
    'key' => env('HTTP3_KEY'),
    'max_concurrent_streams' => 100,
    'max_idle_timeout' => 30,
];
```

## 技术实现

- 基于 QUIC 协议
- 0-RTT 连接
- 连接迁移支持
- 多路复用无队头阻塞
