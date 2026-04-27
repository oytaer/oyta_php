# HTTP/3 模块

## 模块概述

HTTP/3 模块实现了原生 HTTP/3 和 QUIC 协议支持，提供更快的连接建立、更好的移动网络性能和内置的连接迁移能力。

## 文件结构

```
http3/
├── mod.rs          # 模块入口
├── quic.rs         # QUIC 协议实现
├── h3.rs           # HTTP/3 实现
├── stream.rs       # 流管理
└── connection.rs   # 连接管理
```

## 核心设计

### 协议栈

```
┌─────────────────────────────────────────────────────────────────┐
│                        HTTP/3 协议栈                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                    HTTP/3 (h3)                           │   │
│   │   ┌─────────────────────────────────────────────────┐   │   │
│   │   │  请求/响应 │ 头部压缩 (QPACK) │ 流管理           │   │   │
│   │   └─────────────────────────────────────────────────┘   │   │
│   └─────────────────────────────────────────────────────────┘   │
│                              │                                    │
│                              ▼                                    │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                    QUIC                                  │   │
│   │   ┌─────────────────────────────────────────────────┐   │   │
│   │   │  连接管理 │ 流复用 │ 拥塞控制 │ 加密 (TLS 1.3)   │   │   │
│   │   └─────────────────────────────────────────────────┘   │   │
│   └─────────────────────────────────────────────────────────┘   │
│                              │                                    │
│                              ▼                                    │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                    UDP                                   │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## 核心组件

### QuicConnection

QUIC 连接：

```rust
pub struct QuicConnection {
    /// 连接 ID
    pub connection_id: ConnectionId,
    /// 本地地址
    pub local_addr: SocketAddr,
    /// 远程地址
    pub remote_addr: SocketAddr,
    /// 连接状态
    pub state: ConnectionState,
    /// 配置
    pub config: QuicConfig,
    /// 流管理器
    stream_manager: StreamManager,
    /// 统计信息
    stats: QuicStats,
}

pub struct ConnectionId {
    /// 连接 ID 字节
    pub bytes: [u8; 20],
    /// 长度
    pub len: usize,
}

pub enum ConnectionState {
    /// 正在连接
    Connecting,
    /// 已连接
    Connected,
    /// 正在关闭
    Closing,
    /// 已关闭
    Closed,
    /// 空闲
    Idle,
}

pub struct QuicConfig {
    /// 最大并发流数
    pub max_concurrent_streams: u64,
    /// 初始流控窗口
    pub initial_stream_window: u64,
    /// 初始连接流控窗口
    pub initial_connection_window: u64,
    /// 空闲超时（毫秒）
    pub idle_timeout_ms: u64,
    /// 最大 UDP 负载大小
    pub max_udp_payload_size: u64,
    /// ACK 延迟指数
    pub ack_delay_exponent: u64,
}
```

### QuicStream

QUIC 流：

```rust
pub struct QuicStream {
    /// 流 ID
    pub id: StreamId,
    /// 流类型
    pub stream_type: StreamType,
    /// 流状态
    pub state: StreamState,
    /// 发送缓冲区
    send_buffer: VecDeque<Bytes>,
    /// 接收缓冲区
    recv_buffer: VecDeque<Bytes>,
    /// 流控窗口
    flow_control_window: u64,
}

pub struct StreamId {
    /// 流 ID 值
    pub value: u64,
}

impl StreamId {
    /// 是否为客户端发起
    pub fn is_client_initiated(&self) -> bool;
    
    /// 是否为服务端发起
    pub fn is_server_initiated(&self) -> bool;
    
    /// 是否为双向流
    pub fn is_bidirectional(&self) -> bool;
    
    /// 是否为单向流
    pub fn is_unidirectional(&self) -> bool;
}

pub enum StreamType {
    /// 客户端发起的双向流
    ClientBidi,
    /// 服务端发起的双向流
    ServerBidi,
    /// 客户端发起的单向流
    ClientUni,
    /// 服务端发起的单向流
    ServerUni,
}

