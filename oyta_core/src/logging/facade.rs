//! 日志系统门面
//!
//! 提供静态方法访问日志管理器功能
//! 对应 ThinkPHP 8.0 的 Log 类
//!
//! # 使用示例
//! ```php
//! // 记录不同级别的日志
//! Log::info('user_action', '用户登录成功');
//! Log::error('system', '数据库连接失败');
//! Log::warning('security', '检测到异常登录尝试');
//! Log::debug('sql', 'SELECT * FROM users WHERE id = 1');
//! ```

use std::path::PathBuf;
use parking_lot::RwLock;

use super::channel::{LogManager, LogLevel, ChannelConfig};

/// 全局日志管理器实例
///
/// 使用 RwLock 实现线程安全的单例模式
/// 懒加载初始化，首次使用时创建
static LOG_MANAGER: RwLock<Option<LogManager>> = RwLock::new(None);

/// Log 门面结构体
///
/// 提供静态方法访问日志管理器
/// 所有方法都是线程安全的
pub struct Log;

impl Log {
    /// 初始化全局日志管理器
    ///
    /// 必须在使用其他方法前调用
    /// 通常在应用启动时自动调用
    ///
    /// # 参数
    /// - `log_dir`: 日志文件存储目录
    pub fn init(log_dir: &std::path::Path) {
        // 获取写锁
        let mut guard = LOG_MANAGER.write();
        // 创建新的日志管理器实例
        let mut manager = LogManager::new(log_dir);
        // 初始化默认通道配置
        manager.init_default_channels();
        // 存储到全局静态变量
        *guard = Some(manager);
    }

