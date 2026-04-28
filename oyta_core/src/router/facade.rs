//! 路由系统门面
//!
//! 提供静态方法访问路由注册表功能
//! 对应 ThinkPHP 8.0 的 Route 门面
//!
//! # 使用示例
//! ```php
//! // 注册 GET 路由
//! Route::get('/user/{id}', 'UserController@show');
//!
//! // 注册 POST 路由
//! Route::post('/user', 'UserController@store');
//!
//! // 路由组
//! Route::group(['prefix' => 'admin'], function() {
//!     Route::get('/dashboard', 'AdminController@dashboard');
//! });
//! ```
//!
//! # 注意
//! 门面结构体命名为 RouteFacade，通过 mod.rs 导出为 RouteFacade
//! 路由定义类型 Route 在 types.rs 中定义，通过 mod.rs 直接导出

use std::sync::Arc;
use parking_lot::RwLock;

use super::registry::RouteRegistry;
use super::types::{HttpMethod, Route, RouteGroup, RouteHandler, RouteMatch};

/// 全局路由注册表实例
///
/// 使用 RwLock 实现线程安全的单例模式
/// 懒加载初始化，首次使用时创建
static ROUTE_REGISTRY: RwLock<Option<Arc<RouteRegistry>>> = RwLock::new(None);

/// RouteFacade 门面结构体
///
/// 提供静态方法访问路由注册表
/// 所有方法都是线程安全的
///
/// # 使用方式
/// ```rust
/// use oyta_core::router::RouteFacade;
///
/// // 初始化路由门面
/// RouteFacade::init();
///
/// // 注册路由
/// RouteFacade::get("/user/:id", "UserController@show");
/// RouteFacade::post("/user", "UserController@store");
/// ```
pub struct RouteFacade;

impl RouteFacade {
    /// 初始化全局路由注册表
    ///
    /// 必须在使用其他方法前调用
    /// 通常在应用启动时自动调用
    pub fn init() {
        // 获取写锁
        let mut guard = ROUTE_REGISTRY.write();
        // 创建新的路由注册表实例
        let registry = RouteRegistry::new();
        // 存储到全局静态变量
        *guard = Some(Arc::new(registry));
    }

    /// 获取全局路由注册表实例
    ///
    /// 内部方法，用于获取注册表引用
    fn get_registry() -> Option<Arc<RouteRegistry>> {
        // 获取读锁
        let guard = ROUTE_REGISTRY.read();
        // 克隆内部的 Arc 引用
        guard.clone()
    }

    /// 注册 GET 路由
    ///
    /// 仅响应 HTTP GET 请求
    ///
    /// # 参数
    /// - `path`: 路由路径，支持参数如 `/user/:id`
    /// - `handler`: 路由处理器，格式为 `控制器名@方法名`
    ///
    /// # 示例
    /// ```php
    /// Route::get('/user/{id}', 'UserController@show');
    /// ```
    pub fn get(path: &str, handler: &str) {
        // 调用 register 方法注册 GET 路由
        Self::register(HttpMethod::GET, path, handler);
    }

    /// 注册 POST 路由
    ///
    /// 仅响应 HTTP POST 请求
    ///
    /// # 参数
    /// - `path`: 路由路径
    /// - `handler`: 路由处理器
    ///
    /// # 示例
    /// ```php
    /// Route::post('/user', 'UserController@store');
    /// ```
    pub fn post(path: &str, handler: &str) {
        // 调用 register 方法注册 POST 路由
        Self::register(HttpMethod::POST, path, handler);
    }

    /// 注册 PUT 路由
    ///
    /// 仅响应 HTTP PUT 请求
    /// 通常用于更新资源
    ///
    /// # 参数
    /// - `path`: 路由路径
    /// - `handler`: 路由处理器
    pub fn put(path: &str, handler: &str) {
        // 调用 register 方法注册 PUT 路由
        Self::register(HttpMethod::PUT, path, handler);
    }

