# 序列化模块 (Serialize)

## 模块概述

序列化模块提供高性能的二进制序列化支持，支持 Bincode、MessagePack、CBOR、Postcard 等多种格式。

## 模块结构

```
serialize/
├── mod.rs          # 模块入口
├── engine.rs       # 序列化引擎
├── bincode_format.rs # Bincode 格式
├── msgpack_format.rs # MessagePack 格式
├── cbor_format.rs  # CBOR 格式
├── postcard_format.rs # Postcard 格式
└── stream.rs       # 流式序列化
```

## 支持的格式

| 格式 | 特点 | 性能 |
|------|------|------|
| Bincode | Rust 原生，零拷贝 | 最快 |
| MessagePack | 跨语言兼容 | 快 |
| CBOR | IETF 标准 | 快 |
| Postcard | 嵌入式友好 | 快 |

## 性能对比

| 格式 | 相比 PHP serialize | 体积减少 |
|------|-------------------|----------|
| Bincode | 10-15x | 60-65% |
| MessagePack | 8-12x | 50-60% |
| CBOR | 8-12x | 50-60% |
| Postcard | 10-15x | 55-65% |

## 使用示例

### 序列化

```php
<?php
use oyta\facade\Serialize;

// 序列化为 MessagePack
$packed = Serialize::encode($data, 'msgpack');

// 反序列化
$data = Serialize::decode($packed, 'msgpack');
```

### 配置示例

```php
<?php
// config/serialize.php
return [
    'default' => 'msgpack',
    'formats' => [
        'bincode' => ['compact' => true],
        'msgpack' => ['compatibility' => true],
        'cbor' => ['canonical' => true],
    ],
];
```

## 技术实现

- 零拷贝序列化
- 多格式支持
- 流式处理
- 跨语言兼容
