//! 门面类注册模块
//!
//! 本模块实现 PHP 门面类的注册机制
//! 门面类提供静态方法访问底层功能，对应 ThinkPHP 8.0 的门面模式
//!
//! # 已注册的门面类
//! - Cache: 缓存门面
//! - Db: 数据库门面
//! - Session: 会话门面
//! - Config: 配置门面
//! - Env: 环境变量门面
//! - View: 视图门面
//! - Validate: 验证门面
//! - Log: 日志门面
//! - Event: 事件门面
//! - Request: 请求门面
//! - Response: 响应门面
//! - Crypt: 加密门面
//! - Coroutine: 协程门面
//! - Timer: 定时器门面
//! - Search: 搜索门面
//!
//! # 使用示例 (PHP代码)
//! ```php
//! // 缓存操作
//! Cache::set('key', 'value', 3600);
//! $value = Cache::get('key');
//!
//! // 数据库操作
//! $users = Db::query('SELECT * FROM users');
//! Db::table('users')->insert(['name' => 'test']);
//!
//! // 会话操作
//! Session::set('user_id', 1);
//! $userId = Session::get('user_id');
//!
//! // 配置操作
//! $debug = Config::get('app.debug');
//! Config::set('app.debug', true);
//!
//! // 日志操作
//! Log::info('用户登录', ['user_id' => 1]);
//! Log::error('系统错误', ['message' => $e->getMessage()]);
//! ```

use std::sync::OnceLock;

// 从 builtin_facade 子模块导入类型和方法
pub use super::builtin_facade::{
    FacadeClassDefinition, FacadeClassRegistry, FacadeMethod,
    get_cache_methods, get_session_methods, get_config_methods, get_env_methods,
    get_validate_methods, get_log_methods, get_event_methods, get_request_methods,
    get_response_methods, get_crypt_methods, get_coroutine_methods, get_timer_methods,
    get_search_methods, get_websocket_methods, get_queue_methods, get_upload_methods,
    get_route_methods, get_lang_methods, get_hash_methods, get_csrf_methods,
    get_date_methods, get_middleware_methods, get_db_methods, get_model_methods,
    get_storage_methods, get_view_methods, get_app_methods,
};

/// 全局门面类注册表
///
/// 使用懒加载初始化
pub fn get_facade_class_registry() -> &'static FacadeClassRegistry {
    static REGISTRY: OnceLock<FacadeClassRegistry> = OnceLock::new();
    REGISTRY.get_or_init(create_facade_class_registry)
}

