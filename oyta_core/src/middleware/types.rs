//! 中间件类型定义
//!
//! 定义中间件相关的数据结构
//! 包括：中间件定义、中间件类型、中间件执行结果

use serde::{Deserialize, Serialize};

/// 中间件定义
///
/// 描述一个中间件的注册信息
/// 对应 ThinkPHP 8.0 的中间件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareDef {
    /// 中间件名称（用于配置中引用）
    pub name: String,
    /// 中间件类名（含命名空间）
    /// 例如：app\middleware\AuthCheck
    pub class: String,
    /// 中间件参数
    /// 传递给中间件 handle 方法的额外参数
    pub params: Vec<String>,
    /// 中间件类型
    pub kind: MiddlewareKind,
    /// 是否禁用
    pub disabled: bool,
}

/// 中间件类型
///
/// 区分中间件的执行时机
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MiddlewareKind {
    /// 全局中间件
    /// 在所有请求中执行，最先执行
    Global,
    /// 路由中间件
    /// 在匹配到路由后执行
    Route,
    /// 控制器中间件
    /// 在控制器方法执行前执行
    Controller,
}

/// 中间件执行结果
///
/// 中间件 handle 方法执行后的返回值
#[derive(Debug, Clone)]
pub enum MiddlewareResult {
    /// 继续执行下一个中间件
    /// 中间件调用了 $next($request) 传递请求
    Continue,
    /// 中断请求，直接返回响应
    /// 中间件没有调用 $next，直接返回了响应
    Stop(crate::http::response::OytaResponse),
}

/// 中间件优先级
///
/// 用于控制中间件的执行顺序
/// 数值越小优先级越高，越先执行
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MiddlewarePriority {
    /// 最高优先级（认证、安全检查等）
    Highest = 0,
    /// 高优先级
    High = 1,
    /// 中等优先级（默认）
    Medium = 2,
    /// 低优先级
    Low = 3,
    /// 最低优先级（日志、统计等）
    Lowest = 4,
}

impl Default for MiddlewarePriority {
    fn default() -> Self {
        Self::Medium
    }
}
