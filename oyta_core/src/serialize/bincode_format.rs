//! Bincode 格式序列化模块
//!
//! 实现 Bincode 格式的序列化和反序列化
//! Bincode 是 Rust 原生的二进制序列化格式，性能最优

use serde::{Deserialize, Serialize};

use super::engine::{SerializableValue, SerializeError, SerializeResult};

/// Bincode 序列化器
pub struct BincodeSerializer;

impl BincodeSerializer {
    /// 创建新的序列化器
    pub fn new() -> Self {
        Self
    }
    
    /// 序列化数据
    /// 
    /// # 参数
    /// - `value`: 可序列化的值
    pub fn serialize(value: &SerializableValue) -> SerializeResult<Vec<u8>> {
        // 使用 bincode 进行序列化
        bincode::serialize(value)
            .map_err(|e| SerializeError::SerializeFailed(e.to_string()))
    }
    
    /// 反序列化数据
    /// 
    /// # 参数
    /// - `data`: 序列化数据
    pub fn deserialize(data: &[u8]) -> SerializeResult<SerializableValue> {
        // 使用 bincode 进行反序列化
        bincode::deserialize(data)
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
        bincode::serialize_into(writer, value)
            .map_err(|e| SerializeError::SerializeFailed(e.to_string()))
    }
    
    /// 从读取器反序列化
    /// 
    /// # 参数
    /// - `reader`: 读取器
    pub fn deserialize_from<R: std::io::Read>(reader: &mut R) -> SerializeResult<SerializableValue> {
        bincode::deserialize_from(reader)
            .map_err(|e| SerializeError::DeserializeFailed(e.to_string()))
    }
    
    /// 计算序列化后的大小（不实际序列化）
    /// 
    /// # 参数
    /// - `value`: 可序列化的值
    pub fn size(value: &SerializableValue) -> usize {
        bincode::serialized_size(value).unwrap_or(0) as usize
    }
}

impl Default for BincodeSerializer {
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
        let serialized = BincodeSerializer::serialize(&value).unwrap();
        let deserialized = BincodeSerializer::deserialize(&serialized).unwrap();
        assert_eq!(value, deserialized);
        
        // 测试字符串
        let value = SerializableValue::String("hello".to_string());
        let serialized = BincodeSerializer::serialize(&value).unwrap();
        let deserialized = BincodeSerializer::deserialize(&serialized).unwrap();
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
        
        let serialized = BincodeSerializer::serialize(&value).unwrap();
        let deserialized = BincodeSerializer::deserialize(&serialized).unwrap();
        assert_eq!(value, deserialized);
    }
    
    /// 测试对象序列化
    #[test]
    fn test_object() {
        let value = SerializableValue::Object(vec![
            ("name".to_string(), SerializableValue::String("test".to_string())),
            ("value".to_string(), SerializableValue::Int(123)),
        ]);
        
        let serialized = BincodeSerializer::serialize(&value).unwrap();
        let deserialized = BincodeSerializer::deserialize(&serialized).unwrap();
        assert_eq!(value, deserialized);
    }
}
