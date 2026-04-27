# Facade 门面系统

## 概述

Facade 门面系统提供了静态代理访问底层服务的能力，让开发者可以通过简洁的静态方法调用访问复杂的服务。

## 内置 Facade

### Db - 数据库操作

```php
use oyta\facade\Db;

// 查询所有
$users = Db::table('users')->select();

// 条件查询
$users = Db::table('users')
    ->where('status', 'active')
    ->where('age', '>', 18)
    ->order('created_at', 'desc')
    ->limit(10)
    ->select();

// 插入
Db::table('users')->insert([
    'name' => 'John',
    'email' => 'john@example.com',
]);

// 更新
Db::table('users')
    ->where('id', 1)
    ->update(['name' => 'Jane']);

// 删除
Db::table('users')->where('id', 1)->delete();

// 聚合查询
$count = Db::table('users')->count();
$max = Db::table('users')->max('age');
$avg = Db::table('users')->avg('score');

// 事务
Db::transaction(function () {
    Db::table('users')->insert([...]);
    Db::table('profiles')->insert([...]);
});

// 原生 SQL
$users = Db::query('SELECT * FROM users WHERE id = ?', [1]);
Db::execute('UPDATE users SET name = ? WHERE id = ?', ['John', 1]);
```

### Cache - 缓存操作

```php
use oyta\facade\Cache;

// 获取缓存
$value = Cache::get('key');

// 设置缓存
Cache::put('key', 'value', 3600); // 缓存 1 小时

// 永久缓存
Cache::forever('key', 'value');

// 检查存在
if (Cache::has('key')) {
    // ...
}

// 删除缓存
Cache::forget('key');

// 清空缓存
Cache::flush();

// 获取或设置
$value = Cache::remember('key', 3600, function () {
    return expensiveOperation();
});

// 自增/自减
Cache::increment('counter', 1);
Cache::decrement('counter', 1);

// 标签缓存
Cache::tags(['user', 'profile'])->put('user:1:profile', $profile, 3600);
Cache::tags(['user'])->flush();
```

### Config - 配置操作

```php
use oyta\facade\Config;

// 获取配置
$value = Config::get('app.name');
$value = Config::get('database.connections.mysql.host');

// 获取配置（带默认值）
$value = Config::get('app.debug', false);

// 设置配置
Config::set('app.name', 'My App');

// 检查配置存在
if (Config::has('app.timezone')) {
    // ...
}

// 获取所有配置
$all = Config::all();
```

### Log - 日志操作

```php
use oyta\facade\Log;

// 记录日志
Log::info('User logged in', ['user_id' => 1]);
Log::error('Database error', ['error' => $e->getMessage()]);
Log::warning('Cache miss', ['key' => 'user:1']);
Log::debug('Request data', $request->all());

// 指定通道
Log::channel('daily')->info('Daily log message');

// 写入文件
Log::write('custom', 'Custom log message', ['context' => 'data']);
```

### Event - 事件操作

```php
use oyta\facade\Event;

// 触发事件
Event::dispatch('user.registered', $user);

// 监听事件
Event::listen('user.registered', function ($user) {
    // 发送欢迎邮件
});

// 订阅事件
Event::subscribe(UserEventSubscriber::class);

// 是否有监听器
if (Event::hasListeners('user.registered')) {
    // ...
}
```

### Validate - 验证操作

```php
use oyta\facade\Validate;

// 验证数据
$validator = Validate::make($data, [
    'name' => 'required|string|max:255',
    'email' => 'required|email|unique:users',
    'password' => 'required|string|min:8|confirmed',
]);

if ($validator->fails()) {
    $errors = $validator->errors();
}

// 验证并返回数据
$validated = Validate::validate($data, $rules);

// 自定义错误消息
$validator = Validate::make($data, $rules, [
    'name.required' => '请输入姓名',
    'email.email' => '请输入有效的邮箱地址',
]);
```

### Session - 会话操作

```php
use oyta\facade\Session;

// 获取会话
$value = Session::get('key');

// 设置会话
Session::put('key', 'value');

// 获取并删除
$value = Session::pull('key');

// 检查存在
if (Session::has('key')) {
    // ...
}

// 删除
Session::forget('key');

// 清空
Session::flush();

// 闪存（下次请求后删除）
Session::flash('status', 'Task completed');

// 获取闪存
$status = Session::get('status');
```

### View - 视图操作

```php
use oyta\facade\View;

// 渲染视图
$html = View::render('index', ['title' => 'Home']);

// 渲染并返回响应
return View::display('user/profile', ['user' => $user]);

// 检查视图存在
if (View::exists('user/profile')) {
    // ...
}

// 共享数据
View::share('siteName', 'My Site');

// 添加视图路径
View::addPath('/path/to/views');
```

## 自定义 Facade

### 创建 Facade

```php
<?php
namespace app\facade;

use oyta\Facade;

class Payment extends Facade
{
    protected static function getFacadeAccessor()
    {
        return 'payment';
    }
}
```

### 注册服务

```php
<?php
namespace app\service;

class PaymentService
{
    public function process($amount)
    {
        // 处理支付
    }
}
```

### 配置服务提供者

```php
// config/provider.php
return [
    'payment' => \app\service\PaymentService::class,
];
```

### 使用 Facade

```php
use app\facade\Payment;

Payment::process(100);
```

## Facade 列表

| Facade | 服务 | 说明 |
|--------|------|------|
| `Db` | 数据库 | 数据库查询构建器 |
| `Cache` | 缓存 | 缓存操作 |
| `Config` | 配置 | 配置管理 |
| `Log` | 日志 | 日志记录 |
| `Event` | 事件 | 事件分发 |
| `Validate` | 验证 | 数据验证 |
| `Session` | 会话 | 会话管理 |
| `View` | 视图 | 视图渲染 |
| `Request` | 请求 | 当前请求 |
| `Response` | 响应 | 响应构建 |
| `Cookie` | Cookie | Cookie 管理 |
| `File` | 文件 | 文件操作 |
| `Storage` | 存储 | 文件存储 |
| `Queue` | 队列 | 队列操作 |
| `Mail` | 邮件 | 邮件发送 |
| `Redis` | Redis | Redis 操作 |
| `Monitor` | 监控 | 性能监控 |
| `Serialize` | 序列化 | 序列化操作 |
| `Microservice` | 微服务 | 微服务调用 |
| `Http3` | HTTP/3 | HTTP/3 客户端 |
| `Simd` | SIMD | SIMD 加速 |
| `Search` | 搜索 | 搜索引擎 |
| `Sandbox` | 沙箱 | 沙箱执行 |
| `Timer` | 定时器 | 定时任务 |
| `Coroutine` | 协程 | 协程操作 |
