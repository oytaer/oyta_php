//! Request 门面类
//!
//! 提供静态方法访问当前请求对象
//! 对应 ThinkPHP 的 think\facade\Request 类

use std::sync::RwLock;
use crate::http::request::{Request, UploadedFile, VariableModifier};
use serde_json::Value;
use std::collections::HashMap;

/// 全局请求对象存储
static CURRENT_REQUEST: RwLock<Option<Request>> = RwLock::new(None);

/// Request 门面类
/// 提供静态方法访问当前请求对象
pub struct RequestFacade;

impl RequestFacade {
    /// 设置当前请求对象
    pub fn set_request(request: Request) {
        let mut current = CURRENT_REQUEST.write().unwrap();
        *current = Some(request);
    }

    /// 获取当前请求对象
    pub fn get_request() -> Option<Request> {
        let current = CURRENT_REQUEST.read().unwrap();
        current.clone()
    }

    /// 清除当前请求对象
    pub fn clear_request() {
        let mut current = CURRENT_REQUEST.write().unwrap();
        *current = None;
    }

    /// 获取当前请求对象（带回调）
    fn with_request<F, T>(f: F) -> T
    where
        F: FnOnce(&Request) -> T,
        T: Default,
    {
        let current = CURRENT_REQUEST.read().unwrap();
        if let Some(ref request) = *current {
            f(request)
        } else {
            T::default()
        }
    }

    // ==================== 请求信息获取方法 ====================

    /// 获取当前访问域名或IP
    pub fn host() -> String {
        Self::with_request(|r| r.host())
    }

    /// 获取当前访问协议
    pub fn scheme() -> String {
        Self::with_request(|r| r.scheme())
    }

    /// 获取当前访问的端口
    pub fn port() -> u16 {
        Self::with_request(|r| r.port())
    }

    /// 获取当前请求的 REMOTE_PORT
    pub fn remote_port() -> Option<u16> {
        Self::with_request(|r| r.remote_port())
    }

    /// 获取当前请求的 SERVER_PROTOCOL
    pub fn protocol() -> String {
        Self::with_request(|r| r.protocol())
    }

    /// 获取当前包含协议的域名
    pub fn domain() -> String {
        Self::with_request(|r| r.domain())
    }

    /// 获取当前访问的子域名
    pub fn sub_domain() -> String {
        Self::with_request(|r| r.sub_domain())
    }

    /// 获取当前访问的泛域名
    pub fn pan_domain() -> String {
        Self::with_request(|r| r.pan_domain())
    }

    /// 获取当前访问的根域名
    pub fn root_domain() -> String {
        Self::with_request(|r| r.root_domain())
    }

    /// 获取当前完整URL
    pub fn url(complete: bool) -> String {
        Self::with_request(|r| r.url(complete))
    }

    /// 获取当前URL（不含QUERY_STRING）
    pub fn base_url(complete: bool) -> String {
        Self::with_request(|r| r.base_url(complete))
    }

    /// 获取当前请求的QUERY_STRING参数
    pub fn query_string() -> String {
        Self::with_request(|r| r.query_string_value())
    }

    /// 获取当前执行的文件
    pub fn base_file() -> String {
        Self::with_request(|r| r.base_file())
    }

    /// 获取URL访问根地址
    pub fn root(complete: bool) -> String {
        Self::with_request(|r| r.root(complete))
    }

    /// 获取URL访问根目录
    pub fn root_url() -> String {
        Self::with_request(|r| r.root_url())
    }

    /// 获取当前请求URL的pathinfo信息
    pub fn path_info() -> String {
        Self::with_request(|r| r.path_info())
    }

    /// 获取当前URL的访问后缀
    pub fn ext() -> String {
        Self::with_request(|r| r.ext())
    }

    /// 获取当前请求的时间
    pub fn time() -> i64 {
        Self::with_request(|r| r.time())
    }

