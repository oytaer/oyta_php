//! CSRF 防护门面
//!
//! 提供静态方法访问 CSRF 防护功能
//! 对应 ThinkPHP 8.0 的 CSRF 验证功能
//!
//! # 使用示例
//! ```php
//! // 获取 CSRF Token
//! $token = Csrf::token();
//!
//! // 验证 Token
//! if (Csrf::verify($token)) {
//!     // 验证通过
//! }
//!
//! // 生成隐藏表单字段
//! echo Csrf::field();
//! ```

use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::crypto;

/// CSRF Token 默认有效期（秒）
const DEFAULT_TOKEN_TTL: u64 = 3600;

/// CSRF Token 默认长度
const DEFAULT_TOKEN_LENGTH: usize = 32;

/// 全局 CSRF 配置和 Token 存储
static CSRF_STATE: RwLock<Option<CsrfState>> = RwLock::new(None);

/// CSRF 状态结构体
///
/// 存储 Token 和配置
struct CsrfState {
    /// Token 存储（token -> 过期时间）
    tokens: HashMap<String, u64>,
    /// Token 有效期（秒）
    token_ttl: u64,
    /// Token 长度
    token_length: usize,
    /// 是否启用
    enabled: bool,
}

impl CsrfState {
    /// 创建新的 CSRF 状态
    fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            token_ttl: DEFAULT_TOKEN_TTL,
            token_length: DEFAULT_TOKEN_LENGTH,
            enabled: true,
        }
    }
}

/// Csrf 门面结构体
///
/// 提供静态方法访问 CSRF 防护功能
/// 所有方法都是线程安全的
pub struct Csrf;

impl Csrf {
    /// 初始化 CSRF 门面
    ///
    /// 必须在使用其他方法前调用
    ///
    /// # 参数
    /// - `token_ttl`: Token 有效期（秒），默认 3600
    /// - `token_length`: Token 长度，默认 32
    pub fn init(token_ttl: Option<u64>, token_length: Option<usize>) {
        // 获取写锁
        let mut guard = CSRF_STATE.write();
        // 创建新的 CSRF 状态
        let mut state = CsrfState::new();
        // 设置 Token 有效期
        if let Some(ttl) = token_ttl {
            state.token_ttl = ttl;
        }
        // 设置 Token 长度
        if let Some(len) = token_length {
            state.token_length = len;
        }
        // 存储到全局静态变量
        *guard = Some(state);
    }

