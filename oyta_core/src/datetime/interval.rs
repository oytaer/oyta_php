//! DateInterval 时间间隔类实现
//!
//! 提供完整的 PHP DateInterval 功能
//! 支持：创建、格式化、日期计算等

use chrono::{Duration, Datelike, Timelike};
use serde::{Deserialize, Serialize};
use std::fmt;

/// DateInterval 值类型
///
/// 表示一个时间间隔
/// 对应 PHP 的 DateInterval 类
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateIntervalValue {
    /// 年数
    pub years: i32,
    /// 月数
    pub months: i32,
    /// 日数
    pub days: i32,
    /// 小时数
    pub hours: i32,
    /// 分钟数
    pub minutes: i32,
    /// 秒数
    pub seconds: i32,
    /// 微秒数
    pub microseconds: i32,
    /// 是否为负间隔
    /// true 表示这是一个负的时间间隔
    pub invert: bool,
    /// 总天数（当从 diff() 创建时有值）
    /// 用于精确计算两个日期之间的天数
    pub total_days: Option<i32>,
    /// 特殊天数（用于处理月份边界）
    /// 当从 diff() 创建时，表示跨月份的特殊天数
    pub special_days: Option<i32>,
    /// 是否从 diff() 创建
    /// 从 diff() 创建的间隔有精确的天数信息
    pub from_diff: bool,
}

impl DateIntervalValue {
    /// 从间隔规范字符串创建 DateInterval
    ///
    /// # 参数
    /// - `interval_spec`: 间隔规范字符串，格式为 ISO 8601 持续时间
    ///
    /// # 支持的格式
    /// - "P1Y2M3D": 1年2个月3天
    /// - "P1Y2M3DT4H5M6S": 1年2个月3天4小时5分钟6秒
    /// - "P1W": 1周
    /// - "PT30S": 30秒
    /// - "P1Y": 1年
    ///
    /// # 返回
    /// 成功返回 DateIntervalValue，失败返回错误信息
    pub fn new(interval_spec: &str) -> Result<Self, String> {
        Self::parse_interval_spec(interval_spec)
    }
    
    /// 从 chrono Duration 创建 DateInterval
    ///
    /// # 参数
    /// - `duration`: chrono 的 Duration 对象
    ///
    /// # 返回
    /// DateIntervalValue 实例
    pub fn from_duration(duration: Duration) -> Self {
        // 获取总秒数
        let total_seconds = duration.num_seconds();
        let is_negative = total_seconds < 0;
        let abs_seconds = total_seconds.abs();
        
        // 计算天、小时、分钟、秒
        let days = abs_seconds / 86400;
        let remaining_after_days = abs_seconds % 86400;
        let hours = remaining_after_days / 3600;
        let remaining_after_hours = remaining_after_days % 3600;
        let minutes = remaining_after_hours / 60;
        let seconds = remaining_after_hours % 60;
        
        // 获取微秒
        let microseconds = (duration.num_microseconds().unwrap_or(0) % 1_000_000).abs() as i32;
        
        Self {
            years: 0,
            months: 0,
            days: days as i32,
            hours: hours as i32,
            minutes: minutes as i32,
            seconds: seconds as i32,
            microseconds,
            invert: is_negative,
            total_days: Some(days as i32),
            special_days: None,
            from_diff: true,
        }
    }
    
    /// 创建一个空的间隔（0 时间）
    pub fn empty() -> Self {
        Self {
            years: 0,
            months: 0,
            days: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
            microseconds: 0,
            invert: false,
            total_days: None,
            special_days: None,
            from_diff: false,
        }
    }
    
    /// 创建一个 1 天的间隔
    pub fn one_day() -> Self {
        Self {
            years: 0,
            months: 0,
            days: 1,
            hours: 0,
            minutes: 0,
            seconds: 0,
            microseconds: 0,
            invert: false,
            total_days: Some(1),
            special_days: None,
            from_diff: false,
        }
    }
    
