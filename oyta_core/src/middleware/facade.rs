//! 中间件门面
//!
//! 提供静态方法访问中间件管理功能
//! 对应 ThinkPHP 8.0 的中间件管理功能
//!
//! # 使用示例
//! ```php
//! // 添加全局中间件
//! Middleware::add('app\middleware\Auth');
//!
//! // 获取全局中间件列表
//! $middlewares = Middleware::getGlobal();
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

use super::types::{MiddlewareDef, MiddlewareKind};

/// 全局中间件配置
static MIDDLEWARE_CONFIG: RwLock<Option<MiddlewareState>> = RwLock::new(None);

/// 中间件状态结构体
struct MiddlewareState {
    /// 全局中间件列表
    global: Vec<String>,
    /// 路由中间件映射
    route: HashMap<String, Vec<String>>,
    /// 中间件别名映射
    aliases: HashMap<String, String>,
    /// 中间件分组
    groups: HashMap<String, Vec<String>>,
    /// 排除中间件的路由
    except: Vec<String>,
}

impl MiddlewareState {
    /// 创建新的中间件状态
    fn new() -> Self {
        Self {
            global: Vec::new(),
            route: HashMap::new(),
            aliases: HashMap::new(),
            groups: HashMap::new(),
            except: Vec::new(),
        }
    }
}

/// Middleware 门面结构体
///
/// 提供静态方法访问中间件管理功能
/// 所有方法都是线程安全的
pub struct Middleware;

impl Middleware {
    /// 初始化中间件门面
    ///
    /// 必须在使用其他方法前调用
    pub fn init() {
        // 获取写锁
        let mut guard = MIDDLEWARE_CONFIG.write();
        // 创建新的中间件状态
        *guard = Some(MiddlewareState::new());
    }

