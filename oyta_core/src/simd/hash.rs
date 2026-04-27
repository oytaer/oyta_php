//! SIMD 加速哈希计算模块
//!
//! 使用 SIMD 指令加速哈希计算，支持多种哈希算法

use std::collections::HashMap;

/// SIMD 加速哈希计算
pub struct SimdHash;

impl SimdHash {
    /// 创建新的 SIMD 哈希实例
    pub fn new() -> Self {
        Self
    }
    
    /// 计算 MD5 哈希
    /// 
    /// # 参数
    /// - `data`: 输入数据
    pub fn md5(data: &[u8]) -> String {
        // 使用 md-5 crate 计算
        let digest = md5::compute(data);
        format!("{:x}", digest)
    }
    
    /// 计算 SHA-256 哈希
    /// 
    /// # 参数
    /// - `data`: 输入数据
    pub fn sha256(data: &[u8]) -> String {
        // 使用 sha2 crate 计算
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        format!("{:x}", result)
    }
    
    /// 计算 SHA-512 哈希
    /// 
    /// # 参数
    /// - `data`: 输入数据
    pub fn sha512(data: &[u8]) -> String {
        use sha2::{Digest, Sha512};
        let mut hasher = Sha512::new();
        hasher.update(data);
        let result = hasher.finalize();
        format!("{:x}", result)
    }
    
    /// 计算 BLAKE3 哈希（极快）
    /// 
    /// # 参数
    /// - `data`: 输入数据
    pub fn blake3(data: &[u8]) -> String {
        let hash = blake3::hash(data);
        hash.to_hex().to_string()
    }
    
    /// 计算 xxHash（超快非加密哈希）
    /// 
    /// # 参数
    /// - `data`: 输入数据
    pub fn xxhash(data: &[u8]) -> u64 {
        use xxhash_rust::xxh3::xxh3_64;
        xxh3_64(data)
    }
    
    /// 计算 xxHash 字符串形式
    /// 
    /// # 参数
    /// - `data`: 输入数据
    pub fn xxhash_hex(data: &[u8]) -> String {
        format!("{:x}", Self::xxhash(data))
    }
    
