//! SIMD 向量化加速引擎模块
//!
//! 利用 CPU 的 SIMD（单指令多数据）指令集进行向量化加速
//! 显著提升字符串操作、数值计算、JSON 解析等场景的性能
//! 支持 SSE2、SSE4.2、AVX2、AVX-512、NEON 等指令集

// 引入子模块
pub mod engine;
pub mod string_ops;
pub mod json_parse;
pub mod array_ops;
pub mod hash;

// 重导出主要类型
pub use engine::{SimdEngine, SimdCapabilities};
pub use string_ops::{SimdStringOps, FindResult};
pub use json_parse::SimdJsonParser;
pub use array_ops::SimdArrayOps;
pub use hash::SimdHash;
