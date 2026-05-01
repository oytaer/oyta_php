# Config 配置模块

## 模块结构

```
config/
├── mod.rs       # 模块入口
├── facade.rs    # 配置门面
├── loader.rs    # 配置加载器
└── store.rs     # 配置存储
```

## 基本使用

```php
<?php
// 获取配置
$value = Config::get('app.name');
$value = Config::get('app.name', 'default');

// 设置配置
Config::set('app.debug', true);

// 检查配置是否存在
if (Config::has('app.name')) {
    // ...
}

// 获取所有配置
$all = Config::all();
```

## 环境变量

```php
<?php
// 获取环境变量
$value = Env::get('APP_ENV');
$value = Env::get('APP_ENV', 'production');

// 检查是否存在
if (Env::has('DATABASE_URL')) {
    // ...
}
```

## 配置文件

```php
<?php
// config/app.php
return [
    'name' => 'OYTAPHP',
    'debug' => env('APP_DEBUG', false),
    'timezone' => 'Asia/Shanghai',
];

// config/database.php
return [
    'default' => 'mysql',
    'connections' => [
        'mysql' => [
            'host' => env('DB_HOST', '127.0.0.1'),
            'port' => env('DB_PORT', 3306),
        ],
    ],
];
```

## 加载优先级

1. 环境变量
2. .env 文件
3. config/*.php 配置文件
4. 默认值
