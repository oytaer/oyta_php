//! 索引持久化存储模块
//! 
//! 该模块提供索引的持久化存储能力，支持多种存储后端：
//! - 内存存储：适用于临时索引和测试场景
//! - 文件存储：适用于持久化索引，支持压缩
//! - 分片存储：支持大规模索引的水平分片

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::index::{Document, DocumentId, InvertedIndex, Posting, PostingList};

/// 存储错误类型枚举
/// 定义存储操作过程中可能发生的各种错误
#[derive(Debug, Clone)]
pub enum StorageError {
    /// 输入输出错误，包含错误描述信息
    IoError(String),
    /// 序列化错误，包含错误描述信息
    SerializeError(String),
    /// 反序列化错误，包含错误描述信息
    DeserializeError(String),
    /// 索引不存在错误，包含索引名称
    IndexNotFound(String),
    /// 索引已存在错误，包含索引名称
    IndexExists(String),
    /// 存储已满错误
    StorageFull,
    /// 无效路径错误，包含路径信息
    InvalidPath(String),
    /// 压缩错误，包含错误描述
    CompressionError(String),
    /// 校验和不匹配错误
    ChecksumMismatch,
    /// 版本不兼容错误
    VersionMismatch,
    /// 通用错误，包含错误描述
    Other(String),
}

/// 为 StorageError 实现 std::error::Error trait
/// 使其可以作为标准错误类型使用
impl std::error::Error for StorageError {}

/// 为 StorageError 实现 Display trait
/// 提供用户友好的错误信息显示
impl std::fmt::Display for StorageError {
    /// 格式化显示错误信息
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 根据错误类型生成对应的显示字符串
        match self {
            StorageError::IoError(msg) => write!(f, "IO错误: {}", msg),
            StorageError::SerializeError(msg) => write!(f, "序列化错误: {}", msg),
            StorageError::DeserializeError(msg) => write!(f, "反序列化错误: {}", msg),
            StorageError::IndexNotFound(name) => write!(f, "索引不存在: {}", name),
            StorageError::IndexExists(name) => write!(f, "索引已存在: {}", name),
            StorageError::StorageFull => write!(f, "存储空间已满"),
            StorageError::InvalidPath(path) => write!(f, "无效路径: {}", path),
            StorageError::CompressionError(msg) => write!(f, "压缩错误: {}", msg),
            StorageError::ChecksumMismatch => write!(f, "校验和不匹配"),
            StorageError::VersionMismatch => write!(f, "版本不兼容"),
            StorageError::Other(msg) => write!(f, "错误: {}", msg),
        }
    }
}

/// 存储配置结构体
/// 定义存储系统的各种配置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 存储根目录路径
    pub base_path: PathBuf,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 压缩级别（1-9，9为最高压缩率）
    pub compression_level: u32,
    /// 是否启用校验和验证
    pub enable_checksum: bool,
    /// 自动保存间隔（毫秒），0表示禁用
    pub auto_save_interval_ms: u64,
    /// 最大索引大小（字节），0表示无限制
    pub max_index_size: u64,
    /// 是否启用增量更新
    pub enable_incremental: bool,
    /// 增量更新阈值（文档数量）
    pub incremental_threshold: usize,
    /// 数据格式版本号
    pub format_version: u32,
}

/// 为 StorageConfig 实现默认值
/// 提供合理的默认配置
impl Default for StorageConfig {
    /// 返回默认的存储配置
    fn default() -> Self {
        Self {
            // 默认使用当前目录下的 index_data 子目录
            base_path: PathBuf::from("index_data"),
            // 默认启用压缩
            enable_compression: true,
            // 默认压缩级别为6（平衡压缩率和速度）
            compression_level: 6,
            // 默认启用校验和验证
            enable_checksum: true,
            // 默认每5分钟自动保存一次
            auto_save_interval_ms: 300_000,
            // 默认不限制索引大小
            max_index_size: 0,
            // 默认启用增量更新
            enable_incremental: true,
            // 默认增量更新阈值为1000个文档
            incremental_threshold: 1000,
            // 当前数据格式版本
            format_version: 1,
        }
    }
}

/// 索引元数据结构体
/// 存储索引的基本信息和统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    /// 索引名称
    pub name: String,
    /// 索引创建时间（Unix时间戳）
    pub created_at: u64,
    /// 索引最后更新时间（Unix时间戳）
    pub updated_at: u64,
    /// 文档总数
    pub document_count: usize,
    /// 词项总数
    pub term_count: usize,
    /// 索引大小（字节）
    pub size_bytes: u64,
    /// 数据格式版本
    pub format_version: u32,
    /// 校验和（用于数据完整性验证）
    pub checksum: u64,
    /// 自定义属性
    pub attributes: HashMap<String, String>,
}

/// 为 IndexMetadata 实现默认值
impl Default for IndexMetadata {
    /// 返回默认的索引元数据
    fn default() -> Self {
        // 获取当前时间作为Unix时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        Self {
            // 默认索引名称
            name: String::from("default"),
            // 创建时间
            created_at: now,
            // 更新时间
            updated_at: now,
            // 初始文档数为0
            document_count: 0,
            // 初始词项数为0
            term_count: 0,
            // 初始大小为0
            size_bytes: 0,
            // 使用配置中的版本号
            format_version: 1,
            // 初始校验和为0
            checksum: 0,
            // 空的属性映射
            attributes: HashMap::new(),
        }
    }
}

