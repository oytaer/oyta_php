# HTTP 服务模块 (HTTP)

## 模块概述

HTTP 服务模块提供 OYTAPHP 的 HTTP 服务功能，包括请求处理、响应构建、服务器启动、应用状态管理等。

## 模块结构

```
http/
├── mod.rs          # 模块入口
├── handler.rs      # 请求处理器
├── request.rs      # HTTP 请求
├── response.rs     # HTTP 响应
├── server.rs       # HTTP 服务器
└── state.rs        # 应用状态
```

## Request 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `get` | key: &str | Option<String> | 获取 GET 参数 |
| `post` | key: &str | Option<String> | 获取 POST 参数 |
| `method` | - | String | 获取请求方法 |
| `ip` | - | String | 获取客户端 IP |

## Response 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `create` | content: &str, status: u16 | Response | 创建响应 |
| `json` | data: &HashMap | Response | JSON 响应 |
| `html` | content: &str | Response | HTML 响应 |
| `redirect` | url: &str | Response | 重定向响应 |

## 使用示例

### 请求处理

```php
<?php
use oyta\facade\Request;

// 获取 GET 参数
$id = Request::get('id');

// 获取 POST 参数
$name = Request::post('name');

// 获取请求方法
$method = Request::method();

// 获取客户端 IP
$ip = Request::ip();
```

### 响应构建

```php
<?php
use oyta\facade\Response;

// JSON 响应
return Response::json([
    'code' => 0,
    'message' => 'success',
    'data' => $data,
]);

// HTML 响应
return Response::html('<h1>Hello World</h1>');

// 重定向
return Response::redirect('/login');
```

### 控制器示例

```php
<?php
namespace app\controller;

use oyta\facade\Request;
use oyta\facade\Response;

class UserController
{
    public function index()
    {
        $page = Request::get('page', 1);
        $users = Db::table('users')->page($page, 10)->select();
        
        return Response::json([
            'code' => 0,
            'data' => $users,
        ]);
    }
    
    public function store()
    {
        $name = Request::post('name');
        $email = Request::post('email');
        
        Db::table('users')->insert([
            'name' => $name,
            'email' => $email,
        ]);
        
        return Response::json([
            'code' => 0,
            'message' => '创建成功',
        ]);
    }
}
```

## 技术实现

- 基于 Axum 框架
- 异步请求处理
- 支持中间件
- 支持 WebSocket
