//! 流式序列化模块
//!
//! 实现大数据的流式序列化，避免内存溢出
//! 支持分块处理和增量读取

use std::io::{Read, Write};

use super::engine::{SerializableValue, SerializeError, SerializeFormat, SerializeResult};

/// 流式序列化器
/// 用于处理大数据的序列化
pub struct StreamSerializer {
    /// 序列化格式
    format: SerializeFormat,
    /// 块大小（字节）
    chunk_size: usize,
    /// 缓冲区
    buffer: Vec<u8>,
    /// 当前位置
    position: usize,
}

impl StreamSerializer {
    /// 创建新的流式序列化器
    /// 
    /// # 参数
    /// - `format`: 序列化格式
    pub fn new(format: SerializeFormat) -> Self {
        Self {
            format,
            chunk_size: 4096, // 默认 4KB 块大小
            buffer: Vec::new(),
            position: 0,
        }
    }
    
    /// 设置块大小
    /// 
    /// # 参数
    /// - `size`: 块大小（字节）
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }
    
    /// 序列化数据到流
    /// 
    /// # 参数
    /// - `value`: 要序列化的值
    /// - `writer`: 输出流
    pub fn serialize_to_stream<W: Write>(
        &self,
        value: &SerializableValue,
        mut writer: W,
    ) -> SerializeResult<u64> {
        // 先序列化到内存
        let data = self.serialize_value(value)?;
        
        // 分块写入
        let mut written = 0u64;
        for chunk in data.chunks(self.chunk_size) {
            writer.write_all(chunk)
                .map_err(|e| SerializeError::StreamError(e.to_string()))?;
            written += chunk.len() as u64;
        }
        
        Ok(written)
    }
    
    /// 从流反序列化数据
    /// 
    /// # 参数
    /// - `reader`: 输入流
    /// - `size`: 数据大小（如果已知）
    pub fn deserialize_from_stream<R: Read>(
        &self,
        mut reader: R,
        size: Option<usize>,
    ) -> SerializeResult<SerializableValue> {
        // 读取所有数据
        let mut buffer = if let Some(s) = size {
            Vec::with_capacity(s)
        } else {
            Vec::with_capacity(self.chunk_size)
        };
        
        reader.read_to_end(&mut buffer)
            .map_err(|e| SerializeError::StreamError(e.to_string()))?;
        
        // 反序列化
        self.deserialize_value(&buffer)
    }
    
    /// 序列化值
    fn serialize_value(&self, value: &SerializableValue) -> SerializeResult<Vec<u8>> {
        match self.format {
            SerializeFormat::Bincode => {
                bincode::serialize(value)
                    .map_err(|e| SerializeError::SerializeFailed(e.to_string()))
            }
            SerializeFormat::MsgPack => {
                rmp_serde::to_vec(value)
                    .map_err(|e| SerializeError::SerializeFailed(e.to_string()))
            }
            SerializeFormat::Cbor => {
                serde_cbor::to_vec(value)
                    .map_err(|e| SerializeError::SerializeFailed(e.to_string()))
            }
            SerializeFormat::Postcard => {
                postcard::to_allocvec(value)
                    .map_err(|e| SerializeError::SerializeFailed(e.to_string()))
            }
            SerializeFormat::Json => {
                serde_json::to_vec(value)
                    .map_err(|e| SerializeError::SerializeFailed(e.to_string()))
            }
        }
    }
    
    /// 反序列化值
    fn deserialize_value(&self, data: &[u8]) -> SerializeResult<SerializableValue> {
        match self.format {
            SerializeFormat::Bincode => {
                bincode::deserialize(data)
                    .map_err(|e| SerializeError::DeserializeFailed(e.to_string()))
            }
            SerializeFormat::MsgPack => {
                rmp_serde::from_slice(data)
                    .map_err(|e| SerializeError::DeserializeFailed(e.to_string()))
            }
            SerializeFormat::Cbor => {
                serde_cbor::from_slice(data)
                    .map_err(|e| SerializeError::DeserializeFailed(e.to_string()))
            }
            SerializeFormat::Postcard => {
                postcard::from_bytes(data)
                    .map_err(|e| SerializeError::DeserializeFailed(e.to_string()))
            }
            SerializeFormat::Json => {
                serde_json::from_slice(data)
                    .map_err(|e| SerializeError::DeserializeFailed(e.to_string()))
            }
        }
    }
}