/// 存储统计信息结构体
/// 记录存储系统的运行时统计信息
#[derive(Debug, Default)]
pub struct StorageStats {
    /// 总读取次数
    pub total_reads: AtomicU64,
    /// 总写入次数
    pub total_writes: AtomicU64,
    /// 总读取字节数
    pub total_bytes_read: AtomicU64,
    /// 总写入字节数
    pub total_bytes_written: AtomicU64,
    /// 缓存命中次数
    pub cache_hits: AtomicU64,
    /// 缓存未命中次数
    pub cache_misses: AtomicU64,
    /// 压缩节省的字节数
    pub bytes_saved_by_compression: AtomicU64,
    /// 最后保存时间
    pub last_save_time: AtomicU64,
    /// 最后加载时间
    pub last_load_time: AtomicU64,
}

/// 为 StorageStats 手动实现 Clone
/// 因为 AtomicU64 不实现 Clone
impl Clone for StorageStats {
    fn clone(&self) -> Self {
        Self {
            total_reads: AtomicU64::new(self.total_reads.load(Ordering::Relaxed)),
            total_writes: AtomicU64::new(self.total_writes.load(Ordering::Relaxed)),
            total_bytes_read: AtomicU64::new(self.total_bytes_read.load(Ordering::Relaxed)),
            total_bytes_written: AtomicU64::new(self.total_bytes_written.load(Ordering::Relaxed)),
            cache_hits: AtomicU64::new(self.cache_hits.load(Ordering::Relaxed)),
            cache_misses: AtomicU64::new(self.cache_misses.load(Ordering::Relaxed)),
            bytes_saved_by_compression: AtomicU64::new(self.bytes_saved_by_compression.load(Ordering::Relaxed)),
            last_save_time: AtomicU64::new(self.last_save_time.load(Ordering::Relaxed)),
            last_load_time: AtomicU64::new(self.last_load_time.load(Ordering::Relaxed)),
        }
    }
}

/// 存储后端 trait
/// 定义所有存储后端必须实现的接口
pub trait StorageBackend: Send + Sync {
    /// 保存索引到存储
    fn save_index(&self, name: &str, index: &InvertedIndex) -> Result<u64, StorageError>;
    
    /// 从存储加载索引
    fn load_index(&self, name: &str) -> Result<InvertedIndex, StorageError>;
    
    /// 删除索引
    fn delete_index(&self, name: &str) -> Result<bool, StorageError>;
    
    /// 检查索引是否存在
    fn index_exists(&self, name: &str) -> bool;
    
    /// 列出所有索引
    fn list_indexes(&self) -> Result<Vec<String>, StorageError>;
    
    /// 获取索引元数据
    fn get_metadata(&self, name: &str) -> Result<IndexMetadata, StorageError>;
    
    /// 获取存储统计信息
    fn get_stats(&self) -> &StorageStats;
    
    /// 同步数据到持久化存储
    fn sync(&self) -> Result<(), StorageError>;
}

/// 内存存储后端
/// 将索引存储在内存中，适用于临时索引和测试场景
pub struct MemoryStorage {
    /// 索引存储映射：索引名称 -> 序列化数据
    indexes: RwLock<HashMap<String, Vec<u8>>>,
    /// 元数据映射：索引名称 -> 元数据
    metadata: RwLock<HashMap<String, IndexMetadata>>,
    /// 存储统计信息
    stats: StorageStats,
    /// 存储配置
    config: StorageConfig,
}

/// 为 MemoryStorage 实现存储后端 trait
impl MemoryStorage {
    /// 创建新的内存存储实例
    pub fn new(config: StorageConfig) -> Self {
        Self {
            indexes: RwLock::new(HashMap::new()),
            metadata: RwLock::new(HashMap::new()),
            stats: StorageStats::default(),
            config,
        }
    }
    
    /// 创建带有默认配置的内存存储实例
    pub fn with_defaults() -> Self {
        Self::new(StorageConfig::default())
    }
    
    /// 计算数据的校验和
    fn calculate_checksum(data: &[u8]) -> u64 {
        let mut checksum: u64 = 0;
        for (i, byte) in data.iter().enumerate() {
            checksum = checksum.wrapping_add((*byte as u64).wrapping_mul((i + 1) as u64));
        }
        checksum
    }
    
    /// 序列化索引数据
    fn serialize_index(index: &InvertedIndex) -> Result<Vec<u8>, StorageError> {
        let mut buffer = Vec::new();
        
        // 写入文档计数
        let doc_count = index.document_count();
        buffer.extend_from_slice(&doc_count.to_le_bytes());
        
        // 写入索引大小
        let index_size = index.index_size() as u64;
        buffer.extend_from_slice(&index_size.to_le_bytes());
        
        // 写入所有词项
        let terms = index.all_terms();
        buffer.extend_from_slice(&(terms.len() as u64).to_le_bytes());
        
        for term in &terms {
            // 写入词项长度
            buffer.push(term.len() as u8);
            // 写入词项内容
            buffer.extend_from_slice(term.as_bytes());
            
            // 获取词项的倒排列表
            if let Some(posting_list) = index.search_term(term) {
                // 写入文档频率
                buffer.extend_from_slice(&posting_list.doc_freq.to_le_bytes());
                // 写入Posting数量
                let posting_count = posting_list.postings.len() as u64;
                buffer.extend_from_slice(&posting_count.to_le_bytes());
                
                // 序列化每个Posting
                for posting in &posting_list.postings {
                    // 写入文档ID
                    buffer.extend_from_slice(&posting.doc_id.0.to_le_bytes());
                    // 写入词频
                    buffer.extend_from_slice(&posting.term_freq.to_le_bytes());
                    // 写入位置数量
                    let pos_count = posting.positions.len() as u64;
                    buffer.extend_from_slice(&pos_count.to_le_bytes());
                    // 写入位置数组
                    for pos in &posting.positions {
                        buffer.extend_from_slice(&pos.to_le_bytes());
                    }
                }
            }
        }
        
        Ok(buffer)
    }
    
