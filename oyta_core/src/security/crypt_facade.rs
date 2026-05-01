//! 加密解密门面
//!
//! 提供静态方法访问加密解密功能
//! 对应 ThinkPHP 8.0 的 Crypt 类
//!
//! # 使用示例
//! ```php
//! // 加密数据
//! $encrypted = Crypt::encrypt('敏感数据');
//!
//! // 解密数据
//! $decrypted = Crypt::decrypt($encrypted);
//!
//! // 生成密钥
//! $key = Crypt::generateKey();
//! ```

use std::sync::Arc;
use parking_lot::RwLock;

use super::crypto;

/// 默认密钥长度（字节）
const DEFAULT_KEY_LENGTH: usize = 32;

/// 全局加密密钥
///
/// 使用 RwLock 实现线程安全的存储
static ENCRYPTION_KEY: RwLock<Option<Arc<[u8; 32]>>> = RwLock::new(None);

/// Crypt 门面结构体
///
/// 提供静态方法访问加密解密功能
/// 所有方法都是线程安全的
pub struct Crypt;

impl Crypt {
    /// 初始化加密门面
    ///
    /// 设置加密密钥，必须在使用其他方法前调用
    ///
    /// # 参数
    /// - `key`: 32 字节的加密密钥
    pub fn init(key: [u8; 32]) {
        // 获取写锁
        let mut guard = ENCRYPTION_KEY.write();
        // 存储密钥
        *guard = Some(Arc::new(key));
    }

    /// 从字符串初始化加密门面
    ///
    /// 使用 PBKDF2 密钥派生函数将任意长度字符串转换为安全的 32 字节密钥
    /// 这比简单的填充更安全，可以防止弱密钥攻击
    ///
    /// # 参数
    /// - `key_str`: 密钥字符串（任意长度）
    /// - `salt`: 盐值（可选，建议提供以增加安全性）
    ///
    /// # 安全性
    /// 使用 PBKDF2-HMAC-SHA256 进行密钥派生，迭代次数 100,000
    pub fn init_from_string(key_str: &str, salt: Option<&str>) {
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;
        
        // 使用 PBKDF2 派生密钥
        let salt_bytes = salt
            .map(|s| s.as_bytes().to_vec())
            .unwrap_or_else(|| b"oyta_crypt_default_salt_2024".to_vec());
        
        // 派生 32 字节密钥
        let mut key = [0u8; 32];
        
        // 使用 pbkdf2 库的实现
        pbkdf2_hmac::<Sha256>(key_str.as_bytes(), &salt_bytes, 100_000, &mut key);
        
        // 初始化
        Self::init(key);
    }
    
    /// 从字符串初始化加密门面（简单版本，向后兼容）
    ///
    /// # 警告
    /// 此方法安全性较低，建议使用 init_from_string(key, Some(salt))
    ///
    /// # 参数
    /// - `key_str`: 密钥字符串
    #[deprecated(since = "2.2.0", note = "请使用 init_from_string(key, Some(salt)) 以获得更好的安全性")]
    pub fn init_from_string_simple(key_str: &str) {
        // 使用默认盐值派生密钥
        Self::init_from_string(key_str, Some("oyta_default_salt"));
    }

    /// 从十六进制字符串初始化加密门面
    ///
    /// 解析 64 字符的十六进制字符串为 32 字节密钥
    ///
    /// # 参数
    /// - `hex_key`: 64 字符的十六进制字符串
    ///
    /// # 返回
    /// 初始化成功返回 Ok(())，失败返回错误信息
    pub fn init_from_hex(hex_key: &str) -> Result<(), String> {
        // 检查长度是否为 64 字符（32 字节的十六进制表示）
        if hex_key.len() != 64 {
            return Err("十六进制密钥长度必须为 64 字符".to_string());
        }
        // 解析十六进制字符串
        let bytes = hex::decode(hex_key)
            .map_err(|e| format!("十六进制解析失败: {}", e))?;
        // 创建固定大小数组
        let mut key = [0u8; 32];
        // 复制字节
        key.copy_from_slice(&bytes);
        // 初始化
        Self::init(key);
        // 返回成功
        Ok(())
    }

    /// 获取当前加密密钥
    ///
    /// 内部方法，用于获取密钥引用
    ///
    /// # 返回
    /// 密钥的可选引用
    fn get_key() -> Option<Arc<[u8; 32]>> {
        // 获取读锁
        let guard = ENCRYPTION_KEY.read();
        // 克隆密钥引用
        guard.clone()
    }

    /// 加密数据
    ///
    /// 使用 AES-256-GCM 算法加密数据
    /// 返回 Base64 编码的密文
    ///
    /// # 参数
    /// - `plaintext`: 明文数据
    ///
    /// # 返回
    /// Base64 编码的密文，包含 nonce 和认证标签
    ///
    /// # 示例
    /// ```php
    /// $encrypted = Crypt::encrypt('敏感数据');
    /// ```
    pub fn encrypt(plaintext: &str) -> Result<String, String> {
        // 获取密钥
        if let Some(key) = Self::get_key() {
            // 调用加密函数
            crypto::aes_encrypt(plaintext.as_bytes(), &key)
        } else {
            // 密钥未初始化
            Err("加密密钥未初始化，请先调用 Crypt::init()".to_string())
        }
    }

