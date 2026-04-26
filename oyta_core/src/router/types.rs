//! 路由类型定义模块
//!
//! 定义路由系统需要的所有类型
//! 包括：路由项、路由组、HTTP 方法、路由参数等

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP 请求方法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    ANY,
}

impl HttpMethod {
    /// 从字符串解析 HTTP 方法
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "PATCH" => HttpMethod::PATCH,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            _ => HttpMethod::ANY,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::ANY => "ANY",
        }
    }

    /// 获取所有标准方法列表
    pub fn all_methods() -> Vec<HttpMethod> {
        vec![
            HttpMethod::GET,
            HttpMethod::POST,
            HttpMethod::PUT,
            HttpMethod::DELETE,
            HttpMethod::PATCH,
            HttpMethod::HEAD,
            HttpMethod::OPTIONS,
        ]
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 路由项
/// 定义一个完整的路由规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// 路由唯一标识
    pub id: String,
    /// HTTP 方法
    pub method: HttpMethod,
    /// 路由路径模式（如 "/user/:id"、"/api/v1/posts"）
    pub path: String,
    /// 目标处理器
    /// 格式: "控制器名@方法名" 或 "控制器名.方法名"
    /// 如 "app\\controller\\Index@index" 或闭包标识
    pub handler: RouteHandler,
    /// 路由所属组名
    pub group: Option<String>,
    /// 路由中间件列表
    pub middleware: Vec<String>,
    /// 路由参数约束（如 id 必须为数字）
    pub where_constraints: HashMap<String, String>,
    /// 路由额外参数
    pub defaults: HashMap<String, String>,
}

/// 路由处理器类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteHandler {
    /// 控制器方法
    /// 格式: "app\\controller\\Index@index"
    ControllerMethod {
        /// 完整控制器类名（含命名空间）
        controller: String,
        /// 方法名
        method: String,
    },
    /// 闭包处理器（用标识符表示，实际执行在解释器中）
    Closure {
        /// 闭包标识（文件路径:行号）
        id: String,
    },
    /// 静态响应（用于简单的路由重定向等）
    Static {
        /// 响应内容
        content: String,
        /// Content-Type
        content_type: String,
        /// HTTP 状态码
        status: u16,
    },
    /// 重定向
    Redirect {
        /// 目标 URL
        url: String,
        /// 状态码（301 或 302）
        status: u16,
    },
}

/// 路由匹配结果
#[derive(Debug, Clone)]
pub struct RouteMatch {
    /// 匹配到的路由
    pub route: Route,
    /// 提取的路由参数
    /// 如路径 "/user/123" 匹配 "/user/:id" 时，params = {"id": "123"}
    pub params: HashMap<String, String>,
}

/// 路由组
/// 用于批量设置路由的公共属性（前缀、中间件等）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteGroup {
    /// 组名
    pub name: String,
    /// 路径前缀
    pub prefix: String,
    /// 组内中间件列表
    pub middleware: Vec<String>,
    /// 组内路由项
    pub routes: Vec<Route>,
    /// 命名空间前缀（用于控制器类名）
    pub namespace: Option<String>,
}

impl RouteGroup {
    /// 创建新的路由组
    pub fn new(name: &str, prefix: &str) -> Self {
        Self {
            name: name.to_string(),
            prefix: prefix.to_string(),
            middleware: Vec::new(),
            routes: Vec::new(),
            namespace: None,
        }
    }

    /// 添加中间件到组
    pub fn middleware(mut self, name: &str) -> Self {
        self.middleware.push(name.to_string());
        self
    }

    /// 设置命名空间
    pub fn namespace(mut self, ns: &str) -> Self {
        self.namespace = Some(ns.to_string());
        self
    }

    /// 添加 GET 路由到组
    pub fn get(mut self, path: &str, handler: &str) -> Self {
        let full_path = format!("{}{}", self.prefix, path);
        let route = create_route(HttpMethod::GET, &full_path, handler, Some(&self.name), &self.middleware);
        self.routes.push(route);
        self
    }

    /// 添加 POST 路由到组
    pub fn post(mut self, path: &str, handler: &str) -> Self {
        let full_path = format!("{}{}", self.prefix, path);
        let route = create_route(HttpMethod::POST, &full_path, handler, Some(&self.name), &self.middleware);
        self.routes.push(route);
        self
    }