    /// 注册 DELETE 路由
    ///
    /// 仅响应 HTTP DELETE 请求
    /// 通常用于删除资源
    ///
    /// # 参数
    /// - `path`: 路由路径
    /// - `handler`: 路由处理器
    pub fn delete(path: &str, handler: &str) {
        // 调用 register 方法注册 DELETE 路由
        Self::register(HttpMethod::DELETE, path, handler);
    }

    /// 注册 PATCH 路由
    ///
    /// 仅响应 HTTP PATCH 请求
    /// 通常用于部分更新资源
    ///
    /// # 参数
    /// - `path`: 路由路径
    /// - `handler`: 路由处理器
    pub fn patch(path: &str, handler: &str) {
        // 调用 register 方法注册 PATCH 路由
        Self::register(HttpMethod::PATCH, path, handler);
    }

    /// 注册 HEAD 路由
    ///
    /// 仅响应 HTTP HEAD 请求
    /// 通常用于获取资源元信息
    ///
    /// # 参数
    /// - `path`: 路由路径
    /// - `handler`: 路由处理器
    pub fn head(path: &str, handler: &str) {
        // 调用 register 方法注册 HEAD 路由
        Self::register(HttpMethod::HEAD, path, handler);
    }

    /// 注册 OPTIONS 路由
    ///
    /// 仅响应 HTTP OPTIONS 请求
    /// 通常用于 CORS 预检请求
    ///
    /// # 参数
    /// - `path`: 路由路径
    /// - `handler`: 路由处理器
    pub fn options(path: &str, handler: &str) {
        // 调用 register 方法注册 OPTIONS 路由
        Self::register(HttpMethod::OPTIONS, path, handler);
    }

    /// 注册任意 HTTP 方法的路由
    ///
    /// 响应所有 HTTP 方法的请求
    ///
    /// # 参数
    /// - `path`: 路由路径
    /// - `handler`: 路由处理器
    pub fn any(path: &str, handler: &str) {
        // 调用 register 方法注册 ANY 路由
        Self::register(HttpMethod::ANY, path, handler);
    }

    /// 注册匹配指定方法的路由
    ///
    /// 响应多个指定的 HTTP 方法
    ///
    /// # 参数
    /// - `methods`: HTTP 方法列表
    /// - `path`: 路由路径
    /// - `handler`: 路由处理器
    pub fn match_methods(methods: &[HttpMethod], path: &str, handler: &str) {
        // 遍历所有指定的方法
        for method in methods {
            // 为每个方法注册路由
            Self::register(*method, path, handler);
        }
    }

    /// 通用路由注册方法
    ///
    /// 内部方法，用于注册单个路由
    ///
    /// # 参数
    /// - `method`: HTTP 方法
    /// - `path`: 路由路径
    /// - `handler`: 路由处理器
    fn register(method: HttpMethod, path: &str, handler: &str) {
        // 尝试获取注册表实例
        if let Some(registry) = Self::get_registry() {
            // 使用 route_parser 解析处理器字符串
            let route_handler = super::route_parser::parse_handler_string(handler);
            // 生成路由 ID
            let id = format!("{}:{}", method.as_str(), path);
            // 创建路由对象，使用 types::Route 类型
            let route = Route {
                // 路由唯一标识
                id,
                // HTTP 方法
                method,
                // 路由路径
                path: path.to_string(),
                // 路由处理器
                handler: route_handler,
                // 所属路由组
                group: None,
                // 中间件列表
                middleware: Vec::new(),
                // 参数约束
                where_constraints: std::collections::HashMap::new(),
                // 默认参数
                defaults: std::collections::HashMap::new(),
            };
            // 添加到注册表
            registry.add_route(route);
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("RouteFacade 门面未初始化，无法注册路由: {} {}", method, path);
        }
    }

