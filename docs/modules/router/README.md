# 路由模块 (Router)

## 模块概述

路由模块提供 OYTAPHP 的路由管理功能，对应 ThinkPHP 8.0 的 Route 门面。支持自动路由、定义路由、资源路由、路由分组等功能。

## 模块结构

```
router/
├── mod.rs          # 模块入口
├── facade.rs       # Route 门面
├── registry.rs     # 路由注册表
├── types.rs        # 路由类型定义
├── route_parser.rs # 路由解析器
└── matcher.rs      # 路由匹配器
```

## Route 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `init` | - | - | 初始化路由门面 |
| `get` | path: &str, handler: &str | - | 注册 GET 路由 |
| `post` | path: &str, handler: &str | - | 注册 POST 路由 |
| `put` | path: &str, handler: &str | - | 注册 PUT 路由 |
| `delete` | path: &str, handler: &str | - | 注册 DELETE 路由 |
| `patch` | path: &str, handler: &str | - | 注册 PATCH 路由 |
| `head` | path: &str, handler: &str | - | 注册 HEAD 路由 |
| `options` | path: &str, handler: &str | - | 注册 OPTIONS 路由 |
| `any` | path: &str, handler: &str | - | 注册任意方法路由 |
| `match_methods` | methods: &[HttpMethod], path: &str, handler: &str | - | 注册多方法路由 |
| `group` | group: RouteGroup | - | 注册路由组 |
| `resource` | name: &str, controller: &str | - | 注册资源路由 |
| `miss` | handler: &str | - | 设置 404 兜底路由 |
| `match_route` | method: HttpMethod, path: &str | Option<RouteMatch> | 匹配路由 |
| `all_routes` | - | Vec<Route> | 获取所有路由 |
| `count` | - | usize | 获取路由数量 |
| `clear` | - | - | 清除所有路由 |

## 使用示例

### 基础路由

```php
<?php
use oyta\facade\Route;

// GET 路由
Route::get('/user/{id}', 'UserController@show');

// POST 路由
Route::post('/user', 'UserController@store');

// PUT 路由
Route::put('/user/{id}', 'UserController@update');

// DELETE 路由
Route::delete('/user/{id}', 'UserController@destroy');

// 任意 HTTP 方法
Route::any('/test', 'TestController@index');

// 匹配多个方法
Route::match_methods(['GET', 'POST'], '/form', 'FormController@handle');
```

### 路由分组

```php
<?php
use oyta\facade\Route;

// 路由组
Route::group([
    'name' => 'admin',
    'prefix' => 'admin',
    'middleware' => ['auth'],
], function() {
    Route::get('/dashboard', 'AdminController@dashboard');
    Route::get('/users', 'AdminController@users');
});
```

### 资源路由

```php
<?php
use oyta\facade\Route;

// 资源路由自动生成 RESTful 路由
Route::resource('user', 'UserController');

// 生成的路由：
// GET    /user           -> UserController@index
// GET    /user/create    -> UserController@create
// POST   /user           -> UserController@store
// GET    /user/{id}      -> UserController@show
// GET    /user/{id}/edit -> UserController@edit
// PUT    /user/{id}      -> UserController@update
// DELETE /user/{id}      -> UserController@destroy
```

### 路由参数

```php
<?php
use oyta\facade\Route;

// 必选参数
Route::get('/user/{id}', 'UserController@show');

// 可选参数
Route::get('/page/{id?}', 'PageController@show');

// 参数约束
Route::get('/user/{id}', 'UserController@show')
    ->where('id', '\d+');
```

### MISS 路由

```php
<?php
use oyta\facade\Route;

// 设置 404 兜底路由
Route::miss('ErrorController@notFound');
```

## 自动路由

OYTAPHP 支持自动路由，无需手动定义路由：

```
访问 /index/index 会自动映射到 app\controller\IndexController::index
访问 /user/show/id/1 会自动映射到 app\controller\UserController::show(['id' => 1])
```

## 路由匹配流程

1. 解析请求 URL 和 HTTP 方法
2. 遍历路由注册表查找匹配
3. 提取路由参数
4. 执行中间件
5. 调用控制器方法

## 技术实现

- 使用 Radix Tree 优化路由匹配性能
- 支持正则表达式参数约束
- 支持路由缓存提升性能
- 线程安全的路由注册表
