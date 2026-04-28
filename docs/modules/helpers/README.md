# 辅助函数模块 (Helpers)

## 模块概述

辅助函数模块提供 ThinkPHP 风格的辅助函数，方便用户快速调用常用功能。

## 模块结构

```
helpers/
├── mod.rs          # 模块入口
└── functions.rs    # 辅助函数
```

## 辅助函数列表

| 函数 | 说明 |
|------|------|
| input() | 获取输入参数 |
| config() | 获取配置值 |
| env() | 获取环境变量 |
| url() | 生成 URL |
| json() | 返回 JSON 响应 |
| redirect() | 重定向 |
| cache() | 缓存操作 |
| session() | Session 操作 |
| cookie() | Cookie 操作 |
| dump() | 打印变量 |
| dd() | 打印并终止 |
| halt() | 中断执行 |
| app() | 获取应用实例 |
| request() | 获取请求对象 |
| response() | 创建响应 |
| view() | 渲染模板 |
| event() | 触发事件 |
| validate() | 数据验证 |
| db() | 数据库查询 |
| model() | 获取模型实例 |
| runtime_path() | 运行时目录 |
| root_path() | 项目根目录 |
| app_path() | 应用目录 |
| config_path() | 配置目录 |
| public_path() | 公共目录 |

## 使用示例

```php
<?php
// 获取输入
$name = input('name');

// 获取配置
$debug = config('app.debug');

// 获取环境变量
$dbHost = env('DB_HOST', 'localhost');

// 返回 JSON
return json(['code' => 0, 'data' => $data]);

// 重定向
return redirect('/login');

// 缓存操作
cache('key', 'value', 3600);
$value = cache('key');

// Session 操作
session('user_id', 1);
$userId = session('user_id');

// 打印调试
dump($data);
dd($data);  // 打印并终止

// 渲染模板
return view('index/index', ['title' => '首页']);

// 数据库查询
$users = db('users')->select();

// 路径函数
$path = runtime_path('cache');
$path = root_path();
$path = app_path('controller');
```

## 技术实现

- 全局辅助函数
- ThinkPHP 兼容
- 简化常用操作
