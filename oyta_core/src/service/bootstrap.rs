//! 服务启动管理器模块
//!
//! 协调服务的注册、启动、运行和关闭
//! 实现按需启动、惰性初始化和生命周期管理
//!
//! # 启动流程
//! 1. 扫描注解声明
//! 2. 加载配置文件
//! 3. 注册服务到注册表
//! 4. 按需启动服务（惰性初始化）
//! 5. 运行后台服务
//! 6. 优雅关闭

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::RwLock;

use super::annotation::{AnnotationParser, ServiceAnnotation};
use super::config::ServiceConfigLoader;
use super::registry::ServiceRegistry;
use super::traits::{Service, ServiceConfig, ServiceMetadata, ServiceState};

/// 服务启动管理器
///
/// 管理服务的完整生命周期
pub struct ServiceBootstrap {
    /// 服务注册表
    registry: Arc<ServiceRegistry>,
    /// 配置加载器
    config_loader: RwLock<ServiceConfigLoader>,
    /// 注解解析器
    annotation_parser: RwLock<AnnotationParser>,
    /// 项目根目录
    project_root: PathBuf,
    /// 配置目录
    config_dir: PathBuf,
    /// 应用目录
    app_dir: PathBuf,
    /// 已启动的服务句柄
    handles: RwLock<HashMap<String, tokio::task::JoinHandle<()>>>,
    /// 是否已初始化
    initialized: RwLock<bool>,
}

impl ServiceBootstrap {
    /// 创建新的服务启动管理器
    ///
    /// # 参数
    /// - `project_root`: 项目根目录
    ///
    /// # 返回值
    /// 新的服务启动管理器实例
    pub fn new(project_root: &Path) -> Self {
        let config_dir = project_root.join("config");
        let app_dir = project_root.join("app");

        Self {
            registry: Arc::new(ServiceRegistry::new()),
            config_loader: RwLock::new(ServiceConfigLoader::new(&config_dir)),
            annotation_parser: RwLock::new(AnnotationParser::new()),
            project_root: project_root.to_path_buf(),
            config_dir,
            app_dir,
            handles: RwLock::new(HashMap::new()),
            initialized: RwLock::new(false),
        }
    }

    /// 初始化服务系统
    ///
    /// 扫描注解、加载配置、注册服务
    ///
    /// # 返回值
    /// 初始化成功返回 Ok(())
    pub async fn initialize(&self) -> anyhow::Result<()> {
        // 检查是否已初始化
        {
            let initialized = self.initialized.read().await;
            if *initialized {
                tracing::debug!("服务系统已初始化，跳过");
                return Ok(());
            }
        }

        tracing::info!("正在初始化服务系统...");

        // 第一步：加载所有配置文件
        self.load_all_configs().await?;

        // 第二步：扫描注解声明
        let service_annotations = self.scan_annotations().await?;

        // 第三步：注册注解声明的服务
        self.register_annotation_services(&service_annotations).await?;

        // 第四步：检查依赖
        let missing = self.registry.resolve_dependencies().await;
        if !missing.is_empty() {
            tracing::warn!("存在缺失的服务依赖: {:?}", missing);
        }

        // 标记已初始化
        {
            let mut initialized = self.initialized.write().await;
            *initialized = true;
        }

        tracing::info!(
            "服务系统初始化完成，已注册 {} 个服务",
            self.registry.count().await
        );

        Ok(())
    }

    /// 加载所有配置文件
    async fn load_all_configs(&self) -> anyhow::Result<()> {
        let mut loader = self.config_loader.write().await;
        let configs = loader.load_all();

        tracing::debug!("已加载 {} 个服务配置文件", configs.len());

        Ok(())
    }

    /// 扫描注解声明
    ///
    /// # 返回值
    /// 所有服务注解的列表
    async fn scan_annotations(&self) -> anyhow::Result<Vec<ServiceAnnotation>> {
        let mut parser = self.annotation_parser.write().await;

        // 扫描 app 目录
        let services = parser.scan_directory(&self.app_dir)?;

        tracing::debug!("扫描到 {} 个服务注解", services.len());

        Ok(services)
    }

