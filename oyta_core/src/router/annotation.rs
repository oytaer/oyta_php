//! 注解路由模块
//!
//! 从 PHP 8.0+ 的 Attribute（注解）中提取路由定义
//! ThinkPHP 8.0 支持使用注解方式定义路由：
//! ```php
//! use app\annotation\Route;
//!
//! #[Route('/hello', method: 'GET')]
//! public function hello() { ... }
//! ```
//!
//! 本模块扫描符号表中所有控制器的注解信息
//! 自动注册对应的路由规则

use std::collections::HashMap;

use super::registry::RouteRegistry;
use super::types::{HttpMethod, Route, RouteHandler};
use crate::symbol_table::types::{AttributeDef, ClassDef};

/// 注解路由提取器
/// 从控制器的注解中提取路由定义并注册到路由表
pub struct AnnotationRouteExtractor;

impl AnnotationRouteExtractor {
    /// 从控制器列表中提取所有注解路由
    /// 遍历每个控制器的类级和方法级注解
    /// 将匹配到的路由注册到路由注册表
    pub fn extract_and_register(
        controllers: &[ClassDef],
        registry: &RouteRegistry,
        class_attributes: &HashMap<String, Vec<AttributeDef>>,
        method_attributes: &HashMap<(String, String), Vec<AttributeDef>>,
    ) -> usize {
        let mut count = 0;

        for controller in controllers {
            // 提取类级路由前缀
            // 如: #[Route('/api/users')] class UserController
            let class_prefix = Self::extract_class_prefix(controller, class_attributes);

            // 提取类级中间件
            let class_middleware = Self::extract_class_middleware(controller, class_attributes);

            // 遍历控制器方法，提取方法级路由注解
            for method in &controller.methods {
                let key = (controller.full_name.clone(), method.name.clone());
                if let Some(attrs) = method_attributes.get(&key) {
                    for attr in attrs {
                        if Self::is_route_attribute(attr) {
                            if let Some(route) = Self::create_route_from_attribute(
                                attr,
                                &controller.full_name,
                                &method.name,
                                &class_prefix,
                                &class_middleware,
                            ) {
                                registry.add_route(route);
                                count += 1;
                            }
                        }
                    }
                }
            }
        }

        if count > 0 {
            tracing::debug!("从注解中提取了 {} 条路由", count);
        }

        count
    }

    /// 从符号表的属性定义中提取注解路由
    /// 直接使用 AttributeDef 结构体进行匹配
    pub fn extract_from_symbol_attributes(
        controllers: &[ClassDef],
        registry: &RouteRegistry,
    ) -> usize {
        let mut count = 0;

        for controller in controllers {
            // 检查类注解
            let class_prefix = String::new();
            let class_middleware: Vec<String> = Vec::new();

            // 遍历控制器方法
            for method in &controller.methods {
                // 检查方法注解
                // 这里通过方法名匹配 Route 注解
                if let Some(route) = Self::try_create_route_from_method(
                    controller,
                    method,
                    &class_prefix,
                    &class_middleware,
                ) {
                    registry.add_route(route);
                    count += 1;
                }
            }
        }

        count
    }

    /// 判断属性是否为路由注解
    /// 支持以下名称：
    /// - Route
    /// - oyta\annotation\Route
    /// - think\annotation\Route
    fn is_route_attribute(attr: &AttributeDef) -> bool {
        let name = attr.name.to_lowercase();
        name == "route"
            || attr.full_name.to_lowercase().ends_with("\\route")
            || attr.full_name.to_lowercase().contains("annotation\\route")
    }

    /// 从类注解中提取路由前缀
    /// 如 #[Route('/api')] → prefix = "/api"
    fn extract_class_prefix(
        _controller: &ClassDef,
        _class_attributes: &HashMap<String, Vec<AttributeDef>>,
    ) -> String {
        // 类级前缀注解提取
        // 当前预留，后续通过完整的注解解析实现
        String::new()
    }

    /// 从类注解中提取中间件
    /// 如 #[Middleware('auth')] → middleware = ["auth"]
    fn extract_class_middleware(
        _controller: &ClassDef,
        _class_attributes: &HashMap<String, Vec<AttributeDef>>,
    ) -> Vec<String> {
        // 类级中间件注解提取
        Vec::new()
    }

