//! DateTime 内置函数注册模块
//!
//! 本模块实现 PHP DateTime 相关的内置函数注册
//! 包括：date_create, date_format, date_modify, date_add, date_sub, date_diff 等
//! 以及过程式日期函数：date, time, strtotime, mktime, checkdate 等

use std::collections::HashMap;

use chrono::{Datelike, Timelike, Weekday};
use super::super::builtins::BuiltinFunction;
use crate::interpreter::value::Value;
use anyhow::Result;

// ============================================================================
// DateTime 对象处理辅助函数
// ============================================================================

/// 从 Value 中提取 DateTime 对象的内部时间戳
///
/// # 参数
/// - `value`: Value 枚举值
///
/// # 返回
/// 成功返回时间戳（秒），失败返回 None
fn extract_datetime_timestamp(value: &Value) -> Option<i64> {
    // 检查是否为对象类型
    if let Value::Object(instance) = value {
        // 检查是否为 DateTime 或 DateTimeImmutable 类
        if instance.class_name == "DateTime" || instance.class_name == "DateTimeImmutable" {
            // 从属性中提取时间戳
            if let Some(Value::Int(timestamp)) = instance.properties.get("timestamp") {
                return Some(*timestamp);
            }
            // 尝试从 ustime 属性提取（微秒时间戳）
            if let Some(Value::Int(ustime)) = instance.properties.get("ustime") {
                return Some(*ustime / 1_000_000);
            }
        }
    }
    None
}

/// 从 Value 中提取 DateTime 对象的时区
///
/// # 参数
/// - `value`: Value 枚举值
///
/// # 返回
/// 成功返回时区名称字符串，失败返回 "UTC"
fn extract_datetime_timezone(value: &Value) -> String {
    // 检查是否为对象类型
    if let Value::Object(instance) = value {
        // 检查是否为 DateTime 类
        if instance.class_name == "DateTime" || instance.class_name == "DateTimeImmutable" {
            // 从属性中提取时区
            if let Some(Value::String(tz)) = instance.properties.get("timezone") {
                return tz.clone();
            }
        }
    }
    // 默认返回 UTC
    "UTC".to_string()
}

/// 创建 DateTime 对象值
///
/// # 参数
/// - `timestamp`: Unix 时间戳（秒）
/// - `timezone`: 时区名称
/// - `immutable`: 是否为不可变对象
///
/// # 返回
/// 返回 DateTime 或 DateTimeImmutable 对象值
fn create_datetime_object(timestamp: i64, timezone: &str, immutable: bool) -> Value {
    use std::collections::HashMap;
    
    // 创建对象实例
    let mut properties: HashMap<String, Value> = HashMap::new();
    
    // 设置时间戳属性
    properties.insert("timestamp".to_string(), Value::Int(timestamp));
    
    // 设置微秒时间戳属性
    properties.insert("ustime".to_string(), Value::Int(timestamp * 1_000_000));
    
    // 设置时区属性
    properties.insert("timezone".to_string(), Value::String(timezone.to_string()));
    
    // 设置日期组件属性（从时间戳计算）
    let datetime = chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    
    properties.insert("year".to_string(), Value::Int(datetime.year() as i64));
    properties.insert("month".to_string(), Value::Int(datetime.month() as i64));
    properties.insert("day".to_string(), Value::Int(datetime.day() as i64));
    properties.insert("hour".to_string(), Value::Int(datetime.hour() as i64));
    properties.insert("minute".to_string(), Value::Int(datetime.minute() as i64));
    properties.insert("second".to_string(), Value::Int(datetime.second() as i64));
    
    // 根据是否不可变选择类名
    let class_name = if immutable {
        "DateTimeImmutable"
    } else {
        "DateTime"
    };
    
    // 返回对象值
    Value::Object(crate::interpreter::value::ObjectInstance {
        class_name: class_name.to_string(),
        properties,
    })
}

/// 创建 DateInterval 对象值
///
/// # 参数
/// - `years`: 年数
/// - `months`: 月数
/// - `days`: 天数
/// - `hours`: 小时数
/// - `minutes`: 分钟数
/// - `seconds`: 秒数
/// - `invert`: 是否为负间隔
///
/// # 返回
/// 返回 DateInterval 对象值
fn create_dateinterval_object(
    years: i64,
    months: i64,
    days: i64,
    hours: i64,
    minutes: i64,
    seconds: i64,
    invert: bool,
) -> Value {
    use std::collections::HashMap;
    
    // 创建对象实例
    let mut properties: HashMap<String, Value> = HashMap::new();
    
    // 设置间隔属性
    properties.insert("y".to_string(), Value::Int(years));
    properties.insert("m".to_string(), Value::Int(months));
    properties.insert("d".to_string(), Value::Int(days));
    properties.insert("h".to_string(), Value::Int(hours));
    properties.insert("i".to_string(), Value::Int(minutes));
    properties.insert("s".to_string(), Value::Int(seconds));
    properties.insert("invert".to_string(), Value::Bool(invert));
    properties.insert("days".to_string(), Value::Bool(false));
    
    // 返回对象值
    Value::Object(crate::interpreter::value::ObjectInstance {
        class_name: "DateInterval".to_string(),
        properties,
    })
}

// ============================================================================
// DateTime 内置函数实现
// ============================================================================

/// date_create — 创建一个新的 DateTime 对象
///
/// # PHP 签名
/// date_create(string $datetime = "now", ?DateTimeZone $timezone = null): DateTime|false
///
/// # 参数
/// - args[0]: 日期时间字符串，默认为 "now"
/// - args[1]: 可选的时区对象
///
/// # 返回
/// 成功返回 DateTime 对象，失败返回 false
pub fn builtin_date_create(args: &[Value]) -> Result<Value> {
    // 获取日期时间字符串，默认为 "now"
    let datetime_str = match args.first() {
        Some(Value::String(s)) => s.as_str(),
        Some(Value::Null) | None => "now",
        _ => "now",
    };
    
    // 获取时区，默认为 UTC
    let timezone = if args.len() > 1 {
        extract_datetime_timezone(&args[1])
    } else {
        "UTC".to_string()
    };
    
    // 解析日期时间字符串
    let timestamp = if datetime_str == "now" {
        // 当前时间
        chrono::Utc::now().timestamp()
    } else if datetime_str.starts_with('@') {
        // Unix 时间戳格式 @1234567890
        datetime_str[1..].parse::<i64>().unwrap_or(chrono::Utc::now().timestamp())
    } else {
        // 尝试解析日期字符串
        chrono::NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S")
            .map(|dt| dt.and_utc().timestamp())
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(datetime_str, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
            })
            .or_else(|_| {
                // 尝试 ISO 8601 格式
                chrono::DateTime::parse_from_rfc3339(datetime_str)
                    .map(|dt| dt.timestamp())
            })
            .unwrap_or_else(|_| chrono::Utc::now().timestamp())
    };
    
    // 创建 DateTime 对象
    Ok(create_datetime_object(timestamp, &timezone, false))
}

