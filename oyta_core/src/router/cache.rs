//! 路由缓存模块
//!
//! 提供路由的序列化缓存功能
//! 在生产环境中，路由解析结果可以缓存到磁盘
//! 避免每次启动都重新扫描和解析路由文件
//! 缓存文件存储在 runtime/ 目录下

use anyhow::Result;
use std::path::Path;

use super::types::{HttpMethod, Route, RouteGroup, RouteHandler};

/// 路由缓存数据结构
/// 包含所有路由和路由组的序列化数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RouteCache {
    /// 缓存版本号
    /// 当路由系统升级时递增，使旧缓存自动失效
    pub version: u32,
    /// 缓存创建时间戳（秒）
    pub created_at: u64,
    /// 缓存的项目根目录（用于验证缓存有效性）
    pub project_root: String,
    /// 缓存的路由列表
    pub routes: Vec<SerializableRoute>,
    /// 缓存的路由组列表
    pub groups: Vec<RouteGroup>,
    /// 缓存的 MISS 路由
    pub miss_route: Option<SerializableRoute>,
}

/// 可序列化的路由项
/// 将 Route 结构体转换为可序列化的格式
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SerializableRoute {
    /// 路由 ID
    pub id: String,
    /// HTTP 方法
    pub method: String,
    /// 路由路径
    pub path: String,
    /// 处理器类型标识
    pub handler_type: String,
    /// 控制器名（ControllerMethod 类型时使用）
    pub controller: Option<String>,
    /// 方法名（ControllerMethod 类型时使用）
    pub method_name: Option<String>,
    /// 闭包 ID（Closure 类型时使用）
    pub closure_id: Option<String>,
    /// 静态内容（Static 类型时使用）
    pub static_content: Option<String>,
    /// 静态内容类型
    pub static_content_type: Option<String>,
    /// 静态状态码
    pub static_status: Option<u16>,
    /// 重定向 URL（Redirect 类型时使用）
    pub redirect_url: Option<String>,
    /// 重定向状态码
    pub redirect_status: Option<u16>,
    /// 所属组名
    pub group: Option<String>,
    /// 中间件列表
    pub middleware: Vec<String>,
    /// 参数约束
    pub where_constraints: std::collections::HashMap<String, String>,
    /// 默认参数
    pub defaults: std::collections::HashMap<String, String>,
}

impl From<&Route> for SerializableRoute {
    fn from(route: &Route) -> Self {
        let (handler_type, controller, method_name, closure_id, static_content, static_content_type, static_status, redirect_url, redirect_status) = match &route.handler {
            RouteHandler::ControllerMethod { controller, method } => (
                "controller".to_string(),
                Some(controller.clone()),
                Some(method.clone()),
                None, None, None, None, None, None,
            ),
            RouteHandler::Closure { id } => (
                "closure".to_string(),
                None, None,
                Some(id.clone()),
                None, None, None, None, None,
            ),
            RouteHandler::Static { content, content_type, status } => (
                "static".to_string(),
                None, None, None,
                Some(content.clone()),
                Some(content_type.clone()),
                Some(*status),
                None, None,
            ),
            RouteHandler::Redirect { url, status } => (
                "redirect".to_string(),
                None, None, None, None, None, None,
                Some(url.clone()),
                Some(*status),
            ),
        };

        Self {
            id: route.id.clone(),
            method: route.method.as_str().to_string(),
            path: route.path.clone(),
            handler_type,
            controller,
            method_name,
            closure_id,
            static_content,
            static_content_type,
            static_status,
            redirect_url,
            redirect_status,
            group: route.group.clone(),
            middleware: route.middleware.clone(),
            where_constraints: route.where_constraints.clone(),
            defaults: route.defaults.clone(),
        }
    }
}

