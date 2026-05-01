# Interpreter 解释器模块

## 模块概述

Interpreter 模块是 OYTAPHP 的核心，实现完整的 PHP 8.5 解释器，包括 AST 执行引擎、值类型系统、内置函数和类、门面类系统、PHP 8 新特性支持等。

## 模块结构

```
interpreter/
├── mod.rs           # 模块入口
├── value.rs         # 值类型系统
├── enum_type.rs     # 枚举类型
├── fiber.rs         # Fiber 协程
├── generator.rs     # 生成器
├── php8_features/   # PHP 8 新特性
│   ├── union_type.rs      # 联合类型
│   ├── promoted_property.rs # 提升属性
│   ├── attribute.rs       # 属性
│   ├── nullsafe.rs        # 空安全操作符
│   ├── named_args.rs      # 命名参数
│   └── match_expr.rs      # Match 表达式
└── engine/          # 执行引擎
    ├── interpreter.rs     # 解释器核心
    ├── expr.rs            # 表达式执行
    ├── stmt.rs            # 语句执行
    ├── binary_op.rs       # 二元运算
    ├── helpers.rs         # 辅助函数
    ├── builtin_classes.rs # 内置类注册
    ├── builtin_facades.rs # 门面类注册
    ├── builtins/          # 基础内置函数
    │   ├── mod.rs
    │   ├── string.rs      # 字符串函数
    │   ├── array.rs       # 数组函数
    │   ├── encoding.rs    # 编码函数
    │   └── helpers.rs     # 辅助函数
    ├── builtins_ext/      # 扩展内置函数
    │   ├── mod.rs
    │   ├── type_check.rs  # 类型检测
    │   ├── type_cast.rs   # 类型转换
    │   ├── math.rs        # 数学函数
    │   ├── debug.rs       # 调试函数
    │   ├── string_ext.rs  # 字符串扩展
    │   ├── hash.rs        # 哈希函数
    │   ├── time.rs        # 时间函数
    │   ├── class_object.rs # 类/对象函数
    │   ├── validation.rs  # 验证函数
    │   ├── datetime.rs    # DateTime 函数
    │   ├── mbstring.rs    # 多字节字符串
    │   ├── gd.rs          # GD 图像
    │   ├── xml.rs         # XML 处理
    │   ├── phar.rs        # Phar 归档
    │   ├── eval.rs        # 动态执行
    │   └── helpers.rs     # OYTAPHP 助手
    ├── builtin_class/     # 内置类实现
    │   ├── mod.rs
    │   ├── types.rs       # 类型定义
    │   ├── datetime.rs    # DateTime 类
    │   ├── dom.rs         # DOM 类
    │   ├── phar.rs        # Phar 类
    │   ├── reflection.rs  # 反射类
    │   ├── request.rs     # Request 类
    │   ├── model.rs       # Model 基类
    │   └── gdimage.rs     # GdImage 类
    └── builtin_facade/    # 门面类实现
        ├── mod.rs
        ├── types.rs       # 类型定义
        ├── app.rs         # App 门面
        ├── cache.rs       # Cache 门面
        ├── session.rs     # Session 门面
        ├── config.rs      # Config 门面
        ├── env.rs         # Env 门面
        ├── validate.rs    # Validate 门面
        ├── log.rs         # Log 门面
        ├── event.rs       # Event 门面
        ├── request.rs     # Request 门面
        ├── response.rs    # Response 门面
        ├── crypt.rs       # Crypt 门面
        ├── coroutine.rs   # Coroutine 门面
        ├── timer.rs       # Timer 门面
        ├── search.rs      # Search 门面
        ├── websocket.rs   # WebSocket 门面
        ├── queue.rs       # Queue 门面
        ├── upload.rs      # Upload 门面
        ├── route.rs       # Route 门面
        ├── lang.rs        # Lang 门面
        ├── hash.rs        # Hash 门面
        ├── csrf.rs        # Csrf 门面
        ├── date.rs        # Date 门面
        ├── middleware.rs  # Middleware 门面
        ├── db.rs          # Db 门面
        ├── model.rs       # Model 门面
        ├── storage.rs     # Storage 门面
        ├── view.rs        # View 门面
        ├── composer.rs    # Composer 门面
        ├── monitor.rs     # Monitor 门面
        ├── container.rs   # Container 门面
        ├── cron.rs        # Cron 门面
        ├── lock.rs        # Lock 门面
        ├── serialize.rs   # Serialize 门面
        └── servicediscovery.rs # ServiceDiscovery 门面
```