/// date_create_immutable — 创建一个新的 DateTimeImmutable 对象
///
/// # PHP 签名
/// date_create_immutable(string $datetime = "now", ?DateTimeZone $timezone = null): DateTimeImmutable|false
///
/// # 参数
/// - args[0]: 日期时间字符串，默认为 "now"
/// - args[1]: 可选的时区对象
///
/// # 返回
/// 成功返回 DateTimeImmutable 对象，失败返回 false
pub fn builtin_date_create_immutable(args: &[Value]) -> Result<Value> {
    // 获取日期时间字符串，默认为 "now"
    let datetime_str = match args.first() {
        Some(Value::String(s)) => s.as_str(),
        Some(Value::Null) | None => "now",
        _ => "now",
    };
    
    // 获取时区，默认为 UTC
    let timezone = if args.len() > 1 {
        extract_datetime_timezone(&args[1])
    } else {
        "UTC".to_string()
    };
    
    // 解析日期时间字符串
    let timestamp = if datetime_str == "now" {
        chrono::Utc::now().timestamp()
    } else if datetime_str.starts_with('@') {
        datetime_str[1..].parse::<i64>().unwrap_or(chrono::Utc::now().timestamp())
    } else {
        chrono::NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S")
            .map(|dt| dt.and_utc().timestamp())
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(datetime_str, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
            })
            .or_else(|_| {
                chrono::DateTime::parse_from_rfc3339(datetime_str)
                    .map(|dt| dt.timestamp())
            })
            .unwrap_or_else(|_| chrono::Utc::now().timestamp())
    };
    
    // 创建 DateTimeImmutable 对象
    Ok(create_datetime_object(timestamp, &timezone, true))
}

/// date_format — 返回格式化的日期时间字符串
///
/// # PHP 签名
/// date_format(DateTimeInterface $object, string $format): string
///
/// # 参数
/// - args[0]: DateTime 对象
/// - args[1]: 格式字符串
///
/// # 返回
/// 格式化后的日期时间字符串
pub fn builtin_date_format(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取 DateTime 对象
    let datetime = &args[0];
    
    // 获取格式字符串
    let format = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => "Y-m-d H:i:s",
    };
    
    // 提取时间戳
    let timestamp = extract_datetime_timestamp(datetime).unwrap_or(chrono::Utc::now().timestamp());
    
    // 创建 chrono DateTime
    let dt = chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    
    // 转换 PHP 格式到 chrono 格式
    // PHP 格式说明：
    // Y: 4位年份, m: 2位月份, d: 2位日期
    // H: 24小时制小时, i: 分钟, s: 秒
    let result = format
        .replace('Y', &dt.format("%Y").to_string())
        .replace('y', &dt.format("%y").to_string())
        .replace('m', &dt.format("%m").to_string())
        .replace('n', &dt.format("%-m").to_string())
        .replace('d', &dt.format("%d").to_string())
        .replace('j', &dt.format("%-d").to_string())
        .replace('H', &dt.format("%H").to_string())
        .replace('G', &dt.format("%-H").to_string())
        .replace('h', &dt.format("%I").to_string())
        .replace('g', &dt.format("%-I").to_string())
        .replace('i', &dt.format("%M").to_string())
        .replace('s', &dt.format("%S").to_string())
        .replace('a', &dt.format("%P").to_string())
        .replace('A', &dt.format("%p").to_string())
        .replace('l', &dt.format("%A").to_string())
        .replace('D', &dt.format("%a").to_string())
        .replace('w', &dt.format("%w").to_string())
        .replace('z', &dt.format("%j").to_string())
        .replace('W', &dt.format("%U").to_string())
        .replace('F', &dt.format("%B").to_string())
        .replace('M', &dt.format("%b").to_string())
        .replace('t', &format!("{}", 
            chrono::NaiveDate::from_ymd_opt(dt.year(), dt.month(), 1)
                .and_then(|d| d.with_day(28))
                .and_then(|d| d.succ_opt())
                .map(|d| d.day() - 1)
                .unwrap_or(31)
        ))
        .replace('L', &if dt.year() % 4 == 0 && (dt.year() % 100 != 0 || dt.year() % 400 == 0) { "1" } else { "0" })
        .replace('c', &dt.format("%Y-%m-%dT%H:%M:%S%:z").to_string())
        .replace('r', &dt.format("%a, %d %b %Y %H:%M:%S %z").to_string())
        .replace('U', &dt.timestamp().to_string());
    
    Ok(Value::String(result))
}

/// date_modify — 修改日期时间
///
/// # PHP 签名
/// date_modify(DateTime $object, string $modifier): DateTime|false
///
/// # 参数
/// - args[0]: DateTime 对象
/// - args[1]: 修改字符串（如 "+1 day", "-1 week"）
///
/// # 返回
/// 修改后的 DateTime 对象
pub fn builtin_date_modify(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取 DateTime 对象
    let datetime = &args[0];
    
    // 获取修改字符串
    let modifier = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 提取时间戳
    let timestamp = extract_datetime_timestamp(datetime).unwrap_or(chrono::Utc::now().timestamp());
    
    // 获取时区
    let timezone = extract_datetime_timezone(datetime);
    
    // 解析修改字符串并应用
    let new_timestamp = parse_relative_time(modifier, timestamp);
    
    // 创建新的 DateTime 对象
    Ok(create_datetime_object(new_timestamp, &timezone, false))
}