    /// 获取全局日志管理器实例
    ///
    /// 内部方法，用于获取管理器引用
    ///
    /// # 返回
    /// 日志管理器的可读引用 guard
    fn get_manager() -> parking_lot::RwLockReadGuard<'static, Option<LogManager>> {
        // 获取读锁
        LOG_MANAGER.read()
    }

    /// 记录日志
    ///
    /// 通用日志记录方法，可指定任意日志级别
    ///
    /// # 参数
    /// - `channel`: 日志通道名称，如 "default"、"error"、"sql"
    /// - `level`: 日志级别
    /// - `message`: 日志消息内容
    pub fn log(channel: &str, level: LogLevel, message: &str) {
        // 获取管理器读锁
        let guard = Self::get_manager();
        // 检查管理器是否已初始化
        if let Some(manager) = guard.as_ref() {
            // 调用管理器的 log 方法写入日志
            manager.log(channel, level, message);
        } else {
            // 未初始化时输出到标准错误
            eprintln!("[{}] [{}] {}", channel, level.as_str(), message);
        }
    }

    /// 记录 INFO 级别日志
    ///
    /// 用于记录一般信息性消息
    /// 如用户操作、业务流程等
    ///
    /// # 参数
    /// - `channel`: 日志通道名称
    /// - `message`: 日志消息内容
    ///
    /// # 示例
    /// ```php
    /// Log::info('user', '用户 ID:123 登录成功');
    /// ```
    pub fn info(channel: &str, message: &str) {
        // 调用通用 log 方法，使用 INFO 级别
        Self::log(channel, LogLevel::Info, message);
    }

    /// 记录 ERROR 级别日志
    ///
    /// 用于记录错误信息
    /// 如异常、失败操作等
    ///
    /// # 参数
    /// - `channel`: 日志通道名称
    /// - `message`: 日志消息内容
    ///
    /// # 示例
    /// ```php
    /// Log::error('system', '数据库连接超时');
    /// ```
    pub fn error(channel: &str, message: &str) {
        // 调用通用 log 方法，使用 ERROR 级别
        Self::log(channel, LogLevel::Error, message);
    }

    /// 记录 WARNING 级别日志
    ///
    /// 用于记录警告信息
    /// 如潜在问题、不推荐用法等
    ///
    /// # 参数
    /// - `channel`: 日志通道名称
    /// - `message`: 日志消息内容
    ///
    /// # 示例
    /// ```php
    /// Log::warning('security', '登录失败次数过多');
    /// ```
    pub fn warning(channel: &str, message: &str) {
        // 调用通用 log 方法，使用 WARNING 级别
        Self::log(channel, LogLevel::Warning, message);
    }

    /// 记录 DEBUG 级别日志
    ///
    /// 用于记录调试信息
    /// 如变量值、执行路径等
    /// 仅在调试模式下输出
    ///
    /// # 参数
    /// - `channel`: 日志通道名称
    /// - `message`: 日志消息内容
    ///
    /// # 示例
    /// ```php
    /// Log::debug('sql', 'SELECT * FROM users WHERE id = 1');
    /// ```
    pub fn debug(channel: &str, message: &str) {
        // 调用通用 log 方法，使用 DEBUG 级别
        Self::log(channel, LogLevel::Debug, message);
    }

    /// 记录 CRITICAL 级别日志
    ///
    /// 用于记录严重错误
    /// 如系统崩溃、数据丢失等
    ///
    /// # 参数
    /// - `channel`: 日志通道名称
    /// - `message`: 日志消息内容
    pub fn critical(channel: &str, message: &str) {
        // 调用通用 log 方法，使用 CRITICAL 级别
        Self::log(channel, LogLevel::Critical, message);
    }

    /// 记录 ALERT 级别日志
    ///
    /// 用于记录需要立即处理的警报
    /// 如安全漏洞、服务不可用等
    ///
    /// # 参数
    /// - `channel`: 日志通道名称
    /// - `message`: 日志消息内容
    pub fn alert(channel: &str, message: &str) {
        // 调用通用 log 方法，使用 ALERT 级别
        Self::log(channel, LogLevel::Alert, message);
    }

    /// 记录 EMERGENCY 级别日志
    ///
    /// 用于记录紧急情况
    /// 如系统不可用、需要立即干预
    ///
    /// # 参数
    /// - `channel`: 日志通道名称
    /// - `message`: 日志消息内容
    pub fn emergency(channel: &str, message: &str) {
        // 调用通用 log 方法，使用 EMERGENCY 级别
        Self::log(channel, LogLevel::Emergency, message);
    }

    /// 记录 NOTICE 级别日志
    ///
    /// 用于记录正常但值得注意的事件
    /// 如任务完成、配置变更等
    ///
    /// # 参数
    /// - `channel`: 日志通道名称
    /// - `message`: 日志消息内容
    pub fn notice(channel: &str, message: &str) {
        // 调用通用 log 方法，使用 NOTICE 级别
        Self::log(channel, LogLevel::Notice, message);
    }

    /// 添加自定义日志通道
    ///
    /// 注册新的日志通道配置
    ///
    /// # 参数
    /// - `name`: 通道名称
    /// - `config`: 通道配置
    ///
    /// # 示例
    /// ```php
    /// Log::addChannel('api', [
    ///     'type' => 'file',
    ///     'level' => 'info',
    ///     'path' => 'runtime/log/api.log',
    /// ]);
    /// ```
    pub fn add_channel(name: &str, config: ChannelConfig) {
        // 获取写锁
        let mut guard = LOG_MANAGER.write();
        // 检查管理器是否已初始化
        if let Some(manager) = guard.as_mut() {
            // 调用管理器的 add_channel 方法添加通道
            manager.add_channel(name, config);
        } else {
            // 未初始化时记录警告
            tracing::warn!("Log 门面未初始化，无法添加通道: {}", name);
        }
    }

    /// 获取日志文件路径
    ///
    /// 返回指定通道的日志文件路径
    ///
    /// # 参数
    /// - `channel`: 通道名称
    ///
    /// # 返回
    /// 日志文件的完整路径
    pub fn get_log_path(channel: &str) -> PathBuf {
        // 获取管理器读锁
        let guard = Self::get_manager();
        // 检查管理器是否已初始化
        if let Some(manager) = guard.as_ref() {
            // 返回通道的日志文件路径
            manager.get_log_path(channel)
        } else {
            // 未初始化时返回默认路径
            PathBuf::from(format!("runtime/log/{}.log", channel))
        }
    }

    /// 获取通道配置
    ///
    /// 返回指定通道的配置信息
    ///
    /// # 参数
    /// - `channel`: 通道名称
    ///
    /// # 返回
    /// 通道配置的可选引用
    pub fn get_channel(channel: &str) -> Option<ChannelConfig> {
        // 获取管理器读锁
        let guard = Self::get_manager();
        // 检查管理器是否已初始化
        if let Some(manager) = guard.as_ref() {
            // 克隆通道配置返回
            manager.get_channel(channel).cloned()
        } else {
            // 未初始化时返回 None
            None
        }
    }

    /// 检查日志门面是否已初始化
    ///
    /// 用于判断是否需要调用 init 方法
    ///
    /// # 返回
    /// 如果已初始化返回 true，否则返回 false
    pub fn is_initialized() -> bool {
        // 获取读锁检查是否有值
        let guard = LOG_MANAGER.read();
        guard.is_some()
    }

    /// 重置日志门面
    ///
    /// 清除日志管理器实例
    /// 主要用于测试环境
    pub fn reset() {
        // 获取写锁
        let mut guard = LOG_MANAGER.write();
        // 清空管理器实例
        *guard = None;
    }

    /// 写入 SQL 日志
    ///
    /// 便捷方法，专门用于记录 SQL 查询
    /// 自动写入 sql 通道
    ///
    /// # 参数
    /// - `sql`: SQL 语句
    /// - `bindings`: 绑定参数（可选）
    pub fn sql(sql: &str, bindings: Option<&Vec<String>>) {
        // 构建日志消息
        let message = if let Some(binds) = bindings {
            // 包含绑定参数的日志
            format!("{} [bindings: {:?}]", sql, binds)
        } else {
            // 不包含绑定参数的日志
            sql.to_string()
        };
        // 写入 sql 通道
        Self::debug("sql", &message);
    }

    /// 写入请求日志
    ///
    /// 便捷方法，用于记录 HTTP 请求
    /// 自动写入 request 通道
    ///
    /// # 参数
    /// - `method`: HTTP 方法
    /// - `path`: 请求路径
    /// - `status`: 响应状态码
    /// - `duration`: 请求耗时（毫秒）
    pub fn request(method: &str, path: &str, status: u16, duration: u64) {
        // 构建请求日志消息
        let message = format!("{} {} - {} ({}ms)", method, path, status, duration);
        // 根据状态码选择日志级别
        if status >= 500 {
            // 服务端错误使用 ERROR 级别
            Self::error("request", &message);
        } else if status >= 400 {
            // 客户端错误使用 WARNING 级别
            Self::warning("request", &message);
        } else {
            // 正常响应使用 INFO 级别
            Self::info("request", &message);
        }
    }

    /// 写入异常日志
    ///
    /// 便捷方法，用于记录异常信息
    /// 自动写入 error 通道
    ///
    /// # 参数
    /// - `exception`: 异常类型
    /// - `message`: 异常消息
    /// - `file`: 文件路径（可选）
    /// - `line`: 行号（可选）
    pub fn exception(exception: &str, message: &str, file: Option<&str>, line: Option<u32>) {
        // 构建异常日志消息
        let location = match (file, line) {
            // 包含文件和行号
            (Some(f), Some(l)) => format!(" at {}:{}", f, l),
            // 仅包含文件
            (Some(f), None) => format!(" at {}", f),
            // 不包含位置信息
            _ => String::new(),
        };
        // 构建完整消息
        let full_message = format!("{}: {}{}", exception, message, location);
        // 写入 error 通道
        Self::error("error", &full_message);
    }
}
