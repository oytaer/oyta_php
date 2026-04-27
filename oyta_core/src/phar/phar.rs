//! Phar 归档核心模块
//! 
//! 本模块实现 Phar 归档的主要功能，包括：
//! - Phar 文件创建
//! - Phar 文件读取
//! - 文件操作（添加、删除、提取）
//! - 签名验证

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::phar::compression::{PharCompression, PharCompressor};
use crate::phar::manifest::{PharEntry, PharEntryFlags, PharManifest};
use crate::phar::stub::{PharStub, PharStubBuilder};

/// Phar 签名类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PharSignatureType {
    /// MD5 签名
    Md5,
    /// SHA1 签名
    Sha1,
    /// SHA256 签名
    Sha256,
    /// SHA512 签名
    Sha512,
    /// OpenSSL 签名
    OpenSSL,
    /// 无签名
    None,
}

impl Default for PharSignatureType {
    fn default() -> Self {
        PharSignatureType::Sha1
    }
}

impl PharSignatureType {
    /// 获取签名类型标志
    pub fn flag(&self) -> u32 {
        match self {
            PharSignatureType::Md5 => 0x0001,
            PharSignatureType::Sha1 => 0x0002,
            PharSignatureType::Sha256 => 0x0003,
            PharSignatureType::Sha512 => 0x0004,
            PharSignatureType::OpenSSL => 0x0010,
            PharSignatureType::None => 0x0000,
        }
    }

    /// 从标志获取签名类型
    pub fn from_flag(flag: u32) -> Self {
        match flag {
            0x0001 => PharSignatureType::Md5,
            0x0002 => PharSignatureType::Sha1,
            0x0003 => PharSignatureType::Sha256,
            0x0004 => PharSignatureType::Sha512,
            0x0010 => PharSignatureType::OpenSSL,
            _ => PharSignatureType::None,
        }
    }

    /// 获取签名长度
    pub fn signature_len(&self) -> usize {
        match self {
            PharSignatureType::Md5 => 16,
            PharSignatureType::Sha1 => 20,
            PharSignatureType::Sha256 => 32,
            PharSignatureType::Sha512 => 64,
            PharSignatureType::OpenSSL => 0,
            PharSignatureType::None => 0,
        }
    }
}

/// Phar 文件信息
#[derive(Debug, Clone)]
pub struct PharFileInfo {
    /// 文件名
    pub name: String,
    /// 文件大小
    pub size: u32,
    /// 压缩后大小
    pub compressed_size: u32,
    /// 时间戳
    pub timestamp: u32,
    /// 是否压缩
    pub compressed: bool,
    /// 是否为目录
    pub is_dir: bool,
    /// CRC32 校验和
    pub crc32: u32,
}

impl PharFileInfo {
    /// 创建新的文件信息
    pub fn new(name: &str, size: u32) -> Self {
        PharFileInfo {
            name: name.to_string(),
            size,
            compressed_size: size,
            timestamp: 0,
            compressed: false,
            is_dir: false,
            crc32: 0,
        }
    }
}

/// Phar 数据类
#[derive(Debug, Clone)]
pub struct PharData {
    /// 文件路径
    path: PathBuf,
    /// 清单
    manifest: PharManifest,
    /// 是否已修改
    modified: bool,
}

impl PharData {
    /// 创建新的 Phar 数据
    pub fn new() -> Self {
        PharData {
            path: PathBuf::new(),
            manifest: PharManifest::new(),
            modified: false,
        }
    }