    /// 获取当前请求的资源类型
    pub fn type_name() -> String {
        Self::with_request(|r| r.type_name())
    }

    /// 获取当前请求类型
    pub fn method(original: bool) -> String {
        Self::with_request(|r| r.method(original))
    }

    /// 获取当前请求的路由对象实例
    pub fn rule() -> Option<String> {
        Self::with_request(|r| r.rule().cloned())
    }

    // ==================== 控制器/操作信息获取 ====================

    /// 获取当前请求的控制器名
    pub fn controller(lowercase: bool, base: bool) -> String {
        Self::with_request(|r| r.controller_name(lowercase, base))
    }

    /// 获取当前请求的操作名
    pub fn action(lowercase: bool) -> String {
        Self::with_request(|r| r.action_name(lowercase))
    }

    /// 获取当前请求的控制器分级目录
    pub fn layer() -> String {
        Self::with_request(|r| r.layer_name())
    }

    /// 获取当前应用名
    pub fn app_name() -> String {
        Self::with_request(|r| r.app_name_value())
    }

    // ==================== 变量获取方法 ====================

    /// 获取PARAM变量
    pub fn param(key: &str) -> Option<String> {
        Self::with_request(|r| r.param_value(key))
    }

    /// 获取PARAM变量（带默认值）
    pub fn param_default(key: &str, default: &str) -> String {
        Self::with_request(|r| r.param_default(key, default))
    }

    /// 获取所有PARAM变量
    pub fn param_all(filter: bool) -> HashMap<String, String> {
        Self::with_request(|r| r.param_all(filter))
    }

    /// 获取GET变量
    pub fn get(key: &str) -> Option<String> {
        Self::with_request(|r| r.get(key))
    }

    /// 获取GET变量（带默认值）
    pub fn get_default(key: &str, default: &str) -> String {
        Self::with_request(|r| r.get_default(key, default))
    }

    /// 获取所有GET变量
    pub fn get_all() -> HashMap<String, String> {
        Self::with_request(|r| r.get_all().clone())
    }

    /// 获取POST变量
    pub fn post(key: &str) -> Option<String> {
        Self::with_request(|r| r.post_value(key))
    }

    /// 获取POST变量（带默认值）
    pub fn post_default(key: &str, default: &str) -> String {
        Self::with_request(|r| r.post(key, default))
    }

    /// 获取所有POST变量
    pub fn post_all() -> HashMap<String, String> {
        Self::with_request(|r| r.post_all().clone())
    }

    /// 获取PUT变量
    pub fn put(key: &str) -> Option<String> {
        Self::with_request(|r| r.put(key))
    }

    /// 获取PUT变量（带默认值）
    pub fn put_default(key: &str, default: &str) -> String {
        Self::with_request(|r| r.put_default(key, default))
    }

    /// 获取所有PUT变量
    pub fn put_all() -> HashMap<String, String> {
        Self::with_request(|r| r.put_all().clone())
    }

    /// 获取DELETE变量
    pub fn delete(key: &str) -> Option<String> {
        Self::with_request(|r| r.delete(key))
    }

    /// 获取DELETE变量（带默认值）
    pub fn delete_default(key: &str, default: &str) -> String {
        Self::with_request(|r| r.delete_default(key, default))
    }

    /// 获取所有DELETE变量
    pub fn delete_all() -> HashMap<String, String> {
        Self::with_request(|r| r.delete_all().clone())
    }

    /// 获取SESSION变量
    pub fn session(key: &str) -> Option<Value> {
        Self::with_request(|r| r.session(key).cloned())
    }

    /// 获取SESSION变量（带默认值）
    pub fn session_default(key: &str, default: Value) -> Value {
        Self::with_request(|r| r.session_default(key, default))
    }

    /// 获取所有SESSION变量
    pub fn session_all() -> HashMap<String, Value> {
        Self::with_request(|r| r.session_all().clone())
    }