/// 解析相对时间字符串
///
/// # 参数
/// - `modifier`: 相对时间字符串（如 "+1 day", "-1 week"）
/// - `base_timestamp`: 基准时间戳
///
/// # 返回
/// 计算后的新时间戳
fn parse_relative_time(modifier: &str, base_timestamp: i64) -> i64 {
    // 创建基准时间
    let base = chrono::DateTime::from_timestamp(base_timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    
    // 解析相对时间格式
    // 支持格式: "+1 day", "-1 week", "+2 months", "next Monday", "last Friday"
    let modifier = modifier.trim().to_lowercase();
    
    // 尝试解析 "+N unit" 或 "-N unit" 格式
    if let Some(captures) = regex::Regex::new(r"^([+-]?\d+)\s+(second|minute|hour|day|week|month|year)s?$")
        .ok()
        .and_then(|re| re.captures(&modifier))
    {
        let amount: i64 = captures[1].parse().unwrap_or(0);
        let unit = &captures[2];
        
        return match unit {
            "second" => base + chrono::Duration::seconds(amount),
            "minute" => base + chrono::Duration::minutes(amount),
            "hour" => base + chrono::Duration::hours(amount),
            "day" => base + chrono::Duration::days(amount),
            "week" => base + chrono::Duration::weeks(amount),
            "month" => {
                // 月份需要特殊处理
                let months = amount as i32;
                let new_month = base.month() as i32 + months;
                let year_adjust = (new_month - 1) / 12;
                let new_month = ((new_month - 1) % 12 + 1) as u32;
                let new_year = base.year() + year_adjust;
                base.with_year(new_year)
                    .and_then(|d| d.with_month(new_month))
                    .unwrap_or(base)
            }
            "year" => {
                let new_year = base.year() + amount as i32;
                base.with_year(new_year).unwrap_or(base)
            }
            _ => base,
        }.timestamp();
    }
    
    // 尝试解析 "next/last weekday" 格式
    if let Some(captures) = regex::Regex::new(r"^(next|last)\s+(monday|tuesday|wednesday|thursday|friday|saturday|sunday)$")
        .ok()
        .and_then(|re| re.captures(&modifier))
    {
        let direction = &captures[1];
        let weekday_name = &captures[2];
        
        // 将星期名称转换为数字（0=Monday, 6=Sunday）
        let target_weekday: chrono::Weekday = match weekday_name {
            "monday" => chrono::Weekday::Mon,
            "tuesday" => chrono::Weekday::Tue,
            "wednesday" => chrono::Weekday::Wed,
            "thursday" => chrono::Weekday::Thu,
            "friday" => chrono::Weekday::Fri,
            "saturday" => chrono::Weekday::Sat,
            "sunday" => chrono::Weekday::Sun,
            _ => return base.timestamp(),
        };
        
        // 计算目标日期
        let current_weekday = base.weekday();
        let days_diff = target_weekday.num_days_from_monday() as i64 
            - current_weekday.num_days_from_monday() as i64;
        
        let days_to_add = if direction == "next" {
            if days_diff <= 0 { days_diff + 7 } else { days_diff }
        } else {
            if days_diff >= 0 { days_diff - 7 } else { days_diff }
        };
        
        return (base + chrono::Duration::days(days_to_add)).timestamp();
    }
    
    // 无法解析，返回原时间戳
    base.timestamp()
}

/// date_add — 给 DateTime 对象添加时间间隔
///
/// # PHP 签名
/// date_add(DateTime $object, DateInterval $interval): DateTime
///
/// # 参数
/// - args[0]: DateTime 对象
/// - args[1]: DateInterval 对象
///
/// # 返回
/// 修改后的 DateTime 对象
pub fn builtin_date_add(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取 DateTime 对象
    let datetime = &args[0];
    
    // 获取 DateInterval 对象
    let interval = &args[1];
    
    // 提取时间戳
    let timestamp = extract_datetime_timestamp(datetime).unwrap_or(chrono::Utc::now().timestamp());
    
    // 获取时区
    let timezone = extract_datetime_timezone(datetime);
    
    // 创建基准时间
    let mut base = chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    
    // 从 DateInterval 对象提取间隔值
    if let Value::Object(interval_instance) = interval {
        if interval_instance.class_name == "DateInterval" {
            // 提取年
            if let Some(Value::Int(years)) = interval_instance.properties.get("y") {
                let new_year = base.year() + *years as i32;
                base = base.with_year(new_year).unwrap_or(base);
            }
            // 提取月
            if let Some(Value::Int(months)) = interval_instance.properties.get("m") {
                let new_month = base.month() as i32 + *months as i32;
                let year_adjust = (new_month - 1) / 12;
                let new_month = ((new_month - 1) % 12 + 1) as u32;
                let new_year = base.year() + year_adjust;
                base = base.with_year(new_year)
                    .and_then(|d| d.with_month(new_month))
                    .unwrap_or(base);
            }
            // 提取日
            if let Some(Value::Int(days)) = interval_instance.properties.get("d") {
                base = base + chrono::Duration::days(*days);
            }
            // 提取小时
            if let Some(Value::Int(hours)) = interval_instance.properties.get("h") {
                base = base + chrono::Duration::hours(*hours);
            }
            // 提取分钟
            if let Some(Value::Int(minutes)) = interval_instance.properties.get("i") {
                base = base + chrono::Duration::minutes(*minutes);
            }
            // 提取秒
            if let Some(Value::Int(seconds)) = interval_instance.properties.get("s") {
                base = base + chrono::Duration::seconds(*seconds);
            }
        }
    }
    
    // 创建新的 DateTime 对象
    Ok(create_datetime_object(base.timestamp(), &timezone, false))
}

/// date_sub — 从 DateTime 对象减去时间间隔
///
/// # PHP 签名
/// date_sub(DateTime $object, DateInterval $interval): DateTime
///
/// # 参数
/// - args[0]: DateTime 对象
/// - args[1]: DateInterval 对象
///
/// # 返回
/// 修改后的 DateTime 对象
pub fn builtin_date_sub(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取 DateTime 对象
    let datetime = &args[0];
    
    // 获取 DateInterval 对象
    let interval = &args[1];
    
    // 提取时间戳
    let timestamp = extract_datetime_timestamp(datetime).unwrap_or(chrono::Utc::now().timestamp());
    
    // 获取时区
    let timezone = extract_datetime_timezone(datetime);
    
    // 创建基准时间
    let mut base = chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    
    // 从 DateInterval 对象提取间隔值并减去
    if let Value::Object(interval_instance) = interval {
        if interval_instance.class_name == "DateInterval" {
            // 提取年
            if let Some(Value::Int(years)) = interval_instance.properties.get("y") {
                let new_year = base.year() - *years as i32;
                base = base.with_year(new_year).unwrap_or(base);
            }
            // 提取月
            if let Some(Value::Int(months)) = interval_instance.properties.get("m") {
                let new_month = base.month() as i32 - *months as i32;
                let year_adjust = if new_month <= 0 { (new_month - 12) / 12 } else { 0 };
                let new_month = if new_month <= 0 { (new_month % 12 + 12) as u32 } else { new_month as u32 };
                let new_year = base.year() + year_adjust;
                base = base.with_year(new_year)
                    .and_then(|d| d.with_month(new_month))
                    .unwrap_or(base);
            }
            // 提取日
            if let Some(Value::Int(days)) = interval_instance.properties.get("d") {
                base = base - chrono::Duration::days(*days);
            }
            // 提取小时
            if let Some(Value::Int(hours)) = interval_instance.properties.get("h") {
                base = base - chrono::Duration::hours(*hours);
            }
            // 提取分钟
            if let Some(Value::Int(minutes)) = interval_instance.properties.get("i") {
                base = base - chrono::Duration::minutes(*minutes);
            }
            // 提取秒
            if let Some(Value::Int(seconds)) = interval_instance.properties.get("s") {
                base = base - chrono::Duration::seconds(*seconds);
            }
        }
    }
    
    // 创建新的 DateTime 对象
    Ok(create_datetime_object(base.timestamp(), &timezone, false))
}

/// date_diff — 计算两个 DateTime 对象的差值
///
/// # PHP 签名
/// date_diff(DateTimeInterface $datetime1, DateTimeInterface $datetime2, bool $absolute = false): DateInterval
///
/// # 参数
/// - args[0]: 第一个 DateTime 对象
/// - args[1]: 第二个 DateTime 对象
/// - args[2]: 是否返回绝对值
///
/// # 返回
/// DateInterval 对象
pub fn builtin_date_diff(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取两个 DateTime 对象的时间戳
    let timestamp1 = extract_datetime_timestamp(&args[0]).unwrap_or(0);
    let timestamp2 = extract_datetime_timestamp(&args[1]).unwrap_or(0);
    
    // 检查是否返回绝对值
    let absolute = args.len() > 2 && args[2].is_truthy();
    
    // 计算差值
    let diff_seconds = timestamp2 - timestamp1;
    let invert = diff_seconds < 0 && !absolute;
    let abs_diff = diff_seconds.abs();
    
    // 转换为天、小时、分钟、秒
    let days = abs_diff / 86400;
    let remaining = abs_diff % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;
    
    // 创建 DateInterval 对象
    Ok(create_dateinterval_object(0, 0, days, hours, minutes, seconds, invert))
}

/// date_timestamp_get — 获取 DateTime 对象的 Unix 时间戳
///
/// # PHP 签名
/// date_timestamp_get(DateTimeInterface $object): int
///
/// # 参数
/// - args[0]: DateTime 对象
///
/// # 返回
/// Unix 时间戳
pub fn builtin_date_timestamp_get(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 提取时间戳
    let timestamp = extract_datetime_timestamp(&args[0]).unwrap_or(0);
    
    Ok(Value::Int(timestamp))
}

/// date_timestamp_set — 设置 DateTime 对象的 Unix 时间戳
///
/// # PHP 签名
/// date_timestamp_set(DateTime $object, int $timestamp): DateTime
///
/// # 参数
/// - args[0]: DateTime 对象
/// - args[1]: Unix 时间戳
///
/// # 返回
/// 修改后的 DateTime 对象
pub fn builtin_date_timestamp_set(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取时区
    let timezone = extract_datetime_timezone(&args[0]);
    
    // 获取新时间戳
    let timestamp = match &args[1] {
        Value::Int(t) => *t,
        Value::Float(f) => *f as i64,
        Value::String(s) => s.parse().unwrap_or(0),
        _ => 0,
    };
    
    // 创建新的 DateTime 对象
    Ok(create_datetime_object(timestamp, &timezone, false))
}

/// date_timezone_get — 获取 DateTime 对象的时区
///
/// # PHP 签名
/// date_timezone_get(DateTimeInterface $object): DateTimeZone
///
/// # 参数
/// - args[0]: DateTime 对象
///
/// # 返回
/// DateTimeZone 对象
pub fn builtin_date_timezone_get(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 提取时区
    let timezone = extract_datetime_timezone(&args[0]);
    
    // 创建 DateTimeZone 对象
    let mut properties: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    properties.insert("name".to_string(), Value::String(timezone));
    
    Ok(Value::Object(crate::interpreter::value::ObjectInstance {
        class_name: "DateTimeZone".to_string(),
        properties,
    }))
}

/// date_timezone_set — 设置 DateTime 对象的时区
///
/// # PHP 签名
/// date_timezone_set(DateTime $object, DateTimeZone $timezone): DateTime
///
/// # 参数
/// - args[0]: DateTime 对象
/// - args[1]: DateTimeZone 对象
///
/// # 返回
/// 修改后的 DateTime 对象
pub fn builtin_date_timezone_set(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 提取时间戳
    let timestamp = extract_datetime_timestamp(&args[0]).unwrap_or(chrono::Utc::now().timestamp());
    
    // 提取新时区
    let timezone = extract_datetime_timezone(&args[1]);
    
    // 创建新的 DateTime 对象
    Ok(create_datetime_object(timestamp, &timezone, false))
}

/// date_get_last_errors — 获取解析过程中的警告和错误
///
/// # PHP 签名
/// date_get_last_errors(): array|false
///
/// # 返回
/// 包含警告和错误的数组
pub fn builtin_date_get_last_errors(args: &[Value]) -> Result<Value> {
    // 忽略未使用的参数
    let _ = args;
    
    // 返回空数组（简化实现）
    Ok(Value::IndexedArray(vec![]))
}

/// date_default_timezone_get — 获取默认时区
///
/// # PHP 签名
/// date_default_timezone_get(): string
///
/// # 返回
/// 默认时区名称
pub fn builtin_date_default_timezone_get(args: &[Value]) -> Result<Value> {
    // 忽略未使用的参数
    let _ = args;
    
    // 返回 UTC 作为默认时区
    Ok(Value::String("UTC".to_string()))
}

/// date_default_timezone_set — 设置默认时区
///
/// # PHP 签名
/// date_default_timezone_set(string $timezoneId): bool
///
/// # 参数
/// - args[0]: 时区标识符
///
/// # 返回
/// 成功返回 true，失败返回 false
pub fn builtin_date_default_timezone_set(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 获取时区标识符
    let _timezone = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 简化实现：总是返回 true
    // 实际实现需要存储全局时区设置
    Ok(Value::Bool(true))
}

/// date_parse — 解析日期字符串
///
/// # PHP 签名
/// date_parse(string $datetime): array
///
/// # 参数
/// - args[0]: 日期时间字符串
///
/// # 返回
/// 包含解析结果的关联数组
pub fn builtin_date_parse(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::IndexedArray(vec![]));
    }
    
    // 获取日期字符串
    let datetime = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::IndexedArray(vec![])),
    };
    
    // 尝试解析日期
    let result = chrono::NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| chrono::NaiveDate::parse_from_str(datetime, "%Y-%m-%d")
            .map(|d| d.and_hms_opt(0, 0, 0).unwrap()))
        .or_else(|_| chrono::DateTime::parse_from_rfc3339(datetime)
            .map(|dt| dt.naive_utc()));
    
    // 构建结果数组
    let mut arr: Vec<Value> = Vec::new();
    
    match result {
        Ok(dt) => {
            arr.push(Value::Int(dt.year() as i64));
            arr.push(Value::Int(dt.month() as i64));
            arr.push(Value::Int(dt.day() as i64));
            arr.push(Value::Int(dt.hour() as i64));
            arr.push(Value::Int(dt.minute() as i64));
            arr.push(Value::Int(dt.second() as i64));
        }
        Err(_) => {
            // 解析失败，返回空数组
        }
    }
    
    Ok(Value::IndexedArray(arr))
}

