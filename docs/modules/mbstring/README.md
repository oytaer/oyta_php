# 多字节字符串模块 (Mbstring)

## 模块概述

多字节字符串模块提供完整的 PHP mbstring 扩展功能，包括字符编码转换、多字节字符串操作、编码检测等。

## 模块结构

```
mbstring/
├── mod.rs          # 模块入口
├── encoding.rs     # 字符编码定义
├── convert.rs      # 编码转换
├── string.rs       # 多字节字符串操作
├── detect.rs       # 编码检测
├── http.rs         # HTTP 输入输出编码
└── regex.rs        # 多字节正则表达式
```

## 主要类型

| 类型 | 说明 |
|------|------|
| MbEncoding | 字符编码 |
| MbConverter | 编码转换器 |
| MbString | 多字节字符串 |
| MbDetector | 编码检测器 |
| MbRegex | 多字节正则 |

## 支持的编码

| 编码 | 说明 |
|------|------|
| UTF-8 | Unicode 编码 |
| GBK | 简体中文 |
| GB2312 | 简体中文 |
| BIG5 | 繁体中文 |
| Shift_JIS | 日文 |
| EUC-KR | 韩文 |
| ISO-8859-1 | 西欧语言 |

## 使用示例

### 字符串长度

```php
<?php
// 多字节字符串长度
$len = mb_strlen('你好世界', 'UTF-8');  // 4
```

### 字符串截取

```php
<?php
// 多字节字符串截取
$str = mb_substr('你好世界', 0, 2, 'UTF-8');  // 你好
```

### 编码转换

```php
<?php
// 编码转换
$utf8 = mb_convert_encoding($gbk, 'UTF-8', 'GBK');
```

### 编码检测

```php
<?php
// 检测编码
$encoding = mb_detect_encoding($str, ['UTF-8', 'GBK', 'BIG5']);
```

## 技术实现

- 基于 encoding_rs 库
- 支持多种字符编码
- 支持编码自动检测
- 支持多字节正则表达式
