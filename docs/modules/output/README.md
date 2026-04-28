# 输出缓冲模块 (Output)

## 模块概述

输出缓冲模块提供完整的 PHP 输出缓冲控制功能，包括 ob_start、ob_get_contents、ob_end_flush 等。

## 模块结构

```
output/
├── mod.rs          # 模块入口
├── buffer.rs       # 输出缓冲栈
├── callback.rs     # 缓冲回调
├── handler.rs      # 处理器标志
└── types.rs        # 类型定义
```

## 主要类型

| 类型 | 说明 |
|------|------|
| OutputBuffer | 输出缓冲 |
| OutputCallback | 输出回调 |
| OutputHandler | 输出处理器 |
| BufferState | 缓冲状态 |

## 使用示例

### 输出缓冲

```php
<?php
// 开始输出缓冲
ob_start();

echo "Hello World";

// 获取缓冲内容
$content = ob_get_contents();

// 清空并关闭缓冲
ob_end_clean();

// 输出缓冲内容并关闭
ob_end_flush();
```

### 回调处理

```php
<?php
// 带回调的输出缓冲
ob_start(function($buffer) {
    return strtoupper($buffer);
});

echo "hello world";  // 输出: HELLO WORLD

ob_end_flush();
```

## 技术实现

- 输出缓冲栈
- 回调处理
- 处理器标志
- 缓冲状态管理
