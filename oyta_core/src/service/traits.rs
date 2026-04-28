//! 服务 trait 定义模块
//!
//! 定义所有服务必须实现的核心 trait
//! 提供统一的服务接口规范，确保所有服务具有一致的行为
//!
//! # 服务生命周期
//! 1. register() - 注册服务，绑定到容器
//! 2. boot() - 启动服务，初始化资源
//! 3. run() - 运行服务（可选，用于后台服务）
//! 4. shutdown() - 关闭服务，释放资源

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 服务状态枚举
///
/// 定义服务可能处于的所有状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    /// 服务未注册
    Unregistered,
    /// 服务已注册但未启动
    Registered,
    /// 服务正在启动中
    Starting,
    /// 服务正在运行
    Running,
    /// 服务正在停止中
    Stopping,
    /// 服务已停止
    Stopped,
    /// 服务启动失败
    Failed,
}

impl ServiceState {
    /// 检查服务是否正在运行
    ///
    /// # 返回值
    /// 如果服务处于 Running 状态返回 true，否则返回 false
    pub fn is_running(&self) -> bool {
        matches!(self, ServiceState::Running)
    }

    /// 检查服务是否可以启动
    ///
    /// # 返回值
    /// 如果服务处于 Registered 或 Stopped 状态返回 true
    pub fn can_start(&self) -> bool {
        matches!(self, ServiceState::Registered | ServiceState::Stopped)
    }

    /// 检查服务是否可以停止
    ///
    /// # 返回值
    /// 如果服务处于 Running 状态返回 true
    pub fn can_stop(&self) -> bool {
        matches!(self, ServiceState::Running)
    }
}

/// 服务配置类型别名
///
/// 使用 HashMap 存储配置键值对，支持嵌套配置
pub type ServiceConfig = HashMap<String, serde_json::Value>;

/// 服务 trait
///
/// 所有服务都必须实现此 trait
/// 定义了服务的完整生命周期方法
pub trait Service: Send + Sync {
    /// 获取服务名称
    ///
    /// 服务名称必须是唯一的，用于标识和查找服务
    ///
    /// # 返回值
    /// 服务名称字符串
    fn name(&self) -> &str;

    /// 获取服务类型
    ///
    /// 用于区分不同类型的服务（如 cache、queue、websocket 等）
    ///
    /// # 返回值
    /// 服务类型字符串
    fn service_type(&self) -> &str;

    /// 注册服务
    ///
    /// 在此方法中进行服务的初始化准备工作
    /// 如：注册依赖、绑定接口等
    /// 此方法不应执行耗时操作
    fn register(&mut self) -> anyhow::Result<()> {
        // 默认空实现，子类可覆盖
        Ok(())
    }

    /// 启动服务
    ///
    /// 在此方法中进行服务的实际启动工作
    /// 如：建立连接、启动监听等
    /// 此方法可以执行耗时操作
    fn boot(&mut self) -> anyhow::Result<()> {
        // 默认空实现，子类可覆盖
        Ok(())
    }

    /// 运行服务
    ///
    /// 对于需要持续运行的后台服务（如 WebSocket、Queue Worker）
    /// 在此方法中实现主循环
    ///
    /// # 返回值
    /// 返回 Ok(()) 表示正常退出，Err 表示错误
    fn run(&mut self) -> anyhow::Result<()> {
        // 默认空实现，后台服务需要覆盖此方法
        Ok(())
    }

    /// 停止服务
    ///
    /// 在此方法中进行服务的清理工作
    /// 如：关闭连接、释放资源等
    fn shutdown(&mut self) -> anyhow::Result<()> {
        // 默认空实现，子类可覆盖
        Ok(())
    }

    /// 获取服务状态
    ///
    /// # 返回值
    /// 当前服务状态
    fn state(&self) -> ServiceState;

    /// 设置服务状态
    ///
    /// # 参数
    /// - `state`: 新的服务状态
    fn set_state(&mut self, state: ServiceState);

    /// 获取服务配置
    ///
    /// # 返回值
    /// 服务配置的引用
    fn config(&self) -> &ServiceConfig;

    /// 更新服务配置
    ///
    /// # 参数
    /// - `config`: 新的配置
    fn update_config(&mut self, config: ServiceConfig) {
        // 默认空实现，子类可覆盖
        let _ = config;
    }

    /// 检查服务是否启用
    ///
    /// # 返回值
    /// 如果配置中 enable 为 true 或未设置，返回 true
    fn is_enabled(&self) -> bool {
        // 从配置中读取 enable 字段，默认为 true
        self.config()
            .get("enable")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    }

    /// 检查服务是否延迟加载
    ///
    /// 延迟加载的服务在首次使用时才启动
    ///
    /// # 返回值
    /// 如果服务配置为延迟加载返回 true
    fn is_deferred(&self) -> bool {
        // 从配置中读取 deferred 字段，默认为 true（惰性加载）
        self.config()
            .get("deferred")
            .and_then(|v| v.as_bool())
            .unwrap_or(true)
    }

    /// 获取服务依赖列表
    ///
    /// # 返回值
    /// 此服务依赖的其他服务名称列表
    fn dependencies(&self) -> Vec<&str> {
        // 默认无依赖
        Vec::new()
    }