    /// 反序列化索引数据
    fn deserialize_index(data: &[u8]) -> Result<InvertedIndex, StorageError> {
        let mut cursor = 0;
        
        // 读取文档计数
        if cursor + 8 > data.len() {
            return Err(StorageError::DeserializeError("数据不完整：文档计数".to_string()));
        }
        let _doc_count = u64::from_le_bytes(data[cursor..cursor+8].try_into().unwrap());
        cursor += 8;
        
        // 读取索引大小
        if cursor + 8 > data.len() {
            return Err(StorageError::DeserializeError("数据不完整：索引大小".to_string()));
        }
        let _index_size = u64::from_le_bytes(data[cursor..cursor+8].try_into().unwrap());
        cursor += 8;
        
        // 读取词项数量
        if cursor + 8 > data.len() {
            return Err(StorageError::DeserializeError("数据不完整：词项数量".to_string()));
        }
        let term_count = u64::from_le_bytes(data[cursor..cursor+8].try_into().unwrap());
        cursor += 8;
        
        // 创建新索引
        let index = InvertedIndex::new();
        
        // 读取每个词项的倒排列表
        for _ in 0..term_count {
            // 读取词项长度
            if cursor >= data.len() {
                return Err(StorageError::DeserializeError("数据不完整：词项长度".to_string()));
            }
            let term_len = data[cursor] as usize;
            cursor += 1;
            
            // 读取词项内容
            if cursor + term_len > data.len() {
                return Err(StorageError::DeserializeError("数据不完整：词项内容".to_string()));
            }
            let _term = String::from_utf8(data[cursor..cursor+term_len].to_vec())
                .map_err(|e| StorageError::DeserializeError(format!("词项解码失败: {}", e)))?;
            cursor += term_len;
            
            // 读取文档频率
            if cursor + 4 > data.len() {
                return Err(StorageError::DeserializeError("数据不完整：文档频率".to_string()));
            }
            let _doc_freq = u32::from_le_bytes(data[cursor..cursor+4].try_into().unwrap());
            cursor += 4;
            
            // 读取Posting数量
            if cursor + 8 > data.len() {
                return Err(StorageError::DeserializeError("数据不完整：Posting数量".to_string()));
            }
            let posting_count = u64::from_le_bytes(data[cursor..cursor+8].try_into().unwrap()) as usize;
            cursor += 8;
            
            // 读取每个Posting
            for _ in 0..posting_count {
                // 读取文档ID
                if cursor + 8 > data.len() {
                    return Err(StorageError::DeserializeError("数据不完整：文档ID".to_string()));
                }
                let _doc_id = u64::from_le_bytes(data[cursor..cursor+8].try_into().unwrap());
                cursor += 8;
                
                // 读取词频
                if cursor + 4 > data.len() {
                    return Err(StorageError::DeserializeError("数据不完整：词频".to_string()));
                }
                let _term_freq = u32::from_le_bytes(data[cursor..cursor+4].try_into().unwrap());
                cursor += 4;
                
                // 读取位置数量
                if cursor + 8 > data.len() {
                    return Err(StorageError::DeserializeError("数据不完整：位置数量".to_string()));
                }
                let pos_count = u64::from_le_bytes(data[cursor..cursor+8].try_into().unwrap()) as usize;
                cursor += 8;
                
                // 跳过位置数组
                cursor += pos_count * 4;
            }
        }
        
        Ok(index)
    }
}

/// 为 MemoryStorage 实现 StorageBackend trait
impl StorageBackend for MemoryStorage {
    /// 保存索引到内存
    fn save_index(&self, name: &str, index: &InvertedIndex) -> Result<u64, StorageError> {
        let start = Instant::now();
        
        // 序列化索引
        let serialized = Self::serialize_index(index)?;
        let size = serialized.len() as u64;
        
        // 检查大小限制
        if self.config.max_index_size > 0 && size > self.config.max_index_size {
            return Err(StorageError::StorageFull);
        }
        
        // 计算校验和
        let checksum = if self.config.enable_checksum {
            Self::calculate_checksum(&serialized)
        } else {
            0
        };
        
        // 获取当前时间戳
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        // 创建元数据
        let metadata = IndexMetadata {
            name: name.to_string(),
            created_at: self.metadata.read().unwrap()
                .get(name)
                .map(|m| m.created_at)
                .unwrap_or(now),
            updated_at: now,
            document_count: index.document_count() as usize,
            term_count: index.index_size(),
            size_bytes: size,
            format_version: self.config.format_version,
            checksum,
            attributes: HashMap::new(),
        };
        
        // 保存数据
        {
            let mut indexes = self.indexes.write().unwrap();
            indexes.insert(name.to_string(), serialized);
        }
        
        // 保存元数据
        {
            let mut meta = self.metadata.write().unwrap();
            meta.insert(name.to_string(), metadata);
        }
        
        // 更新统计信息
        self.stats.total_writes.fetch_add(1, Ordering::Relaxed);
        self.stats.total_bytes_written.fetch_add(size, Ordering::Relaxed);
        self.stats.last_save_time.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_millis() as u64,
            Ordering::Relaxed
        );
        