    /// 从文件加载
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let data = std::fs::read(path).map_err(|e| format!("读取文件失败: {}", e))?;
        Self::parse(&data)
    }

    /// 解析数据
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        if !data.starts_with(b"<?php") {
            return Err("不是有效的 Phar 文件".to_string());
        }
        let (stub, remaining) = PharStub::parse(data)?;
        let (manifest, _) = PharManifest::deserialize(remaining)?;
        Ok(PharData {
            path: PathBuf::new(),
            manifest,
            modified: false,
        })
    }

    /// 获取清单
    pub fn manifest(&self) -> &PharManifest {
        &self.manifest
    }

    /// 获取清单可变引用
    pub fn manifest_mut(&mut self) -> &mut PharManifest {
        self.modified = true;
        &mut self.manifest
    }

    /// 添加文件
    pub fn add_file(&mut self, name: &str, content: Vec<u8>) {
        let entry = PharEntry::new(name, content);
        self.manifest.add_entry(entry);
        self.modified = true;
    }

    /// 添加目录
    pub fn add_directory(&mut self, name: &str) {
        let entry = PharEntry::directory(name);
        self.manifest.add_entry(entry);
        self.modified = true;
    }

    /// 获取文件
    pub fn get_file(&self, name: &str) -> Option<&PharEntry> {
        self.manifest.get_entry(name)
    }

    /// 检查文件是否存在
    pub fn has_file(&self, name: &str) -> bool {
        self.manifest.has_entry(name)
    }

    /// 删除文件
    pub fn remove_file(&mut self, name: &str) -> bool {
        let removed = self.manifest.remove_entry(name);
        if removed {
            self.modified = true;
        }
        removed
    }

    /// 获取文件列表
    pub fn list_files(&self) -> Vec<&PharEntry> {
        self.manifest.entries().iter().collect()
    }

    /// 提取文件
    pub fn extract_file(&self, name: &str, dest: &Path) -> Result<(), String> {
        let entry = self.manifest.get_entry(name)
            .ok_or_else(|| format!("文件不存在: {}", name))?;
        std::fs::write(dest, &entry.content)
            .map_err(|e| format!("写入文件失败: {}", e))
    }

    /// 提取所有文件
    pub fn extract_all(&self, dest: &Path) -> Result<(), String> {
        for entry in self.manifest.entries() {
            let file_path = dest.join(&entry.name);
            if entry.is_dir() {
                std::fs::create_dir_all(&file_path)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            } else {
                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("创建目录失败: {}", e))?;
                }
                std::fs::write(&file_path, &entry.content)
                    .map_err(|e| format!("写入文件失败: {}", e))?;
            }
        }
        Ok(())
    }
}

impl Default for PharData {
    fn default() -> Self {
        Self::new()
    }
}

/// Phar 归档类
#[derive(Debug, Clone)]
pub struct Phar {
    /// 文件路径
    path: PathBuf,
    /// 清单
    manifest: PharManifest,
    /// Stub
    stub: PharStub,
    /// 签名类型
    signature_type: PharSignatureType,
    /// 签名数据
    signature: Vec<u8>,
    /// 是否已修改
    modified: bool,
    /// 默认压缩
    default_compression: PharCompression,
    /// 是否可执行
    executable: bool,
}

impl Phar {
    /// 创建新的 Phar 归档
    pub fn new() -> Self {
        Phar {
            path: PathBuf::new(),
            manifest: PharManifest::new(),
            stub: PharStubBuilder::new().build(),
            signature_type: PharSignatureType::Sha1,
            signature: Vec::new(),
            modified: false,
            default_compression: PharCompression::None,
            executable: true,
        }
    }

