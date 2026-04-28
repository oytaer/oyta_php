//! 序列化引擎核心模块
//!
//! 定义序列化接口和通用功能
//! 支持多种二进制格式的统一接口

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::interpreter::value::Value;

/// 序列化格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializeFormat {
    /// Bincode 格式（Rust 原生，最快）
    Bincode,
    /// MessagePack 格式（跨语言兼容）
    MsgPack,
    /// CBOR 格式（适合 IoT 场景）
    Cbor,
    /// Postcard 格式（嵌入式优化，体积最小）
    Postcard,
    /// JSON 格式（兼容性最好）
    Json,
}

impl SerializeFormat {
    /// 从字符串解析格式
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bincode" => Some(Self::Bincode),
            "msgpack" | "messagepack" => Some(Self::MsgPack),
            "cbor" => Some(Self::Cbor),
            "postcard" => Some(Self::Postcard),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
    
    /// 获取格式名称
    pub fn name(&self) -> &str {
        match self {
            Self::Bincode => "bincode",
            Self::MsgPack => "msgpack",
            Self::Cbor => "cbor",
            Self::Postcard => "postcard",
            Self::Json => "json",
        }
    }
    
    /// 获取格式描述
    pub fn description(&self) -> &str {
        match self {
            Self::Bincode => "Rust 原生二进制格式，序列化速度最快",
            Self::MsgPack => "跨语言兼容的二进制 JSON 格式",
            Self::Cbor => "RFC 8949 标准的二进制格式，适合 IoT 场景",
            Self::Postcard => "嵌入式优化的格式，体积最小",
            Self::Json => "标准 JSON 格式，兼容性最好",
        }
    }
}

/// 序列化结果
pub type SerializeResult<T> = Result<T, SerializeError>;

/// 序列化错误
#[derive(Debug, Clone)]
pub enum SerializeError {
    /// 序列化失败
    SerializeFailed(String),
    /// 反序列化失败
    DeserializeFailed(String),
    /// 不支持的格式
    UnsupportedFormat(String),
    /// 数据类型不支持
    UnsupportedType(String),
    /// 缓冲区溢出
    BufferOverflow,
    /// 流式处理错误
    StreamError(String),
    /// 无效数据
    InvalidData(String),
}

impl std::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerializeFailed(msg) => write!(f, "序列化失败: {}", msg),
            Self::DeserializeFailed(msg) => write!(f, "反序列化失败: {}", msg),
            Self::UnsupportedFormat(fmt) => write!(f, "不支持的格式: {}", fmt),
            Self::UnsupportedType(t) => write!(f, "不支持的数据类型: {}", t),
            Self::BufferOverflow => write!(f, "缓冲区溢出"),
            Self::StreamError(msg) => write!(f, "流式处理错误: {}", msg),
            Self::InvalidData(msg) => write!(f, "无效数据: {}", msg),
        }
    }
}

impl std::error::Error for SerializeError {}

/// 序列化性能统计
#[derive(Debug, Clone, Default)]
pub struct SerializeStats {
    /// 序列化次数
    pub serialize_count: u64,
    /// 反序列化次数
    pub deserialize_count: u64,
    /// 序列化总字节数
    pub total_bytes_serialized: u64,
    /// 反序列化总字节数
    pub total_bytes_deserialized: u64,
    /// 序列化总耗时（微秒）
    pub total_serialize_time_us: u64,
    /// 反序列化总耗时（微秒）
    pub total_deserialize_time_us: u64,
}

impl SerializeStats {
    /// 获取平均序列化速度（MB/s）
    pub fn avg_serialize_speed(&self) -> f64 {
        if self.total_serialize_time_us == 0 {
            return 0.0;
        }
        let bytes = self.total_bytes_serialized as f64;
        let seconds = self.total_serialize_time_us as f64 / 1_000_000.0;
        bytes / seconds / 1_000_000.0
    }
    
    /// 获取平均反序列化速度（MB/s）
    pub fn avg_deserialize_speed(&self) -> f64 {
        if self.total_deserialize_time_us == 0 {
            return 0.0;
        }
        let bytes = self.total_bytes_deserialized as f64;
        let seconds = self.total_deserialize_time_us as f64 / 1_000_000.0;
        bytes / seconds / 1_000_000.0
    }
}

/// 序列化引擎
/// 提供统一的序列化接口
pub struct SerializeEngine {
    /// 性能统计
    stats: SerializeStats,
}

impl SerializeEngine {
    /// 创建新的序列化引擎
    pub fn new() -> Self {
        Self {
            stats: SerializeStats::default(),
        }
    }
    
