//! 密码哈希门面
//!
//! 提供静态方法访问密码哈希功能
//! 对应 ThinkPHP 8.0 的 Hash 类
//!
//! # 使用示例
//! ```php
//! // 生成密码哈希
//! $hash = Hash::make('password123');
//!
//! // 验证密码
//! if (Hash::check('password123', $hash)) {
//!     // 密码正确
//! }
//! ```

use super::hash;

/// Hash 门面结构体
///
/// 提供静态方法访问密码哈希功能
/// 所有方法都是无状态的，不需要初始化
pub struct Hash;

impl Hash {
    /// 生成密码哈希
    ///
    /// 使用 PBKDF2-HMAC-SHA256 算法
    /// 返回格式：$oyta$iterations$salt$hash
    ///
    /// # 参数
    /// - `password`: 明文密码
    ///
    /// # 返回
    /// 哈希后的密码字符串
    ///
    /// # 示例
    /// ```php
    /// $hash = Hash::make('password123');
    /// // 输出类似: $oyta$600000$abc123...$def456...
    /// ```
    pub fn make(password: &str) -> String {
        // 调用哈希模块的密码哈希函数
        hash::hash_password(password)
    }

    /// 验证密码
    ///
    /// 检查明文密码是否与存储的哈希匹配
    /// 使用常量时间比较防止时序攻击
    ///
    /// # 参数
    /// - `password`: 待验证的明文密码
    /// - `hashed_value`: 存储的哈希值
    ///
    /// # 返回
    /// 如果密码匹配返回 true，否则返回 false
    ///
    /// # 示例
    /// ```php
    /// if (Hash::check('password123', $storedHash)) {
    ///     // 密码正确
    /// }
    /// ```
    pub fn check(password: &str, hashed_value: &str) -> bool {
        // 调用哈希模块的密码验证函数
        hash::verify_password(password, hashed_value)
    }

    /// 检查哈希是否需要重新生成
    ///
    /// 判断哈希是否使用了过时的算法或参数
    /// 如果返回 true，建议重新生成哈希
    ///
    /// # 参数
    /// - `hashed_value`: 存储的哈希值
    ///
    /// # 返回
    /// 如果需要重新哈希返回 true
    ///
    /// # 示例
    /// ```php
    /// if (Hash::needsRehash($storedHash)) {
    ///     $newHash = Hash::make($password);
    ///     // 更新存储的哈希
    /// }
    /// ```
    pub fn needs_rehash(hashed_value: &str) -> bool {
        // 解析哈希字符串
        let parts: Vec<&str> = hashed_value.split('$').collect();
        // 检查格式是否正确
        if parts.len() != 5 || parts[1] != "oyta" {
            // 格式不正确，需要重新哈希
            return true;
        }
        // 解析迭代次数
        if let Ok(iterations) = parts[2].parse::<u32>() {
            // 检查迭代次数是否低于当前推荐值
            // 当前推荐值为 600000
            if iterations < 600000 {
                // 迭代次数过低，需要重新哈希
                return true;
            }
        } else {
            // 无法解析迭代次数，需要重新哈希
            return true;
        }
        // 不需要重新哈希
        false
    }

    /// 生成随机盐值
    ///
    /// 生成密码学安全的随机盐
    ///
    /// # 返回
    /// 64 字符的十六进制盐值字符串
    pub fn generate_salt() -> String {
        // 调用哈希模块的盐值生成函数
        hash::generate_salt()
    }

    /// SHA-256 哈希
    ///
    /// 计算数据的 SHA-256 哈希值
    /// 注意：不适用于密码存储，仅用于数据完整性校验
    ///
    /// # 参数
    /// - `data`: 待哈希的数据
    ///
    /// # 返回
    /// 64 字符的十六进制哈希字符串
    ///
    /// # 示例
    /// ```php
    /// $hash = Hash::sha256('hello world');
    /// ```
    pub fn sha256(data: &str) -> String {
        // 调用哈希模块的 SHA-256 函数
        hash::sha256(data.as_bytes())
    }