## 值类型系统 (Value)

```rust
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    IndexedArray(Vec<Value>),
    AssociativeArray(HashMap<String, Value>),
    Object(ObjectInstance),
    Callable(CallableValue),
    Resource(Resource),
    Reference(Box<Value>),
}

pub struct ObjectInstance {
    pub class_name: String,
    pub properties: HashMap<String, Value>,
}

pub struct ExecutionContext {
    pub variables: HashMap<String, Value>,
    pub this: Option<ObjectInstance>,
    pub namespace: Option<String>,
    pub return_value: Option<Value>,
}
```

## 内置函数列表

### 字符串函数 (builtins/string.rs)

```php
<?php
strlen($string);
substr($string, $start, $length);
strpos($string, $search);
strrpos($string, $search);
stripos($string, $search);
strripos($string, $search);
str_replace($search, $replace, $subject);
str_ireplace($search, $replace, $subject);
strtolower($string);
strtoupper($string);
ucfirst($string);
lcfirst($string);
ucwords($string);
trim($string, $characters);
ltrim($string, $characters);
rtrim($string, $characters);
explode($delimiter, $string);
implode($glue, $array);
join($glue, $array);
str_split($string, $length);
chunk_split($string, $length, $end);
str_repeat($string, $times);
str_pad($string, $length, $pad_string, $pad_type);
strrev($string);
str_shuffle($string);
str_word_count($string);
substr_count($string, $needle);
substr_replace($string, $replacement, $start, $length);
strtok($string, $token);
strstr($string, $needle);
stristr($string, $needle);
strchr($string, $needle);
strrchr($string, $needle);
```

### 数组函数 (builtins/array.rs)

```php
<?php
count($array);
sizeof($array);
array_push($array, $value);
array_pop($array);
array_shift($array);
array_unshift($array, $value);
array_merge($array1, $array2);
array_merge_recursive($array1, $array2);
array_combine($keys, $values);
array_keys($array);
array_values($array);
array_flip($array);
array_reverse($array);
array_search($needle, $array);
in_array($needle, $array);
array_key_exists($key, $array);
key_exists($key, $array);
array_unique($array);
array_intersect($array1, $array2);
array_diff($array1, $array2);
array_map($callback, $array);
array_filter($array, $callback);
array_reduce($array, $callback, $initial);
array_walk($array, $callback);
array_walk_recursive($array, $callback);
array_slice($array, $offset, $length);
array_splice($array, $offset, $length, $replacement);
array_fill($start, $count, $value);
array_fill_keys($keys, $value);
array_pad($array, $size, $value);
array_column($array, $column);
array_multisort(...$arrays);
array_rand($array, $count);
array_sum($array);
array_product($array);
array_chunk($array, $size);
```

### 编码函数 (builtins/encoding.rs)

```php
<?php
json_encode($value, $flags, $depth);
json_decode($json, $assoc, $depth, $flags);
base64_encode($data);
base64_decode($data);
urlencode($string);
urldecode($string);
rawurlencode($string);
rawurldecode($string);
htmlentities($string, $flags, $encoding);
html_entity_decode($string, $flags, $encoding);
htmlspecialchars($string, $flags, $encoding);
htmlspecialchars_decode($string, $flags);
```

### 数学函数 (builtins_ext/math.rs)

```php
<?php
abs($number);
max(...$values);
min(...$values);
floor($number);
ceil($number);
round($number, $precision);
sqrt($number);
pow($base, $exp);
exp($number);
log($number, $base);
log10($number);
sin($number);
cos($number);
tan($number);
asin($number);
acos($number);
atan($number);
atan2($y, $x);
deg2rad($number);
rad2deg($number);
rand($min, $max);
mt_rand($min, $max);
srand($seed);
mt_srand($seed);
getrandmax();
mt_getrandmax();
pi();
is_finite($number);
is_infinite($number);
is_nan($number);
```

