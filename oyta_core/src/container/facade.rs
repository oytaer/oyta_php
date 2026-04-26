//! Container 门面模块
//!
//! 对应 ThinkPHP 8.0 的 Container 门面
//! 提供静态方法入口，简化 IoC 容器操作
//! 所有方法委托给全局容器实例

use std::any::Any;
use std::sync::Arc;

use super::container;

/// Container 门面
///
/// 对应 ThinkPHP 8.0 的 \oyta\Container 类
/// 提供静态方法入口，所有方法委托给全局容器
///
/// 使用示例（PHP 代码风格）：
/// ```php
/// Container::bind('cache', function($container) { return new CacheManager(); });
/// Container::singleton('db', function($container) { return new Database(); });
/// $cache = Container::make('cache');
/// ```
pub struct Container;

impl Container {
    /// 注册闭包绑定
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名
    /// - `builder`: 构建器闭包
    pub fn bind<F>(abstract_name: &str, builder: F)
    where
        F: Fn(&container::Container) -> Arc<dyn Any + Send + Sync> + Send + Sync + 'static,
    {
        let c = container::global_container().read();
        c.bind(abstract_name, builder);
    }

    /// 注册单例绑定
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名
    /// - `builder`: 构建器闭包
    pub fn singleton<F>(abstract_name: &str, builder: F)
    where
        F: Fn(&container::Container) -> Arc<dyn Any + Send + Sync> + Send + Sync + 'static,
    {
        let c = container::global_container().read();
        c.singleton(abstract_name, builder);
    }

    /// 注册已构建的实例
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名
    /// - `inst`: 实例
    pub fn instance(abstract_name: &str, inst: Arc<dyn Any + Send + Sync>) {
        let c = container::global_container().read();
        c.instance(abstract_name, inst);
    }

    /// 解析实例
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名
    ///
    /// # 返回
    /// 解析出的实例
    pub fn make(abstract_name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        let c = container::global_container().read();
        c.make(abstract_name)
    }

    /// 解析实例（类型化版本）
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名
    ///
    /// # 返回
    /// 类型化的实例
    pub fn make_as<T: Send + Sync + 'static>(abstract_name: &str) -> Option<Arc<T>> {
        let c = container::global_container().read();
        c.make_as::<T>(abstract_name)
    }

    /// 检查绑定是否存在
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名
    ///
    /// # 返回
    /// 是否已注册
    pub fn has(abstract_name: &str) -> bool {
        let c = container::global_container().read();
        c.has(abstract_name)
    }

    /// 设置别名
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名
    /// - `alias`: 别名
    pub fn alias(abstract_name: &str, alias: &str) {
        let c = container::global_container().read();
        c.alias(abstract_name, alias);
    }

    /// 移除绑定
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名
    pub fn forget(abstract_name: &str) {
        let c = container::global_container().read();
        c.forget(abstract_name);
    }

    /// 刷新所有单例实例
    pub fn flush() {
        let c = container::global_container().read();
        c.flush();
    }

    /// 获取所有已注册的绑定名
    pub fn bindings() -> Vec<String> {
        let c = container::global_container().read();
        c.bindings()
    }
}
