//! 路由缓存模块
//!
//! 提供路由文件的解析和缓存生成功能

use std::path::Path;

use super::types::RouteInfo;

/// 路由缓存生成器
pub struct RouteCacheGenerator;

impl RouteCacheGenerator {
    /// 解析路由文件
    ///
    /// # 参数
    /// - `path`: 路由文件路径
    ///
    /// # 返回
    /// 路由列表
    pub fn parse_route_file(path: &Path) -> anyhow::Result<Vec<RouteInfo>> {
        let mut routes = Vec::new();

        // 读取文件内容
        let content = std::fs::read_to_string(path)?;

        // 简单的路由解析（匹配 Route::get/post/put/delete/patch）
        for line in content.lines() {
            let line = line.trim();

            // 匹配 Route::method('/path', ...)
            if line.starts_with("Route::") {
                if let Some(route) = Self::parse_route_line(line) {
                    routes.push(route);
                }
            }
        }

        tracing::debug!("解析路由文件: {} -> {} 个路由", path.display(), routes.len());

        Ok(routes)
    }

    /// 解析单行路由定义
    ///
    /// # 参数
    /// - `line`: 路由定义行
    ///
    /// # 返回
    /// 路由信息
    fn parse_route_line(line: &str) -> Option<RouteInfo> {
        // 提取方法名
        let methods = ["get", "post", "put", "delete", "patch", "options", "any"];

        for method in methods {
            let prefix = format!("Route::{}(", method);
            if line.starts_with(&prefix) {
                // 提取路径
                let rest = line.strip_prefix(&prefix)?;
                let path_end = rest.find(',')?;
                let path = rest[..path_end].trim().trim_matches('\'').trim_matches('"');

                // 提取处理器
                let handler_part = rest[path_end + 1..].trim();
                let handler = Self::extract_handler(handler_part);

                return Some(RouteInfo {
                    method: method.to_uppercase(),
                    path: path.to_string(),
                    handler,
                });
            }
        }

        None
    }

    /// 提取处理器信息
    ///
    /// # 参数
    /// - `handler_part`: 处理器部分字符串
    ///
    /// # 返回
    /// 处理器字符串
    fn extract_handler(handler_part: &str) -> String {
        // 匹配控制器方法：[Controller::class, 'method']
        if handler_part.contains("::class") {
            if let Some(start) = handler_part.find('[') {
                if let Some(end) = handler_part.find(']') {
                    let inner = &handler_part[start + 1..end];
                    let parts: Vec<&str> = inner.split(',').collect();
                    if parts.len() == 2 {
                        let controller_raw = parts[0].trim().replace("::class", "");
                        let controller = controller_raw.trim_matches('\\');
                        let method = parts[1].trim().trim_matches('\'').trim_matches('"');
                        return format!("{}@{}", controller, method);
                    }
                }
            }
        }

        // 匹配闭包：function() {}
        if handler_part.contains("function") {
            return "Closure".to_string();
        }

        // 匹配控制器字符串：'Controller@method'
        if handler_part.contains('@') {
            if let Some(end) = handler_part.find(')') {
                let inner = &handler_part[..end];
                if let Some(start) = inner.rfind('\'') {
                    if let Some(prev) = inner[..start].rfind('\'') {
                        return inner[prev + 1..start].to_string();
                    }
                }
            }
        }

        "Unknown".to_string()
    }

    /// 生成路由缓存内容
    ///
    /// # 参数
    /// - `route_files`: 路由文件列表
    ///
    /// # 返回
    /// PHP 缓存文件内容
    pub fn generate_cache_content(
        route_files: &[(std::path::PathBuf, Vec<RouteInfo>)],
    ) -> String {
        let mut content = String::new();

        content.push_str("<?php\n");
        content.push_str("/**\n");
        content.push_str(" * 路由缓存文件\n");
        content.push_str(" * 由 OYTAPHP 自动生成\n");
        content.push_str(" * 不要手动编辑此文件\n");
        content.push_str(" */\n\n");
        content.push_str("return [\n");

        for (_, routes) in route_files {
            for route in routes {
                content.push_str(&format!(
                    "    // {} {}\n",
                    route.method, route.path
                ));
                content.push_str(&format!(
                    "    '{}' => [\n",
                    route.path
                ));
                content.push_str(&format!("        'method' => '{}',\n", route.method));
                content.push_str(&format!("        'handler' => '{}',\n", route.handler));
                content.push_str("        'middleware' => [],\n");
                content.push_str("    ],\n\n");
            }
        }

        content.push_str("];\n");

        content
    }
}
