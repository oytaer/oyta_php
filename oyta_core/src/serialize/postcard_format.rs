//! Postcard 格式序列化模块
//!
//! 实现 Postcard 格式的序列化和反序列化
//! Postcard 是嵌入式优化的格式，体积最小

use super::engine::{SerializableValue, SerializeError, SerializeResult};

/// Postcard 序列化器
pub struct PostcardSerializer;

impl PostcardSerializer {
    /// 创建新的序列化器
    pub fn new() -> Self {
        Self
    }
    
    /// 序列化数据
    /// 
    /// # 参数
    /// - `value`: 可序列化的值
    pub fn serialize(value: &SerializableValue) -> SerializeResult<Vec<u8>> {
        // 使用 postcard 进行序列化
        postcard::to_allocvec(value)
            .map_err(|e| SerializeError::SerializeFailed(e.to_string()))
    }
    
    /// 反序列化数据
    /// 
    /// # 参数
    /// - `data`: 序列化数据
    pub fn deserialize(data: &[u8]) -> SerializeResult<SerializableValue> {
        // 使用 postcard 进行反序列化
        postcard::from_bytes(data)
            .map_err(|e| SerializeError::DeserializeFailed(e.to_string()))
    }
    
    /// 序列化到字节数组（需要预分配）
    /// 
    /// # 参数
    /// - `value`: 可序列化的值
    /// - `buffer`: 目标缓冲区
    pub fn serialize_into_slice(value: &SerializableValue, buffer: &mut [u8]) -> SerializeResult<usize> {
        postcard::to_slice(value, buffer)
            .map(|slice| slice.len())
            .map_err(|e| SerializeError::SerializeFailed(e.to_string()))
    }
    
    /// 计算序列化后的大小（通过实际序列化）
    /// 
    /// # 参数
    /// - `value`: 可序列化的值
    pub fn size(value: &SerializableValue) -> usize {
        // postcard 没有提供 size 函数，通过序列化来计算
        Self::serialize(value).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for PostcardSerializer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试基本类型序列化
    #[test]
    fn test_basic_types() {
        // 测试整数
        let value = SerializableValue::Int(42);
        let serialized = PostcardSerializer::serialize(&value).unwrap();
        let deserialized = PostcardSerializer::deserialize(&serialized).unwrap();
        assert_eq!(value, deserialized);
        
        // 测试字符串
        let value = SerializableValue::String("hello".to_string());
        let serialized = PostcardSerializer::serialize(&value).unwrap();
        let deserialized = PostcardSerializer::deserialize(&serialized).unwrap();
        assert_eq!(value, deserialized);
    }
    
    /// 测试数组序列化
    #[test]
    fn test_array() {
        let value = SerializableValue::Array(vec![
            SerializableValue::Int(1),
            SerializableValue::Int(2),
            SerializableValue::Int(3),
        ]);
        
        let serialized = PostcardSerializer::serialize(&value).unwrap();
        let deserialized = PostcardSerializer::deserialize(&serialized).unwrap();
        assert_eq!(value, deserialized);
    }
    
    /// 测试大小计算
    #[test]
    fn test_size() {
        let value = SerializableValue::Int(42);
        let size = PostcardSerializer::size(&value);
        let serialized = PostcardSerializer::serialize(&value).unwrap();
        
        assert_eq!(size, serialized.len());
    }
}
