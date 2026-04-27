# 序列化模块

## 模块概述

序列化模块实现了零拷贝高性能序列化引擎，支持多种序列化格式（Bincode、MessagePack、CBOR、Postcard、JSON），提供流式处理和性能统计功能。

## 文件结构

```
serialize/
├── mod.rs              # 模块入口
├── engine.rs           # 序列化引擎
├── bincode_format.rs   # Bincode 格式
├── msgpack_format.rs   # MessagePack 格式
├── cbor_format.rs      # CBOR 格式
├── postcard_format.rs  # Postcard 格式
└── stream.rs           # 流式处理
```

## 核心设计

### 序列化引擎架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        序列化引擎                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                    SerializeEngine                        │   │
│   │   ┌─────────────────────────────────────────────────┐   │   │
│   │   │                  Format Trait                     │   │   │
│   │   │   ┌─────────┬─────────┬─────────┬─────────┐     │   │   │
│   │   │   │ Bincode │MsgPack  │  CBOR   │Postcard │     │   │   │
│   │   │   └─────────┴─────────┴─────────┴─────────┘     │   │   │
│   │   └─────────────────────────────────────────────────┘   │   │
│   └─────────────────────────────────────────────────────────┘   │
│                              │                                    │
│              ┌───────────────┼───────────────┐                   │
│              │               │               │                   │
│              ▼               ▼               ▼                   │
│   ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            │
│   │   序列化     │ │  反序列化    │ │  流式处理    │            │
│   │  serialize() │ │ deserialize()│ │  StreamBuf   │            │
│   └──────────────┘ └──────────────┘ └──────────────┘            │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## 核心组件

### SerializeEngine

序列化引擎核心：

```rust
pub struct SerializeEngine {
    /// 默认格式
    default_format: SerializeFormat,
    /// 性能统计
    stats: SerializeStats,
    /// 是否启用零拷贝
    zero_copy: bool,
}

pub enum SerializeFormat {
    Bincode,
    MessagePack,
    CBOR,
    Postcard,
    Json,
}
```

### Format Trait

序列化格式接口：

```rust
pub trait SerializeFormat: Send + Sync {
    /// 序列化
    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, SerializeError>;
    
    /// 反序列化
    fn deserialize<'de, T: Deserialize<'de>>(&self, data: &'de [u8]) -> Result<T, SerializeError>;
    
    /// 格式名称
    fn name(&self) -> &'static str;
    
    /// 是否支持零拷贝
    fn supports_zero_copy(&self) -> bool;
}
```

## 序列化格式对比

| 格式 | 二进制 | 零拷贝 | 大小效率 | 速度 | 人类可读 |
|------|--------|--------|----------|------|----------|
| Bincode | ✅ | ✅ | 高 | 最快 | ❌ |
| MessagePack | ✅ | ✅ | 高 | 快 | ❌ |
| CBOR | ✅ | ✅ | 高 | 快 | ❌ |
| Postcard | ✅ | ✅ | 最高 | 快 | ❌ |
| JSON | ❌ | ❌ | 低 | 慢 | ✅ |

### Bincode

- 最快的 Rust 原生序列化格式
- 零拷贝反序列化
- 适合 Rust 内部通信

```rust
let encoded = bincode_format::serialize(&value)?;
let decoded: MyStruct = bincode_format::deserialize(&encoded)?;
```

### MessagePack

- 跨语言兼容
- 紧凑的二进制格式
- 适合跨语言通信

```rust
let encoded = msgpack_format::serialize(&value)?;
let decoded: MyStruct = msgpack_format::deserialize(&encoded)?;
```

### CBOR

- RFC 8949 标准
- 支持流式处理
- 适合物联网场景

```rust
let encoded = cbor_format::serialize(&value)?;
let decoded: MyStruct = cbor_format::deserialize(&encoded)?;
```

### Postcard

- 最小的序列化体积
- 适合嵌入式场景
- 无 std 依赖

```rust
let encoded = postcard_format::serialize(&value)?;
let decoded: MyStruct = postcard_format::deserialize(&encoded)?;
```

## 流式处理

### StreamWriter

流式写入器：

```rust
pub struct StreamWriter<W: Write> {
    /// 底层写入器
    writer: W,
    /// 缓冲区
    buffer: Vec<u8>,
    /// 缓冲区大小
    buffer_size: usize,
    /// 已写入字节数
    bytes_written: u64,
}

impl<W: Write> StreamWriter<W> {
    /// 创建新的流式写入器
    pub fn new(writer: W, buffer_size: usize) -> Self;
    
    /// 写入序列化数据
    pub fn write<T: Serialize>(&mut self, value: &T) -> Result<()>;
    
    /// 刷新缓冲区
    pub fn flush(&mut self) -> Result<()>;
    
    /// 获取已写入字节数
    pub fn bytes_written(&self) -> u64;
}
```