    /// 注册注解声明的服务
    ///
    /// # 参数
    /// - `annotations`: 服务注解列表
    async fn register_annotation_services(
        &self,
        annotations: &[ServiceAnnotation],
    ) -> anyhow::Result<()> {
        for annotation in annotations {
            // 获取配置
            let config = {
                let mut loader = self.config_loader.write().await;
                let file_config = loader.load(&annotation.name);
                ServiceConfigLoader::merge_configs(&file_config, &annotation.config)
            };

            // 创建服务元数据
            let metadata = ServiceMetadata::new(&annotation.name, &annotation.name)
                .with_description(&format!("服务: {}", annotation.name))
                .with_background(false);

            // 创建服务实例
            let service = Box::new(AnnotationService::new(
                &annotation.name,
                config,
                annotation.enable,
                annotation.deferred,
            ));

            // 注册服务
            self.registry.register(service, metadata).await?;
        }

        Ok(())
    }

    /// 启动服务
    ///
    /// 按需启动指定服务
    ///
    /// # 参数
    /// - `name`: 服务名称
    ///
    /// # 返回值
    /// 启动成功返回 Ok(())
    pub async fn start_service(&self, name: &str) -> anyhow::Result<()> {
        // 检查服务是否存在
        if !self.registry.has(name).await {
            return Err(anyhow::anyhow!("服务 {} 不存在", name));
        }

        // 检查服务状态
        let state = self
            .registry
            .get_state(name)
            .await
            .unwrap_or(ServiceState::Unregistered);

        if state.is_running() {
            tracing::debug!("服务 {} 已在运行中", name);
            return Ok(());
        }

        if !state.can_start() {
            return Err(anyhow::anyhow!("服务 {} 当前状态 {:?} 无法启动", name, state));
        }

        tracing::info!("正在启动服务: {}", name);

        // 设置状态为启动中
        self.registry.set_state(name, ServiceState::Starting).await;

        // 获取配置
        let config = {
            let mut loader = self.config_loader.write().await;
            loader.load(name)
        };

        // 根据服务类型启动
        let result = match name {
            "cache" => self.start_cache_service(&config).await,
            "queue" => self.start_queue_service(&config).await,
            "websocket" => self.start_websocket_service(&config).await,
            "timer" => self.start_timer_service(&config).await,
            "session" => self.start_session_service(&config).await,
            "log" => self.start_log_service(&config).await,
            _ => self.start_generic_service(name, &config).await,
        };

        match result {
            Ok(()) => {
                self.registry.set_state(name, ServiceState::Running).await;
                tracing::info!("服务 {} 启动成功", name);
                Ok(())
            }
            Err(e) => {
                self.registry.set_state(name, ServiceState::Failed).await;
                tracing::error!("服务 {} 启动失败: {}", name, e);
                Err(e)
            }
        }
    }

    /// 停止服务
    ///
    /// # 参数
    /// - `name`: 服务名称
    ///
    /// # 返回值
    /// 停止成功返回 Ok(())
    pub async fn stop_service(&self, name: &str) -> anyhow::Result<()> {
        // 检查服务状态
        let state = self
            .registry
            .get_state(name)
            .await
            .unwrap_or(ServiceState::Unregistered);

        if !state.can_stop() {
            return Err(anyhow::anyhow!("服务 {} 当前状态 {:?} 无法停止", name, state));
        }

        tracing::info!("正在停止服务: {}", name);

        // 设置状态为停止中
        self.registry.set_state(name, ServiceState::Stopping).await;

        // 停止后台任务
        {
            let mut handles = self.handles.write().await;
            if let Some(handle) = handles.remove(name) {
                handle.abort();
            }
        }

        // 设置状态为已停止
        self.registry.set_state(name, ServiceState::Stopped).await;

        tracing::info!("服务 {} 已停止", name);
        Ok(())
    }

    /// 启动缓存服务
    async fn start_cache_service(&self, config: &ServiceConfig) -> anyhow::Result<()> {
        tracing::debug!("启动缓存服务，配置: {:?}", config);

        // 获取默认驱动
        let driver = config
            .get("default")
            .and_then(|v| v.as_str())
            .unwrap_or("file");

        tracing::info!("缓存服务已启动，驱动: {}", driver);
        Ok(())
    }