impl From<SerializableRoute> for Route {
    fn from(sr: SerializableRoute) -> Self {
        let handler = match sr.handler_type.as_str() {
            "controller" => RouteHandler::ControllerMethod {
                controller: sr.controller.unwrap_or_default(),
                method: sr.method_name.unwrap_or_default(),
            },
            "closure" => RouteHandler::Closure {
                id: sr.closure_id.unwrap_or_default(),
            },
            "static" => RouteHandler::Static {
                content: sr.static_content.unwrap_or_default(),
                content_type: sr.static_content_type.unwrap_or_default(),
                status: sr.static_status.unwrap_or(200),
            },
            "redirect" => RouteHandler::Redirect {
                url: sr.redirect_url.unwrap_or_default(),
                status: sr.redirect_status.unwrap_or(302),
            },
            _ => RouteHandler::Static {
                content: String::new(),
                content_type: "text/plain".to_string(),
                status: 404,
            },
        };

        Route {
            id: sr.id,
            method: HttpMethod::from_str(&sr.method),
            path: sr.path,
            handler,
            group: sr.group,
            middleware: sr.middleware,
            where_constraints: sr.where_constraints,
            defaults: sr.defaults,
        }
    }
}

/// 路由缓存管理器
/// 负责路由缓存的读写和有效性检查
pub struct RouteCacheManager;

impl RouteCacheManager {
    /// 缓存文件名
    const CACHE_FILE: &'static str = "route_cache.bin";
    /// 缓存版本号
    /// 当路由系统结构变更时递增
    const CACHE_VERSION: u32 = 1;

    /// 获取缓存文件路径
    fn cache_path(runtime_dir: &Path) -> std::path::PathBuf {
        runtime_dir.join(Self::CACHE_FILE)
    }

    /// 将路由数据写入缓存文件
    /// 使用 bincode 进行高效二进制序列化
    pub fn save_cache(
        routes: &[Route],
        groups: &[RouteGroup],
        miss_route: Option<&Route>,
        project_root: &str,
        runtime_dir: &Path,
    ) -> Result<()> {
        let serializable_routes: Vec<SerializableRoute> = routes.iter().map(SerializableRoute::from).collect();
        let serializable_miss = miss_route.map(SerializableRoute::from);

        let cache = RouteCache {
            version: Self::CACHE_VERSION,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            project_root: project_root.to_string(),
            routes: serializable_routes,
            groups: groups.to_vec(),
            miss_route: serializable_miss,
        };

        let cache_path = Self::cache_path(runtime_dir);
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let data = bincode::serialize(&cache)?;
        std::fs::write(&cache_path, data)?;

        tracing::debug!("路由缓存已保存: {} 条路由", routes.len());
        Ok(())
    }

    /// 从缓存文件加载路由数据
    /// 会检查缓存版本和项目路径的有效性
    pub fn load_cache(
        project_root: &str,
        runtime_dir: &Path,
    ) -> Result<Option<RouteCache>> {
        let cache_path = Self::cache_path(runtime_dir);
        if !cache_path.exists() {
            return Ok(None);
        }

        let data = std::fs::read(&cache_path)?;
        let cache: RouteCache = match bincode::deserialize(&data) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("路由缓存反序列化失败，将忽略缓存: {}", e);
                let _ = std::fs::remove_file(&cache_path);
                return Ok(None);
            }
        };

        // 检查缓存版本
        if cache.version != Self::CACHE_VERSION {
            tracing::debug!("路由缓存版本不匹配（当前: {}, 缓存: {}），将忽略缓存",
                Self::CACHE_VERSION, cache.version);
            let _ = std::fs::remove_file(&cache_path);
            return Ok(None);
        }

        // 检查项目路径
        if cache.project_root != project_root {
            tracing::debug!("路由缓存项目路径不匹配，将忽略缓存");
            let _ = std::fs::remove_file(&cache_path);
            return Ok(None);
        }

        tracing::debug!("从缓存加载路由: {} 条路由", cache.routes.len());
        Ok(Some(cache))
    }

    /// 清除路由缓存文件
    pub fn clear_cache(runtime_dir: &Path) -> Result<()> {
        let cache_path = Self::cache_path(runtime_dir);
        if cache_path.exists() {
            std::fs::remove_file(&cache_path)?;
            tracing::debug!("路由缓存已清除");
        }
        Ok(())
    }

    /// 检查路由缓存是否存在且有效
    pub fn is_cache_valid(project_root: &str, runtime_dir: &Path) -> bool {
        if let Ok(Some(cache)) = Self::load_cache(project_root, runtime_dir) {
            cache.version == Self::CACHE_VERSION && cache.project_root == project_root
        } else {
            false
        }
    }
}
