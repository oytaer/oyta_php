# Command 命令行工具模块

## 模块概述

Command 模块提供完整的命令行工具，包括项目管理、代码生成、数据库迁移、缓存管理、队列管理等。对应 ThinkPHP 8.0 的命令行系统。

## 模块结构

```
command/
├── mod.rs       # 命令分发入口
├── init.rs      # 项目初始化命令
├── make.rs      # 代码生成命令
├── args.rs      # 命令参数定义
└── info.rs      # 信息查看命令
```

## 可用命令

### 服务器命令

```bash
# 启动开发服务器
oyta run

# 指定端口
oyta run --port=8080

# 指定主机
oyta run --host=0.0.0.0

# 指定 Worker 数量
oyta run --workers=4

# 生产模式
oyta run --env=production

# 开启热重载
oyta run --watch
```

### 项目管理命令

```bash
# 创建新项目
oyta new myproject

# 在当前目录初始化项目
oyta init

# 创建多应用模块
oyta build admin
oyta build api

# 安装到系统
oyta install

# 卸载
oyta uninstall
```

### 代码生成命令

```bash
# 创建控制器
oyta make:controller UserController
oyta make:controller Admin/UserController  # 带命名空间

# 创建模型
oyta make:model User

# 创建中间件
oyta make:middleware Auth

# 创建验证器
oyta make:validate UserValidate

# 创建事件
oyta make:event UserRegistered

# 创建监听器
oyta make:listener SendWelcomeEmail

# 创建订阅者
oyta make:subscribe UserEventSubscriber

# 创建服务类
oyta make:service PaymentService

# 创建自定义命令
oyta make:command CustomCommand

# 创建队列任务
oyta make:job ProcessPodcast

# 创建定时任务
oyta make:task DailyCleanup
```

### 路由命令

```bash
# 查看所有路由
oyta route:list

# 缓存路由
oyta route:cache

# 清除路由缓存
oyta route:clear
```

### 缓存命令

```bash
# 清除所有缓存
oyta cache:clear

# 清除指定缓存
oyta cache:forget user_list

# 查看缓存状态
oyta cache:status
```

### 队列命令

```bash
# 查看失败任务
oyta queue:failed

# 重试失败任务
oyta queue:retry 1
oyta queue:retry all

# 清除失败任务
oyta queue:flush

# 查看队列状态
oyta queue:status
```

### 数据库迁移命令

```bash
# 创建迁移
oyta migrate:create create_users_table

# 执行迁移
oyta migrate:run

# 回滚迁移
oyta migrate:rollback
oyta migrate:rollback --step=3

# 重置迁移
oyta migrate:reset

# 刷新迁移（重置后重新运行）
oyta migrate:refresh

# 完全重建（删除所有表后重新迁移）
oyta migrate:fresh

# 查看迁移状态
oyta migrate:status

# 数据填充
oyta db:seed
oyta db:seed --class=UserSeeder

# 查看表结构
oyta db:table users
```

### Composer 命令

```bash
# 安装依赖
oyta composer:install

# 更新依赖
oyta composer:update

# 添加依赖
oyta composer:require vendor/package

# 移除依赖
oyta composer:remove vendor/package

# 重新生成自动加载
oyta composer:dump-autoload

# 检查过期依赖
oyta composer:outdated
```

### 信息命令

```bash
# 查看版本
oyta version

# 查看命令列表
oyta list
```

## 生成的控制器模板

```php
<?php
namespace app\controller;

/**
 * UserController 控制器
 */
class UserController
{
    /**
     * 默认方法
     */
    public function index()
    {
        return 'Hello, UserController!';
    }
}
```

## 生成的模型模板

```php
<?php
namespace app\model;

use think\Model;

/**
 * User 模型
 */
class User extends Model
{
    // 表名
    protected $name = 'user';
    
    // 主键
    protected $pk = 'id';
    
    // 自动时间戳
    protected $autoWriteTimestamp = true;
}
```

## 生成的中间件模板

```php
<?php
namespace app\middleware;

/**
 * Auth 中间件
 */
class Auth
{
    /**
     * 处理请求
     */
    public function handle($request, \Closure $next)
    {
        // 前置中间件
        
        $response = $next($request);
        
        // 后置中间件
        
        return $response;
    }
}
```

## 项目结构

```
myproject/
├── app/                    # 应用目录
│   ├── controller/         # 控制器
│   ├── model/              # 模型
│   ├── middleware/         # 中间件
│   ├── validate/           # 验证器
│   ├── event/              # 事件
│   ├── listener/           # 监听器
│   ├── subscribe/          # 订阅者
│   ├── service/            # 服务类
│   ├── command/            # 自定义命令
│   ├── job/                # 队列任务
│   └── task/               # 定时任务
├── config/                 # 配置文件
│   ├── app.php
│   ├── database.php
│   ├── cache.php
│   ├── session.php
│   └── ...
├── public/                 # 公共目录
│   ├── index.php           # 入口文件
│   └── .htaccess
├── runtime/                # 运行时目录
│   ├── cache/              # 缓存
│   ├── log/                # 日志
│   ├── session/            # 会话
│   └── view/               # 视图缓存
├── route/                  # 路由定义
│   └── app.php
├── view/                   # 视图模板
├── .env                    # 环境变量
├── .env.example            # 环境变量示例
└── composer.json           # Composer 配置
```

## 配置文件生成

### app.php

```php
<?php
return [
    'name' => 'OYTAPHP',
    'debug' => env('APP_DEBUG', true),
    'timezone' => 'Asia/Shanghai',
    'charset' => 'UTF-8',
];
```

### database.php

```php
<?php
return [
    'default' => 'mysql',
    'connections' => [
        'mysql' => [
            'type' => 'mysql',
            'host' => env('DB_HOST', '127.0.0.1'),
            'port' => env('DB_PORT', 3306),
            'database' => env('DB_NAME', ''),
            'username' => env('DB_USER', 'root'),
            'password' => env('DB_PASS', ''),
        ],
    ],
];
```

### cache.php

```php
<?php
return [
    'default' => 'file',
    'stores' => [
        'file' => [
            'type' => 'file',
            'path' => runtime_path('cache'),
        ],
    ],
];
```
