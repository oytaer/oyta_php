# 日志模块 (Logging)

## 模块概述

日志模块提供 OYTAPHP 的日志记录功能，支持多种日志通道、日志级别、日志格式化等功能。

## 模块结构

```
logging/
├── mod.rs          # 模块入口
├── facade.rs       # Log 门面
├── setup.rs        # 日志配置
├── file_writer.rs  # 文件写入器
└── channel.rs      # 日志通道
```

## Log 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `info` | message: &str | - | 记录 INFO 日志 |
| `error` | message: &str | - | 记录 ERROR 日志 |
| `warning` | message: &str | - | 记录 WARNING 日志 |
| `warn` | message: &str | - | 记录 WARNING 日志（别名） |
| `debug` | message: &str | - | 记录 DEBUG 日志 |

## 日志级别

| 级别 | 说明 |
|------|------|
| DEBUG | 调试信息 |
| INFO | 一般信息 |
| WARNING | 警告信息 |
| ERROR | 错误信息 |

## 使用示例

### 记录日志

```php
<?php
use oyta\facade\Log;

// 记录不同级别的日志
Log::info('用户登录成功');
Log::warning('磁盘空间不足');
Log::error('数据库连接失败');
Log::debug('调试信息: ' . json_encode($data));
```

### 配置示例

```php
<?php
// config/log.php
return [
    'default' => 'file',
    'channels' => [
        'file' => [
            'driver' => 'file',
            'path' => runtime_path('log'),
            'level' => 'debug',
            'days' => 30,
        ],
        'stdout' => [
            'driver' => 'stdout',
            'level' => 'info',
        ],
    ],
    'format' => '[{datetime}] {level}: {message}',
];
```

## 技术实现

- 基于 tracing 库
- 多通道支持
- 日志轮转
- 异步写入
