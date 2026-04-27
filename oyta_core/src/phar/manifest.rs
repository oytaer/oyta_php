//! Phar 清单模块
//! 
//! 本模块实现 Phar 清单的解析和生成，包括：
//! - 清单结构定义
//! - 条目元数据
//! - 清单序列化和反序列化

use std::collections::HashMap;
use std::io::{Read, Write};

/// Phar 条目标志
/// 
/// 定义文件条目的属性标志
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PharEntryFlags(pub u32);

impl PharEntryFlags {
    /// 文件已压缩（Gzip）
    pub const COMPRESSED_GZ: Self = PharEntryFlags(0x00001000);
    /// 文件已压缩（Bzip2）
    pub const COMPRESSED_BZ2: Self = PharEntryFlags(0x00002000);
    /// 文件是目录
    pub const IS_DIR: Self = PharEntryFlags(0x00000002);
    /// 文件是可执行的 PHP 脚本
    pub const IS_PHP: Self = PharEntryFlags(0x00000001);
    /// 文件权限掩码
    pub const PERMS_MASK: Self = PharEntryFlags(0x0000FFFF);
    /// 压缩掩码
    pub const COMPRESSED_MASK: Self = PharEntryFlags(0x0000F000);

    /// 创建新的标志
    pub fn new() -> Self {
        PharEntryFlags(0o644) // 默认权限
    }

    /// 从 u32 创建标志
    pub fn from_bits(bits: u32) -> Self {
        PharEntryFlags(bits)
    }

    /// 获取标志位
    pub fn bits(&self) -> u32 {
        self.0
    }

    /// 检查是否设置了指定标志
    pub fn contains(&self, flag: PharEntryFlags) -> bool {
        (self.0 & flag.0) != 0
    }

    /// 设置标志
    pub fn set(&mut self, flag: PharEntryFlags) {
        self.0 |= flag.0;
    }

    /// 清除标志
    pub fn clear(&mut self, flag: PharEntryFlags) {
        self.0 &= !flag.0;
    }

    /// 检查是否已压缩
    pub fn is_compressed(&self) -> bool {
        self.contains(Self::COMPRESSED_GZ) || self.contains(Self::COMPRESSED_BZ2)
    }

    /// 获取压缩类型
    pub fn compression_type(&self) -> Option<u32> {
        if self.contains(Self::COMPRESSED_GZ) {
            Some(1)
        } else if self.contains(Self::COMPRESSED_BZ2) {
            Some(2)
        } else {
            None
        }
    }

    /// 检查是否为目录
    pub fn is_dir(&self) -> bool {
        self.contains(Self::IS_DIR)
    }

    /// 设置为目录
    pub fn set_dir(&mut self, is_dir: bool) {
        if is_dir {
            self.set(Self::IS_DIR);
        } else {
            self.clear(Self::IS_DIR);
        }
    }

    /// 检查是否为 PHP 文件
    pub fn is_php(&self) -> bool {
        self.contains(Self::IS_PHP)
    }

    /// 设置为 PHP 文件
    pub fn set_php(&mut self, is_php: bool) {
        if is_php {
            self.set(Self::IS_PHP);
        } else {
            self.clear(Self::IS_PHP);
        }
    }

    /// 获取权限
    pub fn permissions(&self) -> u32 {
        self.0 & Self::PERMS_MASK.0
    }

    /// 设置权限
    pub fn set_permissions(&mut self, perms: u32) {
        self.0 = (self.0 & !Self::PERMS_MASK.0) | (perms & Self::PERMS_MASK.0);
    }
}

impl Default for PharEntryFlags {
    fn default() -> Self {
        Self::new()
    }
}

/// Phar 条目
/// 
/// 表示 Phar 归档中的一个文件条目
#[derive(Debug, Clone)]
pub struct PharEntry {
    /// 文件名（相对路径）
    pub name: String,
    /// 未压缩大小
    pub uncompressed_size: u32,
    /// 时间戳
    pub timestamp: u32,
    /// 压缩后大小
    pub compressed_size: u32,
    /// CRC32 校验和
    pub crc32: u32,
    /// 标志
    pub flags: PharEntryFlags,
    /// 元数据（序列化的 PHP 值）
    pub metadata: Option<Vec<u8>>,
    /// 文件内容
    pub content: Vec<u8>,
}