    /// 从文件加载 Phar
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let data = std::fs::read(path).map_err(|e| format!("读取文件失败: {}", e))?;
        Self::parse(&data)
    }

    /// 解析 Phar 数据
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        if !data.starts_with(b"<?php") {
            return Err("不是有效的 Phar 文件".to_string());
        }
        let (stub, remaining) = PharStub::parse(data)?;
        let (manifest, file_data) = PharManifest::deserialize(remaining)?;
        let (signature_type, signature) = Self::parse_signature(file_data)?;
        let mut phar = Phar::new();
        phar.stub = stub;
        phar.manifest = manifest;
        phar.signature_type = signature_type;
        phar.signature = signature;
        phar.read_file_contents(file_data)?;
        Ok(phar)
    }

    /// 解析签名
    fn parse_signature(data: &[u8]) -> Result<(PharSignatureType, Vec<u8>), String> {
        if data.len() < 8 {
            return Ok((PharSignatureType::None, Vec::new()));
        }
        let sig_len = u32::from_le_bytes([data[data.len() - 8], data[data.len() - 7], data[data.len() - 6], data[data.len() - 5]]);
        let sig_type = u32::from_le_bytes([data[data.len() - 4], data[data.len() - 3], data[data.len() - 2], data[data.len() - 1]]);
        let signature_type = PharSignatureType::from_flag(sig_type);
        if signature_type == PharSignatureType::None {
            return Ok((PharSignatureType::None, Vec::new()));
        }
        let sig_start = data.len() - 8 - sig_len as usize;
        if sig_start < 0 {
            return Err("签名数据无效".to_string());
        }
        let signature = data[sig_start..sig_start + sig_len as usize].to_vec();
        Ok((signature_type, signature))
    }

    /// 读取文件内容
    fn read_file_contents(&mut self, data: &[u8]) -> Result<(), String> {
        let mut offset = 0;
        for entry in &mut self.manifest.entries {
            let size = if entry.is_compressed() {
                entry.compressed_size as usize
            } else {
                entry.uncompressed_size as usize
            };
            if offset + size > data.len() {
                return Err("文件数据不完整".to_string());
            }
            let content = data[offset..offset + size].to_vec();
            if entry.is_compressed() {
                let compression = PharCompression::from_flags(entry.flags.bits());
                entry.content = PharCompressor::decompress(&content, compression)
                    .map_err(|e| format!("解压失败: {}", e))?;
            } else {
                entry.content = content;
            }
            offset += size;
        }
        Ok(())
    }

    /// 设置文件路径
    pub fn set_path(&mut self, path: &Path) {
        self.path = path.to_path_buf();
    }

    /// 获取文件路径
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 获取清单
    pub fn manifest(&self) -> &PharManifest {
        &self.manifest
    }

    /// 获取清单可变引用
    pub fn manifest_mut(&mut self) -> &mut PharManifest {
        self.modified = true;
        &mut self.manifest
    }

    /// 设置别名
    pub fn set_alias(&mut self, alias: &str) {
        self.manifest.set_alias(alias);
        self.modified = true;
    }

    /// 获取别名
    pub fn alias(&self) -> &str {
        self.manifest.alias()
    }

    /// 设置 Stub
    pub fn set_stub(&mut self, stub: PharStub) {
        self.stub = stub;
        self.modified = true;
    }

    /// 获取 Stub
    pub fn stub(&self) -> &PharStub {
        &self.stub
    }

    /// 设置签名类型
    pub fn set_signature_type(&mut self, sig_type: PharSignatureType) {
        self.signature_type = sig_type;
        self.modified = true;
    }

    /// 获取签名类型
    pub fn signature_type(&self) -> PharSignatureType {
        self.signature_type
    }

    /// 设置默认压缩
    pub fn set_compression(&mut self, compression: PharCompression) {
        self.default_compression = compression;
        self.modified = true;
    }

    /// 获取默认压缩
    pub fn compression(&self) -> PharCompression {
        self.default_compression
    }

    /// 添加文件
    pub fn add_file(&mut self, name: &str, content: Vec<u8>) {
        let mut entry = PharEntry::new(name, content);
        if self.default_compression != PharCompression::None {
            let compressed = PharCompressor::compress(&entry.content, self.default_compression).unwrap();
            entry.compressed_size = compressed.len() as u32;
            entry.content = compressed;
            entry.flags.set(match self.default_compression {
                PharCompression::Gzip => PharEntryFlags::COMPRESSED_GZ,
                PharCompression::Bzip2 => PharEntryFlags::COMPRESSED_BZ2,
                _ => PharEntryFlags::new(),
            });
        }
        self.manifest.add_entry(entry);
        self.modified = true;
    }

    /// 从文件系统添加文件
    pub fn add_file_from_path(&mut self, path: &Path, name: Option<&str>) -> Result<(), String> {
        let content = std::fs::read(path).map_err(|e| format!("读取文件失败: {}", e))?;
        let name = name.unwrap_or_else(|| path.file_name().and_then(|n| n.to_str()).unwrap_or(""));
        self.add_file(name, content);
        Ok(())
    }

    /// 添加目录
    pub fn add_directory(&mut self, name: &str) {
        let entry = PharEntry::directory(name);
        self.manifest.add_entry(entry);
        self.modified = true;
    }

    /// 从目录添加所有文件
    pub fn add_from_directory(&mut self, dir: &Path, strip_prefix: bool) -> Result<(), String> {
        let base = dir.canonicalize().map_err(|e| format!("获取绝对路径失败: {}", e))?;
        self.add_directory_recursive(&base, &base, strip_prefix)?;
        Ok(())
    }

    /// 递归添加目录
    fn add_directory_recursive(&mut self, base: &Path, current: &Path, strip_prefix: bool) -> Result<(), String> {
        let entries = std::fs::read_dir(current).map_err(|e| format!("读取目录失败: {}", e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
            let path = entry.path();
            let relative = path.strip_prefix(base).unwrap_or(&path);
            if path.is_dir() {
                let dir_name = relative.to_str().unwrap_or("").replace('\\', "/");
                self.add_directory(&format!("{}/", dir_name));
                self.add_directory_recursive(base, &path, strip_prefix)?;
            } else {
                let content = std::fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))?;
                let name = relative.to_str().unwrap_or("").replace('\\', "/");
                self.add_file(&name, content);
            }
        }
        Ok(())
    }

    /// 获取文件
    pub fn get_file(&self, name: &str) -> Option<&PharEntry> {
        self.manifest.get_entry(name)
    }

    /// 获取文件内容
    pub fn get_file_content(&self, name: &str) -> Option<Vec<u8>> {
        let entry = self.manifest.get_entry(name)?;
        if entry.is_compressed() {
            let compression = PharCompression::from_flags(entry.flags.bits());
            Some(PharCompressor::decompress(&entry.content, compression).unwrap_or_default())
        } else {
            Some(entry.content.clone())
        }
    }

    /// 检查文件是否存在
    pub fn has_file(&self, name: &str) -> bool {
        self.manifest.has_entry(name)
    }

    /// 删除文件
    pub fn remove_file(&mut self, name: &str) -> bool {
        let removed = self.manifest.remove_entry(name);
        if removed {
            self.modified = true;
        }
        removed
    }

    /// 获取文件列表
    pub fn list_files(&self) -> Vec<PharFileInfo> {
        self.manifest.entries().iter().map(|e| PharFileInfo {
            name: e.name.clone(),
            size: e.uncompressed_size,
            compressed_size: e.compressed_size,
            timestamp: e.timestamp,
            compressed: e.is_compressed(),
            is_dir: e.is_dir(),
            crc32: e.crc32,
        }).collect()
    }

    /// 提取文件
    pub fn extract_file(&self, name: &str, dest: &Path) -> Result<(), String> {
        let entry = self.manifest.get_entry(name)
            .ok_or_else(|| format!("文件不存在: {}", name))?;
        let content = if entry.is_compressed() {
            let compression = PharCompression::from_flags(entry.flags.bits());
            PharCompressor::decompress(&entry.content, compression)
                .map_err(|e| format!("解压失败: {}", e))?
        } else {
            entry.content.clone()
        };
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        }
        std::fs::write(dest, content)
            .map_err(|e| format!("写入文件失败: {}", e))
    }

    /// 提取所有文件
    pub fn extract_all(&self, dest: &Path) -> Result<(), String> {
        for entry in self.manifest.entries() {
            let file_path = dest.join(&entry.name);
            if entry.is_dir() {
                std::fs::create_dir_all(&file_path)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            } else {
                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("创建目录失败: {}", e))?;
                }
                let content = if entry.is_compressed() {
                    let compression = PharCompression::from_flags(entry.flags.bits());
                    PharCompressor::decompress(&entry.content, compression)
                        .map_err(|e| format!("解压失败: {}", e))?
                } else {
                    entry.content.clone()
                };
                std::fs::write(&file_path, content)
                    .map_err(|e| format!("写入文件失败: {}", e))?;
            }
        }
        Ok(())
    }

    /// 序列化 Phar
    pub fn serialize(&mut self) -> Result<Vec<u8>, String> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.stub.serialize());
        data.extend_from_slice(&self.manifest.serialize());
        for entry in self.manifest.entries() {
            data.extend_from_slice(&entry.content);
        }
        let signature = self.calculate_signature(&data)?;
        data.extend_from_slice(&signature);
        data.extend_from_slice(&(signature.len() as u32).to_le_bytes());
        data.extend_from_slice(&self.signature_type.flag().to_le_bytes());
        self.signature = signature;
        self.modified = false;
        Ok(data)
    }

    /// 计算签名
    fn calculate_signature(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        match self.signature_type {
            PharSignatureType::Md5 => {
                let digest = md5::compute(data);
                Ok(digest.0.to_vec())
            }
            PharSignatureType::Sha1 => {
                use sha1::{Sha1, Digest};
                let mut hasher = Sha1::new();
                hasher.update(data);
                Ok(hasher.finalize().to_vec())
            }
            PharSignatureType::Sha256 => {
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(data);
                Ok(hasher.finalize().to_vec())
            }
            PharSignatureType::Sha512 => {
                use sha2::{Sha512, Digest};
                let mut hasher = Sha512::new();
                hasher.update(data);
                Ok(hasher.finalize().to_vec())
            }
            PharSignatureType::OpenSSL => {
                Err("OpenSSL 签名需要私钥".to_string())
            }
            PharSignatureType::None => {
                Ok(Vec::new())
            }
        }
    }

    /// 验证签名
    pub fn verify_signature(&self) -> Result<bool, String> {
        if self.signature_type == PharSignatureType::None {
            return Ok(true);
        }
        let mut data = Vec::new();
        data.extend_from_slice(&self.stub.serialize());
        data.extend_from_slice(&self.manifest.serialize());
        for entry in self.manifest.entries() {
            data.extend_from_slice(&entry.content);
        }
        let calculated = self.calculate_signature(&data)?;
        Ok(calculated == self.signature)
    }

    /// 保存到文件
    pub fn save(&mut self, path: &Path) -> Result<(), String> {
        let data = self.serialize()?;
        std::fs::write(path, data).map_err(|e| format!("写入文件失败: {}", e))?;
        self.path = path.to_path_buf();
        Ok(())
    }

    /// 获取文件数量
    pub fn count(&self) -> u32 {
        self.manifest.entry_count()
    }

    /// 检查是否已修改
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// 设置可执行
    pub fn set_executable(&mut self, executable: bool) {
        self.executable = executable;
        self.modified = true;
    }

    /// 检查是否可执行
    pub fn is_executable(&self) -> bool {
        self.executable
    }

    /// 获取元数据
    pub fn get_metadata(&self) -> Option<&[u8]> {
        self.manifest.metadata()
    }

    /// 设置元数据
    pub fn set_metadata(&mut self, metadata: Vec<u8>) {
        self.manifest.set_metadata(metadata);
        self.modified = true;
    }

    /// 压缩所有文件
    pub fn compress_all(&mut self, compression: PharCompression) -> Result<(), String> {
        for entry in &mut self.manifest.entries {
            if entry.is_dir() || entry.is_compressed() {
                continue;
            }
            let compressed = PharCompressor::compress(&entry.content, compression)
                .map_err(|e| format!("压缩失败: {}", e))?;
            entry.compressed_size = compressed.len() as u32;
            entry.content = compressed;
            entry.flags.set(match compression {
                PharCompression::Gzip => PharEntryFlags::COMPRESSED_GZ,
                PharCompression::Bzip2 => PharEntryFlags::COMPRESSED_BZ2,
                _ => PharEntryFlags::new(),
            });
        }
        self.modified = true;
        Ok(())
    }

    /// 解压所有文件
    pub fn decompress_all(&mut self) -> Result<(), String> {
        for entry in &mut self.manifest.entries {
            if !entry.is_compressed() {
                continue;
            }
            let compression = PharCompression::from_flags(entry.flags.bits());
            let decompressed = PharCompressor::decompress(&entry.content, compression)
                .map_err(|e| format!("解压失败: {}", e))?;
            entry.content = decompressed;
            entry.compressed_size = entry.uncompressed_size;
            entry.flags.clear(PharEntryFlags::COMPRESSED_MASK);
        }
        self.modified = true;
        Ok(())
    }
}

