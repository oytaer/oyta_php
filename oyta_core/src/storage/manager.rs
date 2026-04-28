//! 存储管理器
//!
//! 管理多个存储驱动实例
//! 提供统一的存储访问入口

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::traits::{StorageDriver, StorageConfig, FileInfo};
use super::local::LocalStorage;
// use super::s3::{S3Storage, S3Config};
// use super::oss::{OssStorage, OssConfig};
// use super::cos::{CosStorage, CosConfig};
use super::ftp::{FtpStorage, FtpConfig};

/// 存储管理器
///
/// 管理多个存储磁盘（disk）
/// 每个磁盘对应一个存储驱动实例
pub struct StorageManager {
    /// 存储磁盘配置
    disks: RwLock<HashMap<String, DiskConfig>>,
    /// 存储驱动实例缓存
    drivers: RwLock<HashMap<String, Arc<dyn StorageDriver>>>,
    /// 默认磁盘名称
    default_disk: RwLock<String>,
}

/// 磁盘配置
#[derive(Debug, Clone)]
pub struct DiskConfig {
    /// 驱动类型
    pub driver: String,
    /// 根目录
    pub root: String,
    /// URL 前缀
    pub url: String,
    /// 可见性
    pub visibility: String,
    /// S3 配置（如果驱动是 s3）
    pub s3_config: Option<S3ConfigStub>,
    /// OSS 配置（如果驱动是 oss）
    pub oss_config: Option<OssConfigStub>,
    /// COS 配置（如果驱动是 cos）
    pub cos_config: Option<CosConfigStub>,
    /// FTP 配置（如果驱动是 ftp）
    pub ftp_config: Option<FtpConfig>,
}

#[derive(Debug, Clone, Default)]
pub struct S3ConfigStub {
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub bucket: String,
    pub endpoint: Option<String>,
    pub url: String,
    pub path_style: bool,
    pub use_https: bool,
}

#[derive(Debug, Clone, Default)]
pub struct OssConfigStub {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub bucket: String,
    pub endpoint: String,
    pub custom_domain: Option<String>,
    pub use_https: bool,
    pub cname: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CosConfigStub {
    pub secret_id: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
    pub custom_domain: Option<String>,
    pub use_https: bool,
}

impl Default for DiskConfig {
    fn default() -> Self {
        Self {
            driver: "local".to_string(),
            root: "storage/app".to_string(),
            url: "/storage".to_string(),
            visibility: "public".to_string(),
            s3_config: None,
            oss_config: None,
            cos_config: None,
            ftp_config: None,
        }
    }
}

impl StorageManager {
    /// 创建新的存储管理器
    pub fn new() -> Self {
        let mut disks = HashMap::new();
        
        // 默认本地存储磁盘
        disks.insert("local".to_string(), DiskConfig {
            driver: "local".to_string(),
            root: "storage/app".to_string(),
            url: "/storage".to_string(),
            visibility: "public".to_string(),
            ..Default::default()
        });
        
        Self {
            disks: RwLock::new(disks),
            drivers: RwLock::new(HashMap::new()),
            default_disk: RwLock::new("local".to_string()),
        }
    }
    
    /// 从配置创建存储管理器
    pub fn from_config(config: &HashMap<String, serde_json::Value>) -> Self {
        let mut manager = Self::new();
        
        // 解析默认磁盘
        if let Some(default) = config.get("default") {
            if let Some(default_str) = default.as_str() {
                *manager.default_disk.write().unwrap() = default_str.to_string();
            }
        }
        
        // 解析磁盘配置
        if let Some(disks_config) = config.get("disks") {
            if let Some(disks_obj) = disks_config.as_object() {
                let mut disks = manager.disks.write().unwrap();
                
                for (name, disk_config) in disks_obj {
                    if let Some(disk_obj) = disk_config.as_object() {
                        let disk = Self::parse_disk_config(disk_obj);
                        disks.insert(name.clone(), disk);
                    }
                }
            }
        }
        
        manager
    }
    
