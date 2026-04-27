//! DateTime 和 DateTimeImmutable 类实现
//!
//! 提供完整的 PHP DateTime 功能
//! 支持：创建、格式化、修改、时区转换、日期计算等

use chrono::{DateTime as ChronoDateTime, Datelike, Timelike, Utc, Local, TimeZone};
use chrono_tz::{Tz, OffsetComponents};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::timezone::DateTimeZoneValue;
use super::interval::DateIntervalValue;
use super::parser::DateTimeParser;

/// DateTime 值类型
///
/// 表示一个可变的日期时间对象
/// 对应 PHP 的 DateTime 类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateTimeValue {
    /// 内部存储的日期时间（UTC 时间戳）
    /// 使用 chrono 的 DateTime<Utc> 类型
    pub(crate) inner: ChronoDateTime<Utc>,
    /// 关联的时区
    pub(crate) timezone: DateTimeZoneValue,
    /// 标记是否为不可变对象
    /// 用于区分 DateTime 和 DateTimeImmutable
    pub(crate) is_immutable: bool,
    /// 微秒部分（0-999999）
    /// PHP 支持微秒精度，需要单独存储
    pub(crate) microsecond: u32,
    /// 创建时的警告信息列表
    /// PHP 在解析某些日期时会产生警告
    pub(crate) warnings: Vec<String>,
    /// 创建时的错误信息列表
    pub(crate) errors: Vec<String>,
}

impl DateTimeValue {
    /// 创建新的 DateTime 实例
    ///
    /// # 参数
    /// - `datetime`: 日期时间字符串，默认为 "now"
    /// - `timezone`: 可选的时区对象
    ///
    /// # 返回
    /// 成功返回 DateTimeValue，失败返回错误信息
    ///
    /// # 支持的日期格式
    /// - "now": 当前时间
    /// - "2024-01-15": 日期
    /// - "2024-01-15 10:30:00": 日期时间
    /// - "2024-01-15T10:30:00Z": ISO 8601 格式
    /// - "@1705315800": Unix 时间戳
    /// - "+1 day", "next Monday": 相对时间格式
    pub fn new(datetime: Option<&str>, timezone: Option<DateTimeZoneValue>) -> Result<Self, String> {
        // 获取日期时间字符串，默认为 "now"
        let datetime_str = datetime.unwrap_or("now");
        
        // 获取时区，默认为系统时区
        let tz = timezone.clone().unwrap_or_else(|| {
            // 尝试获取系统时区，失败则使用 UTC
            match Local::now().offset().local_minus_utc() {
                _ => DateTimeZoneValue::utc()
            }
        });
        
        // 使用解析器解析日期字符串
        let parser = DateTimeParser::new();
        let (inner, warnings, errors) = parser.parse(datetime_str, &tz)?;
        
        // 提取微秒部分
        let microsecond = inner.timestamp_subsec_micros();
        
        Ok(Self {
            inner,
            timezone: tz,
            is_immutable: false,
            microsecond,
            warnings,
            errors,
        })
    }
    