    /// 获取服务健康状态
    ///
    /// # 返回值
    /// 如果服务健康返回 true
    fn is_healthy(&self) -> bool {
        // 默认实现：检查服务是否正在运行
        self.state().is_running()
    }
}

/// 后台服务 trait
///
/// 需要在后台持续运行的服务实现此 trait
/// 如 WebSocket 服务、Queue Worker 等
pub trait BackgroundService: Service {
    /// 获取服务运行句柄
    ///
    /// 返回一个可以控制服务运行的句柄
    /// 用于在后台启动和停止服务
    ///
    /// # 返回值
    /// 服务运行句柄
    fn handle(&self) -> ServiceHandle;
}

/// 服务运行句柄
///
/// 用于控制后台服务的运行
#[derive(Debug)]
pub struct ServiceHandle {
    /// 服务名称
    pub name: String,
    /// 停止信号发送器
    pub stop_tx: Option<tokio::sync::oneshot::Sender<()>>,
    /// 运行状态共享
    pub running: Arc<RwLock<bool>>,
}

impl ServiceHandle {
    /// 创建新的服务句柄
    ///
    /// # 参数
    /// - `name`: 服务名称
    ///
    /// # 返回值
    /// 新的服务句柄和停止信号接收器
    pub fn new(name: &str) -> (Self, tokio::sync::oneshot::Receiver<()>) {
        // 创建停止信号通道
        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel();
        
        // 创建句柄
        let handle = Self {
            name: name.to_string(),
            stop_tx: Some(stop_tx),
            running: Arc::new(RwLock::new(false)),
        };
        
        (handle, stop_rx)
    }

    /// 停止服务
    ///
    /// 发送停止信号给服务
    pub async fn stop(&mut self) -> anyhow::Result<()> {
        // 设置运行状态为 false
        *self.running.write().await = false;
        
        // 发送停止信号
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        
        tracing::info!("服务 {} 已发送停止信号", self.name);
        Ok(())
    }

    /// 检查服务是否正在运行
    ///
    /// # 返回值
    /// 如果服务正在运行返回 true
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// 设置运行状态
    ///
    /// # 参数
    /// - `running`: 运行状态
    pub async fn set_running(&self, running: bool) {
        *self.running.write().await = running;
    }
}

/// 服务元数据
///
/// 存储服务的描述信息
#[derive(Debug, Clone)]
pub struct ServiceMetadata {
    /// 服务名称
    pub name: String,
    /// 服务类型
    pub service_type: String,
    /// 服务描述
    pub description: String,
    /// 服务版本
    pub version: String,
    /// 服务作者
    pub author: String,
    /// 服务标签
    pub tags: Vec<String>,
    /// 是否为后台服务
    pub is_background: bool,
    /// 依赖的服务列表
    pub dependencies: Vec<String>,
}

impl ServiceMetadata {
    /// 创建新的服务元数据
    ///
    /// # 参数
    /// - `name`: 服务名称
    /// - `service_type`: 服务类型
    ///
    /// # 返回值
    /// 新的服务元数据实例
    pub fn new(name: &str, service_type: &str) -> Self {
        Self {
            name: name.to_string(),
            service_type: service_type.to_string(),
            description: String::new(),
            version: "1.0.0".to_string(),
            author: String::new(),
            tags: Vec::new(),
            is_background: false,
            dependencies: Vec::new(),
        }
    }

    /// 设置服务描述
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// 设置服务版本
    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    /// 设置服务作者
    pub fn with_author(mut self, author: &str) -> Self {
        self.author = author.to_string();
        self
    }

    /// 添加服务标签
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// 设置是否为后台服务
    pub fn with_background(mut self, is_background: bool) -> Self {
        self.is_background = is_background;
        self
    }

    /// 添加依赖服务
    pub fn with_dependency(mut self, dependency: &str) -> Self {
        self.dependencies.push(dependency.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_state() {
        // 测试服务状态转换
        let state = ServiceState::Registered;
        assert!(state.can_start());
        assert!(!state.is_running());
        assert!(!state.can_stop());

        let running_state = ServiceState::Running;
        assert!(running_state.is_running());
        assert!(running_state.can_stop());
        assert!(!running_state.can_start());
    }

    #[test]
    fn test_service_metadata() {
        // 测试服务元数据构建
        let metadata = ServiceMetadata::new("cache", "cache")
            .with_description("缓存服务")
            .with_version("2.0.0")
            .with_author("OYTAPHP Team")
            .with_tag("core")
            .with_background(false);

        assert_eq!(metadata.name, "cache");
        assert_eq!(metadata.description, "缓存服务");
        assert_eq!(metadata.version, "2.0.0");
        assert_eq!(metadata.author, "OYTAPHP Team");
        assert!(metadata.tags.contains(&"core".to_string()));
    }

    #[tokio::test]
    async fn test_service_handle() {
        // 测试服务句柄
        let (mut handle, mut stop_rx) = ServiceHandle::new("test_service");
        
        assert_eq!(handle.name, "test_service");
        assert!(!handle.is_running().await);
        
        // 设置运行状态
        handle.set_running(true).await;
        assert!(handle.is_running().await);
        
        // 测试停止信号
        handle.stop().await.unwrap();
        
        // 接收停止信号
        stop_rx.try_recv().unwrap();
    }
}
