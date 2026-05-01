# HTTP 模块

## 模块概述

HTTP 模块提供完整的 HTTP 服务端实现，基于 Axum 框架构建，支持静态文件服务、PHP 请求处理、中间件等功能。对应 ThinkPHP 8.0 的 HTTP 层。

## 模块结构

```
http/
├── mod.rs       # 模块入口
├── server.rs    # HTTP 服务器
├── request.rs   # 请求处理
├── response.rs  # 响应处理
└── cookie.rs    # Cookie 管理
```

## HTTP 服务器

### 基本配置

```php
<?php
// config/app.php
return [
    'http' => [
        'host' => '0.0.0.0',
        'port' => 8000,
        'workers' => 4,
        'max_connections' => 10000,
    ],
];
```

### 启动服务器

```bash
# 开发模式
oyta run

# 生产模式
oyta run --env=production

# 指定端口
oyta run --port=8080
```

## 请求处理 (Request)

### 获取请求参数

```php
<?php
namespace app\controller;

class Index
{
    public function index(Request $request)
    {
        // 获取所有参数
        $params = $request->param();
        
        // 获取单个参数
        $id = $request->param('id');
        $id = $request->param('id', 0); // 带默认值
        
        // 类型转换
        $id = $request->param('id/d'); // 转为整数
        $name = $request->param('name/s'); // 转为字符串
        $price = $request->param('price/f'); // 转为浮点数
        $ids = $request->param('ids/a'); // 转为数组
        
        // GET 参数
        $page = $request->get('page', 1);
        
        // POST 参数
        $data = $request->post('data');
        
        // JSON 数据
        $json = $request->json();
        
        // 文件上传
        $file = $request->file('avatar');
    }
}
```

### 请求信息

```php
<?php
// 请求方法
$method = $request->method();

// 请求路径
$path = $request->path();
$url = $request->url();
$fullUrl = $request->url(true);

// 请求头
$contentType = $request->header('content-type');
$headers = $request->header();

// IP 地址
$ip = $request->ip();

// User Agent
$ua = $request->header('user-agent');

// 判断请求类型
if ($request->isGet()) { }
if ($request->isPost()) { }
if ($request->isAjax()) { }
if ($request->isJson()) { }
```

### Cookie 操作

```php
<?php
// 获取 Cookie
$value = $request->cookie('name');

// 获取所有 Cookie
$cookies = $request->cookie();
```

### Session 操作

```php
<?php
// 获取 Session
$value = $request->session('key');

// 设置 Session
$request->session('key', 'value');

// 删除 Session
$request->session()->delete('key');

// 清空 Session
$request->session()->clear();
```

## 响应处理 (Response)

### 响应类型

```php
<?php
// 文本响应
return response('Hello World');

// HTML 响应
return response('<h1>Hello</h1>')->contentType('text/html');

// JSON 响应
return json(['code' => 200, 'data' => $data]);

// JSONP 响应
return jsonp($data, 'callback');

// 重定向
return redirect('/login');

// 视图响应
return view('index/index', ['name' => 'John']);
```

### 响应设置

```php
<?php
// 设置状态码
return response('Not Found')->code(404);

// 设置响应头
return response('data')
    ->header('X-Custom', 'value')
    ->header('Cache-Control', 'no-cache');

// 设置 Cookie
return response('ok')
    ->cookie('name', 'value', 3600)
    ->cookie('secure', 'value', 3600, '/', '', true, true, 'Lax');

// 设置内容类型
return response($xml)->contentType('application/xml');
```

## 中间件

### 定义中间件

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

### 注册中间件

```php
<?php
// 全局中间件
return [
    \app\middleware\Cors::class,
    \app\middleware\Log::class,
];

// 路由中间件
Route::group(function() {
    // 路由定义
})->middleware(\app\middleware\Auth::class);
```

## 静态文件服务

```php
<?php
// config/app.php
return [
    'static' => [
        'path' => public_path(),
        'cache' => 86400, // 缓存时间
    ],
];
```

静态文件默认从 `public/` 目录提供。

## 配置示例

```php
<?php
// config/app.php
return [
    'http' => [
        'host' => env('HTTP_HOST', '0.0.0.0'),
        'port' => env('HTTP_PORT', 8000),
        'workers' => env('HTTP_WORKERS', 4),
        'max_connections' => 10000,
        'keep_alive' => 75,
        'request_timeout' => 30,
    ],
    
    'cors' => [
        'allow_origin' => ['*'],
        'allow_methods' => ['GET', 'POST', 'PUT', 'DELETE'],
        'allow_headers' => ['Content-Type', 'Authorization'],
        'allow_credentials' => true,
        'max_age' => 86400,
    ],
];
```

## 性能特性

- **Axum 框架**：高性能异步 HTTP 框架
- **多 Worker 进程**：充分利用多核 CPU
- **连接池**：复用数据库和 Redis 连接
- **静态文件缓存**：减少磁盘 I/O
- **Keep-Alive**：复用 HTTP 连接
