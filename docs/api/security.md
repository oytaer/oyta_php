# Security 安全模块

## 模块概述

Security 模块提供全面的安全防护功能，包括加密解密、哈希算法、XSS 防护、CSRF 防护、SQL 注入防护等。对应 ThinkPHP 8.0 的安全功能。

## 模块结构

```
security/
├── mod.rs            # 模块入口
├── crypto.rs         # 加密解密（AES-256-GCM）
├── hash.rs           # 哈希算法（SHA-256、PBKDF2）
├── csrf.rs           # CSRF 防护核心
├── xss.rs            # XSS 防护
├── sql_injection.rs  # SQL 注入防护
├── crypt_facade.rs   # Crypt 门面
├── hash_facade.rs    # Hash 门面
└── csrf_facade.rs    # CSRF 门面
```

## 加密解密 (crypto.rs)

### AES-256-GCM 对称加密

```rust
pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>>;
pub fn decrypt(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>>;
pub fn generate_key() -> [u8; 32];
pub fn random_bytes(length: usize) -> Vec<u8>;
pub fn random_hex(length: usize) -> String;
```

### Base64 编解码

```rust
pub fn base64_encode(data: &[u8]) -> String;
pub fn base64_decode(encoded: &str) -> Result<Vec<u8>>;
```

### PHP 使用

```php
<?php
// 加密
$encrypted = Crypt::encrypt($data);
$encrypted = Crypt::encrypt($data, $key);

// 解密
$decrypted = Crypt::decrypt($encrypted);
$decrypted = Crypt::decrypt($encrypted, $key);

// 生成密钥
$key = Crypt::generateKey();

// Base64 编解码
$encoded = Crypt::base64Encode($data);
$decoded = Crypt::base64Decode($encoded);

// 随机数据
$bytes = Crypt::randomBytes(32);
$hex = Crypt::randomHex(32);
```

## 哈希算法 (hash.rs)

### 密码哈希

```rust
pub fn hash_password(password: &str) -> Result<String>;
pub fn verify_password(password: &str, hash: &str) -> bool;
pub fn needs_rehash(hash: &str) -> bool;
```

### SHA-256 哈希

```rust
pub fn sha256(data: &[u8]) -> String;
pub fn hmac_sha256(data: &[u8], key: &[u8]) -> String;
```

### PBKDF2 密钥派生

```rust
pub fn pbkdf2(password: &str, salt: &[u8], iterations: u32) -> Result<Vec<u8>>;
pub fn generate_salt() -> [u8; 16];
```

### PHP 使用

```php
<?php
// 密码哈希
$hash = Hash::make('password123');

// 验证密码
if (Hash::check('password123', $hash)) {
    // 密码正确
}

// 检查是否需要重新哈希
if (Hash::needsRehash($hash)) {
    $newHash = Hash::make('password123');
}

// SHA-256
$hash = Hash::sha256($data);
$hmac = Hash::hmacSha256($data, $key);

// PBKDF2
$derived = Hash::pbkdf2($password, $salt, 10000);
$salt = Hash::generateSalt();
```

## XSS 防护 (xss.rs)

### HTML 实体转义

```rust
pub fn escape_html(input: &str) -> String;
pub fn escape_js(input: &str) -> String;
pub fn escape_css(input: &str) -> String;
pub fn escape_url(input: &str) -> String;
```

### 标签清理

```rust
pub fn strip_tags(input: &str) -> String;
pub fn strip_tags_allow(input: &str, allowed: &[&str]) -> String;
pub fn sanitize_html(input: &str) -> String;
```

### 深度清理

```rust
pub fn deep_clean(input: &str) -> String;
pub fn decode_entities(input: &str) -> String;
pub fn clean_filename(filename: &str) -> String;
pub fn filter_url_protocol(url: &str) -> String;
```

### PHP 使用

```php
<?php
// HTML 转义
$safe = Security::escape($input);
$safe = Security::escapeJs($input);
$safe = Security::escapeCss($input);
$safe = Security::escapeUrl($input);

// 标签清理
$clean = Security::stripTags($input);
$clean = Security::stripTagsAllow($input, ['p', 'br', 'strong']);

// 深度清理
$clean = Security::deepClean($input);
$clean = Security::sanitizeHtml($input);

// 文件名安全处理
$safe = Security::cleanFilename($filename);

// URL 协议过滤
$safe = Security::filterUrlProtocol($url);
```

## CSRF 防护 (csrf.rs)

### 令牌管理

```rust
pub struct CsrfToken {
    pub value: String,
    pub expires_at: Instant,
}

pub struct CsrfManager {
    tokens: HashMap<String, CsrfToken>,
    token_lifetime: Duration,
}
```

### 核心方法