    /// 获取COOKIE变量
    pub fn cookie(key: &str) -> Option<String> {
        Self::with_request(|r| r.cookie(key))
    }

    /// 获取COOKIE变量（带默认值）
    pub fn cookie_default(key: &str, default: &str) -> String {
        Self::with_request(|r| r.cookie_default(key, default))
    }

    /// 获取所有COOKIE变量
    pub fn cookie_all() -> HashMap<String, String> {
        Self::with_request(|r| r.cookie_all().clone())
    }

    /// 获取REQUEST变量
    pub fn request(key: &str) -> Option<String> {
        Self::with_request(|r| r.request(key))
    }

    /// 获取REQUEST变量（带默认值）
    pub fn request_default(key: &str, default: &str) -> String {
        Self::with_request(|r| r.request_default(key, default))
    }

    /// 获取SERVER变量
    pub fn server(key: &str) -> Option<String> {
        Self::with_request(|r| r.server(key))
    }

    /// 获取SERVER变量（带默认值）
    pub fn server_default(key: &str, default: &str) -> String {
        Self::with_request(|r| r.server_default(key, default))
    }

    /// 获取所有SERVER变量
    pub fn server_all() -> HashMap<String, String> {
        Self::with_request(|r| r.server_all().clone())
    }

    /// 获取ENV变量
    pub fn env(key: &str) -> Option<String> {
        Self::with_request(|r| r.env(key))
    }

    /// 获取ENV变量（带默认值）
    pub fn env_default(key: &str, default: &str) -> String {
        Self::with_request(|r| r.env_default(key, default))
    }

    /// 获取所有ENV变量
    pub fn env_all() -> HashMap<String, String> {
        Self::with_request(|r| r.env_all().clone())
    }

    /// 获取路由变量
    pub fn route(key: &str) -> Option<String> {
        Self::with_request(|r| r.route(key))
    }

    /// 获取路由变量（带默认值）
    pub fn route_default(key: &str, default: &str) -> String {
        Self::with_request(|r| r.route_default(key, default))
    }

    /// 获取所有路由变量
    pub fn route_all() -> HashMap<String, String> {
        Self::with_request(|r| r.route_all().clone())
    }

    /// 获取中间件传递的变量
    pub fn middleware(key: &str) -> Option<Value> {
        Self::with_request(|r| r.middleware(key).cloned())
    }

    /// 获取中间件传递的变量（带默认值）
    pub fn middleware_default(key: &str, default: Value) -> Value {
        Self::with_request(|r| r.middleware_default(key, default))
    }

    /// 获取所有中间件传递的变量
    pub fn middleware_all() -> HashMap<String, Value> {
        Self::with_request(|r| r.middleware_all().clone())
    }

    /// 获取上传文件
    pub fn file(key: &str) -> Option<Vec<UploadedFile>> {
        Self::with_request(|r| r.file(key).cloned())
    }

    /// 获取单个上传文件
    pub fn file_one(key: &str) -> Option<UploadedFile> {
        Self::with_request(|r| r.file_one(key).cloned())
    }

    /// 获取所有上传文件
    pub fn file_all() -> HashMap<String, Vec<UploadedFile>> {
        Self::with_request(|r| r.file_all().clone())
    }

    /// 获取所有请求变量（包括文件）
    pub fn all() -> HashMap<String, Value> {
        Self::with_request(|r| r.all())
    }

    // ==================== 变量操作方法 ====================

    /// 检测变量是否设置
    pub fn has(key: &str, method: &str) -> bool {
        Self::with_request(|r| r.has(key, method))
    }

    /// 获取部分变量
    pub fn only(keys: &[&str], method: &str) -> HashMap<String, String> {
        Self::with_request(|r| r.only(keys, method))
    }

    /// 排除某些变量后获取
    pub fn except(keys: &[&str], method: &str) -> HashMap<String, String> {
        Self::with_request(|r| r.except(keys, method))
    }