/// mktime — 取得一个日期的 Unix 时间戳
///
/// # PHP 签名
/// mktime(int $hour, int $minute, int $second, int $month, int $day, int $year): int|false
///
/// # 参数
/// - args[0]: 小时
/// - args[1]: 分钟
/// - args[2]: 秒
/// - args[3]: 月
/// - args[4]: 日
/// - args[5]: 年
///
/// # 返回
/// Unix 时间戳
pub fn builtin_mktime(args: &[Value]) -> Result<Value> {
    // 获取当前时间作为默认值
    let now = chrono::Local::now();
    
    // 提取参数
    let hour = args.first()
        .and_then(|v| if let Value::Int(i) = v { Some(*i as u32) } else { None })
        .unwrap_or(now.hour());
    let minute = args.get(1)
        .and_then(|v| if let Value::Int(i) = v { Some(*i as u32) } else { None })
        .unwrap_or(now.minute());
    let second = args.get(2)
        .and_then(|v| if let Value::Int(i) = v { Some(*i as u32) } else { None })
        .unwrap_or(now.second());
    let month = args.get(3)
        .and_then(|v| if let Value::Int(i) = v { Some(*i as u32) } else { None })
        .unwrap_or(now.month());
    let day = args.get(4)
        .and_then(|v| if let Value::Int(i) = v { Some(*i as u32) } else { None })
        .unwrap_or(now.day());
    let year = args.get(5)
        .and_then(|v| if let Value::Int(i) = v { Some(*i as i32) } else { None })
        .unwrap_or(now.year());
    
    // 创建日期时间
    match chrono::NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|d| d.and_hms_opt(hour, minute, second)) {
        Some(dt) => {
            // 明确指定 DateTime<Utc> 类型，避免类型推断错误
            let datetime: chrono::DateTime<chrono::Utc> = 
                chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc);
            Ok(Value::Int(datetime.timestamp()))
        }
        None => Ok(Value::Bool(false)),
    }
}

