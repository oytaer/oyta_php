# 国际化模块 (I18n)

## 模块概述

国际化模块提供 OYTAPHP 的多语言支持，对应 ThinkPHP 8.0 的 Lang 类。

## 模块结构

```
i18n/
├── mod.rs          # 模块入口
├── facade.rs       # Lang 门面
├── i18n.rs         # 国际化核心
└── lang.rs         # 语言包管理
```

## Lang 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `get` | key: &str, vars: HashMap | String | 获取翻译文本 |
| `set` | key: &str, value: &str | - | 设置翻译文本 |
| `has` | key: &str | bool | 检查翻译是否存在 |
| `locale` | - | String | 获取当前语言 |
| `setLocale` | locale: &str | - | 设置当前语言 |

## 使用示例

### 语言包文件

```php
<?php
// lang/zh-cn.php
return [
    'welcome' => '欢迎',
    'hello' => '你好，:name',
    'user_not_found' => '用户不存在',
];

// lang/en-us.php
return [
    'welcome' => 'Welcome',
    'hello' => 'Hello, :name',
    'user_not_found' => 'User not found',
];
```

### 使用翻译

```php
<?php
use oyta\facade\Lang;

// 获取翻译
echo Lang::get('welcome');  // 输出: 欢迎

// 带变量翻译
echo Lang::get('hello', ['name' => '张三']);  // 输出: 你好，张三

// 设置语言
Lang::setLocale('en-us');
echo Lang::get('welcome');  // 输出: Welcome

// 获取当前语言
$locale = Lang::locale();  // 输出: en-us
```

### 配置示例

```php
<?php
// config/lang.php
return [
    'default' => 'zh-cn',
    'fallback' => 'en-us',
    'path' => app_path('lang'),
    'cookie_var' => 'lang',
    'header_var' => 'Accept-Language',
];
```

## 技术实现

- 多语言包管理
- 变量替换
- 自动语言检测
- 语言切换