impl Default for Phar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phar_creation() {
        let phar = Phar::new();
        assert_eq!(phar.count(), 0);
        assert!(phar.is_executable());
    }

    #[test]
    fn test_add_file() {
        let mut phar = Phar::new();
        phar.add_file("test.php", b"<?php echo 'hello';".to_vec());
        assert_eq!(phar.count(), 1);
        assert!(phar.has_file("test.php"));
    }

    #[test]
    fn test_remove_file() {
        let mut phar = Phar::new();
        phar.add_file("test.php", b"<?php echo 'hello';".to_vec());
        assert!(phar.remove_file("test.php"));
        assert_eq!(phar.count(), 0);
    }

    #[test]
    fn test_serialize_deserialize() {
        let mut phar = Phar::new();
        phar.add_file("index.php", b"<?php echo 'hello';".to_vec());
        phar.set_alias("test.phar");
        let serialized = phar.serialize().unwrap();
        let parsed = Phar::parse(&serialized).unwrap();
        assert_eq!(parsed.alias(), "test.phar");
        assert_eq!(parsed.count(), 1);
    }

    #[test]
    fn test_signature_type() {
        assert_eq!(PharSignatureType::Sha1.flag(), 0x0002);
        assert_eq!(PharSignatureType::from_flag(0x0002), PharSignatureType::Sha1);
    }

    #[test]
    fn test_compression() {
        let mut phar = Phar::new();
        phar.set_compression(PharCompression::Gzip);
        phar.add_file("test.txt", b"This is a test file for compression.".to_vec());
        let entry = phar.get_file("test.txt").unwrap();
        assert!(entry.is_compressed());
    }
}
