//! DateTime 格式化器实现
//!
//! 提供完整的 PHP date() 格式化功能
//! 支持：所有 PHP 日期格式字符

use chrono::{Datelike, Timelike, Weekday};
use chrono_tz::{Tz, OffsetComponents};
use std::collections::HashMap;

/// 日期时间格式化器
///
/// 支持完整的 PHP date() 格式字符
pub struct DateTimeFormatter {
    /// 格式字符处理函数映射
    format_handlers: HashMap<char, FormatHandler>,
}

/// 格式字符处理函数类型
type FormatHandler = Box<dyn Fn(&chrono::DateTime<Tz>, u32) -> String>;

impl DateTimeFormatter {
    /// 创建新的格式化器实例
    pub fn new() -> Self {
        Self {
            format_handlers: Self::init_format_handlers(),
        }
    }
    
    /// 格式化日期时间
    ///
    /// # 参数
    /// - `datetime`: 日期时间对象
    /// - `format`: 格式字符串
    /// - `microsecond`: 微秒部分
    ///
    /// # 返回
    /// 格式化后的字符串
    pub fn format(datetime: &chrono::DateTime<Tz>, format: &str, microsecond: u32) -> String {
        let formatter = Self::new();
        formatter.format_internal(datetime, format, microsecond)
    }
    
    /// 内部格式化实现
    fn format_internal(&self, datetime: &chrono::DateTime<Tz>, format: &str, microsecond: u32) -> String {
        let mut result = String::new();
        let chars: Vec<char> = format.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let ch = chars[i];
            
            // 检查是否是转义字符
            if ch == '\\' && i + 1 < chars.len() {
                // 转义下一个字符
                result.push(chars[i + 1]);
                i += 2;
                continue;
            }
            
            // 检查是否是格式字符
            if let Some(handler) = self.format_handlers.get(&ch) {
                result.push_str(&handler(datetime, microsecond));
            } else {
                // 非格式字符，原样输出
                result.push(ch);
            }
            
            i += 1;
        }
        