### 时间函数 (builtins_ext/time.rs)

```php
<?php
time();
date($format, $timestamp);
strtotime($time, $now);
mktime($hour, $minute, $second, $month, $day, $year);
gmmktime($hour, $minute, $second, $month, $day, $year);
checkdate($month, $day, $year);
date_default_timezone_get();
date_default_timezone_set($timezone);
microtime($as_float);
getdate($timestamp);
gettimeofday($as_float);
idate($format, $timestamp);
```

### 类型检测函数 (builtins_ext/type_check.rs)

```php
<?php
is_null($value);
is_array($value);
is_string($value);
is_int($value);
is_integer($value);
is_long($value);
is_float($value);
is_double($value);
is_real($value);
is_bool($value);
is_object($value);
is_resource($value);
is_callable($value);
is_numeric($value);
is_scalar($value);
is_iterable($value);
is_countable($value);
is_empty($value);
isset($value);
gettype($value);
get_debug_type($value);
```

### 类型转换函数 (builtins_ext/type_cast.rs)

```php
<?php
intval($value, $base);
floatval($value);
doubleval($value);
strval($value);
boolval($value);
settype($value, $type);
```

### 调试函数 (builtins_ext/debug.rs)

```php
<?php
var_dump(...$values);
print_r($expression, $return);
debug_zval_dump($value);
debug_print_backtrace($options, $limit);
debug_backtrace($options, $limit);
json($data);
dd(...$values);
dump(...$values);
```

### 验证函数 (builtins_ext/validation.rs)

```php
<?php
filter_var($value, $filter, $options);
filter_input($type, $variable, $filter, $options);
validate_email($email);
validate_url($url);
validate_ip($ip, $flags);
validate_regex($value, $pattern);
validate_length($value, $min, $max);
validate_range($value, $min, $max);
validate_required($value);
```

### 类/对象函数 (builtins_ext/class_object.rs)

```php
<?php
class_exists($class, $autoload);
interface_exists($interface, $autoload);
trait_exists($trait, $autoload);
method_exists($object, $method);
property_exists($class, $property);
get_class($object);
get_parent_class($object);
get_class_methods($class);
get_class_vars($class);
get_object_vars($object);
get_declared_classes();
get_declared_interfaces();
get_declared_traits();
call_user_func($callback, ...$args);
call_user_func_array($callback, $args);
func_num_args();
func_get_arg($index);
func_get_args();
```

### 多字节字符串函数 (builtins_ext/mbstring.rs)

```php
<?php
mb_strlen($string, $encoding);
mb_strpos($string, $needle, $offset, $encoding);
mb_strrpos($string, $needle, $offset, $encoding);
mb_stripos($string, $needle, $offset, $encoding);
mb_strripos($string, $needle, $offset, $encoding);
mb_strstr($string, $needle, $before, $encoding);
mb_stristr($string, $needle, $before, $encoding);
mb_substr($string, $start, $length, $encoding);
mb_strcut($string, $start, $length, $encoding);
mb_strtolower($string, $encoding);
mb_strtoupper($string, $encoding);
mb_convert_case($string, $mode, $encoding);
mb_convert_encoding($string, $to, $from);
mb_detect_encoding($string, $encodings, $strict);
mb_internal_encoding($encoding);
mb_split($pattern, $string, $limit);
mb_ereg($pattern, $string, $matches);
mb_eregi($pattern, $string, $matches);
mb_ereg_replace($pattern, $replacement, $string, $options);
mb_eregi_replace($pattern, $replacement, $string, $options);
```

## 内置类列表

### DateTime 类 (builtin_class/datetime.rs)