    /// 从注解属性创建路由
    /// 解析注解的参数：
    /// - 第一个位置参数为路径
    /// - method 命名参数为 HTTP 方法
    /// - name 命名参数为路由名称
    /// - middleware 命名参数为中间件
    fn create_route_from_attribute(
        attr: &AttributeDef,
        controller: &str,
        method_name: &str,
        class_prefix: &str,
        class_middleware: &[String],
    ) -> Option<Route> {
        // 提取路径（第一个位置参数）
        let path = attr.positional_args.first()?;

        // 提取 HTTP 方法
        let http_method = attr.named_args.iter()
            .find(|(k, _)| k.to_lowercase() == "method")
            .and_then(|(_, v)| Some(HttpMethod::from_str(v)))
            .unwrap_or(HttpMethod::GET);

        // 提取中间件
        let method_middleware: Vec<String> = attr.named_args.iter()
            .find(|(k, _)| k.to_lowercase() == "middleware")
            .map(|(_, v)| v.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect())
            .unwrap_or_default();

        // 合并类级和方法级中间件
        let mut middleware = class_middleware.to_vec();
        middleware.extend(method_middleware);

        // 构建完整路径
        let full_path = if class_prefix.is_empty() {
            path.clone()
        } else {
            format!("{}{}", class_prefix, path)
        };

        // 提取路由名称
        let route_name = attr.named_args.iter()
            .find(|(k, _)| k.to_lowercase() == "name")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| format!("annotation:{}:{}", controller, method_name));

        Some(Route {
            id: route_name,
            method: http_method,
            path: full_path,
            handler: RouteHandler::ControllerMethod {
                controller: controller.to_string(),
                method: method_name.to_string(),
            },
            group: Some("annotation".to_string()),
            middleware,
            where_constraints: HashMap::new(),
            defaults: HashMap::new(),
        })
    }

    /// 尝试从方法名和命名约定创建路由
    /// 当没有显式注解时，根据方法名推断路由
    fn try_create_route_from_method(
        controller: &ClassDef,
        method: &crate::symbol_table::types::MethodDef,
        class_prefix: &str,
        class_middleware: &[String],
    ) -> Option<Route> {
        // 仅处理带有特定命名模式的方法
        // 如: getHello, postCreate, putUpdate, deleteRemove
        let method_name = &method.name;
        let (http_method, path_suffix) = Self::parse_method_name(method_name)?;

        let controller_path = camel_to_kebab(&controller.name);
        let full_path = if class_prefix.is_empty() {
            format!("/{}/{}", controller_path, path_suffix)
        } else {
            format!("{}/{}/{}", class_prefix, controller_path, path_suffix)
        };

        Some(Route {
            id: format!("annotation:{}:{}", controller.full_name, method_name),
            method: http_method,
            path: full_path,
            handler: RouteHandler::ControllerMethod {
                controller: controller.full_name.clone(),
                method: method_name.clone(),
            },
            group: Some("annotation".to_string()),
            middleware: class_middleware.to_vec(),
            where_constraints: HashMap::new(),
            defaults: HashMap::new(),
        })
    }

    /// 解析方法名中的 HTTP 方法前缀
    /// getHello → (GET, "hello")
    /// postCreate → (POST, "create")
    /// putUpdate → (PUT, "update")
    /// deleteRemove → (DELETE, "remove")
    fn parse_method_name(name: &str) -> Option<(HttpMethod, String)> {
        let lower = name.to_lowercase();
        let (method, suffix) = if lower.starts_with("get") && name.len() > 3 {
            (HttpMethod::GET, &name[3..])
        } else if lower.starts_with("post") && name.len() > 4 {
            (HttpMethod::POST, &name[4..])
        } else if lower.starts_with("put") && name.len() > 3 {
            (HttpMethod::PUT, &name[3..])
        } else if lower.starts_with("delete") && name.len() > 6 {
            (HttpMethod::DELETE, &name[6..])
        } else if lower.starts_with("patch") && name.len() > 5 {
            (HttpMethod::PATCH, &name[5..])
        } else {
            return None;
        };

        Some((method, camel_to_kebab(suffix)))
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