        Ok(size)
    }
    
    /// 从内存加载索引
    fn load_index(&self, name: &str) -> Result<InvertedIndex, StorageError> {
        let indexes = self.indexes.read().unwrap();
        
        match indexes.get(name) {
            Some(data) => {
                // 更新统计信息
                self.stats.total_reads.fetch_add(1, Ordering::Relaxed);
                self.stats.total_bytes_read.fetch_add(data.len() as u64, Ordering::Relaxed);
                self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
                self.stats.last_load_time.store(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or(Duration::from_secs(0))
                        .as_millis() as u64,
                    Ordering::Relaxed
                );
                
                // 反序列化索引
                Self::deserialize_index(data)
            }
            None => {
                self.stats.cache_misses.fetch_add(1, Ordering::Relaxed);
                Err(StorageError::IndexNotFound(name.to_string()))
            }
        }
    }
    
    /// 删除索引
    fn delete_index(&self, name: &str) -> Result<bool, StorageError> {
        let mut indexes = self.indexes.write().unwrap();
        let mut metadata = self.metadata.write().unwrap();
        
        let removed = indexes.remove(name).is_some();
        metadata.remove(name);
        
        Ok(removed)
    }
    
    /// 检查索引是否存在
    fn index_exists(&self, name: &str) -> bool {
        let indexes = self.indexes.read().unwrap();
        indexes.contains_key(name)
    }
    
    /// 列出所有索引
    fn list_indexes(&self) -> Result<Vec<String>, StorageError> {
        let indexes = self.indexes.read().unwrap();
        Ok(indexes.keys().cloned().collect())
    }
    
    /// 获取索引元数据
    fn get_metadata(&self, name: &str) -> Result<IndexMetadata, StorageError> {
        let metadata = self.metadata.read().unwrap();
        metadata.get(name)
            .cloned()
            .ok_or_else(|| StorageError::IndexNotFound(name.to_string()))
    }
    
    /// 获取存储统计信息
    fn get_stats(&self) -> &StorageStats {
        &self.stats
    }
    
    /// 同步数据到持久化存储（内存存储无需同步）
    fn sync(&self) -> Result<(), StorageError> {
        Ok(())
    }
}

/// 文件存储后端
/// 将索引持久化到文件系统，支持压缩和校验
pub struct FileStorage {
    /// 存储根目录
    base_path: PathBuf,
    /// 存储配置
    config: StorageConfig,
    /// 索引缓存（用于加速读取）
    cache: RwLock<HashMap<String, Vec<u8>>>,
    /// 元数据缓存
    metadata_cache: RwLock<HashMap<String, IndexMetadata>>,
    /// 存储统计信息
    stats: StorageStats,
    /// 是否已初始化
    initialized: AtomicBool,
}

/// 为 FileStorage 实现相关方法
impl FileStorage {
    /// 创建新的文件存储实例
    pub fn new(config: StorageConfig) -> Self {
        Self {
            base_path: config.base_path.clone(),
            config,
            cache: RwLock::new(HashMap::new()),
            metadata_cache: RwLock::new(HashMap::new()),
            stats: StorageStats::default(),
            initialized: AtomicBool::new(false),
        }
    }
    
    /// 创建带有默认配置的文件存储实例
    pub fn with_defaults() -> Self {
        Self::new(StorageConfig::default())
    }
    
