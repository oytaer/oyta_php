# SIMD 加速模块 (SIMD)

## 模块概述

SIMD 模块利用 CPU 的 SIMD（单指令多数据）指令集进行向量化加速，显著提升字符串操作、数值计算、JSON 解析等场景的性能。

## 模块结构

```
simd/
├── mod.rs          # 模块入口
├── engine.rs       # SIMD 引擎
├── string_ops.rs   # 字符串操作
├── json_parse.rs   # JSON 解析
├── array_ops.rs    # 数组操作
└── hash.rs         # 哈希计算
```

## 支持的指令集

| 指令集 | 平台 | 说明 |
|--------|------|------|
| SSE2 | x86_64 | 128位向量 |
| SSE4.2 | x86_64 | 增强指令 |
| AVX2 | x86_64 | 256位向量 |
| AVX-512 | x86_64 | 512位向量 |
| NEON | ARM64 | 128位向量 |

## 主要类型

| 类型 | 说明 |
|------|------|
| SimdEngine | SIMD 引擎 |
| SimdCapabilities | CPU 能力检测 |
| SimdStringOps | 字符串操作 |
| SimdJsonParser | JSON 解析器 |
| SimdArrayOps | 数组操作 |
| SimdHash | 哈希计算 |

## 加速的操作

| 操作 | 加速倍数 |
|------|----------|
| 字符串查找 | 4-8x |
| JSON 解析 | 2-4x |
| 数组操作 | 4-8x |
| 哈希计算 | 2-4x |

## 使用示例

### 字符串操作

```php
<?php
// 自动使用 SIMD 加速
$pos = strpos($haystack, $needle);  // SIMD 加速
$lower = strtolower($string);        // SIMD 加速
$upper = strtoupper($string);        // SIMD 加速
```

### JSON 解析

```php
<?php
// 自动使用 SIMD 加速的 JSON 解析
$data = json_decode($jsonString, true);
```

## 技术实现

- 运行时 CPU 能力检测
- 自动选择最优指令集
- 透明加速，无需修改代码
- 支持 x86_64 和 ARM64
