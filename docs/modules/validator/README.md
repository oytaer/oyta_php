# 验证器模块 (Validator)

## 模块概述

验证器模块提供 OYTAPHP 的数据验证功能，对应 ThinkPHP 8.0 的 Validate 类。支持多种验证规则、自定义验证消息、验证场景等功能。

## 模块结构

```
validator/
├── mod.rs          # 模块入口
├── facade.rs       # Validate 门面
├── validator.rs    # 验证器核心
└── rules.rs        # 验证规则
```

## Validate 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `check` | data: &HashMap, rules: &HashMap | bool | 验证数据 |
| `make` | data: &HashMap, rules: &HashMap | Validator | 创建验证器实例 |
| `getError` | - | Option<String> | 获取错误信息 |
| `scene` | name: &str | Validator | 设置验证场景 |
| `rule` | field: &str, rule: &str | Validator | 添加验证规则 |
| `message` | field: &str, message: &str | Validator | 设置错误消息 |

## 验证规则

| 规则 | 说明 | 示例 |
|------|------|------|
| `required` | 必填 | `required` |
| `email` | 邮箱格式 | `email` |
| `url` | URL 格式 | `url` |
| `ip` | IP 地址 | `ip` |
| `integer` | 整数 | `integer` |
| `float` | 浮点数 | `float` |
| `boolean` | 布尔值 | `boolean` |
| `array` | 数组 | `array` |
| `min` | 最小值/长度 | `min:6` |
| `max` | 最大值/长度 | `max:20` |
| `between` | 范围 | `between:1,10` |
| `in` | 在列表中 | `in:1,2,3` |
| `not_in` | 不在列表中 | `not_in:1,2,3` |
| `regex` | 正则匹配 | `regex:/^\d+$/` |
| `date` | 日期格式 | `date` |
| `alpha` | 字母 | `alpha` |
| `alpha_num` | 字母数字 | `alpha_num` |
| `confirmed` | 确认字段 | `confirmed:password_confirm` |

## 使用示例

### 创建验证器

```php
<?php
namespace app\validate;

class UserValidate
{
    protected $rule = [
        'name' => 'require|max:25',
        'email' => 'require|email',
        'password' => 'require|min:6',
        'age' => 'between:1,120',
    ];
    
    protected $message = [
        'name.require' => '用户名不能为空',
        'name.max' => '用户名最多25个字符',
        'email.require' => '邮箱不能为空',
        'email.email' => '邮箱格式不正确',
        'password.require' => '密码不能为空',
        'password.min' => '密码至少6个字符',
        'age.between' => '年龄必须在1-120之间',
    ];
    
    protected $scene = [
        'login' => ['email', 'password'],
        'register' => ['name', 'email', 'password', 'age'],
    ];
}
```

### 使用验证器

```php
<?php
use oyta\facade\Validate;
use app\validate\UserValidate;

// 验证数据
$validator = new UserValidate();
if (!$validator->check($data)) {
    return json([
        'code' => 400,
        'msg' => $validator->getError(),
    ]);
}

// 使用场景验证
$validator->scene('login')->check($data);

// 门面方式验证
$rules = [
    'name' => 'require|max:25',
    'email' => 'require|email',
];

if (!Validate::check($data, $rules)) {
    return Validate::getError();
}
```

### 自定义验证规则

```php
<?php
use oyta\facade\Validate;

// 添加自定义规则
Validate::extend('mobile', function($value) {
    return preg_match('/^1[3-9]\d{9}$/', $value);
}, '手机号格式不正确');

// 使用自定义规则
$rules = [
    'phone' => 'require|mobile',
];
```

## 技术实现

- 支持多种内置验证规则
- 支持自定义验证规则
- 支持验证场景
- 支持批量验证
