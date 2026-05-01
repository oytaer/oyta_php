# Router 路由模块

## 模块概述

Router 模块提供灵活的路由定义和匹配功能，支持 RESTful 路由、路由组、中间件、路由缓存等。对应 ThinkPHP 8.0 的路由系统。

## 模块结构

```
router/
├── mod.rs          # 模块入口
├── types.rs        # 路由类型定义
├── registry.rs     # 路由注册表
├── facade.rs       # 路由门面
├── annotation.rs   # 注解路由
├── route_parser.rs # 路由解析器
└── cache.rs        # 路由缓存
```

## 基本路由

```php
<?php
use think\facade\Route;

// GET 路由
Route::get('user/:id', 'user/read');

// POST 路由
Route::post('user', 'user/create');

// PUT 路由
Route::put('user/:id', 'user/update');

// DELETE 路由
Route::delete('user/:id', 'user/delete');

// 多方法路由
Route::rule('user/:id', 'user/read', ['GET', 'POST']);

// 任意方法
Route::any('user/:id', 'user/action');
```

## 路由参数

```php
<?php
// 必填参数
Route::get('user/:id', 'user/read');

// 可选参数
Route::get('blog/:id/[:page]', 'blog/list');

// 参数约束
Route::get('user/:id', 'user/read')
    ->pattern(['id' => '\d+']);

// 全局约束
Route::pattern('id', '\d+');
```

## 路由组

```php
<?php
Route::group('api', function() {
    Route::get('user/:id', 'api/user/read');
    Route::post('user', 'api/user/create');
    Route::put('user/:id', 'api/user/update');
    Route::delete('user/:id', 'api/user/delete');
})->middleware(\app\middleware\Auth::class)
  ->allowCrossDomain();
```

## 资源路由

```php
<?php
// 创建 RESTful 资源路由
Route::resource('user', 'user');

// 生成的路由：
// GET    /user        -> user/index
// GET    /user/create -> user/create
// POST   /user        -> user/save
// GET    /user/:id    -> user/read
// GET    /user/:id/edit -> user/edit
// PUT    /user/:id    -> user/update
// DELETE /user/:id    -> user/delete

// 仅指定方法
Route::resource('user', 'user')->only(['index', 'read']);

// 排除方法
Route::resource('user', 'user')->except(['delete']);
```

## 闭包路由

```php
<?php
Route::get('hello/:name', function($name) {
    return 'Hello, ' . $name;
});
```

## 路由重定向

```php
<?php
// 永久重定向
Route::redirect('old', 'new', 301);

// 临时重定向
Route::redirect('old', 'new', 302);
```

## 路由中间件

```php
<?php
// 单个中间件
Route::get('admin', 'admin/index')
    ->middleware(\app\middleware\Auth::class);

// 多个中间件
Route::group('admin', function() {
    // 路由定义
})->middleware([
    \app\middleware\Auth::class,
    \app\middleware\Log::class,
]);
```

## 路由域名

```php
<?php
Route::domain('api', function() {
    Route::get('user/:id', 'api/user/read');
});

// 带参数的域名
Route::domain(':sub.example.com', function($sub) {
    Route::get('/', 'index/domain');
});
```

## 路由缓存

```php
<?php
// 开启路由缓存（生产环境）
Route::cache(true);

// 清除路由缓存
php oyta clear:route
```

## 注解路由

```php
<?php
namespace app\controller;

/**
 * @Route("/api/user")
 */
class User
{
    /**
     * @Route("/:id", methods={"GET"})
     */
    public function read($id)
    {
        // ...
    }
    
    /**
     * @Route("/", methods={"POST"})
     */
    public function create()
    {
        // ...
    }
}
```

## 路由配置

```php
<?php
// config/route.php
return [
    // 路由模式：1=普通模式 2=混合模式 3=强制路由
    'url_route_on' => true,
    'url_route_must' => false,
    
    // 路由缓存
    'route_cache' => env('APP_DEBUG') === false,
    
    // 域名部署
    'url_domain_deploy' => true,
    
    // 路由映射文件
    'route_map' => [],
];
```

## 性能特性

- **matchit 库**：高性能路由匹配
- **路由缓存**：编译路由规则缓存
- **延迟解析**：按需解析路由参数
