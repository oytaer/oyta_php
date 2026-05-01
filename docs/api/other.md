# 其他模块文档

## Helpers 辅助函数模块

```php
<?php
// 调试输出
dump($var);
dd($var); // dump and die

// 追踪
trace('调试信息');

// 数组操作
$data = array_wrap($value);
$first = array_first($array, $callback);
$last = array_last($array, $callback);

// 字符串操作
$slug = str_slug('Hello World'); // hello-world
$random = str_random(32); // 32位随机字符串
$limit = str_limit($string, 100, '...');

// URL 操作
$url = url('path/to/page');
$asset = asset('css/style.css');

// 路径操作
$path = base_path('config/app.php');
$path = app_path('model/User.php');
$path = config_path('app.php');
$path = runtime_path('log/app.log');
$path = public_path('uploads');

// 环境判断
if (app()->environment('production')) {
    // 生产环境
}

// 重试
retry(3, function() {
    // 尝试3次
}, 1000); // 间隔1秒
```

## Env Loader 环境变量加载模块

```php
<?php
// .env 文件
APP_NAME=OYTAPHP
APP_DEBUG=true
DB_HOST=127.0.0.1
DB_PORT=3306

// 使用
$env = env('APP_NAME', 'default');
$debug = env('APP_DEBUG', false);
```

## Output 输出缓冲模块

```php
<?php
// 开启缓冲
ob_start();

// 获取缓冲内容
$content = ob_get_contents();

// 清空缓冲
ob_clean();

// 输出并清空
ob_end_flush();

// 清空并关闭
ob_end_clean();

// 获取级别
$level = ob_get_level();

// 回调处理
ob_start(function($buffer) {
    return str_replace('old', 'new', $buffer);
});
```

## Search 全文搜索模块

```php
<?php
// 创建索引
Search::index('articles')->add([
    'id' => 1,
    'title' => '文章标题',
    'content' => '文章内容',
]);

// 搜索
$results = Search::index('articles')
    ->query('关键词')
    ->fields(['title', 'content'])
    ->limit(10)
    ->search();

// 删除文档
Search::index('articles')->delete(1);

// 批量操作
Search::index('articles')->bulk([
    ['index' => ['_id' => 1]],
    ['title' => '标题1', 'content' => '内容1'],
    ['index' => ['_id' => 2]],
    ['title' => '标题2', 'content' => '内容2'],
]);
```

## Watcher 文件监听模块

```php
<?php
// 监听文件变化
Watcher::watch('/path/to/dir', function($event, $path) {
    echo "$event: $path\n";
});

// 监听特定文件类型
Watcher::watch('/path/to/dir', function($event, $path) {
    // 处理变化
}, ['*.php', '*.json']);

// 热重载
php oyta run --watch
```

## Debug 调试工具模块

```php
<?php
// 设置断点
Debug::breakpoint();

// 查看变量
Debug::variable($var);

// 查看调用栈
Debug::backtrace();

// 性能分析
Debug::profileStart('operation');
// ... 操作
$time = Debug::profileEnd('operation');

// 内存使用
$memory = Debug::memoryUsage();
$peak = Debug::memoryPeak();
```

## Monitor 性能监控模块

```php
<?php
// 开始监控
Monitor::start();

// 记录指标
Monitor::recordRequest($requestTime, $memoryUsage);
Monitor::recordQuery($sql, $executionTime);
Monitor::recordCache($operation, $hit);

// 获取指标
$metrics = Monitor::metrics();
$dashboard = Monitor::dashboard();

// 获取内存使用
$memory = Monitor::getMemoryUsage();
$peak = Monitor::getPeakMemory();

// 获取请求统计
$count = Monitor::getRequestCount();
$avgTime = Monitor::getAverageResponseTime();
```

## Reflection 反射系统模块

```php
<?php
// 类反射
$reflection = new ReflectionClass(User::class);
$methods = $reflection->getMethods();
$properties = $reflection->getProperties();

// 方法反射
$method = new ReflectionMethod(User::class, 'getName');
$parameters = $method->getParameters();

// 函数反射
$function = new ReflectionFunction('strlen');

// 属性反射
$property = new ReflectionProperty(User::class, 'name');
$property->setAccessible(true);
$value = $property->getValue($user);
```

## Composer 依赖管理模块

```php
<?php
// 安装依赖
Composer::install();

// 更新依赖
Composer::update();

// 添加依赖
Composer::require('vendor/package', '^1.0');

// 移除依赖
Composer::remove('vendor/package');

// 重新生成自动加载
Composer::dumpAutoload();

// 搜索包
$results = Composer::search('query');

// 查看已安装
$packages = Composer::show();

// 验证 composer.json
Composer::validate();

// 检查过期依赖
$outdated = Composer::outdated();
```

## Service 服务模块

```php
<?php
// 注册服务
Service::register('mailer', function() {
    return new Mailer(config('mail'));
});

// 获取服务
$mailer = Service::get('mailer');

// 检查服务
if (Service::has('mailer')) {
    // ...
}

// 移除服务
Service::forget('mailer');
```

## Error 错误处理模块

```php
<?php
// 设置错误处理器
Error::handler(function($errno, $errstr, $errfile, $errline) {
    Log::error("$errstr in $errfile:$errline");
});

// 设置异常处理器
Error::exceptionHandler(function($exception) {
    Log::error($exception->getMessage());
    return response('Server Error', 500);
});

// 抛出异常
throw new \Exception('错误信息');

// 自定义异常
class ValidationException extends \Exception {}
```