pub enum StreamState {
    /// 空闲
    Idle,
    /// 打开（可读可写）
    Open,
    /// 半关闭（本地关闭）
    HalfClosedLocal,
    /// 半关闭（远程关闭）
    HalfClosedRemote,
    /// 已关闭
    Closed,
}
```

### H3Request / H3Response

HTTP/3 请求和响应：

```rust
pub struct H3Request {
    /// 请求方法
    pub method: Method,
    /// 请求路径
    pub path: String,
    /// 请求头
    pub headers: HeaderMap,
    /// 请求体
    pub body: Bytes,
    /// 查询参数
    pub query: HashMap<String, String>,
    /// 协议版本
    pub version: HttpVersion,
}

pub struct H3Response {
    /// 状态码
    pub status: StatusCode,
    /// 响应头
    pub headers: HeaderMap,
    /// 响应体
    pub body: Bytes,
    /// Trailer
    pub trailers: Option<HeaderMap>,
}

pub enum HttpVersion {
    Http3,
    Http2,
    Http1_1,
    Http1_0,
}
```

### H3Server

HTTP/3 服务器：

```rust
pub struct H3Server {
    /// QUIC 连接管理器
    connection_manager: ConnectionManager,
    /// 监听地址
    listen_addr: SocketAddr,
    /// TLS 配置
    tls_config: TlsConfig,
    /// 请求处理器
    handler: Box<dyn H3Handler>,
    /// 配置
    config: H3Config,
}

pub struct H3Config {
    /// 最大并发连接数
    pub max_connections: usize,
    /// 最大并发流数
    pub max_streams_per_connection: u64,
    /// 请求超时（毫秒）
    pub request_timeout_ms: u64,
    /// 头部最大大小
    pub max_header_size: usize,
    /// 请求体最大大小
    pub max_body_size: usize,
}

pub trait H3Handler: Send + Sync {
    /// 处理请求
    async fn handle(&self, request: H3Request) -> H3Response;
}
```

### StreamManager

流管理器：

```rust
pub struct StreamManager {
    /// 活跃流
    streams: DashMap<StreamId, QuicStream>,
    /// 下一个流 ID
    next_stream_id: AtomicU64,
    /// 最大并发流数
    max_concurrent_streams: u64,
    /// 当前并发流数
    current_streams: AtomicU64,
}

impl StreamManager {
    /// 创建新流
    pub fn create_stream(&self, stream_type: StreamType) -> Result<StreamId, QuicError>;
    
    /// 获取流
    pub fn get_stream(&self, id: &StreamId) -> Option<QuicStream>;
    
    /// 关闭流
    pub fn close_stream(&self, id: &StreamId) -> Result<(), QuicError>;
    
    /// 获取活跃流数量
    pub fn active_stream_count(&self) -> u64;
}
```

### ConnectionManager

连接管理器：

```rust
pub struct ConnectionManager {
    /// 活跃连接
    connections: DashMap<ConnectionId, QuicConnection>,
    /// 连接池配置
    config: ConnectionPoolConfig,
    /// 统计信息
    stats: ConnectionStats,
}

pub struct ConnectionPoolConfig {
    /// 最大连接数
    pub max_connections: usize,
    /// 连接空闲超时（毫秒）
    pub idle_timeout_ms: u64,
    /// 连接保活间隔（毫秒）
    pub keep_alive_interval_ms: u64,
    /// 最大连接重试次数
    pub max_retries: u32,
}

pub struct ConnectionStats {
    /// 总连接数
    pub total_connections: AtomicU64,
    /// 活跃连接数
    pub active_connections: AtomicU64,
    /// 总请求数
    pub total_requests: AtomicU64,
    /// 失败请求数
    pub failed_requests: AtomicU64,
}
```

## PHP API

### 创建 HTTP/3 服务器

```php
// 创建 HTTP/3 服务器
$server = new \oyta\Http3\Server([
    'listen' => '0.0.0.0:443',
    'cert' => '/path/to/cert.pem',
    'key' => '/path/to/key.pem',
    'max_connections' => 10000,
]);