```php
<?php
// DateTime
$date = new DateTime($time, $timezone);
$date->format($format);
$date->modify($modify);
$date->add($interval);
$date->sub($interval);
$date->diff($date, $absolute);
$date->setDate($year, $month, $day);
$date->setTime($hour, $minute, $second);
$date->setTimezone($timezone);
$date->getTimestamp();
$date->setTimestamp($timestamp);
$date->getTimezone();

// DateTimeImmutable
$date = new DateTimeImmutable($time, $timezone);
$newDate = $date->modify($modify);

// DateTimeZone
$tz = new DateTimeZone($timezone);
$tz->getName();
$tz->getLocation();
$tz->getTransitions();
$tz->listIdentifiers();

// DateInterval
$interval = new DateInterval($spec);
$interval->format($format);
DateInterval::createFromDateString($time);

// DatePeriod
$period = new DatePeriod($start, $interval, $end);
foreach ($period as $date) {}
```

### DOM 类 (builtin_class/dom.rs)

```php
<?php
// DOMDocument
$doc = new DOMDocument();
$doc->load($filename);
$doc->loadXML($source);
$doc->save($filename);
$doc->saveXML();
$doc->createElement($name, $value);
$doc->createTextNode($content);
$doc->getElementById($id);
$doc->getElementsByTagName($name);

// SimpleXMLElement
$xml = simplexml_load_file($filename);
$xml = simplexml_load_string($data);
$xml->xpath($path);
$xml->asXML();
$xml->addChild($name, $value);
$xml->addAttribute($name, $value);
```

### Phar 类 (builtin_class/phar.rs)

```php
<?php
$phar = new Phar($filename);
$phar->buildFromDirectory($dir, $pattern);
$phar->buildFromIterator($iterator);
$phar->addFile($file, $localName);
$phar->addFromString($localName, $contents);
$phar->extractTo($dir, $files, $overwrite);
$phar->setStub($stub);
$phar->getStub();
$phar->setSignatureAlgorithm($algorithm, $privateKey);
$phar->getSignature();
$phar->startBuffering();
$phar->stopBuffering();
$phar->compress($compression);
$phar->decompress();
$phar->convertToExecutable($format, $compression);
```

### Reflection 类 (builtin_class/reflection.rs)

```php
<?php
// ReflectionClass
$ref = new ReflectionClass($class);
$ref->getName();
$ref->getNamespaceName();
$ref->getShortName();
$ref->isAbstract();
$ref->isFinal();
$ref->isInterface();
$ref->isTrait();
$ref->getParentClass();
$ref->getMethods($filter);
$ref->getMethod($name);
$ref->getProperties($filter);
$ref->getProperty($name);
$ref->getConstants();
$ref->getConstant($name);
$ref->newInstance($args);
$ref->newInstanceWithoutConstructor();

// ReflectionMethod
$method = new ReflectionMethod($class, $name);
$method->getName();
$method->isPublic();
$method->isPrivate();
$method->isProtected();
$method->isStatic();
$method->isAbstract();
$method->isFinal();
$method->getParameters();
$method->getReturnType();
$method->invoke($object, $args);
$method->invokeArgs($object, $args);

// ReflectionProperty
$prop = new ReflectionProperty($class, $name);
$prop->getName();
$prop->getValue($object);
$prop->setValue($object, $value);
$prop->isPublic();
$prop->isPrivate();
$prop->isProtected();
$prop->isStatic();

// ReflectionParameter
$param = new ReflectionParameter($function, $param);
$param->getName();
$param->getType();
$param->isOptional();
$param->isDefaultValueAvailable();
$param->getDefaultValue();

// ReflectionFunction
$func = new ReflectionFunction($name);
$func->getName();
$func->getParameters();
$func->getReturnType();
$func->invoke($args);
$func->invokeArgs($args);
```

### GdImage 类 (builtin_class/gdimage.rs)

```php
<?php
$image = new GdImage();
$image->new($width, $height);
$image->load($filename);
$image->save($filename, $quality);
$image->resize($width, $height);
$image->crop($x, $y, $width, $height);
$image->rotate($angle);
$image->flip($mode);
$image->filter($filter, $args);
$image->text($text, $x, $y, $options);
$image->line($x1, $y1, $x2, $y2, $color);
$image->rectangle($x1, $y1, $x2, $y2, $color);
$image->ellipse($cx, $cy, $width, $height, $color);
$image->getWidth();
$image->getHeight();
$image->getType();
$image->destroy();
$image->output($type);
$image->toBase64($type);
```