```rust
pub fn generate_token(&mut self, session_id: &str) -> String;
pub fn validate_token(&self, session_id: &str, token: &str) -> bool;
pub fn consume_token(&mut self, session_id: &str, token: &str) -> bool;
pub fn invalidate_token(&mut self, session_id: &str);
pub fn cleanup_expired(&mut self);
```

### PHP 使用

```php
<?php
// 生成令牌
$token = Csrf::token();
$token = Csrf::token($sessionId);

// 验证令牌
if (Csrf::verify($token)) {
    // 验证通过
}

// 消费式验证（验证后令牌失效）
if (Csrf::check($token)) {
    // 验证通过
}

// 使令牌失效
Csrf::invalidate();

// 生成表单字段
echo Csrf::field();
// 输出: <input type="hidden" name="_token" value="...">

// 生成 meta 标签
echo Csrf::meta();
// 输出: <meta name="csrf-token" content="...">
```

## SQL 注入防护 (sql_injection.rs)

### 检测与转义

```rust
pub fn detect_sqli(input: &str) -> bool;
pub fn escape_string(input: &str) -> String;
pub fn quote_identifier(identifier: &str) -> String;
```

### 安全构建

```rust
pub fn safe_limit(limit: &str) -> Result<String>;
pub fn safe_order_by(column: &str, direction: &str) -> Result<String>;
pub fn escape_like_pattern(pattern: &str) -> String;
pub fn safe_table_name(name: &str) -> Result<String>;
pub fn safe_column_name(name: &str) -> Result<String>;
```

### PHP 使用

```php
<?php
// 检测 SQL 注入
if (Security::detectSqli($input)) {
    // 可能存在注入
}

// 转义字符串
$safe = Security::escapeString($input);

// 引用标识符
$safe = Security::quoteIdentifier($column);

// 安全 LIMIT
$safe = Security::safeLimit($limit);

// 安全 ORDER BY
$safe = Security::safeOrderBy($column, 'ASC');

// LIKE 模式转义
$safe = Security::escapeLikePattern($pattern);

// 安全表名/列名
$safe = Security::safeTableName($name);
$safe = Security::safeColumnName($name);
```

## Crypt 门面 API

```php
<?php
// 加密解密
Crypt::encrypt($data);
Crypt::decrypt($encrypted);
Crypt::encryptWithKey($data, $key);
Crypt::decryptWithKey($encrypted, $key);

// 密钥生成
Crypt::generateKey();
Crypt::generateSalt();

// 随机数据
Crypt::randomBytes($length);
Crypt::randomHex($length);
Crypt::randomInt($min, $max);
Crypt::randomString($length);

// Base64
Crypt::base64Encode($data);
Crypt::base64Decode($encoded);
Crypt::urlSafeBase64Encode($data);
Crypt::urlSafeBase64Decode($encoded);
```

## Hash 门面 API

```php
<?php
// 密码哈希
Hash::make($password);
Hash::check($password, $hash);
Hash::needsRehash($hash);

// SHA
Hash::sha256($data);
Hash::sha512($data);
Hash::hmacSha256($data, $key);
Hash::hmacSha512($data, $key);

// PBKDF2
Hash::pbkdf2($password, $salt, $iterations);
Hash::generateSalt();

// 兼容旧版
Hash::md5($data);
Hash::sha1($data);
```

## Csrf 门面 API

```php
<?php
// 令牌操作
Csrf::token();
Csrf::verify($token);
Csrf::check($token);
Csrf::invalidate();

// 表单辅助
Csrf::field();
Csrf::meta();

// 管理
Csrf::setLifetime($seconds);
Csrf::clearExpired();
```

## 配置示例

```php
<?php
// config/security.php
return [
    // 加密配置
    'encryption' => [
        'key' => env('APP_KEY'),
        'cipher' => 'AES-256-GCM',
    ],
    
    // 哈希配置
    'hashing' => [
        'driver' => 'pbkdf2',
        'iterations' => 100000,
    ],
    
    // CSRF 配置
    'csrf' => [
        'enabled' => true,
        'token_name' => '_token',
        'lifetime' => 3600,
    ],
    
    // XSS 配置
    'xss' => [
        'enabled' => true,
        'allowed_tags' => ['p', 'br', 'strong', 'em'],
    ],
];
```

## 安全最佳实践

1. **永远不要信任用户输入**
   ```php
   $id = (int) $request->param('id');
   $name = Security::escape($request->param('name'));
   ```

2. **使用参数化查询防止 SQL 注入**
   ```php
   $users = Db::table('users')->where('id', $id)->select();
   ```

3. **所有表单添加 CSRF 保护**
   ```php
   <form method="POST">
       <?= Csrf::field() ?>
   </form>
   ```

4. **使用强密码哈希**
   ```php
   $hash = Hash::make($password);
   ```

5. **输出时进行 XSS 防护**
   ```php
   echo Security::escape($userInput);
   ```