    /// 加密字节数据
    ///
    /// 使用 AES-256-GCM 算法加密字节数据
    ///
    /// # 参数
    /// - `data`: 字节数据
    ///
    /// # 返回
    /// Base64 编码的密文
    pub fn encrypt_bytes(data: &[u8]) -> Result<String, String> {
        // 获取密钥
        if let Some(key) = Self::get_key() {
            // 调用加密函数
            crypto::aes_encrypt(data, &key)
        } else {
            // 密钥未初始化
            Err("加密密钥未初始化，请先调用 Crypt::init()".to_string())
        }
    }

    /// 解密数据
    ///
    /// 解密 Base64 编码的密文
    ///
    /// # 参数
    /// - `ciphertext`: Base64 编码的密文
    ///
    /// # 返回
    /// 解密后的明文字符串
    ///
    /// # 示例
    /// ```php
    /// $decrypted = Crypt::decrypt($encrypted);
    /// ```
    pub fn decrypt(ciphertext: &str) -> Result<String, String> {
        // 获取密钥
        if let Some(key) = Self::get_key() {
            // 调用解密函数
            let bytes = crypto::aes_decrypt(ciphertext, &key)?;
            // 转换为字符串
            String::from_utf8(bytes)
                .map_err(|e| format!("解密数据不是有效的 UTF-8: {}", e))
        } else {
            // 密钥未初始化
            Err("加密密钥未初始化，请先调用 Crypt::init()".to_string())
        }
    }

    /// 解密为字节数据
    ///
    /// 解密并返回原始字节数据
    ///
    /// # 参数
    /// - `ciphertext`: Base64 编码的密文
    ///
    /// # 返回
    /// 解密后的字节数据
    pub fn decrypt_bytes(ciphertext: &str) -> Result<Vec<u8>, String> {
        // 获取密钥
        if let Some(key) = Self::get_key() {
            // 调用解密函数
            crypto::aes_decrypt(ciphertext, &key)
        } else {
            // 密钥未初始化
            Err("加密密钥未初始化，请先调用 Crypt::init()".to_string())
        }
    }

    /// 生成随机密钥
    ///
    /// 生成 32 字节的密码学安全随机密钥
    /// 用于初始化加密门面
    ///
    /// # 返回
    /// 32 字节的随机密钥
    ///
    /// # 示例
    /// ```php
    /// $key = Crypt::generateKey();
    /// Crypt::init($key);
    /// ```
    pub fn generate_key() -> [u8; 32] {
        // 调用加密模块的密钥生成函数
        crypto::generate_key()
    }

    /// 生成十六进制格式的密钥
    ///
    /// 生成 64 字符的十六进制字符串
    /// 便于存储和配置
    ///
    /// # 返回
    /// 64 字符的十六进制密钥字符串
    pub fn generate_key_hex() -> String {
        // 生成密钥
        let key = Self::generate_key();
        // 转换为十六进制字符串
        hex::encode(key)
    }

    /// Base64 编码
    ///
    /// 将字节数据编码为 Base64 字符串
    ///
    /// # 参数
    /// - `data`: 待编码的数据
    ///
    /// # 返回
    /// Base64 编码字符串
    pub fn base64_encode(data: &[u8]) -> String {
        // 调用加密模块的 Base64 编码函数
        crypto::base64_encode(data)
    }

    /// Base64 解码
    ///
    /// 将 Base64 字符串解码为字节数据
    ///
    /// # 参数
    /// - `data`: Base64 编码字符串
    ///
    /// # 返回
    /// 解码后的字节数据
    pub fn base64_decode(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
        // 调用加密模块的 Base64 解码函数
        crypto::base64_decode(data)
    }

    /// 生成随机字节
    ///
    /// 生成指定长度的密码学安全随机字节
    ///
    /// # 参数
    /// - `len`: 字节长度
    ///
    /// # 返回
    /// 随机字节数组
    pub fn random_bytes(len: usize) -> Vec<u8> {
        // 调用加密模块的随机字节生成函数
        crypto::random_bytes(len)
    }

    /// 生成随机十六进制字符串
    ///
    /// 生成指定长度的随机十六进制字符串
    ///
    /// # 参数
    /// - `len`: 字符串长度
    ///
    /// # 返回
    /// 随机十六进制字符串
    pub fn random_hex(len: usize) -> String {
        // 调用加密模块的随机十六进制生成函数
        crypto::random_hex(len)
    }

    /// 检查加密门面是否已初始化
    ///
    /// 用于判断是否需要调用 init 方法
    ///
    /// # 返回
    /// 如果已初始化返回 true，否则返回 false
    pub fn is_initialized() -> bool {
        // 获取读锁检查是否有值
        let guard = ENCRYPTION_KEY.read();
        guard.is_some()
    }

    /// 重置加密门面
    ///
    /// 清除加密密钥
    /// 主要用于测试环境
    pub fn reset() {
        // 获取写锁
        let mut guard = ENCRYPTION_KEY.write();
        // 清空密钥
        *guard = None;
    }

    /// 获取当前密钥的十六进制表示
    ///
    /// 用于调试或备份密钥
    ///
    /// # 返回
    /// 密钥的十六进制字符串，如果未初始化返回 None
    pub fn get_key_hex() -> Option<String> {
        // 获取密钥
        if let Some(key) = Self::get_key() {
            // 转换为十六进制字符串
            Some(hex::encode(key.as_ref()))
        } else {
            // 未初始化
            None
        }
    }
}