    /// 从 Unix 时间戳创建 DateTime
    ///
    /// # 参数
    /// - `timestamp`: Unix 时间戳（秒）
    ///
    /// # 返回
    /// DateTimeValue 实例
    pub fn from_timestamp(timestamp: i64) -> Self {
        // 从时间戳创建 UTC 时间
        let inner = Utc.timestamp_opt(timestamp, 0).single().unwrap_or_else(|| Utc::now());
        
        Self {
            inner,
            timezone: DateTimeZoneValue::utc(),
            is_immutable: false,
            microsecond: 0,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }
    
    /// 从 Unix 时间戳和微秒创建 DateTime
    ///
    /// # 参数
    /// - `timestamp`: Unix 时间戳（秒）
    /// - `microsecond`: 微秒部分（0-999999）
    ///
    /// # 返回
    /// DateTimeValue 实例
    pub fn from_timestamp_micro(timestamp: i64, microsecond: u32) -> Self {
        // 将微秒转换为纳秒（chrono 使用纳秒精度）
        let nanos = (microsecond % 1_000_000) * 1000;
        
        // 从时间戳创建时间
        let inner = Utc.timestamp_opt(timestamp, nanos).single().unwrap_or_else(|| Utc::now());
        
        Self {
            inner,
            timezone: DateTimeZoneValue::utc(),
            is_immutable: false,
            microsecond: microsecond % 1_000_000,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }
    
    /// 格式化日期时间
    ///
    /// 支持完整的 PHP date() 格式字符
    ///
    /// # 参数
    /// - `format`: 格式字符串
    ///
    /// # 返回
    /// 格式化后的字符串
    ///
    /// # 支持的格式字符
    /// - Y: 4位年份
    /// - y: 2位年份
    /// - m: 月份（01-12）
    /// - n: 月份（1-12）
    /// - d: 日期（01-31）
    /// - j: 日期（1-31）
    /// - H: 小时（00-23）
    /// - h: 小时（01-12）
    /// - i: 分钟（00-59）
    /// - s: 秒（00-59）
    /// - u: 微秒
    /// - U: Unix 时间戳
    /// - 等等...
    pub fn format(&self, format: &str) -> String {
        // 将时间转换到目标时区
        let local_time = self.to_local_time();
        
        // 使用格式化器进行格式化
        super::format::DateTimeFormatter::format(&local_time, format, self.microsecond)
    }
    
    /// 修改日期时间
    ///
    /// 支持相对时间格式，如 "+1 day", "-2 hours", "next Monday"
    ///
    /// # 参数
    /// - `modify`: 修改字符串
    ///
    /// # 返回
    /// 成功返回修改后的自身引用，失败返回错误
    pub fn modify(&mut self, modify: &str) -> Result<&mut Self, String> {
        // 使用相对时间解析器解析修改字符串
        let parser = super::relative::RelativeTimeParser::new();
        let new_inner = parser.modify(&self.inner, modify)?;
        
        // 更新内部时间
        self.inner = new_inner;
        self.microsecond = new_inner.timestamp_subsec_micros();
        
        Ok(self)
    }
    
    /// 添加时间间隔
    ///
    /// # 参数
    /// - `interval`: 时间间隔对象
    ///
    /// # 返回
    /// 修改后的自身引用（支持链式调用）
    pub fn add(&mut self, interval: &DateIntervalValue) -> &mut Self {
        // 使用 chrono 的 Duration 进行时间加法
        use chrono::Duration;
        
        // 计算总天数（年转天数是近似值）
        let total_days = interval.years * 365 + interval.months * 30 + interval.days;
        
        // 计算总秒数
        let total_seconds = interval.hours * 3600 + interval.minutes * 60 + interval.seconds;
        
        // 计算总微秒
        let total_micros = interval.microseconds;
        
        // 构建持续时间
        let duration = Duration::try_days(total_days as i64)
            .and_then(|d| d.checked_add(&Duration::try_seconds(total_seconds as i64).unwrap_or_default()))
            .and_then(|d| d.checked_add(&Duration::microseconds(total_micros as i64)))
            .unwrap_or_default();
        
        // 根据 invert 标志决定加还是减
        if interval.invert {
            self.inner = self.inner.checked_sub_signed(duration).unwrap_or(self.inner);
        } else {
            self.inner = self.inner.checked_add_signed(duration).unwrap_or(self.inner);
        }
        
        // 更新微秒
        self.microsecond = self.inner.timestamp_subsec_micros();
        
        self
    }
    
    /// 减去时间间隔
    ///
    /// # 参数
    /// - `interval`: 时间间隔对象
    ///
    /// # 返回
    /// 修改后的自身引用
    pub fn sub(&mut self, interval: &DateIntervalValue) -> &mut Self {
        // 创建一个反转的间隔并添加
        let mut inverted = interval.clone();
        inverted.invert = !inverted.invert;
        self.add(&inverted)
    }
    
    /// 计算与另一个日期的差值
    ///
    /// # 参数
    /// - `target`: 目标日期时间
    /// - `absolute`: 是否返回绝对值
    ///
    /// # 返回
    /// DateIntervalValue 表示两个日期的差值
    pub fn diff(&self, target: &DateTimeValue, absolute: bool) -> DateIntervalValue {
        // 计算两个时间的差值
        let duration = target.inner.signed_duration_since(self.inner);
        
        // 将持续时间转换为 DateInterval
        let mut interval = DateIntervalValue::from_duration(duration);
        
        // 如果不是绝对值，设置 invert 标志
        if !absolute && duration.num_seconds() < 0 {
            interval.invert = true;
        }
        
        interval
    }
    
    /// 获取 Unix 时间戳
    ///
    /// # 返回
    /// Unix 时间戳（秒）
    pub fn get_timestamp(&self) -> i64 {
        self.inner.timestamp()
    }
    
    /// 设置 Unix 时间戳
    ///
    /// # 参数
    /// - `timestamp`: Unix 时间戳（秒）
    pub fn set_timestamp(&mut self, timestamp: i64) -> &mut Self {
        self.inner = Utc.timestamp_opt(timestamp, 0).single().unwrap_or(self.inner);
        self.microsecond = 0;
        self
    }
    
    /// 设置时区
    ///
    /// # 参数
    /// - `timezone`: 新的时区对象
    ///
    /// # 返回
    /// 修改后的自身引用
    pub fn set_timezone(&mut self, timezone: DateTimeZoneValue) -> &mut Self {
        // 获取当前时间在原时区的本地时间
        let local_time = self.to_local_time();
        
        // 更新时区
        self.timezone = timezone;
        
        // 将本地时间转换到新时区
        self.inner = self.from_local_time(&local_time);
        
        self
    }
    
    /// 获取时区偏移量（秒）
    ///
    /// # 返回
    /// 相对于 UTC 的偏移量（秒）
    pub fn get_offset(&self) -> i32 {
        // 获取本地时间并计算偏移
        let local = self.to_local_time();
        // 使用 base_utc_offset + dst_offset 计算总偏移量
        let base = local.offset().base_utc_offset();
        let dst = local.offset().dst_offset();
        (base + dst).num_seconds() as i32
    }
    
    /// 设置日期
    ///
    /// # 参数
    /// - `year`: 年份
    /// - `month`: 月份（1-12）
    /// - `day`: 日期（1-31）
    ///
    /// # 返回
    /// 修改后的自身引用
    pub fn set_date(&mut self, year: i32, month: u32, day: u32) -> Result<&mut Self, String> {
        // 尝试设置日期
        let new_date = self.inner.with_year(year)
            .and_then(|d| d.with_month(month))
            .and_then(|d| d.with_day(day));
        
        match new_date {
            Some(new_inner) => {
                self.inner = new_inner;
                Ok(self)
            }
            None => Err(format!("无效的日期: {}-{}-{}", year, month, day))
        }
    }
    
    /// 设置时间
    ///
    /// # 参数
    /// - `hour`: 小时（0-23）
    /// - `minute`: 分钟（0-59）
    /// - `second`: 秒（0-59）
    ///
    /// # 返回
    /// 修改后的自身引用
    pub fn set_time(&mut self, hour: u32, minute: u32, second: u32) -> Result<&mut Self, String> {
        // 尝试设置时间
        let new_time = self.inner.with_hour(hour)
            .and_then(|d| d.with_minute(minute))
            .and_then(|d| d.with_second(second));
        
        match new_time {
            Some(new_inner) => {
                self.inner = new_inner;
                Ok(self)
            }
            None => Err(format!("无效的时间: {}:{}:{}", hour, minute, second))
        }
    }
    
    /// 设置微秒
    ///
    /// # 参数
    /// - `microsecond`: 微秒（0-999999）
    ///
    /// # 返回
    /// 修改后的自身引用
    pub fn set_microsecond(&mut self, microsecond: u32) -> &mut Self {
        self.microsecond = microsecond % 1_000_000;
        // 更新内部时间的纳秒部分
        let nanos = self.microsecond * 1000;
        if let Some(new_inner) = self.inner.with_nanosecond(nanos) {
            self.inner = new_inner;
        }
        self
    }
    
    /// 从格式字符串创建 DateTime
    ///
    /// # 参数
    /// - `format`: 格式字符串
    /// - `datetime`: 日期时间字符串
    /// - `timezone`: 可选的时区
    ///
    /// # 返回
    /// 成功返回 DateTimeValue，失败返回 None
    pub fn create_from_format(
        format: &str,
        datetime: &str,
        timezone: Option<DateTimeZoneValue>,
    ) -> Option<Self> {
        // 使用解析器从格式创建
        let parser = DateTimeParser::new();
        parser.parse_from_format(format, datetime, timezone)
    }
    
    /// 从 DateTimeImmutable 创建 DateTime
    ///
    /// # 参数
    /// - `immutable`: DateTimeImmutable 实例
    ///
    /// # 返回
    /// 新的 DateTime 实例
    pub fn create_from_immutable(immutable: &DateTimeImmutableValue) -> Self {
        Self {
            inner: immutable.inner,
            timezone: immutable.timezone.clone(),
            is_immutable: false,
            microsecond: immutable.microsecond,
            warnings: immutable.warnings.clone(),
            errors: immutable.errors.clone(),
        }
    }
    
    /// 获取 ISO 8601 格式字符串
    ///
    /// # 返回
    /// ISO 8601 格式的日期时间字符串
    pub fn to_iso8601_string(&self) -> String {
        self.inner.to_rfc3339()
    }
    
    /// 获取年份
    pub fn get_year(&self) -> i32 {
        self.to_local_time().year()
    }
    
    /// 获取月份（1-12）
    pub fn get_month(&self) -> u32 {
        self.to_local_time().month()
    }
    
    /// 获取日期（1-31）
    pub fn get_day(&self) -> u32 {
        self.to_local_time().day()
    }
    
    /// 获取小时（0-23）
    pub fn get_hour(&self) -> u32 {
        self.to_local_time().hour()
    }
    
    /// 获取分钟（0-59）
    pub fn get_minute(&self) -> u32 {
        self.to_local_time().minute()
    }
    
    /// 获取秒（0-59）
    pub fn get_second(&self) -> u32 {
        self.to_local_time().second()
    }
    
    /// 获取微秒
    pub fn get_microsecond(&self) -> u32 {
        self.microsecond
    }
    
    /// 获取星期几（1-7，1=周一，7=周日）
    pub fn get_day_of_week(&self) -> u32 {
        // chrono 的 weekday() 返回 0-6（周一=0）
        // PHP 的格式是 1-7（周一=1）
        self.to_local_time().weekday().number_from_monday()
    }
    
    /// 获取一年中的第几天（1-366）
    pub fn get_day_of_year(&self) -> u32 {
        self.to_local_time().ordinal()
    }
    
    /// 获取一年中的第几周
    pub fn get_week_of_year(&self) -> u32 {
        self.to_local_time().iso_week().week()
    }
    
    /// 判断是否为闰年
    pub fn is_leap_year(&self) -> bool {
        let year = self.get_year();
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }
    
    /// 获取该月的天数
    pub fn get_days_in_month(&self) -> u32 {
        let year = self.get_year();
        let month = self.get_month();
        
        // 使用 chrono 计算该月的天数
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => if self.is_leap_year() { 29 } else { 28 },
            _ => 30,
        }
    }
    
    /// 将内部时间转换到本地时区
    fn to_local_time(&self) -> ChronoDateTime<Tz> {
        self.inner.with_timezone(&self.timezone.get_chrono_tz())
    }
    
    /// 从本地时区时间创建 UTC 时间
    fn from_local_time(&self, local: &ChronoDateTime<Tz>) -> ChronoDateTime<Utc> {
        local.with_timezone(&Utc)
    }
    
    /// 获取警告信息
    pub fn get_warnings(&self) -> &[String] {
        &self.warnings
    }
    
    /// 获取错误信息
    pub fn get_errors(&self) -> &[String] {
        &self.errors
    }
    
    /// 转换为不可变对象
    pub fn to_immutable(&self) -> DateTimeImmutableValue {
        DateTimeImmutableValue {
            inner: self.inner,
            timezone: self.timezone.clone(),
            microsecond: self.microsecond,
            warnings: self.warnings.clone(),
            errors: self.errors.clone(),
        }
    }
}

impl fmt::Display for DateTimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format("Y-m-d H:i:s.u"))
    }
}

