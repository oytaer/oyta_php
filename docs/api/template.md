# Template 模板模块

## 模块结构

```
template/
├── mod.rs        # 模块入口
├── engine.rs     # 模板引擎接口
├── tera_engine.rs # Tera 引擎实现
└── facade.rs     # 视图门面
```

## 基本使用

```php
<?php
// 渲染模板
return View::fetch('index/index', [
    'title' => '首页',
    'users' => $users,
]);

// 渲染字符串
$html = View::display('Hello, {{ name }}!', ['name' => 'World']);

// 模板变量赋值
View::assign('site_name', 'OYTAPHP');
View::assign(['key1' => 'value1', 'key2' => 'value2']);

// 获取变量
$name = View::get('site_name');
```

## 模板语法

```html
<!-- 变量输出 -->
<h1>{{ title }}</h1>

<!-- 条件判断 -->
{% if user.is_admin %}
    <span>管理员</span>
{% elseif user.is_vip %}
    <span>VIP用户</span>
{% else %}
    <span>普通用户</span>
{% endif %}

<!-- 循环 -->
{% for user in users %}
    <li>{{ user.name }}</li>
{% endfor %}

<!-- 包含子模板 -->
{% include 'header.html' %}

<!-- 继承模板 -->
{% extends 'layout.html' %}

{% block content %}
    <p>页面内容</p>
{% endblock %}
```

## 配置

```php
<?php
// config/view.php
return [
    'type' => 'tera',
    'view_path' => app_path('view'),
    'cache_path' => runtime_path('view/cache'),
    'cache' => !env('APP_DEBUG'),
];
```