/// gmmktime — 取得 GMT 日期的 Unix 时间戳
///
/// # PHP 签名
/// gmmktime(int $hour, int $minute, int $second, int $month, int $day, int $year): int|false
///
/// # 参数
/// 同 mktime
///
/// # 返回
/// Unix 时间戳（GMT）
pub fn builtin_gmmktime(args: &[Value]) -> Result<Value> {
    // 获取当前 UTC 时间作为默认值
    let now = chrono::Utc::now();
    
    // 提取参数
    let hour = args.first()
        .and_then(|v| if let Value::Int(i) = v { Some(*i as u32) } else { None })
        .unwrap_or(now.hour());
    let minute = args.get(1)
        .and_then(|v| if let Value::Int(i) = v { Some(*i as u32) } else { None })
        .unwrap_or(now.minute());
    let second = args.get(2)
        .and_then(|v| if let Value::Int(i) = v { Some(*i as u32) } else { None })
        .unwrap_or(now.second());
    let month = args.get(3)
        .and_then(|v| if let Value::Int(i) = v { Some(*i as u32) } else { None })
        .unwrap_or(now.month());
    let day = args.get(4)
        .and_then(|v| if let Value::Int(i) = v { Some(*i as u32) } else { None })
        .unwrap_or(now.day());
    let year = args.get(5)
        .and_then(|v| if let Value::Int(i) = v { Some(*i as i32) } else { None })
        .unwrap_or(now.year());
    
    // 创建 GMT 日期时间
    // from_naive_utc_and_offset 返回 DateTime<Utc> 而不是 Option，需要分开处理
    let dt_opt = chrono::NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|d| d.and_hms_opt(hour, minute, second));
    
    match dt_opt {
        Some(naive_dt) => {
            // 创建 UTC 时间
            let datetime: chrono::DateTime<chrono::Utc> = 
                chrono::DateTime::from_naive_utc_and_offset(naive_dt, chrono::Utc);
            Ok(Value::Int(datetime.timestamp()))
        }
        None => Ok(Value::Bool(false)),
    }
}

/// checkdate — 验证格里高里日期
///
/// # PHP 签名
/// checkdate(int $month, int $day, int $year): bool
///
/// # 参数
/// - args[0]: 月 (1-12)
/// - args[1]: 日 (1-31)
/// - args[2]: 年 (1-32767)
///
/// # 返回
/// 有效返回 true，无效返回 false
pub fn builtin_checkdate(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 3 {
        return Ok(Value::Bool(false));
    }
    
    // 提取参数
    let month = match &args[0] {
        Value::Int(i) => *i as u32,
        _ => return Ok(Value::Bool(false)),
    };
    let day = match &args[1] {
        Value::Int(i) => *i as u32,
        _ => return Ok(Value::Bool(false)),
    };
    let year = match &args[2] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    
    // 验证年份范围
    if year < 1 || year > 32767 {
        return Ok(Value::Bool(false));
    }
    
    // 验证月份范围
    if month < 1 || month > 12 {
        return Ok(Value::Bool(false));
    }
    
    // 验证日期
    let valid = chrono::NaiveDate::from_ymd_opt(year, month, day).is_some();
    
    Ok(Value::Bool(valid))
}