## 门面类列表

### App 门面

```php
<?php
App::version();
App::environment();
App::isProduction();
App::isDevelopment();
App::getLocale();
App::setLocale($locale);
App::getNamespace();
App::path();
App::basePath();
App::configPath();
App::runtimePath();
```

### Cache 门面

```php
<?php
Cache::get($key, $default);
Cache::set($key, $value, $ttl);
Cache::has($key);
Cache::delete($key);
Cache::clear();
Cache::increment($key, $step);
Cache::decrement($key, $step);
Cache::remember($key, $ttl, $default);
Cache::store($name);
```

### Session 门面

```php
<?php
Session::get($key, $default);
Session::set($key, $value);
Session::has($key);
Session::delete($key);
Session::clear();
Session::getId();
Session::regenerate();
Session::flash($key, $value);
Session::getFlash($key);
```

### Config 门面

```php
<?php
Config::get($key, $default);
Config::set($key, $value);
Config::has($key);
Config::all();
```

### Env 门面

```php
<?php
Env::get($key, $default);
Env::has($key);
```

### Log 门面

```php
<?php
Log::emergency($message, $context);
Log::alert($message, $context);
Log::critical($message, $context);
Log::error($message, $context);
Log::warning($message, $context);
Log::notice($message, $context);
Log::info($message, $context);
Log::debug($message, $context);
Log::channel($channel);
```

### Event 门面

```php
<?php
Event::listen($event, $listener);
Event::dispatch($event, $payload);
Event::hasListeners($event);
Event::forget($event);
Event::subscribe($subscriber);
```

### Request 门面

```php
<?php
Request::param($name, $default);
Request::get($name, $default);
Request::post($name, $default);
Request::input($name, $default);
Request::file($name);
Request::header($name, $default);
Request::method();
Request::path();
Request::url();
Request::ip();
Request::isMethod($method);
Request::isAjax();
Request::isJson();
```

### Response 门面

```php
<?php
Response::make($content, $status);
Response::json($data, $status);
Response::html($content, $status);
Response::text($content, $status);
Response::redirect($url, $status);
Response::download($file, $name);
Response::file($file);
```

### Db 门面

```php
<?php
Db::table($table);
Db::name($name);
Db::query($sql, $bindings);
Db::execute($sql, $bindings);
Db::transaction($callback);
Db::startTrans();
Db::commit();
Db::rollback();
Db::getLastSql();
```

### Model 门面

```php
<?php
Model::find($id);
Model::findOrFail($id);
Model::first();
Model::all();
Model::create($data);
Model::update($data);
Model::delete($id);
Model::where($column, $operator, $value);
Model::orderBy($column, $direction);
Model::limit($limit);
Model::offset($offset);
Model::paginate($perPage);
```

### View 门面

```php
<?php
View::fetch($template, $data);
View::display($content, $data);
View::assign($name, $value);
View::get($name);
```

### Storage 门面

```php
<?php
Storage::disk($name);
Storage::put($path, $content);
Storage::get($path);
Storage::exists($path);
Storage::delete($path);
Storage::url($path);
Storage::path($path);
Storage::files($directory);
Storage::allFiles($directory);
Storage::directories($directory);
Storage::makeDirectory($path);
Storage::deleteDirectory($directory);
```

### Crypt 门面

```php
<?php
Crypt::encrypt($value);
Crypt::decrypt($value);
Crypt::encryptString($value);
Crypt::decryptString($value);
Crypt::generateKey();
```

### Hash 门面

```php
<?php
Hash::make($value);
Hash::check($value, $hash);
Hash::needsRehash($hash);
```

### Coroutine 门面

```php
<?php
Coroutine::create($callback);
Coroutine::id();
Coroutine::yield();
Coroutine::resume($id);
Coroutine::sleep($ms);
Coroutine::parallel($tasks);
```

### Timer 门面

```php
<?php
Timer::after($ms, $callback);
Timer::tick($ms, $callback);
Timer::clear($id);
Timer::defer($callback);
```

