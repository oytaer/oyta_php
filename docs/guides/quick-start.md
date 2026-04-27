# 快速开始

## 安装

### 方式一：使用 CLI 工具（推荐）

```bash
# 安装 CLI 工具
cargo install oyta-cli

# 创建新项目
oyta-cli new myapp

# 进入项目目录
cd myapp

# 启动开发服务器
./oyta start
```

### 方式二：手动安装

1. 下载最新版本：

```bash
# Linux x86_64
wget https://github.com/oyta-php/oyta/releases/latest/download/oyta-linux-x86_64 -O oyta
chmod +x oyta

# macOS x86_64
wget https://github.com/oyta-php/oyta/releases/latest/download/oyta-macos-x86_64 -O oyta
chmod +x oyta

# macOS Apple Silicon
wget https://github.com/oyta-php/oyta/releases/latest/download/oyta-macos-aarch64 -O oyta
chmod +x oyta
```

2. 创建项目结构：

```bash
mkdir -p myapp/{app/controller,config,public,route,runtime}
cd myapp
```

3. 创建入口文件 `public/index.php`：

```php
<?php
// 引导应用
\oyta\App::run();
```

4. 创建配置文件 `config/app.php`：

```php
<?php
return [
    'name' => 'My App',
    'debug' => true,
    'timezone' => 'Asia/Shanghai',
];
```

5. 创建路由文件 `route/route.php`：

```php
<?php
use oyta\Route;

Route::get('/', function () {
    return 'Hello OYTAPHP!';
});
```

6. 启动服务器：

```bash
./oyta start
```

## 目录结构

```
myapp/
├── app/                    # 应用目录
│   ├── controller/         # 控制器
│   │   └── IndexController.php
│   ├── model/              # 模型
│   ├── middleware/         # 中间件
│   ├── service/            # 服务类
│   └── common.php          # 公共函数
│
├── config/                 # 配置目录
│   ├── app.php             # 应用配置
│   ├── database.php        # 数据库配置
│   ├── cache.php           # 缓存配置
│   └── route.php           # 路由配置
│
├── public/                 # 公共目录
│   ├── index.php           # 入口文件
│   └── static/             # 静态文件
│
├── route/                  # 路由定义
│   └── route.php
│
├── runtime/                # 运行时目录
│   ├── cache/              # 缓存文件
│   ├── log/                # 日志文件
│   └── session/            # Session 文件
│
├── view/                   # 视图目录
│
├── .env                    # 环境变量
├── composer.json           # Composer 配置
└── oyta                    # OYTAPHP 二进制
```

## 第一个控制器

### 创建控制器

`app/controller/IndexController.php`：

```php
<?php
namespace app\controller;

class IndexController
{
    public function index()
    {
        return 'Hello World!';
    }
    
    public function hello($name)
    {
        return "Hello, {$name}!";
    }
    
    public function json()
    {
        return json([
            'status' => 'ok',
            'message' => 'Hello JSON!',
        ]);
    }
}
```

### 定义路由

`route/route.php`：

```php
<?php
use oyta\Route;
use app\controller\IndexController;

// 闭包路由
Route::get('/', function () {
    return 'Welcome!';
});

// 控制器路由
Route::get('/hello/:name', [IndexController::class, 'hello']);

// 资源路由
Route::resource('/users', 'app\controller\UserController');

// 路由组
Route::group('/api', function () {
    Route::get('/users', [IndexController::class, 'json']);
    Route::post('/users', [IndexController::class, 'create']);
})->middleware(['auth']);
```

## 数据库操作

### 配置数据库

`config/database.php`：

```php
<?php
return [
    'default' => 'mysql',
    'connections' => [
        'mysql' => [
            'driver' => 'mysql',
            'host' => env('DB_HOST', 'localhost'),
            'port' => env('DB_PORT', 3306),
            'database' => env('DB_DATABASE', 'myapp'),
            'username' => env('DB_USERNAME', 'root'),
            'password' => env('DB_PASSWORD', ''),
            'charset' => 'utf8mb4',
        ],
    ],
];
```

### 使用查询构建器

```php
<?php
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
$id = Db::table('users')->insertGetId([
    'name' => 'John',
    'email' => 'john@example.com',
]);

// 更新
Db::table('users')
    ->where('id', 1)
    ->update(['name' => 'Jane']);

// 删除
Db::table('users')->where('id', 1)->delete();

// 事务
Db::transaction(function () {
    Db::table('users')->insert([...]);
    Db::table('profiles')->insert([...]);
});
```

## 缓存操作

### 配置缓存

`config/cache.php`：

```php
<?php
return [
    'default' => 'redis',
    'stores' => [
        'redis' => [
            'driver' => 'redis',
            'connection' => 'cache',
        ],
        'file' => [
            'driver' => 'file',
            'path' => runtime_path('cache'),
        ],
    ],
];
```

### 使用缓存

```php
<?php
use oyta\facade\Cache;

// 获取
$value = Cache::get('key');

// 设置
Cache::put('key', 'value', 3600);

// 获取或设置
$value = Cache::remember('users', 3600, function () {
    return Db::table('users')->select();
});
```

## 热重载

修改 PHP 代码后，无需重启服务器，修改会自动生效。

```bash
# 启动开发服务器（默认开启热重载）
./oyta start

# 禁用热重载
./oyta start --no-watch
```

## 生产部署

### 编译优化

```bash
# 优化路由缓存
./oyta optimize:route

# 优化配置缓存
./oyta optimize:config
```

### 启动生产服务器

```bash
# 启动生产服务器
./oyta start --env=production

# 指定端口和进程数
./oyta start --port=8080 --workers=4
```

### 使用 Systemd

创建服务文件 `/etc/systemd/system/oyta.service`：

```ini
[Unit]
Description=OYTAPHP Application
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/var/www/myapp
ExecStart=/var/www/myapp/oyta start --env=production
Restart=always

[Install]
WantedBy=multi-user.target
```

启动服务：

```bash
sudo systemctl enable oyta
sudo systemctl start oyta
```

## 下一步

- [命令行工具](./cli.md) - 了解更多 CLI 命令
- [配置说明](./configuration.md) - 详细配置选项
- [Facade 门面](../api/facade.md) - 使用 Facade 系统
- [助手函数](../api/helpers.md) - 使用助手函数