    /// 序列化数据
    /// 
    /// # 参数
    /// - `value`: 要序列化的值
    /// - `format`: 序列化格式
    pub fn serialize(&mut self, value: &Value, format: SerializeFormat) -> SerializeResult<Vec<u8>> {
        let start = Instant::now();
        
        // 根据格式选择序列化方法
        let result = match format {
            SerializeFormat::Bincode => self.serialize_bincode(value),
            SerializeFormat::MsgPack => self.serialize_msgpack(value),
            SerializeFormat::Cbor => self.serialize_cbor(value),
            SerializeFormat::Postcard => self.serialize_postcard(value),
            SerializeFormat::Json => self.serialize_json(value),
        };
        
        // 更新统计
        if let Ok(ref data) = result {
            self.stats.serialize_count += 1;
            self.stats.total_bytes_serialized += data.len() as u64;
            self.stats.total_serialize_time_us += start.elapsed().as_micros() as u64;
        }
        
        result
    }
    
    /// 反序列化数据
    /// 
    /// # 参数
    /// - `data`: 序列化数据
    /// - `format`: 序列化格式
    pub fn deserialize(&mut self, data: &[u8], format: SerializeFormat) -> SerializeResult<Value> {
        let start = Instant::now();
        
        // 根据格式选择反序列化方法
        let result = match format {
            SerializeFormat::Bincode => self.deserialize_bincode(data),
            SerializeFormat::MsgPack => self.deserialize_msgpack(data),
            SerializeFormat::Cbor => self.deserialize_cbor(data),
            SerializeFormat::Postcard => self.deserialize_postcard(data),
            SerializeFormat::Json => self.deserialize_json(data),
        };
        
        // 更新统计
        if result.is_ok() {
            self.stats.deserialize_count += 1;
            self.stats.total_bytes_deserialized += data.len() as u64;
            self.stats.total_deserialize_time_us += start.elapsed().as_micros() as u64;
        }
        
        result
    }
    
    /// Bincode 序列化
    fn serialize_bincode(&self, value: &Value) -> SerializeResult<Vec<u8>> {
        // 将 Value 转换为可序列化的中间格式
        let serializable = self.value_to_serializable(value)?;
        
        // 执行序列化
        bincode::serialize(&serializable)
            .map_err(|e| SerializeError::SerializeFailed(e.to_string()))
    }
    
    /// Bincode 反序列化
    fn deserialize_bincode(&self, data: &[u8]) -> SerializeResult<Value> {
        // 执行反序列化
        let serializable: SerializableValue = bincode::deserialize(data)
            .map_err(|e| SerializeError::DeserializeFailed(e.to_string()))?;
        
        // 转换回 Value
        self.serializable_to_value(serializable)
    }
    
    /// MessagePack 序列化
    fn serialize_msgpack(&self, value: &Value) -> SerializeResult<Vec<u8>> {
        let serializable = self.value_to_serializable(value)?;
        
        rmp_serde::to_vec(&serializable)
            .map_err(|e| SerializeError::SerializeFailed(e.to_string()))
    }
    
    /// MessagePack 反序列化
    fn deserialize_msgpack(&self, data: &[u8]) -> SerializeResult<Value> {
        let serializable: SerializableValue = rmp_serde::from_slice(data)
            .map_err(|e| SerializeError::DeserializeFailed(e.to_string()))?;
        
        self.serializable_to_value(serializable)
    }
    
    /// CBOR 序列化
    fn serialize_cbor(&self, value: &Value) -> SerializeResult<Vec<u8>> {
        let serializable = self.value_to_serializable(value)?;
        
        serde_cbor::to_vec(&serializable)
            .map_err(|e| SerializeError::SerializeFailed(e.to_string()))
    }
    
    /// CBOR 反序列化
    fn deserialize_cbor(&self, data: &[u8]) -> SerializeResult<Value> {
        let serializable: SerializableValue = serde_cbor::from_slice(data)
            .map_err(|e| SerializeError::DeserializeFailed(e.to_string()))?;
        
        self.serializable_to_value(serializable)
    }
    
    /// Postcard 序列化
    fn serialize_postcard(&self, value: &Value) -> SerializeResult<Vec<u8>> {
        let serializable = self.value_to_serializable(value)?;
        
        postcard::to_allocvec(&serializable)
            .map_err(|e| SerializeError::SerializeFailed(e.to_string()))
    }
    
    /// Postcard 反序列化
    fn deserialize_postcard(&self, data: &[u8]) -> SerializeResult<Value> {
        let serializable: SerializableValue = postcard::from_bytes(data)
            .map_err(|e| SerializeError::DeserializeFailed(e.to_string()))?;
        
        self.serializable_to_value(serializable)
    }
    
    /// JSON 序列化
    fn serialize_json(&self, value: &Value) -> SerializeResult<Vec<u8>> {
        let json = value.to_json_string();
        Ok(json.into_bytes())
    }
    