    /// 添加资源路由（RESTful 风格）
    /// 自动生成 index/create/store/read/edit/update/delete 七个路由
    pub fn resource(mut self, name: &str, controller: &str) -> Self {
        let ns = self.namespace.as_deref().unwrap_or("app\\controller");
        let full_controller = format!("{}\\{}", ns, controller);
        let prefix = self.prefix.clone();
        let mw = self.middleware.clone();
        let group_name = self.name.clone();

        // index: GET /name
        self.routes.push(Route {
            id: format!("{}.index", name),
            method: HttpMethod::GET,
            path: format!("{}/{}", prefix, name),
            handler: RouteHandler::ControllerMethod {
                controller: full_controller.clone(),
                method: "index".to_string(),
            },
            group: Some(group_name.clone()),
            middleware: mw.clone(),
            where_constraints: HashMap::new(),
            defaults: HashMap::new(),
        });

        // create: GET /name/create
        self.routes.push(Route {
            id: format!("{}.create", name),
            method: HttpMethod::GET,
            path: format!("{}/{}/create", prefix, name),
            handler: RouteHandler::ControllerMethod {
                controller: full_controller.clone(),
                method: "create".to_string(),
            },
            group: Some(group_name.clone()),
            middleware: mw.clone(),
            where_constraints: HashMap::new(),
            defaults: HashMap::new(),
        });

        // store: POST /name
        self.routes.push(Route {
            id: format!("{}.store", name),
            method: HttpMethod::POST,
            path: format!("{}/{}", prefix, name),
            handler: RouteHandler::ControllerMethod {
                controller: full_controller.clone(),
                method: "store".to_string(),
            },
            group: Some(group_name.clone()),
            middleware: mw.clone(),
            where_constraints: HashMap::new(),
            defaults: HashMap::new(),
        });

        // read: GET /name/:id
        self.routes.push(Route {
            id: format!("{}.read", name),
            method: HttpMethod::GET,
            path: format!("{}/{}/:id", prefix, name),
            handler: RouteHandler::ControllerMethod {
                controller: full_controller.clone(),
                method: "read".to_string(),
            },
            group: Some(group_name.clone()),
            middleware: mw.clone(),
            where_constraints: HashMap::new(),
            defaults: HashMap::new(),
        });

        // edit: GET /name/:id/edit
        self.routes.push(Route {
            id: format!("{}.edit", name),
            method: HttpMethod::GET,
            path: format!("{}/{}/:id/edit", prefix, name),
            handler: RouteHandler::ControllerMethod {
                controller: full_controller.clone(),
                method: "edit".to_string(),
            },
            group: Some(group_name.clone()),
            middleware: mw.clone(),
            where_constraints: HashMap::new(),
            defaults: HashMap::new(),
        });

        // update: PUT /name/:id
        self.routes.push(Route {
            id: format!("{}.update", name),
            method: HttpMethod::PUT,
            path: format!("{}/{}/:id", prefix, name),
            handler: RouteHandler::ControllerMethod {
                controller: full_controller.clone(),
                method: "update".to_string(),
            },
            group: Some(group_name.clone()),
            middleware: mw.clone(),
            where_constraints: HashMap::new(),
            defaults: HashMap::new(),
        });

        // delete: DELETE /name/:id
        self.routes.push(Route {
            id: format!("{}.delete", name),
            method: HttpMethod::DELETE,
            path: format!("{}/{}/:id", prefix, name),
            handler: RouteHandler::ControllerMethod {
                controller: full_controller,
                method: "delete".to_string(),
            },
            group: Some(group_name),
            middleware: mw,
            where_constraints: HashMap::new(),
            defaults: HashMap::new(),
        });

        self
    }
}

/// 创建路由项的辅助函数
fn create_route(
    method: HttpMethod,
    path: &str,
    handler: &str,
    group: Option<&str>,
    middleware: &[String],
) -> Route {
    // 解析处理器字符串
    // 支持格式: "Controller@method" 或 "Controller.method"
    let route_handler = if let Some((ctrl, meth)) = handler.split_once('@').or_else(|| handler.split_once('.')) {
        RouteHandler::ControllerMethod {
            controller: ctrl.to_string(),
            method: meth.to_string(),
        }
    } else {
        RouteHandler::Static {
            content: handler.to_string(),
            content_type: "text/plain".to_string(),
            status: 200,
        }
    };

    // 生成路由 ID
    let id = format!("{}:{}", method.as_str(), path);

    Route {
        id,
        method,
        path: path.to_string(),
        handler: route_handler,
        group: group.map(|g| g.to_string()),
        middleware: middleware.to_vec(),
        where_constraints: HashMap::new(),
        defaults: HashMap::new(),
    }
}