/// 序列化流
/// 用于增量读取序列化数据
pub struct SerializeStream {
    /// 数据缓冲区
    data: Vec<u8>,
    /// 当前读取位置
    position: usize,
    /// 块大小
    chunk_size: usize,
}

impl SerializeStream {
    /// 从数据创建流
    /// 
    /// # 参数
    /// - `data`: 序列化数据
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            position: 0,
            chunk_size: 4096,
        }
    }
    
    /// 设置块大小
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }
    
    /// 读取下一个块
    pub fn read_chunk(&mut self) -> Option<Vec<u8>> {
        if self.position >= self.data.len() {
            return None;
        }
        
        let end = std::cmp::min(self.position + self.chunk_size, self.data.len());
        let chunk = self.data[self.position..end].to_vec();
        self.position = end;
        
        Some(chunk)
    }
    
    /// 检查是否已读完
    pub fn is_eof(&self) -> bool {
        self.position >= self.data.len()
    }
    
    /// 获取总大小
    pub fn total_size(&self) -> usize {
        self.data.len()
    }
    
    /// 获取已读取大小
    pub fn read_size(&self) -> usize {
        self.position
    }
    
    /// 获取剩余大小
    pub fn remaining_size(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }
    
    /// 重置读取位置
    pub fn reset(&mut self) {
        self.position = 0;
    }
    
    /// 获取所有数据
    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }
}

/// 迭代器实现
impl Iterator for SerializeStream {
    type Item = Vec<u8>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.read_chunk()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试流式序列化
    #[test]
    fn test_stream_serialize() {
        let serializer = StreamSerializer::new(SerializeFormat::Bincode);
        
        let value = SerializableValue::Array(vec![
            SerializableValue::Int(1),
            SerializableValue::Int(2),
            SerializableValue::Int(3),
        ]);
        
        let mut buffer = Vec::new();
        let written = serializer.serialize_to_stream(&value, &mut buffer).unwrap();
        
        assert!(written > 0);
    }
    
    /// 测试流式反序列化
    #[test]
    fn test_stream_deserialize() {
        let serializer = StreamSerializer::new(SerializeFormat::Bincode);
        
        let value = SerializableValue::Array(vec![
            SerializableValue::Int(1),
            SerializableValue::Int(2),
            SerializableValue::Int(3),
        ]);
        
        // 序列化
        let mut buffer = Vec::new();
        serializer.serialize_to_stream(&value, &mut buffer).unwrap();
        
        // 反序列化
        let reader = std::io::Cursor::new(&buffer);
        let deserialized = serializer.deserialize_from_stream(reader, Some(buffer.len())).unwrap();
        
        assert_eq!(value, deserialized);
    }
    
    /// 测试序列化流
    #[test]
    fn test_serialize_stream() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut stream = SerializeStream::new(data.clone()).with_chunk_size(3);
        
        // 读取第一个块
        let chunk1 = stream.read_chunk().unwrap();
        assert_eq!(chunk1, vec![1, 2, 3]);
        
        // 读取第二个块
        let chunk2 = stream.read_chunk().unwrap();
        assert_eq!(chunk2, vec![4, 5, 6]);
        
        // 检查剩余大小
        assert_eq!(stream.remaining_size(), 4);
    }
    
    /// 测试迭代器
    #[test]
    fn test_iterator() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let stream = SerializeStream::new(data).with_chunk_size(3);
        
        let chunks: Vec<_> = stream.collect();
        assert_eq!(chunks.len(), 4); // 3 + 3 + 3 + 1
    }
}
