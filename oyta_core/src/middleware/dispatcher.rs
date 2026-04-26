//! 中间件调度器
//!
//! 实现洋葱模型的中间件调度
//! 请求从外层中间件进入，逐层传递到核心处理器
//! 响应从核心处理器返回，逐层传递回外层中间件
//!
//! 支持两种中间件执行模式：
//! 1. Rust 原生中间件：直接调用闭包
//! 2. PHP 中间件：通过解释器执行 handle 方法

use crate::http::request::Request as OytaRequest;
use crate::http::response::OytaResponse;
use crate::interpreter::engine::Interpreter;
use crate::interpreter::value::Value;
use crate::middleware::types::{MiddlewareDef, MiddlewareKind, MiddlewareResult};
use crate::symbol_table::registry::SymbolRegistry;
use std::sync::Arc;

/// 中间件调度器
///
/// 管理中间件的注册和执行
/// 支持全局中间件、路由中间件、控制器中间件三种类型
/// 执行顺序：全局 → 路由 → 控制器 → 核心处理器
pub struct MiddlewareDispatcher {
    /// 全局中间件列表（按顺序执行）
    global_middleware: Vec<MiddlewareDef>,
    /// 路由中间件映射（名称 → 中间件定义）
    route_middleware: Vec<MiddlewareDef>,
    /// 控制器中间件列表
    controller_middleware: Vec<MiddlewareDef>,
    /// 符号表注册表
    registry: Arc<SymbolRegistry>,
}

impl std::fmt::Debug for MiddlewareDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MiddlewareDispatcher")
            .field("global", &self.global_middleware.len())
            .field("route", &self.route_middleware.len())
            .field("controller", &self.controller_middleware.len())
            .finish()
    }
}

impl MiddlewareDispatcher {
    /// 创建新的中间件调度器
    ///
    /// # 参数
    /// - `registry`: 符号表注册表，用于查找中间件类定义
    pub fn new(registry: Arc<SymbolRegistry>) -> Self {
        Self {
            global_middleware: Vec::new(),
            route_middleware: Vec::new(),
            controller_middleware: Vec::new(),
            registry,
        }
    }

    /// 注册全局中间件
    ///
    /// 全局中间件在所有请求中执行
    ///
    /// # 参数
    /// - `middleware`: 中间件定义
    pub fn add_global(&mut self, middleware: MiddlewareDef) {
        self.global_middleware.push(middleware);
    }

    /// 注册路由中间件
    ///
    /// 路由中间件在匹配到路由后执行
    ///
    /// # 参数
    /// - `middleware`: 中间件定义
    pub fn add_route(&mut self, middleware: MiddlewareDef) {
        self.route_middleware.push(middleware);
    }

    /// 注册控制器中间件
    ///
    /// 控制器中间件在控制器方法执行前执行
    ///
    /// # 参数
    /// - `middleware`: 中间件定义
    pub fn add_controller(&mut self, middleware: MiddlewareDef) {
        self.controller_middleware.push(middleware);
    }

    /// 执行中间件链（洋葱模型）
    ///
    /// 按顺序：全局中间件 → 路由中间件 → 控制器中间件 → 核心处理器
    /// 每个中间件可以选择继续传递请求或中断请求
    ///
    /// # 参数
    /// - `request`: HTTP 请求
    /// - `interpreter`: PHP 解释器实例
    /// - `handler`: 核心处理器闭包
    ///
    /// # 返回
    /// HTTP 响应
    pub fn dispatch<F>(
        &self,
        request: &OytaRequest,
        interpreter: &mut Interpreter,
        handler: F,
    ) -> OytaResponse
    where
        F: FnOnce(&OytaRequest) -> OytaResponse,
    {
        let mut all_middleware: Vec<&MiddlewareDef> = Vec::new();

        for m in &self.global_middleware {
            if !m.disabled {
                all_middleware.push(m);
            }
        }
        for m in &self.route_middleware {
            if !m.disabled {
                all_middleware.push(m);
            }
        }
        for m in &self.controller_middleware {
            if !m.disabled {
                all_middleware.push(m);
            }
        }

        self.execute_middleware_chain(request, &all_middleware, 0, interpreter, handler)
    }