    /// 解析磁盘配置
    fn parse_disk_config(config: &serde_json::Map<String, serde_json::Value>) -> DiskConfig {
        let driver = config.get("driver")
            .and_then(|v| v.as_str())
            .unwrap_or("local")
            .to_string();
        
        let root = config.get("root")
            .and_then(|v| v.as_str())
            .unwrap_or("storage/app")
            .to_string();
        
        let url = config.get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("/storage")
            .to_string();
        
        let visibility = config.get("visibility")
            .and_then(|v| v.as_str())
            .unwrap_or("public")
            .to_string();
        
        // 解析 S3 配置
        let s3_config = if driver == "s3" {
            Some(S3ConfigStub {
                access_key: config.get("key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                secret_key: config.get("secret")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                region: config.get("region")
                    .and_then(|v| v.as_str())
                    .unwrap_or("us-east-1")
                    .to_string(),
                bucket: config.get("bucket")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                endpoint: config.get("endpoint")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                url: url.clone(),
                path_style: config.get("path_style")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                use_https: config.get("use_https")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
            })
        } else {
            None
        };
        
        // 解析 OSS 配置
        let oss_config = if driver == "oss" {
            Some(OssConfigStub {
                access_key_id: config.get("access_key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                access_key_secret: config.get("secret_key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                bucket: config.get("bucket")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                endpoint: config.get("endpoint")
                    .and_then(|v| v.as_str())
                    .unwrap_or("oss-cn-hangzhou.aliyuncs.com")
                    .to_string(),
                custom_domain: config.get("custom_domain")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                use_https: config.get("use_https")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                cname: config.get("cname")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            })
        } else {
            None
        };
        
        // 解析 COS 配置
        let cos_config = if driver == "cos" {
            Some(CosConfigStub {
                secret_id: config.get("secret_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                secret_key: config.get("secret_key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                bucket: config.get("bucket")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                region: config.get("region")
                    .and_then(|v| v.as_str())
                    .unwrap_or("ap-guangzhou")
                    .to_string(),
                custom_domain: config.get("custom_domain")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                use_https: config.get("use_https")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
            })
        } else {
            None
        };
        
        // 解析 FTP 配置
        let ftp_config = if driver == "ftp" {
            Some(FtpConfig {
                host: config.get("host")
                    .and_then(|v| v.as_str())
                    .unwrap_or("127.0.0.1")
                    .to_string(),
                port: config.get("port")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(21) as u16,
                username: config.get("username")
                    .and_then(|v| v.as_str())
                    .unwrap_or("anonymous")
                    .to_string(),
                password: config.get("password")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                root: root.clone(),
                passive: config.get("passive")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                ssl: config.get("ssl")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                timeout: config.get("timeout")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(30),
            })
        } else {
            None
        };
        
        DiskConfig {
            driver,
            root,
            url,
            visibility,
            s3_config,
            oss_config,
            cos_config,
            ftp_config,
        }
    }
    
    /// 获取磁盘驱动
    ///
    /// # 参数
    /// - `name`: 磁盘名称，为 None 时使用默认磁盘
    ///
    /// # 返回
    /// 存储驱动实例
    pub fn disk(&self, name: Option<&str>) -> Result<Arc<dyn StorageDriver>> {
        // 获取磁盘名称
        let disk_name = match name {
            Some(n) => n.to_string(),
            None => self.default_disk.read().unwrap().clone(),
        };
        
        // 检查缓存
        {
            let drivers = self.drivers.read().unwrap();
            if let Some(driver) = drivers.get(&disk_name) {
                return Ok(driver.clone());
            }
        }
        
        // 创建新的驱动实例
        let disks = self.disks.read().unwrap();
        let disk_config = disks.get(&disk_name)
            .ok_or_else(|| anyhow::anyhow!("磁盘 '{}' 不存在", disk_name))?;
        
        let driver = self.create_driver(disk_config)?;
        
        // 缓存驱动实例
        {
            let mut drivers = self.drivers.write().unwrap();
            drivers.insert(disk_name.clone(), driver.clone());
        }
        
        Ok(driver)
    }
    
    /// 创建存储驱动实例
    fn create_driver(&self, config: &DiskConfig) -> Result<Arc<dyn StorageDriver>> {
        let storage_config = StorageConfig {
            driver: config.driver.clone(),
            root: config.root.clone(),
            url: config.url.clone(),
            visibility: config.visibility.clone(),
            throw: false,
        };
        
        match config.driver.as_str() {
            "local" => Ok(Arc::new(LocalStorage::new(storage_config))),

            "s3" => {
                Err(anyhow::anyhow!("S3 存储驱动暂不可用"))
            }

            "oss" => {
                Err(anyhow::anyhow!("OSS 存储驱动暂不可用"))
            }

            "cos" => {
                Err(anyhow::anyhow!("COS 存储驱动暂不可用"))
            }

            "ftp" => {
                let ftp_config = config.ftp_config.clone().unwrap_or_default();
                let storage = FtpStorage::new(storage_config, ftp_config)?;
                Ok(Arc::new(storage))
            }

            _ => Err(anyhow::anyhow!("不支持的存储驱动: {}", config.driver)),
        }
    }
    
    /// 添加磁盘配置
    pub fn add_disk(&self, name: &str, config: DiskConfig) {
        let mut disks = self.disks.write().unwrap();
        disks.insert(name.to_string(), config);
    }
    
    /// 移除磁盘
    pub fn remove_disk(&self, name: &str) {
        let mut disks = self.disks.write().unwrap();
        disks.remove(name);
        
        let mut drivers = self.drivers.write().unwrap();
        drivers.remove(name);
    }
    
    /// 设置默认磁盘
    pub fn set_default_disk(&self, name: &str) {
        let mut default = self.default_disk.write().unwrap();
        *default = name.to_string();
    }
    
    /// 获取默认磁盘名称
    pub fn get_default_disk(&self) -> String {
        self.default_disk.read().unwrap().clone()
    }
    
    /// 获取所有磁盘名称
    pub fn get_disks(&self) -> Vec<String> {
        self.disks.read().unwrap().keys().cloned().collect()
    }
    
    /// 清除驱动缓存
    pub fn clear_cache(&self) {
        let mut drivers = self.drivers.write().unwrap();
        drivers.clear();
    }
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局存储管理器实例
static STORAGE_MANAGER: std::sync::OnceLock<Arc<StorageManager>> = std::sync::OnceLock::new();

/// 获取全局存储管理器实例
pub fn get_storage_manager() -> Arc<StorageManager> {
    STORAGE_MANAGER.get_or_init(|| Arc::new(StorageManager::new())).clone()
}

/// 初始化全局存储管理器
pub fn init_storage_manager(config: &HashMap<String, serde_json::Value>) {
    let manager = StorageManager::from_config(config);
    let _ = STORAGE_MANAGER.set(Arc::new(manager));
}
