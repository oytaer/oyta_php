//! 路由文件解析器模块
//!
//! 解析 route/ 目录下的 PHP 路由定义文件
//! ThinkPHP 8.0 路由定义格式：
//! ```php
//! use think\facade\Route;
//! Route::get('hello/:name', 'index/hello');
//! Route::post('user/create', 'user/create');
//! Route::group('api', function() {
//!     Route::get('v1/users', 'api.v1.User/index');
//! })->middleware('auth');
//! ```

use anyhow::Result;
use std::path::Path;

use super::registry::RouteRegistry;
use super::types::{HttpMethod, Route, RouteHandler};
use crate::parser::php_parser::PhpParser;
use crate::symbol_table::types::FileParseResult;

/// 路由文件解析器
/// 扫描 route/ 目录下的 PHP 文件，提取路由定义
pub struct RouteFileParser;

impl RouteFileParser {
    /// 解析路由目录下的所有文件
    /// 将发现的路由注册到路由注册表中
    pub fn parse_route_dir(
        route_dir: &Path,
        registry: &RouteRegistry,
        parser: &mut PhpParser,
    ) -> Result<usize> {
        if !route_dir.is_dir() {
            tracing::debug!("路由目录不存在: {}", route_dir.display());
            return Ok(0);
        }

        let mut route_count = 0;
        let entries = std::fs::read_dir(route_dir)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("php") {
                let count = Self::parse_route_file(&path, registry, parser)?;
                route_count += count;
            }
        }