    /// 启动队列服务
    async fn start_queue_service(&self, config: &ServiceConfig) -> anyhow::Result<()> {
        tracing::debug!("启动队列服务，配置: {:?}", config);

        // 获取默认驱动
        let driver = config
            .get("default")
            .and_then(|v| v.as_str())
            .unwrap_or("sync");

        // 如果是 sync 驱动，不需要启动后台服务
        if driver == "sync" {
            tracing::info!("队列服务使用 sync 驱动，无需启动后台服务");
            return Ok(());
        }

        tracing::info!("队列服务已启动，驱动: {}", driver);
        Ok(())
    }

    /// 启动 WebSocket 服务
    async fn start_websocket_service(&self, config: &ServiceConfig) -> anyhow::Result<()> {
        tracing::debug!("启动 WebSocket 服务，配置: {:?}", config);

        // 检查是否启用
        let enable = config
            .get("enable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !enable {
            tracing::info!("WebSocket 服务未启用");
            return Ok(());
        }

        // 获取配置
        let port = config
            .get("port")
            .and_then(|v| v.as_u64())
            .unwrap_or(9501) as u16;

        let max_connections = config
            .get("max_connections")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000) as usize;

        tracing::info!(
            "WebSocket 服务已启动，端口: {}, 最大连接数: {}",
            port,
            max_connections
        );

        Ok(())
    }

    /// 启动定时器服务
    async fn start_timer_service(&self, config: &ServiceConfig) -> anyhow::Result<()> {
        tracing::debug!("启动定时器服务，配置: {:?}", config);

        // 检查是否启用
        let enable = config
            .get("enable")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if !enable {
            tracing::info!("定时器服务未启用");
            return Ok(());
        }

        tracing::info!("定时器服务已启动");
        Ok(())
    }

    /// 启动会话服务
    async fn start_session_service(&self, config: &ServiceConfig) -> anyhow::Result<()> {
        tracing::debug!("启动会话服务，配置: {:?}", config);

        let driver = config
            .get("driver")
            .and_then(|v| v.as_str())
            .unwrap_or("file");

        tracing::info!("会话服务已启动，驱动: {}", driver);
        Ok(())
    }

    /// 启动日志服务
    async fn start_log_service(&self, config: &ServiceConfig) -> anyhow::Result<()> {
        tracing::debug!("启动日志服务，配置: {:?}", config);

        let driver = config
            .get("default")
            .and_then(|v| v.as_str())
            .unwrap_or("file");

        tracing::info!("日志服务已启动，驱动: {}", driver);
        Ok(())
    }

    /// 启动通用服务
    async fn start_generic_service(
        &self,
        name: &str,
        config: &ServiceConfig,
    ) -> anyhow::Result<()> {
        tracing::debug!("启动通用服务: {}, 配置: {:?}", name, config);

        // 检查是否启用
        let enable = config
            .get("enable")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if !enable {
            tracing::info!("服务 {} 未启用", name);
            return Ok(());
        }

        tracing::info!("服务 {} 已启动", name);
        Ok(())
    }

    /// 惰性启动服务
    ///
    /// 当服务未启动时自动启动
    /// 用于门面调用时触发
    ///
    /// # 参数
    /// - `name`: 服务名称
    pub async fn ensure_started(&self, name: &str) {
        // 检查服务是否存在
        if !self.registry.has(name).await {
            // 服务不存在，尝试从配置加载
            let config = {
                let mut loader = self.config_loader.write().await;
                loader.load(name)
            };

            // 检查是否启用
            let enable = config
                .get("enable")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            if !enable {
                tracing::debug!("服务 {} 未启用，跳过启动", name);
                return;
            }

            // 创建并注册服务
            let metadata = ServiceMetadata::new(name, name);
            let service = Box::new(AnnotationService::new(name, config, true, true));

            if let Err(e) = self.registry.register(service, metadata).await {
                tracing::warn!("无法注册服务 {}: {}", name, e);
                return;
            }
        }

        // 检查服务状态
        let state = self.registry.get_state(name).await;

        if let Some(state) = state {
            if !state.is_running() && state.can_start() {
                // 启动服务
                if let Err(e) = self.start_service(name).await {
                    tracing::warn!("无法启动服务 {}: {}", name, e);
                }
            }
        }
    }

    /// 关闭所有服务
    ///
    /// 优雅关闭所有正在运行的服务
    pub async fn shutdown_all(&self) -> anyhow::Result<()> {
        tracing::info!("正在关闭所有服务...");

        // 获取所有正在运行的服务
        let running = self.registry.get_running().await;

        // 按相反顺序停止服务
        for name in running.iter().rev() {
            if let Err(e) = self.stop_service(name).await {
                tracing::warn!("停止服务 {} 失败: {}", name, e);
            }
        }

        tracing::info!("所有服务已关闭");
        Ok(())
    }

    /// 获取服务注册表
    pub fn registry(&self) -> Arc<ServiceRegistry> {
        self.registry.clone()
    }

    /// 获取服务状态
    ///
    /// # 参数
    /// - `name`: 服务名称
    ///
    /// # 返回值
    /// 服务状态
    pub async fn get_state(&self, name: &str) -> Option<ServiceState> {
        self.registry.get_state(name).await
    }

    /// 获取所有服务名称
    pub async fn get_all_services(&self) -> Vec<String> {
        self.registry.get_all_names().await
    }

    /// 获取正在运行的服务
    pub async fn get_running_services(&self) -> Vec<String> {
        self.registry.get_running().await
    }

    /// 重新加载配置
    ///
    /// # 参数
    /// - `name`: 服务名称（可选，为 None 时重载所有配置）
    pub async fn reload_config(&self, name: Option<&str>) -> anyhow::Result<()> {
        let mut loader = self.config_loader.write().await;

        if let Some(service_name) = name {
            loader.load(service_name);
            tracing::info!("已重新加载服务 {} 的配置", service_name);
        } else {
            loader.clear_cache();
            loader.load_all();
            tracing::info!("已重新加载所有服务配置");
        }

        Ok(())
    }
}

