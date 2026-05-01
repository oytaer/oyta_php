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
//! - `instance`: 模型实例（通过内置类暴露）
//! - `helpers`: 辅助函数（内部实现）
//! - `paginate`: 分页结果（通过内置类暴露）
//! - `collection`: 模型集合类（内部实现）
//! - `events`: 模型事件系统（内部实现）
//! - `scope`: 查询范围（内部实现）
//! - `accessor`: 获取器扩展（内部实现）
//! - `mutator`: 修改器扩展（内部实现）
//! - `searcher`: 搜索器扩展（内部实现）
//! - `soft_delete_ext`: 软删除扩展（内部实现）
//! - `output`: 模型输出（内部实现）
//! - `relation_ops`: 关联操作（内部实现）
//! - `relation_stats`: 关联统计（内部实现）
//! - `preload`: 关联预载入扩展（内部实现）
//! - `virtual_model`: 虚拟模型（内部实现）
//! - `entity`: 实体模型（内部实现）
//! - `field_map`: 字段映射（内部实现）
//! - `lazy_write`: 延迟写入（内部实现）
//! - `static_methods`: 静态方法（通过内置类暴露）
//! - `query_ext`: 查询扩展（内部实现）
//!
//! # 内部实现说明
//! 大部分扩展模块为内部实现，通过 Model 内置类间接暴露功能

// 允许内部实现未使用警告
#![allow(dead_code)]

// 基础模块
pub mod helpers;
pub mod instance;
pub mod paginate;
pub mod types;

// 扩展模块 - ThinkPHP 8.0 ORM 完整功能
pub mod accessor;
pub mod collection;
pub mod entity;
pub mod events;
pub mod field_map;
pub mod lazy_write;
pub mod mutator;
pub mod output;
pub mod preload;
pub mod query_ext;
pub mod relation_ops;
pub mod relation_stats;
pub mod scope;
pub mod searcher;
pub mod soft_delete_ext;
pub mod static_methods;
pub mod virtual_model;

// 重新导出主要类型 - 基础模块
pub use helpers::{apply_field_type, class_to_table, current_datetime, json_to_value, soft_delete_condition, value_to_json_string};
pub use instance::ModelInstance;
pub use paginate::PaginateResult;
pub use types::{FieldType, ModelConfig, RelationDef, RelationType, SoftDeleteDefault};

// 重新导出主要类型 - 扩展模块
pub use accessor::{GetterDefinition, GetterExt, GetterManager, PredefinedGetters, ResultProcessor};
pub use collection::ModelCollection;
pub use entity::{EntityBuilder, EntityModel, EntityProperty, PropertyDefinition};
pub use events::{EventDispatcher, EventResult, ModelEventManager, ModelEventType, ModelObserver};
pub use field_map::{FieldMapping, FieldMappingProcessor};
pub use lazy_write::{LazyWriteConfig, LazyWriteItem, LazyWriteManager, LazyWriteQueue};
pub use mutator::{DataSetter, PredefinedSetters, SetterDefinition, SetterExt, SetterManager};
pub use output::{CollectionOutput, ModelOutput, OutputConfig, OutputExt, RelationOutputConfig};
pub use preload::{LazyPreloader, PreloadBuilder, PreloadCache, PreloadConfig, PreloadType};
pub use query_ext::{ModelQueryBuilder, PaginateResult as QueryPaginateResult, WhereClause};
pub use relation_ops::{RelationBinder, RelationOperator, RelationQuery};
pub use relation_stats::{AggregateDefinition, AggregateExecutor, AggregateQueryBuilder, AggregateType};
pub use scope::{PredefinedScopes, ScopeBuilder, ScopeDefinition, ScopeManager};
pub use searcher::{PredefinedSearchers, SearcherBuilder, SearcherDefinition, SearcherManager};
pub use soft_delete_ext::{SoftDeleteExt, SoftDeleteOperator, SoftDeleteQueryBuilder, TrashedState};
pub use static_methods::StaticMethods;
pub use virtual_model::{VirtualModel, VirtualModelCollection, VirtualModelFactory, VirtualModelInstance};