impl PharEntry {
    /// 创建新的条目
    /// 
    /// # 参数
    /// - `name`: 文件名
    /// - `content`: 文件内容
    /// 
    /// # 返回
    /// 返回新条目
    pub fn new(name: &str, content: Vec<u8>) -> Self {
        let mut flags = PharEntryFlags::new();
        if name.ends_with('/') {
            flags.set_dir(true);
        } else if name.ends_with(".php") {
            flags.set_php(true);
        }
        let crc32 = crc32fast::hash(&content);
        PharEntry {
            name: name.to_string(),
            uncompressed_size: content.len() as u32,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32,
            compressed_size: content.len() as u32,
            crc32,
            flags,
            metadata: None,
            content,
        }
    }

    /// 创建目录条目
    /// 
    /// # 参数
    /// - `name`: 目录名
    /// 
    /// # 返回
    /// 返回目录条目
    pub fn directory(name: &str) -> Self {
        let name = if name.ends_with('/') { name.to_string() } else { format!("{}/", name) };
        let mut flags = PharEntryFlags::new();
        flags.set_dir(true);
        PharEntry {
            name,
            uncompressed_size: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32,
            compressed_size: 0,
            crc32: 0,
            flags,
            metadata: None,
            content: Vec::new(),
        }
    }

    /// 获取文件名
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取未压缩大小
    pub fn uncompressed_size(&self) -> u32 {
        self.uncompressed_size
    }

    /// 获取压缩后大小
    pub fn compressed_size(&self) -> u32 {
        self.compressed_size
    }

    /// 检查是否已压缩
    pub fn is_compressed(&self) -> bool {
        self.flags.is_compressed()
    }

    /// 检查是否为目录
    pub fn is_dir(&self) -> bool {
        self.flags.is_dir()
    }

    /// 检查是否为 PHP 文件
    pub fn is_php(&self) -> bool {
        self.flags.is_php()
    }

    /// 获取内容
    pub fn content(&self) -> &[u8] {
        &self.content
    }

    /// 设置内容
    pub fn set_content(&mut self, content: Vec<u8>) {
        self.crc32 = crc32fast::hash(&content);
        self.uncompressed_size = content.len() as u32;
        self.content = content;
    }

    /// 设置元数据
    pub fn set_metadata(&mut self, metadata: Vec<u8>) {
        self.metadata = Some(metadata);
    }

    /// 获取元数据
    pub fn metadata(&self) -> Option<&[u8]> {
        self.metadata.as_deref()
    }
}

/// Phar 清单
/// 
/// 包含 Phar 归档的所有元数据
#[derive(Debug, Clone)]
pub struct PharManifest {
    /// 版本号
    pub version: u32,
    /// 标志
    pub flags: u32,
    /// 别名
    pub alias: String,
    /// 元数据
    pub metadata: Option<Vec<u8>>,
    /// 条目数量
    pub entry_count: u32,
    /// 条目列表
    pub entries: Vec<PharEntry>,
    /// 文件条目映射
    entry_map: HashMap<String, usize>,
}

impl PharManifest {
    /// 创建新的清单
    pub fn new() -> Self {
        PharManifest {
            version: 0x00010000, // 版本 1.0
            flags: 0,
            alias: String::new(),
            metadata: None,
            entry_count: 0,
            entries: Vec::new(),
            entry_map: HashMap::new(),
        }
    }

    /// 设置别名
    pub fn set_alias(&mut self, alias: &str) {
        self.alias = alias.to_string();
    }

    /// 获取别名
    pub fn alias(&self) -> &str {
        &self.alias
    }

    /// 设置元数据
    pub fn set_metadata(&mut self, metadata: Vec<u8>) {
        self.metadata = Some(metadata);
    }

    /// 获取元数据
    pub fn metadata(&self) -> Option<&[u8]> {
        self.metadata.as_deref()
    }

    /// 添加条目
    /// 
    /// # 参数
    /// - `entry`: 条目
    pub fn add_entry(&mut self, entry: PharEntry) {
        let name = entry.name.clone();
        let idx = self.entries.len();
        self.entries.push(entry);
        self.entry_map.insert(name, idx);
        self.entry_count = self.entries.len() as u32;
    }

    /// 获取条目
    /// 
    /// # 参数
    /// - `name`: 文件名
    /// 
    /// # 返回
    /// 返回条目引用
    pub fn get_entry(&self, name: &str) -> Option<&PharEntry> {
        self.entry_map.get(name).map(|&idx| &self.entries[idx])
    }

    /// 获取条目可变引用
    /// 
    /// # 参数
    /// - `name`: 文件名
    /// 
    /// # 返回
    /// 返回条目可变引用
    pub fn get_entry_mut(&mut self, name: &str) -> Option<&mut PharEntry> {
        if let Some(&idx) = self.entry_map.get(name) {
            Some(&mut self.entries[idx])
        } else {
            None
        }
    }

