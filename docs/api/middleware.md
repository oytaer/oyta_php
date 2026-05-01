# Middleware 中间件模块

## 模块结构

```
middleware/
├── mod.rs       # 模块入口
└── builtin.rs   # 内置中间件
```

## 定义中间件

```php
<?php
namespace app\middleware;

class Auth
{
    public function handle($request, \Closure $next)
    {
        // 前置中间件
        if (!$request->session('user_id')) {
            return redirect('/login');
        }
        
        $response = $next($request);
        
        // 后置中间件
        return $response;
    }
}
```

## 注册中间件

```php
<?php
// config/middleware.php
return [
    // 全局中间件
    'global' => [
        \app\middleware\Cors::class,
        \app\middleware\Log::class,
    ],
    
    // 路由中间件
    'route' => [
        'auth' => \app\middleware\Auth::class,
        'throttle' => \app\middleware\Throttle::class,
    ],
];
```

## 使用中间件

```php
<?php
// 路由中使用
Route::group('admin', function() {
    // 路由定义
})->middleware('auth');

// 控制器中使用
class UserController
{
    protected $middleware = [
        'auth' => ['except' => ['login', 'register']],
    ];
}
```

## 内置中间件

- **Cors**: 跨域处理
- **Auth**: 认证检查
- **Throttle**: 请求限流
- **Log**: 请求日志