    /// 获取当前时间戳（秒）
    ///
    /// 内部方法，用于计算过期时间
    fn current_timestamp() -> u64 {
        // 获取系统时间
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// 生成 CSRF Token
    ///
    /// 生成密码学安全的随机 Token
    /// 并存储到 Token 列表中
    ///
    /// # 返回
    /// 生成的 CSRF Token 字符串
    ///
    /// # 示例
    /// ```php
    /// $token = Csrf::token();
    /// ```
    pub fn token() -> String {
        // 获取写锁
        let mut guard = CSRF_STATE.write();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_mut() {
            // 生成随机 Token
            let token = crypto::random_hex(state.token_length);
            // 计算过期时间
            let expires_at = Self::current_timestamp() + state.token_ttl;
            // 存储 Token
            state.tokens.insert(token.clone(), expires_at);
            // 返回 Token
            token
        } else {
            // 未初始化时自动初始化并生成 Token
            *guard = Some(CsrfState::new());
            drop(guard);
            // 递归调用
            Self::token()
        }
    }

    /// 验证 CSRF Token
    ///
    /// 检查 Token 是否有效且未过期
    /// 验证成功后可选择是否使 Token 失效
    ///
    /// # 参数
    /// - `token`: 待验证的 Token
    /// - `invalidate`: 验证成功后是否使 Token 失效，默认为 true
    ///
    /// # 返回
    /// 如果 Token 有效返回 true，否则返回 false
    ///
    /// # 示例
    /// ```php
    /// if (Csrf::verify($token)) {
    ///     // 验证通过
    /// }
    /// ```
    pub fn verify(token: &str, invalidate: bool) -> bool {
        // 获取写锁
        let mut guard = CSRF_STATE.write();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_mut() {
            // 获取当前时间戳
            let now = Self::current_timestamp();
            // 查找 Token
            if let Some(&expires_at) = state.tokens.get(token) {
                // 检查是否过期
                if expires_at > now {
                    // Token 有效
                    if invalidate {
                        // 使 Token 失效
                        state.tokens.remove(token);
                    }
                    // 返回验证成功
                    return true;
                }
            }
            // Token 无效或已过期
            false
        } else {
            // 未初始化，验证失败
            false
        }
    }

    /// 验证 Token 但不使其失效
    ///
    /// 允许同一 Token 多次验证
    ///
    /// # 参数
    /// - `token`: 待验证的 Token
    ///
    /// # 返回
    /// 如果 Token 有效返回 true
    pub fn check(token: &str) -> bool {
        // 调用 verify 方法，不使 Token 失效
        Self::verify(token, false)
    }

    /// 生成隐藏表单字段
    ///
    /// 生成包含 CSRF Token 的隐藏 input 字段
    /// 用于表单提交
    ///
    /// # 返回
    /// HTML 隐藏 input 字段字符串
    ///
    /// # 示例
    /// ```php
    /// <form method="POST">
    ///     <?= Csrf::field() ?>
    ///     <!-- 其他表单字段 -->
    /// </form>
    /// ```
    pub fn field() -> String {
        // 生成 Token
        let token = Self::token();
        // 返回 HTML 隐藏字段
        format!("<input type=\"hidden\" name=\"_token\" value=\"{}\" />", token)
    }

    /// 生成 meta 标签
    ///
    /// 生成包含 CSRF Token 的 meta 标签
    /// 用于 AJAX 请求
    ///
    /// # 返回
    /// HTML meta 标签字符串
    ///
    /// # 示例
    /// ```php
    /// <head>
    ///     <?= Csrf::meta() ?>
    /// </head>
    /// ```
    pub fn meta() -> String {
        // 生成 Token
        let token = Self::token();
        // 返回 HTML meta 标签
        format!("<meta name=\"csrf-token\" content=\"{}\" />", token)
    }

    /// 使 Token 失效
    ///
    /// 手动使指定 Token 失效
    ///
    /// # 参数
    /// - `token`: 要失效的 Token
    pub fn invalidate(token: &str) {
        // 获取写锁
        let mut guard = CSRF_STATE.write();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_mut() {
            // 移除 Token
            state.tokens.remove(token);
        }
    }

    /// 清理过期 Token
    ///
    /// 移除所有已过期的 Token
    /// 建议定期调用以释放内存
    pub fn gc() {
        // 获取写锁
        let mut guard = CSRF_STATE.write();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_mut() {
            // 获取当前时间戳
            let now = Self::current_timestamp();
            // 保留未过期的 Token
            state.tokens.retain(|_, &mut expires_at| expires_at > now);
        }
    }

    /// 获取有效 Token 数量
    ///
    /// 返回当前存储的有效 Token 数量
    ///
    /// # 返回
    /// 有效 Token 数量
    pub fn count() -> usize {
        // 获取读锁
        let guard = CSRF_STATE.read();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_ref() {
            // 返回 Token 数量
            state.tokens.len()
        } else {
            // 未初始化返回 0
            0
        }
    }

    /// 清除所有 Token
    ///
    /// 删除所有已存储的 Token
    /// 主要用于测试环境
    pub fn clear() {
        // 获取写锁
        let mut guard = CSRF_STATE.write();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_mut() {
            // 清空 Token 列表
            state.tokens.clear();
        }
    }

    /// 启用 CSRF 防护
    pub fn enable() {
        // 获取写锁
        let mut guard = CSRF_STATE.write();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_mut() {
            // 设置启用标志
            state.enabled = true;
        }
    }

    /// 禁用 CSRF 防护
    ///
    /// 仅用于特殊场景，不建议在生产环境禁用
    pub fn disable() {
        // 获取写锁
        let mut guard = CSRF_STATE.write();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_mut() {
            // 设置禁用标志
            state.enabled = false;
        }
    }

    /// 检查 CSRF 防护是否启用
    ///
    /// # 返回
    /// 如果启用返回 true
    pub fn is_enabled() -> bool {
        // 获取读锁
        let guard = CSRF_STATE.read();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_ref() {
            // 返回启用状态
            state.enabled
        } else {
            // 未初始化时默认启用
            true
        }
    }

    /// 检查 CSRF 门面是否已初始化
    ///
    /// # 返回
    /// 如果已初始化返回 true
    pub fn is_initialized() -> bool {
        // 获取读锁检查是否有值
        let guard = CSRF_STATE.read();
        guard.is_some()
    }

    /// 重置 CSRF 门面
    ///
    /// 清除所有状态
    /// 主要用于测试环境
    pub fn reset() {
        // 获取写锁
        let mut guard = CSRF_STATE.write();
        // 清空状态
        *guard = None;
    }
}