    /// 创建一个 1 周的间隔
    pub fn one_week() -> Self {
        Self {
            years: 0,
            months: 0,
            days: 7,
            hours: 0,
            minutes: 0,
            seconds: 0,
            microseconds: 0,
            invert: false,
            total_days: Some(7),
            special_days: None,
            from_diff: false,
        }
    }
    
    /// 创建一个 1 月的间隔
    pub fn one_month() -> Self {
        Self {
            years: 0,
            months: 1,
            days: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
            microseconds: 0,
            invert: false,
            total_days: None,
            special_days: None,
            from_diff: false,
        }
    }
    
    /// 创建一个 1 年的间隔
    pub fn one_year() -> Self {
        Self {
            years: 1,
            months: 0,
            days: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
            microseconds: 0,
            invert: false,
            total_days: None,
            special_days: None,
            from_diff: false,
        }
    }
    
    /// 格式化时间间隔
    ///
    /// # 参数
    /// - `format`: 格式字符串
    ///
    /// # 支持的格式字符
    /// - %Y: 年
    /// - %y: 年（带前导零）
    /// - %M: 月
    /// - %m: 月（带前导零）
    /// - %D: 日
    /// - %d: 日（带前导零）
    /// - %H: 小时
    /// - %h: 小时（带前导零）
    /// - %I: 分钟
    /// - %i: 分钟（带前导零）
    /// - %S: 秒
    /// - %s: 秒（带前导零）
    /// - %F: 微秒
    /// - %f: 微秒（带前导零）
    /// - %a: 总天数（从 diff() 创建时有效）
    /// - %R: 符号（+/-）
    /// - %r: 符号（负号时显示，正号时隐藏）
    /// - %%: 百分号
    ///
    /// # 返回
    /// 格式化后的字符串
    pub fn format(&self, format: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = format.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            // 检查是否是格式占位符
            if chars[i] == '%' && i + 1 < chars.len() {
                let format_char = chars[i + 1];
                
                match format_char {
                    // 年
                    'Y' => result.push_str(&self.years.to_string()),
                    'y' => result.push_str(&format!("{:02}", self.years)),
                    
                    // 月
                    'M' => result.push_str(&self.months.to_string()),
                    'm' => result.push_str(&format!("{:02}", self.months)),
                    
                    // 日
                    'D' => result.push_str(&self.days.to_string()),
                    'd' => result.push_str(&format!("{:02}", self.days)),
                    
                    // 小时
                    'H' => result.push_str(&self.hours.to_string()),
                    'h' => result.push_str(&format!("{:02}", self.hours)),
                    
                    // 分钟
                    'I' => result.push_str(&self.minutes.to_string()),
                    'i' => result.push_str(&format!("{:02}", self.minutes)),
                    
                    // 秒
                    'S' => result.push_str(&self.seconds.to_string()),
                    's' => result.push_str(&format!("{:02}", self.seconds)),
                    
                    // 微秒
                    'F' => result.push_str(&self.microseconds.to_string()),
                    'f' => result.push_str(&format!("{:06}", self.microseconds)),
                    
                    // 总天数
                    'a' => {
                        if let Some(total) = self.total_days {
                            result.push_str(&total.to_string());
                        } else {
                            result.push_str("(unknown)");
                        }
                    }
                    
                    // 符号
                    'R' => {
                        if self.invert {
                            result.push('-');
                        } else {
                            result.push('+');
                        }
                    }
                    'r' => {
                        if self.invert {
                            result.push('-');
                        }
                        // 正数时不显示符号
                    }
                    
                    // 百分号
                    '%' => result.push('%'),
                    
                    // 未知格式，原样输出
                    _ => {
                        result.push('%');
                        result.push(format_char);
                    }
                }
                
                i += 2;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        
        result
    }
    
    /// 设置为正间隔
    pub fn set_positive(&mut self) -> &mut Self {
        self.invert = false;
        self
    }
    
    /// 设置为负间隔
    pub fn set_negative(&mut self) -> &mut Self {
        self.invert = true;
        self
    }
    
    /// 设置年数
    pub fn set_years(&mut self, years: i32) -> &mut Self {
        self.years = years;
        self
    }
    
    /// 设置月数
    pub fn set_months(&mut self, months: i32) -> &mut Self {
        self.months = months;
        self
    }
    
    /// 设置日数
    pub fn set_days(&mut self, days: i32) -> &mut Self {
        self.days = days;
        self
    }
    
    /// 设置小时数
    pub fn set_hours(&mut self, hours: i32) -> &mut Self {
        self.hours = hours;
        self
    }
    
    /// 设置分钟数
    pub fn set_minutes(&mut self, minutes: i32) -> &mut Self {
        self.minutes = minutes;
        self
    }
    
    /// 设置秒数
    pub fn set_seconds(&mut self, seconds: i32) -> &mut Self {
        self.seconds = seconds;
        self
    }
    
    /// 设置微秒数
    pub fn set_microseconds(&mut self, microseconds: i32) -> &mut Self {
        self.microseconds = microseconds;
        self
    }
    
    /// 计算总秒数（近似值）
    ///
    /// 注意：由于年和月的天数不固定，此值为近似值
    /// 年按 365 天计算，月按 30 天计算
    pub fn total_seconds(&self) -> i64 {
        let total_days = self.years as i64 * 365 + self.months as i64 * 30 + self.days as i64;
        let total = total_days * 86400 
            + self.hours as i64 * 3600 
            + self.minutes as i64 * 60 
            + self.seconds as i64;
        
        if self.invert {
            -total
        } else {
            total
        }
    }
    
    /// 计算总天数（近似值）
    ///
    /// 注意：由于年和月的天数不固定，此值为近似值
    pub fn total_days(&self) -> i64 {
        let total = self.years as i64 * 365 + self.months as i64 * 30 + self.days as i64;
        
        if self.invert {
            -total
        } else {
            total
        }
    }
    
    /// 转换为 ISO 8601 持续时间字符串
    ///
    /// # 返回
    /// ISO 8601 格式的持续时间字符串，如 "P1Y2M3DT4H5M6S"
    pub fn to_iso8601_string(&self) -> String {
        let mut result = String::new();
        
        // 负号
        if self.invert {
            result.push('-');
        }
        
        result.push('P');
        
        // 日期部分
        if self.years > 0 {
            result.push_str(&format!("{}Y", self.years));
        }
        if self.months > 0 {
            result.push_str(&format!("{}M", self.months));
        }
        if self.days > 0 {
            result.push_str(&format!("{}D", self.days));
        }
        
        // 时间部分
        if self.hours > 0 || self.minutes > 0 || self.seconds > 0 || self.microseconds > 0 {
            result.push('T');
            
            if self.hours > 0 {
                result.push_str(&format!("{}H", self.hours));
            }
            if self.minutes > 0 {
                result.push_str(&format!("{}M", self.minutes));
            }
            if self.seconds > 0 || self.microseconds > 0 {
                if self.microseconds > 0 {
                    result.push_str(&format!("{}.{}S", self.seconds, self.microseconds));
                } else {
                    result.push_str(&format!("{}S", self.seconds));
                }
            }
        }
        
        // 如果没有任何部分，输出 P0D
        if result == "P" || result == "-P" {
            result.push_str("0D");
        }
        
        result
    }
    
    /// 解析 ISO 8601 间隔规范字符串
    fn parse_interval_spec(spec: &str) -> Result<Self, String> {
        let s = spec.trim();
        
        // 检查空字符串
        if s.is_empty() {
            return Err("间隔规范字符串不能为空".to_string());
        }
        
        // 检查是否以 P 开头
        if !s.starts_with('P') {
            return Err(format!("无效的间隔规范: {}", spec));
        }
        
        let mut interval = Self::empty();
        let rest = &s[1..]; // 跳过 P
        let mut time_part = false;
        let mut current_number = String::new();
        
        for ch in rest.chars() {
            match ch {
                // 数字
                '0'..='9' | '.' => {
                    current_number.push(ch);
                }
                // 时间分隔符
                'T' => {
                    time_part = true;
                }
                // 年
                'Y' => {
                    interval.years = current_number.parse().unwrap_or(0);
                    current_number.clear();
                }
                // 月
                'M' => {
                    if time_part {
                        interval.minutes = current_number.parse().unwrap_or(0);
                    } else {
                        interval.months = current_number.parse().unwrap_or(0);
                    }
                    current_number.clear();
                }
                // 周
                'W' => {
                    let weeks: i32 = current_number.parse().unwrap_or(0);
                    interval.days += weeks * 7;
                    current_number.clear();
                }
                // 日
                'D' => {
                    interval.days = current_number.parse().unwrap_or(0);
                    current_number.clear();
                }
                // 小时
                'H' => {
                    interval.hours = current_number.parse().unwrap_or(0);
                    current_number.clear();
                }
                // 秒
                'S' => {
                    // 检查是否有小数部分（微秒）
                    if current_number.contains('.') {
                        let parts: Vec<&str> = current_number.split('.').collect();
                        interval.seconds = parts[0].parse().unwrap_or(0);
                        if parts.len() > 1 {
                            // 微秒部分，补齐到 6 位
                            let micro_str = format!("{:0<6}", parts[1]);
                            interval.microseconds = micro_str[..6].parse().unwrap_or(0);
                        }
                    } else {
                        interval.seconds = current_number.parse().unwrap_or(0);
                    }
                    current_number.clear();
                }
                // 未知字符
                _ => {
                    return Err(format!("无效的间隔规范字符: {}", ch));
                }
            }
        }
        
        Ok(interval)
    }
}

impl fmt::Display for DateIntervalValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_iso8601_string())
    }
}