    /// JSON 反序列化
    fn deserialize_json(&self, data: &[u8]) -> SerializeResult<Value> {
        let json_str = String::from_utf8(data.to_vec())
            .map_err(|e| SerializeError::InvalidData(e.to_string()))?;
        
        let json_value: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| SerializeError::DeserializeFailed(e.to_string()))?;
        
        self.json_to_value(json_value)
    }
    
    /// 将 Value 转换为可序列化的中间格式
    fn value_to_serializable(&self, value: &Value) -> SerializeResult<SerializableValue> {
        match value {
            Value::Null => Ok(SerializableValue::Null),
            Value::Bool(b) => Ok(SerializableValue::Bool(*b)),
            Value::Int(i) => Ok(SerializableValue::Int(*i)),
            Value::Float(f) => Ok(SerializableValue::Float(*f)),
            Value::String(s) => Ok(SerializableValue::String(s.clone())),
            Value::IndexedArray(arr) => {
                // 递归转换数组元素
                let items: Result<Vec<_>, _> = arr
                    .iter()
                    .map(|v| self.value_to_serializable(v))
                    .collect();
                Ok(SerializableValue::Array(items?))
            }
            Value::AssociativeArray(map) => {
                // 递归转换关联数组
                let items: Result<Vec<_>, _> = map
                    .iter()
                    .map(|(k, v)| {
                        self.value_to_serializable(v)
                            .map(|sv| (k.clone(), sv))
                    })
                    .collect();
                Ok(SerializableValue::Object(items?))
            }
            Value::Object(obj) => {
                // 转换对象
                let mut fields = Vec::new();
                fields.push(("_class".to_string(), SerializableValue::String(obj.class_name.clone())));
                for (k, v) in &obj.properties {
                    fields.push((k.clone(), self.value_to_serializable(v)?));
                }
                Ok(SerializableValue::Object(fields))
            }
            Value::Callable(_) => {
                // 可调用对象序列化为字符串标识
                Ok(SerializableValue::String("__callable__".to_string()))
            }
            Value::Resource(r) => {
                // 资源序列化为描述字符串
                Ok(SerializableValue::String(format!("__resource__:{}", r)))
            }
            Value::Generator(_) => {
                Ok(SerializableValue::String("__generator__".to_string()))
            }
            Value::Fiber(_) => {
                Ok(SerializableValue::String("__fiber__".to_string()))
            }
        }
    }
    
    /// 将可序列化的中间格式转换回 Value
    fn serializable_to_value(&self, sv: SerializableValue) -> SerializeResult<Value> {
        match sv {
            SerializableValue::Null => Ok(Value::Null),
            SerializableValue::Bool(b) => Ok(Value::Bool(b)),
            SerializableValue::Int(i) => Ok(Value::Int(i)),
            SerializableValue::Float(f) => Ok(Value::Float(f)),
            SerializableValue::String(s) => Ok(Value::String(s)),
            SerializableValue::Array(items) => {
                // 递归转换数组元素
                let values: Result<Vec<_>, _> = items
                    .into_iter()
                    .map(|sv| self.serializable_to_value(sv))
                    .collect();
                Ok(Value::IndexedArray(values?))
            }
            SerializableValue::Object(fields) => {
                // 检查是否是对象（包含 _class 字段）
                let is_object = fields.iter().any(|(k, _)| k == "_class");
                
                if is_object {
                    // 转换为对象
                    use std::collections::HashMap as StdHashMap;
                    let mut class_name = String::new();
                    let mut properties = StdHashMap::new();
                    
                    for (k, v) in fields {
                        if k == "_class" {
                            if let SerializableValue::String(name) = v {
                                class_name = name;
                            }
                        } else {
                            properties.insert(k, self.serializable_to_value(v)?);
                        }
                    }
                    
                    Ok(Value::Object(crate::interpreter::value::ObjectInstance {
                        class_name,
                        properties,
                    }))
                } else {
                    // 转换为关联数组
                    let values: Result<Vec<_>, _> = fields
                        .into_iter()
                        .map(|(k, sv)| {
                            self.serializable_to_value(sv)
                                .map(|v| (k, v))
                        })
                        .collect();
                    Ok(Value::AssociativeArray(values?))
                }
            }
        }
    }
    
    /// 将 JSON 值转换为 Value
    fn json_to_value(&self, json: serde_json::Value) -> SerializeResult<Value> {
        match json {
            serde_json::Value::Null => Ok(Value::Null),
            serde_json::Value::Bool(b) => Ok(Value::Bool(b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Int(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::Float(f))
                } else {
                    Ok(Value::Int(0))
                }
            }
            serde_json::Value::String(s) => Ok(Value::String(s)),
            serde_json::Value::Array(arr) => {
                let values: Result<Vec<_>, _> = arr
                    .into_iter()
                    .map(|v| self.json_to_value(v))
                    .collect();
                Ok(Value::IndexedArray(values?))
            }
            serde_json::Value::Object(obj) => {
                let values: Result<Vec<_>, _> = obj
                    .into_iter()
                    .map(|(k, v)| {
                        self.json_to_value(v)
                            .map(|val| (k, val))
                    })
                    .collect();
                Ok(Value::AssociativeArray(values?))
            }
        }
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> &SerializeStats {
        &self.stats
    }
    
    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.stats = SerializeStats::default();
    }
    
    /// 性能基准测试
    /// 
    /// # 参数
    /// - `value`: 测试数据
    /// - `iterations`: 迭代次数
    pub fn benchmark(&mut self, value: &Value, iterations: u32) -> HashMap<String, BenchmarkResult> {
        let mut results = HashMap::new();
        
        // 测试各种格式
        for format in [
            SerializeFormat::Bincode,
            SerializeFormat::MsgPack,
            SerializeFormat::Cbor,
            SerializeFormat::Postcard,
            SerializeFormat::Json,
        ] {
            let result = self.benchmark_format(value, format, iterations);
            results.insert(format.name().to_string(), result);
        }
        
        results
    }
    
    /// 测试单个格式的性能
    fn benchmark_format(&mut self, value: &Value, format: SerializeFormat, iterations: u32) -> BenchmarkResult {
        let mut serialize_times = Vec::with_capacity(iterations as usize);
        let mut deserialize_times = Vec::with_capacity(iterations as usize);
        let mut data_size = 0usize;
        
        for _ in 0..iterations {
            // 序列化
            let start = Instant::now();
            let serialized = self.serialize(value, format).unwrap_or_default();
            serialize_times.push(start.elapsed().as_nanos() as f64);
            
            // 反序列化
            let start = Instant::now();
            let _ = self.deserialize(&serialized, format);
            deserialize_times.push(start.elapsed().as_nanos() as f64);
            
            data_size = serialized.len();
        }
        
        // 计算统计数据
        let avg_serialize_ns = serialize_times.iter().sum::<f64>() / iterations as f64;
        let avg_deserialize_ns = deserialize_times.iter().sum::<f64>() / iterations as f64;
        
        BenchmarkResult {
            format: format.name().to_string(),
            data_size,
            avg_serialize_ns,
            avg_deserialize_ns,
            serialize_ops_per_sec: 1_000_000_000.0 / avg_serialize_ns,
            deserialize_ops_per_sec: 1_000_000_000.0 / avg_deserialize_ns,
        }
    }
}