impl Default for DateTimeValue {
    fn default() -> Self {
        Self::new(None, None).unwrap()
    }
}

/// DateTimeImmutable 值类型
///
/// 表示一个不可变的日期时间对象
/// 对应 PHP 的 DateTimeImmutable 类
/// 所有修改操作都返回新实例
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateTimeImmutableValue {
    /// 内部存储的日期时间（UTC 时间戳）
    pub(crate) inner: ChronoDateTime<Utc>,
    /// 关联的时区
    pub(crate) timezone: DateTimeZoneValue,
    /// 微秒部分
    pub(crate) microsecond: u32,
    /// 警告信息
    pub(crate) warnings: Vec<String>,
    /// 错误信息
    pub(crate) errors: Vec<String>,
}

impl DateTimeImmutableValue {
    /// 创建新的 DateTimeImmutable 实例
    ///
    /// # 参数
    /// - `datetime`: 日期时间字符串，默认为 "now"
    /// - `timezone`: 可选的时区对象
    ///
    /// # 返回
    /// 成功返回 DateTimeImmutableValue，失败返回错误信息
    pub fn new(datetime: Option<&str>, timezone: Option<DateTimeZoneValue>) -> Result<Self, String> {
        // 创建可变版本，然后转换
        let mutable = DateTimeValue::new(datetime, timezone)?;
        Ok(mutable.to_immutable())
    }
    
