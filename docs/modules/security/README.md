# 安全模块 (Security)

## 模块概述

安全模块提供 OYTAPHP 的安全功能，包括加密解密、哈希、CSRF 防护、XSS 防护、SQL 注入防护等。

## 模块结构

```
security/
├── mod.rs          # 模块入口
├── crypto.rs       # 加密解密
├── csrf.rs         # CSRF 防护
├── hash.rs         # 哈希处理
├── xss.rs          # XSS 防护
├── sql_injection.rs # SQL 注入防护
├── crypt_facade.rs # Crypt 门面
├── hash_facade.rs  # Hash 门面
└── csrf_facade.rs  # Csrf 门面
```

## Crypt 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `encrypt` | value: &str | Result<String> | 加密数据 |
| `decrypt` | value: &str | Result<String> | 解密数据 |

## Hash 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `make` | value: &str | String | 生成哈希 |
| `check` | value: &str, hash: &str | bool | 验证哈希 |
| `needsRehash` | hash: &str | bool | 检查是否需要重新哈希 |

## Csrf 门面

### 方法列表

| 方法 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `token` | - | String | 生成 CSRF Token |
| `verify` | token: &str | bool | 验证 CSRF Token |
| `refresh` | - | String | 刷新 Token |

## 使用示例

### 加密解密

```php
<?php
use oyta\facade\Crypt;

// 加密
$encrypted = Crypt::encrypt('敏感数据');

// 解密
$decrypted = Crypt::decrypt($encrypted);
```

### 密码哈希

```php
<?php
use oyta\facade\Hash;

// 生成密码哈希
$hash = Hash::make('password123');

// 验证密码
if (Hash::check('password123', $hash)) {
    echo "密码正确";
}

// 检查是否需要重新哈希
if (Hash::needsRehash($hash)) {
    $newHash = Hash::make('password123');
}
```

### CSRF 防护

```php
<?php
use oyta\facade\Csrf;

// 生成 Token
$token = Csrf::token();

// 表单中输出
echo '<input type="hidden" name="_token" value="' . $token . '">';

// 验证 Token
if (!Csrf::verify($_POST['_token'])) {
    throw new \Exception('CSRF Token 验证失败');
}
```

### XSS 防护

```php
<?php
// 自动转义输出
echo htmlspecialchars($userInput, ENT_QUOTES, 'UTF-8');
```

## 配置示例

```php
<?php
// config/app.php
return [
    'key' => env('APP_KEY', 'your-secret-key'),
    'cipher' => 'AES-256-GCM',
];
```

## 技术实现

- AES-GCM 加密算法
- SHA-256/SHA-3 哈希算法
- CSRF Token 验证
- XSS 过滤
- SQL 注入防护
