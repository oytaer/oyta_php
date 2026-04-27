# 助手函数

## 概述

OYTAPHP 提供了丰富的内置助手函数，简化常见操作。

## 数组函数

### arr_get

从数组中获取值，支持点号分隔的键：

```php
$value = arr_get($array, 'key.subkey', $default);
```

### arr_set

设置数组值，支持点号分隔的键：

```php
arr_set($array, 'key.subkey', $value);
```

### arr_has

检查数组键是否存在：

```php
if (arr_has($array, 'key.subkey')) {
    // ...
}
```

### arr_forget

删除数组键：

```php
arr_forget($array, 'key.subkey');
```

### arr_only

只获取指定的键：

```php
$subset = arr_only($array, ['name', 'email']);
```

### arr_except

排除指定的键：

```php
$filtered = arr_except($array, ['password', 'token']);
```

### arr_pluck

从数组中提取指定键的值：

```php
$names = arr_pluck($users, 'name');
$emails = arr_pluck($users, 'email', 'id');
```

### arr_first / arr_last

获取第一个/最后一个元素：

```php
$first = arr_first($array, fn($value) => $value > 10);
$last = arr_last($array, fn($value) => $value > 10);
```

## 字符串函数

### str_random

生成随机字符串：

```php
$random = str_random(32); // 32 位随机字符串
```

### str_slug

生成 URL 友好的 slug：

```php
$slug = str_slug('Hello World'); // "hello-world"
```

### str_limit

截断字符串：

```php
$limited = str_limit('Long text here', 10); // "Long te..."
$limited = str_limit('Long text here', 10, '…'); // "Long text…"
```

### str_starts_with / str_ends_with

检查字符串开头/结尾：

```php
if (str_starts_with($str, 'Hello')) {
    // ...
}

if (str_ends_with($str, '.php')) {
    // ...
}
```

### str_contains

检查字符串包含：

```php
if (str_contains($str, 'needle')) {
    // ...
}
```

### str_replace_first / str_replace_last

替换第一个/最后一个匹配：

```php
$new = str_replace_first('a', 'b', 'abcabc'); // "bbcabc"
$new = str_replace_last('a', 'b', 'abcabc'); // "abcbc"
```

### str_camel / str_snake / str_kebab / str_studly

字符串格式转换：

```php
str_camel('hello_world'); // "helloWorld"
str_snake('helloWorld'); // "hello_world"
str_kebab('helloWorld'); // "hello-world"
str_studly('hello_world'); // "HelloWorld"
```

## 路径函数

### base_path

获取项目根目录路径：

```php
$path = base_path(); // /path/to/project
$path = base_path('config/app.php'); // /path/to/project/config/app.php
```

### app_path

获取 app 目录路径：

```php
$path = app_path(); // /path/to/project/app
$path = app_path('service/UserService.php');
```

### config_path

获取 config 目录路径：

```php
$path = config_path(); // /path/to/project/config
$path = config_path('database.php');
```

### runtime_path

获取 runtime 目录路径：

```php
$path = runtime_path(); // /path/to/project/runtime
$path = runtime_path('log/app.log');
```

### public_path

获取 public 目录路径：

```php
$path = public_path(); // /path/to/project/public
$path = public_path('uploads/avatar.jpg');
```

## URL 函数

### url

生成 URL：

```php
$url = url('user/profile');
$url = url('user/profile', ['id' => 1]);
$url = url()->full(); // 当前完整 URL
$url = url()->current(); // 当前 URL（不含查询参数）
```

### asset

生成静态资源 URL：

```php
$url = asset('css/app.css');
$url = asset('js/app.js');
```

### route

生成命名路由 URL：

```php
$url = route('user.profile', ['id' => 1]);
```

## 响应函数

### response

创建响应：

```php
return response('Hello World');
return response()->json(['status' => 'ok']);
return response()->view('index', ['title' => 'Home']);
```

### json

返回 JSON 响应：