impl Default for SerializeEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 可序列化的中间格式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SerializableValue {
    /// null 值
    Null,
    /// 布尔值
    Bool(bool),
    /// 整数值
    Int(i64),
    /// 浮点数值
    Float(f64),
    /// 字符串值
    String(String),
    /// 数组
    Array(Vec<SerializableValue>),
    /// 对象/关联数组
    Object(Vec<(String, SerializableValue)>),
}

/// 基准测试结果
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// 格式名称
    pub format: String,
    /// 数据大小（字节）
    pub data_size: usize,
    /// 平均序列化时间（纳秒）
    pub avg_serialize_ns: f64,
    /// 平均反序列化时间（纳秒）
    pub avg_deserialize_ns: f64,
    /// 序列化吞吐量（ops/s）
    pub serialize_ops_per_sec: f64,
    /// 反序列化吞吐量（ops/s）
    pub deserialize_ops_per_sec: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试序列化格式解析
    #[test]
    fn test_format_from_str() {
        assert_eq!(SerializeFormat::from_str("bincode"), Some(SerializeFormat::Bincode));
        assert_eq!(SerializeFormat::from_str("msgpack"), Some(SerializeFormat::MsgPack));
        assert_eq!(SerializeFormat::from_str("cbor"), Some(SerializeFormat::Cbor));
        assert_eq!(SerializeFormat::from_str("postcard"), Some(SerializeFormat::Postcard));
        assert_eq!(SerializeFormat::from_str("json"), Some(SerializeFormat::Json));
        assert_eq!(SerializeFormat::from_str("unknown"), None);
    }
    
    /// 测试 JSON 序列化
    #[test]
    fn test_json_serialize() {
        let mut engine = SerializeEngine::new();
        
        let value = Value::Int(42);
        let serialized = engine.serialize(&value, SerializeFormat::Json).unwrap();
        
        assert!(!serialized.is_empty());
        
        let deserialized = engine.deserialize(&serialized, SerializeFormat::Json).unwrap();
        assert_eq!(value, deserialized);
    }
    
    /// 测试统计信息
    #[test]
    fn test_stats() {
        let mut engine = SerializeEngine::new();
        
        let value = Value::String("test".to_string());
        engine.serialize(&value, SerializeFormat::Json).unwrap();
        
        let stats = engine.get_stats();
        assert_eq!(stats.serialize_count, 1);
    }
}
