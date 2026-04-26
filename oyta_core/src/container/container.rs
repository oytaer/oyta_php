//! IoC 容器实现
//!
//! 实现完整的依赖注入容器
//! 支持：闭包绑定、单例绑定、实例绑定、别名、自动依赖注入
//! 对应 ThinkPHP 8.0 的 Container 类
//!
//! 核心概念：
//! - bind: 注册一个闭包工厂，每次 make 时调用闭包创建新实例
//! - singleton: 注册一个闭包工厂，首次 make 时创建并缓存，后续返回同一实例
//! - instance: 直接注册一个已创建的实例
//! - alias: 为抽象名设置别名，解析时自动映射

use dashmap::DashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// 构建器闭包类型
///
/// 接收容器引用，返回构建好的实例
/// 闭包内可以通过容器解析其他依赖
type Builder = Box<dyn Fn(&Container) -> Arc<dyn Any + Send + Sync> + Send + Sync>;

/// 绑定定义
///
/// 描述容器中一个抽象类型的注册信息
struct Binding {
    /// 是否单例模式
    /// true: 首次解析后缓存实例，后续返回同一实例
    /// false: 每次解析都调用构建器创建新实例
    is_singleton: bool,
    /// 构建器闭包
    /// 调用时创建新实例
    builder: Option<Builder>,
    /// 已构建的实例（仅单例模式下使用）
    instance: Option<Arc<dyn Any + Send + Sync>>,
}

/// IoC 容器
///
/// 依赖注入容器，管理对象的创建和依赖关系
/// 支持三种注册方式：
/// 1. bind — 每次解析创建新实例
/// 2. singleton — 首次解析创建并缓存
/// 3. instance — 直接注册已有实例
///
/// 支持别名系统，将接口名映射到实现类名
pub struct Container {
    /// 绑定映射（抽象名 → 绑定定义）
    bindings: DashMap<String, Binding>,
    /// 别名映射（别名 → 抽象名）
    aliases: DashMap<String, String>,
    /// 参数覆盖映射（抽象名 → 参数名 → 值）
    /// 用于上下文绑定和构造函数参数注入
    parameter_overrides: DashMap<String, HashMap<String, Arc<dyn Any + Send + Sync>>>,
}

impl Container {
    /// 创建新的空容器
    pub fn new() -> Self {
        Self {
            bindings: DashMap::new(),
            aliases: DashMap::new(),
            parameter_overrides: DashMap::new(),
        }
    }

    /// 注册闭包绑定
    ///
    /// 每次调用 make() 时都会执行闭包创建新实例
    /// 闭包接收容器引用，可以解析其他依赖
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名（通常是接口或类名）
    /// - `builder`: 构建器闭包
    ///
    /// # 示例
    /// ```ignore
    /// container.bind("cache", |c| {
    ///     Arc::new(CacheManager::new(c.make("config").unwrap()))
    /// });
    /// ```
    pub fn bind<F>(&self, abstract_name: &str, builder: F)
    where
        F: Fn(&Container) -> Arc<dyn Any + Send + Sync> + Send + Sync + 'static,
    {
        self.bindings.insert(
            abstract_name.to_string(),
            Binding {
                is_singleton: false,
                builder: Some(Box::new(builder)),
                instance: None,
            },
        );
    }

    /// 注册单例绑定
    ///
    /// 首次调用 make() 时执行闭包创建实例并缓存
    /// 后续调用 make() 直接返回缓存的实例
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名
    /// - `builder`: 构建器闭包
    pub fn singleton<F>(&self, abstract_name: &str, builder: F)
    where
        F: Fn(&Container) -> Arc<dyn Any + Send + Sync> + Send + Sync + 'static,
    {
        self.bindings.insert(
            abstract_name.to_string(),
            Binding {
                is_singleton: true,
                builder: Some(Box::new(builder)),
                instance: None,
            },
        );
    }

    /// 注册已构建的实例
    ///
    /// 直接将一个已创建的对象实例注册到容器中
    /// 实例始终以单例模式存储
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名
    /// - `inst`: 实例的 Arc 引用
    pub fn instance(&self, abstract_name: &str, inst: Arc<dyn Any + Send + Sync>) {
        self.bindings.insert(
            abstract_name.to_string(),
            Binding {
                is_singleton: true,
                builder: None,
                instance: Some(inst),
            },
        );
    }

    /// 注册简单绑定（无构建器）
    ///
    /// 仅注册名称，不提供构建器
    /// 用于标记某个类已在容器中注册，但不提供自动构建能力
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名
    pub fn bind_simple(&self, abstract_name: &str) {
        self.bindings.insert(
            abstract_name.to_string(),
            Binding {
                is_singleton: false,
                builder: None,
                instance: None,
            },
        );
    }

    /// 设置别名
    ///
    /// 将别名映射到抽象名
    /// 解析别名时自动转换为抽象名
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名（目标）
    /// - `alias`: 别名（源）
    pub fn alias(&self, abstract_name: &str, alias: &str) {
        self.aliases.insert(alias.to_string(), abstract_name.to_string());
    }

    /// 解析抽象名为实际名
    ///
    /// 先检查别名映射，再返回原始名称
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名或别名
    ///
    /// # 返回
    /// 解析后的实际抽象名
    fn resolve_name(&self, abstract_name: &str) -> String {
        self.aliases
            .get(abstract_name)
            .map(|a| a.value().clone())
            .unwrap_or_else(|| abstract_name.to_string())
    }