/// strftime — 根据区域设置格式化本地时间/日期
///
/// # PHP 签名
/// strftime(string $format, ?int $timestamp = null): string|false
///
/// # 参数
/// - args[0]: 格式字符串
/// - args[1]: 可选的时间戳
///
/// # 返回
/// 格式化后的字符串
pub fn builtin_strftime(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 获取格式字符串
    let format = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取时间戳
    let timestamp = args.get(1)
        .and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None })
        .unwrap_or_else(|| chrono::Utc::now().timestamp());
    
    // 创建日期时间
    let dt = chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    
    // 简化实现：将 strftime 格式转换为类似 date 格式
    let result = format
        .replace("%Y", &dt.format("%Y").to_string())
        .replace("%y", &dt.format("%y").to_string())
        .replace("%m", &dt.format("%m").to_string())
        .replace("%d", &dt.format("%d").to_string())
        .replace("%H", &dt.format("%H").to_string())
        .replace("%M", &dt.format("%M").to_string())
        .replace("%S", &dt.format("%S").to_string())
        .replace("%A", &dt.format("%A").to_string())
        .replace("%a", &dt.format("%a").to_string())
        .replace("%B", &dt.format("%B").to_string())
        .replace("%b", &dt.format("%b").to_string());
    
    Ok(Value::String(result))
}

/// gmstrftime — 根据区域设置格式化 GMT/UTC 时间/日期
///
/// # PHP 签名
/// gmstrftime(string $format, ?int $timestamp = null): string|false
///
/// # 参数
/// - args[0]: 格式字符串
/// - args[1]: 可选的时间戳
///
/// # 返回
/// 格式化后的 GMT 字符串
pub fn builtin_gmstrftime(args: &[Value]) -> Result<Value> {
    // 与 strftime 相同，因为我们都使用 UTC
    builtin_strftime(args)
}

/// getdate — 取得日期/时间信息
///
/// # PHP 签名
/// getdate(?int $timestamp = null): array
///
/// # 参数
/// - args[0]: 可选的时间戳
///
/// # 返回
/// 包含日期信息的关联数组
pub fn builtin_getdate(args: &[Value]) -> Result<Value> {
    // 获取时间戳
    let timestamp = args.first()
        .and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None })
        .unwrap_or_else(|| chrono::Utc::now().timestamp());
    
    // 创建日期时间
    let dt = chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    
    // 构建关联数组（使用 Vec 模拟）
    let mut arr: Vec<Value> = Vec::new();
    
    // seconds - 秒 (0-59)
    arr.push(Value::Int(dt.second() as i64));
    // minutes - 分钟 (0-59)
    arr.push(Value::Int(dt.minute() as i64));
    // hours - 小时 (0-23)
    arr.push(Value::Int(dt.hour() as i64));
    // mday - 月份中的第几天 (1-31)
    arr.push(Value::Int(dt.day() as i64));
    // wday - 星期中的第几天 (0=周日, 6=周六)
    arr.push(Value::Int(dt.weekday().num_days_from_sunday() as i64));
    // mon - 月份 (1-12)
    arr.push(Value::Int(dt.month() as i64));
    // year - 年份
    arr.push(Value::Int(dt.year() as i64));
    // yday - 一年中的第几天 (0-365)
    arr.push(Value::Int(dt.ordinal() as i64 - 1));
    // weekday - 星期几的文本表示
    arr.push(Value::String(dt.weekday().to_string()));
    // month - 月份的文本表示
    arr.push(Value::String(dt.format("%B").to_string()));
    // 0 - Unix 时间戳
    arr.push(Value::Int(dt.timestamp()));
    
    Ok(Value::IndexedArray(arr))
}

/// gettimeofday — 取得当前时间
///
/// # PHP 签名
/// gettimeofday(bool $as_float = false): array|float
///
/// # 参数
/// - args[0]: 是否返回浮点数
///
/// # 返回
/// 时间信息数组或浮点数时间戳
pub fn builtin_gettimeofday(args: &[Value]) -> Result<Value> {
    // 检查是否返回浮点数
    let as_float = args.first().map(|v| v.is_truthy()).unwrap_or(false);
    
    // 获取当前时间
    let now = chrono::Utc::now();
    
    if as_float {
        // 返回浮点数时间戳（带微秒）
        let micros = now.timestamp() as f64 + now.timestamp_subsec_micros() as f64 / 1_000_000.0;
        Ok(Value::Float(micros))
    } else {
        // 返回数组
        let mut arr: Vec<Value> = Vec::new();
        // sec - 秒
        arr.push(Value::Int(now.timestamp()));
        // usec - 微秒
        arr.push(Value::Int(now.timestamp_subsec_micros() as i64));
        // minuteswest - 格林威治时间的分钟偏移
        arr.push(Value::Int(0));
        // dsttime - 夏令时修正
        arr.push(Value::Int(0));
        Ok(Value::IndexedArray(arr))
    }
}

/// idate — 将本地时间/日期格式化为整数
///
/// # PHP 签名
/// idate(string $format, ?int $timestamp = null): int|false
///
/// # 参数
/// - args[0]: 格式字符
/// - args[1]: 可选的时间戳
///
/// # 返回
/// 整数值
pub fn builtin_idate(args: &[Value]) -> Result<Value> {
    // 检查参数
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    
    // 获取格式字符
    let format = match &args[0] {
        Value::String(s) if !s.is_empty() => s.chars().next().unwrap(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取时间戳
    let timestamp = args.get(1)
        .and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None })
        .unwrap_or_else(|| chrono::Utc::now().timestamp());
    
    // 创建日期时间
    let dt = chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    
    // 根据格式返回对应值
    let result = match format {
        'd' => dt.day() as i64,
        'j' => dt.day() as i64,
        'w' => dt.weekday().num_days_from_sunday() as i64,
        'z' => dt.ordinal() as i64 - 1,
        'W' => dt.iso_week().week() as i64,
        'm' => dt.month() as i64,
        'n' => dt.month() as i64,
        't' => {
            // 获取当月天数
            (chrono::NaiveDate::from_ymd_opt(dt.year(), dt.month(), 28)
                .and_then(|d| d.succ_opt())
                .map(|d| d.day() - 1)
                .unwrap_or(31)) as i64
        }
        'Y' => dt.year() as i64,
        'y' => (dt.year() % 100) as i64,
        'L' => if dt.year() % 4 == 0 && (dt.year() % 100 != 0 || dt.year() % 400 == 0) { 1 } else { 0 },
        'H' => dt.hour() as i64,
        'h' => (dt.hour() % 12) as i64,
        'i' => dt.minute() as i64,
        's' => dt.second() as i64,
        'U' => dt.timestamp(),
        _ => return Ok(Value::Bool(false)),
    };
    
    Ok(Value::Int(result))
}

