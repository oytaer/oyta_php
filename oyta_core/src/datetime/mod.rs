//! DateTime 日期时间处理模块入口
//!
//! 提供 PHP 风格的日期时间处理功能
//! 包括：DateTime、DateTimeImmutable、DateTimeZone、DateInterval、DatePeriod 类
//!
//! # 模块结构
//! - `datetime`: DateTime 和 DateTimeImmutable 类实现
//! - `timezone`: DateTimeZone 时区类实现
//! - `interval`: DateInterval 时间间隔类实现
//! - `period`: DatePeriod 时间周期类实现
//! - `parser`: 日期字符串解析器
//! - `format`: 日期格式化器
//! - `relative`: 相对时间解析器

pub mod datetime;
pub mod timezone;
pub mod interval;
pub mod period;
pub mod parser;
pub mod format;
pub mod relative;
pub mod facade;

// 重新导出主要类型，方便外部使用
pub use datetime::{DateTimeValue, DateTimeImmutableValue};
pub use timezone::DateTimeZoneValue;
pub use interval::DateIntervalValue;
pub use period::DatePeriodValue;
pub use parser::DateTimeParser;
pub use format::DateTimeFormatter;
pub use relative::RelativeTimeParser;
pub use facade::Date;