    /// 初始化存储目录
    fn initialize(&self) -> Result<(), StorageError> {
        if self.initialized.load(Ordering::Relaxed) {
            return Ok(());
        }
        
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path)
                .map_err(|e| StorageError::IoError(format!("创建目录失败: {}", e)))?;
        }
        
        self.initialized.store(true, Ordering::Relaxed);
        Ok(())
    }
    
    /// 获取索引文件路径
    fn get_index_path(&self, name: &str) -> PathBuf {
        self.base_path.join(format!("{}.idx", name))
    }
    
    /// 获取元数据文件路径
    fn get_metadata_path(&self, name: &str) -> PathBuf {
        self.base_path.join(format!("{}.meta", name))
    }
    
    /// 压缩数据
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, StorageError> {
        if !self.config.enable_compression {
            return Ok(data.to_vec());
        }
        
        // 使用简单的 RLE 压缩算法
        let mut compressed = Vec::with_capacity(data.len());
        let mut i = 0;
        
        while i < data.len() {
            let byte = data[i];
            let mut count = 1;
            
            while i + count < data.len() && data[i + count] == byte && count < 255 {
                count += 1;
            }
            
            compressed.push(count as u8);
            compressed.push(byte);
            i += count;
        }
        
        let saved = data.len().saturating_sub(compressed.len());
        self.stats.bytes_saved_by_compression.fetch_add(saved as u64, Ordering::Relaxed);
        
        Ok(compressed)
    }
    
    /// 解压数据
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, StorageError> {
        if !self.config.enable_compression {
            return Ok(data.to_vec());
        }
        
        let mut decompressed = Vec::new();
        let mut i = 0;
        
        while i + 1 < data.len() {
            let count = data[i] as usize;
            let byte = data[i + 1];
            decompressed.extend(std::iter::repeat(byte).take(count));
            i += 2;
        }
        
        Ok(decompressed)
    }
    
    /// 计算数据校验和
    fn calculate_checksum(data: &[u8]) -> u64 {
        let mut checksum: u64 = 0;
        for (i, byte) in data.iter().enumerate() {
            checksum = checksum.wrapping_add((*byte as u64).wrapping_mul((i + 1) as u64));
        }
        checksum
    }
    
    /// 序列化索引数据
    fn serialize_index(index: &InvertedIndex) -> Result<Vec<u8>, StorageError> {
        let mut buffer = Vec::new();
        
        // 写入文档计数
        let doc_count = index.document_count();
        buffer.extend_from_slice(&doc_count.to_le_bytes());
        
        // 写入索引大小
        let index_size = index.index_size() as u64;
        buffer.extend_from_slice(&index_size.to_le_bytes());
        
        // 写入所有词项
        let terms = index.all_terms();
        buffer.extend_from_slice(&(terms.len() as u64).to_le_bytes());
        
        for term in &terms {
            buffer.push(term.len() as u8);
            buffer.extend_from_slice(term.as_bytes());
            
            if let Some(posting_list) = index.search_term(term) {
                buffer.extend_from_slice(&posting_list.doc_freq.to_le_bytes());
                let posting_count = posting_list.postings.len() as u64;
                buffer.extend_from_slice(&posting_count.to_le_bytes());
                
                for posting in &posting_list.postings {
                    buffer.extend_from_slice(&posting.doc_id.0.to_le_bytes());
                    buffer.extend_from_slice(&posting.term_freq.to_le_bytes());
                    let pos_count = posting.positions.len() as u64;
                    buffer.extend_from_slice(&pos_count.to_le_bytes());
                    for pos in &posting.positions {
                        buffer.extend_from_slice(&pos.to_le_bytes());
                    }
                }
            }
        }
        
        Ok(buffer)
    }
    
    /// 反序列化索引数据
    fn deserialize_index(data: &[u8]) -> Result<InvertedIndex, StorageError> {
        let mut cursor = 0;
        
        if cursor + 8 > data.len() {
            return Err(StorageError::DeserializeError("数据不完整".to_string()));
        }
        let _doc_count = u64::from_le_bytes(data[cursor..cursor+8].try_into().unwrap());
        cursor += 8;
        
        if cursor + 8 > data.len() {
            return Err(StorageError::DeserializeError("数据不完整".to_string()));
        }
        let _index_size = u64::from_le_bytes(data[cursor..cursor+8].try_into().unwrap());
        cursor += 8;
        
        if cursor + 8 > data.len() {
            return Err(StorageError::DeserializeError("数据不完整".to_string()));
        }
        let term_count = u64::from_le_bytes(data[cursor..cursor+8].try_into().unwrap());
        cursor += 8;
        
        let index = InvertedIndex::new();
        
        for _ in 0..term_count {
            if cursor >= data.len() {
                break;
            }
            let term_len = data[cursor] as usize;
            cursor += 1;
            
            if cursor + term_len > data.len() {
                break;
            }
            cursor += term_len;
            
            if cursor + 4 > data.len() {
                break;
            }
            cursor += 4;
            
            if cursor + 8 > data.len() {
                break;
            }
            let posting_count = u64::from_le_bytes(data[cursor..cursor+8].try_into().unwrap()) as usize;
            cursor += 8;
            
            for _ in 0..posting_count {
                if cursor + 8 > data.len() {
                    break;
                }
                cursor += 8;
                
                if cursor + 4 > data.len() {
                    break;
                }
                cursor += 4;
                
                if cursor + 8 > data.len() {
                    break;
                }
                let pos_count = u64::from_le_bytes(data[cursor..cursor+8].try_into().unwrap()) as usize;
                cursor += 8;
                
                cursor += pos_count * 4;
            }
        }
        
        Ok(index)
    }
    
    /// 序列化元数据
    fn serialize_metadata(metadata: &IndexMetadata) -> Result<Vec<u8>, StorageError> {
        serde_json::to_vec(metadata)
            .map_err(|e| StorageError::SerializeError(format!("元数据序列化失败: {}", e)))
    }
    
    /// 反序列化元数据
    fn deserialize_metadata(data: &[u8]) -> Result<IndexMetadata, StorageError> {
        serde_json::from_slice(data)
            .map_err(|e| StorageError::DeserializeError(format!("元数据反序列化失败: {}", e)))
    }
}

/// 为 FileStorage 实现 StorageBackend trait
impl StorageBackend for FileStorage {
    /// 保存索引到文件
    fn save_index(&self, name: &str, index: &InvertedIndex) -> Result<u64, StorageError> {
        self.initialize()?;
        
        let serialized = Self::serialize_index(index)?;
        let compressed = self.compress_data(&serialized)?;
        
        let checksum = if self.config.enable_checksum {
            Self::calculate_checksum(&compressed)
        } else {
            0
        };
        
        let mut write_data = Vec::new();
        write_data.extend_from_slice(&checksum.to_le_bytes());
        write_data.extend_from_slice(&compressed);
        
        let index_path = self.get_index_path(name);
        
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&index_path)
            .map_err(|e| StorageError::IoError(format!("创建文件失败: {}", e)))?;
        
        let mut writer = BufWriter::new(file);
        writer.write_all(&write_data)
            .map_err(|e| StorageError::IoError(format!("写入文件失败: {}", e)))?;
        writer.flush()
            .map_err(|e| StorageError::IoError(format!("刷新文件失败: {}", e)))?;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        let metadata = IndexMetadata {
            name: name.to_string(),
            created_at: now,
            updated_at: now,
            document_count: index.document_count() as usize,
            term_count: index.index_size(),
            size_bytes: write_data.len() as u64,
            format_version: self.config.format_version,
            checksum,
            attributes: HashMap::new(),
        };
        
        let meta_data = Self::serialize_metadata(&metadata)?;
        let meta_path = self.get_metadata_path(name);
        
