# 配置模块 (Config)

## 模块概述

配置模块提供 OYTAPHP 的配置管理功能，支持 PHP 配置文件、环境变量、多源配置。

## 模块结构

```
config/
├── mod.rs          # 模块入口
├── facade.rs       # Config 门面
├── loader.rs       # 配置加载器
└── store.rs        # 配置存储
```

## Config 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `get` | key: &str | Option<Value> | 获取配置值 |
| `set` | key: &str, value: Value | - | 设置配置值 |
| `has` | key: &str | bool | 检查配置是否存在 |

## Env 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `get` | key: &str | Option<String> | 获取环境变量 |
| `has` | key: &str | bool | 检查环境变量是否存在 |

## 使用示例

### 配置文件

```php
<?php
// config/app.php
return [
    'name' => 'OYTAPHP',
    'debug' => true,
    'timezone' => 'Asia/Shanghai',
    'default_lang' => 'zh-cn',
];
```

### 读取配置

```php
<?php
use oyta\facade\Config;

// 获取配置
$name = Config::get('app.name');
$debug = Config::get('app.debug');

// 获取配置，带默认值
$timeout = Config::get('app.timeout', 30);

// 检查配置是否存在
if (Config::has('app.name')) {
    echo "应用名称: " . Config::get('app.name');
}

// 设置配置
Config::set('app.debug', false);
```

### 环境变量

```php
<?php
use oyta\facade\Env;

// 获取环境变量
$dbHost = Env::get('DB_HOST');
$dbPort = Env::get('DB_PORT', 3306);

// 检查环境变量
if (Env::has('APP_DEBUG')) {
    $debug = Env::get('APP_DEBUG') === 'true';
}
```

### .env 文件

```ini
# 应用配置
APP_NAME=OYTAPHP
APP_DEBUG=true

# 数据库配置
DB_HOST=127.0.0.1
DB_PORT=3306
DB_NAME=test
DB_USER=root
DB_PASS=
```

## 配置目录结构

```
config/
├── app.php        # 应用配置
├── cache.php      # 缓存配置
├── database.php   # 数据库配置
├── session.php    # Session 配置
├── view.php       # 视图配置
├── route.php      # 路由配置
└── middleware.php # 中间件配置
```

## 技术实现

- 支持 PHP 配置文件格式
- 支持 .env 环境变量文件
- 支持配置缓存
- 支持热重载配置