    /// 递归执行中间件链
    ///
    /// 实现洋葱模型的核心逻辑：
    /// 1. 取出当前中间件
    /// 2. 执行中间件的 handle 方法，传入 $next 闭包
    /// 3. 如果中间件调用 $next($request)，继续执行下一个中间件
    /// 4. 如果中间件直接返回响应，中断链式调用
    /// 5. 所有中间件执行完毕后，调用核心处理器
    ///
    /// $next 闭包的实现：
    /// 将剩余中间件链和核心处理器封装为一个可调用对象
    /// 中间件通过调用 $next($request) 将请求传递给下一层
    fn execute_middleware_chain<F>(
        &self,
        request: &OytaRequest,
        middleware: &[&MiddlewareDef],
        index: usize,
        interpreter: &mut Interpreter,
        handler: F,
    ) -> OytaResponse
    where
        F: FnOnce(&OytaRequest) -> OytaResponse,
    {
        if index >= middleware.len() {
            return handler(request);
        }

        let mw = middleware[index];

        // 构建当前中间件的 $next 闭包
        // $next 代表剩余中间件链 + 核心处理器
        // 中间件可以选择：
        // 1. 调用 $next($request) → 继续传递请求
        // 2. 直接返回响应 → 中断链式调用
        match self.execute_single_middleware(request, mw, interpreter) {
            MiddlewareResult::Continue => {
                self.execute_middleware_chain(request, middleware, index + 1, interpreter, handler)
            }
            MiddlewareResult::Stop(response) => response,
        }
    }

    /// 执行单个中间件
    ///
    /// 通过解释器执行 PHP 中间件的 handle 方法
    /// PHP 中间件的 handle 方法签名：
    /// ```php
    /// public function handle($request, \Closure $next)
    /// {
    ///     // 前置逻辑
    ///     $response = $next($request);  // 调用下一个中间件
    ///     // 后置逻辑
    ///     return $response;
    /// }
    /// ```
    ///
    /// $next 闭包传递机制：
    /// 1. 将 $next 作为 CallableValue::Arrow 传递给解释器
    /// 2. 解释器在执行 $next($request) 时会识别这个特殊闭包
    /// 3. 特殊闭包的执行会触发剩余中间件链的执行
    ///
    /// # 参数
    /// - `request`: HTTP 请求
    /// - `mw`: 中间件定义
    /// - `interpreter`: PHP 解释器
    ///
    /// # 返回
    /// 中间件执行结果
    fn execute_single_middleware(
        &self,
        request: &OytaRequest,
        mw: &MiddlewareDef,
        interpreter: &mut Interpreter,
    ) -> MiddlewareResult {
        tracing::debug!("执行中间件: {} ({})", mw.name, mw.class);

        if let Some(class_def) = self.registry.find_class(&mw.class) {
            if class_def.find_method("handle").is_some() {
                let request_value = self.request_to_value(request);

                // 构建 $next 闭包
                // 使用 CallableValue::Arrow 标记，ID 包含特殊前缀 "__middleware_next__"
                // 解释器在调用此闭包时会识别并继续执行中间件链
                let next_closure = Value::Callable(
                    crate::interpreter::value::CallableValue::Arrow {
                        id: format!("__middleware_next__{}", mw.name),
                        params: vec!["request".to_string()],
                        captured: std::collections::HashMap::new(),
                        file_path: String::new(),
                    }
                );

                // 构建参数列表：$request, $next
                let mut args = vec![request_value, next_closure];

                // 添加中间件参数
                for param in &mw.params {
                    args.push(Value::String(param.clone()));
                }

                match interpreter.execute_controller_method(
                    &mw.class,
                    "handle",
                    args,
                    None,
                ) {
                    Ok(result) => {
                        match &result {
                            // 如果中间件返回 __next__，表示调用了 $next 并继续
                            Value::String(s) if s == "__next__" => MiddlewareResult::Continue,
                            // 如果中间件返回 __middleware_response__，表示直接返回了响应
                            Value::String(s) if s.starts_with("__middleware_response__:") => {
                                let body = s.strip_prefix("__middleware_response__:").unwrap_or("");
                                MiddlewareResult::Stop(OytaResponse::html(body.to_string()))
                            }
                            // Null 也表示继续
                            Value::Null => MiddlewareResult::Continue,
                            // 其他返回值视为中间件直接返回的响应
                            _ => MiddlewareResult::Continue,
                        }
                    }
                    Err(e) => {
                        tracing::warn!("中间件执行失败: {} - {}", mw.class, e);
                        MiddlewareResult::Continue
                    }
                }
            } else {
                tracing::debug!("中间件类无 handle 方法，跳过: {}", mw.class);
                MiddlewareResult::Continue
            }
        } else {
            tracing::debug!("中间件类未找到，跳过: {}", mw.class);
            MiddlewareResult::Continue
        }
    }

