//! CSRF 防护
//!
//! 提供跨站请求伪造（CSRF）令牌的生成与验证
//! 使用密码学安全随机数生成令牌
//! 支持令牌过期机制和可配置的令牌长度

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// CSRF 令牌条目
struct CsrfEntry {
    /// 令牌创建时间戳（秒）
    created_at: u64,
}

/// CSRF 令牌存储
static CSRF_TOKENS: Lazy<RwLock<HashMap<String, CsrfEntry>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

/// 默认令牌过期时间（秒），2 小时
const DEFAULT_TOKEN_TTL: u64 = 7200;

/// 默认令牌长度（字节的十六进制字符数）
const DEFAULT_TOKEN_LENGTH: usize = 32;

/// 获取当前时间戳（秒）
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 生成密码学安全随机令牌
fn secure_random_token(len: usize) -> String {
    crate::security::crypto::random_hex(len)
}

/// 生成 CSRF 令牌
///
/// 使用密码学安全随机数生成器
/// 令牌自动存储并设置过期时间
///
/// # 返回
/// 生成的 CSRF 令牌字符串
pub fn generate_token() -> String {
    generate_token_with_ttl(DEFAULT_TOKEN_TTL)
}

/// 生成带自定义过期时间的 CSRF 令牌
///
/// # 参数
/// - `ttl_seconds`: 令牌有效期（秒）
pub fn generate_token_with_ttl(ttl_seconds: u64) -> String {
    let token = secure_random_token(DEFAULT_TOKEN_LENGTH);
    let entry = CsrfEntry {
        created_at: current_timestamp(),
    };
    let mut tokens = CSRF_TOKENS.write();
    cleanup_expired_tokens(&mut tokens, ttl_seconds);
    tokens.insert(token.clone(), entry);
    token
}

/// 验证 CSRF 令牌
///
/// 验证成功后令牌不会被删除（可重复验证）
/// 过期令牌验证失败
///
/// # 参数
/// - `token`: 待验证的令牌
///
/// # 返回
/// 令牌是否有效
pub fn verify_token(token: &str) -> bool {
    verify_token_with_ttl(token, DEFAULT_TOKEN_TTL)
}

/// 验证 CSRF 令牌（带自定义过期时间）
pub fn verify_token_with_ttl(token: &str, ttl_seconds: u64) -> bool {
    let mut tokens = CSRF_TOKENS.write();
    cleanup_expired_tokens(&mut tokens, ttl_seconds);
    if let Some(entry) = tokens.get(token) {
        let now = current_timestamp();
        now.saturating_sub(entry.created_at) <= ttl_seconds
    } else {
        false
    }
}

/// 消费式验证 CSRF 令牌
///
/// 验证成功后令牌立即删除（一次性令牌）
/// 适用于表单提交等场景
pub fn consume_token(token: &str) -> bool {
    consume_token_with_ttl(token, DEFAULT_TOKEN_TTL)
}

/// 消费式验证 CSRF 令牌（带自定义过期时间）
pub fn consume_token_with_ttl(token: &str, ttl_seconds: u64) -> bool {
    let mut tokens = CSRF_TOKENS.write();
    cleanup_expired_tokens(&mut tokens, ttl_seconds);
    if let Some(entry) = tokens.remove(token) {
        let now = current_timestamp();
        now.saturating_sub(entry.created_at) <= ttl_seconds
    } else {
        false
    }
}

/// 清除过期令牌
fn cleanup_expired_tokens(tokens: &mut HashMap<String, CsrfEntry>, ttl_seconds: u64) {
    let now = current_timestamp();
    tokens.retain(|_, entry| now.saturating_sub(entry.created_at) <= ttl_seconds);
}

/// 清除所有令牌
pub fn clear_all_tokens() {
    let mut tokens = CSRF_TOKENS.write();
    tokens.clear();
}

/// 获取当前存储的令牌数量
pub fn token_count() -> usize {
    let tokens = CSRF_TOKENS.read();
    tokens.len()
}

/// 生成隐藏表单域 HTML
///
/// 生成包含 CSRF 令牌的隐藏 input 标签
/// 适用于传统表单提交
pub fn hidden_field(field_name: &str) -> String {
    let token = generate_token();
    format!(
        r#"<input type="hidden" name="{}" value="{}" />"#,
        field_name, token
    )
}

/// 从请求参数中验证 CSRF 令牌
///
/// # 参数
/// - `params`: 请求参数
/// - `field_name`: CSRF 令牌字段名
///
/// # 返回
/// 令牌是否有效
pub fn verify_request(params: &HashMap<String, String>, field_name: &str) -> bool {
    if let Some(token) = params.get(field_name) {
        consume_token(token)
    } else {
        false
    }
}
