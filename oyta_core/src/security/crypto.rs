//! 加密解密功能
//!
//! 提供 AES-256-GCM 对称加密/解密
//! 提供 Base64 编码/解码
//! 兼容 ThinkPHP 8.0 的加密服务

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use aes_gcm::aead::rand_core::RngCore;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

/// Base64 编码
pub fn base64_encode(data: &[u8]) -> String {
    BASE64.encode(data)
}

/// Base64 解码
pub fn base64_decode(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
    BASE64.decode(data)
}

/// AES-256-GCM 加密
///
/// 使用 256 位密钥和 96 位随机 nonce
/// 返回格式：Base64(nonce + ciphertext + tag)
///
/// # 参数
/// - `plaintext`: 明文数据
/// - `key`: 32 字节密钥
///
/// # 返回
/// Base64 编码的密文（包含 nonce）
pub fn aes_encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("密钥无效: {}", e))?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("加密失败: {}", e))?;
    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    Ok(BASE64.encode(&combined))
}

/// AES-256-GCM 解密
///
/// 从 Base64 编码的密文中提取 nonce 并解密
///
/// # 参数
/// - `ciphertext_b64`: Base64 编码的密文（包含 nonce）
/// - `key`: 32 字节密钥
///
/// # 返回
/// 解密后的明文
pub fn aes_decrypt(ciphertext_b64: &str, key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let combined = BASE64
        .decode(ciphertext_b64)
        .map_err(|e| format!("Base64 解码失败: {}", e))?;
    if combined.len() < 12 {
        return Err("密文数据过短".to_string());
    }
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("密钥无效: {}", e))?;
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("解密失败: {}", e))
}

/// 生成 32 字节随机密钥
pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

/// 生成随机字节
pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut buf = vec![0u8; len];
    OsRng.fill_bytes(&mut buf);
    buf
}

/// 生成随机十六进制字符串
pub fn random_hex(len: usize) -> String {
    let bytes = random_bytes((len + 1) / 2);
    hex::encode(bytes).chars().take(len).collect()
}