impl Default for DateIntervalValue {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_interval_creation() {
        // 测试从规范字符串创建
        let interval = DateIntervalValue::new("P1Y2M3D").unwrap();
        assert_eq!(interval.years, 1);
        assert_eq!(interval.months, 2);
        assert_eq!(interval.days, 3);
    }
    
    #[test]
    fn test_interval_with_time() {
        // 测试包含时间的间隔
        let interval = DateIntervalValue::new("P1Y2M3DT4H5M6S").unwrap();
        assert_eq!(interval.years, 1);
        assert_eq!(interval.hours, 4);
        assert_eq!(interval.minutes, 5);
        assert_eq!(interval.seconds, 6);
    }
    
    #[test]
    fn test_interval_weeks() {
        // 测试周
        let interval = DateIntervalValue::new("P1W").unwrap();
        assert_eq!(interval.days, 7);
    }
    
    #[test]
    fn test_interval_format() {
        // 测试格式化
        let interval = DateIntervalValue::new("P1Y2M3DT4H5M6S").unwrap();
        assert_eq!(interval.format("%Y-%M-%D %H:%I:%S"), "1-2-3 4:5:6");
    }
    
    #[test]
    fn test_interval_iso8601() {
        // 测试 ISO 8601 输出
        let interval = DateIntervalValue::new("P1Y2M3D").unwrap();
        assert_eq!(interval.to_iso8601_string(), "P1Y2M3D");
    }
    
    #[test]
    fn test_interval_from_duration() {
        // 测试从 Duration 创建
        let duration = Duration::try_days(5).unwrap()
            .checked_add(&Duration::try_hours(3).unwrap()).unwrap()
            .checked_add(&Duration::try_minutes(30).unwrap()).unwrap();
        
        let interval = DateIntervalValue::from_duration(duration);
        assert_eq!(interval.days, 5);
        assert_eq!(interval.hours, 3);
        assert_eq!(interval.minutes, 30);
    }
}
