# 命令行工具

## 概述

OYTAPHP 提供了丰富的命令行工具，用于项目管理和开发。

## 基本用法

```bash
./oyta [command] [options]
```

## 服务命令

### start

启动 HTTP 服务器：

```bash
# 启动开发服务器
./oyta start

# 指定端口
./oyta start --port=8080

# 指定监听地址
./oyta start --host=0.0.0.0

# 指定工作进程数
./oyta start --workers=4

# 指定环境
./oyta start --env=production

# 禁用热重载
./oyta start --no-watch

# 启用调试模式
./oyta start --debug
```

### stop

停止服务器：

```bash
./oyta stop
```

### restart

重启服务器：

```bash
./oyta restart
```

### status

查看服务器状态：

```bash
./oyta status
```

## 优化命令

### optimize:route

优化路由缓存：

```bash
./oyta optimize:route
```

将路由编译为二进制缓存，加速路由匹配。

### optimize:config

优化配置缓存：

```bash
./oyta optimize:config
```

将配置序列化为二进制缓存，加速配置加载。

### optimize:clear

清除所有优化缓存：

```bash
./oyta optimize:clear
```

## 数据库命令

### db:migrate

执行数据库迁移：

```bash
# 执行所有迁移
./oyta db:migrate

# 指定数据库连接
./oyta db:migrate --connection=mysql
```

### db:seed

执行数据填充：

```bash
./oyta db:seed

# 指定填充类
./oyta db:seed --class=UserSeeder
```

### db:create

创建数据库：

```bash
./oyta db:create
```

## 缓存命令

### cache:clear

清除缓存：

```bash
# 清除所有缓存
./oyta cache:clear

# 清除指定标签的缓存
./oyta cache:clear --tags=user,profile
```

### cache:warmup

预热缓存：

```bash
./oyta cache:warmup
```

## 队列命令

### queue:work

启动队列工作进程：

```bash
# 启动默认队列
./oyta queue:work

# 指定队列
./oyta queue:work --queue=high,default

# 指定进程数
./oyta queue:work --workers=4

# 指定最大任务数
./oyta queue:work --max-jobs=1000

# 指定最大内存（MB）
./oyta queue:work --memory=128
```

### queue:restart

重启队列工作进程：

```bash
./oyta queue:restart
```

### queue:retry

重试失败的任务：

```bash
# 重试所有失败任务
./oyta queue:retry

# 重试指定任务
./oyta queue:retry --id=1
```

## 定时任务命令

### schedule:run

运行定时任务：

```bash
./oyta schedule:run
```

通常配合 cron 使用：

```cron
* * * * * cd /path/to/project && ./oyta schedule:run >> /dev/null 2>&1
```

### schedule:list

列出所有定时任务：

```bash
./oyta schedule:list
```

## Composer 命令

### composer:install

安装 Composer 依赖：

```bash
./oyta composer:install

# 指定 PHP 版本
./oyta composer:install --php=8.1
```

### composer:update

更新 Composer 依赖：

```bash
./oyta composer:update
```

### composer:require

安装指定包：

```bash
./oyta composer:require monolog/monolog
```

## 监控命令

### monitor:start

启动监控面板：

```bash
./oyta monitor:start --port=9090
```

访问 `http://localhost:9090` 查看监控面板。

### monitor:stats

查看运行统计：

```bash
./oyta monitor:stats
```

## 开发命令

### make:controller

创建控制器：

```bash
./oyta make:controller UserController

# 指定命名空间
./oyta make:controller admin/UserController

# 创建资源控制器
./oyta make:controller UserController --resource
```

### make:model

创建模型：

```bash
./oyta make:model User

# 同时创建迁移
./oyta make:model User --migration
```

### make:middleware

创建中间件：

```bash
./oyta make:middleware AuthMiddleware
```

### make:command

创建自定义命令：

```bash
./oyta make:command SendEmailCommand
```

### make:migration

创建迁移文件：

```bash
./oyta make:migration create_users_table
```

### make:seeder

创建数据填充：

```bash
./oyta make:seeder UserSeeder
```

## 测试命令

### test

运行测试：

```bash
# 运行所有测试
./oyta test

# 运行指定测试
./oyta test --filter=UserTest

# 显示详细输出
./oyta test --verbose
```

## 信息命令

### version

查看版本信息：

```bash
./oyta version
```

### info

查看系统信息：

```bash
./oyta info
```

显示 PHP 版本、扩展、内存限制等信息。

### route:list

列出所有路由：

```bash
./oyta route:list

# 过滤路由
./oyta route:list --path=api

# 显示中间件
./oyta route:list --with-middleware
```

## 命令选项

### 全局选项

| 选项 | 说明 |
|------|------|
| `-h, --help` | 显示帮助信息 |
| `-V, --version` | 显示版本 |
| `-v, --verbose` | 详细输出 |
| `-q, --quiet` | 静默模式 |
| `--no-ansi` | 禁用 ANSI 颜色 |
| `--env=ENV` | 指定环境 |

## 自定义命令

### 创建自定义命令

1. 创建命令类 `app/command/SendEmailCommand.php`：

```php
<?php
namespace app\command;

use oyta\Command;

class SendEmailCommand extends Command
{
    protected $signature = 'email:send {user} {--queue}';
    
    protected $description = '发送邮件给用户';
    
    public function handle()
    {
        $user = $this->argument('user');
        $queue = $this->option('queue');
        
        $this->info("Sending email to {$user}...");
        
        if ($queue) {
            $this->info('Using queue');
        }
    }
}
```

2. 注册命令 `config/console.php`：

```php
<?php
return [
    'commands' => [
        \app\command\SendEmailCommand::class,
    ],
];
```

3. 运行命令：

```bash
./oyta email:send john@example.com --queue
```

### 命令参数和选项

```php
// 参数
protected $signature = 'command:name {arg1} {arg2?} {arg3=default}';

// 选项
protected $signature = 'command:name {--option} {--option=value} {--o|option}';

// 获取参数
$this->argument('arg1');
$this->arguments(); // 所有参数

// 获取选项
$this->option('option');
$this->options(); // 所有选项
```

### 输出方法

```php
// 信息输出
$this->info('Info message');
$this->error('Error message');
$this->warn('Warning message');
$this->line('Plain message');

// 表格输出
$this->table(['Name', 'Email'], [
    ['John', 'john@example.com'],
    ['Jane', 'jane@example.com'],
]);

// 进度条
$bar = $this->createProgressBar(100);
foreach ($items as $item) {
    // 处理
    $bar->advance();
}
$bar->finish();

// 交互式输入
$name = $this->ask('What is your name?');
$password = $this->secret('Enter password:');
$confirm = $this->confirm('Do you want to continue?');
$choice = $this->choice('Select one', ['a', 'b', 'c']);
```
