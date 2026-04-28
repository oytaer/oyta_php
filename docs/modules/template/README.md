# 模板引擎模块 (Template)

## 模块概述

模板引擎模块提供 OYTAPHP 的模板渲染功能，对应 ThinkPHP 8.0 的 View 门面。支持模板渲染、变量分配、模板缓存等功能。

## 模块结构

```
template/
├── mod.rs          # 模块入口
├── facade.rs       # View 门面
├── engine.rs       # 模板引擎核心
├── parser.rs       # 模板解析器
├── compiler.rs     # 模板编译器
└── cache.rs        # 模板缓存
```

## View 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `init` | view_dir: &Path | Result<()> | 初始化模板引擎 |
| `fetch` | template_name: &str, data: &HashMap | Result<String> | 渲染模板 |
| `display` | template_name: &str, data: &HashMap | Result<Response> | 渲染并返回响应 |
| `assign` | name: &str, value: Value | - | 分配模板变量 |
| `assign_batch` | variables: &HashMap | - | 批量分配变量 |
| `get` | name: &str | Option<Value> | 获取已分配的变量 |
| `has` | name: &str | bool | 检查变量是否存在 |
| `remove` | name: &str | - | 移除变量 |
| `clear` | - | - | 清空所有变量 |
| `all` | - | HashMap | 获取所有变量 |
| `reload` | - | Result<()> | 重新加载模板 |
| `set_theme` | theme: &str | - | 设置模板主题 |
| `get_theme` | - | String | 获取当前主题 |
| `register_filter` | name: &str, filter: Fn | - | 注册自定义过滤器 |
| `register_function` | name: &str, func: Fn | - | 注册模板函数 |
| `add_template` | name: &str, content: &str | Result<()> | 添加原始模板 |

## 使用示例

### 基础用法

```php
<?php
use oyta\facade\View;

// 分配单个变量
View::assign('title', '网站标题');

// 批量分配变量
View::assign_batch([
    'title' => '网站标题',
    'keywords' => '关键词',
    'description' => '网站描述',
]);

// 渲染模板
$html = View::fetch('index/index', [
    'content' => '页面内容',
]);

// 渲染并返回响应
return View::display('index/index', [
    'content' => '页面内容',
]);
```

### 控制器中使用

```php
<?php
namespace app\controller;

use oyta\facade\View;

class Index
{
    public function index()
    {
        return View::fetch('index/index', [
            'title' => '首页',
            'list' => [
                ['id' => 1, 'name' => '项目1'],
                ['id' => 2, 'name' => '项目2'],
            ],
        ]);
    }
}
```

### 模板语法

```html
<!-- 变量输出 -->
<h1>{$title}</h1>

<!-- 条件判断 -->
{if $status == 1}
    <span class="success">正常</span>
{else}
    <span class="error">禁用</span>
{/if}

<!-- 循环 -->
{foreach $list as $item}
    <li>{$item.name}</li>
{/foreach}

<!-- 包含文件 -->
{include file="common/header"}

<!-- 布局继承 -->
{extend name="layout/base" /}
{block name="content"}
    页面内容
{/block}
```

## 配置示例

```php
<?php
// config/view.php
return [
    'type' => 'native',
    'view_dir' => app_path('view'),
    'view_suffix' => '.html',
    'cache_enabled' => true,
    'cache_dir' => runtime_path('cache/view'),
    'theme' => 'default',
    'taglib_begin' => '{',
    'taglib_end' => '}',
];
```

## 技术实现

- 基于 Tera 模板引擎
- 支持模板缓存
- 支持自定义过滤器和函数
- 支持模板继承和布局