/// localtime — 取得本地时间
///
/// # PHP 签名
/// localtime(?int $timestamp = null, bool $associative = false): array
///
/// # 参数
/// - args[0]: 可选的时间戳
/// - args[1]: 是否返回关联数组
///
/// # 返回
/// 时间信息数组
pub fn builtin_localtime(args: &[Value]) -> Result<Value> {
    // 获取时间戳
    let timestamp = args.first()
        .and_then(|v| if let Value::Int(i) = v { Some(*i) } else { None })
        .unwrap_or_else(|| chrono::Utc::now().timestamp());
    
    // 检查是否返回关联数组
    let _associative = args.get(1).map(|v| v.is_truthy()).unwrap_or(false);
    
    // 创建日期时间
    let dt = chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    
    // 构建数组
    let mut arr: Vec<Value> = Vec::new();
    // tm_sec - 秒 (0-59)
    arr.push(Value::Int(dt.second() as i64));
    // tm_min - 分钟 (0-59)
    arr.push(Value::Int(dt.minute() as i64));
    // tm_hour - 小时 (0-23)
    arr.push(Value::Int(dt.hour() as i64));
    // tm_mday - 月份中的第几天 (1-31)
    arr.push(Value::Int(dt.day() as i64));
    // tm_mon - 月份 (0-11)
    arr.push(Value::Int((dt.month() - 1) as i64));
    // tm_year - 年份（从 1900 开始）
    arr.push(Value::Int((dt.year() - 1900) as i64));
    // tm_wday - 星期中的第几天 (0=周日, 6=周六)
    arr.push(Value::Int(dt.weekday().num_days_from_sunday() as i64));
    // tm_yday - 一年中的第几天 (0-365)
    arr.push(Value::Int(dt.ordinal() as i64 - 1));
    // tm_isdst - 是否夏令时
    arr.push(Value::Int(0));
    
    Ok(Value::IndexedArray(arr))
}