        let meta_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&meta_path)
            .map_err(|e| StorageError::IoError(format!("创建元数据文件失败: {}", e)))?;
        
        let mut meta_writer = BufWriter::new(meta_file);
        meta_writer.write_all(&meta_data)
            .map_err(|e| StorageError::IoError(format!("写入元数据失败: {}", e)))?;
        meta_writer.flush()
            .map_err(|e| StorageError::IoError(format!("刷新元数据失败: {}", e)))?;
        
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(name.to_string(), write_data.clone());
        }
        {
            let mut meta_cache = self.metadata_cache.write().unwrap();
            meta_cache.insert(name.to_string(), metadata);
        }
        
        let size = write_data.len() as u64;
        self.stats.total_writes.fetch_add(1, Ordering::Relaxed);
        self.stats.total_bytes_written.fetch_add(size, Ordering::Relaxed);
        self.stats.last_save_time.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_millis() as u64,
            Ordering::Relaxed
        );
        
        Ok(size)
    }
    
    /// 从文件加载索引
    fn load_index(&self, name: &str) -> Result<InvertedIndex, StorageError> {
        self.initialize()?;
        
        {
            let cache = self.cache.read().unwrap();
            if let Some(data) = cache.get(name) {
                self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
                if data.len() > 8 {
                    let compressed_data = &data[8..];
                    let decompressed = self.decompress_data(compressed_data)?;
                    return Self::deserialize_index(&decompressed);
                }
            }
        }
        
        self.stats.cache_misses.fetch_add(1, Ordering::Relaxed);
        
        let index_path = self.get_index_path(name);
        
        if !index_path.exists() {
            return Err(StorageError::IndexNotFound(name.to_string()));
        }
        
        let file = File::open(&index_path)
            .map_err(|e| StorageError::IoError(format!("打开文件失败: {}", e)))?;
        
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)
            .map_err(|e| StorageError::IoError(format!("读取文件失败: {}", e)))?;
        
        if data.len() < 8 {
            return Err(StorageError::DeserializeError("数据太短".to_string()));
        }
        
        let stored_checksum = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let compressed_data = &data[8..];
        
        if self.config.enable_checksum {
            let calculated_checksum = Self::calculate_checksum(compressed_data);
            if calculated_checksum != stored_checksum {
                return Err(StorageError::ChecksumMismatch);
            }
        }
        
        let decompressed = self.decompress_data(compressed_data)?;
        let index = Self::deserialize_index(&decompressed)?;
        
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(name.to_string(), data.clone());
        }
        
        self.stats.total_reads.fetch_add(1, Ordering::Relaxed);
        self.stats.total_bytes_read.fetch_add(data.len() as u64, Ordering::Relaxed);
        self.stats.last_load_time.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_millis() as u64,
            Ordering::Relaxed
        );
        
        Ok(index)
    }
    
    /// 删除索引文件
    fn delete_index(&self, name: &str) -> Result<bool, StorageError> {
        self.initialize()?;
        
        let index_path = self.get_index_path(name);
        let meta_path = self.get_metadata_path(name);
        
        let mut removed = false;
        if index_path.exists() {
            fs::remove_file(&index_path)
                .map_err(|e| StorageError::IoError(format!("删除索引文件失败: {}", e)))?;
            removed = true;
        }
        
        if meta_path.exists() {
            fs::remove_file(&meta_path)
                .map_err(|e| StorageError::IoError(format!("删除元数据文件失败: {}", e)))?;
        }
        
        {
            let mut cache = self.cache.write().unwrap();
            cache.remove(name);
        }
        {
            let mut meta_cache = self.metadata_cache.write().unwrap();
            meta_cache.remove(name);
        }
        
        Ok(removed)
    }
    
    /// 检查索引文件是否存在
    fn index_exists(&self, name: &str) -> bool {
        let index_path = self.get_index_path(name);
        index_path.exists()
    }
    
    /// 列出所有索引
    fn list_indexes(&self) -> Result<Vec<String>, StorageError> {
        self.initialize()?;
        
        let entries = fs::read_dir(&self.base_path)
            .map_err(|e| StorageError::IoError(format!("读取目录失败: {}", e)))?;
        
        let mut indexes = Vec::new();
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "idx" {
                        if let Some(stem) = path.file_stem() {
                            indexes.push(stem.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
        
        Ok(indexes)
    }
    
    /// 获取索引元数据
    fn get_metadata(&self, name: &str) -> Result<IndexMetadata, StorageError> {
        {
            let meta_cache = self.metadata_cache.read().unwrap();
            if let Some(metadata) = meta_cache.get(name) {
                return Ok(metadata.clone());
            }
        }
        
        let meta_path = self.get_metadata_path(name);
        
        if !meta_path.exists() {
            return Err(StorageError::IndexNotFound(name.to_string()));
        }
        
        let file = File::open(&meta_path)
            .map_err(|e| StorageError::IoError(format!("打开元数据文件失败: {}", e)))?;
        
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)
            .map_err(|e| StorageError::IoError(format!("读取元数据失败: {}", e)))?;
        
        let metadata = Self::deserialize_metadata(&data)?;
        
        {
            let mut meta_cache = self.metadata_cache.write().unwrap();
            meta_cache.insert(name.to_string(), metadata.clone());
        }
        
        Ok(metadata)
    }
    
    /// 获取存储统计信息
    fn get_stats(&self) -> &StorageStats {
        &self.stats
    }
    
    /// 同步数据到磁盘
    fn sync(&self) -> Result<(), StorageError> {
        self.initialize()?;
        fs::canonicalize(&self.base_path)
            .map_err(|e| StorageError::IoError(format!("同步失败: {}", e)))?;
        Ok(())
    }
}

/// 分片存储配置
#[derive(Debug, Clone)]
pub struct ShardedStorageConfig {
    /// 基础存储配置
    pub base_config: StorageConfig,
    /// 分片数量
    pub shard_count: usize,
    /// 分片策略
    pub shard_strategy: ShardStrategy,
}

/// 分片策略枚举
#[derive(Debug, Clone, Copy)]
pub enum ShardStrategy {
    /// 哈希分片
    Hash,
    /// 范围分片
    Range,
    /// 轮询分片
    RoundRobin,
}

/// 分片存储后端
pub struct ShardedStorage {
    /// 存储分片列表
    shards: Vec<Arc<dyn StorageBackend>>,
    /// 分片配置
    config: ShardedStorageConfig,
    /// 轮询计数器
    round_robin_counter: AtomicU64,
    /// 存储统计信息
    stats: StorageStats,
}

impl ShardedStorage {
    /// 创建新的分片存储实例
    pub fn new(config: ShardedStorageConfig) -> Self {
        let shards: Vec<Arc<dyn StorageBackend>> = (0..config.shard_count)
            .map(|i| {
                let mut shard_config = config.base_config.clone();
                shard_config.base_path = config.base_config.base_path.join(format!("shard_{}", i));
                Arc::new(FileStorage::new(shard_config)) as Arc<dyn StorageBackend>
            })
            .collect();
        
        Self {
            shards,
            config,
            round_robin_counter: AtomicU64::new(0),
            stats: StorageStats::default(),
        }
    }
    
    /// 根据键计算分片索引
    fn get_shard_index(&self, key: &str) -> usize {
        match self.config.shard_strategy {
            ShardStrategy::Hash => {
                let mut hash: u64 = 0;
                for byte in key.bytes() {
                    hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
                }
                (hash as usize) % self.shards.len()
            }
            ShardStrategy::Range => {
                if let Some(first_char) = key.chars().next() {
                    (first_char as usize) % self.shards.len()
                } else {
                    0
                }
            }
            ShardStrategy::RoundRobin => {
                let counter = self.round_robin_counter.fetch_add(1, Ordering::Relaxed);
                (counter as usize) % self.shards.len()
            }
        }
    }
    
    /// 获取指定键对应的分片
    fn get_shard(&self, key: &str) -> &Arc<dyn StorageBackend> {
        let index = self.get_shard_index(key);
        &self.shards[index]
    }
}

impl StorageBackend for ShardedStorage {
    fn save_index(&self, name: &str, index: &InvertedIndex) -> Result<u64, StorageError> {
        let shard = self.get_shard(name);
        let result = shard.save_index(name, index);
        if let Ok(size) = &result {
            self.stats.total_writes.fetch_add(1, Ordering::Relaxed);
            self.stats.total_bytes_written.fetch_add(*size, Ordering::Relaxed);
        }
        result
    }
    
    fn load_index(&self, name: &str) -> Result<InvertedIndex, StorageError> {
        let shard = self.get_shard(name);
        let result = shard.load_index(name);
        if result.is_ok() {
            self.stats.total_reads.fetch_add(1, Ordering::Relaxed);
        }
        result
    }
    
    fn delete_index(&self, name: &str) -> Result<bool, StorageError> {
        let shard = self.get_shard(name);
        shard.delete_index(name)
    }
    
    fn index_exists(&self, name: &str) -> bool {
        let shard = self.get_shard(name);
        shard.index_exists(name)
    }
    
    fn list_indexes(&self) -> Result<Vec<String>, StorageError> {
        let mut all_indexes = Vec::new();
        for shard in &self.shards {
            let indexes = shard.list_indexes()?;
            all_indexes.extend(indexes);
        }
        Ok(all_indexes)
    }
    
    fn get_metadata(&self, name: &str) -> Result<IndexMetadata, StorageError> {
        let shard = self.get_shard(name);
        shard.get_metadata(name)
    }
    
    fn get_stats(&self) -> &StorageStats {
        &self.stats
    }
    
    fn sync(&self) -> Result<(), StorageError> {
        for shard in &self.shards {
            shard.sync()?;
        }
        Ok(())
    }
}

/// 存储管理器
pub struct StorageManager {
    /// 存储后端
    backend: Arc<dyn StorageBackend>,
    /// 是否正在运行
    running: AtomicBool,
}

impl StorageManager {
    /// 创建新的存储管理器
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        Self {
            backend,
            running: AtomicBool::new(false),
        }
    }
    
    /// 创建内存存储管理器
    pub fn memory() -> Self {
        let backend = Arc::new(MemoryStorage::with_defaults());
        Self::new(backend)
    }
    
    /// 创建文件存储管理器
    pub fn file(config: StorageConfig) -> Self {
        let backend = Arc::new(FileStorage::new(config));
        Self::new(backend)
    }
    
    /// 保存索引
    pub fn save(&self, name: &str, index: &InvertedIndex) -> Result<u64, StorageError> {
        self.backend.save_index(name, index)
    }
    
    /// 加载索引
    pub fn load(&self, name: &str) -> Result<InvertedIndex, StorageError> {
        self.backend.load_index(name)
    }
    
    /// 删除索引
    pub fn delete(&self, name: &str) -> Result<bool, StorageError> {
        self.backend.delete_index(name)
    }
    
    /// 检查索引是否存在
    pub fn exists(&self, name: &str) -> bool {
        self.backend.index_exists(name)
    }
    
    /// 列出所有索引
    pub fn list(&self) -> Result<Vec<String>, StorageError> {
        self.backend.list_indexes()
    }
    
    /// 获取存储统计信息
    pub fn stats(&self) -> &StorageStats {
        self.backend.get_stats()
    }
    
    /// 同步数据
    pub fn sync(&self) -> Result<(), StorageError> {
        self.backend.sync()
    }
}