        Ok(route_count)
    }

    /// 解析单个路由文件
    fn parse_route_file(
        path: &Path,
        registry: &RouteRegistry,
        parser: &mut PhpParser,
    ) -> Result<usize> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("读取路由文件失败: {} - {}", path.display(), e);
                return Ok(0);
            }
        };

        let result = parser.parse_file(path);
        if let Ok(parse_result) = &result {
            Self::extract_routes_from_result(parse_result, registry);
        }

        let count = Self::parse_routes_from_content(&content, registry, &path.to_string_lossy());

        tracing::debug!(
            "路由文件解析: {} ({} 条路由)",
            path.display(),
            count
        );

        Ok(count)
    }

    /// 从解析结果中提取路由定义
    fn extract_routes_from_result(_result: &FileParseResult, _registry: &RouteRegistry) {
        // AST 分析方式预留，当前使用正则方式
    }

    /// 从文件内容中解析路由定义
    /// 使用正则匹配 Route::get/post/put/delete/patch/any/group/resource/miss 等调用
    fn parse_routes_from_content(
        content: &str,
        registry: &RouteRegistry,
        file_path: &str,
    ) -> usize {
        let mut count = 0;

        let methods = [
            ("get", HttpMethod::GET),
            ("post", HttpMethod::POST),
            ("put", HttpMethod::PUT),
            ("delete", HttpMethod::DELETE),
            ("patch", HttpMethod::PATCH),
            ("any", HttpMethod::ANY),
        ];

        // 解析简单路由定义
        // Route::get('hello/:name', 'index/hello')
        let route_re = regex::Regex::new(
            r#"Route::(get|post|put|delete|patch|any)\s*\(\s*['"]([^'"]+)['"]\s*,\s*['"]([^'"]+)['"]"#
        ).unwrap();

        for cap in route_re.captures_iter(content) {
            let method_str = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let path = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let handler = cap.get(3).map(|m| m.as_str()).unwrap_or("");

            let http_method = methods.iter()
                .find(|(name, _)| *name == method_str)
                .map(|(_, m)| *m)
                .unwrap_or(HttpMethod::GET);

            if !path.is_empty() && !handler.is_empty() {
                let route = parse_route_definition(
                    http_method,
                    path,
                    handler,
                    Some(file_path),
                    &[],
                );
                registry.add_route(route);
                count += 1;
            }
        }

        // 解析 MISS 路由
        // Route::miss('index/miss')
        let miss_re = regex::Regex::new(
            r#"Route::miss\s*\(\s*['"]([^'"]+)['"]"#
        ).unwrap();

        for cap in miss_re.captures_iter(content) {
            let handler = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            if !handler.is_empty() {
                registry.set_miss_route(handler);
                count += 1;
            }
        }

        // 解析资源路由
        // Route::resource('user', 'User')
        let resource_re = regex::Regex::new(
            r#"Route::resource\s*\(\s*['"]([^'"]+)['"]\s*,\s*['"]([^'"]+)['"]"#
        ).unwrap();

        for cap in resource_re.captures_iter(content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let controller = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            if !name.is_empty() && !controller.is_empty() {
                let full_controller = expand_controller_name(controller);
                registry.add_resource(
                    name,
                    &full_controller,
                    "",
                    &[],
                    &[],
                    &[],
                );
                count += 7; // 资源路由生成 7 条路由
            }
        }

        // 解析路由组
        // Route::group('api', function() { ... })->middleware('auth')
        // 由于正则无法完全解析闭包内的路由，这里提取组属性
        // 闭包内的路由通过嵌套解析处理
        count += Self::parse_route_groups(content, registry, file_path);

        count
    }

    /// 解析路由组定义
    /// 提取 Route::group() 调用的前缀、中间件等属性
    /// 以及组内的路由定义
    fn parse_route_groups(
        content: &str,
        registry: &RouteRegistry,
        file_path: &str,
    ) -> usize {
        let mut count = 0;

        // 匹配 Route::group('prefix', function() { ... })
        let group_re = regex::Regex::new(
            r#"Route::group\s*\(\s*['"]([^'"]+)['"]\s*,\s*function\s*\(\s*\)\s*\{([^}]*)\}"#
        ).unwrap();

        // 匹配组级中间件 ->middleware('name')
        let middleware_re = regex::Regex::new(
            r#"->middleware\s*\(\s*['"]([^'"]+)['"]"#
        ).unwrap();

        // 匹配组级命名空间 ->namespace('app\controller\api')
        let namespace_re = regex::Regex::new(
            r#"->namespace\s*\(\s*['"]([^'"]+)['"]"#
        ).unwrap();

        for cap in group_re.captures_iter(content) {
            let prefix = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let group_body = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            // 提取组级中间件
            let group_middleware: Vec<String> = middleware_re
                .captures_iter(content)
                .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
                .collect();

            // 提取组级命名空间
            let group_namespace = namespace_re
                .captures_iter(content)
                .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
                .next();

            // 解析组内的路由定义
            let methods = [
                ("get", HttpMethod::GET),
                ("post", HttpMethod::POST),
                ("put", HttpMethod::PUT),
                ("delete", HttpMethod::DELETE),
                ("patch", HttpMethod::PATCH),
                ("any", HttpMethod::ANY),
            ];

            let inner_route_re = regex::Regex::new(
                r#"Route::(get|post|put|delete|patch|any)\s*\(\s*['"]([^'"]+)['"]\s*,\s*['"]([^'"]+)['"]"#
            ).unwrap();

            let mut group_routes: Vec<(HttpMethod, String, String)> = Vec::new();

            for inner_cap in inner_route_re.captures_iter(group_body) {
                let method_str = inner_cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let path = inner_cap.get(2).map(|m| m.as_str()).unwrap_or("");
                let handler = inner_cap.get(3).map(|m| m.as_str()).unwrap_or("");

                let http_method = methods.iter()
                    .find(|(name, _)| *name == method_str)
                    .map(|(_, m)| *m)
                    .unwrap_or(HttpMethod::GET);

                if !path.is_empty() && !handler.is_empty() {
                    // 组内路由路径加上组前缀
                    let full_path = format!("{}{}", prefix, path);
                    // 如果有命名空间，扩展控制器名
                    let full_handler = if let Some(ref ns) = group_namespace {
                        expand_handler_with_namespace(handler, ns)
                    } else {
                        handler.to_string()
                    };

                    let route = parse_route_definition(
                        http_method,
                        &full_path,
                        &full_handler,
                        Some(file_path),
                        &group_middleware,
                    );
                    registry.add_route(route);
                    group_routes.push((http_method, path.to_string(), handler.to_string()));
                    count += 1;
                }
            }

            // 解析组内资源路由
            let inner_resource_re = regex::Regex::new(
                r#"Route::resource\s*\(\s*['"]([^'"]+)['"]\s*,\s*['"]([^'"]+)['"]"#
            ).unwrap();

            for inner_cap in inner_resource_re.captures_iter(group_body) {
                let name = inner_cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let controller = inner_cap.get(2).map(|m| m.as_str()).unwrap_or("");
                if !name.is_empty() && !controller.is_empty() {
                    let full_controller = if let Some(ref ns) = group_namespace {
                        format!("{}\\{}", ns, controller)
                    } else {
                        expand_controller_name(controller)
                    };
                    registry.add_resource(
                        name,
                        &full_controller,
                        prefix,
                        &group_middleware,
                        &[],
                        &[],
                    );
                    count += 7;
                }
            }
        }

        count
    }
}

