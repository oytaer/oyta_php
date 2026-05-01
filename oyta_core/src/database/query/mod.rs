//! 数据库查询扩展模块
//!
//! 提供丰富的数据库查询构建功能
//! 支持 MySQL、PostgreSQL、SQLite 三种数据库
//!
//! ## 模块结构
//!
//! - `where_ext`: WHERE 条件扩展
//! - `aggregate`: 聚合查询
//! - `join_ext`: JOIN 查询扩展
//! - `subquery`: 子查询
//! - `order_ext`: 排序扩展
//! - `paging`: 分页查询
//! - `lock`: 锁定查询
//! - `having`: HAVING 条件
//! - `union_query`: UNION 查询
//! - `field`: 字段操作

pub mod aggregate;
pub mod field;
pub mod having;
pub mod join_ext;
pub mod lock;
pub mod order_ext;
pub mod paging;
pub mod subquery;
pub mod union_query;
pub mod where_ext;

// 重新导出常用类型
pub use aggregate::{AggregateBuilder, AggregateExecutor, AggregateType, CountBuilder, GroupedAggregateBuilder};
pub use field::{AliasBuilder, Field, FieldBuilder, FieldCaster, FieldType};
pub use having::{HavingBuilder, HavingClause, HavingType, HavingValue};
pub use join_ext::{JoinBuilder, JoinClause, JoinClauseBuilder, JoinType, RelationJoinBuilder};
pub use lock::{LockBuilder, LockType, LockWaitConfig, OptimisticLockConfig, PessimisticLockBuilder};
pub use order_ext::{FieldOrderBuilder, OrderBuilder, OrderClause, SortDirection, SortRule};
pub use paging::{CursorPaginateBuilder, CursorDirection, PaginateBuilder, PaginateConfig, PaginateResult, SimplePaginateResult};
pub use subquery::{NestedQueryBuilder, SubQueryBuilder, SubQueryFactory, SubQueryType};
pub use union_query::{UnionBuilder, UnionClause, UnionFactory, UnionType};
pub use where_ext::{DatabaseType, WhereBuilder, WhereClause, WhereType, WhereValue};