### Queue 门面

```php
<?php
Queue::push($job, $data, $queue);
Queue::later($delay, $job, $data, $queue);
Queue::pop($queue);
Queue::size($queue);
Queue::clear($queue);
```

### Cron 门面

```php
<?php
Cron::add($expression, $callback);
Cron::remove($id);
Cron::list();
Cron::run();
Cron::enable($id);
Cron::disable($id);
Cron::exists($id);
Cron::nextRun($id);
Cron::lastRun($id);
```

### Lock 门面

```php
<?php
Lock::acquire($key, $timeout);
Lock::tryAcquire($key, $timeout);
Lock::release($lock);
Lock::forceRelease($key);
Lock::isLocked($key);
Lock::guard($key, $timeout, $callback);
```

### ServiceDiscovery 门面

```php
<?php
ServiceDiscovery::register($name, $host, $port);
ServiceDiscovery::deregister($id);
ServiceDiscovery::discover($name);
ServiceDiscovery::health();
ServiceDiscovery::list();
ServiceDiscovery::heartbeat($id);
ServiceDiscovery::getOne($name);
```

### Container 门面

```php
<?php
Container::bind($abstract, $concrete);
Container::singleton($abstract, $concrete);
Container::instance($abstract, $instance);
Container::make($abstract);
Container::has($abstract);
Container::forget($abstract);
Container::flush();
Container::bindings();
```

### Monitor 门面

```php
<?php
Monitor::start();
Monitor::stop();
Monitor::metrics();
Monitor::dashboard();
Monitor::recordRequest($time, $memory);
Monitor::recordQuery($sql, $time);
Monitor::recordCache($operation, $hit);
Monitor::getMemoryUsage();
Monitor::getPeakMemory();
Monitor::getRequestCount();
Monitor::getAverageResponseTime();
```

### Serialize 门面

```php
<?php
Serialize::encode($data, $format);
Serialize::decode($data, $format);
Serialize::encodeBincode($data);
Serialize::decodeBincode($data);
Serialize::encodeMsgPack($data);
Serialize::decodeMsgPack($data);
Serialize::encodeCbor($data);
Serialize::decodeCbor($data);
Serialize::encodePostcard($data);
Serialize::decodePostcard($data);
Serialize::getSize($data, $format);
```

### Composer 门面

```php
<?php
Composer::install();
Composer::update();
Composer::require($package, $version);
Composer::remove($package);
Composer::dumpAutoload();
Composer::search($query);
Composer::show();
Composer::validate();
Composer::outdated();
```

## PHP 8 新特性支持

### 联合类型

```php
<?php
function process(int|float $number): int|float
{
    return $number * 2;
}

class User
{
    public int|string $id;
}
```

### 提升属性

```php
<?php
class User
{
    public function __construct(
        public string $name,
        public int $age,
        protected string $email,
    ) {}
}
```

### 属性（Attribute）

```php
<?php
#[Attribute]
class Route
{
    public function __construct(public string $path) {}
}

#[Route('/api/user')]
class UserController
{
    #[HttpGet('/:id')]
    public function read(int $id) {}
}
```

### 空安全操作符

```php
<?php
$name = $user?->profile?->name;
$country = $user?->address?->country ?? 'Unknown';
```

### 命名参数

```php
<?php
$result = function_call(
    param1: 'value1',
    param2: 'value2',
);

$user = new User(
    name: 'John',
    age: 30,
);
```

### Match 表达式

```php
<?php
$status = match($code) {
    200 => 'OK',
    404 => 'Not Found',
    500 => 'Server Error',
    default => 'Unknown',
};
```

## 继承链方法查找

解释器支持完整的方法继承链查找：

```php
<?php
class BaseController
{
    protected function jsonResponse($data)
    {
        return json_encode($data);
    }
}

class UserController extends BaseController
{
    public function index()
    {
        return $this->jsonResponse(['users' => []]);
    }
}
```

解释器会自动：
1. 在当前类查找方法
2. 如果未找到，递归查找父类
3. 收集所有继承的属性
4. 正确设置 `$this` 上下文
