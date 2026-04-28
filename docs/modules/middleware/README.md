# 中间件模块 (Middleware)

## 模块概述

中间件模块提供 OYTAPHP 的中间件功能，对应 ThinkPHP 8.0 的中间件系统。支持洋葱模型、全局/路由/控制器中间件。

## 模块结构

```
middleware/
├── mod.rs          # 模块入口
├── facade.rs       # Middleware 门面
├── dispatcher.rs   # 中间件调度器
├── types.rs        # 类型定义
└── builtin/        # 内置中间件
    ├── session.rs  # Session 初始化
    ├── cors.rs     # 跨域处理
    ├── lang.rs     # 语言包加载
    └── cache.rs    # 请求缓存
```

## Middleware 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `add` | middleware: &str | - | 添加中间件 |
| `remove` | middleware: &str | - | 移除中间件 |
| `has` | middleware: &str | bool | 检查中间件是否存在 |
| `clear` | - | - | 清空中间件 |
| `get` | - | Vec<String> | 获取所有中间件 |

## 使用示例

### 定义中间件

```php
<?php
namespace app\middleware;

class Auth
{
    public function handle($request, \Closure $next)
    {
        // 前置中间件
        if (!session('user_id')) {
            return redirect('/login');
        }
        
        $response = $next($request);
        
        // 后置中间件
        return $response;
    }
}
```

### 全局中间件

```php
<?php
// config/middleware.php
return [
    'global' => [
        \app\middleware\SessionInit::class,
        \app\middleware\CORS::class,
    ],
];
```

### 路由中间件

```php
<?php
use oyta\facade\Route;

Route::group([
    'middleware' => ['auth'],
], function() {
    Route::get('/dashboard', 'AdminController@dashboard');
});
```

### 控制器中间件

```php
<?php
namespace app\controller;

class UserController
{
    protected $middleware = [
        'auth' => ['except' => ['login', 'register']],
    ];
    
    public function profile()
    {
        return '用户中心';
    }
    
    public function login()
    {
        return '登录页面';
    }
}
```

## 内置中间件

| 中间件 | 说明 |
|--------|------|
| SessionInit | Session 初始化 |
| CORS | 跨域请求处理 |
| LoadLangPack | 语言包加载 |
| RequestCache | 请求缓存 |

## 洋葱模型

```
请求 → 中间件1 → 中间件2 → 控制器 → 中间件2 → 中间件1 → 响应
```

## 技术实现

- 使用洋葱模型实现中间件链
- 支持前置和后置中间件
- 支持中间件参数传递
- 线程安全的中间件管理