/// 从路由定义字符串解析路由
pub fn parse_route_definition(
    method: HttpMethod,
    path: &str,
    handler: &str,
    group: Option<&str>,
    middleware: &[String],
) -> Route {
    let route_handler = parse_handler_string(handler);

    Route {
        id: format!("{}:{}", method.as_str(), path),
        method,
        path: path.to_string(),
        handler: route_handler,
        group: group.map(|g| g.to_string()),
        middleware: middleware.to_vec(),
        where_constraints: std::collections::HashMap::new(),
        defaults: std::collections::HashMap::new(),
    }
}

/// 解析处理器字符串
/// 支持格式:
/// - "Controller@method" (ThinkPHP 标准格式)
/// - "Controller/method"
/// - "controller.method" (点号分隔)
/// - "app\controller\Index@index" (完整命名空间)
pub fn parse_handler_string(handler: &str) -> RouteHandler {
    if let Some((ctrl, method)) = handler.split_once('@') {
        return RouteHandler::ControllerMethod {
            controller: expand_controller_name(ctrl),
            method: method.to_string(),
        };
    }

    if let Some(pos) = handler.rfind('.') {
        let ctrl = &handler[..pos];
        let method = &handler[pos + 1..];
        return RouteHandler::ControllerMethod {
            controller: expand_controller_name(ctrl),
            method: method.to_string(),
        };
    }

    if let Some(pos) = handler.rfind('/') {
        let ctrl = &handler[..pos];
        let method = &handler[pos + 1..];
        return RouteHandler::ControllerMethod {
            controller: expand_controller_name(ctrl),
            method: method.to_string(),
        };
    }

    RouteHandler::Static {
        content: handler.to_string(),
        content_type: "text/plain".to_string(),
        status: 200,
    }
}

/// 将简短控制器名扩展为完整命名空间
/// "index/hello" → "app\\controller\\index\\hello"
/// "Index" → "app\\controller\\Index"
fn expand_controller_name(name: &str) -> String {
    if name.contains('\\') || name.contains("app\\") {
        return name.to_string();
    }

    let parts: Vec<&str> = name.split('.').collect();
    if parts.len() > 1 {
        let namespace = parts[..parts.len() - 1].join("\\");
        let class = parts[parts.len() - 1];
        format!("app\\controller\\{}\\{}", namespace, class)
    } else {
        format!("app\\controller\\{}", name)
    }
}

/// 使用指定命名空间扩展处理器字符串
/// 当路由组设置了命名空间时，控制器名需要加上命名空间前缀
/// 如 handler = "User@index", namespace = "app\\controller\\api"
/// 结果 = "app\\controller\\api\\User@index"
fn expand_handler_with_namespace(handler: &str, namespace: &str) -> String {
    // 如果处理器已经包含完整命名空间，直接返回
    if handler.contains('\\') || handler.contains("app\\") {
        return handler.to_string();
    }

    // 分离控制器名和方法名
    if let Some((ctrl, method)) = handler.split_once('@') {
        let expanded_ctrl = format!("{}\\{}", namespace, ctrl);
        format!("{}@{}", expanded_ctrl, method)
    } else if let Some(pos) = handler.rfind('.') {
        let ctrl = &handler[..pos];
        let method = &handler[pos + 1..];
        let expanded_ctrl = format!("{}\\{}", namespace, ctrl);
        format!("{}@{}", expanded_ctrl, method)
    } else {
        handler.to_string()
    }
}