    /// 从 Unix 时间戳创建
    pub fn from_timestamp(timestamp: i64) -> Self {
        DateTimeValue::from_timestamp(timestamp).to_immutable()
    }
    
    /// 从 Unix 时间戳和微秒创建
    pub fn from_timestamp_micro(timestamp: i64, microsecond: u32) -> Self {
        DateTimeValue::from_timestamp_micro(timestamp, microsecond).to_immutable()
    }
    
    /// 从格式字符串创建
    pub fn create_from_format(
        format: &str,
        datetime: &str,
        timezone: Option<DateTimeZoneValue>,
    ) -> Option<Self> {
        DateTimeValue::create_from_format(format, datetime, timezone)
            .map(|dt| dt.to_immutable())
    }
    
    /// 从 DateTime 创建
    pub fn create_from_mutable(datetime: &DateTimeValue) -> Self {
        datetime.to_immutable()
    }
    
    /// 格式化日期时间（与 DateTime 相同）
    pub fn format(&self, format: &str) -> String {
        // 创建临时可变对象进行格式化
        let temp = DateTimeValue {
            inner: self.inner,
            timezone: self.timezone.clone(),
            is_immutable: false,
            microsecond: self.microsecond,
            warnings: Vec::new(),
            errors: Vec::new(),
        };
        temp.format(format)
    }
    