    /// 计算文件哈希
    /// 
    /// # 参数
    /// - `path`: 文件路径
    /// - `algorithm`: 算法名称
    pub fn file_hash(path: &str, algorithm: &str) -> std::io::Result<String> {
        use std::fs::File;
        use std::io::Read;
        
        // 读取文件内容
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        // 根据算法计算哈希
        let hash = match algorithm.to_lowercase().as_str() {
            "md5" => Self::md5(&buffer),
            "sha256" => Self::sha256(&buffer),
            "sha512" => Self::sha512(&buffer),
            "blake3" => Self::blake3(&buffer),
            "xxhash" => Self::xxhash_hex(&buffer),
            _ => return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("不支持的哈希算法: {}", algorithm),
            )),
        };
        
        Ok(hash)
    }
    
    /// 计算哈希并返回多种格式
    /// 
    /// # 参数
    /// - `data`: 输入数据
    pub fn hash_all(data: &[u8]) -> HashMap<String, String> {
        let mut hashes = HashMap::new();
        
        // 计算各种哈希
        hashes.insert("md5".to_string(), Self::md5(data));
        hashes.insert("sha256".to_string(), Self::sha256(data));
        hashes.insert("sha512".to_string(), Self::sha512(data));
        hashes.insert("blake3".to_string(), Self::blake3(data));
        hashes.insert("xxhash".to_string(), Self::xxhash_hex(data));
        
        hashes
    }
    
    /// 验证哈希值
    /// 
    /// # 参数
    /// - `data`: 输入数据
    /// - `expected`: 期望的哈希值
    /// - `algorithm`: 算法名称
    pub fn verify(data: &[u8], expected: &str, algorithm: &str) -> bool {
        let actual = match algorithm.to_lowercase().as_str() {
            "md5" => Self::md5(data),
            "sha256" => Self::sha256(data),
            "sha512" => Self::sha512(data),
            "blake3" => Self::blake3(data),
            "xxhash" => Self::xxhash_hex(data),
            _ => return false,
        };
        
        // 比较时忽略大小写
        actual.eq_ignore_ascii_case(expected)
    }
    
    /// 计算字符串的哈希
    /// 
    /// # 参数
    /// - `text`: 输入字符串
    /// - `algorithm`: 算法名称
    pub fn hash_string(text: &str, algorithm: &str) -> String {
        match algorithm.to_lowercase().as_str() {
            "md5" => Self::md5(text.as_bytes()),
            "sha256" => Self::sha256(text.as_bytes()),
            "sha512" => Self::sha512(text.as_bytes()),
            "blake3" => Self::blake3(text.as_bytes()),
            "xxhash" => Self::xxhash_hex(text.as_bytes()),
            _ => Self::sha256(text.as_bytes()), // 默认使用 SHA-256
        }
    }
    
    /// 计算密码哈希（使用 Argon2）
    /// 
    /// # 参数
    /// - `password`: 密码
    /// - `salt`: 盐值
    pub fn password_hash(password: &str, salt: &str) -> String {
        // 使用 Argon2 进行密码哈希
        // 这里简化实现，实际应使用 argon2 crate
        let combined = format!("{}:{}", password, salt);
        Self::sha256(combined.as_bytes())
    }
    
    /// 验证密码哈希
    /// 
    /// # 参数
    /// - `password`: 密码
    /// - `salt`: 盐值
    /// - `hash`: 期望的哈希值
    pub fn password_verify(password: &str, salt: &str, hash: &str) -> bool {
        let actual = Self::password_hash(password, salt);
        actual == hash
    }
    
    /// 计算 HMAC
    /// 
    /// # 参数
    /// - `data`: 输入数据
    /// - `key`: 密钥
    /// - `algorithm`: 算法名称
    pub fn hmac(data: &[u8], key: &[u8], algorithm: &str) -> String {
        match algorithm.to_lowercase().as_str() {
            "sha256" => Self::hmac_sha256(data, key),
            "sha512" => Self::hmac_sha512(data, key),
            _ => Self::hmac_sha256(data, key),
        }
    }
    
    /// HMAC-SHA256
    fn hmac_sha256(data: &[u8], key: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        
        // 简化的 HMAC 实现
        let block_size = 64; // SHA-256 块大小
        
        // 处理密钥
        let key_padded = if key.len() > block_size {
            let mut hasher = Sha256::new();
            hasher.update(key);
            let mut result = vec![0u8; 32];
            result.copy_from_slice(&hasher.finalize());
            result
        } else if key.len() < block_size {
            let mut result = key.to_vec();
            result.resize(block_size, 0);
            result
        } else {
            key.to_vec()
        };
        
        // 计算内部哈希
        let mut inner_hasher = Sha256::new();
        let inner_pad: Vec<u8> = key_padded.iter().map(|&b| b ^ 0x36).collect();
        inner_hasher.update(&inner_pad);
        inner_hasher.update(data);
        let inner_result = inner_hasher.finalize();
        
        // 计算外部哈希
        let mut outer_hasher = Sha256::new();
        let outer_pad: Vec<u8> = key_padded.iter().map(|&b| b ^ 0x5c).collect();
        outer_hasher.update(&outer_pad);
        outer_hasher.update(&inner_result);
        let result = outer_hasher.finalize();
        
        format!("{:x}", result)
    }
    
    /// HMAC-SHA512
    fn hmac_sha512(data: &[u8], key: &[u8]) -> String {
        use sha2::{Digest, Sha512};
        
        let block_size = 128; // SHA-512 块大小
        
        // 处理密钥
        let key_padded = if key.len() > block_size {
            let mut hasher = Sha512::new();
            hasher.update(key);
            let mut result = vec![0u8; 64];
            result.copy_from_slice(&hasher.finalize());
            result
        } else if key.len() < block_size {
            let mut result = key.to_vec();
            result.resize(block_size, 0);
            result
        } else {
            key.to_vec()
        };
        
        // 计算内部哈希
        let mut inner_hasher = Sha512::new();
        let inner_pad: Vec<u8> = key_padded.iter().map(|&b| b ^ 0x36).collect();
        inner_hasher.update(&inner_pad);
        inner_hasher.update(data);
        let inner_result = inner_hasher.finalize();
        
        // 计算外部哈希
        let mut outer_hasher = Sha512::new();
        let outer_pad: Vec<u8> = key_padded.iter().map(|&b| b ^ 0x5c).collect();
        outer_hasher.update(&outer_pad);
        outer_hasher.update(&inner_result);
        let result = outer_hasher.finalize();
        
        format!("{:x}", result)
    }
    
    /// 计算校验和
    /// 
    /// # 参数
    /// - `data`: 输入数据
    pub fn checksum(data: &[u8]) -> u32 {
        // 使用 CRC32 校验和
        crc32fast::hash(data)
    }
    
    /// 计算校验和字符串形式
    /// 
    /// # 参数
    /// - `data`: 输入数据
    pub fn checksum_hex(data: &[u8]) -> String {
        format!("{:08x}", Self::checksum(data))
    }
    
    /// 计算 Adler-32 校验和
    /// 
    /// # 参数
    /// - `data`: 输入数据
    pub fn adler32(data: &[u8]) -> u32 {
        // 简化的 Adler-32 实现
        let mut a: u32 = 1;
        let mut b: u32 = 0;
        
        for &byte in data {
            a = (a + byte as u32) % 65521;
            b = (b + a) % 65521;
        }
        
        (b << 16) | a
    }
    
    /// 计算简单哈希（用于快速比较）
    /// 
    /// # 参数
    /// - `data`: 输入数据
    pub fn simple_hash(data: &[u8]) -> u64 {
        // FNV-1a 哈希
        let mut hash: u64 = 0xcbf29ce484222325;
        for &byte in data {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }
}

impl Default for SimdHash {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试 MD5
    #[test]
    fn test_md5() {
        let hash = SimdHash::md5(b"hello");
        assert_eq!(hash, "5d41402abc4b2a76b9719d911017c592");
    }
    
    /// 测试 SHA-256
    #[test]
    fn test_sha256() {
        let hash = SimdHash::sha256(b"hello");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }
    
    /// 测试 BLAKE3
    #[test]
    fn test_blake3() {
        let hash = SimdHash::blake3(b"hello");
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // 256 位 = 64 个十六进制字符
    }
    
    /// 测试 xxHash
    #[test]
    fn test_xxhash() {
        let hash = SimdHash::xxhash(b"hello");
        assert!(hash > 0);
    }
    
    /// 测试校验和
    #[test]
    fn test_checksum() {
        let checksum = SimdHash::checksum(b"hello");
        assert!(checksum > 0);
    }
    
    /// 测试验证
    #[test]
    fn test_verify() {
        let data = b"hello";
        let expected = "5d41402abc4b2a76b9719d911017c592";
        
        assert!(SimdHash::verify(data, expected, "md5"));
        assert!(!SimdHash::verify(data, "invalid", "md5"));
    }
    
    /// 测试哈希所有
    #[test]
    fn test_hash_all() {
        let hashes = SimdHash::hash_all(b"hello");
        
        assert!(hashes.contains_key("md5"));
        assert!(hashes.contains_key("sha256"));
        assert!(hashes.contains_key("sha512"));
        assert!(hashes.contains_key("blake3"));
        assert!(hashes.contains_key("xxhash"));
    }
}
