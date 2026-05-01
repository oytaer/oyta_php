//! 门面类模块
//!
//! 本模块实现 PHP 门面类的注册机制
//! 门面类提供静态方法访问底层功能，对应 ThinkPHP 8.0 的门面模式

// 子模块声明
pub mod types;
pub mod cache;
pub mod session;
pub mod config;
pub mod env;
pub mod validate;
pub mod log;
pub mod event;
pub mod request;
pub mod response;
pub mod crypt;
pub mod coroutine;
pub mod timer;
pub mod search;
pub mod websocket;
pub mod queue;
pub mod upload;
pub mod route;
pub mod lang;
pub mod hash;
pub mod csrf;
pub mod date;
pub mod middleware;
pub mod db;
pub mod model;
pub mod storage;
pub mod view;
pub mod app;
pub mod lock;
pub mod servicediscovery;
pub mod composer;
pub mod cron;
pub mod container;
pub mod monitor;
pub mod serialize;

// 重新导出主要类型
pub use types::{FacadeClassDefinition, FacadeClassRegistry, FacadeMethod};

// 重新导出各门面类方法获取函数
pub use cache::get_cache_methods;
pub use session::get_session_methods;
pub use config::get_config_methods;
pub use env::get_env_methods;
pub use validate::get_validate_methods;
pub use log::get_log_methods;
pub use event::get_event_methods;
pub use request::get_request_methods;
pub use response::get_response_methods;
pub use crypt::get_crypt_methods;
pub use coroutine::get_coroutine_methods;
pub use timer::get_timer_methods;
pub use search::get_search_methods;
pub use websocket::get_websocket_methods;
pub use queue::get_queue_methods;
pub use upload::get_upload_methods;
pub use route::get_route_methods;
pub use lang::get_lang_methods;
pub use hash::get_hash_methods;
pub use csrf::get_csrf_methods;
pub use date::get_date_methods;
pub use middleware::get_middleware_methods;
pub use db::get_db_methods;
pub use model::get_model_methods;
pub use storage::get_storage_methods;
pub use view::get_view_methods;
pub use app::get_app_methods;
pub use lock::get_lock_methods;
pub use servicediscovery::get_servicediscovery_methods;
pub use composer::get_composer_methods;
pub use cron::get_cron_methods;
pub use container::get_container_methods;
pub use monitor::get_monitor_methods;
pub use serialize::get_serialize_methods;