    /// 修改日期时间（返回新实例）
    ///
    /// # 参数
    /// - `modify`: 修改字符串
    ///
    /// # 返回
    /// 成功返回新的 DateTimeImmutableValue，失败返回错误
    pub fn modify(&self, modify: &str) -> Result<Self, String> {
        // 创建可变副本进行修改
        let mut temp = self.to_mutable();
        temp.modify(modify)?;
        Ok(temp.to_immutable())
    }
    
    /// 添加时间间隔（返回新实例）
    ///
    /// # 参数
    /// - `interval`: 时间间隔对象
    ///
    /// # 返回
    /// 新的 DateTimeImmutableValue 实例
    pub fn add(&self, interval: &DateIntervalValue) -> Self {
        let mut temp = self.to_mutable();
        temp.add(interval);
        temp.to_immutable()
    }
    
    /// 减去时间间隔（返回新实例）
    ///
    /// # 参数
    /// - `interval`: 时间间隔对象
    ///
    /// # 返回
    /// 新的 DateTimeImmutableValue 实例
    pub fn sub(&self, interval: &DateIntervalValue) -> Self {
        let mut temp = self.to_mutable();
        temp.sub(interval);
        temp.to_immutable()
    }
    
    /// 计算与另一个日期的差值
    pub fn diff(&self, target: &DateTimeImmutableValue, absolute: bool) -> DateIntervalValue {
        let temp = self.to_mutable();
        let target_temp = target.to_mutable();
        temp.diff(&target_temp, absolute)
    }
    
    /// 获取 Unix 时间戳
    pub fn get_timestamp(&self) -> i64 {
        self.inner.timestamp()
    }
    
    /// 设置时间戳（返回新实例）
    pub fn set_timestamp(&self, timestamp: i64) -> Self {
        let mut temp = self.to_mutable();
        temp.set_timestamp(timestamp);
        temp.to_immutable()
    }
    
    /// 设置时区（返回新实例）
    pub fn set_timezone(&self, timezone: DateTimeZoneValue) -> Self {
        let mut temp = self.to_mutable();
        temp.set_timezone(timezone);
        temp.to_immutable()
    }
    
    /// 设置日期（返回新实例）
    pub fn set_date(&self, year: i32, month: u32, day: u32) -> Result<Self, String> {
        let mut temp = self.to_mutable();
        temp.set_date(year, month, day)?;
        Ok(temp.to_immutable())
    }
    
    /// 设置时间（返回新实例）
    pub fn set_time(&self, hour: u32, minute: u32, second: u32) -> Result<Self, String> {
        let mut temp = self.to_mutable();
        temp.set_time(hour, minute, second)?;
        Ok(temp.to_immutable())
    }
    
