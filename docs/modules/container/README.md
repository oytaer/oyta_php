# 容器模块 (Container)

## 模块概述

容器模块提供 OYTAPHP 的 IoC 容器和依赖注入功能，对应 ThinkPHP 8.0 的 Container 类。支持服务绑定、依赖解析、单例模式等功能。

## 模块结构

```
container/
├── mod.rs          # 模块入口
├── facade.rs       # Container 门面
├── container.rs    # 容器核心
└── service_provider.rs # 服务提供者
```

## Container 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `make` | abstract: &str | Option<Value> | 从容器获取实例 |
| `bind` | abstract: &str, concrete: Value | - | 绑定服务 |
| `singleton` | abstract: &str, concrete: Value | - | 绑定单例 |
| `has` | abstract: &str | bool | 检查绑定是否存在 |
| `instance` | abstract: &str, instance: Value | - | 注册实例 |
| `alias` | abstract: &str, alias: &str | - | 设置别名 |

## 使用示例

### 服务绑定

```php
<?php
use oyta\facade\Container;

// 绑定服务
Container::bind('cache', function() {
    return new \app\service\CacheService();
});

// 绑定单例
Container::singleton('db', function() {
    return new \app\service\DatabaseService();
});

// 注册实例
Container::instance('config', $configInstance);
```

### 获取服务

```php
<?php
use oyta\facade\Container;

// 获取服务实例
$cache = Container::make('cache');
$db = Container::make('db');

// 检查服务是否存在
if (Container::has('cache')) {
    $cache = Container::make('cache');
}
```

### 依赖注入

```php
<?php
namespace app\controller;

use app\service\UserService;

class UserController
{
    protected $userService;
    
    // 构造函数注入
    public function __construct(UserService $userService)
    {
        $this->userService = $userService;
    }
    
    public function index()
    {
        $users = $this->userService->getAll();
        return json($users);
    }
}
```

### 服务提供者

```php
<?php
namespace app\provider;

use oyta\container\ServiceProvider;

class CacheServiceProvider extends ServiceProvider
{
    public function register()
    {
        $this->app->singleton('cache', function() {
            return new \app\service\CacheService();
        });
    }
    
    public function boot()
    {
        // 启动逻辑
    }
}
```

## 技术实现

- 基于 IoC 容器模式
- 支持依赖自动注入
- 支持单例模式
- 支持服务提供者
