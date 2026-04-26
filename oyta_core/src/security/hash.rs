//! 哈希功能
//!
//! 提供密码学哈希和密码哈希功能
//! 密码哈希使用 PBKDF2-HMAC-SHA256 进行密钥派生
//! 兼容 ThinkPHP 8.0 的密码哈希接口

use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};

type HmacSha256 = Hmac<Sha256>;

/// PBKDF2 迭代次数
const PBKDF2_ITERATIONS: u32 = 600_000;

/// 盐值长度（字节）
const SALT_LENGTH: usize = 32;

/// SHA-256 哈希
pub fn sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// SHA-256 HMAC
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC 可以接受任意长度的密钥");
    mac.update(data);
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

/// PBKDF2 密钥派生
///
/// 使用 HMAC-SHA256 进行多轮迭代
///
/// # 参数
/// - `password`: 密码
/// - `salt`: 盐值
/// - `iterations`: 迭代次数
/// - `output_len`: 输出长度（字节）
///
/// # 返回
/// 派生密钥的十六进制字符串
pub fn pbkdf2(password: &[u8], salt: &[u8], iterations: u32, output_len: usize) -> String {
    let mut result = vec![0u8; output_len];
    pbkdf2::pbkdf2::<HmacSha256>(password, salt, iterations, &mut result)
        .expect("PBKDF2 输出长度有效");
    hex::encode(result)
}

/// 生成密码学安全随机盐
pub fn generate_salt() -> String {
    crate::security::crypto::random_hex(SALT_LENGTH)
}

/// 密码哈希（PBKDF2-HMAC-SHA256）
///
/// 使用高迭代次数的 PBKDF2 进行密钥派生
/// 返回格式：$oyta$iterations$salt$hash
///
/// # 参数
/// - `password`: 明文密码
///
/// # 返回
/// 哈希后的密码字符串
pub fn hash_password(password: &str) -> String {
    let salt = generate_salt();
    let hash = pbkdf2(
        password.as_bytes(),
        salt.as_bytes(),
        PBKDF2_ITERATIONS,
        32,
    );
    format!("$oyta${}${}${}", PBKDF2_ITERATIONS, salt, hash)
}

/// 验证密码
///
/// 从哈希字符串中提取参数并重新计算
///
/// # 参数
/// - `password`: 待验证的明文密码
/// - `stored_hash`: 存储的哈希字符串
///
/// # 返回
/// 密码是否匹配
pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    let parts: Vec<&str> = stored_hash.split('$').collect();
    if parts.len() != 5 || parts[1] != "oyta" {
        return false;
    }
    let iterations: u32 = match parts[2].parse() {
        Ok(n) => n,
        Err(_) => return false,
    };
    let salt = parts[3];
    let expected_hash = parts[4];
    let computed = pbkdf2(
        password.as_bytes(),
        salt.as_bytes(),
        iterations,
        32,
    );
    constant_time_eq(expected_hash.as_bytes(), computed.as_bytes())
}

/// 常量时间比较
///
/// 防止时序攻击
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// 兼容旧版：SHA-256 + 盐值密码哈希
pub fn hash_password_legacy(password: &str, salt: &str) -> String {
    let data = format!("{}{}", password, salt);
    sha256(data.as_bytes())
}

/// 兼容旧版：验证 SHA-256 + 盐值密码
pub fn verify_password_legacy(password: &str, salt: &str, hash: &str) -> bool {
    let computed = hash_password_legacy(password, salt);
    constant_time_eq(computed.as_bytes(), hash.as_bytes())
}
