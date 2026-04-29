//! 路由注册表模块
//!
//! 管理所有注册的路由项
//! 支持路由匹配、自动路由生成、路由缓存等功能
//! 使用 matchit 实现高性能路由匹配

use dashmap::DashMap;
use matchit::Router as MatchitRouter;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::types::{HttpMethod, Route, RouteGroup, RouteHandler, RouteMatch};
use crate::symbol_table::types::Visibility;

/// 路由注册表
/// 存储和管理所有路由项
/// 支持按方法+路径快速匹配
/// 支持 MISS 路由（404 兜底）、资源路由、路由分组
pub struct RouteRegistry {
    /// 所有注册的路由项
    /// 键：路由 ID，值：路由定义
    routes: DashMap<String, Route>,
    /// 按方法分组的 matchit 路由器
    /// 键：HTTP 方法，值：对应的 matchit 路由器实例
    matchers: RwLock<HashMap<HttpMethod, MatchitRouter<String>>>,
    /// ANY 方法的路由器（匹配所有 HTTP 方法）
    any_matcher: RwLock<MatchitRouter<String>>,
    /// 路由组
    /// 键：组名，值：路由组定义
    groups: DashMap<String, RouteGroup>,
    /// MISS 路由（404 兜底处理器）
    /// 当所有路由都不匹配时使用
    miss_route: RwLock<Option<Route>>,
    /// 已注册的路由数量
    route_count: AtomicUsize,
}

impl std::fmt::Debug for RouteRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouteRegistry")
            .field("route_count", &self.route_count.load(Ordering::Relaxed))
            .finish()
    }
}

impl RouteRegistry {
    /// 创建新的路由注册表
    pub fn new() -> Self {
        Self {
            routes: DashMap::new(),
            matchers: RwLock::new(HashMap::new()),
            any_matcher: RwLock::new(MatchitRouter::new()),
            groups: DashMap::new(),
            miss_route: RwLock::new(None),
            route_count: AtomicUsize::new(0),
        }
    }

    /// 注册单个路由
    pub fn add_route(&self, route: Route) {
        let route_id = route.id.clone();
        let method = route.method;
        let path = route.path.clone();

        // 存储路由项
        self.routes.insert(route_id.clone(), route);
        self.route_count.fetch_add(1, Ordering::Relaxed);

        // 添加到对应的 matchit 路由器
        if method == HttpMethod::ANY {
            let mut matcher = self.any_matcher.write();
            if let Err(e) = matcher.insert(&path, route_id) {
                tracing::debug!("路由注册失败(ANY): {} - {}", path, e);
            }
        } else {
            let mut matchers = self.matchers.write();
            let matcher = matchers.entry(method).or_insert_with(MatchitRouter::new);
            if let Err(e) = matcher.insert(&path, route_id) {
                tracing::debug!("路由注册失败({}): {} - {}", method, path, e);
            }
        }
    }

    /// 注册路由组
    pub fn add_group(&self, group: RouteGroup) {
        let group_name = group.name.clone();
        let routes = group.routes.clone();

        // 存储路由组
        self.groups.insert(group_name, group);

        // 注册组内的所有路由
        for route in routes {
            self.add_route(route);
        }
    }

    /// 匹配路由
    /// 根据请求方法和路径查找匹配的路由
    pub fn match_route(&self, method: &HttpMethod, path: &str) -> Option<RouteMatch> {
        // 先在对应方法的路由器中查找
        let matchers = self.matchers.read();
        if let Some(matcher) = matchers.get(method) {
            if let Ok(matched) = matcher.at(path) {
                if let Some(route) = self.routes.get(matched.value) {
                    let mut params = HashMap::new();
                    for (key, value) in matched.params.iter() {
                        params.insert(key.to_string(), value.to_string());
                    }
                    return Some(RouteMatch {
                        route: route.value().clone(),
                        params,
                    });
                }
            }
        }
        drop(matchers);

        // 如果没有找到，尝试在 ANY 路由器中查找
        let any_matcher = self.any_matcher.read();
        if let Ok(matched) = any_matcher.at(path) {
            if let Some(route) = self.routes.get(matched.value) {
                let mut params = HashMap::new();
                for (key, value) in matched.params.iter() {
                    params.insert(key.to_string(), value.to_string());
                }
                return Some(RouteMatch {
                    route: route.value().clone(),
                    params,
                });
            }
        }

        None
    }

