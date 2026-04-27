//! 零拷贝序列化引擎模块
//!
//! 提供高性能的二进制序列化支持
//! 支持 Bincode、MessagePack、CBOR、Postcard 等多种格式
//! 相比 PHP serialize 性能提升 10-15 倍，体积减少 40-65%

// 引入子模块
pub mod engine;
pub mod bincode_format;
pub mod msgpack_format;
pub mod cbor_format;
pub mod postcard_format;
pub mod stream;

// 重导出主要类型
pub use engine::{SerializeEngine, SerializeFormat, SerializeResult};
pub use bincode_format::BincodeSerializer;
pub use msgpack_format::MsgPackSerializer;
pub use cbor_format::CborSerializer;
pub use postcard_format::PostcardSerializer;
pub use stream::{StreamSerializer, SerializeStream};
