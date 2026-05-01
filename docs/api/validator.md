# Validator 数据验证模块

## 模块结构

```
validator/
├── mod.rs       # 模块入口
├── validator.rs # 验证器实现
└── rules.rs     # 验证规则
```

## 基本使用

```php
<?php
$validate = new Validate();

$validate->rule([
    'name'  => 'require|max:25',
    'age'   => 'number|between:1,120',
    'email' => 'email',
]);

if (!$validate->check($data)) {
    echo $validate->getError();
}
```

## 验证规则

```php
<?php
// 必填
'name' => 'require'

// 长度
'name' => 'length:1,25'
'name' => 'max:25'
'name' => 'min:5'

// 数字
'age' => 'number'
'age' => 'integer'
'price' => 'float'
'age' => 'between:1,120'
'score' => 'in:1,2,3,4,5'

// 格式
'email' => 'email'
'url' => 'url'
'phone' => 'mobile'
'date' => 'date'
'ip' => 'ip'

// 正则
'code' => 'regex:/^\d{6}$/'

// 比较
'password' => 'confirm:repassword'
'password' => 'different:old_password'
'age' => 'eq:18'
'score' => 'gt:60'

// 文件
'avatar' => 'file'
'avatar' => 'fileExt:jpg,png'
'avatar' => 'fileSize:102400'
```

## 场景验证

```php
<?php
$validate = new Validate();

$validate->rule([
    'name' => 'require',
    'email' => 'require|email',
    'password' => 'require|min:6',
]);

// 定义场景
$validate->scene('login', ['email', 'password']);
$validate->scene('register', ['name', 'email', 'password']);

// 使用场景
if (!$validate->scene('login')->check($data)) {
    // ...
}
```

## 自定义规则

```php
<?php
Validate::extend('custom', function($value, $rule) {
    return $value === $rule;
}, '验证失败');

// 使用
'field' => 'custom:value'
```