    /// 获取所有路由项
    pub fn all_routes(&self) -> Vec<Route> {
        self.routes.iter().map(|e| e.value().clone()).collect()
    }

    /// 获取路由数量
    pub fn route_count(&self) -> usize {
        self.route_count.load(Ordering::Relaxed)
    }

    /// 获取所有路由组
    pub fn all_groups(&self) -> Vec<RouteGroup> {
        self.groups.iter().map(|e| e.value().clone()).collect()
    }

    /// 清空所有路由
    pub fn clear(&self) {
        self.routes.clear();
        self.groups.clear();
        {
            let mut matchers = self.matchers.write();
            matchers.clear();
        }
        {
            let mut any_matcher = self.any_matcher.write();
            *any_matcher = MatchitRouter::new();
        }
        {
            let mut miss = self.miss_route.write();
            *miss = None;
        }
        self.route_count.store(0, Ordering::Relaxed);
    }

    /// 设置 MISS 路由（404 兜底处理器）
    /// 当所有路由都不匹配时，使用此路由处理请求
    /// ThinkPHP 8.0 中的 Route::miss() 功能
    pub fn set_miss_route(&self, handler: &str) {
        let route = Route {
            id: "__miss__".to_string(),
            method: HttpMethod::ANY,
            path: "/__miss__".to_string(),
            handler: super::route_parser::parse_handler_string(handler),
            group: None,
            middleware: Vec::new(),
            where_constraints: HashMap::new(),
            defaults: HashMap::new(),
        };
        let mut miss = self.miss_route.write();
        *miss = Some(route);
    }

    /// 获取 MISS 路由
    pub fn get_miss_route(&self) -> Option<Route> {
        self.miss_route.read().clone()
    }

    /// 注册资源路由（RESTful 风格）
    /// 自动生成 index/create/store/read/edit/update/delete 七个路由
    /// 对应 ThinkPHP 8.0 中的 Route::resource() 功能
    ///
    /// # 参数
    /// - `name`: 资源名称（如 "user"），用于生成路由路径
    /// - `controller`: 控制器类名（如 "app\\controller\\User"）
    /// - `prefix`: 路径前缀（如 "/api"）
    /// - `middleware`: 中间件列表
    /// - `only`: 仅生成指定的路由（为空则生成全部）
    /// - `except`: 排除指定的路由
    pub fn add_resource(
        &self,
        name: &str,
        controller: &str,
        prefix: &str,
        middleware: &[String],
        only: &[&str],
        except: &[&str],
    ) {
        // 定义资源路由的默认动作
        let resource_actions = [
            ("index", HttpMethod::GET, format!("{}/{}", prefix, name)),
            ("create", HttpMethod::GET, format!("{}/{}/create", prefix, name)),
            ("store", HttpMethod::POST, format!("{}/{}", prefix, name)),
            ("read", HttpMethod::GET, format!("{}/{}/:id", prefix, name)),
            ("edit", HttpMethod::GET, format!("{}/{}/:id/edit", prefix, name)),
            ("update", HttpMethod::PUT, format!("{}/{}/:id", prefix, name)),
            ("delete", HttpMethod::DELETE, format!("{}/{}/:id", prefix, name)),
        ];

        for (action, method, path) in &resource_actions {
            // 检查 only 约束
            if !only.is_empty() && !only.contains(action) {
                continue;
            }
            // 检查 except 约束
            if except.contains(action) {
                continue;
            }

            let route = Route {
                id: format!("resource:{}.{}", name, action),
                method: *method,
                path: path.clone(),
                handler: RouteHandler::ControllerMethod {
                    controller: controller.to_string(),
                    method: action.to_string(),
                },
                group: Some(format!("resource:{}", name)),
                middleware: middleware.to_vec(),
                where_constraints: HashMap::new(),
                defaults: HashMap::new(),
            };
            self.add_route(route);
        }
    }