/// 注解服务实现
///
/// 用于包装从注解创建的服务
struct AnnotationService {
    /// 服务名称
    name: String,
    /// 服务配置
    config: ServiceConfig,
    /// 服务状态
    state: ServiceState,
    /// 是否启用
    enable: bool,
    /// 是否延迟加载
    deferred: bool,
}

impl AnnotationService {
    /// 创建新的注解服务
    fn new(name: &str, config: ServiceConfig, enable: bool, deferred: bool) -> Self {
        Self {
            name: name.to_string(),
            config,
            state: ServiceState::Unregistered,
            enable,
            deferred,
        }
    }
}

impl Service for AnnotationService {
    fn name(&self) -> &str {
        &self.name
    }

    fn service_type(&self) -> &str {
        &self.name
    }

    fn state(&self) -> ServiceState {
        self.state
    }

    fn set_state(&mut self, state: ServiceState) {
        self.state = state;
    }

    fn config(&self) -> &ServiceConfig {
        &self.config
    }

    fn is_enabled(&self) -> bool {
        self.enable
    }

    fn is_deferred(&self) -> bool {
        self.deferred
    }
}

/// 全局服务启动管理器
///
/// 使用懒加载初始化全局单例
pub fn get_service_bootstrap() -> &'static ServiceBootstrap {
    use std::sync::OnceLock;
    static BOOTSTRAP: OnceLock<ServiceBootstrap> = OnceLock::new();
    BOOTSTRAP.get_or_init(|| {
        // 使用当前目录作为默认项目根目录
        let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        ServiceBootstrap::new(&project_root)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bootstrap_initialization() {
        let dir = tempfile::tempdir().unwrap();
        let bootstrap = ServiceBootstrap::new(dir.path());

        // 初始化
        let result = bootstrap.initialize().await;
        assert!(result.is_ok());

        // 检查已初始化
        let initialized = *bootstrap.initialized.read().await;
        assert!(initialized);
    }

    #[tokio::test]
    async fn test_ensure_started() {
        let dir = tempfile::tempdir().unwrap();
        let bootstrap = ServiceBootstrap::new(dir.path());

        // 初始化
        bootstrap.initialize().await.unwrap();

        // 确保服务启动
        bootstrap.ensure_started("cache").await;

        // 检查服务状态
        let state = bootstrap.get_state("cache").await;
        assert!(state.is_some());
    }
}