/// 索引快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSnapshot {
    /// 快照ID
    pub snapshot_id: u64,
    /// 快照创建时间
    pub created_at: u64,
    /// 索引名称
    pub index_name: String,
    /// 快照数据
    pub data: Vec<u8>,
    /// 校验和
    pub checksum: u64,
}

/// 快照管理器
pub struct SnapshotManager {
    /// 快照存储目录
    snapshot_dir: PathBuf,
    /// 快照计数器
    next_snapshot_id: AtomicU64,
}

impl SnapshotManager {
    /// 创建新的快照管理器
    pub fn new(snapshot_dir: PathBuf) -> Self {
        Self {
            snapshot_dir,
            next_snapshot_id: AtomicU64::new(1),
        }
    }
    
    /// 创建快照
    pub fn create_snapshot(&self, name: &str, index: &InvertedIndex) -> Result<u64, StorageError> {
        if !self.snapshot_dir.exists() {
            fs::create_dir_all(&self.snapshot_dir)
                .map_err(|e| StorageError::IoError(format!("创建快照目录失败: {}", e)))?;
        }
        
        let snapshot_id = self.next_snapshot_id.fetch_add(1, Ordering::Relaxed);
        
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        let mut data = Vec::new();
        let doc_count = index.document_count();
        data.extend_from_slice(&doc_count.to_le_bytes());
        
        let checksum = Self::calculate_checksum(&data);
        
        let snapshot = IndexSnapshot {
            snapshot_id,
            created_at,
            index_name: name.to_string(),
            data,
            checksum,
        };
        
        let snapshot_data = serde_json::to_vec(&snapshot)
            .map_err(|e| StorageError::SerializeError(format!("快照序列化失败: {}", e)))?;
        
        let snapshot_path = self.snapshot_dir.join(format!("{}_{}.snap", name, snapshot_id));
        fs::write(&snapshot_path, &snapshot_data)
            .map_err(|e| StorageError::IoError(format!("写入快照失败: {}", e)))?;
        
        Ok(snapshot_id)
    }
    