/// 创建并填充门面类注册表
///
/// # 返回
/// 包含所有门面类的注册表
pub fn create_facade_class_registry() -> FacadeClassRegistry {
    let mut registry = FacadeClassRegistry::new();

    // 注册 Cache 门面类
    let mut cache_class = FacadeClassDefinition::new("Cache", "缓存门面类");
    for (name, method) in get_cache_methods() {
        cache_class = cache_class.add_static_method(name, method);
    }
    registry.register(cache_class);

    // 注册 Session 门面类
    let mut session_class = FacadeClassDefinition::new("Session", "会话门面类");
    for (name, method) in get_session_methods() {
        session_class = session_class.add_static_method(name, method);
    }
    registry.register(session_class);

    // 注册 Config 门面类
    let mut config_class = FacadeClassDefinition::new("Config", "配置门面类");
    for (name, method) in get_config_methods() {
        config_class = config_class.add_static_method(name, method);
    }
    registry.register(config_class);

    // 注册 Env 门面类
    let mut env_class = FacadeClassDefinition::new("Env", "环境变量门面类");
    for (name, method) in get_env_methods() {
        env_class = env_class.add_static_method(name, method);
    }
    registry.register(env_class);

    // 注册 Validate 门面类
    let mut validate_class = FacadeClassDefinition::new("Validate", "验证门面类");
    for (name, method) in get_validate_methods() {
        validate_class = validate_class.add_static_method(name, method);
    }
    registry.register(validate_class);

    // 注册 Log 门面类
    let mut log_class = FacadeClassDefinition::new("Log", "日志门面类");
    for (name, method) in get_log_methods() {
        log_class = log_class.add_static_method(name, method);
    }
    registry.register(log_class);

    // 注册 Event 门面类
    let mut event_class = FacadeClassDefinition::new("Event", "事件门面类");
    for (name, method) in get_event_methods() {
        event_class = event_class.add_static_method(name, method);
    }
    registry.register(event_class);

    // 注册 Request 门面类
    let mut request_class = FacadeClassDefinition::new("Request", "请求门面类");
    for (name, method) in get_request_methods() {
        request_class = request_class.add_static_method(name, method);
    }
    registry.register(request_class);

    // 注册 Response 门面类
    let mut response_class = FacadeClassDefinition::new("Response", "响应门面类");
    for (name, method) in get_response_methods() {
        response_class = response_class.add_static_method(name, method);
    }
    registry.register(response_class);

    // 注册 Crypt 门面类
    let mut crypt_class = FacadeClassDefinition::new("Crypt", "加密门面类");
    for (name, method) in get_crypt_methods() {
        crypt_class = crypt_class.add_static_method(name, method);
    }
    registry.register(crypt_class);

    // 注册 Coroutine 门面类
    let mut coroutine_class = FacadeClassDefinition::new("Coroutine", "协程门面类");
    for (name, method) in get_coroutine_methods() {
        coroutine_class = coroutine_class.add_static_method(name, method);
    }
    registry.register(coroutine_class);

    // 注册 Timer 门面类
    let mut timer_class = FacadeClassDefinition::new("Timer", "定时器门面类");
    for (name, method) in get_timer_methods() {
        timer_class = timer_class.add_static_method(name, method);
    }
    registry.register(timer_class);

    // 注册 Search 门面类
    let mut search_class = FacadeClassDefinition::new("Search", "搜索门面类");
    for (name, method) in get_search_methods() {
        search_class = search_class.add_static_method(name, method);
    }
    registry.register(search_class);

    // 注册 WebSocket 门面类
    let mut websocket_class = FacadeClassDefinition::new("WebSocket", "WebSocket门面类");
    for (name, method) in get_websocket_methods() {
        websocket_class = websocket_class.add_static_method(name, method);
    }
    registry.register(websocket_class);

    // 注册 Queue 门面类
    let mut queue_class = FacadeClassDefinition::new("Queue", "队列门面类");
    for (name, method) in get_queue_methods() {
        queue_class = queue_class.add_static_method(name, method);
    }
    registry.register(queue_class);

    // 注册 Upload 门面类
    let mut upload_class = FacadeClassDefinition::new("Upload", "上传门面类");
    for (name, method) in get_upload_methods() {
        upload_class = upload_class.add_static_method(name, method);
    }
    registry.register(upload_class);

    // 注册 Route 门面类
    let mut route_class = FacadeClassDefinition::new("Route", "路由门面类");
    for (name, method) in get_route_methods() {
        route_class = route_class.add_static_method(name, method);
    }
    registry.register(route_class);

    // 注册 Lang 门面类
    let mut lang_class = FacadeClassDefinition::new("Lang", "语言门面类");
    for (name, method) in get_lang_methods() {
        lang_class = lang_class.add_static_method(name, method);
    }
    registry.register(lang_class);

    // 注册 Hash 门面类
    let mut hash_class = FacadeClassDefinition::new("Hash", "哈希门面类");
    for (name, method) in get_hash_methods() {
        hash_class = hash_class.add_static_method(name, method);
    }
    registry.register(hash_class);

    // 注册 Csrf 门面类
    let mut csrf_class = FacadeClassDefinition::new("Csrf", "CSRF门面类");
    for (name, method) in get_csrf_methods() {
        csrf_class = csrf_class.add_static_method(name, method);
    }
    registry.register(csrf_class);

    // 注册 Date 门面类
    let mut date_class = FacadeClassDefinition::new("Date", "日期门面类");
    for (name, method) in get_date_methods() {
        date_class = date_class.add_static_method(name, method);
    }
    registry.register(date_class);

    // 注册 Middleware 门面类
    let mut middleware_class = FacadeClassDefinition::new("Middleware", "中间件门面类");
    for (name, method) in get_middleware_methods() {
        middleware_class = middleware_class.add_static_method(name, method);
    }
    registry.register(middleware_class);

    // 注册 Db 门面类
    let mut db_class = FacadeClassDefinition::new("Db", "数据库门面类");
    for (name, method) in get_db_methods() {
        db_class = db_class.add_static_method(name, method);
    }
    registry.register(db_class);

    // 注册 Model 门面类
    let mut model_class = FacadeClassDefinition::new("Model", "模型门面类");
    for (name, method) in get_model_methods() {
        model_class = model_class.add_static_method(name, method);
    }
    registry.register(model_class);

    // 注册 Storage 门面类
    let mut storage_class = FacadeClassDefinition::new("Storage", "存储门面类");
    for (name, method) in get_storage_methods() {
        storage_class = storage_class.add_static_method(name, method);
    }
    registry.register(storage_class);

    // 注册 View 门面类
    let mut view_class = FacadeClassDefinition::new("View", "视图门面类");
    for (name, method) in get_view_methods() {
        view_class = view_class.add_static_method(name, method);
    }
    registry.register(view_class);

    // 注册 App 门面类
    let mut app_class = FacadeClassDefinition::new("App", "应用门面类");
    for (name, method) in get_app_methods() {
        app_class = app_class.add_static_method(name, method);
    }
    registry.register(app_class);

    registry
}
