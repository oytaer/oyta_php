# 日期时间模块 (DateTime)

## 模块概述

日期时间模块提供 PHP 风格的日期时间处理功能，包括 DateTime、DateTimeImmutable、DateTimeZone、DateInterval、DatePeriod 类。

## 模块结构

```
datetime/
├── mod.rs          # 模块入口
├── datetime.rs     # DateTime 类
├── timezone.rs     # DateTimeZone 类
├── interval.rs     # DateInterval 类
├── period.rs       # DatePeriod 类
├── parser.rs       # 日期解析器
├── format.rs       # 日期格式化
├── relative.rs     # 相对时间解析
└── facade.rs       # Date 门面
```

## Date 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `now` | - | DateTime | 获取当前时间 |
| `format` | format: &str | String | 格式化日期 |
| `parse` | date: &str | DateTime | 解析日期字符串 |
| `timestamp` | - | i64 | 获取时间戳 |

## 使用示例

### 创建日期

```php
<?php
use oyta\facade\Date;

// 当前时间
$now = Date::now();

// 指定日期
$date = new DateTime('2024-01-01');

// 从时间戳创建
$date = new DateTime('@' . time());
```

### 日期格式化

```php
<?php
$date = new DateTime('2024-01-15 10:30:00');

echo $date->format('Y-m-d');      // 2024-01-15
echo $date->format('Y-m-d H:i:s'); // 2024-01-15 10:30:00
echo $date->format('Y年m月d日');    // 2024年01月15日
```

### 日期操作

```php
<?php
$date = new DateTime('2024-01-15');

// 添加时间
$date->modify('+1 day');    // 2024-01-16
$date->modify('+1 month');  // 2024-02-16
$date->modify('+1 year');   // 2025-02-16

// 减少时间
$date->modify('-1 day');    // 2025-02-15

// 时间差
$diff = $date->diff(new DateTime('2024-01-01'));
echo $diff->days;  // 天数差
```

### 时区处理

```php
<?php
// 设置时区
$date = new DateTime('now', new DateTimeZone('Asia/Shanghai'));

// 切换时区
$date->setTimezone(new DateTimeZone('UTC'));
```

## 格式化字符

| 字符 | 说明 | 示例 |
|------|------|------|
| Y | 4位年份 | 2024 |
| m | 2位月份 | 01-12 |
| d | 2位日期 | 01-31 |
| H | 24小时制 | 00-23 |
| i | 分钟 | 00-59 |
| s | 秒 | 00-59 |

## 技术实现

- 基于 chrono 库
- 支持时区处理
- 支持日期计算
- 支持相对时间解析