    /// 注册路由组
    ///
    /// 将多个路由组织在一起，共享公共属性
    ///
    /// # 参数
    /// - `group`: 路由组定义
    ///
    /// # 示例
    /// ```php
    /// Route::group([
    ///     'name' => 'admin',
    ///     'prefix' => 'admin',
    ///     'middleware' => ['auth'],
    /// ], function() {
    ///     Route::get('/dashboard', 'AdminController@dashboard');
    /// });
    /// ```
    pub fn group(group: RouteGroup) {
        // 尝试获取注册表实例
        if let Some(registry) = Self::get_registry() {
            // 添加路由组到注册表
            registry.add_group(group);
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("RouteFacade 门面未初始化，无法注册路由组: {}", group.name);
        }
    }

    /// 匹配路由
    ///
    /// 根据请求方法和路径查找匹配的路由
    ///
    /// # 参数
    /// - `method`: HTTP 方法
    /// - `path`: 请求路径
    ///
    /// # 返回
    /// 匹配结果，包含路由信息和提取的参数
    pub fn match_route(method: HttpMethod, path: &str) -> Option<RouteMatch> {
        // 尝试获取注册表实例
        if let Some(registry) = Self::get_registry() {
            // 调用注册表的 match_route 方法
            registry.match_route(&method, path)
        } else {
            // 未初始化时返回 None
            None
        }
    }

    /// 获取所有已注册的路由
    ///
    /// 返回所有路由定义的列表
    ///
    /// # 返回
    /// 路由列表
    pub fn all_routes() -> Vec<Route> {
        // 尝试获取注册表实例
        if let Some(registry) = Self::get_registry() {
            // 调用注册表的 all_routes 方法
            registry.all_routes()
        } else {
            // 未初始化时返回空列表
            Vec::new()
        }
    }

    /// 获取路由数量
    ///
    /// 返回已注册路由的总数
    ///
    /// # 返回
    /// 路由数量
    pub fn count() -> usize {
        // 尝试获取注册表实例
        if let Some(registry) = Self::get_registry() {
            // 调用注册表的 route_count 方法
            registry.route_count()
        } else {
            // 未初始化时返回 0
            0
        }
    }

    /// 清除所有路由
    ///
    /// 删除所有已注册的路由
    /// 主要用于测试环境
    pub fn clear() {
        // 尝试获取注册表实例
        if let Some(registry) = Self::get_registry() {
            // 调用注册表的 clear 方法
            registry.clear();
        }
    }

    /// 设置 MISS 路由
    ///
    /// 当所有路由都不匹配时调用的处理器
    /// 通常用于 404 页面
    ///
    /// # 参数
    /// - `handler`: 路由处理器字符串
    pub fn miss(handler: &str) {
        // 尝试获取注册表实例
        if let Some(registry) = Self::get_registry() {
            // 调用注册表的 set_miss_route 方法
            registry.set_miss_route(handler);
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("RouteFacade 门面未初始化，无法设置 MISS 路由");
        }
    }

    /// 注册资源路由
    ///
    /// 自动生成 RESTful 风格的路由集合
    ///
    /// # 参数
    /// - `name`: 资源名称
    /// - `controller`: 控制器名称
    ///
    /// # 生成的路由
    /// - GET /{name} -> index
    /// - GET /{name}/create -> create
    /// - POST /{name} -> store
    /// - GET /{name}/{id} -> show
    /// - GET /{name}/{id}/edit -> edit
    /// - PUT /{name}/{id} -> update
    /// - DELETE /{name}/{id} -> destroy
    pub fn resource(name: &str, controller: &str) {
        // 注册 index 路由 - 列表页
        Self::get(
            &format!("/{}", name),
            &format!("{}@index", controller),
        );
        // 注册 create 路由 - 创建页
        Self::get(
            &format!("/{}/create", name),
            &format!("{}@create", controller),
        );
        // 注册 store 路由 - 保存新资源
        Self::post(
            &format!("/{}", name),
            &format!("{}@store", controller),
        );
        // 注册 show 路由 - 详情页
        Self::get(
            &format!("/{}/:id", name),
            &format!("{}@show", controller),
        );
        // 注册 edit 路由 - 编辑页
        Self::get(
            &format!("/{}/:id/edit", name),
            &format!("{}@edit", controller),
        );
        // 注册 update 路由 - 更新资源
        Self::put(
            &format!("/{}/:id", name),
            &format!("{}@update", controller),
        );
        // 注册 destroy 路由 - 删除资源
        Self::delete(
            &format!("/{}/:id", name),
            &format!("{}@destroy", controller),
        );
    }

