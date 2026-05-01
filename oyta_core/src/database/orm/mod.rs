//! 数据库 ORM 扩展模块
//!
//! 提供丰富的 ORM 扩展功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//!
//! ## 模块结构
//!
//! - `timestamp`: 自动时间戳
//! - `soft_delete`: 软删除
//! - `json_field`: JSON 字段操作
//! - `events`: 模型事件

pub mod events;
pub mod json_field;
pub mod soft_delete;
pub mod timestamp;

// 重新导出常用类型
pub use events::{EventContext, EventManager, EventType, ModelEvents};
pub use json_field::{JsonFieldAccessor, JsonFieldOperator, JsonPath};
pub use soft_delete::{Restorable, SoftDeleteConfig, SoftDeleteManager, SoftDeleteType};
pub use timestamp::{AutoTimestampConfig, TimestampManager, TimestampType, Timezone};
