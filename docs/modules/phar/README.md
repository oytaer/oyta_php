# Phar 归档模块 (Phar)

## 模块概述

Phar 归档模块实现 PHP Phar 归档功能，包括 Phar 文件创建和读取、Phar 清单解析和生成、Phar Stub 处理、文件压缩支持等。

## 模块结构

```
phar/
├── mod.rs          # 模块入口
├── phar.rs         # Phar 核心
├── manifest.rs     # 清单处理
├── stub.rs         # Stub 处理
└── compression.rs  # 压缩支持
```

## 主要类型

| 类型 | 说明 |
|------|------|
| Phar | Phar 归档 |
| PharManifest | Phar 清单 |
| PharEntry | Phar 条目 |
| PharStub | Phar Stub |
| PharCompression | 压缩方式 |

## 支持的压缩方式

| 压缩方式 | 说明 |
|----------|------|
| None | 无压缩 |
| Gzip | Gzip 压缩 |
| Bzip2 | Bzip2 压缩 |

## 使用示例

### 创建 Phar

```php
<?php
$phar = new Phar('app.phar');
$phar->startBuffering();
$phar->addFromString('index.php', '<?php echo "Hello";');
$phar->setStub('<?php __HALT_COMPILER(); ?>');
$phar->stopBuffering();
```

### 读取 Phar

```php
<?php
$phar = new Phar('app.phar');
$content = $phar['index.php']->getContent();
```

## 技术实现

- 支持 Phar 文件格式
- 支持压缩
- 支持 Stub 处理
- 支持签名验证