    /// 检查条目是否存在
    /// 
    /// # 参数
    /// - `name`: 文件名
    /// 
    /// # 返回
    /// 如果存在返回 true
    pub fn has_entry(&self, name: &str) -> bool {
        self.entry_map.contains_key(name)
    }

    /// 删除条目
    /// 
    /// # 参数
    /// - `name`: 文件名
    /// 
    /// # 返回
    /// 如果删除成功返回 true
    pub fn remove_entry(&mut self, name: &str) -> bool {
        if let Some(idx) = self.entry_map.remove(name) {
            self.entries.remove(idx);
            // 更新映射
            self.entry_map.clear();
            for (i, entry) in self.entries.iter().enumerate() {
                self.entry_map.insert(entry.name.clone(), i);
            }
            self.entry_count = self.entries.len() as u32;
            true
        } else {
            false
        }
    }

    /// 获取条目数量
    pub fn entry_count(&self) -> u32 {
        self.entry_count
    }

    /// 获取所有条目
    pub fn entries(&self) -> &[PharEntry] {
        &self.entries
    }

    /// 序列化清单
    /// 
    /// # 返回
    /// 返回序列化后的字节
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        // 清单长度占位符（稍后填充）
        buf.extend_from_slice(&0u32.to_le_bytes());
        // 条目数量
        buf.extend_from_slice(&self.entry_count.to_le_bytes());
        // 版本
        buf.extend_from_slice(&self.version.to_le_bytes());
        // 标志
        buf.extend_from_slice(&self.flags.to_le_bytes());
        // 别名长度和别名
        let alias_bytes = self.alias.as_bytes();
        buf.extend_from_slice(&(alias_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(alias_bytes);
        // 元数据长度和元数据
        if let Some(ref metadata) = self.metadata {
            buf.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
            buf.extend_from_slice(metadata);
        } else {
            buf.extend_from_slice(&0u32.to_le_bytes());
        }
        // 条目信息
        for entry in &self.entries {
            self.serialize_entry(&mut buf, entry);
        }
        // 更新清单长度
        let manifest_len = (buf.len() - 4) as u32;
        buf[0..4].copy_from_slice(&manifest_len.to_le_bytes());
        buf
    }

    /// 序列化条目
    fn serialize_entry(&self, buf: &mut Vec<u8>, entry: &PharEntry) {
        // 文件名长度和文件名
        let name_bytes = entry.name.as_bytes();
        buf.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(name_bytes);
        // 未压缩大小
        buf.extend_from_slice(&entry.uncompressed_size.to_le_bytes());
        // 时间戳
        buf.extend_from_slice(&entry.timestamp.to_le_bytes());
        // 压缩后大小
        buf.extend_from_slice(&entry.compressed_size.to_le_bytes());
        // CRC32
        buf.extend_from_slice(&entry.crc32.to_le_bytes());
        // 标志
        buf.extend_from_slice(&entry.flags.bits().to_le_bytes());
        // 元数据长度和元数据
        if let Some(ref metadata) = entry.metadata {
            buf.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
            buf.extend_from_slice(metadata);
        } else {
            buf.extend_from_slice(&0u32.to_le_bytes());
        }
    }

    /// 反序列化清单
    /// 
    /// # 参数
    /// - `data`: 字节数据
    /// 
    /// # 返回
    /// 成功返回清单和剩余数据，失败返回错误信息
    pub fn deserialize(data: &[u8]) -> Result<(Self, &[u8]), String> {
        if data.len() < 4 {
            return Err("数据太短".to_string());
        }
        let mut manifest = PharManifest::new();
        let mut offset = 0;
        // 读取清单长度
        let manifest_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        offset += 4;
        if data.len() < offset + manifest_len {
            return Err("清单数据不完整".to_string());
        }
        // 条目数量
        if data.len() < offset + 4 {
            return Err("缺少条目数量".to_string());
        }
        manifest.entry_count = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]);
        offset += 4;
        // 版本
        if data.len() < offset + 4 {
            return Err("缺少版本号".to_string());
        }
        manifest.version = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]);
        offset += 4;
        // 标志
        if data.len() < offset + 4 {
            return Err("缺少标志".to_string());
        }
        manifest.flags = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]);
        offset += 4;
        // 别名
        if data.len() < offset + 4 {
            return Err("缺少别名长度".to_string());
        }
        let alias_len = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]) as usize;
        offset += 4;
        if data.len() < offset + alias_len {
            return Err("别名数据不完整".to_string());
        }
        manifest.alias = String::from_utf8_lossy(&data[offset..offset + alias_len]).to_string();
        offset += alias_len;
        // 元数据
        if data.len() < offset + 4 {
            return Err("缺少元数据长度".to_string());
        }
        let metadata_len = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]) as usize;
        offset += 4;
        if metadata_len > 0 {
            if data.len() < offset + metadata_len {
                return Err("元数据不完整".to_string());
            }
            manifest.metadata = Some(data[offset..offset + metadata_len].to_vec());
            offset += metadata_len;
        }
        // 读取条目
        for _ in 0..manifest.entry_count {
            let (entry, new_offset) = Self::deserialize_entry(data, offset)?;
            manifest.entry_map.insert(entry.name.clone(), manifest.entries.len());
            manifest.entries.push(entry);
            offset = new_offset;
        }
        Ok((manifest, &data[manifest_len + 4..]))
    }

    /// 反序列化条目
    fn deserialize_entry(data: &[u8], offset: usize) -> Result<(PharEntry, usize), String> {
        let mut offset = offset;
        // 文件名
        if data.len() < offset + 4 {
            return Err("缺少文件名长度".to_string());
        }
        let name_len = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]) as usize;
        offset += 4;
        if data.len() < offset + name_len {
            return Err("文件名数据不完整".to_string());
        }
        let name = String::from_utf8_lossy(&data[offset..offset + name_len]).to_string();
        offset += name_len;
        // 未压缩大小
        if data.len() < offset + 4 {
            return Err("缺少未压缩大小".to_string());
        }
        let uncompressed_size = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]);
        offset += 4;
        // 时间戳
        if data.len() < offset + 4 {
            return Err("缺少时间戳".to_string());
        }
        let timestamp = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]);
        offset += 4;
        // 压缩后大小
        if data.len() < offset + 4 {
            return Err("缺少压缩后大小".to_string());
        }
        let compressed_size = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]);
        offset += 4;
        // CRC32
        if data.len() < offset + 4 {
            return Err("缺少 CRC32".to_string());
        }
        let crc32 = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]);
        offset += 4;
        // 标志
        if data.len() < offset + 4 {
            return Err("缺少标志".to_string());
        }
        let flags = PharEntryFlags::from_bits(u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]));
        offset += 4;
        // 元数据
        if data.len() < offset + 4 {
            return Err("缺少条目元数据长度".to_string());
        }
        let metadata_len = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]) as usize;
        offset += 4;
        let metadata = if metadata_len > 0 {
            if data.len() < offset + metadata_len {
                return Err("条目元数据不完整".to_string());
            }
            Some(data[offset..offset + metadata_len].to_vec())
        } else {
            None
        };
        offset += metadata_len;
        Ok((
            PharEntry {
                name,
                uncompressed_size,
                timestamp,
                compressed_size,
                crc32,
                flags,
                metadata,
                content: Vec::new(),
            },
            offset,
        ))
    }
}