    /// 注册资源路由（带配置）
    ///
    /// 自动生成 RESTful 风格的路由集合
    /// 支持配置前缀、中间件、仅生成指定路由、排除指定路由
    ///
    /// # 参数
    /// - `name`: 资源名称
    /// - `controller`: 控制器名称
    /// - `prefix`: 路径前缀
    /// - `middleware`: 中间件列表
    /// - `only`: 仅生成指定的路由（为空则生成全部）
    /// - `except`: 排除指定的路由
    pub fn resource_with_config(
        name: &str,
        controller: &str,
        prefix: &str,
        middleware: &[String],
        only: &[&str],
        except: &[&str],
    ) {
        // 尝试获取注册表实例
        if let Some(registry) = Self::get_registry() {
            // 调用注册表的 add_resource 方法
            registry.add_resource(name, controller, prefix, middleware, only, except);
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("RouteFacade 门面未初始化，无法注册资源路由: {}", name);
        }
    }

    /// 注册路由组（简化版）
    ///
    /// 批量设置路由的公共属性
    ///
    /// # 参数
    /// - `name`: 组名
    /// - `prefix`: 路径前缀
    /// - `namespace`: 控制器命名空间前缀
    /// - `middleware`: 组内所有路由共享的中间件
    /// - `routes`: 组内路由定义列表
    pub fn group_simple(
        name: &str,
        prefix: &str,
        namespace: Option<&str>,
        middleware: &[String],
        routes: &[(HttpMethod, &str, &str)],
    ) {
        // 尝试获取注册表实例
        if let Some(registry) = Self::get_registry() {
            // 调用注册表的 add_route_group 方法
            registry.add_route_group(name, prefix, namespace, middleware, routes);
        } else {
            // 未初始化时记录警告日志
            tracing::warn!("RouteFacade 门面未初始化，无法注册路由组: {}", name);
        }
    }

    /// 获取 MISS 路由
    ///
    /// 返回设置的 MISS 路由（404 兜底处理器）
    ///
    /// # 返回
    /// MISS 路由定义，如果未设置返回 None
    pub fn get_miss_route() -> Option<Route> {
        // 尝试获取注册表实例
        if let Some(registry) = Self::get_registry() {
            // 调用注册表的 get_miss_route 方法
            registry.get_miss_route()
        } else {
            // 未初始化时返回 None
            None
        }
    }

    /// 获取所有路由组
    ///
    /// 返回所有已注册的路由组
    ///
    /// # 返回
    /// 路由组列表
    pub fn all_groups() -> Vec<RouteGroup> {
        // 尝试获取注册表实例
        if let Some(registry) = Self::get_registry() {
            // 调用注册表的 all_groups 方法
            registry.all_groups()
        } else {
            // 未初始化时返回空列表
            Vec::new()
        }
    }

    /// 检查路由门面是否已初始化
    ///
    /// 用于判断是否需要调用 init 方法
    ///
    /// # 返回
    /// 如果已初始化返回 true，否则返回 false
    pub fn is_initialized() -> bool {
        // 获取读锁检查是否有值
        let guard = ROUTE_REGISTRY.read();
        guard.is_some()
    }

    /// 重置路由门面
    ///
    /// 清除路由注册表实例
    /// 主要用于测试环境
    pub fn reset() {
        // 获取写锁
        let mut guard = ROUTE_REGISTRY.write();
        // 清空注册表实例
        *guard = None;
    }
}
