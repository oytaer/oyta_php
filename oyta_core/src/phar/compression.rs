//! Phar 压缩模块
//! 
//! 本模块实现 Phar 归档的压缩功能，包括：
//! - Gzip 压缩/解压
//! - Bzip2 压缩/解压
//! - 压缩类型检测

use std::io::{Read, Write};

/// Phar 压缩类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PharCompression {
    /// 无压缩
    None,
    /// Gzip 压缩
    Gzip,
    /// Bzip2 压缩
    Bzip2,
}

impl Default for PharCompression {
    fn default() -> Self {
        PharCompression::None
    }
}

impl PharCompression {
    /// 从标志位获取压缩类型
    /// 
    /// # 参数
    /// - `flags`: 条目标志
    /// 
    /// # 返回
    /// 返回压缩类型
    pub fn from_flags(flags: u32) -> Self {
        match flags & 0x0000F000 {
            0x00001000 => PharCompression::Gzip,
            0x00002000 => PharCompression::Bzip2,
            _ => PharCompression::None,
        }
    }

    /// 转换为标志位
    /// 
    /// # 返回
    /// 返回标志位值
    pub fn to_flags(&self) -> u32 {
        match self {
            PharCompression::None => 0,
            PharCompression::Gzip => 0x00001000,
            PharCompression::Bzip2 => 0x00002000,
        }
    }

    /// 获取压缩类型名称
    /// 
    /// # 返回
    /// 返回压缩类型名称字符串
    pub fn name(&self) -> &'static str {
        match self {
            PharCompression::None => "none",
            PharCompression::Gzip => "gzip",
            PharCompression::Bzip2 => "bzip2",
        }
    }
}

/// Phar 压缩器
/// 
/// 提供压缩和解压功能
pub struct PharCompressor;

impl PharCompressor {
    /// 压缩数据
    /// 
    /// # 参数
    /// - `data`: 原始数据
    /// - `compression`: 压缩类型
    /// 
    /// # 返回
    /// 成功返回压缩后的数据，失败返回错误信息
    pub fn compress(data: &[u8], compression: PharCompression) -> Result<Vec<u8>, String> {
        match compression {
            PharCompression::None => Ok(data.to_vec()),
            PharCompression::Gzip => Self::compress_gzip(data),
            PharCompression::Bzip2 => Self::compress_bzip2(data),
        }
    }

    /// 解压数据
    /// 
    /// # 参数
    /// - `data`: 压缩数据
    /// - `compression`: 压缩类型
    /// 
    /// # 返回
    /// 成功返回解压后的数据，失败返回错误信息
    pub fn decompress(data: &[u8], compression: PharCompression) -> Result<Vec<u8>, String> {
        match compression {
            PharCompression::None => Ok(data.to_vec()),
            PharCompression::Gzip => Self::decompress_gzip(data),
            PharCompression::Bzip2 => Self::decompress_bzip2(data),
        }
    }