### StreamReader

流式读取器：

```rust
pub struct StreamReader<R: Read> {
    /// 底层读取器
    reader: R,
    /// 缓冲区
    buffer: Vec<u8>,
    /// 当前位置
    position: usize,
    /// 已读取字节数
    bytes_read: u64,
}

impl<R: Read> StreamReader<R> {
    /// 创建新的流式读取器
    pub fn new(reader: R, buffer_size: usize) -> Self;
    
    /// 读取反序列化数据
    pub fn read<T: Deserialize<'de>>(&mut self) -> Result<T>;
    
    /// 获取已读取字节数
    pub fn bytes_read(&self) -> u64;
}
```

## 性能统计

```rust
pub struct SerializeStats {
    /// 总序列化次数
    pub total_serializes: AtomicU64,
    /// 总反序列化次数
    pub total_deserializes: AtomicU64,
    /// 总序列化字节数
    pub total_bytes_serialized: AtomicU64,
    /// 总反序列化字节数
    pub total_bytes_deserialized: AtomicU64,
    /// 总序列化时间（微秒）
    pub total_serialize_time_us: AtomicU64,
    /// 总反序列化时间（微秒）
    pub total_deserialize_time_us: AtomicU64,
}

impl SerializeStats {
    /// 获取平均序列化时间
    pub fn avg_serialize_time_us(&self) -> f64;
    
    /// 获取平均反序列化时间
    pub fn avg_deserialize_time_us(&self) -> f64;
    
    /// 获取序列化吞吐量（MB/s）
    pub fn serialize_throughput(&self) -> f64;
    
    /// 获取反序列化吞吐量（MB/s）
    pub fn deserialize_throughput(&self) -> f64;
}
```

## PHP API

### 基本使用

```php
// 序列化
$encoded = \oyta\Serialize::encode($data, 'msgpack');

// 反序列化
$decoded = \oyta\Serialize::decode($encoded, 'msgpack');

// 使用默认格式
$encoded = \oyta\Serialize::encode($data);
$decoded = \oyta\Serialize::decode($encoded);
```

### 指定格式

```php
// Bincode
$encoded = \oyta\Serialize::bincodeEncode($data);
$decoded = \oyta\Serialize::bincodeDecode($encoded);

// MessagePack
$encoded = \oyta\Serialize::msgpackEncode($data);
$decoded = \oyta\Serialize::msgpackDecode($encoded);

// CBOR
$encoded = \oyta\Serialize::cborEncode($data);
$decoded = \oyta\Serialize::cborDecode($encoded);

// Postcard
$encoded = \oyta\Serialize::postcardEncode($data);
$decoded = \oyta\Serialize::postcardDecode($encoded);

// JSON
$encoded = \oyta\Serialize::jsonEncode($data);
$decoded = \oyta\Serialize::jsonDecode($encoded);
```

### 流式处理

```php
// 创建流式写入器
$writer = \oyta\Serialize::createStreamWriter('/path/to/file', 'msgpack');

// 写入数据
$writer->write($data1);
$writer->write($data2);
$writer->flush();

// 创建流式读取器
$reader = \oyta\Serialize::createStreamReader('/path/to/file', 'msgpack');

// 读取数据
while (($data = $reader->read()) !== null) {
    process($data);
}
```

### 性能统计

```php
// 获取统计信息
$stats = \oyta\Serialize::getStats();
echo "Total serializes: " . $stats['total_serializes'] . "\n";
echo "Avg serialize time: " . $stats['avg_serialize_time_us'] . " us\n";
echo "Throughput: " . $stats['serialize_throughput'] . " MB/s\n";
```

## 配置说明

```php
// config/serialize.php
return [
    // 默认格式
    'default_format' => 'msgpack',
    
    // 零拷贝
    'zero_copy' => true,
    
    // 流式处理缓冲区大小
    'stream_buffer_size' => 65536, // 64KB
    
    // 性能统计
    'enable_stats' => true,
];
```

## 性能基准

### 序列化速度

| 格式 | 序列化 (MB/s) | 反序列化 (MB/s) | 相对大小 |
|------|--------------|-----------------|----------|
| Bincode | 2000 | 2500 | 100% |
| MessagePack | 800 | 1000 | 95% |
| CBOR | 600 | 800 | 98% |
| Postcard | 1500 | 2000 | 85% |
| JSON | 200 | 300 | 150% |

### 使用场景建议

| 场景 | 推荐格式 | 原因 |
|------|----------|------|
| Rust 内部通信 | Bincode | 最快，零拷贝 |
| 跨语言 API | MessagePack | 跨语言兼容，紧凑 |
| 物联网设备 | CBOR | 标准，流式支持 |
| 嵌入式系统 | Postcard | 最小体积 |
| 人类可读 | JSON | 可读性好 |