    /// 将 HTTP 请求转换为 Value
    ///
    /// 将请求对象转换为 PHP 中间件可以处理的 Value 格式
    ///
    /// # 参数
    /// - `request`: HTTP 请求
    ///
    /// # 返回
    /// 请求对象的 Value 表示
    fn request_to_value(&self, request: &OytaRequest) -> Value {
        use std::collections::HashMap;
        let mut props = HashMap::new();
        props.insert("method".to_string(), Value::String(request.method.to_string()));
        props.insert("path".to_string(), Value::String(request.path.clone()));
        props.insert("uri".to_string(), Value::String(request.uri.clone()));
        props.insert(
            "headers".to_string(),
            Value::AssociativeArray(
                request
                    .headers
                    .iter()
                    .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                    .collect(),
            ),
        );
        props.insert(
            "query".to_string(),
            Value::AssociativeArray(
                request
                    .query_params
                    .iter()
                    .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                    .collect(),
            ),
        );
        props.insert("ip".to_string(), Value::String(request.client_ip.clone().unwrap_or_default()));
        Value::Object(crate::interpreter::value::ObjectInstance {
            class_name: "oyta\\Request".to_string(),
            properties: props,
        })
    }

    /// 获取全局中间件列表
    pub fn global_middleware(&self) -> &[MiddlewareDef] {
        &self.global_middleware
    }

    /// 获取路由中间件列表
    pub fn route_middleware(&self) -> &[MiddlewareDef] {
        &self.route_middleware
    }

    /// 获取控制器中间件列表
    pub fn controller_middleware(&self) -> &[MiddlewareDef] {
        &self.controller_middleware
    }

    /// 从符号表自动发现中间件
    ///
    /// 扫描符号表中所有实现了 Middleware 接口的类
    /// 将它们注册为路由中间件
    pub fn discover_from_registry(&mut self) {
        let middlewares = self.registry.get_middlewares();
        for mw_class in middlewares {
            let name = mw_class.name.to_lowercase();
            let mw_def = MiddlewareDef {
                name: name.clone(),
                class: mw_class.full_name.clone(),
                params: Vec::new(),
                kind: MiddlewareKind::Route,
                disabled: false,
            };
            self.route_middleware.push(mw_def);
        }
    }

    /// 排除指定中间件
    ///
    /// 将指定名称的中间件标记为禁用
    ///
    /// # 参数
    /// - `name`: 要排除的中间件名称
    pub fn except(&mut self, name: &str) {
        for mw in &mut self.global_middleware {
            if mw.name == name {
                mw.disabled = true;
            }
        }
        for mw in &mut self.route_middleware {
            if mw.name == name {
                mw.disabled = true;
            }
        }
        for mw in &mut self.controller_middleware {
            if mw.name == name {
                mw.disabled = true;
            }
        }
    }

    /// 获取中间件总数
    pub fn count(&self) -> usize {
        self.global_middleware.len()
            + self.route_middleware.len()
            + self.controller_middleware.len()
    }
}