impl Default for PharManifest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_flags() {
        let mut flags = PharEntryFlags::new();
        assert!(!flags.is_compressed());
        flags.set(PharEntryFlags::COMPRESSED_GZ);
        assert!(flags.is_compressed());
        assert!(!flags.is_dir());
        flags.set_dir(true);
        assert!(flags.is_dir());
    }

    #[test]
    fn test_phar_entry() {
        let entry = PharEntry::new("test.php", b"<?php echo 'hello';".to_vec());
        assert_eq!(entry.name(), "test.php");
        assert!(entry.is_php());
        assert!(!entry.is_dir());
    }

    #[test]
    fn test_directory_entry() {
        let entry = PharEntry::directory("src/");
        assert!(entry.is_dir());
    }

    #[test]
    fn test_manifest() {
        let mut manifest = PharManifest::new();
        manifest.set_alias("test.phar");
        let entry = PharEntry::new("index.php", b"<?php echo 'hello';".to_vec());
        manifest.add_entry(entry);
        assert_eq!(manifest.entry_count(), 1);
        assert!(manifest.has_entry("index.php"));
    }

    #[test]
    fn test_manifest_serialize_deserialize() {
        let mut manifest = PharManifest::new();
        manifest.set_alias("test.phar");
        let entry = PharEntry::new("index.php", b"<?php echo 'hello';".to_vec());
        manifest.add_entry(entry);
        let serialized = manifest.serialize();
        let (deserialized, _) = PharManifest::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.alias(), "test.phar");
        assert_eq!(deserialized.entry_count(), 1);
    }
}
