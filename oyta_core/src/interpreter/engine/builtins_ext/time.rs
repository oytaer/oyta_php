//! 时间函数模块
//!
//! 包含 time、date、microtime、strtotime 等时间函数

use anyhow::Result;

use crate::interpreter::value::Value;

/// time — 返回当前的 Unix 时间戳
pub fn builtin_time(args: &[Value]) -> Result<Value> {
    let _ = args;
    Ok(Value::Int(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    ))
}

/// date — 格式化一个本地时间/日期
pub fn builtin_date(args: &[Value]) -> Result<Value> {
    let _format = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let now = chrono::Local::now();
    Ok(Value::String(now.format("%Y-%m-%d %H:%M:%S").to_string()))
}

/// microtime — 返回当前 Unix 时间戳和微秒数
pub fn builtin_microtime(args: &[Value]) -> Result<Value> {
    let _ = args;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    Ok(Value::String(format!(
        "{:.6} {}",
        now.as_secs_f64() % 1.0,
        now.as_secs()
    )))
}

/// strtotime — 将任何英文文本的日期时间描述解析为 Unix 时间戳
/// 支持：now, +1 day, +1 week, +1 month, +1 year, tomorrow, yesterday 等
pub fn builtin_strtotime(args: &[Value]) -> Result<Value> {
    let time_str = args.first().map(|v| v.to_string_value()).unwrap_or_default();
    let now = chrono::Local::now();

    let result = match time_str.to_lowercase().as_str() {
        "now" => now.timestamp(),
        "today" => now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
        "tomorrow" => (now.date_naive() + chrono::Duration::days(1))
            .and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
        "yesterday" => (now.date_naive() - chrono::Duration::days(1))
            .and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
        s if s.starts_with('+') => {
            // 解析 "+N unit" 格式
            let s = s.trim_start_matches('+').trim();
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(n) = parts[0].parse::<i64>() {
                    let unit = parts[1].to_lowercase();
                    match unit.as_str() {
                        "second" | "seconds" => now.timestamp() + n,
                        "minute" | "minutes" => now.timestamp() + n * 60,
                        "hour" | "hours" => now.timestamp() + n * 3600,
                        "day" | "days" => now.timestamp() + n * 86400,
                        "week" | "weeks" => now.timestamp() + n * 604800,
                        "month" | "months" => now.timestamp() + n * 2592000,
                        "year" | "years" => now.timestamp() + n * 31536000,
                        _ => now.timestamp(),
                    }
                } else {
                    now.timestamp()
                }
            } else {
                now.timestamp()
            }
        }
        s if s.starts_with('-') => {
            let s = s.trim_start_matches('-').trim();
            let parts: Vec<&str> = s.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(n) = parts[0].parse::<i64>() {
                    let unit = parts[1].to_lowercase();
                    match unit.as_str() {
                        "second" | "seconds" => now.timestamp() - n,
                        "minute" | "minutes" => now.timestamp() - n * 60,
                        "hour" | "hours" => now.timestamp() - n * 3600,
                        "day" | "days" => now.timestamp() - n * 86400,
                        "week" | "weeks" => now.timestamp() - n * 604800,
                        "month" | "months" => now.timestamp() - n * 2592000,
                        "year" | "years" => now.timestamp() - n * 31536000,
                        _ => now.timestamp(),
                    }
                } else {
                    now.timestamp()
                }
            } else {
                now.timestamp()
            }
        }
        _ => {
            // 尝试解析日期字符串
            chrono::NaiveDateTime::parse_from_str(&time_str, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.and_utc().timestamp())
                .unwrap_or_else(|_| {
                    chrono::NaiveDate::parse_from_str(&time_str, "%Y-%m-%d")
                        .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp())
                        .unwrap_or(0)
                })
        }
    };

    Ok(Value::Int(result))
}
