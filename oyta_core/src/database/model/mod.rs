//! Model 基类模块
//!
//! 实现 ThinkPHP 8.0 风格的 ORM 模型
//! 支持 Active Record 模式、关联关系、获取器/修改器、软删除、自动时间戳等
//!
//! # 使用方式
//! 在 PHP 代码中定义模型类继承 Model：
//! ```php
//! class User extends Model {
//!     protected $table = 'users';
//!     protected $pk = 'id';
//! }
//! ```
//!
//! Rust 端通过 ModelInstance 在解释器中实现模型操作
//!
//! # 模块结构
//! - `types`: 类型定义
//! - `instance`: 模型实例
//! - `helpers`: 辅助函数
//! - `paginate`: 分页结果

pub mod helpers;
pub mod instance;
pub mod paginate;
pub mod types;

// 重新导出主要类型
pub use helpers::{apply_field_type, class_to_table, current_datetime, json_to_value, soft_delete_condition, value_to_json_string};
pub use instance::ModelInstance;
pub use paginate::PaginateResult;
pub use types::{FieldType, ModelConfig, RelationDef, RelationType, SoftDeleteDefault};