    /// 获取中间件状态
    fn get_state() -> parking_lot::RwLockReadGuard<'static, Option<MiddlewareState>> {
        // 获取读锁
        MIDDLEWARE_CONFIG.read()
    }

    /// 获取可变中间件状态
    fn get_state_mut() -> parking_lot::RwLockWriteGuard<'static, Option<MiddlewareState>> {
        // 获取写锁
        MIDDLEWARE_CONFIG.write()
    }

    /// 添加全局中间件
    ///
    /// 将中间件添加到全局中间件列表
    /// 全局中间件会在每个请求中执行
    ///
    /// # 参数
    /// - `middleware`: 中间件类名（含命名空间）
    ///
    /// # 示例
    /// ```php
    /// Middleware::add('app\middleware\Auth');
    /// ```
    pub fn add(middleware: &str) {
        // 获取写锁
        let mut guard = Self::get_state_mut();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_mut() {
            // 添加到全局中间件列表
            state.global.push(middleware.to_string());
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Middleware 门面未初始化，无法添加中间件: {}", middleware);
        }
    }

    /// 批量添加全局中间件
    ///
    /// 一次性添加多个全局中间件
    ///
    /// # 参数
    /// - `middlewares`: 中间件类名列表
    pub fn add_many(middlewares: &[&str]) {
        // 遍历并添加每个中间件
        for m in middlewares {
            Self::add(m);
        }
    }

    /// 获取全局中间件列表
    ///
    /// 返回所有已注册的全局中间件
    ///
    /// # 返回
    /// 中间件类名列表
    pub fn get_global() -> Vec<String> {
        // 获取读锁
        let guard = Self::get_state();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_ref() {
            // 返回全局中间件列表
            state.global.clone()
        } else {
            // 未初始化时返回空列表
            Vec::new()
        }
    }

    /// 注册路由中间件
    ///
    /// 为指定路由注册中间件
    ///
    /// # 参数
    /// - `route`: 路由名称或模式
    /// - `middleware`: 中间件类名
    ///
    /// # 示例
    /// ```php
    /// Middleware::route('admin/*', 'app\middleware\AdminAuth');
    /// ```
    pub fn route(route: &str, middleware: &str) {
        // 获取写锁
        let mut guard = Self::get_state_mut();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_mut() {
            // 获取或创建路由中间件列表
            let list = state.route.entry(route.to_string()).or_default();
            // 添加中间件
            list.push(middleware.to_string());
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Middleware 门面未初始化，无法注册路由中间件: {} -> {}", route, middleware);
        }
    }

    /// 获取路由中间件
    ///
    /// 返回指定路由的中间件列表
    ///
    /// # 参数
    /// - `route`: 路由名称或模式
    ///
    /// # 返回
    /// 中间件类名列表
    pub fn get_route(route: &str) -> Vec<String> {
        // 获取读锁
        let guard = Self::get_state();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_ref() {
            // 查找路由中间件
            state.route.get(route).cloned().unwrap_or_default()
        } else {
            // 未初始化时返回空列表
            Vec::new()
        }
    }

    /// 注册中间件别名
    ///
    /// 为中间件类定义短别名
    ///
    /// # 参数
    /// - `alias`: 别名
    /// - `class`: 中间件类名
    ///
    /// # 示例
    /// ```php
    /// Middleware::alias('auth', 'app\middleware\Auth');
    /// // 使用别名
    /// Middleware::add('auth');
    /// ```
    pub fn alias(alias: &str, class: &str) {
        // 获取写锁
        let mut guard = Self::get_state_mut();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_mut() {
            // 添加别名映射
            state.aliases.insert(alias.to_string(), class.to_string());
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Middleware 门面未初始化，无法注册别名: {} -> {}", alias, class);
        }
    }

    /// 解析中间件名称
    ///
    /// 将别名解析为完整类名
    ///
    /// # 参数
    /// - `name`: 中间件名称或别名
    ///
    /// # 返回
    /// 完整的中间件类名
    pub fn resolve(name: &str) -> String {
        // 获取读锁
        let guard = Self::get_state();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_ref() {
            // 尝试从别名映射中查找
            if let Some(class) = state.aliases.get(name) {
                // 返回完整类名
                return class.clone();
            }
        }
        // 未找到别名，返回原名称
        name.to_string()
    }

    /// 创建中间件分组
    ///
    /// 将多个中间件组织成一个分组
    ///
    /// # 参数
    /// - `name`: 分组名称
    /// - `middlewares`: 中间件列表
    ///
    /// # 示例
    /// ```php
    /// Middleware::group('web', ['auth', 'csrf', 'session']);
    /// // 使用分组
    /// Middleware::add('web');
    /// ```
    pub fn group(name: &str, middlewares: Vec<String>) {
        // 获取写锁
        let mut guard = Self::get_state_mut();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_mut() {
            // 添加分组
            state.groups.insert(name.to_string(), middlewares);
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Middleware 门面未初始化，无法创建分组: {}", name);
        }
    }

    /// 获取分组中间件
    ///
    /// 返回指定分组的中间件列表
    ///
    /// # 参数
    /// - `name`: 分组名称
    ///
    /// # 返回
    /// 中间件列表
    pub fn get_group(name: &str) -> Vec<String> {
        // 获取读锁
        let guard = Self::get_state();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_ref() {
            // 查找分组
            state.groups.get(name).cloned().unwrap_or_default()
        } else {
            // 未初始化时返回空列表
            Vec::new()
        }
    }

    /// 排除路由
    ///
    /// 将路由添加到中间件排除列表
    /// 排除的路由不会执行全局中间件
    ///
    /// # 参数
    /// - `route`: 路由名称或模式
    ///
    /// # 示例
    /// ```php
    /// Middleware::except('login');
    /// Middleware::except('api/*');
    /// ```
    pub fn except(route: &str) {
        // 获取写锁
        let mut guard = Self::get_state_mut();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_mut() {
            // 添加到排除列表
            state.except.push(route.to_string());
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("Middleware 门面未初始化，无法排除路由: {}", route);
        }
    }

    /// 检查路由是否被排除
    ///
    /// 判断指定路由是否在排除列表中
    ///
    /// # 参数
    /// - `route`: 路由名称或模式
    ///
    /// # 返回
    /// 如果被排除返回 true
    pub fn is_excepted(route: &str) -> bool {
        // 获取读锁
        let guard = Self::get_state();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_ref() {
            // 检查是否在排除列表中
            state.except.iter().any(|r| {
                // 精确匹配
                r == route ||
                // 通配符匹配
                (r.ends_with('*') && route.starts_with(&r[..r.len()-1]))
            })
        } else {
            // 未初始化时返回 false
            false
        }
    }

    /// 移除中间件
    ///
    /// 从全局中间件列表中移除指定中间件
    ///
    /// # 参数
    /// - `middleware`: 中间件名称或别名
    pub fn remove(middleware: &str) {
        // 获取写锁
        let mut guard = Self::get_state_mut();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_mut() {
            // 解析中间件名称
            let resolved = Self::resolve(middleware);
            // 从全局列表中移除
            state.global.retain(|m| m != &resolved && m != middleware);
        }
    }

    /// 清空全局中间件
    ///
    /// 删除所有全局中间件
    pub fn clear_global() {
        // 获取写锁
        let mut guard = Self::get_state_mut();
        // 检查状态是否已初始化
        if let Some(state) = guard.as_mut() {
            // 清空全局中间件列表
            state.global.clear();
        }
    }

    /// 检查中间件门面是否已初始化
    ///
    /// # 返回
    /// 如果已初始化返回 true
    pub fn is_initialized() -> bool {
        // 获取读锁检查是否有值
        let guard = MIDDLEWARE_CONFIG.read();
        guard.is_some()
    }

    /// 重置中间件门面
    ///
    /// 清除所有中间件配置
    /// 主要用于测试环境
    pub fn reset() {
        // 获取写锁
        let mut guard = MIDDLEWARE_CONFIG.write();
        // 清空状态
        *guard = None;
    }
}