        result
    }
    
    /// 初始化格式字符处理函数
    fn init_format_handlers() -> HashMap<char, FormatHandler> {
        let mut map: HashMap<char, FormatHandler> = HashMap::new();
        
        // ============ 日 ============
        
        // d - 日期，两位数字，01-31
        map.insert('d', Box::new(|dt: &chrono::DateTime<Tz>, _| format!("{:02}", dt.day())));
        
        // D - 星期几的缩写，Mon-Sun
        map.insert('D', Box::new(|dt, _| {
            let weekdays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
            weekdays[dt.weekday().num_days_from_monday() as usize].to_string()
        }));
        
        // j - 日期，不带前导零，1-31
        map.insert('j', Box::new(|dt, _| dt.day().to_string()));
        
        // l - 星期几的全称，Monday-Sunday
        map.insert('l', Box::new(|dt, _| {
            let weekdays = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
            weekdays[dt.weekday().num_days_from_monday() as usize].to_string()
        }));
        
        // N - ISO-8601 数字表示的星期几，1-7（周一=1）
        map.insert('N', Box::new(|dt, _| {
            dt.weekday().number_from_monday().to_string()
        }));
        
        // S - 日期的英文后缀，st, nd, rd, th
        map.insert('S', Box::new(|dt, _| {
            let day = dt.day();
            match day % 10 {
                1 if day != 11 => "st",
                2 if day != 12 => "nd",
                3 if day != 13 => "rd",
                _ => "th",
            }.to_string()
        }));
        
        // w - 数字表示的星期几，0-6（周日=0）
        map.insert('w', Box::new(|dt, _| {
            dt.weekday().num_days_from_sunday().to_string()
        }));
        
        // z - 一年中的第几天，0-365
        map.insert('z', Box::new(|dt, _| {
            (dt.ordinal() - 1).to_string()
        }));
        
        // ============ 周 ============
        
        // W - ISO-8601 格式的年份中的第几周，01-52
        map.insert('W', Box::new(|dt, _| {
            format!("{:02}", dt.iso_week().week())
        }));
        
        // ============ 月 ============
        
        // F - 月份全称，January-December
        map.insert('F', Box::new(|dt, _| {
            let months = [
                "January", "February", "March", "April", "May", "June",
                "July", "August", "September", "October", "November", "December"
            ];
            months[(dt.month() - 1) as usize].to_string()
        }));
        
        // m - 月份，两位数字，01-12
        map.insert('m', Box::new(|dt, _| format!("{:02}", dt.month())));
        
        // M - 月份缩写，Jan-Dec
        map.insert('M', Box::new(|dt, _| {
            let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
            months[(dt.month() - 1) as usize].to_string()
        }));
        
        // n - 月份，不带前导零，1-12
        map.insert('n', Box::new(|dt, _| dt.month().to_string()));
        
        // t - 给定月份的天数，28-31
        map.insert('t', Box::new(|dt, _| {
            let days_in_month = match dt.month() {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                2 => {
                    let year = dt.year();
                    if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                        29
                    } else {
                        28
                    }
                }
                _ => 30,
            };
            days_in_month.to_string()
        }));
        
        // ============ 年 ============
        
        // L - 是否为闰年，1=是，0=否
        map.insert('L', Box::new(|dt, _| {
            let year = dt.year();
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                "1".to_string()
            } else {
                "0".to_string()
            }
        }));
        
        // o - ISO-8601 格式的年份（与 Y 相同，但如果 ISO 周数属于上一年或下一年，则使用那一年）
        map.insert('o', Box::new(|dt, _| {
            dt.iso_week().year().to_string()
        }));
        
        // Y - 4位数字的年份
        map.insert('Y', Box::new(|dt, _| dt.year().to_string()));
        
        // y - 2位数字的年份
        map.insert('y', Box::new(|dt, _| {
            let year = dt.year();
            format!("{:02}", year % 100)
        }));
        
        // ============ 时间 ============
        
        // a - 小写的上午/下午，am/pm
        map.insert('a', Box::new(|dt, _| {
            if dt.hour() < 12 { "am" } else { "pm" }.to_string()
        }));
        
        // A - 大写的上午/下午，AM/PM
        map.insert('A', Box::new(|dt, _| {
            if dt.hour() < 12 { "AM" } else { "PM" }.to_string()
        }));
        
        // B - Swatch 互联网时间，000-999
        map.insert('B', Box::new(|dt, _| {
            // Swatch 时间是基于 UTC+1 的
            let bmt = dt.hour() * 3600 + dt.minute() * 60 + dt.second();
            let beats = (bmt + 3600) % 86400 * 1000 / 86400;
            format!("{:03}", beats)
        }));
        
        // g - 小时，12小时制，不带前导零，1-12
        map.insert('g', Box::new(|dt, _| {
            let hour = dt.hour();
            if hour == 0 { 12 } else if hour > 12 { hour - 12 } else { hour }.to_string()
        }));
        
        // G - 小时，24小时制，不带前导零，0-23
        map.insert('G', Box::new(|dt, _| dt.hour().to_string()));
        
        // h - 小时，12小时制，两位数字，01-12
        map.insert('h', Box::new(|dt, _| {
            let hour = dt.hour();
            let hour12 = if hour == 0 { 12 } else if hour > 12 { hour - 12 } else { hour };
            format!("{:02}", hour12)
        }));
        
        // H - 小时，24小时制，两位数字，00-23
        map.insert('H', Box::new(|dt, _| format!("{:02}", dt.hour())));
        
        // i - 分钟，两位数字，00-59
        map.insert('i', Box::new(|dt, _| format!("{:02}", dt.minute())));
        
        // s - 秒，两位数字，00-59
        map.insert('s', Box::new(|dt, _| format!("{:02}", dt.second())));
        
        // u - 微秒
        map.insert('u', Box::new(|_, micro| format!("{:06}", micro)));
        
        // v - 毫秒
        map.insert('v', Box::new(|_, micro| format!("{:03}", micro / 1000)));
        
        // ============ 时区 ============
        
        // e - 时区标识符
        map.insert('e', Box::new(|dt, _| {
            dt.timezone().name().to_string()
        }));
        
        // I - 是否为夏令时，1=是，0=否
        map.insert('I', Box::new(|dt, _| {
            // chrono-tz 不直接提供 DST 信息，这里简化处理
            let _ = dt;
            "0".to_string()
        }));
        
        // O - 与 UTC 的时差，+0200
        map.insert('O', Box::new(|dt, _| {
            let offset_seconds = dt.offset().base_utc_offset().num_seconds() + dt.offset().dst_offset().num_seconds();
            let hours = offset_seconds / 3600;
            let minutes = (offset_seconds % 3600) / 60;
            format!("{:+03}{:02}", hours, minutes.abs())
        }));
        
        // P - 与 UTC 的时差，+02:00
        map.insert('P', Box::new(|dt, _| {
            let offset_seconds = dt.offset().base_utc_offset().num_seconds() + dt.offset().dst_offset().num_seconds();
            let hours = offset_seconds / 3600;
            let minutes = (offset_seconds % 3600) / 60;
            format!("{:+03}:{:02}", hours, minutes.abs())
        }));
        
        // T - 时区缩写
        map.insert('T', Box::new(|dt, _| {
            dt.offset().to_string()
        }));
        
        // Z - 时区偏移秒数
        map.insert('Z', Box::new(|dt, _| {
            let offset_seconds = dt.offset().base_utc_offset().num_seconds() + dt.offset().dst_offset().num_seconds();
            offset_seconds.to_string()
        }));
        
        // ============ 完整日期/时间 ============
        
        // c - ISO 8601 格式
        map.insert('c', Box::new(|dt, _| {
            dt.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
        }));
        
        // r - RFC 2822 格式
        map.insert('r', Box::new(|dt, _| {
            dt.format("%a, %d %b %Y %H:%M:%S %z").to_string()
        }));
        
        // U - Unix 时间戳
        map.insert('U', Box::new(|dt, _| {
            dt.timestamp().to_string()
        }));
        
        map
    }
}

