# Container 依赖注入容器模块

## 模块结构

```
container/
├── mod.rs              # 模块入口
├── container.rs        # IoC 容器实现
├── service_provider.rs # 服务提供者
└── facade.rs           # 容器门面
```

## 基本使用

```php
<?php
// 绑定闭包
Container::bind('cache', function($container) {
    return new Cache($container->make('config'));
});

// 绑定单例
Container::singleton('db', function($container) {
    return new Database($container->make('config'));
});

// 绑定实例
Container::instance('config', $configInstance);

// 解析
$cache = Container::make('cache');
$db = Container::get('db');

// 检查绑定
if (Container::has('cache')) {
    // ...
}

// 移除绑定
Container::forget('cache');
```

## 自动注入

```php
<?php
class UserController
{
    // 自动注入
    public function __construct(
        private UserService $userService,
        private Cache $cache,
    ) {}
    
    public function index(Request $request)
    {
        // $request 自动注入
    }
}
```

## 服务提供者

```php
<?php
namespace app\provider;

class CacheServiceProvider extends ServiceProvider
{
    public function register()
    {
        $this->app->singleton('cache', function($app) {
            return new CacheManager($app['config']['cache']);
        });
    }
    
    public function boot()
    {
        // 启动逻辑
    }
}
```
