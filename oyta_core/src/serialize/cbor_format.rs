//! CBOR 格式序列化模块
//!
//! 实现 CBOR (Concise Binary Object Representation) 格式的序列化和反序列化
//! CBOR 是 RFC 8949 标准的二进制格式，适合 IoT 场景

use super::engine::{SerializableValue, SerializeError, SerializeResult};

/// CBOR 序列化器
pub struct CborSerializer;

impl CborSerializer {
    /// 创建新的序列化器
    pub fn new() -> Self {
        Self
    }
    
    /// 序列化数据
    /// 
    /// # 参数
    /// - `value`: 可序列化的值
    pub fn serialize(value: &SerializableValue) -> SerializeResult<Vec<u8>> {
        // 使用 serde_cbor 进行序列化
        serde_cbor::to_vec(value)
            .map_err(|e| SerializeError::SerializeFailed(e.to_string()))
    }
    
    /// 反序列化数据
    /// 
    /// # 参数
    /// - `data`: 序列化数据
    pub fn deserialize(data: &[u8]) -> SerializeResult<SerializableValue> {
        // 使用 serde_cbor 进行反序列化
        serde_cbor::from_slice(data)
            .map_err(|e| SerializeError::DeserializeFailed(e.to_string()))
    }
    
    /// 序列化到写入器
    /// 
    /// # 参数
    /// - `value`: 可序列化的值
    /// - `writer`: 写入器
    pub fn serialize_into<W: std::io::Write>(
        value: &SerializableValue,
        writer: &mut W,
    ) -> SerializeResult<()> {
        serde_cbor::to_writer(writer, value)
            .map_err(|e| SerializeError::SerializeFailed(e.to_string()))
    }
    
    /// 从读取器反序列化
    /// 
    /// # 参数
    /// - `reader`: 读取器
    pub fn deserialize_from<R: std::io::Read>(reader: &mut R) -> SerializeResult<SerializableValue> {
        serde_cbor::from_reader(reader)
            .map_err(|e| SerializeError::DeserializeFailed(e.to_string()))
    }
}

impl Default for CborSerializer {
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
        let serialized = CborSerializer::serialize(&value).unwrap();
        let deserialized = CborSerializer::deserialize(&serialized).unwrap();
        assert_eq!(value, deserialized);
        
        // 测试字符串
        let value = SerializableValue::String("hello".to_string());
        let serialized = CborSerializer::serialize(&value).unwrap();
        let deserialized = CborSerializer::deserialize(&serialized).unwrap();
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
        
        let serialized = CborSerializer::serialize(&value).unwrap();
        let deserialized = CborSerializer::deserialize(&serialized).unwrap();
        assert_eq!(value, deserialized);
    }
}