/// date_create_from_format — 根据指定格式创建 DateTime 对象
///
/// # PHP 签名
/// date_create_from_format(string $format, string $datetime, ?DateTimeZone $timezone = null): DateTime|false
///
/// # 参数
/// - args[0]: 格式字符串
/// - args[1]: 日期时间字符串
/// - args[2]: 可选的时区
///
/// # 返回
/// DateTime 对象或 false
pub fn builtin_date_create_from_format(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 2 {
        return Ok(Value::Bool(false));
    }
    
    // 获取格式字符串
    let format = match &args[0] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取日期时间字符串
    let datetime = match &args[1] {
        Value::String(s) => s.as_str(),
        _ => return Ok(Value::Bool(false)),
    };
    
    // 获取时区
    let timezone = if args.len() > 2 {
        extract_datetime_timezone(&args[2])
    } else {
        "UTC".to_string()
    };
    
    // 将 PHP 格式转换为 chrono 格式
    let chrono_format = format
        .replace('Y', "%Y")
        .replace('y', "%y")
        .replace('m', "%m")
        .replace('d', "%d")
        .replace('H', "%H")
        .replace('i', "%M")
        .replace('s', "%S");
    
    // 尝试解析
    let result = chrono::NaiveDateTime::parse_from_str(datetime, &chrono_format)
        .or_else(|_| chrono::NaiveDate::parse_from_str(datetime, &chrono_format.replace(" %H:%M:%S", ""))
            .map(|d| d.and_hms_opt(0, 0, 0).unwrap()));
    
    match result {
        Ok(dt) => {
            let timestamp = dt.and_utc().timestamp();
            Ok(create_datetime_object(timestamp, &timezone, false))
        }
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// date_set_time — 设置 DateTime 对象的时间
///
/// # PHP 签名
/// date_time_set(DateTime $object, int $hour, int $minute, int $second = 0): DateTime
///
/// # 参数
/// - args[0]: DateTime 对象
/// - args[1]: 小时
/// - args[2]: 分钟
/// - args[3]: 秒（可选）
///
/// # 返回
/// 修改后的 DateTime 对象
pub fn builtin_date_time_set(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 3 {
        return Ok(Value::Bool(false));
    }
    
    // 提取时间戳
    let timestamp = extract_datetime_timestamp(&args[0]).unwrap_or(chrono::Utc::now().timestamp());
    
    // 获取时区
    let timezone = extract_datetime_timezone(&args[0]);
    
    // 创建日期时间
    let mut dt = chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    
    // 提取新时间
    let hour = match &args[1] {
        Value::Int(i) => *i as u32,
        _ => return Ok(Value::Bool(false)),
    };
    let minute = match &args[2] {
        Value::Int(i) => *i as u32,
        _ => return Ok(Value::Bool(false)),
    };
    let second = args.get(3)
        .and_then(|v| if let Value::Int(i) = v { Some(*i as u32) } else { None })
        .unwrap_or(0);
    
    // 设置新时间
    dt = dt.with_hour(hour).unwrap_or(dt)
        .with_minute(minute).unwrap_or(dt)
        .with_second(second).unwrap_or(dt);
    
    // 创建新的 DateTime 对象
    Ok(create_datetime_object(dt.timestamp(), &timezone, false))
}

/// date_set_date — 设置 DateTime 对象的日期
///
/// # PHP 签名
/// date_date_set(DateTime $object, int $year, int $month, int $day): DateTime
///
/// # 参数
/// - args[0]: DateTime 对象
/// - args[1]: 年
/// - args[2]: 月
/// - args[3]: 日
///
/// # 返回
/// 修改后的 DateTime 对象
pub fn builtin_date_date_set(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 4 {
        return Ok(Value::Bool(false));
    }
    
    // 提取时间戳
    let timestamp = extract_datetime_timestamp(&args[0]).unwrap_or(chrono::Utc::now().timestamp());
    
    // 获取时区
    let timezone = extract_datetime_timezone(&args[0]);
    
    // 创建日期时间
    let mut dt = chrono::DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    
    // 提取新日期
    let year = match &args[1] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    let month = match &args[2] {
        Value::Int(i) => *i as u32,
        _ => return Ok(Value::Bool(false)),
    };
    let day = match &args[3] {
        Value::Int(i) => *i as u32,
        _ => return Ok(Value::Bool(false)),
    };
    
    // 设置新日期
    dt = dt.with_year(year).unwrap_or(dt)
        .with_month(month).unwrap_or(dt)
        .with_day(day).unwrap_or(dt);
    
    // 创建新的 DateTime 对象
    Ok(create_datetime_object(dt.timestamp(), &timezone, false))
}

/// date_isodate_set — 设置 ISO 日期
///
/// # PHP 签名
/// date_isodate_set(DateTime $object, int $year, int $week, int $dayOfWeek = 1): DateTime
///
/// # 参数
/// - args[0]: DateTime 对象
/// - args[1]: 年
/// - args[2]: 周
/// - args[3]: 星期几（可选）
///
/// # 返回
/// 修改后的 DateTime 对象
pub fn builtin_date_isodate_set(args: &[Value]) -> Result<Value> {
    // 检查参数数量
    if args.len() < 3 {
        return Ok(Value::Bool(false));
    }
    
    // 获取时区
    let timezone = extract_datetime_timezone(&args[0]);
    
    // 提取参数
    let year = match &args[1] {
        Value::Int(i) => *i as i32,
        _ => return Ok(Value::Bool(false)),
    };
    let week = match &args[2] {
        Value::Int(i) => *i as u32,
        _ => return Ok(Value::Bool(false)),
    };
    let day_of_week = args.get(3)
        .and_then(|v| if let Value::Int(i) = v { Some(*i as u32) } else { None })
        .unwrap_or(1);
    
    // 计算 ISO 周日期
    // ISO 周从周一开始，第 1 周是包含第一个周四的周
    // 使用 NaiveDate::from_isoywd_opt 来创建 ISO 周日期
    // 将 day_of_week (1-7) 转换为 chrono::Weekday
    let weekday = match day_of_week {
        1 => chrono::Weekday::Mon,
        2 => chrono::Weekday::Tue,
        3 => chrono::Weekday::Wed,
        4 => chrono::Weekday::Thu,
        5 => chrono::Weekday::Fri,
        6 => chrono::Weekday::Sat,
        7 => chrono::Weekday::Sun,
        _ => chrono::Weekday::Mon, // 默认周一
    };
    
    // 使用 from_isoywd_opt 创建 ISO 周日期
    match chrono::NaiveDate::from_isoywd_opt(year, week, weekday) {
        Some(naive_date) => {
            // 设置时间为午夜
            match naive_date.and_hms_opt(0, 0, 0) {
                Some(naive_dt) => {
                    // 创建 UTC 时间
                    let datetime: chrono::DateTime<chrono::Utc> = 
                        chrono::DateTime::from_naive_utc_and_offset(naive_dt, chrono::Utc);
                    // 创建新的 DateTime 对象
                    Ok(create_datetime_object(datetime.timestamp(), &timezone, false))
                }
                None => Ok(Value::Bool(false)),
            }
        }
        None => Ok(Value::Bool(false)),
    }
}

// ============================================================================
// 函数注册
// ============================================================================

/// 注册 DateTime 相关的内置函数
///
/// 将所有 DateTime 函数注册到内置函数映射表中
///
/// # 参数
/// - `map`: 内置函数映射表的可变引用
pub fn register_datetime_functions(map: &mut HashMap<String, BuiltinFunction>) {
    // DateTime 对象创建函数
    map.insert("date_create".to_string(), builtin_date_create);
    map.insert("date_create_immutable".to_string(), builtin_date_create_immutable);
    map.insert("date_create_from_format".to_string(), builtin_date_create_from_format);
    map.insert("date_create_immutable_from_format".to_string(), builtin_date_create_from_format);
    
    // DateTime 格式化和获取函数
    map.insert("date_format".to_string(), builtin_date_format);
    map.insert("date_timestamp_get".to_string(), builtin_date_timestamp_get);
    map.insert("date_timestamp_set".to_string(), builtin_date_timestamp_set);
    map.insert("date_timezone_get".to_string(), builtin_date_timezone_get);
    map.insert("date_timezone_set".to_string(), builtin_date_timezone_set);
    map.insert("date_get_last_errors".to_string(), builtin_date_get_last_errors);
    
    // DateTime 修改函数
    map.insert("date_modify".to_string(), builtin_date_modify);
    map.insert("date_add".to_string(), builtin_date_add);
    map.insert("date_sub".to_string(), builtin_date_sub);
    map.insert("date_diff".to_string(), builtin_date_diff);
    map.insert("date_time_set".to_string(), builtin_date_time_set);
    map.insert("date_date_set".to_string(), builtin_date_date_set);
    map.insert("date_isodate_set".to_string(), builtin_date_isodate_set);
    
    // 时区函数
    map.insert("date_default_timezone_get".to_string(), builtin_date_default_timezone_get);
    map.insert("date_default_timezone_set".to_string(), builtin_date_default_timezone_set);
    
    // 解析函数
    map.insert("date_parse".to_string(), builtin_date_parse);
    map.insert("date_parse_from_format".to_string(), builtin_date_parse);
    
    // 过程式日期函数
    map.insert("mktime".to_string(), builtin_mktime);
    map.insert("gmmktime".to_string(), builtin_gmmktime);
    map.insert("checkdate".to_string(), builtin_checkdate);
    map.insert("strftime".to_string(), builtin_strftime);
    map.insert("gmstrftime".to_string(), builtin_gmstrftime);
    map.insert("getdate".to_string(), builtin_getdate);
    map.insert("gettimeofday".to_string(), builtin_gettimeofday);
    map.insert("idate".to_string(), builtin_idate);
    map.insert("localtime".to_string(), builtin_localtime);
    
    // 别名
    map.insert("getdate".to_string(), builtin_getdate);
}