    /// Gzip 压缩
    fn compress_gzip(data: &[u8]) -> Result<Vec<u8>, String> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).map_err(|e| format!("Gzip 压缩失败: {}", e))?;
        encoder.finish().map_err(|e| format!("Gzip 完成失败: {}", e))
    }

    /// Gzip 解压
    fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, String> {
        use flate2::read::GzDecoder;
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).map_err(|e| format!("Gzip 解压失败: {}", e))?;
        Ok(decompressed)
    }

    /// Bzip2 压缩
    fn compress_bzip2(data: &[u8]) -> Result<Vec<u8>, String> {
        use bzip2::write::BzEncoder;
        use bzip2::Compression;
        let mut encoder = BzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).map_err(|e| format!("Bzip2 压缩失败: {}", e))?;
        encoder.finish().map_err(|e| format!("Bzip2 完成失败: {}", e))
    }

    /// Bzip2 解压
    fn decompress_bzip2(data: &[u8]) -> Result<Vec<u8>, String> {
        use bzip2::read::BzDecoder;
        let mut decoder = BzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).map_err(|e| format!("Bzip2 解压失败: {}", e))?;
        Ok(decompressed)
    }

    /// 检测压缩类型
    /// 
    /// 通过魔术字节检测数据的压缩类型
    /// 
    /// # 参数
    /// - `data`: 数据
    /// 
    /// # 返回
    /// 返回检测到的压缩类型
    pub fn detect_compression(data: &[u8]) -> PharCompression {
        if data.len() < 2 {
            return PharCompression::None;
        }
        // Gzip 魔术字节: 1f 8b
        if data[0] == 0x1f && data[1] == 0x8b {
            return PharCompression::Gzip;
        }
        // Bzip2 魔术字节: 42 5a (BZ)
        if data[0] == 0x42 && data[1] == 0x5a {
            return PharCompression::Bzip2;
        }
        PharCompression::None
    }

    /// 获取压缩级别
    /// 
    /// # 参数
    /// - `compression`: 压缩类型
    /// - `level`: 压缩级别 (1-9)
    /// 
    /// # 返回
    /// 返回压缩后的数据
    pub fn compress_with_level(data: &[u8], compression: PharCompression, level: u32) -> Result<Vec<u8>, String> {
        let level = level.clamp(1, 9);
        match compression {
            PharCompression::None => Ok(data.to_vec()),
            PharCompression::Gzip => Self::compress_gzip_with_level(data, level),
            PharCompression::Bzip2 => Self::compress_bzip2_with_level(data, level),
        }
    }

    /// Gzip 压缩（指定级别）
    fn compress_gzip_with_level(data: &[u8], level: u32) -> Result<Vec<u8>, String> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        let compression = Compression::new(level);
        let mut encoder = GzEncoder::new(Vec::new(), compression);
        encoder.write_all(data).map_err(|e| format!("Gzip 压缩失败: {}", e))?;
        encoder.finish().map_err(|e| format!("Gzip 完成失败: {}", e))
    }

    /// Bzip2 压缩（指定级别）
    fn compress_bzip2_with_level(data: &[u8], level: u32) -> Result<Vec<u8>, String> {
        use bzip2::write::BzEncoder;
        use bzip2::Compression;
        let compression = Compression::new(level);
        let mut encoder = BzEncoder::new(Vec::new(), compression);
        encoder.write_all(data).map_err(|e| format!("Bzip2 压缩失败: {}", e))?;
        encoder.finish().map_err(|e| format!("Bzip2 完成失败: {}", e))
    }

    /// 计算压缩率
    /// 
    /// # 参数
    /// - `original_size`: 原始大小
    /// - `compressed_size`: 压缩后大小
    /// 
    /// # 返回
    /// 返回压缩率 (0.0-1.0)
    pub fn compression_ratio(original_size: u64, compressed_size: u64) -> f64 {
        if original_size == 0 {
            return 0.0;
        }
        1.0 - (compressed_size as f64 / original_size as f64)
    }

    /// 检查是否支持压缩类型
    /// 
    /// # 参数
    /// - `compression`: 压缩类型
    /// 
    /// # 返回
    /// 如果支持返回 true
    pub fn is_supported(compression: PharCompression) -> bool {
        matches!(compression, PharCompression::None | PharCompression::Gzip | PharCompression::Bzip2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_type() {
        assert_eq!(PharCompression::from_flags(0x00001000), PharCompression::Gzip);
        assert_eq!(PharCompression::from_flags(0x00002000), PharCompression::Bzip2);
        assert_eq!(PharCompression::from_flags(0), PharCompression::None);
    }

    #[test]
    fn test_compress_decompress_gzip() {
        let data = b"Hello, World! This is a test string for compression.";
        let compressed = PharCompressor::compress(data, PharCompression::Gzip).unwrap();
        let decompressed = PharCompressor::decompress(&compressed, PharCompression::Gzip).unwrap();
        assert_eq!(data.to_vec(), decompressed);
    }

    #[test]
    fn test_detect_compression() {
        let gzip_data = [0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(PharCompressor::detect_compression(&gzip_data), PharCompression::Gzip);
        let bzip2_data = [0x42, 0x5a, 0x68, 0x39];
        assert_eq!(PharCompressor::detect_compression(&bzip2_data), PharCompression::Bzip2);
    }

    #[test]
    fn test_compression_ratio() {
        let ratio = PharCompressor::compression_ratio(1000, 500);
        assert!((ratio - 0.5).abs() < 0.001);
    }
}
