# Composer 依赖管理模块 (Composer)

## 模块概述

Composer 依赖管理模块提供 PHP Composer 兼容的依赖管理功能，支持 composer.json 解析、依赖解析、包下载、自动加载生成。

## 模块结构

```
composer/
├── mod.rs          # 模块入口
├── manager.rs      # Composer 管理器
├── parser.rs       # composer.json 解析
├── resolver.rs     # 依赖解析
├── packagist.rs    # Packagist API
├── downloader.rs   # 包下载
├── extractor.rs    # 包解压
├── lock.rs         # composer.lock
└── autoload.rs     # 自动加载生成
```

## 支持的命令

| 命令 | 说明 |
|------|------|
| install | 安装依赖 |
| update | 更新依赖 |
| require | 添加依赖 |
| remove | 移除依赖 |
| dump-autoload | 重新生成自动加载 |
| show | 查看已安装的包 |
| outdated | 查看过时的包 |
| validate | 验证 composer.json |
| config | 查看 Composer 配置 |
| clear-cache | 清除 Composer 缓存 |
| diagnose | 诊断 Composer 问题 |

## 使用示例

### 安装依赖

```bash
oyta composer install
oyta composer install --no-dev
```

### 添加依赖

```bash
oyta composer require monolog/monolog
oyta composer require --dev phpunit/phpunit
```

### 更新依赖

```bash
oyta composer update
oyta composer update monolog/monolog
```

### 生成自动加载

```bash
oyta composer dump-autoload
oyta composer dump-autoload --optimize
```

## 技术实现

- composer.json/composer.lock 解析
- Packagist API 集成
- 依赖版本解析
- PSR-4 自动加载生成