// 设置请求处理器
$server->onRequest(function ($request) {
    return new \oyta\Http3\Response(200, [
        'Content-Type' => 'application/json',
    ], json_encode(['message' => 'Hello HTTP/3']));
});

// 启动服务器
$server->start();
```

### 发送 HTTP/3 请求

```php
// 创建 HTTP/3 客户端
$client = new \oyta\Http3\Client([
    'timeout' => 5000,
    'max_streams' => 100,
]);

// 发送 GET 请求
$response = $client->get('https://example.com/api/users');

// 发送 POST 请求
$response = $client->post('https://example.com/api/users', [
    'headers' => [
        'Content-Type' => 'application/json',
    ],
    'body' => json_encode(['name' => 'John']),
]);

// 获取响应
echo $response->getStatusCode();
echo $response->getBody();
```

### 连接管理

```php
// 获取连接统计
$stats = $server->getConnectionStats();
echo "Active connections: " . $stats['active_connections'] . "\n";
echo "Total requests: " . $stats['total_requests'] . "\n";

// 关闭连接
$server->closeConnection($connectionId);
```

## HTTP/3 优势

### 与 HTTP/1.1 和 HTTP/2 对比

| 特性 | HTTP/1.1 | HTTP/2 | HTTP/3 |
|------|----------|--------|--------|
| 传输层 | TCP | TCP | UDP (QUIC) |
| 多路复用 | ❌ | ✅ | ✅ |
| 队头阻塞 | 有 | TCP 层有 | 无 |
| 连接建立 | 1-3 RTT | 1-3 RTT | 0-1 RTT |
| 连接迁移 | ❌ | ❌ | ✅ |
| 前向纠错 | ❌ | ❌ | ✅ |

### 性能优势

1. **更快的连接建立**：0-RTT 连接恢复，复用之前的连接状态
2. **无队头阻塞**：独立的流，一个丢包不影响其他流
3. **连接迁移**：网络切换时保持连接（如 WiFi 切 4G）
4. **更好的移动网络性能**：适应网络变化

## 配置说明

```php
// config/http3.php
return [
    // 服务器配置
    'server' => [
        'listen' => '0.0.0.0:443',
        'cert' => env('HTTP3_CERT'),
        'key' => env('HTTP3_KEY'),
        'max_connections' => 10000,
        'max_streams_per_connection' => 100,
        'request_timeout_ms' => 30000,
    ],
    
    // 客户端配置
    'client' => [
        'timeout_ms' => 5000,
        'max_streams' => 100,
        'idle_timeout_ms' => 60000,
        'keep_alive_interval_ms' => 30000,
    ],
    
    // QUIC 配置
    'quic' => [
        'initial_stream_window' => 65536,
        'initial_connection_window' => 1048576,
        'idle_timeout_ms' => 30000,
        'max_udp_payload_size' => 1350,
    ],
];
```

## 使用场景

### API 服务

```php
// 高性能 API 服务
$server = new \oyta\Http3\Server($config);

$server->onRequest(function ($request) {
    $path = $request->getPath();
    $method = $request->getMethod();
    
    // 路由处理
    $response = match ($path) {
        '/api/users' => handleUsers($method, $request),
        '/api/orders' => handleOrders($method, $request),
        default => new \oyta\Http3\Response(404),
    };
    
    return $response;
});
```

### 微服务通信

```php
// 服务间 HTTP/3 通信
$client = new \oyta\Http3\Client();

// 并发请求
$responses = $client->parallel([
    'users' => 'https://user-service/api/users',
    'orders' => 'https://order-service/api/orders',
    'products' => 'https://product-service/api/products',
]);
```

### 实时通信

```php
// 使用 HTTP/3 流实现实时通信
$stream = $client->createStream('https://example.com/realtime');

// 发送数据
$stream->write(['type' => 'ping']);

// 接收数据
while ($data = $stream->read()) {
    handleRealtimeData($data);
}
```