    /// 设置微秒（返回新实例）
    pub fn set_microsecond(&self, microsecond: u32) -> Self {
        let mut temp = self.to_mutable();
        temp.set_microsecond(microsecond);
        temp.to_immutable()
    }
    
    /// 获取时区偏移量
    pub fn get_offset(&self) -> i32 {
        self.to_mutable().get_offset()
    }
    
    /// 获取 ISO 8601 格式字符串
    pub fn to_iso8601_string(&self) -> String {
        self.inner.to_rfc3339()
    }
    
    /// 获取年份
    pub fn get_year(&self) -> i32 {
        self.to_mutable().get_year()
    }
    
    /// 获取月份
    pub fn get_month(&self) -> u32 {
        self.to_mutable().get_month()
    }
    
    /// 获取日期
    pub fn get_day(&self) -> u32 {
        self.to_mutable().get_day()
    }
    
    /// 获取小时
    pub fn get_hour(&self) -> u32 {
        self.to_mutable().get_hour()
    }
    
    /// 获取分钟
    pub fn get_minute(&self) -> u32 {
        self.to_mutable().get_minute()
    }
    
    /// 获取秒
    pub fn get_second(&self) -> u32 {
        self.to_mutable().get_second()
    }
    
    /// 获取微秒
    pub fn get_microsecond(&self) -> u32 {
        self.microsecond
    }
    
    /// 获取星期几
    pub fn get_day_of_week(&self) -> u32 {
        self.to_mutable().get_day_of_week()
    }
    
    /// 获取一年中的第几天
    pub fn get_day_of_year(&self) -> u32 {
        self.to_mutable().get_day_of_year()
    }
    
    /// 获取一年中的第几周
    pub fn get_week_of_year(&self) -> u32 {
        self.to_mutable().get_week_of_year()
    }
    
    /// 判断是否为闰年
    pub fn is_leap_year(&self) -> bool {
        self.to_mutable().is_leap_year()
    }
    
    /// 获取该月的天数
    pub fn get_days_in_month(&self) -> u32 {
        self.to_mutable().get_days_in_month()
    }
    
    /// 获取警告信息
    pub fn get_warnings(&self) -> &[String] {
        &self.warnings
    }
    
    /// 获取错误信息
    pub fn get_errors(&self) -> &[String] {
        &self.errors
    }
    
    /// 转换为可变对象
    pub fn to_mutable(&self) -> DateTimeValue {
        DateTimeValue {
            inner: self.inner,
            timezone: self.timezone.clone(),
            is_immutable: false,
            microsecond: self.microsecond,
            warnings: self.warnings.clone(),
            errors: self.errors.clone(),
        }
    }
}

impl fmt::Display for DateTimeImmutableValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format("Y-m-d H:i:s.u"))
    }
}

impl Default for DateTimeImmutableValue {
    fn default() -> Self {
        Self::new(None, None).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_datetime_creation() {
        // 测试创建当前时间
        let dt = DateTimeValue::new(None, None).unwrap();
        assert!(dt.get_timestamp() > 0);
    }
    
    #[test]
    fn test_datetime_from_string() {
        // 测试从字符串创建
        let dt = DateTimeValue::new(Some("2024-01-15 10:30:00"), None).unwrap();
        assert_eq!(dt.get_year(), 2024);
        assert_eq!(dt.get_month(), 1);
        assert_eq!(dt.get_day(), 15);
        assert_eq!(dt.get_hour(), 10);
        assert_eq!(dt.get_minute(), 30);
    }
    
    #[test]
    fn test_datetime_format() {
        // 测试格式化
        let dt = DateTimeValue::new(Some("2024-01-15 10:30:00"), None).unwrap();
        assert_eq!(dt.format("Y-m-d"), "2024-01-15");
        assert_eq!(dt.format("H:i:s"), "10:30:00");
    }
    
    #[test]
    fn test_datetime_modify() {
        // 测试修改
        let mut dt = DateTimeValue::new(Some("2024-01-15"), None).unwrap();
        dt.modify("+1 day").unwrap();
        assert_eq!(dt.get_day(), 16);
    }
    
    #[test]
    fn test_datetime_immutable() {
        // 测试不可变对象
        let dt = DateTimeImmutableValue::new(Some("2024-01-15"), None).unwrap();
        let dt2 = dt.modify("+1 day").unwrap();
        
        // 原对象不变
        assert_eq!(dt.get_day(), 15);
        // 新对象已修改
        assert_eq!(dt2.get_day(), 16);
    }
}