    // ==================== 变量修饰符支持 ====================

    /// 获取变量并应用修饰符
    pub fn param_with_modifier(key: &str) -> Value {
        Self::with_request(|r| r.param_with_modifier(key))
    }

    /// 获取GET变量并应用修饰符
    pub fn get_with_modifier(key: &str) -> Value {
        Self::with_request(|r| r.get_with_modifier(key))
    }

    /// 获取POST变量并应用修饰符
    pub fn post_with_modifier(key: &str) -> Value {
        Self::with_request(|r| r.post_with_modifier(key))
    }

    // ==================== 请求类型判断方法 ====================

    /// 判断是否为 GET 请求
    pub fn is_get() -> bool {
        Self::with_request(|r| r.is_get())
    }

    /// 判断是否为 POST 请求
    pub fn is_post() -> bool {
        Self::with_request(|r| r.is_post())
    }

    /// 判断是否为 PUT 请求
    pub fn is_put() -> bool {
        Self::with_request(|r| r.is_put())
    }

    /// 判断是否为 DELETE 请求
    pub fn is_delete() -> bool {
        Self::with_request(|r| r.is_delete())
    }

    /// 判断是否为 HEAD 请求
    pub fn is_head() -> bool {
        Self::with_request(|r| r.is_head())
    }

    /// 判断是否为 PATCH 请求
    pub fn is_patch() -> bool {
        Self::with_request(|r| r.is_patch())
    }

    /// 判断是否为 OPTIONS 请求
    pub fn is_options() -> bool {
        Self::with_request(|r| r.is_options())
    }

    /// 判断是否为 AJAX 请求
    pub fn is_ajax() -> bool {
        Self::with_request(|r| r.is_ajax())
    }

    /// 判断是否为 PJAX 请求
    pub fn is_pjax() -> bool {
        Self::with_request(|r| r.is_pjax())
    }

    /// 判断是否为 JSON 请求
    pub fn is_json() -> bool {
        Self::with_request(|r| r.is_json())
    }

    /// 判断是否为手机访问
    pub fn is_mobile() -> bool {
        Self::with_request(|r| r.is_mobile())
    }

    /// 判断是否为 CLI 模式
    pub fn is_cli() -> bool {
        Self::with_request(|r| r.is_cli_mode())
    }

    /// 判断是否为 CGI 模式
    pub fn is_cgi() -> bool {
        Self::with_request(|r| r.is_cgi_mode())
    }

    // ==================== HTTP头信息方法 ====================

    /// 获取请求头
    pub fn header(key: &str) -> Option<String> {
        Self::with_request(|r| r.header(key))
    }

    /// 获取请求头（带默认值）
    pub fn header_default(key: &str, default: &str) -> String {
        Self::with_request(|r| r.header_default(key, default))
    }

    /// 获取所有HTTP头信息
    pub fn header_all() -> HashMap<String, String> {
        Self::with_request(|r| r.header_all())
    }

    // ==================== 其他方法 ====================

    /// 获取客户端 IP
    pub fn ip() -> String {
        Self::with_request(|r| r.ip())
    }

    /// 获取 Content-Type
    pub fn content_type() -> Option<String> {
        Self::with_request(|r| r.content_type())
    }

    /// 获取 JSON 请求体中的字段
    pub fn json(key: &str) -> Option<Value> {
        Self::with_request(|r| r.json(key).cloned())
    }

    /// 获取请求体原始字节
    pub fn body() -> Option<Vec<u8>> {
        Self::with_request(|r| r.body.clone())
    }

    /// 获取请求路径
    pub fn path() -> String {
        Self::with_request(|r| r.path.clone())
    }

    /// 获取请求URI
    pub fn uri() -> String {
        Self::with_request(|r| r.uri.clone())
    }

    /// 获取HTTP版本
    pub fn version() -> String {
        Self::with_request(|r| r.version.clone())
    }
}