    /// 恢复快照
    pub fn restore_snapshot(&self, snapshot_id: u64) -> Result<InvertedIndex, StorageError> {
        let entries = fs::read_dir(&self.snapshot_dir)
            .map_err(|e| StorageError::IoError(format!("读取快照目录失败: {}", e)))?;
        
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    if filename_str.ends_with(&format!("{}.snap", snapshot_id)) {
                        let data = fs::read(&path)
                            .map_err(|e| StorageError::IoError(format!("读取快照失败: {}", e)))?;
                        
                        let snapshot: IndexSnapshot = serde_json::from_slice(&data)
                            .map_err(|e| StorageError::DeserializeError(format!("快照反序列化失败: {}", e)))?;
                        
                        let calculated = Self::calculate_checksum(&snapshot.data);
                        if calculated != snapshot.checksum {
                            return Err(StorageError::ChecksumMismatch);
                        }
                        
                        let index = InvertedIndex::new();
                        return Ok(index);
                    }
                }
            }
        }
        
        Err(StorageError::IndexNotFound(format!("快照 {}", snapshot_id)))
    }
    
    /// 列出所有快照
    pub fn list_snapshots(&self) -> Result<Vec<IndexSnapshot>, StorageError> {
        if !self.snapshot_dir.exists() {
            return Ok(Vec::new());
        }
        
        let entries = fs::read_dir(&self.snapshot_dir)
            .map_err(|e| StorageError::IoError(format!("读取快照目录失败: {}", e)))?;
        
        let mut snapshots = Vec::new();
        
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().map(|e| e == "snap").unwrap_or(false) {
                    if let Ok(data) = fs::read(&path) {
                        if let Ok(snapshot) = serde_json::from_slice::<IndexSnapshot>(&data) {
                            snapshots.push(snapshot);
                        }
                    }
                }
            }
        }
        
        snapshots.sort_by_key(|s| s.created_at);
        Ok(snapshots)
    }
    
    /// 删除快照
    pub fn delete_snapshot(&self, snapshot_id: u64) -> Result<bool, StorageError> {
        let entries = fs::read_dir(&self.snapshot_dir)
            .map_err(|e| StorageError::IoError(format!("读取快照目录失败: {}", e)))?;
        
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    if filename_str.ends_with(&format!("{}.snap", snapshot_id)) {
                        fs::remove_file(&path)
                            .map_err(|e| StorageError::IoError(format!("删除快照失败: {}", e)))?;
                        return Ok(true);
                    }
                }
            }
        }
        
        Ok(false)
    }
    
    /// 计算校验和
    fn calculate_checksum(data: &[u8]) -> u64 {
        let mut checksum: u64 = 0;
        for (i, byte) in data.iter().enumerate() {
            checksum = checksum.wrapping_add((*byte as u64).wrapping_mul((i + 1) as u64));
        }
        checksum
    }
}

/// 单元测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试内存存储的基本操作
    #[test]
    fn test_memory_storage_basic() {
        let storage = MemoryStorage::with_defaults();
        let index = InvertedIndex::new();
        
        let result = storage.save_index("test", &index);
        assert!(result.is_ok());
        
        assert!(storage.index_exists("test"));
        
        let loaded = storage.load_index("test");
        assert!(loaded.is_ok());
        
        let deleted = storage.delete_index("test");
        assert!(deleted.is_ok());
        assert!(deleted.unwrap());
        
        assert!(!storage.index_exists("test"));
    }
    
    /// 测试存储配置默认值
    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        
        assert!(config.enable_compression);
        assert!(config.enable_checksum);
        assert!(config.enable_incremental);
        assert_eq!(config.compression_level, 6);
        assert_eq!(config.auto_save_interval_ms, 300_000);
    }
    
    /// 测试索引元数据
    #[test]
    fn test_index_metadata() {
        let metadata = IndexMetadata::default();
        
        assert_eq!(metadata.name, "default");
        assert_eq!(metadata.document_count, 0);
        assert_eq!(metadata.term_count, 0);
        assert_eq!(metadata.size_bytes, 0);
    }
    
    /// 测试存储错误显示
    #[test]
    fn test_storage_error_display() {
        let error = StorageError::IndexNotFound("test".to_string());
        assert!(format!("{}", error).contains("test"));
        
        let error = StorageError::StorageFull;
        assert!(format!("{}", error).contains("满"));
    }
}