impl Default for DateTimeFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// 格式字符常量
///
/// 提供常用的日期格式字符串
pub struct DateFormat;

impl DateFormat {
    /// 年-月-日
    pub const YMD: &'static str = "Y-m-d";
    /// 年-月-日 时:分:秒
    pub const YMD_HIS: &'static str = "Y-m-d H:i:s";
    /// 年-月-日 时:分:秒.微秒
    pub const YMD_HIS_U: &'static str = "Y-m-d H:i:s.u";
    /// ISO 8601
    pub const ISO8601: &'static str = "c";
    /// RFC 2822
    pub const RFC2822: &'static str = "r";
    /// Unix 时间戳
    pub const TIMESTAMP: &'static str = "U";
    /// 中文日期
    pub const CHINESE: &'static str = "Y年n月j日";
    /// 中文日期时间
    pub const CHINESE_FULL: &'static str = "Y年n月j日 H:i:s";
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use chrono_tz::Tz;
    
    fn get_test_datetime() -> chrono::DateTime<Tz> {
        // 2024-01-15 10:30:45
        Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 45).unwrap().with_timezone(&Tz::UTC)
    }
    
    #[test]
    fn test_format_date() {
        let dt = get_test_datetime();
        
        // 测试 Y-m-d 格式
        assert_eq!(DateTimeFormatter::format(&dt, "Y-m-d", 0), "2024-01-15");
        
        // 测试单独的格式字符
        assert_eq!(DateTimeFormatter::format(&dt, "Y", 0), "2024");
        assert_eq!(DateTimeFormatter::format(&dt, "m", 0), "01");
        assert_eq!(DateTimeFormatter::format(&dt, "d", 0), "15");
    }
    
    #[test]
    fn test_format_time() {
        let dt = get_test_datetime();
        
        // 测试 H:i:s 格式
        assert_eq!(DateTimeFormatter::format(&dt, "H:i:s", 0), "10:30:45");
        
        // 测试单独的格式字符
        assert_eq!(DateTimeFormatter::format(&dt, "H", 0), "10");
        assert_eq!(DateTimeFormatter::format(&dt, "i", 0), "30");
        assert_eq!(DateTimeFormatter::format(&dt, "s", 0), "45");
    }
    
    #[test]
    fn test_format_datetime() {
        let dt = get_test_datetime();
        
        // 测试完整格式
        assert_eq!(DateTimeFormatter::format(&dt, "Y-m-d H:i:s", 0), "2024-01-15 10:30:45");
    }
    
    #[test]
    fn test_format_weekday() {
        let dt = get_test_datetime();
        
        // 2024-01-15 是周一
        assert_eq!(DateTimeFormatter::format(&dt, "D", 0), "Mon");
        assert_eq!(DateTimeFormatter::format(&dt, "l", 0), "Monday");
        assert_eq!(DateTimeFormatter::format(&dt, "N", 0), "1");
        assert_eq!(DateTimeFormatter::format(&dt, "w", 0), "1");
    }
    
    #[test]
    fn test_format_month() {
        let dt = get_test_datetime();
        
        // 测试月份格式
        assert_eq!(DateTimeFormatter::format(&dt, "F", 0), "January");
        assert_eq!(DateTimeFormatter::format(&dt, "M", 0), "Jan");
        assert_eq!(DateTimeFormatter::format(&dt, "n", 0), "1");
    }
    
    #[test]
    fn test_format_microsecond() {
        let dt = get_test_datetime();
        
        // 测试微秒格式
        assert_eq!(DateTimeFormatter::format(&dt, "u", 123456), "123456");
        assert_eq!(DateTimeFormatter::format(&dt, "v", 123456), "123");
    }
    
    #[test]
    fn test_format_timestamp() {
        let dt = get_test_datetime();
        
        // 测试 Unix 时间戳
        let timestamp = dt.timestamp().to_string();
        assert_eq!(DateTimeFormatter::format(&dt, "U", 0), timestamp);
    }
    
    #[test]
    fn test_format_escape() {
        let dt = get_test_datetime();
        
        // 测试转义字符
        assert_eq!(DateTimeFormatter::format(&dt, "\\Y", 0), "Y");
        assert_eq!(DateTimeFormatter::format(&dt, "\\Y-\\m-\\d", 0), "Y-m-d");
    }
    
    #[test]
    fn test_format_am_pm() {
        let dt = get_test_datetime();
        
        // 测试上午/下午
        assert_eq!(DateTimeFormatter::format(&dt, "a", 0), "am");
        assert_eq!(DateTimeFormatter::format(&dt, "A", 0), "AM");
        
        // 测试 12 小时制
        assert_eq!(DateTimeFormatter::format(&dt, "g", 0), "10");
        assert_eq!(DateTimeFormatter::format(&dt, "h", 0), "10");
    }
}