```php
return json(['status' => 'ok', 'data' => $data]);
```

### redirect

重定向：

```php
return redirect('/home');
return redirect()->route('user.profile', ['id' => 1]);
return redirect()->back();
```

### abort

中断请求：

```php
abort(404);
abort(403, 'Unauthorized');
abort(500, 'Server Error');
```

## 调试函数

### dd

打印并终止：

```php
dd($variable);
dd($var1, $var2, $var3);
```

### dump

打印变量：

```php
dump($variable);
```

### info

记录信息日志：

```php
info('User logged in', ['user_id' => 1]);
```

### logger

记录日志：

```php
logger('Debug message');
logger()->error('Error message');
```

## 其他函数

### env

获取环境变量：

```php
$debug = env('APP_DEBUG', false);
$dbHost = env('DB_HOST', 'localhost');
```

### config

获取配置：

```php
$appName = config('app.name');
$dbHost = config('database.connections.mysql.host', 'localhost');
```

### app

获取应用实例或服务：

```php
$app = app();
$db = app('db');
$cache = app('cache');
```

### value

获取值（支持闭包）：

```php
$value = value($variable);
$value = value(fn() => expensiveOperation());
```

### tap

链式操作：

```php
$user = tap(User::find(1), function ($user) {
    $user->update(['last_login' => now()]);
});
```

### throw_if / throw_unless

条件抛出异常：

```php
throw_if($condition, Exception::class, 'Message');
throw_unless($condition, Exception::class, 'Message');
```

### retry

重试操作：

```php
$result = retry(3, function () {
    return Http::get('https://api.example.com');
}, 100); // 重试 3 次，间隔 100ms
```

### now / today

获取当前时间：

```php
$now = now();
$today = today();
```

### bcrypt / password_verify

密码哈希：

```php
$hash = bcrypt('password');
$verified = password_verify('password', $hash);
```

### encrypt / decrypt

加密解密：

```php
$encrypted = encrypt($data);
$decrypted = decrypt($encrypted);
```

## 完整助手函数列表

| 函数 | 说明 |
|------|------|
| `arr_get` | 数组取值 |
| `arr_set` | 数组设值 |
| `arr_has` | 数组检查 |
| `arr_forget` | 数组删除 |
| `arr_only` | 数组筛选 |
| `arr_except` | 数组排除 |
| `arr_pluck` | 数组提取 |
| `arr_first` | 数组首个 |
| `arr_last` | 数组末个 |
| `str_random` | 随机字符串 |
| `str_slug` | URL slug |
| `str_limit` | 字符串截断 |
| `str_starts_with` | 开头检查 |
| `str_ends_with` | 结尾检查 |
| `str_contains` | 包含检查 |
| `str_camel` | 驼峰转换 |
| `str_snake` | 蛇形转换 |
| `str_kebab` | 短横转换 |
| `str_studly` | 大驼峰 |
| `base_path` | 根目录 |
| `app_path` | app 目录 |
| `config_path` | config 目录 |
| `runtime_path` | runtime 目录 |
| `public_path` | public 目录 |
| `url` | URL 生成 |
| `asset` | 资源 URL |
| `route` | 路由 URL |
| `response` | 响应创建 |
| `json` | JSON 响应 |
| `redirect` | 重定向 |
| `abort` | 中断请求 |
| `dd` | 打印终止 |
| `dump` | 打印变量 |
| `info` | 信息日志 |
| `logger` | 日志记录 |
| `env` | 环境变量 |
| `config` | 配置获取 |
| `app` | 应用实例 |
| `value` | 获取值 |
| `tap` | 链式操作 |
| `throw_if` | 条件抛出 |
| `throw_unless` | 条件抛出 |
| `retry` | 重试操作 |
| `now` | 当前时间 |
| `today` | 今天日期 |
| `bcrypt` | 密码哈希 |
| `encrypt` | 加密 |
| `decrypt` | 解密 |