    /// 解析实例
    ///
    /// 从容器中获取指定抽象名的实例
    /// 解析流程：
    /// 1. 通过别名解析实际抽象名
    /// 2. 检查是否有已缓存的单例实例
    /// 3. 检查是否有直接注册的实例
    /// 4. 调用构建器闭包创建新实例
    /// 5. 单例模式下缓存新创建的实例
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名或别名
    ///
    /// # 返回
    /// 解析出的实例，如果未注册返回 None
    pub fn make(&self, abstract_name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        let name = self.resolve_name(abstract_name);

        if let Some(mut binding) = self.bindings.get_mut(&name) {
            if let Some(ref inst) = binding.instance {
                return Some(inst.clone());
            }

            if let Some(ref builder) = binding.builder {
                let instance = builder(self);
                if binding.is_singleton {
                    binding.instance = Some(instance.clone());
                }
                return Some(instance);
            }
        }

        None
    }

    /// 解析实例（类型化版本）
    ///
    /// 将解析结果向下转型为指定类型
    /// 如果类型不匹配返回 None
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名
    ///
    /// # 返回
    /// 类型化的实例引用
    pub fn make_as<T: Send + Sync + 'static>(&self, abstract_name: &str) -> Option<Arc<T>> {
        self.make(abstract_name).and_then(|inst| {
            inst.downcast::<T>().ok().map(|arc| arc)
        })
    }

    /// 检查绑定是否存在
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名或别名
    ///
    /// # 返回
    /// 是否已注册
    pub fn has(&self, abstract_name: &str) -> bool {
        let name = self.resolve_name(abstract_name);
        self.bindings.contains_key(&name)
    }

    /// 移除绑定
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名
    pub fn forget(&self, abstract_name: &str) {
        let name = self.resolve_name(abstract_name);
        self.bindings.remove(&name);
    }

    /// 刷新单例实例
    ///
    /// 清除所有单例的缓存实例，下次 make 时重新构建
    /// 用于测试或需要重新初始化的场景
    pub fn flush(&self) {
        for mut entry in self.bindings.iter_mut() {
            entry.value_mut().instance = None;
        }
    }

    /// 设置参数覆盖
    ///
    /// 为指定抽象名的构造函数参数设置覆盖值
    /// 在自动依赖注入时使用
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名
    /// - `param_name`: 参数名
    /// - `value`: 参数值
    pub fn set_parameter(&self, abstract_name: &str, param_name: &str, value: Arc<dyn Any + Send + Sync>) {
        let name = self.resolve_name(abstract_name);
        let mut overrides = self.parameter_overrides.entry(name).or_insert_with(HashMap::new);
        overrides.insert(param_name.to_string(), value);
    }

    /// 获取参数覆盖
    ///
    /// # 参数
    /// - `abstract_name`: 抽象名
    /// - `param_name`: 参数名
    ///
    /// # 返回
    /// 参数覆盖值
    pub fn get_parameter(&self, abstract_name: &str, param_name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        let name = self.resolve_name(abstract_name);
        self.parameter_overrides
            .get(&name)
            .and_then(|overrides| overrides.get(param_name).cloned())
    }

    /// 获取所有已注册的绑定名
    ///
    /// # 返回
    /// 绑定名列表
    pub fn bindings(&self) -> Vec<String> {
        self.bindings.iter().map(|e| e.key().clone()).collect()
    }

    /// 获取所有别名
    ///
    /// # 返回
    /// 别名到抽象名的映射
    pub fn aliases(&self) -> HashMap<String, String> {
        self.aliases
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect()
    }

    /// 注册核心服务
    ///
    /// 注册 OYTAPHP 框架的核心服务到容器中
    /// 包括：config、db、cache、session、request 等
    pub fn register_core_services(&self) {
        self.bind_simple("config");
        self.bind_simple("db");
        self.bind_simple("cache");
        self.bind_simple("session");
        self.bind_simple("request");
        self.bind_simple("response");
        self.bind_simple("router");
        self.bind_simple("middleware");
        self.bind_simple("event");
        self.bind_simple("log");
        self.bind_simple("template");
        self.bind_simple("validator");
        self.bind_simple("i18n");

        self.alias("oyta\\Config", "config");
        self.alias("oyta\\Db", "db");
        self.alias("oyta\\Cache", "cache");
        self.alias("oyta\\Session", "session");
        self.alias("oyta\\Request", "request");
        self.alias("oyta\\Response", "response");
        self.alias("oyta\\Route", "router");
        self.alias("oyta\\Middleware", "middleware");
        self.alias("oyta\\Event", "event");
        self.alias("oyta\\Log", "log");
        self.alias("oyta\\Template", "template");
        self.alias("oyta\\Validate", "validator");
        self.alias("oyta\\Lang", "i18n");
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局容器实例
///
/// 使用 Lazy + RwLock 实现线程安全的全局单例
/// 所有模块通过此容器获取依赖
static CONTAINER: Lazy<RwLock<Container>> = Lazy::new(|| {
    let container = Container::new();
    container.register_core_services();
    RwLock::new(container)
});

/// 获取全局容器的读锁
pub fn global_container() -> &'static Lazy<RwLock<Container>> {
    &CONTAINER
}

/// 获取全局容器的读锁
pub fn container_read() -> parking_lot::RwLockReadGuard<'static, Container> {
    CONTAINER.read()
}

/// 获取全局容器的写锁
pub fn container_write() -> parking_lot::RwLockWriteGuard<'static, Container> {
    CONTAINER.write()
}