    /// SHA-256 哈希（字节数据）
    ///
    /// 计算字节数据的 SHA-256 哈希值
    ///
    /// # 参数
    /// - `data`: 字节数据
    ///
    /// # 返回
    /// 64 字符的十六进制哈希字符串
    pub fn sha256_bytes(data: &[u8]) -> String {
        // 调用哈希模块的 SHA-256 函数
        hash::sha256(data)
    }

    /// HMAC-SHA256
    ///
    /// 计算消息认证码
    /// 用于验证消息完整性和真实性
    ///
    /// # 参数
    /// - `key`: 密钥
    /// - `data`: 待认证的数据
    ///
    /// # 返回
    /// 64 字符的十六进制 HMAC 字符串
    ///
    /// # 示例
    /// ```php
    /// $hmac = Hash::hmacSha256('secret_key', 'message');
    /// ```
    pub fn hmac_sha256(key: &str, data: &str) -> String {
        // 调用哈希模块的 HMAC-SHA256 函数
        hash::hmac_sha256(key.as_bytes(), data.as_bytes())
    }

    /// PBKDF2 密钥派生
    ///
    /// 使用 PBKDF2 算法从密码派生密钥
    ///
    /// # 参数
    /// - `password`: 密码
    /// - `salt`: 盐值
    /// - `iterations`: 迭代次数
    /// - `output_len`: 输出长度（字节）
    ///
    /// # 返回
    /// 派生密钥的十六进制字符串
    pub fn pbkdf2(password: &str, salt: &str, iterations: u32, output_len: usize) -> String {
        // 调用哈希模块的 PBKDF2 函数
        hash::pbkdf2(
            password.as_bytes(),
            salt.as_bytes(),
            iterations,
            output_len,
        )
    }

    /// 兼容旧版密码验证
    ///
    /// 验证使用旧版 SHA-256 + 盐值方式哈希的密码
    /// 用于迁移旧系统密码
    ///
    /// # 参数
    /// - `password`: 明文密码
    /// - `salt`: 盐值
    /// - `hashed_value`: 存储的哈希值
    ///
    /// # 返回
    /// 如果密码匹配返回 true
    pub fn check_legacy(password: &str, salt: &str, hashed_value: &str) -> bool {
        // 调用哈希模块的旧版验证函数
        hash::verify_password_legacy(password, salt, hashed_value)
    }

    /// 兼容旧版密码哈希
    ///
    /// 使用旧版 SHA-256 + 盐值方式生成哈希
    /// 仅用于向后兼容，新系统请使用 make 方法
    ///
    /// # 参数
    /// - `password`: 明文密码
    /// - `salt`: 盐值
    ///
    /// # 返回
    /// 哈希字符串
    pub fn make_legacy(password: &str, salt: &str) -> String {
        // 调用哈希模块的旧版哈希函数
        hash::hash_password_legacy(password, salt)
    }

    /// 计算文件 SHA-256
    ///
    /// 计算文件的 SHA-256 哈希值
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// 文件的 SHA-256 哈希值，失败返回错误信息
    pub fn file_sha256(path: &str) -> Result<String, String> {
        // 读取文件内容
        match std::fs::read(path) {
            // 读取成功
            Ok(content) => {
                // 计算哈希
                Ok(hash::sha256(&content))
            }
            // 读取失败
            Err(e) => {
                // 返回错误信息
                Err(format!("无法读取文件: {}", e))
            }
        }
    }

    /// 计算数据 MD5（不推荐用于安全场景）
    ///
    /// 仅用于兼容旧系统或非安全场景
    /// 不应用于密码存储或安全验证
    ///
    /// # 参数
    /// - `data`: 待哈希的数据
    ///
    /// # 返回
    /// 32 字符的十六进制 MD5 哈希字符串
    #[deprecated(note = "MD5 不安全，仅用于兼容旧系统")]
    pub fn md5(data: &str) -> String {
        // 使用 md5 算法计算哈希
        format!("{:x}", md5::compute(data.as_bytes()))
    }
}