    /// 注册路由组
    /// 批量设置路由的公共属性（前缀、中间件、命名空间）
    /// 对应 ThinkPHP 8.0 中的 Route::group() 功能
    ///
    /// # 参数
    /// - `name`: 组名
    /// - `prefix`: 路径前缀
    /// - `namespace`: 控制器命名空间前缀
    /// - `middleware`: 组内所有路由共享的中间件
    /// - `routes`: 组内路由定义列表（(方法, 路径, 处理器) 元组）
    pub fn add_route_group(
        &self,
        name: &str,
        prefix: &str,
        namespace: Option<&str>,
        middleware: &[String],
        route_defs: &[(HttpMethod, &str, &str)],
    ) {
        let mut group = RouteGroup::new(name, prefix);
        if let Some(ns) = namespace {
            group = group.namespace(ns);
        }
        for mw in middleware {
            group = group.middleware(mw);
        }

        for (method, path, handler) in route_defs {
            let full_path = format!("{}{}", prefix, path);
            let route = super::route_parser::parse_route_definition(
                *method,
                &full_path,
                handler,
                Some(name),
                middleware,
            );
            self.add_route(route);
        }

        self.groups.insert(name.to_string(), group);
    }

    /// 生成自动路由
    /// 根据符号表中的控制器类自动生成路由
    /// 规则：
    /// - 单应用: /控制器名/方法名
    /// - 多应用: /应用名/控制器名/方法名
    pub fn generate_auto_routes(
        &self,
        controllers: &[crate::symbol_table::types::ClassDef],
        is_multi_app: bool,
        app_name: Option<&str>,
    ) {
        for controller in controllers {
            // 提取控制器短名
            let controller_name = controller.name.clone();

            // 将控制器名转为蛇形路径
            // 如 "Index" → "index", "UserInfo" → "user_info"
            let path_name = camel_to_kebab(&controller_name);

            for method in &controller.methods {
                // 跳过构造方法和私有方法
                if method.name == "__construct" || method.visibility != Visibility::Public {
                    continue;
                }
                // 跳过下划线开头的方法
                if method.name.starts_with('_') {
                    continue;
                }

                let method_path_name = camel_to_kebab(&method.name);

                // 单应用模式下，跳过冗余路由 /index/index
                // index 是默认控制器名和默认方法名，这种路由应该通过 / 或 /index 访问
                if !is_multi_app && path_name == "index" && method_path_name == "index" {
                    tracing::debug!("跳过冗余自动路由: /index/index");
                    continue;
                }

                // 构建路由路径
                let route_path = if is_multi_app {
                    if let Some(app) = app_name {
                        format!("/{}/{}/{}", app, path_name, method_path_name)
                    } else {
                        format!("/{}/{}", path_name, method_path_name)
                    }
                } else {
                    format!("/{}/{}", path_name, method_path_name)
                };

                // 根据方法名推断 HTTP 方法
                let http_method = infer_http_method(&method.name);

                // 创建路由
                let route = Route {
                    id: format!("auto:{}:{}", http_method, route_path),
                    method: http_method,
                    path: route_path,
                    handler: RouteHandler::ControllerMethod {
                        controller: controller.full_name.clone(),
                        method: method.name.clone(),
                    },
                    group: Some("auto".to_string()),
                    middleware: Vec::new(),
                    where_constraints: HashMap::new(),
                    defaults: HashMap::new(),
                };

                self.add_route(route);
            }
        }
    }
}

impl Default for RouteRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 将驼峰命名转为 kebab-case
fn camel_to_kebab(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// 根据控制器方法名推断 HTTP 方法
/// ThinkPHP 约定：
/// - index/list → GET
/// - create/read/show → GET
/// - save/store → POST
/// - update → PUT
/// - delete/destroy → DELETE
fn infer_http_method(method_name: &str) -> HttpMethod {
    match method_name {
        "index" | "list" | "create" | "read" | "show" | "edit" => HttpMethod::GET,
        "save" | "store" => HttpMethod::POST,
        "update" => HttpMethod::PUT,
        "delete" | "destroy" => HttpMethod::DELETE,
        _ => HttpMethod::GET,
    }
}
